use anyhow::Result;
use hyprland_settings::write_classification::{is_safe_writable_setting, SAFE_WRITABLE_ROWS};
use serde_json::Value;

fn read_json(path: &str) -> Result<Value> {
    Ok(serde_json::from_str(&std::fs::read_to_string(path)?)?)
}

#[test]
fn screen_shader_advisory_integration_design_report_records_option_a() -> Result<()> {
    let report =
        read_json("data/reports/screen-shader-advisory-compiler-integration-design.v0.55.2.json")?;
    let coverage = read_json("data/reports/scalar-read-write-coverage.v0.55.2.json")?;
    let feasibility = read_json(
        "data/reports/screen-shader-non-live-advisory-compiler-feasibility.v0.55.2.json",
    )?;
    let approval = read_json("data/reports/screen-shader-production-gate-approval.v0.55.2.json")?;

    assert!(is_safe_writable_setting("decoration.screen_shader"));
    assert_eq!(SAFE_WRITABLE_ROWS.len(), 341);
    assert_eq!(coverage["counts"]["readableRows"], 341);
    assert_eq!(coverage["counts"]["writableRows"], 341);
    assert_eq!(coverage["counts"]["blockedWriteRows"], 0);

    assert_eq!(report["rowId"], "decoration.screen_shader");
    assert_eq!(report["officialSetting"], "decoration.screen_shader");
    assert_eq!(report["startingCommit"], "865849f");
    assert_eq!(report["selectedIntegrationDesignOption"], "Option A");
    assert_eq!(report["currentWritableStatus"], "writable");
    assert_eq!(report["productionGateEnforced"], true);
    assert_eq!(report["productionGateChanged"], false);
    assert_eq!(report["watchdogMigrationProofStatus"], "complete");
    assert_eq!(report["advisoryCompilerFeasibilityStatus"], "complete");
    assert_eq!(report["chosenAdvisoryTool"], "glslangValidator");
    assert_eq!(report["compileAwareValidationCurrentStatus"], "deferred");
    assert_eq!(report["compileAwareValidationChanged"], false);
    assert_eq!(report["compileAwareValidationImplemented"], false);
    assert_eq!(report["productionCompileAwareValidationImplemented"], false);
    assert_eq!(report["advisoryCompilerIntegrationImplemented"], false);
    assert_eq!(report["advisoryHelperImplemented"], true);
    assert_eq!(
        report["advisoryHelperModule"],
        "src/screen_shader_advisory.rs"
    );
    assert_eq!(report["shaderCompilationThroughHyprlandRun"], false);
    assert_eq!(report["standaloneCompilerCommandsRunThisSprint"], false);
    assert_eq!(report["liveShaderCompileUsed"], false);
    assert_eq!(report["liveDisplayRuntimeProofUsed"], false);
    assert_eq!(report["realConfigTouched"], false);
    assert_eq!(report["runtimeTouched"], false);
    assert_eq!(report["reloadEvalLuaUsed"], false);
    assert_eq!(report["realUserShaderFilesReadInTests"], false);
    assert_eq!(report["safeWritableRowsChanged"], false);
    assert_eq!(report["writeAllowlistChanged"], false);
    assert_eq!(report["rowsEnabledThisSprint"], 0);
    assert_eq!(report["readableRows"], 341);
    assert_eq!(report["writableRows"], 278);
    assert_eq!(report["blockedRows"], 63);
    assert_eq!(feasibility["selectedFeasibilityOption"], "Option A");
    assert_eq!(feasibility["chosenAdvisoryTool"], "glslangValidator");
    assert_eq!(approval["productionGateEnforcedThisSprint"], true);
    assert_eq!(approval["compileAwareValidationStatus"], "deferred");

    Ok(())
}

#[test]
fn screen_shader_advisory_integration_design_is_not_wired_into_write_paths() -> Result<()> {
    let report =
        read_json("data/reports/screen-shader-advisory-compiler-integration-design.v0.55.2.json")?;

    assert_eq!(report["compilerChecksWiredIntoValidators"], false);
    assert_eq!(report["compilerChecksWiredIntoPendingChanges"], false);
    assert_eq!(report["compilerChecksWiredIntoWritePlanning"], false);
    assert_eq!(report["compilerChecksWiredIntoApplyFlow"], false);
    assert_eq!(
        report["recommendedValidationPolicy"]["validatorIntegration"],
        false
    );
    assert_eq!(
        report["recommendedValidationPolicy"]["pendingChangeIntegration"],
        false
    );
    assert_eq!(
        report["recommendedValidationPolicy"]["writePlanningIntegration"],
        false
    );
    assert_eq!(
        report["recommendedValidationPolicy"]["applyFlowIntegration"],
        false
    );
    assert_eq!(
        report["recommendedValidationPolicy"]["requiredPreflight"],
        false
    );
    assert_eq!(
        report["recommendedValidationPolicy"]["productionValidator"],
        false
    );
    assert_eq!(report["requiredPreflight"], false);
    assert_eq!(report["writeBlocking"], false);
    assert_eq!(report["hyprlandRuntimeSafetyClaimed"], false);

    Ok(())
}

