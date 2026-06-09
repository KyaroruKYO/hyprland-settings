use std::collections::BTreeSet;
use std::path::PathBuf;

use anyhow::Result;
use hyprland_settings::config_backup::ConfigBackup;
use hyprland_settings::config_parser::parse_hyprland_config_text;
use hyprland_settings::current_config::CurrentConfigSnapshot;
use hyprland_settings::high_risk_recovery::{
    refuse_inert_live_config_execution, validate_inert_live_config_plan, HighRiskRecoveryPlanner,
};
use hyprland_settings::pending_change::stage_pending_change;
use hyprland_settings::write_classification::{
    high_risk_write_policy, is_safe_writable_setting, SAFE_WRITABLE_ROWS,
};
use hyprland_settings::write_safety::{review_write_plan, WritePlanRequest};
use serde_json::Value;

fn read_json(path: &str) -> Result<Value> {
    Ok(serde_json::from_str(&std::fs::read_to_string(path)?)?)
}

fn snapshot_for(path: &PathBuf, contents: &str) -> CurrentConfigSnapshot {
    CurrentConfigSnapshot::from_parsed(parse_hyprland_config_text(path, contents))
}

#[test]
fn screen_shader_production_gate_decision_report_records_option_a() -> Result<()> {
    let coverage = read_json("data/reports/scalar-read-write-coverage.v0.55.2.json")?;
    let decision =
        read_json("data/reports/screen-shader-production-gate-enforcement-decision.v0.55.2.json")?;
    let watchdog = read_json("data/reports/screen-shader-watchdog-migration-proof.v0.55.2.json")?;

    assert!(is_safe_writable_setting("decoration.screen_shader"));
    assert_eq!(SAFE_WRITABLE_ROWS.len(), 340);
    assert_eq!(coverage["counts"]["readableRows"], 341);
    assert_eq!(coverage["counts"]["writableRows"], 340);
    assert_eq!(coverage["counts"]["blockedWriteRows"], 1);

    assert_eq!(decision["rowId"], "decoration.screen_shader");
    assert_eq!(decision["officialSetting"], "decoration.screen_shader");
    assert_eq!(decision["startingCommit"], "4662b86");
    assert_eq!(decision["decisionOption"], "Option A");
    assert_eq!(decision["currentWritableStatus"], "writable");
    assert_eq!(decision["writableMigrationCandidate"], true);
    assert_eq!(decision["watchdogMigrationProofStatus"], "complete");
    assert_eq!(decision["countedAsEnabledHighRiskRow"], false);
    assert_eq!(decision["readableRows"], 341);
    assert_eq!(decision["writableRows"], 278);
    assert_eq!(decision["blockedRows"], 63);
    assert_eq!(decision["safeWritableRowsChanged"], false);
    assert_eq!(decision["writeAllowlistChanged"], false);
    assert_eq!(decision["rowsEnabledThisSprint"], 0);
    assert_eq!(decision["realConfigTouched"], false);
    assert_eq!(decision["runtimeTouched"], false);
    assert_eq!(decision["reloadEvalLuaUsed"], false);
    assert_eq!(decision["liveShaderCompileUsed"], false);
    assert_eq!(decision["liveDisplayRuntimeProofUsed"], false);
    assert_eq!(decision["compileAwareValidationChanged"], false);
    assert_eq!(decision["compileAwareValidationStatus"], "deferred");
    assert_eq!(decision["productionEnforcementChanged"], false);
    assert_eq!(decision["productionGateEnforcedThisSprint"], false);
    assert_eq!(decision["productionWriteFlowChanged"], false);
    assert_eq!(decision["normalPathOnlyApprovalStillAccepted"], true);
    assert_eq!(decision["highRiskGateRequiredForProduction"], false);
    assert_eq!(decision["userApprovalRequiredBeforeRemoval"], true);
    assert!(decision["proofStillMissing"]
        .as_array()
        .unwrap()
        .iter()
        .any(|item| item
            .as_str()
            .unwrap()
            .contains("production write-flow primitive")));
    assert!(decision["nextRecommendedSprint"]
        .as_str()
        .unwrap()
        .contains("production high-risk gate enforcement architecture"));

    assert_eq!(watchdog["watchdogMigrationProofStatus"], "complete");
    assert_eq!(watchdog["productionEnforcementChanged"], false);
    assert_eq!(watchdog["productionGateEnforcedThisSprint"], false);

    Ok(())
}

