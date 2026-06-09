use anyhow::Result;
use hyprland_settings::ui::model::{
    run_screen_shader_advisory_ui_action, screen_shader_advisory_gtk_widget_projection,
    ScreenShaderAdvisoryUiActionRequest, ScreenShaderAdvisoryUiResultState,
};
use hyprland_settings::write_classification::{is_safe_writable_setting, SAFE_WRITABLE_ROWS};
use serde_json::Value;

fn read_json(path: &str) -> Result<Value> {
    Ok(serde_json::from_str(&std::fs::read_to_string(path)?)?)
}

#[test]
fn gtk_widget_wiring_report_records_option_a_and_counts() -> Result<()> {
    let report =
        read_json("data/reports/screen-shader-advisory-gtk-widget-wiring-proof.v0.55.2.json")?;
    let coverage = read_json("data/reports/scalar-read-write-coverage.v0.55.2.json")?;
    let ui_impl =
        read_json("data/reports/screen-shader-advisory-ui-implementation-proof.v0.55.2.json")?;
    let approval = read_json("data/reports/screen-shader-production-gate-approval.v0.55.2.json")?;

    assert!(is_safe_writable_setting("decoration.screen_shader"));
    assert_eq!(SAFE_WRITABLE_ROWS.len(), 340);
    assert_eq!(coverage["counts"]["readableRows"], 341);
    assert_eq!(coverage["counts"]["writableRows"], 340);
    assert_eq!(coverage["counts"]["blockedWriteRows"], 1);

    assert_eq!(report["rowId"], "decoration.screen_shader");
    assert_eq!(report["officialSetting"], "decoration.screen_shader");
    assert_eq!(report["startingCommit"], "ae6242b");
    assert_eq!(report["selectedGtkWidgetWiringProofOption"], "Option A");
    assert_eq!(report["currentWritableStatus"], "writable");
    assert_eq!(report["productionGateEnforced"], true);
    assert_eq!(report["productionGateChanged"], false);
    assert_eq!(report["watchdogMigrationProofStatus"], "complete");
    assert_eq!(report["advisoryCompilerFeasibilityStatus"], "complete");
    assert_eq!(
        report["advisoryCompilerIntegrationDesignStatus"],
        "complete"
    );
    assert_eq!(report["advisoryHelperImplementationStatus"], "complete");
    assert_eq!(report["advisoryUiExposureDesignStatus"], "complete");
    assert_eq!(report["advisoryUiImplementationStatus"], "complete");
    assert_eq!(report["chosenAdvisoryTool"], "glslangValidator");
    assert_eq!(report["visibleGtkWidgetImplemented"], true);
    assert_eq!(
        report["gtkWidgetModule"],
        "src/ui/window.rs::append_screen_shader_advisory_controls"
    );
    assert_eq!(report["compileAwareValidationCurrentStatus"], "deferred");
    assert_eq!(report["compileAwareValidationChanged"], false);
    assert_eq!(report["compileAwareValidationImplemented"], false);
    assert_eq!(report["productionCompileAwareValidationImplemented"], false);
    assert_eq!(report["safeWritableRowsChanged"], false);
    assert_eq!(report["writeAllowlistChanged"], false);
    assert_eq!(report["rowsEnabledThisSprint"], 0);
    assert_eq!(report["readableRows"], 341);
    assert_eq!(report["writableRows"], 278);
    assert_eq!(report["blockedRows"], 63);
    assert_eq!(ui_impl["advisoryUiActionImplemented"], true);
    assert_eq!(approval["productionGateEnforcedThisSprint"], true);

    Ok(())
}

#[test]
fn gtk_widget_projection_is_screen_shader_only_and_separated_from_apply() -> Result<()> {
    let widget = screen_shader_advisory_gtk_widget_projection("decoration.screen_shader")
        .expect("screen shader widget projection expected");

    assert!(widget.visible_gtk_widget_implemented);
    assert_eq!(
        widget.gtk_widget_module,
        "src/ui/window.rs::append_screen_shader_advisory_controls"
    );
    assert!(!widget.file_chooser_execution_implemented);
    assert!(widget.selected_file_action_model_implemented);
    assert_eq!(
        widget.file_chooser_module,
        "not-implemented-direct-gtk-file-chooser-deferred"
    );
    assert_eq!(
        widget.selected_file_action_module,
        "src/ui/model.rs::run_screen_shader_advisory_selected_file_ui_action"
    );
    assert_eq!(
        widget.placement,
        "advanced-display-render-screen-shader-advisory-section-separated-from-apply-action"
    );
    assert!(widget.advanced_mode_required);
    assert!(widget.explicit_user_trigger_required);
    assert!(widget.separated_from_apply_action);
    assert_eq!(widget.button_label, "Run optional advisory check");
    assert_eq!(widget.default_state, "not_run");
    assert_eq!(widget.missing_selection_state, "not_run");
    assert!(!widget.file_chooser_behavior_proven);
    assert!(!widget.selected_file_boundary_proven);
    assert!(widget.missing_selection_behavior_proven);
    assert!(!widget.cancellation_or_progress_behavior_proven);
    assert!(!widget.can_approve_write);
    assert!(!widget.can_block_write);
    assert!(!widget.can_bypass_production_gate);
    assert!(widget
        .result_states_rendered
        .iter()
        .any(|state| state == "cleanup_warning"));

    assert!(screen_shader_advisory_gtk_widget_projection("appearance.blur.size").is_none());

    Ok(())
}

