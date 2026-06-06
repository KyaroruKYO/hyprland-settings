use std::collections::BTreeSet;
use std::fs;
use std::path::{Path, PathBuf};
use std::process::{Command, Output, Stdio};
use std::thread;
use std::time::{Duration, Instant, SystemTime, UNIX_EPOCH};

use anyhow::{bail, Result};
use hyprland_settings::high_risk_recovery::{
    load_watchdog_result, HighRiskRecoveryPlanner, RecoveryStatus,
};
use hyprland_settings::write_classification::{is_safe_writable_setting, SAFE_WRITABLE_ROWS};
use serde_json::Value;

fn read_json(path: &str) -> Result<Value> {
    Ok(serde_json::from_str(&fs::read_to_string(path)?)?)
}

fn temp_case(name: &str) -> Result<PathBuf> {
    let nanos = SystemTime::now().duration_since(UNIX_EPOCH)?.as_nanos();
    let dir = std::env::temp_dir().join(format!(
        "hyprland-settings-cursor-hide-on-key-press-{name}-{}-{nanos}",
        std::process::id()
    ));
    fs::create_dir_all(&dir)?;
    Ok(dir)
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
) -> Result<Output> {
    let mut child = Command::new(watchdog_binary())
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
        .arg(now.to_string())
        .args(token.into_iter().flat_map(|token| ["--token", token]))
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()?;

    let deadline = Instant::now() + Duration::from_secs(10);
    loop {
        if child.try_wait()?.is_some() {
            return Ok(child.wait_with_output()?);
        }
        if Instant::now() >= deadline {
            let _ = child.kill();
            let _ = child.wait();
            bail!("watchdog CLI timed out");
        }
        thread::sleep(Duration::from_millis(10));
    }
}