#[test]
fn screen_shader_remains_migration_candidate_not_enabled_high_risk_row() -> Result<()> {
    let reconciliation =
        read_json("data/reports/high-risk-unified-pipeline-reconciliation.v0.55.2.json")?;
    let pipeline = read_json("data/reports/all-341-unified-pipeline.v0.55.2.json")?;
    let decision =
        read_json("data/reports/screen-shader-production-gate-enforcement-decision.v0.55.2.json")?;

    assert_eq!(reconciliation["counts"]["enabledHighRiskRowsAudited"], 9);
    assert_eq!(reconciliation["counts"]["displayRenderBlockedRows"], 23);
    assert_eq!(reconciliation["counts"]["cursorInputBlockedRows"], 18);
    assert_eq!(reconciliation["counts"]["debugCrashBlockedRows"], 22);
    assert_eq!(pipeline["counts"]["highRiskRowsEnabled"], 71);
    assert_eq!(pipeline["counts"]["writableRows"], 340);
    assert_eq!(pipeline["counts"]["blockedRows"], 1);

    let row = pipeline["rows"]
        .as_array()
        .unwrap()
        .iter()
        .find(|row| row["rowId"] == "decoration.screen_shader")
        .expect("screen shader row exists");
    assert_eq!(row["currentWriteStatus"], "writable");
    assert_eq!(
        row["gateStatus"],
        "production-screen-shader-gate-enforced-compile-aware-validation-deferred"
    );
    assert_eq!(row["productionGateEnforcedThisSprint"], true);
    assert_eq!(row["countedAsEnabledHighRiskRow"], false);
    assert!(row["uiReviewWarning"]
        .as_str()
        .unwrap()
        .contains("production apply requires the screen-shader high-risk watchdog gate"));

    assert_eq!(decision["writableMigrationCandidate"], true);
    assert_eq!(decision["countedAsEnabledHighRiskRow"], false);

    Ok(())
}

#[test]
fn current_fixture_write_review_still_accepts_normal_path_only_approval() -> Result<()> {
    let config_path = PathBuf::from("/tmp/screen-shader-production-decision.conf");
    let backup_path = PathBuf::from("/tmp/screen-shader-production-decision.conf.bak");
    let snapshot = snapshot_for(
        &config_path,
        "decoration:screen_shader = ./old-screen-shader.frag\n",
    );
    let current_value = snapshot.value_for("decoration.screen_shader");
    let pending = stage_pending_change(
        "decoration.screen_shader",
        &current_value,
        "./new-screen-shader.frag",
    );
    let review = review_write_plan(WritePlanRequest {
        known_setting_ids: BTreeSet::from(["decoration.screen_shader".to_string()]),
        detected_config_path: config_path.clone(),
        current_value,
        pending_change: pending,
        backup: Some(ConfigBackup {
            source_path: config_path,
            backup_path,
            byte_len: 64,
        }),
    });

    assert!(
        review.is_approved(),
        "production write review is still normal safe-write review; this proves Option C is not implemented"
    );
    let policy = high_risk_write_policy("decoration.screen_shader")
        .expect("screen shader should still carry high-risk migration metadata");
    assert!(policy.review_warning.contains("Display/render sensitive"));
    assert!(policy
        .review_warning
        .contains("Path validation is not display/render safety proof"));

    Ok(())
}

#[test]
fn production_watchdog_execution_is_still_planned_disabled() -> Result<()> {
    let dir = std::env::temp_dir().join(format!(
        "screen-shader-production-decision-{}",
        std::process::id()
    ));
    std::fs::create_dir_all(&dir)?;
    let planner = HighRiskRecoveryPlanner::new(dir.join("backups"), 1000);
    let plan = planner.build_inert_live_config_plan(
        "/home/kyo/.config/hypr/hyprland.conf",
        dir.join("planned-screen-shader-backup.conf"),
        dir.join("planned-screen-shader-result.json"),
        "decoration:screen_shader = ./candidate-screen-shader.frag",
        "sha256:known-good-placeholder",
        30,
    )?;

    validate_inert_live_config_plan(&plan)?;
    assert!(!plan.live_execution_enabled);
    let error = refuse_inert_live_config_execution(&plan)
        .expect_err("live production watchdog execution must remain disabled");
    assert!(error
        .to_string()
        .contains("live config watchdog execution is disabled"));

    std::fs::remove_dir_all(dir)?;
    Ok(())
}
