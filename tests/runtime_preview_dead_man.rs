use std::fs;

use hyprland_settings::runtime_preview::RuntimePreviewCapability;
use hyprland_settings::runtime_preview_dead_man::{
    classify_dead_man_row, dead_man_classification_summary, dead_man_row_classifications,
    dead_man_ui_state, RuntimePreviewDeadManClassification, RuntimePreviewDeadManController,
    RuntimePreviewDeadManUiError, RuntimePreviewDeadManUiPhase, DEAD_MAN_COUNTDOWN_SECONDS,
};
use hyprland_settings::runtime_preview_executor::RuntimePreviewRunner;
use hyprland_settings::runtime_preview_ui_projection::runtime_preview_ui_projections;
use hyprland_settings::write_classification::SAFE_WRITABLE_ROWS;

const CLASSIFICATION_REPORT: &str =
    "data/reports/runtime-preview-dead-man-classification.v0.55.2.json";

struct RecordingRunner {
    log: std::rc::Rc<std::cell::RefCell<Vec<Vec<String>>>>,
    getoption_response: String,
}

impl RuntimePreviewRunner for RecordingRunner {
    fn run(&mut self, _program: &str, args: &[String]) -> Result<String, String> {
        self.log.borrow_mut().push(args.to_vec());
        if args.first().map(String::as_str) == Some("getoption") {
            Ok(self.getoption_response.clone())
        } else {
            Ok(String::from("ok"))
        }
    }
}

fn candidate_row_id() -> &'static str {
    SAFE_WRITABLE_ROWS
        .iter()
        .find(|row| row.official_setting == "animations.enabled")
        .expect("animations.enabled row should exist")
        .row_id
}

fn controller_with_log() -> (
    RuntimePreviewDeadManController,
    std::rc::Rc<std::cell::RefCell<Vec<Vec<String>>>>,
) {
    let log = std::rc::Rc::new(std::cell::RefCell::new(Vec::new()));
    let runner = RecordingRunner {
        log: log.clone(),
        getoption_response: "bool: true\nset: true".to_string(),
    };
    let controller = RuntimePreviewDeadManController::new(candidate_row_id(), Box::new(runner))
        .expect("controller builds");
    (controller, log)
}

#[test]
fn all_78_dead_man_rows_are_reclassified_with_specific_statuses() {
    let rows = dead_man_row_classifications();
    assert_eq!(rows.len(), 78, "all dead-man rows must be reclassified");

    let summary = dead_man_classification_summary();
    assert_eq!(summary.dead_man_rows_total, 78);
    let accounted = summary.candidates
        + summary.candidates_needing_live_proof
        + summary.model_only
        + summary.blocked_no_safe_runtime_mechanism
        + summary.blocked_requires_relog
        + summary.blocked_requires_restart
        + summary.blocked_no_visible_effect
        + summary.blocked_too_dangerous;
    assert_eq!(accounted, 78, "every row falls into exactly one bucket");
    assert_eq!(
        summary.candidates, 2,
        "only the two animation toggles are proven candidates"
    );
    assert_eq!(summary.candidates_needing_live_proof, 63);
    assert_eq!(summary.model_only, 5);
    assert_eq!(summary.blocked_no_safe_runtime_mechanism, 8);

    for row in &rows {
        assert!(!row.reason.is_empty(), "{} needs a reason", row.row_id);
        // Every dead-man classified row is dead-man gated in the matrix.
        assert!(classify_dead_man_row(row.row_id).is_some());
        // Input/cursor rows are never plain candidates.
        if row.official_setting.starts_with("input.") || row.official_setting.starts_with("cursor.")
        {
            assert_ne!(
                row.classification,
                RuntimePreviewDeadManClassification::DeadManPreviewCandidate,
                "{} must not be an enabled candidate without live proof",
                row.row_id
            );
        }
    }

    // Non-dead-man rows return no classification.
    let normal_row = SAFE_WRITABLE_ROWS
        .iter()
        .find(|row| row.official_setting == "general.gaps_in")
        .expect("gaps row exists");
    assert!(classify_dead_man_row(normal_row.row_id).is_none());
}

