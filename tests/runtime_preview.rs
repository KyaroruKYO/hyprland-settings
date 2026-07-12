use std::collections::BTreeMap;
use std::fs;

use hyprland_settings::runtime_preview::{
    classify_runtime_preview_row, runtime_preview_capability_matrix,
    runtime_preview_family_capabilities, runtime_preview_matrix_summary,
    runtime_preview_row_capability, RuntimePreviewCapability, RuntimePreviewRiskClass,
};
use hyprland_settings::runtime_preview_executor::{
    apply_runtime_preview_value, build_runtime_preview_command, mark_runtime_preview_session_saved,
    parse_getoption_value, revert_runtime_preview_session, runtime_option_query,
    start_runtime_preview_session, RuntimePreviewDeadMan, RuntimePreviewDeadManVerdict,
    RuntimePreviewError, RuntimePreviewRunner, RuntimePreviewSessionState, RuntimePreviewThrottle,
};
use hyprland_settings::write_classification::SAFE_WRITABLE_ROWS;

const MATRIX_REPORT: &str = "data/reports/runtime-preview-capability-matrix.v0.55.2.json";

/// Mock runner: records every command, answers getoption reads, never touches
/// the live compositor.
struct MockRunner {
    getoption_response: String,
    calls: Vec<(String, Vec<String>)>,
}

impl MockRunner {
    fn new(getoption_response: &str) -> Self {
        Self {
            getoption_response: getoption_response.to_string(),
            calls: Vec::new(),
        }
    }
}

impl RuntimePreviewRunner for MockRunner {
    fn run(&mut self, program: &str, args: &[String]) -> Result<String, String> {
        self.calls.push((program.to_string(), args.to_vec()));
        if args.first().map(String::as_str) == Some("getoption") {
            Ok(self.getoption_response.clone())
        } else {
            Ok(String::from("ok"))
        }
    }
}

#[test]
fn every_scalar_row_and_family_is_classified() {
    let rows = runtime_preview_capability_matrix();
    assert_eq!(SAFE_WRITABLE_ROWS.len(), 341);
    assert_eq!(rows.len(), 341);
    let families = runtime_preview_family_capabilities();
    assert_eq!(families.len(), 7);

    let summary = runtime_preview_matrix_summary();
    assert_eq!(summary.scalar_rows_total, 341);
    assert_eq!(summary.scalar_rows_classified, 341);
    assert_eq!(summary.structured_families_total, 7);
    assert_eq!(summary.structured_families_classified, 7);
    let accounted = summary.live_preview_supported
        + summary.live_preview_supported_with_throttle
        + summary.dead_man_required
        + summary.requires_config_write
        + summary.requires_reload
        + summary.requires_relog
        + summary.requires_restart
        + summary.blocked_high_risk
        + summary.blocked_unsupported_grammar
        + summary.not_proven_yet;
    assert_eq!(
        accounted, 341,
        "every row must fall into exactly one bucket"
    );
    assert!(
        summary.live_preview_supported + summary.live_preview_supported_with_throttle >= 80,
        "the broad safe visual/layout categories should make a large share of rows previewable, got {}",
        summary.live_preview_supported + summary.live_preview_supported_with_throttle
    );
}

#[test]
fn classification_respects_risk_and_grammar_gates() {
    for row_capability in runtime_preview_capability_matrix() {
        // Default-previewable rows must be low-risk and never dead-man.
        if row_capability.capability.live_previewable_by_default() {
            assert!(
                matches!(
                    row_capability.risk,
                    RuntimePreviewRiskClass::LowRiskVisual | RuntimePreviewRiskClass::LowRiskLayout
                ),
                "{} is default-previewable but not low-risk",
                row_capability.row_id
            );
            assert!(!row_capability.dead_man_required);
        }
        // Dead-man rows are never enabled by default.
        if row_capability.capability == RuntimePreviewCapability::LivePreviewSupportedWithDeadMan {
            assert!(row_capability.dead_man_required);
        }
        // Blocked/unsupported rows carry no runtime strategy.
        if !row_capability.capability.live_previewable_with_dead_man() {
            assert_eq!(row_capability.runtime_command_strategy, "none");
            assert_eq!(row_capability.revert_strategy, "none");
        }
        assert!(!row_capability.reason.is_empty());
        assert!(!row_capability.evidence.is_empty());
    }

    // Known anchors.
    let gaps = runtime_preview_row_capability("appearance.gaps.inner")
        .or_else(|| {
            SAFE_WRITABLE_ROWS
                .iter()
                .find(|row| row.official_setting == "general.gaps_in")
                .map(classify_runtime_preview_row)
        })
        .expect("gaps_in row should exist");
    assert!(gaps.capability.live_previewable_by_default());

    let shader = SAFE_WRITABLE_ROWS
        .iter()
        .find(|row| row.row_id == "decoration.screen_shader")
        .map(classify_runtime_preview_row);
    if let Some(shader) = shader {
        assert_eq!(shader.capability, RuntimePreviewCapability::BlockedHighRisk);
    }
}

