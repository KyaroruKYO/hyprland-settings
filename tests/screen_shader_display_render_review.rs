use std::path::PathBuf;

use anyhow::Result;
use hyprland_settings::config_parser::parse_hyprland_config_text;
use hyprland_settings::current_config::CurrentConfigSnapshot;
use hyprland_settings::write_classification::{
    high_risk_write_policy, is_safe_writable_setting, safe_writable_value_kind,
    ScalarWriteValueKind, SAFE_WRITABLE_ROWS,
};
use hyprland_settings::write_flow::pending_projection_for_value;
use serde_json::Value;

fn read_json(path: &str) -> Result<Value> {
    Ok(serde_json::from_str(&std::fs::read_to_string(path)?)?)
}

fn snapshot_for(path: &PathBuf, contents: &str) -> CurrentConfigSnapshot {
    CurrentConfigSnapshot::from_parsed(parse_hyprland_config_text(path, contents))
}

fn assert_review_safety(report: &Value) {
    assert_eq!(report["counts"]["rowsEnabledThisSprint"], 0);
    assert_eq!(report["counts"]["writeAllowlistChanged"], false);
    assert_eq!(report["counts"]["safeWritableRowsChanged"], false);
    assert_eq!(report["counts"]["recoveryGatesChanged"], false);
    assert_eq!(report["counts"]["realConfigModified"], false);
    assert_eq!(report["counts"]["activeRuntimeModified"], false);
    assert_eq!(report["counts"]["reloadEvalLuaUsed"], false);
    assert_eq!(report["counts"]["liveShaderCompileUsed"], false);
}

#[test]
fn screen_shader_review_reports_exist_and_preserve_current_state() -> Result<()> {
    let coverage = read_json("data/reports/scalar-read-write-coverage.v0.55.2.json")?;
    let review = read_json("data/reports/screen-shader-display-render-review.v0.55.2.json")?;
    let policy = read_json("data/reports/screen-shader-write-policy-decision.v0.55.2.json")?;
    let boundary = read_json("data/reports/screen-shader-validation-boundary.v0.55.2.json")?;
    let mapping = read_json("data/reports/screen-shader-high-risk-template-mapping.v0.55.2.json")?;
    let next_step = read_json("data/reports/screen-shader-next-step-plan.v0.55.2.json")?;
    let migration = read_json("data/reports/screen-shader-high-risk-gate-migration.v0.55.2.json")?;

    assert_eq!(SAFE_WRITABLE_ROWS.len(), 278);
    assert_eq!(coverage["counts"]["writableRows"], 278);
    assert_eq!(coverage["counts"]["blockedWriteRows"], 63);
    assert!(is_safe_writable_setting("decoration.screen_shader"));
    assert_eq!(
        safe_writable_value_kind("decoration.screen_shader"),
        Some(ScalarWriteValueKind::Path)
    );

    for report in [
        &review, &policy, &boundary, &mapping, &next_step, &migration,
    ] {
        assert_review_safety(report);
    }

    assert_eq!(review["counts"]["screenShaderReadable"], true);
    assert_eq!(review["counts"]["screenShaderWritable"], true);
    assert_eq!(review["row"]["rowId"], "decoration.screen_shader");
    assert_eq!(review["row"]["currentWriteStatus"], "writable");
    assert_eq!(policy["selectedPolicy"], "Policy D");
    assert_eq!(policy["counts"]["writableRows"], 278);
    assert_eq!(policy["counts"]["blockedRows"], 63);
    assert_eq!(mapping["counts"]["rowsMapped"], 1);
    assert_eq!(mapping["counts"]["gateAppliedThisSprint"], false);
    assert_eq!(migration["selectedMigrationOption"], "Option A");
    assert_eq!(migration["row"]["currentWritableStatus"], "writable");
    assert_eq!(migration["row"]["writeAllowlistChanged"], false);
    assert_eq!(
        migration["requiredHighRiskGateMetadata"]["recoveryBucket"],
        "display-render-recovery:screen-shader-gate-migration-design"
    );
    assert_eq!(migration["proofExists"]["screenShaderWatchdogProof"], true);
    assert!(migration["proofStillMissing"]
        .as_array()
        .unwrap()
        .iter()
        .any(|item| item
            .as_str()
            .unwrap()
            .contains("production high-risk gate enforcement decision")));
    assert!(next_step["stoppingPoint"]
        .as_str()
        .unwrap()
        .contains("Do not continue"));

    Ok(())
}

