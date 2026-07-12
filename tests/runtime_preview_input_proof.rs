use std::fs;

use hyprland_settings::runtime_preview_dead_man::{
    dead_man_row_classifications, dead_man_ui_state, RuntimePreviewDeadManClassification,
};
use hyprland_settings::runtime_preview_input_proof::{
    input_proof_plan, input_proof_plans, input_proof_summary, live_proof_gate, proven_input_row,
    RuntimePreviewInputProofClassification, PROVEN_INPUT_ROWS,
};
use hyprland_settings::runtime_preview_ui_projection::runtime_preview_ui_projections;
use hyprland_settings::write_classification::SAFE_WRITABLE_ROWS;

const PROOF_PLAN_REPORT: &str = "data/reports/runtime-preview-input-cursor-proof-plan.v0.55.2.json";

#[test]
fn all_needs_live_proof_rows_have_specific_proof_plans() {
    let plans = input_proof_plans();
    assert_eq!(
        plans.len(),
        63,
        "all 63 original needs-live-proof rows (including any promoted) have plans"
    );

    let summary = input_proof_summary();
    assert_eq!(summary.needs_live_proof_rows_total, 63);
    let accounted = summary.keyboard_risk_rows
        + summary.pointer_risk_rows
        + summary.touchpad_risk_rows
        + summary.cursor_risk_rows
        + summary.focus_risk_rows
        + summary.gesture_risk_rows
        + summary.proof_model_only_rows
        + summary.proof_blocked_no_safe_fallback_rows
        + summary.proof_blocked_no_runtime_verification_rows
        + summary.proof_blocked_too_dangerous_rows
        + summary.proof_ready_for_env_gated_live_test_rows
        + summary.proof_passed_armable_candidate_rows;
    assert_eq!(accounted, 63, "every row falls into exactly one bucket");
    assert_eq!(summary.keyboard_risk_rows, 6);
    assert_eq!(summary.pointer_risk_rows, 11);
    assert_eq!(summary.touchpad_risk_rows, 18);
    assert_eq!(summary.cursor_risk_rows, 15);
    assert_eq!(summary.focus_risk_rows, 7);
    assert_eq!(summary.proof_blocked_too_dangerous_rows, 1);
    assert_eq!(summary.proof_blocked_no_runtime_verification_rows, 4);
    assert_eq!(summary.proof_passed_armable_candidate_rows, 1);
    assert_eq!(
        summary.proof_ready_for_env_gated_live_test_rows, 0,
        "the only proof-ready row was proven and promoted this sprint"
    );

    for plan in &plans {
        // Every plan answers the required questions.
        assert!(!plan.what_it_controls.is_empty());
        assert!(!plan.what_could_go_wrong.is_empty());
        assert!(!plan.minimal_preview_value.is_empty());
        assert!(!plan.manual_warning.is_empty());
        assert!(!plan.recovery_instruction.is_empty());
        assert!(plan
            .recovery_instruction
            .contains("countdown reverts automatically"));
        // Every plan has a believable fallback.
        assert!(
            plan.fallback.keyboard_remains_usable || plan.fallback.pointer_remains_usable,
            "{} needs at least one usable fallback input path",
            plan.row_id
        );
        assert!(plan.fallback.timeout_auto_revert_needs_no_input);
        assert!(!plan.fallback.tty_rollback_instruction.is_empty());
        // Blocked rows carry their reason.
        if matches!(
            plan.proof_classification,
            RuntimePreviewInputProofClassification::ProofBlockedNoSafeFallback
                | RuntimePreviewInputProofClassification::ProofBlockedNoRuntimeVerification
                | RuntimePreviewInputProofClassification::ProofBlockedTooDangerous
        ) {
            assert!(
                plan.blocked_reason.is_some(),
                "{} needs a blocked reason",
                plan.row_id
            );
        }
        // The env command is row-specific.
        assert!(plan.live_proof_env.contains(plan.official_setting));
    }
}

#[test]
fn promotion_requires_a_recorded_proof_receipt() {
    for plan in input_proof_plans() {
        if plan.proof_classification.armable() {
            let receipt = proven_input_row(plan.official_setting)
                .expect("armable rows must have a proof receipt");
            assert!(!receipt.original_value.is_empty());
            assert!(!receipt.preview_value.is_empty());
            assert!(!receipt.fallback_used.is_empty());
            assert!(!receipt.proof_env.is_empty());
        } else {
            // No dead-man arming without a receipt.
            let dead_man = dead_man_row_classifications()
                .into_iter()
                .find(|row| row.row_id == plan.row_id)
                .expect("dead-man classification exists");
            assert_ne!(
                dead_man.classification,
                RuntimePreviewDeadManClassification::DeadManPreviewCandidate,
                "{} must not arm without a proof receipt",
                plan.row_id
            );
            let ui = dead_man_ui_state(plan.row_id).expect("ui state exists");
            assert!(!ui.arm_enabled, "{} must render disarmed", plan.row_id);
        }
    }
    // The single recorded receipt is the row proven this sprint.
    assert_eq!(PROVEN_INPUT_ROWS.len(), 1);
    assert_eq!(
        PROVEN_INPUT_ROWS[0].official_setting,
        "cursor.inactive_timeout"
    );

    // The promoted row arms in the dead-man UI.
    let promoted = SAFE_WRITABLE_ROWS
        .iter()
        .find(|row| row.official_setting == "cursor.inactive_timeout")
        .expect("promoted row exists");
    let ui = dead_man_ui_state(promoted.row_id).expect("ui state exists");
    assert!(ui.arm_enabled, "the proof-passed row becomes armable");
}