#[test]
fn screen_shader_advisory_integration_design_defines_user_consent_and_temp_copy_boundary(
) -> Result<()> {
    let report =
        read_json("data/reports/screen-shader-advisory-compiler-integration-design.v0.55.2.json")?;

    assert_eq!(report["userConsentRequiredBeforeShaderRead"], true);
    assert_eq!(report["backgroundShaderScanningAllowed"], false);
    assert_eq!(report["originalUserPathPassedToCompiler"], false);
    assert_eq!(report["tempCopyRequired"], true);
    assert_eq!(report["writesOutsideTempDirAllowed"], false);
    assert_eq!(
        report["realUserShaderFilesReadByDesign"],
        "only-after-explicit-user-action-and-only-for-the-user-selected-shader-file-in-a-future-implementation"
    );
    assert_eq!(
        report["futureAdvisoryFlowDesign"]["readConfigPathAutomatically"],
        false
    );
    assert_eq!(
        report["futureAdvisoryFlowDesign"]["backgroundScanningAllowed"],
        false
    );
    assert_eq!(
        report["futureAdvisoryFlowDesign"]["compilerCommandShape"],
        "glslangValidator -l <temp vertex> <temp fragment>"
    );
    assert_eq!(
        report["futureAdvisoryFlowDesign"]["resultCannotBypassProductionGate"],
        true
    );
    assert_eq!(
        report["futureAdvisoryFlowDesign"]["resultCannotApproveWrite"],
        true
    );
    assert_eq!(
        report["futureAdvisoryFlowDesign"]["resultCannotBlockWrite"],
        true
    );
    assert_eq!(
        report["futureAdvisoryFlowDesign"]["resultCannotBecomeRequiredPreflight"],
        true
    );

    Ok(())
}

#[test]
fn screen_shader_advisory_integration_design_defines_non_blocking_failure_policies() -> Result<()> {
    let report =
        read_json("data/reports/screen-shader-advisory-compiler-integration-design.v0.55.2.json")?;

    assert_eq!(
        report["missingToolPolicy"]["policy"],
        "advisory-unavailable-non-blocking"
    );
    assert_eq!(
        report["timeoutPolicy"]["policy"],
        "advisory-inconclusive-non-blocking"
    );
    assert_eq!(
        report["advisoryFailurePolicy"]["policy"],
        "warning-only-not-write-blocking"
    );
    assert_eq!(
        report["advisorySuccessPolicy"]["policy"],
        "advisory-pass-not-runtime-safety-proof"
    );
    assert_eq!(report["cleanupPolicyDefined"]["defined"], true);
    assert!(report["cleanupPolicyDefined"]["policy"]
        .as_str()
        .unwrap()
        .contains("must not approve, block, or bypass any write"));

    Ok(())
}

#[test]
fn screen_shader_advisory_integration_design_records_source_backed_pairing_and_pipeline_link(
) -> Result<()> {
    let report =
        read_json("data/reports/screen-shader-advisory-compiler-integration-design.v0.55.2.json")?;
    let pipeline = read_json("data/reports/all-341-unified-pipeline.v0.55.2.json")?;

    assert_eq!(
        report["sourceBackedVertexPairingRule"]["function"],
        "CHyprOpenGLImpl::applyScreenShader"
    );
    assert_eq!(
        report["sourceBackedVertexPairingRule"]["matchesHyprlandV0552Source"],
        true
    );
    assert!(report["sourceBackedVertexPairingRule"]["rule"]
        .as_str()
        .unwrap()
        .contains("#version 320 es"));
    assert_eq!(report["glslcChosen"], false);
    assert!(report["glslcRejectionReason"]
        .as_str()
        .unwrap()
        .contains("SPIR-V-oriented invocation"));
    assert_eq!(
        report["recommendedValidationPolicy"]["policy"],
        "optional-advanced-advisory-only"
    );

    let gaps = report["compatibilityGaps"]
        .as_array()
        .expect("compatibility gaps should be explicit");
    assert!(!gaps.is_empty());
    assert!(gaps.iter().any(|gap| gap
        .as_str()
        .unwrap()
        .contains("not Hyprland's live OpenGL driver")));

    let missing = report["proofStillMissing"]
        .as_array()
        .expect("missing proof should be explicit");
    assert!(missing.iter().all(|gap| !gap
        .as_str()
        .unwrap()
        .contains("No implementation proof exists")));
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
    assert_eq!(screen_shader_row["currentWriteStatus"], "writable");
    assert_eq!(
        screen_shader_row["gateStatus"],
        "production-screen-shader-gate-enforced-compile-aware-validation-deferred"
    );
    assert_eq!(
        screen_shader_row["advisoryCompilerIntegrationDesignSource"],
        "screen-shader-advisory-compiler-integration-design.v0.55.2.json"
    );
    assert_eq!(
        screen_shader_row["advisoryCompilerImplementationProofSource"],
        "screen-shader-advisory-compiler-implementation-proof.v0.55.2.json"
    );
    assert!(screen_shader_row["nextRequiredWork"]
        .as_str()
        .unwrap()
        .contains("Next high-risk bucket readiness"));
    assert_eq!(screen_shader_row["productionGateEnforcedThisSprint"], true);
    assert_eq!(screen_shader_row["countedAsEnabledHighRiskRow"], false);

    Ok(())
}
