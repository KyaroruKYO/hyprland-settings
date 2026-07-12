use std::fs;

use hyprland_settings::runtime_preview::RuntimePreviewCapability;
use hyprland_settings::runtime_preview_executor::RuntimePreviewRunner;
use hyprland_settings::runtime_preview_ui_projection::{
    runtime_preview_ui_projections, runtime_preview_ui_row_state, RuntimePreviewUiControlKind,
    RuntimePreviewUiController, RuntimePreviewUiError, RuntimePreviewUiSessionState,
};
use hyprland_settings::write_classification::{ScalarWriteValueKind, SAFE_WRITABLE_ROWS};

const UI_CONTROLS_REPORT: &str = "data/reports/runtime-preview-ui-controls.v0.55.2.json";

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

/// Shared mock that records calls across a controller's lifetime.
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

fn gaps_row_id() -> &'static str {
    SAFE_WRITABLE_ROWS
        .iter()
        .find(|row| row.official_setting == "general.gaps_in")
        .expect("gaps_in row should exist")
        .row_id
}

#[test]
fn every_scalar_row_gets_a_ui_projection_and_only_previewable_rows_are_enabled() {
    let projections = runtime_preview_ui_projections();
    assert_eq!(projections.len(), 341, "all 341 rows must project");

    let enabled: Vec<_> = projections
        .iter()
        .filter(|state| state.preview_enabled)
        .collect();
    assert_eq!(
        enabled.len(),
        135,
        "exactly the 135 default-previewable rows get enabled controls"
    );

    for state in &projections {
        if state.preview_enabled {
            assert!(state.capability.live_previewable_by_default());
            assert_ne!(state.control_kind, RuntimePreviewUiControlKind::NoControl);
            assert!(state.revert_available);
            assert!(state.cancel_available);
            assert!(state.unavailable_reason.is_none());
        } else {
            assert_eq!(
                state.control_kind,
                RuntimePreviewUiControlKind::NoControl,
                "{} must not expose an enabled control",
                state.row_id
            );
            assert!(state.unavailable_reason.is_some());
        }
        // Blocked and dead-man rows are never preview-enabled.
        if matches!(
            state.capability,
            RuntimePreviewCapability::BlockedHighRisk
                | RuntimePreviewCapability::BlockedUnsupportedGrammar
                | RuntimePreviewCapability::LivePreviewSupportedWithDeadMan
                | RuntimePreviewCapability::NotProvenYet
                | RuntimePreviewCapability::RequiresConfigWrite
        ) {
            assert!(!state.preview_enabled, "{} must be disabled", state.row_id);
        }
        if state.capability == RuntimePreviewCapability::LivePreviewSupportedWithDeadMan {
            assert!(state.dead_man_required);
            assert_eq!(state.capability_badge, "Dead-man preview required");
        }
        assert!(!state.capability_badge.is_empty());
        assert!(!state.status_text.is_empty());
    }
}