fn prove_watchdog_process_for_hide_on_key_press() -> Result<()> {
    let row_id = "cursor.hide_on_key_press";
    let config_key = "cursor:hide_on_key_press";

    let confirm_dir = temp_case("confirm")?;
    let confirm_config = confirm_dir.join("hyprland.conf");
    let confirm_backup_root = confirm_dir.join("backups");
    let confirm_plan = confirm_dir.join("watchdog-plan.json");
    let confirm_result = confirm_dir.join("watchdog-result.json");
    let wrong_token_result = confirm_dir.join("wrong-token-result.json");
    let original = format!("{config_key} = false\n");
    let proposed = format!("{config_key} = true\n");
    fs::write(&confirm_config, &original)?;

    let planner = HighRiskRecoveryPlanner::new(&confirm_backup_root, 1000);
    let plan = planner.arm_watchdog_for_temp_config(
        &confirm_config,
        &proposed,
        10,
        &confirm_plan,
        &confirm_result,
    )?;
    assert!(
        confirm_plan.exists(),
        "{row_id} plan must exist before mutation"
    );
    assert!(
        plan.recovery.backup_path.exists(),
        "{row_id} backup must exist before mutation"
    );
    assert_eq!(fs::read_to_string(&confirm_config)?, proposed);

    let wrong = run_watchdog_cli(
        &confirm_plan,
        &wrong_token_result,
        &confirm_backup_root,
        "confirm",
        Some("wrong-token"),
        1005,
    )?;
    assert!(!wrong.status.success(), "{row_id} wrong token must fail");
    assert!(!wrong_token_result.exists());
    assert_eq!(fs::read_to_string(&confirm_config)?, proposed);

    let confirmed = run_watchdog_cli(
        &confirm_plan,
        &confirm_result,
        &confirm_backup_root,
        "confirm",
        Some(&plan.recovery.confirmation_token),
        1005,
    )?;
    assert!(
        confirmed.status.success(),
        "{row_id} confirm failed: {}",
        String::from_utf8_lossy(&confirmed.stderr)
    );
    let confirmed_result = load_watchdog_result(&confirm_result)?;
    assert_eq!(confirmed_result.recovery.status, RecoveryStatus::Confirmed);
    assert_eq!(fs::read_to_string(&confirm_config)?, proposed);
    assert!(!confirmed_result.recovery.requires_visible_display);
    assert!(!confirmed_result.recovery.requires_mouse_input);
    assert!(!confirmed_result.recovery.requires_app_ui);
    assert!(!confirmed_result.recovery.requires_hyprland_keybind);
    assert!(!confirmed_result.recovery.reload_run);
    assert!(!confirmed_result.recovery.eval_run);
    assert!(!confirmed_result.recovery.lua_executed);
    assert!(!confirmed_result.recovery.active_config_modified);
    assert!(!confirmed_result.recovery.active_runtime_modified);

    let expire_dir = temp_case("expire")?;
    let expire_config = expire_dir.join("hyprland.conf");
    let expire_backup_root = expire_dir.join("backups");
    let expire_plan = expire_dir.join("watchdog-plan.json");
    let expire_result = expire_dir.join("watchdog-result.json");
    fs::write(&expire_config, &original)?;

    let expire_planner = HighRiskRecoveryPlanner::new(&expire_backup_root, 2000);
    let expire_watchdog = expire_planner.arm_watchdog_for_temp_config(
        &expire_config,
        &proposed,
        10,
        &expire_plan,
        &expire_result,
    )?;
    assert!(expire_plan.exists());
    assert!(expire_watchdog.recovery.backup_path.exists());
    assert_eq!(fs::read_to_string(&expire_config)?, proposed);

    let expired = run_watchdog_cli(
        &expire_plan,
        &expire_result,
        &expire_backup_root,
        "expire",
        None,
        2011,
    )?;
    assert!(
        expired.status.success(),
        "{row_id} expire failed: {}",
        String::from_utf8_lossy(&expired.stderr)
    );
    let reverted_result = load_watchdog_result(&expire_result)?;
    assert_eq!(reverted_result.recovery.status, RecoveryStatus::Reverted);
    assert!(reverted_result.recovery.restore_verified);
    assert_eq!(fs::read_to_string(&expire_config)?, original);
    assert!(!reverted_result.recovery.reload_run);
    assert!(!reverted_result.recovery.eval_run);
    assert!(!reverted_result.recovery.lua_executed);
    assert!(!reverted_result.recovery.active_config_modified);
    assert!(!reverted_result.recovery.active_runtime_modified);

    Ok(())
}

#[test]
fn cursor_hide_on_key_press_reports_are_proof_only_and_keep_counts() -> Result<()> {
    let coverage = read_json("data/reports/scalar-read-write-coverage.v0.55.2.json")?;
    let usability =
        read_json("data/reports/cursor-hide-on-key-press-usability-proof.v0.55.2.json")?;
    let watchdog = read_json("data/reports/cursor-hide-on-key-press-watchdog-proof.v0.55.2.json")?;
    let readiness = read_json("data/reports/cursor-hide-on-key-press-readiness.v0.55.2.json")?;

    assert_eq!(coverage["counts"]["writableRows"], 277);
    assert_eq!(coverage["counts"]["blockedWriteRows"], 64);
    assert_eq!(SAFE_WRITABLE_ROWS.len(), 277);
    assert!(!is_safe_writable_setting("cursor.hide_on_key_press"));

    for report in [&usability, &watchdog] {
        assert_eq!(report["counts"]["rows"], 1);
        assert_eq!(report["counts"]["rowsEnabled"], 0);
        assert_eq!(report["counts"]["finalWritableRows"], 277);
        assert_eq!(report["counts"]["finalBlockedRows"], 64);
        assert_eq!(report["counts"]["cursorInputBlockedRows"], 19);
        assert_eq!(report["counts"]["displayRenderBlockedRows"], 23);
        assert_eq!(report["counts"]["debugCrashBlockedRows"], 22);
        assert_eq!(report["counts"]["writeAllowlistChanged"], false);
        assert_eq!(report["counts"]["productionBehaviorChanged"], false);
        assert_eq!(report["counts"]["recoveryGateWeakenedRows"], 0);
        assert_eq!(report["counts"]["realConfigModified"], false);
        assert_eq!(report["counts"]["activeRuntimeModified"], false);
        assert_eq!(report["counts"]["reloadEvalLuaUsed"], false);
    }

    assert_eq!(readiness["counts"]["rowsReviewed"], 1);
    assert_eq!(readiness["counts"]["readyForLaterEnablementRows"], 1);
    assert_eq!(readiness["counts"]["rowsEnabled"], 0);
    assert_eq!(
        readiness["readinessDecision"].as_str(),
        Some("ready-for-later-enable-sprint-no-enablements-now")
    );

    Ok(())
}

