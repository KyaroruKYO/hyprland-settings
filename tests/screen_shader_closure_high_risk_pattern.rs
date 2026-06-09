use anyhow::Result;
use hyprland_settings::write_classification::{is_safe_writable_setting, SAFE_WRITABLE_ROWS};
use serde_json::Value;

fn read_json(path: &str) -> Result<Value> {
    Ok(serde_json::from_str(&std::fs::read_to_string(path)?)?)
}

#[test]
fn screen_shader_track_closure_report_closes_shader_track_without_behavior_change() -> Result<()> {
    let closure = read_json("data/reports/screen-shader-track-closure.v0.55.2.json")?;
    let coverage = read_json("data/reports/scalar-read-write-coverage.v0.55.2.json")?;
    let approval = read_json("data/reports/screen-shader-production-gate-approval.v0.55.2.json")?;

    assert!(is_safe_writable_setting("decoration.screen_shader"));
    assert_eq!(SAFE_WRITABLE_ROWS.len(), 341);
    assert_eq!(coverage["counts"]["readableRows"], 341);
    assert_eq!(coverage["counts"]["writableRows"], 341);
    assert_eq!(coverage["counts"]["blockedWriteRows"], 0);

    assert_eq!(closure["rowId"], "decoration.screen_shader");
    assert_eq!(closure["officialSetting"], "decoration.screen_shader");
    assert_eq!(closure["startingCommit"], "2d98e7c");
    assert_eq!(closure["currentWritableStatus"], "writable");
    assert_eq!(closure["productionGateEnforced"], true);
    assert_eq!(approval["productionGateEnforcedThisSprint"], true);
    assert_eq!(closure["watchdogMigrationProofStatus"], "complete");
    assert_eq!(closure["advisoryCompilerFeasibilityStatus"], "complete");
    assert_eq!(
        closure["advisoryCompilerIntegrationDesignStatus"],
        "complete"
    );
    assert_eq!(closure["advisoryHelperImplementationStatus"], "complete");
    assert_eq!(closure["advisoryUiExposureDesignStatus"], "complete");
    assert_eq!(closure["advisoryUiImplementationStatus"], "complete");
    assert_eq!(closure["gtkWidgetWiringStatus"], "complete");
    assert_eq!(closure["selectedFileActionModelStatus"], "complete");
    assert_eq!(closure["compileAwareValidationStatus"], "deferred");
    assert_eq!(closure["directGtkFileChooserVisualProofStatus"], "deferred");
    assert_eq!(
        closure["hyprlandRuntimeSafetyProofStatus"],
        "notProvenDeferred"
    );
    assert_eq!(closure["screenShaderTrackClosedForNow"], true);
    assert_eq!(closure["safeWritableRowsChanged"], false);
    assert_eq!(closure["writeAllowlistChanged"], false);
    assert_eq!(closure["rowsEnabledThisSprint"], 0);
    assert_eq!(closure["readableRows"], 341);
    assert_eq!(closure["writableRows"], 278);
    assert_eq!(closure["blockedRows"], 63);
    assert!(closure["currentDecision"]
        .as_str()
        .unwrap()
        .contains("closed for now"));
    assert!(closure["nextScreenShaderWorkPolicy"]
        .as_str()
        .unwrap()
        .contains("deferred-by-default"));
    assert!(closure["deferredShaderSpecificWork"]
        .as_array()
        .unwrap()
        .iter()
        .any(|item| item.as_str().unwrap().contains("direct GTK file chooser")));
    assert!(closure["deferredShaderSpecificWork"]
        .as_array()
        .unwrap()
        .iter()
        .any(|item| item
            .as_str()
            .unwrap()
            .contains("production compile-aware validation")));
    assert_eq!(
        closure["nextRecommendedSprint"],
        "Next high-risk bucket readiness and batching sprint"
    );

    Ok(())
}

