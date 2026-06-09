use anyhow::Result;
use hyprland_settings::write_classification::{is_safe_writable_setting, SAFE_WRITABLE_ROWS};
use serde_json::Value;

fn read_json(path: &str) -> Result<Value> {
    Ok(serde_json::from_str(&std::fs::read_to_string(path)?)?)
}

#[test]
fn screen_shader_advisory_compiler_feasibility_report_records_option_a() -> Result<()> {
    let report = read_json(
        "data/reports/screen-shader-non-live-advisory-compiler-feasibility.v0.55.2.json",
    )?;
    let coverage = read_json("data/reports/scalar-read-write-coverage.v0.55.2.json")?;
    let approval = read_json("data/reports/screen-shader-production-gate-approval.v0.55.2.json")?;
    let compile_research =
        read_json("data/reports/screen-shader-compile-aware-validation-research.v0.55.2.json")?;

    assert!(is_safe_writable_setting("decoration.screen_shader"));
    assert_eq!(SAFE_WRITABLE_ROWS.len(), 341);
    assert_eq!(coverage["counts"]["readableRows"], 341);
    assert_eq!(coverage["counts"]["writableRows"], 341);
    assert_eq!(coverage["counts"]["blockedWriteRows"], 0);

    assert_eq!(report["rowId"], "decoration.screen_shader");
    assert_eq!(report["officialSetting"], "decoration.screen_shader");
    assert_eq!(report["startingCommit"], "df2626a");
    assert_eq!(report["selectedFeasibilityOption"], "Option A");
    assert_eq!(report["currentWritableStatus"], "writable");
    assert_eq!(report["productionGateEnforced"], true);
    assert_eq!(report["productionGateChanged"], false);
    assert_eq!(report["watchdogMigrationProofStatus"], "complete");
    assert_eq!(report["compileAwareValidationCurrentStatus"], "deferred");
    assert_eq!(report["compileAwareValidationChanged"], false);
    assert_eq!(report["compileAwareValidationImplemented"], false);
    assert_eq!(report["productionCompileAwareValidationImplemented"], false);
    assert_eq!(report["shaderCompilationThroughHyprlandRun"], false);
    assert_eq!(report["liveShaderCompileUsed"], false);
    assert_eq!(report["liveDisplayRuntimeProofUsed"], false);
    assert_eq!(report["realConfigTouched"], false);
    assert_eq!(report["runtimeTouched"], false);
    assert_eq!(report["reloadEvalLuaUsed"], false);
    assert_eq!(report["safeWritableRowsChanged"], false);
    assert_eq!(report["writeAllowlistChanged"], false);
    assert_eq!(report["rowsEnabledThisSprint"], 0);
    assert_eq!(report["readableRows"], 341);
    assert_eq!(report["writableRows"], 278);
    assert_eq!(report["blockedRows"], 63);
    assert_eq!(approval["productionGateEnforcedThisSprint"], true);
    assert_eq!(approval["compileAwareValidationStatus"], "deferred");
    assert_eq!(
        compile_research["compileAwareValidationCurrentStatus"],
        "deferred"
    );
    assert_eq!(compile_research["compileAwareValidationImplemented"], false);

    Ok(())
}