#[test]
fn cursor_hide_on_key_press_reports_represent_exact_target_row() -> Result<()> {
    let usability =
        read_json("data/reports/cursor-hide-on-key-press-usability-proof.v0.55.2.json")?;
    let watchdog = read_json("data/reports/cursor-hide-on-key-press-watchdog-proof.v0.55.2.json")?;
    let readiness = read_json("data/reports/cursor-hide-on-key-press-readiness.v0.55.2.json")?;

    for report in [&usability, &watchdog, &readiness] {
        let rows = report["rows"].as_array().expect("rows");
        assert_eq!(rows.len(), 1);
        assert_eq!(rows[0]["rowId"].as_str(), Some("cursor.hide_on_key_press"));
        assert_eq!(rows[0]["currentWriteStatus"].as_str(), Some("high-risk"));
        assert_eq!(rows[0]["enabled"].as_bool(), Some(false));
    }

    let blocked = readiness["stillBlockedRows"]
        .as_array()
        .unwrap()
        .iter()
        .map(|row| row.as_str().unwrap())
        .collect::<BTreeSet<_>>();
    assert!(blocked.contains("cursor.hide_on_key_press"));
    assert!(blocked.contains("cursor.invisible"));
    assert!(blocked.contains("cursor.inactive_timeout"));

    Ok(())
}