#[test]
fn structured_family_classifications_are_honest() {
    let families: BTreeMap<_, _> = runtime_preview_family_capabilities()
        .into_iter()
        .map(|family| (family.family_id, family))
        .collect();
    for family_id in [
        "hl.monitor",
        "hl.bind",
        "hl.animation",
        "hl.curve",
        "hl.gesture",
        "hl.device",
        "hl.permission",
    ] {
        assert!(families.contains_key(family_id), "missing {family_id}");
    }
    assert_eq!(
        families["hl.monitor"].capability,
        RuntimePreviewCapability::BlockedHighRisk
    );
    assert_eq!(
        families["hl.bind"].capability,
        RuntimePreviewCapability::BlockedHighRisk
    );
    assert_eq!(
        families["hl.device"].capability,
        RuntimePreviewCapability::BlockedHighRisk
    );
    assert_eq!(
        families["hl.permission"].capability,
        RuntimePreviewCapability::BlockedHighRisk
    );
    assert_eq!(
        families["hl.permission"].risk,
        RuntimePreviewRiskClass::HighRiskSecurity
    );
    assert_eq!(
        families["hl.animation"].capability,
        RuntimePreviewCapability::NotProvenYet
    );
    assert_eq!(
        families["hl.curve"].capability,
        RuntimePreviewCapability::NotProvenYet
    );
    assert_eq!(
        families["hl.gesture"].capability,
        RuntimePreviewCapability::BlockedStructuredFamilySemantics
    );
    for family in families.values() {
        assert!(
            !family.capability.live_previewable_with_dead_man(),
            "no structured family is live-previewable in this phase"
        );
    }
}

#[test]
fn command_construction_builds_safe_hl_config_expressions() {
    let gaps_row = SAFE_WRITABLE_ROWS
        .iter()
        .find(|row| row.official_setting == "general.gaps_in")
        .expect("gaps_in row should exist");
    let command = build_runtime_preview_command(gaps_row.row_id, "7", false)
        .expect("gaps_in preview command should build");
    assert_eq!(command.program, "hyprctl");
    assert_eq!(command.args[0], "eval");
    assert_eq!(command.args[1], "hl.config({ general = { gaps_in = 7 } })");

    // Nested option path.
    let nested = SAFE_WRITABLE_ROWS
        .iter()
        .find(|row| {
            row.official_setting.matches('.').count() >= 2
                && classify_runtime_preview_row(row)
                    .capability
                    .live_previewable_by_default()
        })
        .expect("a nested previewable row should exist");
    let command = build_runtime_preview_command(
        nested.row_id,
        match nested.value_kind {
            hyprland_settings::write_classification::ScalarWriteValueKind::Boolean => "true",
            _ => "1",
        },
        false,
    )
    .expect("nested preview command should build");
    let expression = &command.args[1];
    assert!(expression.starts_with("hl.config({ "));
    assert!(
        expression.matches("{ ").count() >= 3,
        "nested settings must produce nested tables: {expression}"
    );
    assert!(!expression.contains("reload"));

    // String-like values are quoted.
    if let Some(color_row) = SAFE_WRITABLE_ROWS.iter().find(|row| {
        matches!(
            row.value_kind,
            hyprland_settings::write_classification::ScalarWriteValueKind::Color
        ) && classify_runtime_preview_row(row)
            .capability
            .live_previewable_by_default()
    }) {
        let command = build_runtime_preview_command(color_row.row_id, "rgba(33ccffee)", false)
            .expect("color preview command should build");
        assert!(command.args[1].contains("\"rgba(33ccffee)\""));
    }
}