#[test]
fn screen_shader_policy_metadata_marks_future_display_render_gate_requirement() {
    let policy = high_risk_write_policy("decoration.screen_shader")
        .expect("screen shader should have migration policy metadata");

    assert_eq!(
        policy.recovery_bucket,
        "display-render-recovery:screen-shader-gate-migration-design"
    );
    assert!(policy.approval_gate.contains("display-render"));
    assert!(policy.approval_gate.contains("watchdog-required"));
    assert!(policy.watchdog_requirement.contains("proof is required"));
    assert!(policy
        .watchdog_requirement
        .contains("separate process confirm"));
    assert!(policy.watchdog_requirement.contains("timeout restore"));
    assert!(policy.review_warning.contains("Display/render sensitive"));
    assert!(policy
        .review_warning
        .contains("Path validation is not display/render safety proof"));
}

#[test]
fn screen_shader_source_behavior_and_validation_boundary_are_recorded() -> Result<()> {
    let review = read_json("data/reports/screen-shader-display-render-review.v0.55.2.json")?;
    let boundary = read_json("data/reports/screen-shader-validation-boundary.v0.55.2.json")?;

    let behavior = &review["sourceBehavior"];
    assert_eq!(behavior["emptyStringDisablesShader"], true);
    assert_eq!(behavior["emptySentinelDisablesShader"], true);
    assert_eq!(behavior["nonEmptyPathIsConfigRelative"], true);
    assert_eq!(behavior["fileIsRead"], true);
    assert_eq!(behavior["fileIsCompiledAsFinalScreenFragmentShader"], true);
    assert_eq!(behavior["couldAffectDisplayOutput"], true);

    let allowed_now = boundary["allowedNow"].as_array().expect("allowedNow array");
    assert!(allowed_now
        .iter()
        .any(|item| item.as_str() == Some("line-safe path validation")));
    assert!(allowed_now
        .iter()
        .any(|item| item.as_str() == Some("report/UI warning metadata")));

    let deferred = boundary["deferredUnlessApprovedLater"]
        .as_array()
        .expect("deferredUnlessApprovedLater array");
    for item in [
        "shader compile validation",
        "GPU/render validation",
        "live display/runtime proof",
        "reload or runtime apply",
        "real config mutation",
    ] {
        assert!(
            deferred.iter().any(|entry| entry.as_str() == Some(item)),
            "{item} should be deferred"
        );
    }

    Ok(())
}