#[test]
fn high_risk_pattern_is_framework_not_automatic_enablement() -> Result<()> {
    let pattern = read_json("data/reports/high-risk-row-pattern-from-screen-shader.v0.55.2.json")?;

    assert_eq!(
        pattern["patternName"],
        "source-backed-row-specific-high-risk-gated-write-pattern"
    );
    assert_eq!(pattern["derivedFromRow"], "decoration.screen_shader");
    assert!(pattern["notUniversalWarning"]
        .as_str()
        .unwrap()
        .contains("not automatic permission to enable rows"));
    assert_eq!(pattern["sourceProofRequired"], true);
    assert_eq!(pattern["validatorProofRequired"], true);
    assert_eq!(pattern["writeSafetyProofRequired"], true);
    assert_eq!(pattern["riskClassificationRequired"], true);
    assert_eq!(pattern["gateRequiredWhenRisky"], true);
    assert_eq!(pattern["watchdogRequiredWhenRisky"], true);
    assert_eq!(pattern["recoveryProofRequiredWhenRisky"], true);
    assert_eq!(pattern["blockedIfProofMissing"], true);
    assert_eq!(pattern["safeWritableRowsChanged"], false);
    assert_eq!(pattern["writeAllowlistChanged"], false);
    assert_eq!(pattern["rowsEnabledThisSprint"], 0);

    let do_not_infer = pattern["doNotInferSafetyFrom"].as_array().unwrap();
    for forbidden in [
        "parser acceptance",
        "HyprMod exposure",
        "UI metadata alone",
        "advisory helper existence alone",
        "standalone compiler output alone",
    ] {
        assert!(do_not_infer
            .iter()
            .any(|item| item.as_str().unwrap() == forbidden));
    }
    assert!(pattern["advisoryChecksPolicy"]
        .as_str()
        .unwrap()
        .contains("do not replace"));
    assert!(pattern["batchingPolicy"]
        .as_str()
        .unwrap()
        .contains("grouped high-risk bucket"));

    Ok(())
}

#[test]
fn return_to_341_roadmap_recommends_grouped_high_risk_planning() -> Result<()> {
    let roadmap = read_json("data/reports/return-to-341-writable-roadmap.v0.55.2.json")?;

    assert!(roadmap["projectGoal"]
        .as_str()
        .unwrap()
        .contains("Make all 341 official Hyprland scalar rows writable where possible"));
    assert_eq!(roadmap["currentCounts"]["readableRows"], 341);
    assert_eq!(roadmap["currentCounts"]["writableRows"], 278);
    assert_eq!(roadmap["currentCounts"]["blockedRows"], 63);
    assert_eq!(
        roadmap["screenShaderStatus"]["rowId"],
        "decoration.screen_shader"
    );
    assert_eq!(
        roadmap["screenShaderStatus"]["currentWritableStatus"],
        "writable"
    );
    assert_eq!(
        roadmap["screenShaderStatus"]["productionGateEnforced"],
        true
    );
    assert_eq!(
        roadmap["screenShaderStatus"]["screenShaderTrackClosedForNow"],
        true
    );
    assert_eq!(
        roadmap["nextWorkMode"],
        "grouped-high-risk-bucket-level-planning"
    );
    assert_eq!(
        roadmap["remainingBlockedBuckets"]["displayRenderBlockedRows"],
        23
    );
    assert_eq!(
        roadmap["remainingBlockedBuckets"]["cursorInputBlockedRows"],
        18
    );
    assert_eq!(
        roadmap["remainingBlockedBuckets"]["debugCrashBlockedRows"],
        22
    );
    assert_eq!(
        roadmap["recommendedNextSprint"],
        "Next high-risk bucket readiness and batching sprint"
    );
    assert!(roadmap["avoidRabbitHolePolicy"]
        .as_str()
        .unwrap()
        .contains("Do not continue one-row deep dives"));
    assert_eq!(roadmap["doNotEnableWithoutApproval"], true);
    assert_eq!(roadmap["safeWritableRowsChanged"], false);
    assert_eq!(roadmap["writeAllowlistChanged"], false);
    assert_eq!(roadmap["rowsEnabledThisSprint"], 0);
    assert_eq!(roadmap["readableRows"], 341);
    assert_eq!(roadmap["writableRows"], 278);
    assert_eq!(roadmap["blockedRows"], 63);

    Ok(())
}