#[test]
fn command_construction_rejects_unsafe_values_rows_and_missing_dead_man() {
    let gaps_row = SAFE_WRITABLE_ROWS
        .iter()
        .find(|row| row.official_setting == "general.gaps_in")
        .expect("gaps_in row should exist");

    for bad_value in [
        "7; rm -rf /",
        "7' or '1",
        "7\"",
        "7\n8",
        "$(reboot)",
        "`id`",
        "{ nested = true }",
        "not-a-number",
        "",
    ] {
        assert!(
            build_runtime_preview_command(gaps_row.row_id, bad_value, false).is_err(),
            "value {bad_value:?} must be rejected"
        );
    }

    // Unknown row.
    assert!(matches!(
        build_runtime_preview_command("no.such.row", "1", false),
        Err(RuntimePreviewError::UnknownRow(_))
    ));

    // Blocked high-risk row is rejected even with dead-man confirmation.
    if let Some(blocked) = runtime_preview_capability_matrix()
        .into_iter()
        .find(|row| row.capability == RuntimePreviewCapability::BlockedHighRisk)
    {
        assert!(matches!(
            build_runtime_preview_command(blocked.row_id, "1", true),
            Err(RuntimePreviewError::RowNotLivePreviewable { .. })
        ));
    }

    // Dead-man rows are rejected without confirmation and allowed with it.
    if let Some(dead_man_row) = runtime_preview_capability_matrix().into_iter().find(|row| {
        row.capability == RuntimePreviewCapability::LivePreviewSupportedWithDeadMan
            && matches!(
                SAFE_WRITABLE_ROWS
                    .iter()
                    .find(|candidate| candidate.row_id == row.row_id)
                    .map(|candidate| candidate.value_kind),
                Some(hyprland_settings::write_classification::ScalarWriteValueKind::Boolean)
                    | Some(hyprland_settings::write_classification::ScalarWriteValueKind::Number)
            )
    }) {
        assert!(matches!(
            build_runtime_preview_command(&dead_man_row.row_id.to_string(), "1", false),
            Err(RuntimePreviewError::DeadManConfirmationRequired(_))
        ));
        assert!(build_runtime_preview_command(dead_man_row.row_id, "1", true).is_ok());
    }

    // RequiresConfigWrite / NotProvenYet rows are rejected.
    for capability in [
        RuntimePreviewCapability::RequiresConfigWrite,
        RuntimePreviewCapability::NotProvenYet,
    ] {
        if let Some(row) = runtime_preview_capability_matrix()
            .into_iter()
            .find(|row| row.capability == capability)
        {
            assert!(matches!(
                build_runtime_preview_command(row.row_id, "1", true),
                Err(RuntimePreviewError::RowNotLivePreviewable { .. })
            ));
        }
    }
}

#[test]
fn preview_session_captures_original_applies_and_reverts_without_config_writes() {
    let gaps_row = SAFE_WRITABLE_ROWS
        .iter()
        .find(|row| row.official_setting == "general.gaps_in")
        .expect("gaps_in row should exist");
    let mut runner = MockRunner::new("int: 5\nset: true");

    let mut session = start_runtime_preview_session(&mut runner, gaps_row.row_id, false)
        .expect("session should start");
    assert_eq!(session.original_value, "5");
    assert_eq!(session.state, RuntimePreviewSessionState::Active);
    assert!(session.dead_man.is_none());
    assert_eq!(
        runner.calls[0].1,
        runtime_option_query("general.gaps_in"),
        "session start must be a read-only getoption"
    );

    let receipt = apply_runtime_preview_value(&mut runner, &mut session, "9")
        .expect("preview apply should succeed");
    assert_eq!(receipt.applied_value, "9");
    assert_eq!(receipt.original_value, "5");
    assert!(!receipt.config_written);
    assert!(!receipt.reload_run);

    let revert =
        revert_runtime_preview_session(&mut runner, &mut session).expect("revert should succeed");
    assert_eq!(revert.restored_value, "5");
    assert!(!revert.config_written);
    assert!(!revert.reload_run);
    assert_eq!(session.state, RuntimePreviewSessionState::Reverted);
    let last_call = runner.calls.last().expect("revert call recorded");
    assert!(last_call.1[1].contains("gaps_in = 5"));

    // Every issued command was either a getoption read or an hl.config set;
    // never a reload, never a config write.
    for (program, args) in &runner.calls {
        assert_eq!(program, "hyprctl");
        assert!(args[0] == "getoption" || args[0] == "eval");
        assert!(!args.join(" ").contains("reload"));
    }
    assert_eq!(session.config_writes_during_preview, 0);
    assert_eq!(session.reload_runs_during_preview, 0);

    // Save marks the session and defers persistence to the config write path.
    let mut save_session = start_runtime_preview_session(&mut runner, gaps_row.row_id, false)
        .expect("session should start");
    apply_runtime_preview_value(&mut runner, &mut save_session, "8").expect("apply");
    let instruction =
        mark_runtime_preview_session_saved(&mut save_session).expect("save should mark");
    assert!(instruction.contains("existing backup/write/reread config path"));
    assert_eq!(save_session.state, RuntimePreviewSessionState::Saved);
    assert!(apply_runtime_preview_value(&mut runner, &mut save_session, "3").is_err());
}

#[test]
fn throttle_keeps_only_latest_value_and_respects_interval() {
    let mut throttle = RuntimePreviewThrottle::new(150);
    assert_eq!(throttle.offer("1", 0), Some("1".to_string()));
    assert_eq!(throttle.offer("2", 40), None);
    assert_eq!(throttle.offer("3", 80), None);
    assert_eq!(
        throttle.pending_value,
        Some("3".to_string()),
        "only the latest pending value is kept"
    );
    assert_eq!(throttle.take_due(100), None);
    assert_eq!(throttle.take_due(160), Some("3".to_string()));
    assert_eq!(throttle.take_due(200), None);
    assert_eq!(throttle.offer("4", 400), Some("4".to_string()));
}

