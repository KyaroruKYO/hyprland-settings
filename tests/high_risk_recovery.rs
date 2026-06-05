use std::fs;
use std::path::{Path, PathBuf};
use std::time::{SystemTime, UNIX_EPOCH};

use anyhow::Result;
use hyprland_settings::high_risk_recovery::{
    ensure_dry_run_target_path, HighRiskRecoveryPlanner, RecoveryStatus,
};
use hyprland_settings::write_classification::SAFE_WRITABLE_ROWS;
use serde_json::Value;

fn temp_case(name: &str) -> Result<PathBuf> {
    let nanos = SystemTime::now().duration_since(UNIX_EPOCH)?.as_nanos();
    let dir = std::env::temp_dir().join(format!(
        "hyprland-settings-high-risk-recovery-{name}-{}-{nanos}",
        std::process::id()
    ));
    fs::create_dir_all(&dir)?;
    Ok(dir)
}

fn read_json(path: &str) -> Result<Value> {
    Ok(serde_json::from_str(&fs::read_to_string(path)?)?)
}

#[test]
fn dry_run_watchdog_confirm_path_keeps_simulated_mutation() -> Result<()> {
    let dir = temp_case("confirm")?;
    let config_path = dir.join("hyprland.conf");
    let backup_root = dir.join("backups");
    let original = "cursor:no_warps = false\n";
    let proposed = "cursor:no_warps = true\n";
    fs::write(&config_path, original)?;

    let planner = HighRiskRecoveryPlanner::new(&backup_root, 100);
    let (plan, armed) = planner.arm_for_temp_config(&config_path, proposed, 5)?;

    assert_eq!(armed.status, RecoveryStatus::Armed);
    assert_eq!(fs::read_to_string(&config_path)?, proposed);
    assert!(plan.backup_path.exists());
    assert_eq!(plan.confirmation_deadline_unix_seconds, 105);
    assert!(armed.backup_created_before_mutation);
    assert!(armed.simulated_mutation_applied);
    assert!(!armed.reload_run);
    assert!(!armed.eval_run);
    assert!(!armed.lua_executed);
    assert!(!armed.active_config_modified);
    assert!(!armed.active_runtime_modified);

    let confirmed = planner.confirm(plan.clone(), &plan.confirmation_token);
    assert_eq!(confirmed.status, RecoveryStatus::Confirmed);
    assert_eq!(fs::read_to_string(&config_path)?, proposed);
    assert!(!confirmed.requires_hyprland_keybind);
    assert!(!confirmed.requires_app_ui);
    assert!(!confirmed.requires_visible_display);
    assert!(!confirmed.requires_mouse_input);

    Ok(())
}

#[test]
fn dry_run_watchdog_timeout_restores_backup_and_verifies_reread() -> Result<()> {
    let dir = temp_case("revert")?;
    let config_path = dir.join("hyprland.conf");
    let backup_root = dir.join("backups");
    let original = "render:direct_scanout = 0\n";
    let proposed = "render:direct_scanout = 2\n";
    fs::write(&config_path, original)?;

    let planner = HighRiskRecoveryPlanner::new(&backup_root, 200);
    let (plan, _) = planner.arm_for_temp_config(&config_path, proposed, 5)?;
    assert_eq!(fs::read_to_string(&config_path)?, proposed);

    let reverted = planner.expire_and_recover(&plan);
    assert_eq!(reverted.status, RecoveryStatus::Reverted);
    assert!(reverted.restore_attempted);
    assert!(reverted.restore_verified);
    assert_eq!(fs::read_to_string(&config_path)?, original);
    assert!(!reverted.reload_run);
    assert!(!reverted.eval_run);
    assert!(!reverted.lua_executed);
    assert!(!reverted.active_config_modified);
    assert!(!reverted.active_runtime_modified);

    Ok(())
}