#[test]
fn monitor_and_display_rows_are_not_dead_man_and_stay_blocked() {
    // The dead-man set contains no monitor/display/render rows: those remain
    // BlockedHighRisk in the capability matrix with no supervised panel.
    for row in dead_man_row_classifications() {
        let section = row.official_setting.split('.').next().unwrap_or_default();
        assert!(
            !matches!(
                section,
                "monitor" | "render" | "opengl" | "xwayland" | "experimental"
            ),
            "{} must not be dead-man gated",
            row.row_id
        );
    }
    for state in runtime_preview_ui_projections() {
        if state.capability == RuntimePreviewCapability::BlockedHighRisk {
            assert!(
                dead_man_ui_state(state.row_id).is_none(),
                "{} is blocked high-risk and must have no dead-man panel",
                state.row_id
            );
            assert!(!state.preview_enabled);
        }
    }
}

#[test]
fn dead_man_ui_states_enable_only_proven_candidates() {
    let mut armed = 0;
    let mut panels = 0;
    for row in dead_man_row_classifications() {
        let ui = dead_man_ui_state(row.row_id).expect("dead-man rows project UI state");
        assert_eq!(ui.badge, "Dead-man preview required");
        assert_eq!(ui.countdown_seconds, DEAD_MAN_COUNTDOWN_SECONDS);
        assert!(!ui.recovery_instruction.is_empty());
        assert!(ui.warning_text.contains("reverts automatically"));
        if ui.arm_enabled {
            armed += 1;
            assert_eq!(
                row.classification,
                RuntimePreviewDeadManClassification::DeadManPreviewCandidate
            );
            assert!(ui.disabled_reason.is_none());
        } else {
            assert!(
                ui.disabled_reason.is_some(),
                "{} needs a reason",
                row.row_id
            );
        }
        if ui.shows_panel {
            panels += 1;
        } else {
            assert!(matches!(
                row.classification,
                RuntimePreviewDeadManClassification::DeadManPreviewBlockedNoSafeRuntimeMechanism
                    | RuntimePreviewDeadManClassification::DeadManPreviewBlockedRequiresRelog
                    | RuntimePreviewDeadManClassification::DeadManPreviewBlockedRequiresRestart
                    | RuntimePreviewDeadManClassification::DeadManPreviewBlockedNoVisibleEffect
                    | RuntimePreviewDeadManClassification::DeadManPreviewBlockedTooDangerous
            ));
        }
    }
    assert_eq!(armed, 2, "only the two animation candidates arm");
    assert_eq!(
        panels, 70,
        "candidates + needs-live-proof + model-only show the panel"
    );
}