#[test]
fn screen_shader_policy_followup_is_projected_in_aggregate_reports() -> Result<()> {
    for path in [
        "data/reports/all-341-unified-pipeline.v0.55.2.json",
        "data/reports/writable-value-type-evidence-matrix.v0.55.2.json",
        "data/reports/writable-value-type-gap-summary.v0.55.2.json",
        "data/reports/deferred-validator-remaining-items.v0.55.2.json",
        "data/reports/next-high-risk-bucket-readiness.v0.55.2.json",
    ] {
        let report = read_json(path)?;
        let follow_up = &report["screenShaderDisplayRenderReviewFollowUp"];
        assert_eq!(follow_up["rowId"], "decoration.screen_shader", "{path}");
        assert_eq!(follow_up["currentWriteStatus"], "writable", "{path}");
        assert_eq!(follow_up["selectedPolicy"], "Policy D", "{path}");
        assert_eq!(follow_up["selectedMigrationOption"], "Option A", "{path}");
        assert_eq!(
            follow_up["recoveryBucket"],
            "display-render-recovery:screen-shader-gate-migration-design",
            "{path}"
        );
        assert_eq!(follow_up["writeAllowlistChanged"], false, "{path}");
        assert_eq!(follow_up["rowsEnabledThisSprint"], 0, "{path}");
        assert_eq!(follow_up["liveShaderCompileUsed"], false, "{path}");
        assert_eq!(
            follow_up["watchdogProofSource"], "screen-shader-watchdog-migration-proof.v0.55.2.json",
            "{path}"
        );
        assert_eq!(
            follow_up["watchdogMigrationProofStatus"], "complete",
            "{path}"
        );
        assert_eq!(follow_up["productionEnforcementChanged"], false, "{path}");
        assert_eq!(
            follow_up["productionGateEnforcedThisSprint"], false,
            "{path}"
        );
        assert_eq!(follow_up["countedAsEnabledHighRiskRow"], false, "{path}");
        assert_eq!(
            follow_up["compileAwareValidationStatus"], "deferred",
            "{path}"
        );
        assert!(follow_up["previousRecommendedNextSprint"]
            .as_str()
            .unwrap()
            .contains("production gate enforcement decision"));
        assert!(follow_up["recommendedNextSprint"]
            .as_str()
            .unwrap()
            .contains("advisory compiler integration design"));
    }

    Ok(())
}

#[test]
fn screen_shader_unified_pipeline_row_records_production_gate_enforced() -> Result<()> {
    let pipeline = read_json("data/reports/all-341-unified-pipeline.v0.55.2.json")?;
    let row = pipeline["rows"]
        .as_array()
        .unwrap()
        .iter()
        .find(|row| row["rowId"] == "decoration.screen_shader")
        .expect("screen shader row should exist");

    assert_eq!(row["currentWriteStatus"], "writable");
    assert_eq!(row["safeWriteSupported"], true);
    assert_eq!(
        row["riskClass"],
        "display_render_screen_shader_compile_sensitive"
    );
    assert_eq!(
        row["pipelineTemplate"],
        "display-render-screen-shader-watchdog-template"
    );
    assert_eq!(
        row["recoveryBucket"],
        "display-render-recovery:screen-shader-gate-migration-design"
    );
    assert_eq!(
        row["gateStatus"],
        "production-screen-shader-gate-enforced-compile-aware-validation-deferred"
    );
    assert_eq!(
        row["watchdogProofSource"],
        "screen-shader-watchdog-migration-proof.v0.55.2.json"
    );
    assert_eq!(row["productionGateEnforcedThisSprint"], true);
    assert_eq!(row["countedAsEnabledHighRiskRow"], false);
    assert!(row["uiReviewWarning"]
        .as_str()
        .unwrap()
        .contains("production apply requires the screen-shader high-risk watchdog gate"));
    assert_eq!(row["writeAllowlistChanged"], false);
    assert_eq!(row["productionBehaviorChanged"], true);

    Ok(())
}

#[test]
fn screen_shader_write_review_projects_display_render_warning() {
    let path = PathBuf::from("/tmp/screen-shader-review.conf");
    let snapshot = snapshot_for(
        &path,
        "decoration:screen_shader = ./old-screen-shader.frag\n",
    );
    let current = snapshot.value_for("decoration.screen_shader");
    let projection =
        pending_projection_for_value("decoration.screen_shader", &current, "./new.frag");
    let summary = projection.review_summary.join("\n");

    assert!(projection.can_review);
    assert!(summary.contains("Display/render sensitive"));
    assert!(summary.contains("display-render-recovery:screen-shader-gate-migration-design"));
    assert!(summary.contains("watchdog proof is required"));
    assert!(summary.contains("config-relative shader files"));
    assert!(summary.contains("final screen fragment shader"));
    assert!(summary.contains("future display/render watchdog migration proof"));
}
