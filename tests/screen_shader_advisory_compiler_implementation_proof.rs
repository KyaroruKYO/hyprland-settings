use anyhow::Result;
use hyprland_settings::write_classification::{is_safe_writable_setting, SAFE_WRITABLE_ROWS};
use serde_json::Value;

fn read_json(path: &str) -> Result<Value> {
    Ok(serde_json::from_str(&std::fs::read_to_string(path)?)?)
}

#[test]
fn screen_shader_advisory_helper_implementation_report_records_option_a() -> Result<()> {
    let report = read_json(
        "data/reports/screen-shader-advisory-compiler-implementation-proof.v0.55.2.json",
    )?;
    let coverage = read_json("data/reports/scalar-read-write-coverage.v0.55.2.json")?;
    let feasibility = read_json(
        "data/reports/screen-shader-non-live-advisory-compiler-feasibility.v0.55.2.json",
    )?;
    let design =
        read_json("data/reports/screen-shader-advisory-compiler-integration-design.v0.55.2.json")?;
    let approval = read_json("data/reports/screen-shader-production-gate-approval.v0.55.2.json")?;

    assert!(is_safe_writable_setting("decoration.screen_shader"));
    assert_eq!(SAFE_WRITABLE_ROWS.len(), 278);
    assert_eq!(coverage["counts"]["readableRows"], 341);
    assert_eq!(coverage["counts"]["writableRows"], 278);
    assert_eq!(coverage["counts"]["blockedWriteRows"], 63);

    assert_eq!(report["rowId"], "decoration.screen_shader");
    assert_eq!(report["officialSetting"], "decoration.screen_shader");
    assert_eq!(report["startingCommit"], "aea7586");
    assert_eq!(report["selectedImplementationProofOption"], "Option A");
    assert_eq!(report["currentWritableStatus"], "writable");
    assert_eq!(report["productionGateEnforced"], true);
    assert_eq!(report["productionGateChanged"], false);
    assert_eq!(report["watchdogMigrationProofStatus"], "complete");
    assert_eq!(report["advisoryCompilerFeasibilityStatus"], "complete");
    assert_eq!(
        report["advisoryCompilerIntegrationDesignStatus"],
        "complete"
    );
    assert_eq!(report["chosenAdvisoryTool"], "glslangValidator");
    assert_eq!(report["advisoryHelperImplemented"], true);
    assert_eq!(
        report["advisoryHelperModule"],
        "src/screen_shader_advisory.rs"
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
    assert_eq!(feasibility["selectedFeasibilityOption"], "Option A");
    assert_eq!(design["selectedIntegrationDesignOption"], "Option A");
    assert_eq!(approval["productionGateEnforcedThisSprint"], true);

    Ok(())
}

#[test]
fn screen_shader_advisory_helper_is_not_wired_into_write_safety() -> Result<()> {
    let report = read_json(
        "data/reports/screen-shader-advisory-compiler-implementation-proof.v0.55.2.json",
    )?;

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
fn screen_shader_advisory_helper_records_safety_boundaries() -> Result<()> {
    let report = read_json(
        "data/reports/screen-shader-advisory-compiler-implementation-proof.v0.55.2.json",
    )?;

    assert_eq!(report["shaderCompilationThroughHyprlandRun"], false);
    assert_eq!(report["standaloneCompilerCommandsRunThisSprint"], true);
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
    assert_eq!(
        report["realUserShaderFilesReadByHelperWithoutConsent"],
        false
    );
    assert_eq!(report["explicitUserConsentRequired"], true);
    assert_eq!(report["backgroundShaderScanningAllowed"], false);
    assert_eq!(report["originalUserPathPassedToCompiler"], false);
    assert_eq!(report["tempCopyRequired"], true);
    assert_eq!(report["writesOutsideTempDirAllowed"], false);

    Ok(())
}

#[test]
fn screen_shader_advisory_helper_records_result_behaviors() -> Result<()> {
    let report = read_json(
        "data/reports/screen-shader-advisory-compiler-implementation-proof.v0.55.2.json",
    )?;

    assert_eq!(report["missingToolBehaviorProven"], true);
    assert_eq!(report["timeoutBehaviorProven"], true);
    assert_eq!(report["advisoryPassBehaviorProven"], true);
    assert_eq!(report["advisoryFailBehaviorProven"], true);
    assert_eq!(report["cleanupFailureBehaviorProven"], true);
    assert_eq!(
        report["helperBehaviorProof"]["explicitUserConsentMissingStatus"],
        "missing_consent"
    );
    assert_eq!(
        report["helperBehaviorProof"]["missingToolStatus"],
        "unavailable"
    );
    assert_eq!(report["helperBehaviorProof"]["timeoutStatus"], "timed_out");
    assert_eq!(
        report["helperBehaviorProof"]["advisorySuccessStatus"],
        "passed"
    );
    assert_eq!(
        report["helperBehaviorProof"]["advisoryFailureStatus"],
        "failed"
    );
    assert_eq!(
        report["helperBehaviorProof"]["cleanupFailureStatus"],
        "cleanup_warning"
    );
    assert_eq!(
        report["helperBehaviorProof"]["originalUserPathPassedToCompiler"],
        false
    );
    assert_eq!(
        report["helperBehaviorProof"]["productionWriteDecisionChanged"],
        false
    );
    assert_eq!(report["helperBehaviorProof"]["runtimeSafetyClaimed"], false);
    assert_eq!(report["helperBehaviorProof"]["writeBlocking"], false);

    Ok(())
}

#[test]
fn screen_shader_advisory_helper_records_source_pairing_and_pipeline_link() -> Result<()> {
    let report = read_json(
        "data/reports/screen-shader-advisory-compiler-implementation-proof.v0.55.2.json",
    )?;
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
    assert_eq!(
        report["recommendedValidationPolicy"]["policy"],
        "optional-non-production-advisory-helper-only"
    );

    let gaps = report["compatibilityGaps"]
        .as_array()
        .expect("compatibility gaps should be explicit");
    assert!(!gaps.is_empty());
    assert!(gaps
        .iter()
        .any(|gap| gap.as_str().unwrap().contains("not Hyprland's live OpenGL")));

    let missing = report["proofStillMissing"]
        .as_array()
        .expect("missing proof should be explicit");
    assert!(missing
        .iter()
        .any(|gap| gap.as_str().unwrap().contains("No UI proof exists")));
    assert!(report["nextRecommendedSprint"]
        .as_str()
        .unwrap()
        .contains("optional advisory UI exposure design"));

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
        screen_shader_row["advisoryCompilerImplementationProofSource"],
        "screen-shader-advisory-compiler-implementation-proof.v0.55.2.json"
    );
    assert!(screen_shader_row["nextRequiredWork"]
        .as_str()
        .unwrap()
        .contains("optional advisory UI exposure design"));
    assert_eq!(screen_shader_row["productionGateEnforcedThisSprint"], true);
    assert_eq!(screen_shader_row["countedAsEnabledHighRiskRow"], false);

    Ok(())
}