#[test]
fn dead_man_arm_apply_confirm_and_manual_revert_flow() {
    let (mut controller, log) = controller_with_log();
    assert_eq!(controller.phase(), RuntimePreviewDeadManUiPhase::Disarmed);

    // Arm captures the original value read-only.
    let armed = controller.arm().expect("arm succeeds");
    assert_eq!(armed.action, "arm");
    assert_eq!(armed.original_value.as_deref(), Some("true"));
    assert_eq!(controller.phase(), RuntimePreviewDeadManUiPhase::Armed);
    assert_eq!(log.borrow()[0][0], "getoption");

    // Apply routes through the executor and starts the countdown.
    let applied = controller.apply("false").expect("apply succeeds");
    assert_eq!(applied.action, "apply");
    assert!(!applied.config_written);
    assert!(!applied.reload_run);
    assert_eq!(
        controller.phase(),
        RuntimePreviewDeadManUiPhase::CountingDown
    );
    assert_eq!(controller.remaining_seconds(), DEAD_MAN_COUNTDOWN_SECONDS);
    {
        let calls = log.borrow();
        let apply_call = calls.last().expect("apply call recorded");
        assert_eq!(apply_call[0], "eval");
        assert!(apply_call[1].contains("hl.config"));
        assert!(apply_call[1].contains("animations = { enabled = false }"));
    }

    // Ticking below the timeout keeps the preview.
    assert!(controller.tick(4000).expect("tick ok").is_none());
    assert_eq!(
        controller.remaining_seconds(),
        DEAD_MAN_COUNTDOWN_SECONDS - 4
    );

    // Confirm keeps the preview and stops the countdown.
    let kept = controller.confirm_keep().expect("confirm succeeds");
    assert_eq!(kept.action, "confirm-keep");
    assert_eq!(controller.phase(), RuntimePreviewDeadManUiPhase::Kept);
    let calls_before = log.borrow().len();
    assert!(controller.tick(60_000).expect("tick ok").is_none());
    assert_eq!(
        log.borrow().len(),
        calls_before,
        "kept previews never auto-revert"
    );

    // Manual revert still works after Keep.
    let reverted = controller.revert_now().expect("revert succeeds");
    assert_eq!(reverted.action, "revert-now");
    assert_eq!(reverted.value.as_deref(), Some("true"));
    assert!(log.borrow().last().expect("revert call recorded")[1].contains("enabled = true"));
    assert_eq!(controller.phase(), RuntimePreviewDeadManUiPhase::Reverted);
}

#[test]
fn dead_man_timeout_auto_reverts_the_original_value() {
    let (mut controller, log) = controller_with_log();
    controller.arm().expect("arm");
    controller.apply("false").expect("apply");

    // One tick short of the timeout: still counting.
    assert!(controller
        .tick(DEAD_MAN_COUNTDOWN_SECONDS * 1000 - 1)
        .expect("tick ok")
        .is_none());
    // Crossing the timeout auto-reverts.
    let receipt = controller
        .tick(1)
        .expect("tick ok")
        .expect("timeout fires a receipt");
    assert_eq!(
        receipt.phase,
        RuntimePreviewDeadManUiPhase::TimedOutReverted
    );
    assert!(receipt.status_text.contains("automatically restored"));
    assert_eq!(
        controller.phase(),
        RuntimePreviewDeadManUiPhase::TimedOutReverted
    );
    assert!(log.borrow().last().expect("revert call recorded")[1].contains("enabled = true"));
    assert!(!receipt.config_written);
    assert!(!receipt.reload_run);
}

#[test]
fn dead_man_cancel_and_session_drop_revert_unconfirmed_previews() {
    // Cancel reverts and disarms.
    let (mut controller, log) = controller_with_log();
    controller.arm().expect("arm");
    controller.apply("false").expect("apply");
    let cancelled = controller.cancel().expect("cancel succeeds");
    assert_eq!(cancelled.phase, RuntimePreviewDeadManUiPhase::Cancelled);
    assert!(log.borrow().last().expect("cancel revert recorded")[1].contains("enabled = true"));

    // Session drop reverts an unconfirmed counting-down preview.
    let (mut controller, log) = controller_with_log();
    controller.arm().expect("arm");
    controller.apply("false").expect("apply");
    let receipt = controller
        .revert_if_unconfirmed()
        .expect("unconfirmed preview reverts on session drop");
    assert_eq!(receipt.action, "session-drop");
    assert!(
        log.borrow().last().expect("session-drop revert recorded")[1].contains("enabled = true")
    );

    // Kept previews are not reverted by session drop.
    let (mut controller, _log) = controller_with_log();
    controller.arm().expect("arm");
    controller.apply("false").expect("apply");
    controller.confirm_keep().expect("keep");
    assert!(controller.revert_if_unconfirmed().is_none());

    // Armed-but-never-applied sessions have nothing to revert.
    let (mut controller, _log) = controller_with_log();
    controller.arm().expect("arm");
    assert!(controller.revert_if_unconfirmed().is_none());
}