#[test]
fn aggregate_reports_link_closure_pattern_and_roadmap() -> Result<()> {
    let pipeline = read_json("data/reports/all-341-unified-pipeline.v0.55.2.json")?;
    let deferred = read_json("data/reports/deferred-validator-remaining-items.v0.55.2.json")?;
    let readiness = read_json("data/reports/next-high-risk-bucket-readiness.v0.55.2.json")?;
    let evidence = read_json("data/reports/writable-value-type-evidence-matrix.v0.55.2.json")?;
    let gaps = read_json("data/reports/writable-value-type-gap-summary.v0.55.2.json")?;

    let screen_shader_row = pipeline["rows"]
        .as_array()
        .unwrap()
        .iter()
        .find(|row| row["rowId"] == "decoration.screen_shader")
        .expect("screen shader row should exist in all-341 pipeline");

    assert_eq!(
        screen_shader_row["screenShaderTrackClosureSource"],
        "screen-shader-track-closure.v0.55.2.json"
    );
    assert_eq!(
        screen_shader_row["highRiskPatternSource"],
        "high-risk-row-pattern-from-screen-shader.v0.55.2.json"
    );
    assert_eq!(
        screen_shader_row["returnTo341RoadmapSource"],
        "return-to-341-writable-roadmap.v0.55.2.json"
    );
    assert_eq!(screen_shader_row["screenShaderTrackClosedForNow"], true);
    assert!(screen_shader_row["nextRequiredWork"]
        .as_str()
        .unwrap()
        .contains("Display/render blocked rows source evidence inventory sprint"));
    assert!(!screen_shader_row["nextRequiredWork"]
        .as_str()
        .unwrap()
        .contains("screen shader optional advisory GTK file chooser visual proof"));
    assert_eq!(screen_shader_row["productionGateEnforcedThisSprint"], true);
    assert_eq!(screen_shader_row["countedAsEnabledHighRiskRow"], false);
    assert_eq!(pipeline["counts"]["writableRows"], 341);
    assert_eq!(pipeline["counts"]["blockedRows"], 0);

    for report in [&deferred, &readiness, &evidence, &gaps] {
        let follow_up = &report["screenShaderDisplayRenderReviewFollowUp"];
        assert_eq!(follow_up["screenShaderTrackClosedForNow"], true);
        assert_eq!(
            follow_up["screenShaderTrackClosureReport"],
            "data/reports/screen-shader-track-closure.v0.55.2.json"
        );
        assert_eq!(
            follow_up["highRiskPatternReport"],
            "data/reports/high-risk-row-pattern-from-screen-shader.v0.55.2.json"
        );
        assert_eq!(
            follow_up["returnTo341RoadmapReport"],
            "data/reports/return-to-341-writable-roadmap.v0.55.2.json"
        );
        assert_eq!(
            follow_up["recommendedNextSprint"],
            "Display/render blocked rows source evidence inventory sprint"
        );
        assert_eq!(
            follow_up["nextWorkMode"],
            "grouped-high-risk-bucket-level-readiness"
        );
    }

    let deferred_screen_shader = deferred["rows"]
        .as_array()
        .unwrap()
        .iter()
        .find(|row| row["rowId"] == "decoration.screen_shader")
        .expect("screen shader deferred row should remain represented");
    assert!(deferred_screen_shader["nextAction"]
        .as_str()
        .unwrap()
        .contains("deferred by default"));

    Ok(())
}