#[test]
fn screen_shader_advisory_compiler_feasibility_records_fixture_only_tool_proof() -> Result<()> {
    let report = read_json(
        "data/reports/screen-shader-non-live-advisory-compiler-feasibility.v0.55.2.json",
    )?;

    assert_eq!(report["chosenAdvisoryTool"], "glslangValidator");
    assert_eq!(report["fixtureTempOnly"], true);
    assert_eq!(report["fixtureGoodShaderAccepted"], true);
    assert_eq!(report["fixtureInvalidShaderRejected"], true);
    assert_eq!(report["fixtureUsesHyprlandVertexPairing"], true);
    assert_eq!(report["realUserShaderFilesRead"], false);
    assert_eq!(report["writesOutsideTempDir"], false);
    assert_eq!(report["hyprlandRuntimeRequired"], false);
    assert_eq!(report["displayRuntimeRequired"], false);

    let tools = report["candidateToolsChecked"]
        .as_array()
        .expect("candidate tool availability should be recorded");
    assert!(tools.iter().any(|tool| {
        tool["tool"] == "glslangValidator"
            && tool["available"] == true
            && tool["selectedForAdvisoryResearch"] == true
    }));
    assert!(tools.iter().any(|tool| {
        tool["tool"] == "glslc"
            && tool["available"] == true
            && tool["selectedForAdvisoryResearch"] == false
    }));

    let commands = report["standaloneCompilerCommandsRun"]
        .as_array()
        .expect("standalone compiler command proof should be recorded");
    assert!(commands.iter().any(|case| {
        case["tool"] == "glslangValidator"
            && case["result"] == "accepted-known-good-es300-fixture"
            && case["exitCode"] == 0
    }));
    assert!(commands.iter().any(|case| {
        case["tool"] == "glslangValidator"
            && case["result"] == "rejected-intentionally-invalid-es300-fixture"
            && case["exitCode"] == 2
    }));
    assert!(commands.iter().any(|case| {
        case["tool"] == "glslangValidator"
            && case["result"] == "accepted-known-good-es320-fixture"
            && case["exitCode"] == 0
    }));
    assert!(commands.iter().any(|case| {
        case["tool"] == "glslangValidator"
            && case["result"] == "rejected-intentionally-invalid-es320-fixture"
            && case["exitCode"] == 2
    }));

    Ok(())
}

#[test]
fn screen_shader_advisory_compiler_feasibility_keeps_policy_advisory_only() -> Result<()> {
    let report = read_json(
        "data/reports/screen-shader-non-live-advisory-compiler-feasibility.v0.55.2.json",
    )?;
    let pipeline = read_json("data/reports/all-341-unified-pipeline.v0.55.2.json")?;

    assert_eq!(
        report["recommendedValidationPolicy"]["policy"],
        "optional-advisory-research-only"
    );
    assert_eq!(
        report["recommendedValidationPolicy"]["requiredPreflight"],
        false
    );
    assert_eq!(
        report["recommendedValidationPolicy"]["productionValidator"],
        false
    );
    assert_eq!(
        report["recommendedValidationPolicy"]["writePlanningIntegration"],
        false
    );
    assert_eq!(
        report["recommendedValidationPolicy"]["userShaderFileReadsAllowedByThisSprint"],
        false
    );

    let gaps = report["compatibilityGaps"]
        .as_array()
        .expect("compatibility gaps should be explicit");
    assert!(!gaps.is_empty());
    assert!(gaps.iter().any(|gap| gap
        .as_str()
        .unwrap()
        .contains("not Hyprland's live OpenGL driver")));
    assert!(gaps.iter().any(|gap| gap
        .as_str()
        .unwrap()
        .contains("No official Hyprland non-live screen-shader validation interface")));

    let missing = report["proofStillMissing"]
        .as_array()
        .expect("missing proof should be explicit");
    assert!(!missing.is_empty());
    assert!(missing.iter().any(|gap| gap
        .as_str()
        .unwrap()
        .contains("No production compile-aware validator implementation exists")));
    assert!(report["nextRecommendedSprint"]
        .as_str()
        .unwrap()
        .contains("advisory compiler integration design"));

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
    assert_eq!(screen_shader_row["productionGateEnforcedThisSprint"], true);
    assert_eq!(screen_shader_row["countedAsEnabledHighRiskRow"], false);
    assert_eq!(
        screen_shader_row["advisoryCompilerFeasibilitySource"],
        "screen-shader-non-live-advisory-compiler-feasibility.v0.55.2.json"
    );
    assert!(screen_shader_row["nextRequiredWork"]
        .as_str()
        .unwrap()
        .contains("Next high-risk bucket readiness"));

    Ok(())
}