#[test]
fn dead_man_model_reverts_on_timeout_unless_confirmed() {
    let mut dead_man = RuntimePreviewDeadMan::new(10_000);
    assert_eq!(
        dead_man.evaluate(),
        RuntimePreviewDeadManVerdict::KeepActive
    );
    dead_man.tick(9_999);
    assert_eq!(
        dead_man.evaluate(),
        RuntimePreviewDeadManVerdict::KeepActive
    );
    dead_man.tick(1);
    assert_eq!(
        dead_man.evaluate(),
        RuntimePreviewDeadManVerdict::RevertRequired
    );

    let mut confirmed = RuntimePreviewDeadMan::new(10_000);
    confirmed.confirm();
    confirmed.tick(60_000);
    assert_eq!(
        confirmed.evaluate(),
        RuntimePreviewDeadManVerdict::KeepActive
    );
    assert!(!confirmed.recovery_instruction.is_empty());
}

#[test]
fn getoption_parsing_reads_all_known_value_shapes() {
    assert_eq!(parse_getoption_value("int: 5\nset: true"), Some("5".into()));
    assert_eq!(
        parse_getoption_value("float: 0.50\nset: true"),
        Some("0.50".into())
    );
    assert_eq!(
        parse_getoption_value("bool: false\nset: false"),
        Some("false".into())
    );
    assert_eq!(
        parse_getoption_value("css gap data: 5 5 5 5\nset: true"),
        Some("5 5 5 5".into())
    );
    assert_eq!(parse_getoption_value("garbage"), None);
}

#[test]
fn runtime_preview_sources_have_no_reload_or_config_write_paths() {
    for module in ["src/runtime_preview.rs", "src/runtime_preview_executor.rs"] {
        let source = fs::read_to_string(module).expect("module source should read");
        for forbidden in [
            "hyprctl reload",
            "\"reload\"",
            "fs::write",
            "File::create",
            "write_all",
            "std::fs",
            "hyprland.conf",
            ".config/hypr",
            "write_flow::",
            "crate::write_flow",
            "apply_setting_change(",
            "execute_safe_write_scaffold",
            "execute_structured_family_controlled_write",
            "execute_first_active_config_write_pilot",
        ] {
            assert!(
                !source.contains(forbidden),
                "{module} must not contain {forbidden}"
            );
        }
    }
    // The executor is the only runtime-preview module allowed to run commands.
    let capability_source =
        fs::read_to_string("src/runtime_preview.rs").expect("capability source should read");
    assert!(!capability_source.contains("Command::"));
}

#[test]
fn capability_matrix_report_is_generated_and_consistent() {
    #[derive(serde::Serialize)]
    struct MatrixReport {
        #[serde(rename = "artifactKind")]
        artifact_kind: &'static str,
        #[serde(rename = "projectDataVersion")]
        project_data_version: &'static str,
        #[serde(rename = "mechanismEvidence")]
        mechanism_evidence: &'static str,
        summary: hyprland_settings::runtime_preview::RuntimePreviewMatrixSummary,
        #[serde(rename = "scalarRows")]
        scalar_rows: Vec<hyprland_settings::runtime_preview::RuntimePreviewRowCapability>,
        #[serde(rename = "structuredFamilies")]
        structured_families:
            Vec<hyprland_settings::runtime_preview::RuntimePreviewFamilyCapability>,
    }

    let report = MatrixReport {
        artifact_kind: "runtime-preview-capability-matrix",
        project_data_version: "v0.55.2",
        mechanism_evidence: hyprland_settings::runtime_preview::RUNTIME_PREVIEW_MECHANISM_EVIDENCE,
        summary: runtime_preview_matrix_summary(),
        scalar_rows: runtime_preview_capability_matrix(),
        structured_families: runtime_preview_family_capabilities(),
    };
    let rendered = serde_json::to_string_pretty(&report).expect("matrix should serialize");
    let mut rendered_with_newline = rendered;
    rendered_with_newline.push('\n');
    fs::write(MATRIX_REPORT, &rendered_with_newline).expect("matrix report should write");

    let parsed: serde_json::Value =
        serde_json::from_str(&rendered_with_newline).expect("matrix report should parse");
    assert_eq!(parsed["summary"]["scalar_rows_classified"], 341);
    assert_eq!(parsed["summary"]["structured_families_classified"], 7);
    assert_eq!(
        parsed["scalarRows"]
            .as_array()
            .expect("scalarRows array")
            .len(),
        341
    );
    assert_eq!(
        parsed["structuredFamilies"]
            .as_array()
            .expect("structuredFamilies array")
            .len(),
        7
    );
}