#[test]
fn dry_run_watchdog_records_failed_restore_verification() -> Result<()> {
    let dir = temp_case("failed")?;
    let config_path = dir.join("hyprland.conf");
    let backup_root = dir.join("backups");
    fs::write(&config_path, "debug:manual_crash = 0\n")?;

    let planner = HighRiskRecoveryPlanner::new(&backup_root, 300);
    let (plan, _) = planner.arm_for_temp_config(&config_path, "debug:manual_crash = 1\n", 5)?;
    fs::write(&plan.backup_path, "corrupt backup\n")?;

    let failed = planner.expire_and_recover(&plan);
    assert_eq!(failed.status, RecoveryStatus::Failed);
    assert!(failed.restore_attempted);
    assert!(!failed.restore_verified);
    assert!(failed.error.as_deref().unwrap_or("").contains("restore"));

    Ok(())
}

#[test]
fn dry_run_recovery_refuses_live_or_non_temp_config_paths() {
    assert!(ensure_dry_run_target_path(Path::new("relative.conf")).is_err());
    assert!(ensure_dry_run_target_path(Path::new("/home/kyo/.config/hypr/hyprland.conf")).is_err());
}

#[test]
fn high_risk_recovery_reports_keep_all_high_risk_rows_blocked() -> Result<()> {
    let design = read_json("data/reports/high-risk-dead-man-recovery-design.v0.55.2.json")?;
    let buckets = read_json("data/reports/high-risk-recovery-bucket-plan.v0.55.2.json")?;
    let proof = read_json("data/reports/high-risk-watchdog-dry-run-proof.v0.55.2.json")?;
    let coverage = read_json("data/reports/scalar-read-write-coverage.v0.55.2.json")?;
    let pipeline = read_json("data/reports/all-341-unified-pipeline.v0.55.2.json")?;

    assert_eq!(design["counts"]["rows"], 72);
    assert_eq!(design["counts"]["rowsEnabled"], 0);
    assert_eq!(design["counts"]["finalWritableRows"], 269);
    assert_eq!(design["counts"]["finalBlockedRows"], 72);
    assert_eq!(buckets["counts"]["rows"], 72);
    assert_eq!(proof["counts"]["dryRunScenarios"], 3);
    assert_eq!(proof["counts"]["confirmPathPassed"], 1);
    assert_eq!(proof["counts"]["timeoutRevertPathPassed"], 1);
    assert_eq!(proof["counts"]["restoreFailurePathPassed"], 1);

    assert_eq!(coverage["counts"]["writableRows"], 269);
    assert_eq!(coverage["counts"]["blockedWriteRows"], 72);
    assert_eq!(SAFE_WRITABLE_ROWS.len(), 269);
    assert_eq!(pipeline["counts"]["totalRows"], 341);
    assert_eq!(pipeline["counts"]["writableRows"], 269);
    assert_eq!(pipeline["counts"]["blockedRows"], 72);
    assert_eq!(pipeline["counts"]["metadataGapRows"], 0);

    for row in design["rows"].as_array().unwrap() {
        let row_id = row["rowId"].as_str().unwrap();
        assert_eq!(
            row["writeStatus"].as_str(),
            Some("high-risk"),
            "{row_id} should remain blocked as high-risk"
        );
        assert_eq!(
            row["safeWriteSupported"].as_bool(),
            Some(false),
            "{row_id} must not be marked safe writable"
        );
        assert!(
            row["recoveryBucket"]
                .as_str()
                .is_some_and(|value| !value.is_empty()),
            "{row_id} should have a recovery bucket"
        );
        assert!(
            row["recoveryStrategy"]
                .as_str()
                .is_some_and(|value| !value.is_empty()),
            "{row_id} should have a recovery strategy"
        );
        assert!(
            row["approvalGate"]
                .as_str()
                .is_some_and(|value| !value.is_empty()),
            "{row_id} should have an approval gate"
        );
        assert_eq!(row["watchdogDryRunStatus"].as_str(), Some("passed"));
    }

    for result in proof["scenarios"].as_array().unwrap() {
        assert_eq!(result["activeConfigModified"].as_bool(), Some(false));
        assert_eq!(result["activeRuntimeModified"].as_bool(), Some(false));
        assert_eq!(result["reloadRun"].as_bool(), Some(false));
        assert_eq!(result["evalRun"].as_bool(), Some(false));
        assert_eq!(result["luaExecuted"].as_bool(), Some(false));
        assert!(!result["targetConfigPath"]
            .as_str()
            .unwrap()
            .contains("/.config/hypr/hyprland.conf"));
    }

    Ok(())
}
