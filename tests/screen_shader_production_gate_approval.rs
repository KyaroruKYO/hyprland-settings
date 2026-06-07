use std::collections::BTreeSet;
use std::fs;
use std::path::PathBuf;
use std::time::{SystemTime, UNIX_EPOCH};

use anyhow::Result;
use hyprland_settings::config_backup::BackupManager;
use hyprland_settings::config_discovery::{
    ConfigDiscovery, ConfigDiscoveryStatus, ConfigPathSource,
};
use hyprland_settings::config_parser::parse_hyprland_config_text;
use hyprland_settings::current_config::CurrentConfigSnapshot;
use hyprland_settings::high_risk_recovery::HighRiskRecoveryPlanner;
use hyprland_settings::write_classification::{is_safe_writable_setting, SAFE_WRITABLE_ROWS};
use hyprland_settings::write_flow::{
    apply_setting_change_with_backup_manager,
    apply_setting_change_with_backup_manager_and_high_risk_gate,
};
use hyprland_settings::write_safety::{HighRiskGateProof, SCREEN_SHADER_GATED_SETTING_ID};
use serde_json::Value;

fn read_json(path: &str) -> Result<Value> {
    Ok(serde_json::from_str(&std::fs::read_to_string(path)?)?)
}

fn temp_case(name: &str) -> Result<PathBuf> {
    let nanos = SystemTime::now().duration_since(UNIX_EPOCH)?.as_nanos();
    let dir = std::env::temp_dir().join(format!(
        "hyprland-settings-screen-shader-gate-approval-{name}-{}-{nanos}",
        std::process::id()
    ));
    fs::create_dir_all(&dir)?;
    Ok(dir)
}

fn known_ids() -> BTreeSet<String> {
    SAFE_WRITABLE_ROWS
        .iter()
        .map(|row| row.row_id.to_string())
        .collect()
}

fn discovery_for(path: PathBuf) -> ConfigDiscovery {
    ConfigDiscovery {
        status: ConfigDiscoveryStatus::Found {
            path: path.clone(),
            source: ConfigPathSource::HomeFallback,
        },
        attempted_paths: vec![path],
    }
}

fn snapshot_for(path: &PathBuf, contents: &str) -> CurrentConfigSnapshot {
    CurrentConfigSnapshot::from_parsed(parse_hyprland_config_text(path, contents))
}

fn make_screen_shader_gate_proof(
    dir: &std::path::Path,
    config_path: &std::path::Path,
    original: &str,
    proposed: &str,
) -> Result<HighRiskGateProof> {
    let backup_root = dir.join("watchdog-backups");
    let plan_path = dir.join("screen-shader-watchdog-plan.json");
    let result_path = dir.join("screen-shader-watchdog-result.json");
    let planner = HighRiskRecoveryPlanner::new(&backup_root, 10_000);
    let watchdog_plan = planner.arm_watchdog_for_temp_config(
        config_path,
        proposed,
        10,
        &plan_path,
        &result_path,
    )?;
    fs::write(config_path, original)?;
    Ok(HighRiskGateProof {
        setting_id: SCREEN_SHADER_GATED_SETTING_ID.to_string(),
        recovery_bucket: "display-render-recovery:screen-shader-gate-migration-design".to_string(),
        watchdog_plan,
    })
}