#[test]
fn control_kind_mapping_is_deterministic_and_type_correct() {
    for state in runtime_preview_ui_projections() {
        let row = SAFE_WRITABLE_ROWS
            .iter()
            .find(|row| row.row_id == state.row_id)
            .expect("row exists");
        // Deterministic: recomputing yields the identical kind.
        let again = runtime_preview_ui_row_state(state.row_id).expect("state recomputes");
        assert_eq!(state.control_kind, again.control_kind);

        if !state.preview_enabled {
            continue;
        }
        match row.value_kind {
            ScalarWriteValueKind::Boolean => {
                assert_eq!(state.control_kind, RuntimePreviewUiControlKind::Switch)
            }
            ScalarWriteValueKind::FiniteChoice => {
                assert_eq!(state.control_kind, RuntimePreviewUiControlKind::Dropdown);
                assert!(
                    !state.dropdown_choices.is_empty(),
                    "{} dropdown needs choices",
                    state.row_id
                );
            }
            ScalarWriteValueKind::Number => {
                assert!(matches!(
                    state.control_kind,
                    RuntimePreviewUiControlKind::Slider | RuntimePreviewUiControlKind::SpinRow
                ));
                if state.control_kind == RuntimePreviewUiControlKind::Slider {
                    assert!(state.slider_bounds.is_some());
                }
            }
            ScalarWriteValueKind::Percent => {
                assert_eq!(state.control_kind, RuntimePreviewUiControlKind::Slider);
                assert_eq!(state.slider_bounds, Some((0.0, 1.0)));
            }
            ScalarWriteValueKind::Color | ScalarWriteValueKind::Gradient => {
                assert_eq!(state.control_kind, RuntimePreviewUiControlKind::ColorEntry)
            }
            ScalarWriteValueKind::CssGap
            | ScalarWriteValueKind::Vector2
            | ScalarWriteValueKind::NumericList
            | ScalarWriteValueKind::CommaSeparatedFloatList
            | ScalarWriteValueKind::SourceBacked => {
                assert_eq!(state.control_kind, RuntimePreviewUiControlKind::ValueEntry)
            }
            other => panic!(
                "{} is preview-enabled with unexpected value kind {other:?}",
                state.row_id
            ),
        }
        // Throttled rows carry the throttle policy into the UI state.
        if state.capability == RuntimePreviewCapability::LivePreviewSupportedWithThrottle {
            assert_eq!(state.throttle_ms, Some(150));
        }
    }
}

#[test]
fn controller_preview_save_revert_cancel_route_through_executor() {
    let row_id = gaps_row_id();
    let log = std::rc::Rc::new(std::cell::RefCell::new(Vec::new()));
    let runner = RecordingRunner {
        log: log.clone(),
        getoption_response: "int: 5\nset: true".to_string(),
    };
    let mut controller =
        RuntimePreviewUiController::new(row_id, Box::new(runner)).expect("controller builds");
    assert_eq!(
        controller.session_state(),
        RuntimePreviewUiSessionState::Idle
    );

    // First offer starts a session (read-only getoption) then applies.
    let receipt = controller
        .offer_value("9", 0)
        .expect("offer succeeds")
        .expect("first offer applies immediately");
    assert_eq!(receipt.action, "preview");
    assert_eq!(receipt.original_runtime_value.as_deref(), Some("5"));
    assert!(!receipt.config_written);
    assert!(!receipt.reload_run);
    assert!(receipt.status_text.contains("Previewing Live"));
    assert_eq!(
        controller.session_state(),
        RuntimePreviewUiSessionState::PreviewingLive
    );
    {
        let calls = log.borrow();
        assert_eq!(calls[0][0], "getoption", "session start is read-only");
        assert_eq!(calls[1][0], "eval");
        assert!(calls[1][1].contains("hl.config"));
    }

    // Throttled offers keep only the latest value; drain applies it.
    assert!(controller.offer_value("10", 40).expect("ok").is_none());
    assert!(controller.offer_value("11", 80).expect("ok").is_none());
    let call_count_before = log.borrow().len();
    let drained = controller
        .drain_pending(200)
        .expect("drain ok")
        .expect("pending value applies");
    assert_eq!(drained.value.as_deref(), Some("11"));
    assert_eq!(
        log.borrow().len(),
        call_count_before + 1,
        "exactly one runtime set for two throttled offers"
    );

    // Revert reapplies the original.
    let revert = controller.revert().expect("revert succeeds");
    assert_eq!(revert.action, "revert");
    assert_eq!(revert.value.as_deref(), Some("5"));
    assert!(log.borrow().last().expect("revert call recorded")[1].contains("gaps_in = 5"));
    assert_eq!(
        controller.session_state(),
        RuntimePreviewUiSessionState::Reverted
    );

    // A fresh preview then Cancel: reverts and clears the session.
    controller.offer_value("8", 1_000).expect("ok");
    let cancel = controller.cancel().expect("cancel succeeds");
    assert_eq!(cancel.action, "cancel");
    assert_eq!(
        cancel.session_state,
        RuntimePreviewUiSessionState::Cancelled
    );
    assert_eq!(
        controller.session_state(),
        RuntimePreviewUiSessionState::Idle,
        "cancel clears the session"
    );

    // Save marks the session; persistence is deferred to the config flow.
    controller.offer_value("7", 2_000).expect("ok");
    let save = controller.mark_saved().expect("save marks");
    assert_eq!(save.action, "save");
    assert!(!save.config_written, "the controller itself never persists");
    assert!(save
        .status_text
        .contains("existing backup/write/reread flow"));
    assert_eq!(
        controller.session_state(),
        RuntimePreviewUiSessionState::Saved
    );

    // No call the controller made was a config write or reload.
    for call in log.borrow().iter() {
        assert!(call[0] == "getoption" || call[0] == "eval");
        assert!(!call.join(" ").contains("reload"));
    }
}

