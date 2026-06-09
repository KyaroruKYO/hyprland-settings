use anyhow::Result;
use hyprland_settings::write_classification::{is_safe_writable_setting, SAFE_WRITABLE_ROWS};
use serde_json::Value;

fn read_json(path: &str) -> Result<Value> {
    Ok(serde_json::from_str(&std::fs::read_to_string(path)?)?)
}

#[test]
fn screen_shader_advisory_ui_design_report_records_option_a() -> Result<()> {
    let report = read_json("data/reports/screen-shader-advisory-ui-exposure-design.v0.55.2.json")?;
    let coverage = read_json("data/reports/scalar-read-write-coverage.v0.55.2.json")?;
    let helper = read_json(
        "data/reports/screen-shader-advisory-compiler-implementation-proof.v0.55.2.json",
    )?;
    let approval = read_json("data/reports/screen-shader-production-gate-approval.v0.55.2.json")?;

    assert!(is_safe_writable_setting("decoration.screen_shader"));
    assert_eq!(SAFE_WRITABLE_ROWS.len(), 341);
    assert_eq!(coverage["counts"]["readableRows"], 341);
    assert_eq!(coverage["counts"]["writableRows"], 341);
    assert_eq!(coverage["counts"]["blockedWriteRows"], 0);

    assert_eq!(report["rowId"], "decoration.screen_shader");
    assert_eq!(report["officialSetting"], "decoration.screen_shader");
    assert_eq!(report["startingCommit"], "9363dc6");
    assert_eq!(report["selectedUiExposureDesignOption"], "Option A");
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
    assert_eq!(report["chosenAdvisoryTool"], "glslangValidator");
    assert_eq!(report["advisoryUiExposureImplemented"], false);
    assert_eq!(report["uiExposureDesignOnly"], true);
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
    assert_eq!(helper["advisoryHelperImplemented"], true);
    assert_eq!(approval["productionGateEnforcedThisSprint"], true);

    Ok(())
}

#[test]
fn screen_shader_advisory_ui_design_requires_explicit_advanced_action() -> Result<()> {
    let report = read_json("data/reports/screen-shader-advisory-ui-exposure-design.v0.55.2.json")?;

    assert_eq!(
        report["uiActionPlacement"],
        "advanced-display-render-screen-shader-advisory-section-separated-from-apply-action"
    );
    assert_eq!(report["advancedModeRequired"], true);
    assert_eq!(report["explicitUserTriggerRequired"], true);
    assert_eq!(report["runsOnRowLoad"], false);
    assert_eq!(report["runsOnValueChange"], false);
    assert_eq!(report["runsDuringValidation"], false);
    assert_eq!(report["runsDuringPendingChange"], false);
    assert_eq!(report["runsDuringWritePlanning"], false);
    assert_eq!(report["runsDuringApplyFlow"], false);
    assert_eq!(report["userConsentMessageRequired"], true);
    assert_eq!(report["tempCopyMessageRequired"], true);
    assert_eq!(report["originalPathNotPassedMessageRequired"], true);
    assert_eq!(report["runtimeSafetyDisclaimerRequired"], true);
    assert_eq!(report["productionGateDisclaimerRequired"], true);
    assert_eq!(report["backgroundShaderScanningAllowed"], false);

    Ok(())
}

#[test]
fn screen_shader_advisory_ui_design_keeps_results_non_authoritative() -> Result<()> {
    let report = read_json("data/reports/screen-shader-advisory-ui-exposure-design.v0.55.2.json")?;

    assert_eq!(report["advisoryResultCanApproveWrite"], false);
    assert_eq!(report["advisoryResultCanBlockWrite"], false);
    assert_eq!(report["advisoryResultCanBypassProductionGate"], false);
    assert!(report["advisoryPassPolicy"]["message"]
        .as_str()
        .unwrap()
        .contains("does not approve"));
    assert!(report["advisoryFailurePolicy"]["message"]
        .as_str()
        .unwrap()
        .contains("does not automatically block"));
    assert!(report["missingToolPolicy"]["message"]
        .as_str()
        .unwrap()
        .contains("does not approve or block"));
    assert!(report["timeoutPolicy"]["message"]
        .as_str()
        .unwrap()
        .contains("does not approve or block"));
    assert!(report["cleanupWarningPolicy"]["message"]
        .as_str()
        .unwrap()
        .contains("without approving"));

    Ok(())
}

#[test]
fn screen_shader_advisory_ui_design_is_not_wired_into_write_safety() -> Result<()> {
    let report = read_json("data/reports/screen-shader-advisory-ui-exposure-design.v0.55.2.json")?;

    assert_eq!(report["compilerChecksWiredIntoValidators"], false);
    assert_eq!(report["compilerChecksWiredIntoPendingChanges"], false);
    assert_eq!(report["compilerChecksWiredIntoWritePlanning"], false);
    assert_eq!(report["compilerChecksWiredIntoApplyFlow"], false);
    assert_eq!(report["shaderCompilationThroughHyprlandRun"], false);
    assert_eq!(report["standaloneCompilerCommandsRunThisSprint"], true);
    assert!(report["standaloneCompilerCommandScope"]
        .as_str()
        .unwrap()
        .contains("generated-temp-fixtures"));
    assert_eq!(report["liveShaderCompileUsed"], false);
    assert_eq!(report["liveDisplayRuntimeProofUsed"], false);
    assert_eq!(report["realConfigTouched"], false);
    assert_eq!(report["runtimeTouched"], false);
    assert_eq!(report["reloadEvalLuaUsed"], false);
    assert_eq!(report["realUserShaderFilesReadInTests"], false);

    Ok(())
}

#[test]
fn screen_shader_advisory_ui_design_links_unified_pipeline() -> Result<()> {
    let report = read_json("data/reports/screen-shader-advisory-ui-exposure-design.v0.55.2.json")?;
    let pipeline = read_json("data/reports/all-341-unified-pipeline.v0.55.2.json")?;

    assert_eq!(
        report["recommendedValidationPolicy"]["policy"],
        "optional-advanced-ui-advisory-design-only"
    );
    assert_eq!(
        report["recommendedValidationPolicy"]["uiExecutionImplemented"],
        false
    );
    assert_eq!(
        report["unifiedPipelineRepresentation"]["rowLinkField"],
        "advisoryUiExposureDesignSource"
    );
    assert_eq!(
        report["unifiedPipelineRepresentation"]["metadataOnly"],
        true
    );
    let gaps = report["compatibilityGaps"]
        .as_array()
        .expect("compatibility gaps should be explicit");
    assert!(!gaps.is_empty());
    assert!(gaps
        .iter()
        .any(|gap| gap.as_str().unwrap().contains("UI execution")));
    let missing = report["proofStillMissing"]
        .as_array()
        .expect("missing proof should be explicit");
    assert!(missing
        .iter()
        .any(|gap| gap.as_str().unwrap().contains("No full GTK file chooser")));
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
        screen_shader_row["advisoryUiExposureDesignSource"],
        "screen-shader-advisory-ui-exposure-design.v0.55.2.json"
    );
    assert!(screen_shader_row["nextRequiredWork"]
        .as_str()
        .unwrap()
        .contains("Next high-risk bucket readiness"));
    assert_eq!(screen_shader_row["productionGateEnforcedThisSprint"], true);
    assert_eq!(screen_shader_row["countedAsEnabledHighRiskRow"], false);

    Ok(())
}