#[test]
fn screen_shader_production_gate_approval_report_records_option_c() -> Result<()> {
    let coverage = read_json("data/reports/scalar-read-write-coverage.v0.55.2.json")?;
    let approval = read_json("data/reports/screen-shader-production-gate-approval.v0.55.2.json")?;
    let architecture =
        read_json("data/reports/screen-shader-production-gate-architecture.v0.55.2.json")?;
    let watchdog = read_json("data/reports/screen-shader-watchdog-migration-proof.v0.55.2.json")?;
    let pipeline = read_json("data/reports/all-341-unified-pipeline.v0.55.2.json")?;

    assert!(is_safe_writable_setting("decoration.screen_shader"));
    assert_eq!(SAFE_WRITABLE_ROWS.len(), 278);
    assert_eq!(coverage["counts"]["readableRows"], 341);
    assert_eq!(coverage["counts"]["writableRows"], 278);
    assert_eq!(coverage["counts"]["blockedWriteRows"], 63);
    assert_eq!(approval["rowId"], "decoration.screen_shader");
    assert_eq!(approval["officialSetting"], "decoration.screen_shader");
    assert_eq!(approval["startingCommit"], "6b3bfe1");
    assert_eq!(approval["selectedApprovalOption"], "Option C");
    assert_eq!(approval["currentWritableStatus"], "writable");
    assert_eq!(approval["writableMigrationCandidate"], true);
    assert_eq!(approval["watchdogMigrationProofStatus"], "complete");
    assert_eq!(approval["dryRunGatePrimitiveStatus"], "complete");
    assert_eq!(approval["productionEnforcementChanged"], true);
    assert_eq!(approval["productionGateEnforcedThisSprint"], true);
    assert_eq!(approval["productionWriteFlowChanged"], true);
    assert_eq!(approval["productionApplyFlowGateWired"], true);
    assert_eq!(approval["normalProductionReviewChanged"], true);
    assert_eq!(
        approval["normalPathOnlyApprovalStillAcceptedInProduction"],
        false
    );
    assert_eq!(approval["ungatedProductionFlowScreenShaderRejected"], true);
    assert_eq!(approval["gatedProductionFlowScreenShaderAccepted"], true);
    assert_eq!(approval["invalidProductionGateProofRejected"], true);
    assert_eq!(approval["unrelatedNormalWritableRowsRequireGate"], false);
    assert_eq!(approval["countedAsEnabledHighRiskRow"], false);
    assert_eq!(approval["readableRows"], 341);
    assert_eq!(approval["writableRows"], 278);
    assert_eq!(approval["blockedRows"], 63);
    assert_eq!(approval["safeWritableRowsChanged"], false);
    assert_eq!(approval["writeAllowlistChanged"], false);
    assert_eq!(approval["rowsEnabledThisSprint"], 0);
    assert_eq!(approval["realConfigTouched"], false);
    assert_eq!(approval["runtimeTouched"], false);
    assert_eq!(approval["reloadEvalLuaUsed"], false);
    assert_eq!(approval["liveShaderCompileUsed"], false);
    assert_eq!(approval["liveDisplayRuntimeProofUsed"], false);
    assert_eq!(approval["compileAwareValidationChanged"], false);
    assert_eq!(approval["compileAwareValidationStatus"], "deferred");
    assert_eq!(architecture["dryRunGatePrimitiveAdded"], true);
    assert_eq!(watchdog["watchdogMigrationProofStatus"], "complete");
    let screen_shader_row = pipeline["rows"]
        .as_array()
        .unwrap()
        .iter()
        .find(|row| row["rowId"] == "decoration.screen_shader")
        .expect("screen shader row should exist in all-341 pipeline");
    assert_eq!(
        screen_shader_row["gateStatus"],
        "production-screen-shader-gate-enforced-compile-aware-validation-deferred"
    );
    assert_eq!(screen_shader_row["productionGateEnforcedThisSprint"], true);
    assert_eq!(screen_shader_row["countedAsEnabledHighRiskRow"], false);
    assert_eq!(pipeline["counts"]["writableRows"], 278);
    assert_eq!(pipeline["counts"]["blockedRows"], 63);

    Ok(())
}

#[test]
fn production_apply_flow_rejects_ungated_screen_shader_before_final_apply() -> Result<()> {
    let dir = temp_case("ungated")?;
    let config_path = dir.join("hyprland.conf");
    let original = "decoration:screen_shader = ./old-screen-shader.frag\n";
    fs::write(&config_path, original)?;
    let snapshot = snapshot_for(&config_path, original);
    let result = apply_setting_change_with_backup_manager(
        known_ids(),
        &discovery_for(config_path.clone()),
        &snapshot,
        "decoration.screen_shader",
        "./new-screen-shader.frag",
        &BackupManager::new(dir.join("backups")),
    );

    let failure = result.expect_err("ungated screen_shader write must be rejected");
    assert_eq!(failure.reason, "write plan rejected by high-risk gate");
    assert!(failure
        .failures
        .contains(&"MissingScreenShaderGateProof".to_string()));
    assert_eq!(fs::read_to_string(&config_path)?, original);

    fs::remove_dir_all(dir)?;
    Ok(())
}