#[test]
fn controller_rejects_unsupported_blocked_and_dead_man_rows() {
    for state in runtime_preview_ui_projections() {
        if state.preview_enabled {
            continue;
        }
        let runner = MockRunner::new("int: 1\nset: true");
        let mut controller = RuntimePreviewUiController::new(state.row_id, Box::new(runner))
            .expect("controller builds for any known row");
        let result = controller.offer_value("1", 0);
        assert!(
            matches!(result, Err(RuntimePreviewUiError::RowNotPreviewable(_))),
            "{} must reject preview",
            state.row_id
        );
    }
}

#[test]
fn app_close_model_reverts_only_active_sessions_with_applied_values() {
    let row_id = gaps_row_id();
    let log = std::rc::Rc::new(std::cell::RefCell::new(Vec::new()));
    let runner = RecordingRunner {
        log: log.clone(),
        getoption_response: "int: 5\nset: true".to_string(),
    };
    let mut controller =
        RuntimePreviewUiController::new(row_id, Box::new(runner)).expect("controller builds");

    // Nothing applied: revert_if_active is a no-op.
    assert!(controller.revert_if_active().is_none());

    controller.offer_value("9", 0).expect("apply");
    let receipt = controller
        .revert_if_active()
        .expect("active session with applied value reverts");
    assert_eq!(receipt.action, "revert");
    assert_eq!(receipt.value.as_deref(), Some("5"));

    // Already reverted: second call is a no-op.
    assert!(controller.revert_if_active().is_none());
}

#[test]
fn save_state_reflects_write_flow_availability() {
    for state in runtime_preview_ui_projections() {
        if state.preview_enabled {
            // Every previewable row is on the 341-row safe write allowlist,
            // so Save routes through the existing flow.
            assert!(
                state.save_state.available(),
                "{} should save through the existing write flow",
                state.row_id
            );
            assert!(state.save_state.reason().contains("backup/write/reread"));
        }
    }
}

#[test]
fn ui_source_guards_hold_for_preview_code() {
    let window_source = fs::read_to_string("src/ui/window.rs").expect("window source reads");
    // UI code never constructs runtime commands or runners directly; it goes
    // through the controller wrappers.
    for forbidden in [
        "hl.config",
        "Command::new",
        "std::process::Command",
        "HyprctlRuntimePreviewRunner",
        "build_runtime_preview_command",
        "apply_runtime_preview_value",
        "start_runtime_preview_session",
        "revert_runtime_preview_session",
    ] {
        assert!(
            !window_source.contains(forbidden),
            "src/ui/window.rs must not contain {forbidden}"
        );
    }
    // The preview section defers persistence to the existing safe write
    // flow; it must not call any raw file write API.
    for forbidden in ["fs::write", "File::create", "write_all"] {
        assert!(
            !window_source.contains(forbidden),
            "src/ui/window.rs must not contain {forbidden}"
        );
    }
    for module in ["src/ui/model.rs", "src/ui/app.rs", "src/ui/mod.rs"] {
        let source = fs::read_to_string(module).expect("ui source reads");
        assert!(
            !source.contains("runtime_preview_executor"),
            "{module} must not import the executor directly"
        );
    }
    // The projection module itself never runs commands directly; it routes
    // through the executor's runner abstraction.
    let projection_source = fs::read_to_string("src/runtime_preview_ui_projection.rs")
        .expect("projection source reads");
    for forbidden in [
        "Command::new",
        "std::process",
        "fs::write",
        "File::create",
        "hl.config",
        "hyprctl reload",
    ] {
        assert!(
            !projection_source.contains(forbidden),
            "projection module must not contain {forbidden}"
        );
    }
    // Save from the slider path is impossible: the apply closure calls
    // offer_value/drain_pending only; apply_setting_change appears only in
    // click handlers.
    assert!(
        window_source.contains("save_button.connect_clicked"),
        "save routes through an explicit button"
    );
}