#[test]
fn cursor_hide_on_key_press_preflight_and_keyboard_token_proof_pass() -> Result<()> {
    let usability =
        read_json("data/reports/cursor-hide-on-key-press-usability-proof.v0.55.2.json")?;
    let watchdog = read_json("data/reports/cursor-hide-on-key-press-watchdog-proof.v0.55.2.json")?;
    let readiness = read_json("data/reports/cursor-hide-on-key-press-readiness.v0.55.2.json")?;

    assert_eq!(usability["counts"]["validatorPassedRows"], 1);
    assert_eq!(usability["counts"]["invalidRejectionPassedRows"], 1);
    assert_eq!(usability["counts"]["fixtureReplacePassedRows"], 1);
    assert_eq!(usability["counts"]["fixtureAppendPassedRows"], 1);
    assert_eq!(usability["counts"]["singleMutationVerifiedRows"], 1);
    assert_eq!(usability["counts"]["hyprlandVerifyConfigPassedRows"], 1);
    assert_eq!(usability["counts"]["keyboardTokenUsabilityAnalyzedRows"], 1);
    assert_eq!(watchdog["counts"]["watchdogArmBeforeMutationPassedRows"], 1);
    assert_eq!(watchdog["counts"]["separateProcessConfirmPassedRows"], 1);
    assert_eq!(
        watchdog["counts"]["separateProcessTimeoutRestorePassedRows"],
        1
    );
    assert_eq!(watchdog["counts"]["wrongTokenFailedRows"], 1);
    assert_eq!(watchdog["counts"]["realConfigTargetRefusedInDryRunRows"], 1);
    assert_eq!(watchdog["counts"]["keyboardTokenUsabilityPassedRows"], 1);
    assert_eq!(watchdog["counts"]["recoveryIndependencePassedRows"], 1);

    let row = &usability["rows"][0];
    assert_eq!(row["validatorProof"]["passed"], true);
    assert_eq!(
        row["invalidRejectionProof"]["rejectedBeforeWritePlanning"],
        true
    );
    assert_eq!(row["tempConfigProof"]["passed"], true);
    assert_eq!(row["keyboardTokenUsabilityProof"]["passed"], true);
    assert_eq!(
        row["keyboardTokenUsabilityProof"]["confirmationCanBeTypedAsCliToken"],
        true
    );
    assert_eq!(
        row["keyboardTokenUsabilityProof"]["requiresVisibleCursor"],
        false
    );
    assert_eq!(
        row["keyboardTokenUsabilityProof"]["requiresMouseInput"],
        false
    );
    assert_eq!(row["keyboardTokenUsabilityProof"]["requiresAppUi"], false);
    assert_eq!(
        row["keyboardTokenUsabilityProof"]["requiresHyprlandKeybind"],
        false
    );

    let watchdog_row = &watchdog["rows"][0];
    assert_eq!(
        watchdog_row["watchdogArmBeforeMutationProof"]["planPersistedBeforeMutation"],
        true
    );
    assert_eq!(
        watchdog_row["watchdogArmBeforeMutationProof"]["backupExistsBeforeMutation"],
        true
    );
    assert_eq!(watchdog_row["separateProcessConfirmProof"]["passed"], true);
    assert_eq!(
        watchdog_row["separateProcessTimeoutRestoreProof"]["restoreVerifiedByFileReread"],
        true
    );
    assert_eq!(watchdog_row["wrongTokenFailureProof"]["passed"], true);
    assert_eq!(watchdog_row["dryRunRealConfigRefusalProof"]["passed"], true);
    assert_eq!(watchdog_row["keyboardTokenUsabilityProof"]["passed"], true);
    assert_eq!(
        watchdog_row["keyboardTokenUsabilityProof"]["usesKeyboardTextOnly"],
        true
    );
    assert_eq!(watchdog_row["recoveryIndependenceProof"]["passed"], true);

    let readiness_row = &readiness["rows"][0];
    assert_eq!(readiness_row["readyForLaterEnablementSprint"], true);
    assert_eq!(readiness_row["keyboardTokenUsabilityProofPassed"], true);
    assert_eq!(readiness_row["recoveryIndependenceProofPassed"], true);
    assert_eq!(readiness_row["reloadEvalLuaUsed"], false);
    assert_eq!(readiness_row["activeRuntimeModified"], false);

    Ok(())
}

#[test]
fn cursor_hide_on_key_press_watchdog_cli_proves_confirm_and_timeout() -> Result<()> {
    prove_watchdog_process_for_hide_on_key_press()
}

#[test]
fn cursor_hide_on_key_press_real_config_target_stays_refused_in_dry_run() -> Result<()> {
    let dir = temp_case("real-config-refusal")?;
    let config_path = dir.join("hyprland.conf");
    let backup_root = dir.join("backups");
    let plan_path = dir.join("watchdog-plan.json");
    let result_path = dir.join("watchdog-result.json");
    fs::write(&config_path, "cursor:hide_on_key_press = false\n")?;

    let planner = HighRiskRecoveryPlanner::new(&backup_root, 3000);
    let mut plan = planner.arm_watchdog_for_temp_config(
        &config_path,
        "cursor:hide_on_key_press = true\n",
        10,
        &plan_path,
        &result_path,
    )?;
    plan.recovery.target_config_path = PathBuf::from("/home/kyo/.config/hypr/hyprland.conf");
    fs::write(&plan_path, serde_json::to_vec_pretty(&plan)?)?;

    let refused = run_watchdog_cli(&plan_path, &result_path, &backup_root, "expire", None, 3011)?;
    assert!(!refused.status.success());
    assert!(!result_path.exists());

    Ok(())
}
