use std::fs;
use std::path::{Path, PathBuf};
use std::process::Command;
use std::time::{SystemTime, UNIX_EPOCH};

use anyhow::Result;
use hyprland_settings::high_risk_recovery::{
    ensure_dry_run_target_path, load_watchdog_plan, load_watchdog_result,
    refuse_inert_live_config_execution, validate_inert_live_config_plan, HighRiskRecoveryPlanner,
    HighRiskWatchdogMode, RecoveryStatus,
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

fn watchdog_binary() -> PathBuf {
    PathBuf::from(env!("CARGO_BIN_EXE_hyprland-settings"))
}

fn run_watchdog_cli(
    plan_path: &Path,
    result_path: &Path,
    backup_root: &Path,
    action: &str,
    token: Option<&str>,
    now: u64,
) -> Result<std::process::Output> {
    let mut command = Command::new(watchdog_binary());
    command
        .arg("high-risk-watchdog")
        .arg("--plan")
        .arg(plan_path)
        .arg("--result")
        .arg(result_path)
        .arg("--backup-root")
        .arg(backup_root)
        .arg("--action")
        .arg(action)
        .arg("--now")
        .arg(now.to_string());
    if let Some(token) = token {
        command.arg("--token").arg(token);
    }
    Ok(command.output()?)
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
fn production_watchdog_primitives_persist_load_confirm_and_expire() -> Result<()> {
    let dir = temp_case("production-primitives")?;
    let config_path = dir.join("hyprland.conf");
    let backup_root = dir.join("backups");
    let plan_path = dir.join("watchdog-plan.json");
    let result_path = dir.join("watchdog-result.json");
    fs::write(&config_path, "ecosystem:no_update_news = false\n")?;

    let planner = HighRiskRecoveryPlanner::new(&backup_root, 400);
    let plan = planner.arm_watchdog_for_temp_config(
        &config_path,
        "ecosystem:no_update_news = true\n",
        5,
        &plan_path,
        &result_path,
    )?;
    assert_eq!(
        fs::read_to_string(&config_path)?,
        "ecosystem:no_update_news = true\n"
    );
    assert!(
        plan_path.exists(),
        "plan must be persisted before independent load"
    );
    let loaded = load_watchdog_plan(&plan_path)?;
    assert_eq!(
        loaded.recovery.recovery_session_id,
        plan.recovery.recovery_session_id
    );

    let confirmed = planner.confirm_watchdog(&loaded, &loaded.recovery.confirmation_token)?;
    assert_eq!(confirmed.recovery.status, RecoveryStatus::Confirmed);
    assert!(result_path.exists());
    assert_eq!(
        load_watchdog_result(&result_path)?.recovery.status,
        RecoveryStatus::Confirmed
    );

    let second_config = dir.join("hyprland-second.conf");
    let second_plan = dir.join("watchdog-plan-second.json");
    let second_result = dir.join("watchdog-result-second.json");
    fs::write(&second_config, "ecosystem:no_update_news = false\n")?;
    let timeout_plan = planner.arm_watchdog_for_temp_config(
        &second_config,
        "ecosystem:no_update_news = true\n",
        5,
        &second_plan,
        &second_result,
    )?;
    let loaded_timeout = load_watchdog_plan(&second_plan)?;
    assert_eq!(
        loaded_timeout.recovery.recovery_session_id,
        timeout_plan.recovery.recovery_session_id
    );
    let after_deadline = HighRiskRecoveryPlanner::new(&backup_root, 406);
    let reverted = after_deadline.expire_watchdog_and_restore(&loaded_timeout)?;
    assert_eq!(reverted.recovery.status, RecoveryStatus::Reverted);
    assert!(reverted.recovery.restore_verified);
    assert_eq!(
        fs::read_to_string(&second_config)?,
        "ecosystem:no_update_news = false\n"
    );
    assert_eq!(
        load_watchdog_result(&second_result)?.recovery.status,
        RecoveryStatus::Reverted
    );

    Ok(())
}

#[test]
fn dry_run_recovery_refuses_live_or_non_temp_config_paths() {
    assert!(ensure_dry_run_target_path(Path::new("relative.conf")).is_err());
    assert!(ensure_dry_run_target_path(Path::new("/home/kyo/.config/hypr/hyprland.conf")).is_err());
}

#[test]
fn separate_watchdog_process_confirms_with_token_and_rejects_wrong_token() -> Result<()> {
    let dir = temp_case("separate-confirm")?;
    let config_path = dir.join("hyprland.conf");
    let backup_root = dir.join("backups");
    let plan_path = dir.join("watchdog-plan.json");
    let wrong_result_path = dir.join("wrong-token-result.json");
    let result_path = dir.join("watchdog-result.json");
    fs::write(&config_path, "render:direct_scanout = 0\n")?;

    let planner = HighRiskRecoveryPlanner::new(&backup_root, 600);
    let plan = planner.arm_watchdog_for_temp_config(
        &config_path,
        "render:direct_scanout = 1\n",
        10,
        &plan_path,
        &result_path,
    )?;
    assert!(plan_path.exists());
    assert_eq!(
        fs::read_to_string(&config_path)?,
        "render:direct_scanout = 1\n"
    );

    let wrong = run_watchdog_cli(
        &plan_path,
        &wrong_result_path,
        &backup_root,
        "confirm",
        Some("wrong-token"),
        605,
    )?;
    assert!(
        !wrong.status.success(),
        "wrong token must fail as a separate process"
    );
    assert!(!wrong_result_path.exists());
    assert_eq!(
        fs::read_to_string(&config_path)?,
        "render:direct_scanout = 1\n"
    );

    let confirmed = run_watchdog_cli(
        &plan_path,
        &result_path,
        &backup_root,
        "confirm",
        Some(&plan.recovery.confirmation_token),
        605,
    )?;
    assert!(
        confirmed.status.success(),
        "correct token failed: {}",
        String::from_utf8_lossy(&confirmed.stderr)
    );
    let result = load_watchdog_result(&result_path)?;
    assert_eq!(result.recovery.status, RecoveryStatus::Confirmed);
    assert_eq!(
        fs::read_to_string(&config_path)?,
        "render:direct_scanout = 1\n"
    );
    assert!(!result.recovery.requires_app_ui);
    assert!(!result.recovery.requires_hyprland_keybind);
    assert!(!result.recovery.requires_visible_display);
    assert!(!result.recovery.requires_mouse_input);
    assert!(!result.recovery.reload_run);
    assert!(!result.recovery.eval_run);
    assert!(!result.recovery.lua_executed);
    assert!(!result.recovery.active_config_modified);
    assert!(!result.recovery.active_runtime_modified);

    Ok(())
}

#[test]
fn separate_watchdog_process_expires_and_restores_backup() -> Result<()> {
    let dir = temp_case("separate-expire")?;
    let config_path = dir.join("hyprland.conf");
    let backup_root = dir.join("backups");
    let plan_path = dir.join("watchdog-plan.json");
    let result_path = dir.join("watchdog-result.json");
    fs::write(&config_path, "cursor:no_warps = false\n")?;

    let planner = HighRiskRecoveryPlanner::new(&backup_root, 700);
    let plan = planner.arm_watchdog_for_temp_config(
        &config_path,
        "cursor:no_warps = true\n",
        10,
        &plan_path,
        &result_path,
    )?;
    assert_eq!(plan.recovery.confirmation_deadline_unix_seconds, 710);
    assert_eq!(
        fs::read_to_string(&config_path)?,
        "cursor:no_warps = true\n"
    );

    let before_deadline = run_watchdog_cli(
        &plan_path,
        &dir.join("before-deadline-result.json"),
        &backup_root,
        "expire",
        None,
        705,
    )?;
    assert!(
        !before_deadline.status.success(),
        "expire before deadline must fail"
    );
    assert_eq!(
        fs::read_to_string(&config_path)?,
        "cursor:no_warps = true\n"
    );

    let expired = run_watchdog_cli(&plan_path, &result_path, &backup_root, "expire", None, 711)?;
    assert!(
        expired.status.success(),
        "expire failed: {}",
        String::from_utf8_lossy(&expired.stderr)
    );
    let result = load_watchdog_result(&result_path)?;
    assert_eq!(result.recovery.status, RecoveryStatus::Reverted);
    assert!(result.recovery.restore_verified);
    assert_eq!(
        fs::read_to_string(&config_path)?,
        "cursor:no_warps = false\n"
    );
    assert!(!result.recovery.reload_run);
    assert!(!result.recovery.eval_run);
    assert!(!result.recovery.lua_executed);
    assert!(!result.recovery.active_config_modified);
    assert!(!result.recovery.active_runtime_modified);

    Ok(())
}

#[test]
fn separate_watchdog_process_reports_restore_failure_and_refuses_live_target_plan() -> Result<()> {
    let dir = temp_case("separate-failure")?;
    let config_path = dir.join("hyprland.conf");
    let backup_root = dir.join("backups");
    let plan_path = dir.join("watchdog-plan.json");
    let result_path = dir.join("watchdog-result.json");
    fs::write(&config_path, "debug:manual_crash = 0\n")?;

    let planner = HighRiskRecoveryPlanner::new(&backup_root, 800);
    let plan = planner.arm_watchdog_for_temp_config(
        &config_path,
        "debug:manual_crash = 1\n",
        10,
        &plan_path,
        &result_path,
    )?;
    fs::write(&plan.recovery.backup_path, "corrupt backup\n")?;
    let failed = run_watchdog_cli(&plan_path, &result_path, &backup_root, "expire", None, 811)?;
    assert!(
        !failed.status.success(),
        "restore failure must exit nonzero"
    );
    let result = load_watchdog_result(&result_path)?;
    assert_eq!(result.recovery.status, RecoveryStatus::Failed);
    assert!(result.recovery.restore_attempted);
    assert!(!result.recovery.restore_verified);

    let live_plan_path = dir.join("live-target-plan.json");
    let live_result_path = dir.join("live-target-result.json");
    let mut malicious = load_watchdog_plan(&plan_path).unwrap_or(plan);
    malicious.plan_path = live_plan_path.clone();
    malicious.result_log_path = live_result_path.clone();
    malicious.recovery.target_config_path = PathBuf::from("/home/kyo/.config/hypr/hyprland.conf");
    fs::write(&live_plan_path, serde_json::to_vec_pretty(&malicious)?)?;
    let refused = run_watchdog_cli(
        &live_plan_path,
        &live_result_path,
        &backup_root,
        "expire",
        None,
        811,
    )?;
    assert!(
        !refused.status.success(),
        "dry-run CLI must refuse plans targeting live config"
    );

    Ok(())
}

#[test]
fn inert_live_config_watchdog_plan_can_be_represented_but_not_executed() -> Result<()> {
    let dir = temp_case("inert-live-config")?;
    let planner = HighRiskRecoveryPlanner::new(dir.join("backups"), 500);
    let plan = planner.build_inert_live_config_plan(
        "/home/kyo/.config/hypr/hyprland.conf",
        dir.join("planned-live-backup.conf"),
        dir.join("planned-live-result.json"),
        "render:direct_scanout = 1",
        "sha256:known-good-placeholder",
        10,
    )?;

    assert_eq!(plan.mode, HighRiskWatchdogMode::LiveConfigPlannedDisabled);
    assert!(!plan.live_execution_enabled);
    assert_eq!(plan.confirmation_deadline_unix_seconds, 510);
    assert!(plan
        .restore_command_description
        .contains("file reread verification"));
    validate_inert_live_config_plan(&plan)?;
    let execution_error = refuse_inert_live_config_execution(&plan)
        .expect_err("live config execution must remain disabled");
    assert!(execution_error
        .to_string()
        .contains("live config watchdog execution is disabled"));

    let mut unsafe_plan = plan.clone();
    unsafe_plan.live_execution_enabled = true;
    assert!(validate_inert_live_config_plan(&unsafe_plan).is_err());

    Ok(())
}

#[test]
fn high_risk_recovery_reports_keep_all_high_risk_rows_blocked() -> Result<()> {
    let design = read_json("data/reports/high-risk-dead-man-recovery-design.v0.55.2.json")?;
    let buckets = read_json("data/reports/high-risk-recovery-bucket-plan.v0.55.2.json")?;
    let proof = read_json("data/reports/high-risk-watchdog-dry-run-proof.v0.55.2.json")?;
    let production = read_json("data/reports/production-high-risk-watchdog.v0.55.2.json")?;
    let controlled = read_json("data/reports/controlled-live-watchdog-design.v0.55.2.json")?;
    let confirmation = read_json("data/reports/out-of-band-confirmation-design.v0.55.2.json")?;
    let readiness = read_json("data/reports/next-high-risk-bucket-readiness.v0.55.2.json")?;
    let separate = read_json("data/reports/separate-watchdog-process-proof.v0.55.2.json")?;
    let cli_confirm = read_json("data/reports/watchdog-cli-token-confirmation-proof.v0.55.2.json")?;
    let cli_timeout =
        read_json("data/reports/watchdog-timeout-restore-process-proof.v0.55.2.json")?;
    let ecosystem = read_json("data/reports/high-risk-ecosystem-bucket-proof.v0.55.2.json")?;
    let enablements = read_json("data/reports/high-risk-first-bucket-enablements.v0.55.2.json")?;
    let coverage = read_json("data/reports/scalar-read-write-coverage.v0.55.2.json")?;
    let pipeline = read_json("data/reports/all-341-unified-pipeline.v0.55.2.json")?;

    assert_eq!(design["counts"]["rows"], 72);
    assert_eq!(design["counts"]["rowsEnabled"], 8);
    assert_eq!(design["counts"]["finalWritableRows"], 277);
    assert_eq!(design["counts"]["finalBlockedRows"], 64);
    assert_eq!(buckets["counts"]["rows"], 72);
    assert_eq!(proof["counts"]["dryRunScenarios"], 3);
    assert_eq!(proof["counts"]["confirmPathPassed"], 1);
    assert_eq!(proof["counts"]["timeoutRevertPathPassed"], 1);
    assert_eq!(proof["counts"]["restoreFailurePathPassed"], 1);
    assert_eq!(production["counts"]["watchdogPrimitivesImplemented"], true);
    assert_eq!(production["counts"]["persistLoadPassed"], true);
    assert_eq!(production["counts"]["confirmPathPassed"], true);
    assert_eq!(production["counts"]["timeoutRestorePassed"], true);
    assert_eq!(production["counts"]["restoreFailurePassed"], true);
    assert_eq!(production["counts"]["liveExecutionEnabled"], false);
    assert_eq!(production["counts"]["separateProcessProofPassed"], true);
    assert_eq!(controlled["counts"]["liveExecutionEnabled"], false);
    assert_eq!(controlled["counts"]["rowsEnabled"], 0);
    assert_eq!(controlled["counts"]["finalWritableRows"], 275);
    assert_eq!(controlled["counts"]["finalBlockedRows"], 66);
    assert_eq!(confirmation["counts"]["confirmationOptions"], 6);
    assert_eq!(readiness["counts"]["remainingBlockedRows"], 64);
    assert_eq!(
        readiness["counts"]["recommendedNextBucket"].as_str(),
        Some("none-selected-after-cursor-theme-sync-smoke-subset")
    );
    assert_eq!(separate["counts"]["separateProcessConfirmPassed"], true);
    assert_eq!(separate["counts"]["wrongTokenFailed"], true);
    assert_eq!(
        separate["counts"]["separateProcessTimeoutRestorePassed"],
        true
    );
    assert_eq!(separate["counts"]["restoreFailurePathPassed"], true);
    assert_eq!(separate["counts"]["rowsEnabled"], 2);
    assert_eq!(cli_confirm["counts"]["correctTokenConfirmPassed"], true);
    assert_eq!(cli_confirm["counts"]["wrongTokenFailed"], true);
    assert_eq!(cli_timeout["counts"]["timeoutRestorePassed"], true);
    assert_eq!(cli_timeout["counts"]["restoreFailurePathPassed"], true);
    assert_eq!(ecosystem["counts"]["rows"], 3);
    assert_eq!(ecosystem["counts"]["safeToEnable"], 3);
    assert_eq!(enablements["counts"]["enabledRows"], 3);
    assert_eq!(enablements["counts"]["displayRenderRowsEnabled"], 0);
    assert_eq!(enablements["counts"]["cursorInputRowsEnabled"], 0);
    assert_eq!(enablements["counts"]["debugCrashRowsEnabled"], 0);

    assert_eq!(coverage["counts"]["writableRows"], 277);
    assert_eq!(coverage["counts"]["blockedWriteRows"], 64);
    assert_eq!(SAFE_WRITABLE_ROWS.len(), 277);
    assert_eq!(pipeline["counts"]["totalRows"], 341);
    assert_eq!(pipeline["counts"]["writableRows"], 277);
    assert_eq!(pipeline["counts"]["blockedRows"], 64);
    assert_eq!(pipeline["counts"]["metadataGapRows"], 0);

    for row in design["rows"].as_array().unwrap() {
        let row_id = row["rowId"].as_str().unwrap();
        if row["recoveryBucket"].as_str() == Some("ecosystem-permission-policy")
            || row["rowId"].as_str().is_some_and(|row_id| {
                matches!(
                    row_id,
                    "xwayland.use_nearest_neighbor"
                        | "xwayland.force_zero_scaling"
                        | "cursor.sync_gsettings_theme"
                        | "cursor.hide_on_touch"
                        | "cursor.hide_on_tablet"
                )
            })
        {
            assert_eq!(row["writeStatus"].as_str(), Some("writable"));
            assert_eq!(row["safeWriteSupported"].as_bool(), Some(true));
        } else {
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
        }
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