#[test]
fn visible_widget_missing_selection_path_does_not_read_or_compile() -> Result<()> {
    let render = run_screen_shader_advisory_ui_action(ScreenShaderAdvisoryUiActionRequest {
        row_id: "decoration.screen_shader".to_string(),
        explicit_user_trigger: true,
        helper_request: None,
    });

    assert_eq!(render.state, ScreenShaderAdvisoryUiResultState::NotRun);
    assert!(render.consent_required);
    assert!(!render.helper_invoked);
    assert!(!render.selected_shader_read);
    assert!(!render.compiler_invoked);
    assert!(render.compiler_args.is_empty());
    assert!(!render.can_approve_write);
    assert!(!render.can_block_write);
    assert!(!render.can_bypass_production_gate);
    assert!(!render.production_write_decision_changed);
    assert!(!render.runtime_safety_claimed);
    assert!(!render.write_blocking);

    Ok(())
}

#[test]
fn gtk_widget_wiring_report_keeps_action_out_of_write_safety() -> Result<()> {
    let report =
        read_json("data/reports/screen-shader-advisory-gtk-widget-wiring-proof.v0.55.2.json")?;

    assert_eq!(report["runsOnRowLoad"], false);
    assert_eq!(report["runsOnValueChange"], false);
    assert_eq!(report["runsDuringValidation"], false);
    assert_eq!(report["runsDuringPendingChange"], false);
    assert_eq!(report["runsDuringWritePlanning"], false);
    assert_eq!(report["runsDuringApplyFlow"], false);
    assert_eq!(report["advisoryResultCanApproveWrite"], false);
    assert_eq!(report["advisoryResultCanBlockWrite"], false);
    assert_eq!(report["advisoryResultCanBypassProductionGate"], false);
    assert_eq!(report["compilerChecksWiredIntoValidators"], false);
    assert_eq!(report["compilerChecksWiredIntoPendingChanges"], false);
    assert_eq!(report["compilerChecksWiredIntoWritePlanning"], false);
    assert_eq!(report["compilerChecksWiredIntoApplyFlow"], false);
    assert_eq!(report["shaderCompilationThroughHyprlandRun"], false);
    assert_eq!(
        report["standaloneCompilerCommandsRunOnlyOnTempFixtures"],
        true
    );
    assert_eq!(report["liveShaderCompileUsed"], false);
    assert_eq!(report["liveDisplayRuntimeProofUsed"], false);
    assert_eq!(report["realConfigTouched"], false);
    assert_eq!(report["runtimeTouched"], false);
    assert_eq!(report["reloadEvalLuaUsed"], false);
    assert_eq!(report["realUserShaderFilesReadInTests"], false);
    assert_eq!(report["backgroundShaderScanningAllowed"], false);
    assert_eq!(report["originalUserPathPassedToCompiler"], false);
    assert_eq!(report["tempCopyRequired"], true);
    assert_eq!(report["writesOutsideTempDirAllowed"], false);

    Ok(())
}

#[test]
fn gtk_widget_wiring_links_unified_pipeline() -> Result<()> {
    let report =
        read_json("data/reports/screen-shader-advisory-gtk-widget-wiring-proof.v0.55.2.json")?;
    let pipeline = read_json("data/reports/all-341-unified-pipeline.v0.55.2.json")?;

    assert_eq!(
        report["unifiedPipelineRepresentation"]["rowLinkField"],
        "advisoryGtkWidgetWiringProofSource"
    );
    assert_eq!(
        report["unifiedPipelineRepresentation"]["doesNotChangeWriteDecision"],
        true
    );
    assert_eq!(
        report["recommendedValidationPolicy"]["requiredPreflight"],
        false
    );
    assert_eq!(
        report["recommendedValidationPolicy"]["applyFlowIntegration"],
        false
    );
    assert!(report["compatibilityGaps"]
        .as_array()
        .expect("compatibility gaps should be explicit")
        .iter()
        .any(|gap| gap.as_str().unwrap().contains("file chooser")));
    assert!(report["proofStillMissing"]
        .as_array()
        .expect("missing proof should be explicit")
        .iter()
        .any(|gap| gap.as_str().unwrap().contains("file chooser")));
    assert!(report["nextRecommendedSprint"]
        .as_str()
        .unwrap()
        .contains("GTK file chooser visual proof"));

    let screen_shader_row = pipeline["rows"]
        .as_array()
        .unwrap()
        .iter()
        .find(|row| row["rowId"] == "decoration.screen_shader")
        .expect("screen shader row should exist in all-341 pipeline");
    assert_eq!(
        screen_shader_row["advisoryGtkWidgetWiringProofSource"],
        "screen-shader-advisory-gtk-widget-wiring-proof.v0.55.2.json"
    );
    assert_eq!(screen_shader_row["productionGateEnforcedThisSprint"], true);
    assert_eq!(screen_shader_row["countedAsEnabledHighRiskRow"], false);
    assert!(screen_shader_row["nextRequiredWork"]
        .as_str()
        .unwrap()
        .contains("Next high-risk bucket readiness"));

    Ok(())
}
