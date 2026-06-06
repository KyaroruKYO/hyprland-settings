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
use hyprland_settings::write_classification::SAFE_WRITABLE_ROWS;
use serde_json::Value;

fn read_json(path: &str) -> Result<Value> {
    Ok(serde_json::from_str(&fs::read_to_string(path)?)?)
}

fn temp_case(name: &str) -> Result<PathBuf> {
    let nanos = SystemTime::now().duration_since(UNIX_EPOCH)?.as_nanos();
    let dir = std::env::temp_dir().join(format!(
        "hyprland-settings-cursor-visibility-conditional-{name}-{}-{nanos}",
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

fn prove_watchdog_process_for_row(row_id: &str) -> Result<()> {
    let config_key = row_id.replace('.', ":");

    let confirm_dir = temp_case(&format!("{row_id}-confirm"))?;
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
    assert!(confirm_plan.exists());
    assert!(plan.recovery.backup_path.exists());
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

    let expire_dir = temp_case(&format!("{row_id}-expire"))?;
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
fn conditional_policy_reports_keep_counts_and_allowlist_unchanged() -> Result<()> {
    let coverage = read_json("data/reports/scalar-read-write-coverage.v0.55.2.json")?;
    let proof = read_json("data/reports/cursor-visibility-conditional-policy-proof.v0.55.2.json")?;
    let watchdog =
        read_json("data/reports/cursor-visibility-conditional-policy-watchdog-proof.v0.55.2.json")?;
    let readiness =
        read_json("data/reports/cursor-visibility-conditional-policy-readiness.v0.55.2.json")?;

    assert_eq!(coverage["counts"]["writableRows"], 278);
    assert_eq!(coverage["counts"]["blockedWriteRows"], 63);
    assert_eq!(SAFE_WRITABLE_ROWS.len(), 278);

    for report in [&proof, &watchdog] {
        assert_eq!(report["counts"]["rows"], 2);
        assert_eq!(report["counts"]["rowsEnabled"], 0);
        assert_eq!(report["counts"]["finalWritableRows"], 275);
        assert_eq!(report["counts"]["finalBlockedRows"], 66);
        assert_eq!(report["counts"]["cursorInputBlockedRows"], 21);
        assert_eq!(report["counts"]["displayRenderBlockedRows"], 23);
        assert_eq!(report["counts"]["debugCrashBlockedRows"], 22);
        assert_eq!(report["counts"]["writeAllowlistChanged"], false);
        assert_eq!(report["counts"]["productionBehaviorChanged"], false);
        assert_eq!(report["counts"]["recoveryGateWeakenedRows"], 0);
    }

    assert_eq!(readiness["counts"]["rowsReviewed"], 2);
    assert_eq!(readiness["counts"]["readyForLaterEnablementRows"], 2);
    assert_eq!(readiness["counts"]["notReadyRows"], 0);
    assert_eq!(readiness["counts"]["rowsEnabled"], 0);
    assert_eq!(
        readiness["readinessDecision"].as_str(),
        Some("both-rows-ready-for-later-enable-sprint-no-enablements-now")
    );

    Ok(())
}

#[test]
fn conditional_policy_reports_represent_only_touch_and_tablet_rows() -> Result<()> {
    let proof = read_json("data/reports/cursor-visibility-conditional-policy-proof.v0.55.2.json")?;
    let watchdog =
        read_json("data/reports/cursor-visibility-conditional-policy-watchdog-proof.v0.55.2.json")?;
    let readiness =
        read_json("data/reports/cursor-visibility-conditional-policy-readiness.v0.55.2.json")?;

    let expected = ["cursor.hide_on_touch", "cursor.hide_on_tablet"]
        .into_iter()
        .collect::<BTreeSet<_>>();

    for report in [&proof, &watchdog, &readiness] {
        let rows = report["rows"].as_array().expect("rows");
        assert_eq!(rows.len(), 2);
        let actual = rows
            .iter()
            .map(|row| row["rowId"].as_str().expect("row id"))
            .collect::<BTreeSet<_>>();
        assert_eq!(actual, expected);
        for row in rows {
            assert_eq!(row["currentWriteStatus"].as_str(), Some("high-risk"));
            assert_eq!(row["enabled"].as_bool(), Some(false));
        }
    }

    Ok(())
}

#[test]
fn conditional_policy_preflight_and_watchdog_proof_passes_for_both_rows() -> Result<()> {
    let proof = read_json("data/reports/cursor-visibility-conditional-policy-proof.v0.55.2.json")?;
    let watchdog =
        read_json("data/reports/cursor-visibility-conditional-policy-watchdog-proof.v0.55.2.json")?;
    let readiness =
        read_json("data/reports/cursor-visibility-conditional-policy-readiness.v0.55.2.json")?;

    assert_eq!(proof["counts"]["validatorPassedRows"], 2);
    assert_eq!(proof["counts"]["invalidRejectionPassedRows"], 2);
    assert_eq!(proof["counts"]["fixtureReplacePassedRows"], 2);
    assert_eq!(proof["counts"]["fixtureAppendPassedRows"], 2);
    assert_eq!(proof["counts"]["singleMutationVerifiedRows"], 2);
    assert_eq!(proof["counts"]["hyprlandVerifyConfigPassedRows"], 2);
    assert_eq!(watchdog["counts"]["watchdogArmBeforeMutationPassedRows"], 2);
    assert_eq!(watchdog["counts"]["separateProcessConfirmPassedRows"], 2);
    assert_eq!(
        watchdog["counts"]["separateProcessTimeoutRestorePassedRows"],
        2
    );
    assert_eq!(watchdog["counts"]["wrongTokenFailedRows"], 2);
    assert_eq!(watchdog["counts"]["realConfigTargetRefusedInDryRunRows"], 2);
    assert_eq!(watchdog["counts"]["recoveryIndependencePassedRows"], 2);

    for row in proof["rows"].as_array().expect("proof rows") {
        assert_eq!(row["validatorProof"]["passed"].as_bool(), Some(true));
        assert_eq!(
            row["invalidRejectionProof"]["rejectedBeforeWritePlanning"].as_bool(),
            Some(true)
        );
        assert_eq!(row["tempConfigProof"]["passed"].as_bool(), Some(true));
        assert_eq!(
            row["safeForLaterEnablementByPreflight"].as_bool(),
            Some(true)
        );
        assert_eq!(
            row["hyprmodMetadataUse"]["hyprmodRecoveryEvidenceFound"].as_bool(),
            Some(false)
        );
    }

    for row in watchdog["rows"].as_array().expect("watchdog rows") {
        assert_eq!(
            row["watchdogArmBeforeMutationProof"]["planPersistedBeforeMutation"].as_bool(),
            Some(true)
        );
        assert_eq!(
            row["watchdogArmBeforeMutationProof"]["backupExistsBeforeMutation"].as_bool(),
            Some(true)
        );
        assert_eq!(
            row["separateProcessConfirmProof"]["passed"].as_bool(),
            Some(true)
        );
        assert_eq!(
            row["separateProcessTimeoutRestoreProof"]["restoreVerifiedByFileReread"].as_bool(),
            Some(true)
        );
        assert_eq!(
            row["wrongTokenFailureProof"]["passed"].as_bool(),
            Some(true)
        );
        assert_eq!(
            row["dryRunRealConfigRefusalProof"]["passed"].as_bool(),
            Some(true)
        );
        let independence = &row["recoveryIndependenceProof"];
        assert_eq!(independence["passed"].as_bool(), Some(true));
        assert_eq!(independence["requiresVisibleCursor"].as_bool(), Some(false));
        assert_eq!(independence["requiresMouseInput"].as_bool(), Some(false));
        assert_eq!(independence["requiresAppUi"].as_bool(), Some(false));
        assert_eq!(
            independence["requiresHyprlandKeybind"].as_bool(),
            Some(false)
        );
        assert_eq!(independence["requiresPointerFocus"].as_bool(), Some(false));
        assert_eq!(
            independence["requiresWorkspaceFocus"].as_bool(),
            Some(false)
        );
        assert_eq!(
            independence["requiresNormalPointerBehavior"].as_bool(),
            Some(false)
        );
    }

    for row in readiness["rows"].as_array().expect("readiness rows") {
        assert_eq!(row["readyForLaterEnablementSprint"].as_bool(), Some(true));
        assert_eq!(row["validatorProofPassed"].as_bool(), Some(true));
        assert_eq!(row["invalidRejectionProofPassed"].as_bool(), Some(true));
        assert_eq!(
            row["watchdogArmBeforeMutationProofPassed"].as_bool(),
            Some(true)
        );
        assert_eq!(row["recoveryIndependenceProofPassed"].as_bool(), Some(true));
        assert_eq!(row["reloadEvalLuaUsed"].as_bool(), Some(false));
        assert_eq!(row["activeRuntimeModified"].as_bool(), Some(false));
    }

    Ok(())
}

#[test]
fn conditional_policy_watchdog_cli_proves_confirm_and_timeout_for_each_row() -> Result<()> {
    prove_watchdog_process_for_row("cursor.hide_on_touch")?;
    prove_watchdog_process_for_row("cursor.hide_on_tablet")?;
    Ok(())
}

#[test]
fn conditional_policy_real_config_target_stays_refused_in_dry_run() -> Result<()> {
    let dir = temp_case("real-config-refusal")?;
    let config_path = dir.join("hyprland.conf");
    let backup_root = dir.join("backups");
    let plan_path = dir.join("watchdog-plan.json");
    let result_path = dir.join("watchdog-result.json");
    fs::write(&config_path, "cursor:hide_on_touch = false\n")?;

    let planner = HighRiskRecoveryPlanner::new(&backup_root, 3000);
    let mut plan = planner.arm_watchdog_for_temp_config(
        &config_path,
        "cursor:hide_on_touch = true\n",
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
