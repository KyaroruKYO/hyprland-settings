use std::path::PathBuf;

use anyhow::Result;
use hyprland_settings::config_parser::parse_hyprland_config_text;
use hyprland_settings::current_config::CurrentConfigSnapshot;
use hyprland_settings::write_classification::{
    is_safe_writable_setting, safe_writable_value_kind, ScalarWriteValueKind, SAFE_WRITABLE_ROWS,
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

    assert_eq!(SAFE_WRITABLE_ROWS.len(), 278);
    assert_eq!(coverage["counts"]["writableRows"], 278);
    assert_eq!(coverage["counts"]["blockedWriteRows"], 63);
    assert!(is_safe_writable_setting("decoration.screen_shader"));
    assert_eq!(
        safe_writable_value_kind("decoration.screen_shader"),
        Some(ScalarWriteValueKind::Path)
    );

    for report in [&review, &policy, &boundary, &mapping, &next_step] {
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
    assert!(next_step["stoppingPoint"]
        .as_str()
        .unwrap()
        .contains("Do not continue"));

    Ok(())
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
        assert_eq!(follow_up["writeAllowlistChanged"], false, "{path}");
        assert_eq!(follow_up["rowsEnabledThisSprint"], 0, "{path}");
        assert_eq!(follow_up["liveShaderCompileUsed"], false, "{path}");
        assert!(follow_up["recommendedNextSprint"]
            .as_str()
            .unwrap()
            .contains("Display/render high-risk gate migration"));
    }

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
    assert!(summary.contains("config-relative shader files"));
    assert!(summary.contains("final screen fragment shader"));
    assert!(summary.contains("display/render high-risk sprint"));
}
