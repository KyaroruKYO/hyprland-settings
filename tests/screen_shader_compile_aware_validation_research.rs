use anyhow::Result;
use hyprland_settings::write_classification::{is_safe_writable_setting, SAFE_WRITABLE_ROWS};
use serde_json::Value;

fn read_json(path: &str) -> Result<Value> {
    Ok(serde_json::from_str(&std::fs::read_to_string(path)?)?)
}

#[test]
fn screen_shader_compile_aware_research_report_records_advisory_outcome() -> Result<()> {
    let report =
        read_json("data/reports/screen-shader-compile-aware-validation-research.v0.55.2.json")?;
    let coverage = read_json("data/reports/scalar-read-write-coverage.v0.55.2.json")?;
    let approval = read_json("data/reports/screen-shader-production-gate-approval.v0.55.2.json")?;
    let pipeline = read_json("data/reports/all-341-unified-pipeline.v0.55.2.json")?;

    assert!(is_safe_writable_setting("decoration.screen_shader"));
    assert_eq!(SAFE_WRITABLE_ROWS.len(), 278);
    assert_eq!(coverage["counts"]["readableRows"], 341);
    assert_eq!(coverage["counts"]["writableRows"], 278);
    assert_eq!(coverage["counts"]["blockedWriteRows"], 63);

    assert_eq!(report["rowId"], "decoration.screen_shader");
    assert_eq!(report["officialSetting"], "decoration.screen_shader");
    assert_eq!(report["startingCommit"], "1791924");
    assert_eq!(report["selectedResearchOption"], "Option C");
    assert_eq!(report["currentWritableStatus"], "writable");
    assert_eq!(report["productionGateEnforced"], true);
    assert_eq!(
        report["productionGateProofSource"],
        "screen-shader-production-gate-approval.v0.55.2.json"
    );
    assert_eq!(report["watchdogMigrationProofStatus"], "complete");
    assert_eq!(report["compileAwareValidationCurrentStatus"], "deferred");
    assert_eq!(report["compileAwareValidationChanged"], false);
    assert_eq!(report["compileAwareValidationImplemented"], false);
    assert_eq!(report["shaderCompilationRun"], false);
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
    assert_eq!(report["nonLiveValidationDesignable"], true);
    assert_eq!(
        report["recommendedValidationPolicy"]["policy"],
        "advisory-or-research-only"
    );
    assert_eq!(
        report["recommendedValidationPolicy"]["notRequiredPreflight"],
        true
    );
    assert_eq!(report["futureImplementationAllowedThisSprint"], false);
    assert_eq!(approval["productionGateEnforcedThisSprint"], true);
    assert_eq!(approval["compileAwareValidationStatus"], "deferred");

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
    assert_eq!(
        screen_shader_row["compileAwareResearchSource"],
        "screen-shader-compile-aware-validation-research.v0.55.2.json"
    );
    assert!(screen_shader_row["nextRequiredWork"]
        .as_str()
        .unwrap()
        .contains("optional advisory UI exposure design"));

    Ok(())
}

#[test]
fn screen_shader_compile_aware_research_records_official_source_evidence() -> Result<()> {
    let report =
        read_json("data/reports/screen-shader-compile-aware-validation-research.v0.55.2.json")?;
    let sources = report["officialHyprlandSourcesInspected"]
        .as_array()
        .expect("official source evidence should be an array");
    let source_paths = sources
        .iter()
        .filter_map(|source| source["path"].as_str())
        .collect::<Vec<_>>();

    for expected in [
        "/tmp/Hyprland-v0.55.2-full/src/config/values/ConfigValues.cpp",
        "/tmp/Hyprland-v0.55.2-full/src/render/OpenGL.cpp",
        "/tmp/Hyprland-v0.55.2-full/src/render/Shader.cpp",
        "/tmp/Hyprland-v0.55.2-full/src/render/ShaderLoader.cpp",
        "/tmp/Hyprland-v0.55.2-full/src/render/shaders/glsl/tex300.vert",
        "/tmp/Hyprland-v0.55.2-full/src/render/shaders/glsl/tex320.vert",
    ] {
        assert!(
            source_paths.contains(&expected),
            "missing official source evidence for {expected}"
        );
    }

    assert_eq!(
        report["hyprlandGeneratedWrapperOrPreludeEvidence"]["fragmentWrapperFound"],
        false
    );
    assert_eq!(
        report["hyprlandGeneratedWrapperOrPreludeEvidence"]["fragmentPreludeFound"],
        false
    );
    assert_eq!(
        report["runtimeCompilePathEvidence"]["backendPath"],
        "OpenGL renderer path in CHyprOpenGLImpl::applyScreenShader"
    );
    assert_eq!(report["runtimeCompilePathEvidence"]["dynamicCompile"], true);
    assert_eq!(
        report["runtimeCompilePathEvidence"]["otherBackendCompilePathProven"],
        false
    );

    Ok(())
}

#[test]
fn screen_shader_compile_aware_research_records_remaining_gaps_and_next_step() -> Result<()> {
    let report =
        read_json("data/reports/screen-shader-compile-aware-validation-research.v0.55.2.json")?;
    let gaps = report["standaloneCompilerCompatibilityGaps"]
        .as_array()
        .expect("compatibility gaps should be recorded");
    let missing = report["proofStillMissing"]
        .as_array()
        .expect("missing proof should be recorded");

    assert!(!gaps.is_empty());
    assert!(gaps
        .iter()
        .any(|gap| gap.as_str().unwrap().contains("same live OpenGL context")));
    assert!(missing.iter().any(|gap| gap
        .as_str()
        .unwrap()
        .contains("No compatibility proof exists")));
    assert!(report["nextRecommendedSprint"]
        .as_str()
        .unwrap()
        .contains("fixture/temp shader files only"));

    for candidate in report["standaloneCompilerCandidates"]
        .as_array()
        .expect("standalone compiler candidates should be recorded")
    {
        assert_eq!(candidate["usedThisSprint"], false);
    }

    Ok(())
}