#[test]
fn production_apply_flow_accepts_gated_screen_shader_fixture_write() -> Result<()> {
    let dir = temp_case("gated")?;
    let config_path = dir.join("hyprland.conf");
    let original = "decoration:screen_shader = ./old-screen-shader.frag\n";
    let proposed_line = "decoration:screen_shader = ./new-screen-shader.frag\n";
    fs::write(&config_path, original)?;
    let gate_proof = make_screen_shader_gate_proof(&dir, &config_path, original, proposed_line)?;
    let snapshot = snapshot_for(&config_path, original);

    let outcome = apply_setting_change_with_backup_manager_and_high_risk_gate(
        known_ids(),
        &discovery_for(config_path.clone()),
        &snapshot,
        "decoration.screen_shader",
        "./new-screen-shader.frag",
        &BackupManager::new(dir.join("backups")),
        Some(gate_proof),
    )
    .expect("gated fixture screen_shader write should be accepted");

    assert_eq!(outcome.setting_id, "decoration.screen_shader");
    assert_eq!(
        outcome.verified_value.as_deref(),
        Some("./new-screen-shader.frag")
    );
    assert_eq!(fs::read_to_string(&config_path)?, proposed_line);

    fs::remove_dir_all(dir)?;
    Ok(())
}

#[test]
fn production_apply_flow_rejects_invalid_screen_shader_gate_proof() -> Result<()> {
    let dir = temp_case("invalid-proof")?;
    let config_path = dir.join("hyprland.conf");
    let wrong_config_path = dir.join("wrong-hyprland.conf");
    let original = "decoration:screen_shader = ./old-screen-shader.frag\n";
    fs::write(&config_path, original)?;
    fs::write(&wrong_config_path, original)?;

    let gate_proof = make_screen_shader_gate_proof(
        &dir,
        &wrong_config_path,
        original,
        "decoration:screen_shader = ./new-screen-shader.frag\n",
    )?;
    let snapshot = snapshot_for(&config_path, original);
    let result = apply_setting_change_with_backup_manager_and_high_risk_gate(
        known_ids(),
        &discovery_for(config_path.clone()),
        &snapshot,
        "decoration.screen_shader",
        "./new-screen-shader.frag",
        &BackupManager::new(dir.join("backups")),
        Some(gate_proof),
    );

    let failure = result.expect_err("mismatched proof must be rejected");
    assert_eq!(failure.reason, "write plan rejected by high-risk gate");
    assert!(failure
        .failures
        .contains(&"GateProofTargetMismatch".to_string()));
    assert_eq!(fs::read_to_string(&config_path)?, original);

    fs::remove_dir_all(dir)?;
    Ok(())
}

#[test]
fn production_apply_flow_keeps_unrelated_rows_on_normal_path() -> Result<()> {
    let dir = temp_case("unrelated")?;
    let config_path = dir.join("hyprland.conf");
    let original = "windows:snap:enabled = true\n";
    fs::write(&config_path, original)?;
    let snapshot = snapshot_for(&config_path, original);
    let outcome = apply_setting_change_with_backup_manager(
        known_ids(),
        &discovery_for(config_path.clone()),
        &snapshot,
        "windows.snap.enabled",
        "false",
        &BackupManager::new(dir.join("backups")),
    )
    .expect("unrelated normal writable row should not require screen-shader gate");

    assert_eq!(outcome.setting_id, "windows.snap.enabled");
    assert_eq!(outcome.verified_value.as_deref(), Some("false"));
    assert_eq!(
        fs::read_to_string(&config_path)?,
        "windows:snap:enabled = true\ngeneral:snap:enabled = false\n"
    );

    fs::remove_dir_all(dir)?;
    Ok(())
}