#[test]
fn live_proof_gate_fails_closed() {
    // Unknown rows, non-input rows, unproven rows, and blocked rows refuse.
    assert!(live_proof_gate("no.such.row").is_err());
    assert!(live_proof_gate("general.gaps_in").is_err());
    assert!(live_proof_gate("input.left_handed").is_err());
    assert!(live_proof_gate("cursor.invisible").is_err());
    assert!(live_proof_gate("cursor.no_hardware_cursors").is_err());
    // The already-proven row is no longer proof-ready (it is passed), so the
    // harness will not re-run it accidentally.
    assert!(live_proof_gate("cursor.inactive_timeout").is_err());
}

#[test]
fn animation_candidates_and_blocked_rows_are_unchanged() {
    let rows = dead_man_row_classifications();
    for setting in ["animations.enabled", "animations.workspace_wraparound"] {
        let row = rows
            .iter()
            .find(|row| row.official_setting == setting)
            .expect("animation row exists");
        assert_eq!(
            row.classification,
            RuntimePreviewDeadManClassification::DeadManPreviewCandidate,
            "{setting} stays armed"
        );
    }
    // Normal preview projections unchanged.
    let projections = runtime_preview_ui_projections();
    assert_eq!(projections.len(), 341);
    assert_eq!(
        projections
            .iter()
            .filter(|state| state.preview_enabled)
            .count(),
        135
    );
    // Monitor/display rows remain blocked high-risk with no proof plans.
    for state in &projections {
        if state.capability
            == hyprland_settings::runtime_preview::RuntimePreviewCapability::BlockedHighRisk
        {
            assert!(
                input_proof_plan(state.row_id).is_none(),
                "{} is blocked high-risk and gets no input proof plan",
                state.row_id
            );
        }
    }
}

#[test]
fn proof_sources_have_no_command_or_config_write_paths() {
    let module_source =
        fs::read_to_string("src/runtime_preview_input_proof.rs").expect("module source reads");
    for forbidden in [
        "Command::new",
        "std::process",
        "fs::write",
        "File::create",
        "hl.config",
        "hyprctl reload",
        ".config/hypr",
        "write_flow::",
        "apply_setting_change(",
    ] {
        assert!(
            !module_source.contains(forbidden),
            "input proof module must not contain {forbidden}"
        );
    }
    // The live harness is ignored by default and env-gated.
    let harness_source = fs::read_to_string("tests/runtime_preview_input_live_proof.rs")
        .expect("harness source reads");
    assert!(harness_source.contains("#[ignore"));
    assert!(harness_source.contains("HYPRLAND_SETTINGS_RUN_INPUT_LIVE_PROOF"));
    assert!(harness_source.contains("HYPRLAND_SETTINGS_INPUT_PROOF_ROW"));
    assert!(!harness_source.contains("hyprctl reload"));
    assert!(!harness_source.contains("\"reload\""));
    // The UI never imports proof internals beyond the plan projection.
    let window_source = fs::read_to_string("src/ui/window.rs").expect("window source reads");
    assert!(!window_source.contains("live_proof_gate"));
    assert!(!window_source.contains("PROVEN_INPUT_ROWS"));
}

#[test]
fn input_proof_plan_report_is_generated_and_consistent() {
    #[derive(serde::Serialize)]
    struct ProofPlanReport {
        #[serde(rename = "artifactKind")]
        artifact_kind: &'static str,
        #[serde(rename = "projectDataVersion")]
        project_data_version: &'static str,
        summary: hyprland_settings::runtime_preview_input_proof::RuntimePreviewInputProofSummary,
        #[serde(rename = "provenRows")]
        proven_rows: Vec<hyprland_settings::runtime_preview_input_proof::ProvenInputRow>,
        plans: Vec<hyprland_settings::runtime_preview_input_proof::RuntimePreviewInputProofPlan>,
    }
    let report = ProofPlanReport {
        artifact_kind: "runtime-preview-input-cursor-proof-plan",
        project_data_version: "v0.55.2",
        summary: input_proof_summary(),
        proven_rows: PROVEN_INPUT_ROWS.to_vec(),
        plans: input_proof_plans(),
    };
    let mut rendered = serde_json::to_string_pretty(&report).expect("report serializes");
    rendered.push('\n');
    fs::write(PROOF_PLAN_REPORT, &rendered).expect("report writes");

    let parsed: serde_json::Value = serde_json::from_str(&rendered).expect("report parses");
    assert_eq!(parsed["summary"]["needs_live_proof_rows_total"], 63);
    assert_eq!(parsed["plans"].as_array().expect("plans array").len(), 63);
    assert_eq!(
        parsed["provenRows"].as_array().expect("proven array").len(),
        1
    );
}
