use std::fs;
use std::path::{Path, PathBuf};
use std::process::Command;
use std::time::{SystemTime, UNIX_EPOCH};

use anyhow::Result;
use hyprland_settings::high_risk_recovery::{
    ensure_dry_run_target_path, load_watchdog_plan, load_watchdog_result, HighRiskRecoveryPlanner,
    RecoveryStatus,
};
use hyprland_settings::write_classification::{is_safe_writable_setting, SAFE_WRITABLE_ROWS};
use serde_json::Value;

fn temp_case(name: &str) -> Result<PathBuf> {
    let nanos = SystemTime::now().duration_since(UNIX_EPOCH)?.as_nanos();
    let dir = std::env::temp_dir().join(format!(
        "hyprland-settings-screen-shader-watchdog-{name}-{}-{nanos}",
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

fn assert_screen_shader_recovery_independent(
    result: &hyprland_settings::high_risk_recovery::HighRiskWatchdogResult,
) {
    assert!(!result.recovery.requires_visible_display);
    assert!(!result.recovery.requires_mouse_input);
    assert!(!result.recovery.requires_app_ui);
    assert!(!result.recovery.requires_hyprland_keybind);
    assert!(!result.recovery.reload_run);
    assert!(!result.recovery.eval_run);
    assert!(!result.recovery.lua_executed);
    assert!(!result.recovery.active_runtime_modified);
    assert!(!result.recovery.active_config_modified);
}

#[test]
fn screen_shader_watchdog_confirm_path_is_fixture_temp_only() -> Result<()> {
    let dir = temp_case("confirm")?;
    let config_path = dir.join("hyprland.conf");
    let backup_root = dir.join("backups");
    let plan_path = dir.join("screen-shader-watchdog-plan.json");
    let result_path = dir.join("screen-shader-watchdog-result.json");
    let original = "decoration:screen_shader = ./old-screen-shader.frag\n";
    let proposed = "decoration:screen_shader = ./candidate-screen-shader.frag\n";
    fs::write(&config_path, original)?;

    ensure_dry_run_target_path(&config_path)?;
    let planner = HighRiskRecoveryPlanner::new(&backup_root, 1000);
    let plan = planner.arm_watchdog_for_temp_config(
        &config_path,
        proposed,
        10,
        &plan_path,
        &result_path,
    )?;

    assert!(plan_path.exists(), "watchdog plan must be persisted");
    let loaded = load_watchdog_plan(&plan_path)?;
    assert_eq!(
        loaded.recovery.recovery_session_id,
        plan.recovery.recovery_session_id
    );
    assert!(loaded.recovery.backup_path.exists());
    assert_eq!(fs::read_to_string(&config_path)?, proposed);

    let confirmed = run_watchdog_cli(
        &plan_path,
        &result_path,
        &backup_root,
        "confirm",
        Some(&loaded.recovery.confirmation_token),
        1005,
    )?;
    assert!(
        confirmed.status.success(),
        "confirm failed: {}",
        String::from_utf8_lossy(&confirmed.stderr)
    );
    assert!(result_path.exists(), "confirm result log must be written");

    let result = load_watchdog_result(&result_path)?;
    assert_eq!(result.recovery.status, RecoveryStatus::Confirmed);
    assert!(result.recovery.backup_created_before_mutation);
    assert!(result.recovery.simulated_mutation_applied);
    assert_eq!(fs::read_to_string(&config_path)?, proposed);
    assert_screen_shader_recovery_independent(&result);

    fs::remove_dir_all(dir)?;
    Ok(())
}

#[test]
fn screen_shader_watchdog_timeout_restore_is_fixture_temp_only() -> Result<()> {
    let dir = temp_case("timeout")?;
    let config_path = dir.join("hyprland.conf");
    let backup_root = dir.join("backups");
    let plan_path = dir.join("screen-shader-watchdog-plan.json");
    let result_path = dir.join("screen-shader-watchdog-result.json");
    let original = "decoration:screen_shader = ./old-screen-shader.frag\n";
    let proposed = "decoration:screen_shader = ./candidate-screen-shader.frag\n";
    fs::write(&config_path, original)?;

    let planner = HighRiskRecoveryPlanner::new(&backup_root, 2000);
    let plan = planner.arm_watchdog_for_temp_config(
        &config_path,
        proposed,
        10,
        &plan_path,
        &result_path,
    )?;
    assert!(plan_path.exists());
    assert!(plan.recovery.backup_path.exists());
    assert_eq!(fs::read_to_string(&config_path)?, proposed);

    let expired = run_watchdog_cli(&plan_path, &result_path, &backup_root, "expire", None, 2011)?;
    assert!(
        expired.status.success(),
        "timeout restore failed: {}",
        String::from_utf8_lossy(&expired.stderr)
    );
    assert!(result_path.exists(), "timeout result log must be written");

    let result = load_watchdog_result(&result_path)?;
    assert_eq!(result.recovery.status, RecoveryStatus::Reverted);
    assert!(result.recovery.backup_created_before_mutation);
    assert!(result.recovery.restore_attempted);
    assert!(result.recovery.restore_verified);
    assert_eq!(fs::read_to_string(&config_path)?, original);
    assert_screen_shader_recovery_independent(&result);

    fs::remove_dir_all(dir)?;
    Ok(())
}

#[test]
fn screen_shader_watchdog_failure_path_writes_diagnostic_result() -> Result<()> {
    let dir = temp_case("failure")?;
    let config_path = dir.join("hyprland.conf");
    let backup_root = dir.join("backups");
    let plan_path = dir.join("screen-shader-watchdog-plan.json");
    let result_path = dir.join("screen-shader-watchdog-result.json");
    let original = "decoration:screen_shader = ./old-screen-shader.frag\n";
    let proposed = "decoration:screen_shader = ./candidate-screen-shader.frag\n";
    fs::write(&config_path, original)?;

    let planner = HighRiskRecoveryPlanner::new(&backup_root, 3000);
    let plan = planner.arm_watchdog_for_temp_config(
        &config_path,
        proposed,
        10,
        &plan_path,
        &result_path,
    )?;
    fs::write(&plan.recovery.backup_path, "corrupt backup\n")?;

    let failed = run_watchdog_cli(&plan_path, &result_path, &backup_root, "expire", None, 3011)?;
    assert!(
        !failed.status.success(),
        "corrupt backup restore should fail"
    );
    assert!(result_path.exists(), "failure result log must be written");

    let result = load_watchdog_result(&result_path)?;
    assert_eq!(result.recovery.status, RecoveryStatus::Failed);
    assert!(result.recovery.backup_created_before_mutation);
    assert!(result.recovery.restore_attempted);
    assert!(!result.recovery.restore_verified);
    assert!(result
        .recovery
        .error
        .as_deref()
        .unwrap_or("")
        .contains("restore"));
    assert_screen_shader_recovery_independent(&result);

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
        3011,
    )?;
    assert!(
        !refused.status.success(),
        "dry-run watchdog must refuse a live config target"
    );

    fs::remove_dir_all(dir)?;
    Ok(())
}

#[test]
fn screen_shader_watchdog_migration_proof_report_preserves_project_boundaries() -> Result<()> {
    let coverage = read_json("data/reports/scalar-read-write-coverage.v0.55.2.json")?;
    let proof = read_json("data/reports/screen-shader-watchdog-migration-proof.v0.55.2.json")?;
    let migration = read_json("data/reports/screen-shader-high-risk-gate-migration.v0.55.2.json")?;

    assert!(is_safe_writable_setting("decoration.screen_shader"));
    assert_eq!(SAFE_WRITABLE_ROWS.len(), 341);
    assert_eq!(coverage["counts"]["readableRows"], 341);
    assert_eq!(coverage["counts"]["writableRows"], 341);
    assert_eq!(coverage["counts"]["blockedWriteRows"], 0);

    assert_eq!(proof["rowId"], "decoration.screen_shader");
    assert_eq!(proof["officialSetting"], "decoration.screen_shader");
    assert_eq!(proof["startingCommit"], "e9c665c");
    assert_eq!(proof["selectedMigrationOption"], "Option A");
    assert_eq!(proof["currentWritableStatus"], "writable");
    assert_eq!(proof["readableRows"], 341);
    assert_eq!(proof["writableRows"], 278);
    assert_eq!(proof["blockedRows"], 63);
    assert_eq!(proof["safeWritableRowsChanged"], false);
    assert_eq!(proof["writeAllowlistChanged"], false);
    assert_eq!(proof["rowsEnabledThisSprint"], 0);
    assert_eq!(proof["countedAsEnabledHighRiskRow"], false);
    assert_eq!(proof["realConfigTouched"], false);
    assert_eq!(proof["runtimeTouched"], false);
    assert_eq!(proof["reloadEvalLuaUsed"], false);
    assert_eq!(proof["liveShaderCompileUsed"], false);
    assert_eq!(proof["liveDisplayRuntimeProofUsed"], false);
    assert_eq!(proof["proofOnly"], true);
    assert_eq!(proof["fixtureTempOnly"], true);
    assert_eq!(proof["planPersistedBeforeMutationProof"], true);
    assert_eq!(proof["backupBeforeMutationProof"], true);
    assert_eq!(proof["fixtureTempMutationOnlyProof"], true);
    assert_eq!(proof["separateProcessConfirmProof"], true);
    assert_eq!(proof["timeoutRestoreProof"], true);
    assert_eq!(proof["resultLogProof"], true);
    assert_eq!(proof["visibleDisplayIndependentRecoveryProof"], true);
    assert_eq!(proof["liveRenderStateIndependentRecoveryProof"], true);
    assert_eq!(proof["failurePathProof"], true);
    assert_eq!(proof["watchdogMigrationProofStatus"], "complete");
    assert_eq!(proof["productionEnforcementChanged"], false);
    assert_eq!(proof["productionGateEnforcedThisSprint"], false);
    assert_eq!(proof["compileAwareValidationChanged"], false);
    assert_eq!(proof["compileAwareValidationStatus"], "deferred");
    assert_eq!(
        proof["classificationAfterSprint"]["writableMigrationCandidate"],
        true
    );
    assert_eq!(
        proof["classificationAfterSprint"]["countedAsEnabledHighRiskRow"],
        false
    );

    assert_eq!(migration["proofExists"]["screenShaderWatchdogProof"], true);

    Ok(())
}