#[test]
fn ui_controls_report_is_generated_and_consistent() {
    let projections = runtime_preview_ui_projections();
    let count_kind = |kind: RuntimePreviewUiControlKind| {
        projections
            .iter()
            .filter(|state| state.preview_enabled && state.control_kind == kind)
            .count()
    };
    let enabled = projections
        .iter()
        .filter(|state| state.preview_enabled)
        .count();

    #[derive(serde::Serialize)]
    struct UiControlsReport {
        #[serde(rename = "artifactKind")]
        artifact_kind: &'static str,
        #[serde(rename = "projectDataVersion")]
        project_data_version: &'static str,
        #[serde(rename = "scalarRowsTotal")]
        scalar_rows_total: usize,
        #[serde(rename = "uiControlProjectionsTotal")]
        ui_control_projections_total: usize,
        #[serde(rename = "uiControlProjectionsEnabled")]
        ui_control_projections_enabled: usize,
        #[serde(rename = "uiControlProjectionsDisabled")]
        ui_control_projections_disabled: usize,
        #[serde(rename = "switchControls")]
        switch_controls: usize,
        #[serde(rename = "sliderControls")]
        slider_controls: usize,
        #[serde(rename = "spinControls")]
        spin_controls: usize,
        #[serde(rename = "colorControls")]
        color_controls: usize,
        #[serde(rename = "entryControls")]
        entry_controls: usize,
        #[serde(rename = "dropdownControls")]
        dropdown_controls: usize,
        rows: Vec<hyprland_settings::runtime_preview_ui_projection::RuntimePreviewUiRowState>,
    }

    let report = UiControlsReport {
        artifact_kind: "runtime-preview-ui-controls",
        project_data_version: "v0.55.2",
        scalar_rows_total: 341,
        ui_control_projections_total: projections.len(),
        ui_control_projections_enabled: enabled,
        ui_control_projections_disabled: projections.len() - enabled,
        switch_controls: count_kind(RuntimePreviewUiControlKind::Switch),
        slider_controls: count_kind(RuntimePreviewUiControlKind::Slider),
        spin_controls: count_kind(RuntimePreviewUiControlKind::SpinRow),
        color_controls: count_kind(RuntimePreviewUiControlKind::ColorEntry),
        entry_controls: count_kind(RuntimePreviewUiControlKind::ValueEntry),
        dropdown_controls: count_kind(RuntimePreviewUiControlKind::Dropdown),
        rows: projections,
    };
    let mut rendered = serde_json::to_string_pretty(&report).expect("report serializes");
    rendered.push('\n');
    fs::write(UI_CONTROLS_REPORT, &rendered).expect("report writes");

    let parsed: serde_json::Value = serde_json::from_str(&rendered).expect("report parses");
    assert_eq!(parsed["uiControlProjectionsTotal"], 341);
    assert_eq!(parsed["uiControlProjectionsEnabled"], 135);
    let kinds_sum = report.switch_controls
        + report.slider_controls
        + report.spin_controls
        + report.color_controls
        + report.entry_controls
        + report.dropdown_controls;
    assert_eq!(kinds_sum, 135, "every enabled row has exactly one control");
}