#[test]
fn dead_man_controller_rejects_unproven_rows_and_out_of_phase_actions() {
    // Needs-live-proof and model-only rows cannot arm.
    for row in dead_man_row_classifications() {
        if row.classification.supervised_preview_enabled() {
            continue;
        }
        let runner = RecordingRunner {
            log: std::rc::Rc::new(std::cell::RefCell::new(Vec::new())),
            getoption_response: "bool: true\nset: true".to_string(),
        };
        let mut controller = RuntimePreviewDeadManController::new(row.row_id, Box::new(runner))
            .expect("controller builds for any dead-man row");
        assert!(
            matches!(
                controller.arm(),
                Err(RuntimePreviewDeadManUiError::RowNotSupervisable(_))
            ),
            "{} must refuse to arm",
            row.row_id
        );
    }

    // Out-of-phase actions fail closed.
    let (mut controller, _log) = controller_with_log();
    assert!(controller.apply("false").is_err(), "apply before arm fails");
    assert!(controller.confirm_keep().is_err());
    assert!(controller.revert_now().is_err());

    // Normal rows cannot get a dead-man controller.
    let normal_row = SAFE_WRITABLE_ROWS
        .iter()
        .find(|row| row.official_setting == "general.gaps_in")
        .expect("gaps row exists");
    let runner = RecordingRunner {
        log: std::rc::Rc::new(std::cell::RefCell::new(Vec::new())),
        getoption_response: "int: 5\nset: true".to_string(),
    };
    assert!(RuntimePreviewDeadManController::new(normal_row.row_id, Box::new(runner)).is_err());
}

#[test]
fn normal_preview_projections_are_unchanged_by_the_dead_man_layer() {
    let projections = runtime_preview_ui_projections();
    assert_eq!(projections.len(), 341);
    assert_eq!(
        projections
            .iter()
            .filter(|state| state.preview_enabled)
            .count(),
        135,
        "the 135 default-previewable rows keep their normal controls"
    );
    for state in &projections {
        if state.capability == RuntimePreviewCapability::LivePreviewSupportedWithDeadMan {
            assert!(
                !state.preview_enabled,
                "{} stays out of normal preview mode",
                state.row_id
            );
        }
    }
}

#[test]
fn dead_man_sources_have_no_command_or_config_write_paths() {
    let module_source =
        fs::read_to_string("src/runtime_preview_dead_man.rs").expect("dead-man source reads");
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
            "dead-man module must not contain {forbidden}"
        );
    }
    let window_source = fs::read_to_string("src/ui/window.rs").expect("window source reads");
    for forbidden in [
        "hl.config",
        "Command::new",
        "apply_runtime_preview_value_supervised",
        "revert_runtime_preview_session_supervised",
        "start_runtime_preview_session",
    ] {
        assert!(
            !window_source.contains(forbidden),
            "UI must route through the dead-man controller, not contain {forbidden}"
        );
    }
}

#[test]
fn dead_man_classification_report_is_generated_and_consistent() {
    #[derive(serde::Serialize)]
    struct ClassificationReport {
        #[serde(rename = "artifactKind")]
        artifact_kind: &'static str,
        #[serde(rename = "projectDataVersion")]
        project_data_version: &'static str,
        summary:
            hyprland_settings::runtime_preview_dead_man::RuntimePreviewDeadManClassificationSummary,
        rows: Vec<
            hyprland_settings::runtime_preview_dead_man::RuntimePreviewDeadManRowClassification,
        >,
    }
    let report = ClassificationReport {
        artifact_kind: "runtime-preview-dead-man-classification",
        project_data_version: "v0.55.2",
        summary: dead_man_classification_summary(),
        rows: dead_man_row_classifications(),
    };
    let mut rendered = serde_json::to_string_pretty(&report).expect("report serializes");
    rendered.push('\n');
    fs::write(CLASSIFICATION_REPORT, &rendered).expect("report writes");

    let parsed: serde_json::Value = serde_json::from_str(&rendered).expect("report parses");
    assert_eq!(parsed["summary"]["dead_man_rows_total"], 78);
    assert_eq!(parsed["rows"].as_array().expect("rows array").len(), 78);
}
