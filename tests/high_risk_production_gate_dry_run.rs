use std::collections::{BTreeMap, BTreeSet};
use std::fs;
use std::path::{Path, PathBuf};

use hyprland_settings::blocked_row_pre_enablement::{
    blocked_pre_enablement_rows, valid_pre_enablement_example,
};
use hyprland_settings::config_backup::BackupManager;
use hyprland_settings::config_discovery::{
    ConfigDiscovery, ConfigDiscoveryStatus, ConfigPathSource,
};
use hyprland_settings::config_parser::parse_hyprland_config_text;
use hyprland_settings::current_config::CurrentConfigSnapshot;
use hyprland_settings::high_risk_persisted_recovery::{
    create_temp_config_backup, create_temp_recovery_plan, restore_temp_config_from_backup,
    HighRiskRecoveryBucket,
};
use hyprland_settings::high_risk_production_gate::{
    evaluate_high_risk_production_gate, high_risk_production_gate_rows,
    HighRiskProductionGateDecisionKind, HighRiskProductionGateError, HighRiskProductionGateMode,
    HighRiskProductionGateProof, HighRiskProductionGateRequest,
};
use hyprland_settings::write_classification::config_key_from_official_setting;
use hyprland_settings::write_classification::{
    is_high_risk_gated_writable_setting, is_safe_writable_setting, SAFE_WRITABLE_ROWS,
};
use hyprland_settings::write_flow::apply_setting_change_with_backup_manager;
use serde_json::Value;

const DRY_RUN_REPORT: &str = "data/reports/high-risk-production-gate-dry-run.v0.55.2.json";
const DRY_RUN_TESTS_REPORT: &str =
    "data/reports/high-risk-production-gate-dry-run-tests.v0.55.2.json";
const DRY_RUN_BLOCKERS_REPORT: &str =
    "data/reports/high-risk-production-gate-dry-run-blockers.v0.55.2.json";
const UNIFIED_PIPELINE_REPORT: &str = "data/reports/all-341-unified-pipeline.v0.55.2.json";
const SCREEN_SHADER_CLOSURE_REPORT: &str = "data/reports/screen-shader-track-closure.v0.55.2.json";

fn temp_root(label: &str) -> PathBuf {
    let mut root = std::env::temp_dir();
    root.push(format!(
        "hyprland-settings-high-risk-gate-{label}-{}-{}",
        std::process::id(),
        std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .expect("system time should be after unix epoch")
            .as_nanos()
    ));
    root
}

fn complete_proof(
    row_id: &str,
) -> anyhow::Result<(
    HighRiskProductionGateRequest,
    PathBuf,
    String,
    HighRiskRecoveryBucket,
)> {
    let row = blocked_pre_enablement_rows()
        .into_iter()
        .find(|row| row.row_id == row_id)
        .expect("test row should be in blocked pre-enablement list");
    let key = config_key_from_official_setting(row.official_setting);
    let proposed = valid_pre_enablement_example(row);
    let previous = if proposed == "1" {
        "0".to_string()
    } else {
        proposed.clone()
    };

    let root = temp_root(row_id.replace('.', "-").replace(':', "-").as_str());
    fs::create_dir_all(&root)?;
    let target_path = root.join("hyprland.conf");
    let backup_path = root.join("hyprland.conf.backup");
    fs::write(&target_path, format!("{key} = {previous}\n"))?;

    let plan = create_temp_recovery_plan(
        row.row_id,
        &proposed,
        Some(previous.clone()),
        target_path.clone(),
        backup_path,
        1_700_000_000,
        None,
        60,
    )?;
    let backup_proof = create_temp_config_backup(&plan)?;
    fs::write(&target_path, format!("{key} = {proposed}\n"))?;
    let rollback_proof = restore_temp_config_from_backup(&plan)?;

    let request = HighRiskProductionGateRequest {
        mode: HighRiskProductionGateMode::ReportOnlyDryRun,
        row_id: row.row_id.to_string(),
        official_setting: row.official_setting.to_string(),
        bucket: row.bucket.into(),
        requested_keep_apply: true,
        now_unix_seconds: plan.created_unix_seconds + 1,
        proof: Some(HighRiskProductionGateProof {
            recovery_plan: plan.clone(),
            backup_proof: Some(backup_proof),
            rollback_proof: Some(rollback_proof),
            confirmation_token: Some(plan.confirmation_token.as_str().to_string()),
            explicit_high_risk_approval: false,
        }),
        runtime_oracle_proven: false,
    };

    Ok((
        request,
        root,
        row.official_setting.to_string(),
        row.bucket.into(),
    ))
}

fn assert_error_contains(
    errors: &[HighRiskProductionGateError],
    expected: impl Fn(&HighRiskProductionGateError) -> bool,
) {
    assert!(
        errors.iter().any(expected),
        "expected error was missing from {errors:?}"
    );
}

fn read_json(path: &str) -> Value {
    let bytes = fs::read(path).unwrap_or_else(|error| panic!("failed to read {path}: {error}"));
    serde_json::from_slice(&bytes).unwrap_or_else(|error| panic!("failed to parse {path}: {error}"))
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

#[test]
fn dry_run_gate_evaluates_all_blocked_rows_and_preserves_bucket_counts() {
    let evaluations = high_risk_production_gate_rows();
    assert_eq!(evaluations.len(), 63);

    let mut counts: BTreeMap<&str, usize> = BTreeMap::new();
    for evaluation in &evaluations {
        *counts.entry(evaluation.bucket.as_str()).or_default() += 1;
        assert_eq!(
            evaluation.decision.kind,
            HighRiskProductionGateDecisionKind::ProductionWriteRefused
        );
        if evaluation.row_id == "cursor.default_monitor" {
            assert!(evaluation
                .decision
                .errors
                .contains(&HighRiskProductionGateError::ProductionWriteDisabled));
            assert!(!evaluation.is_safe_writable_setting);
        } else {
            assert!(evaluation
                .decision
                .errors
                .contains(&HighRiskProductionGateError::MissingRecoveryPlan));
            assert!(evaluation.is_safe_writable_setting);
        }
    }

    assert_eq!(counts.get("display/render"), Some(&23));
    assert_eq!(counts.get("cursor/input"), Some(&18));
    assert_eq!(counts.get("debug/crash"), Some(&22));
    assert!(evaluations
        .iter()
        .any(|row| row.row_id == "cursor.default_monitor" && row.runtime_dynamic_special_case));
}

#[test]
fn complete_temp_only_scaffold_proof_is_accepted_for_non_runtime_dynamic_rows() -> anyhow::Result<()>
{
    let mut accepted = 0;
    for row in blocked_pre_enablement_rows() {
        let (request, root, _, _) = complete_proof(row.row_id)?;
        let evaluation = evaluate_high_risk_production_gate(request);
        if row.row_id == "cursor.default_monitor" {
            assert_eq!(
                evaluation.decision.kind,
                HighRiskProductionGateDecisionKind::ReportOnlyDryRunRejected
            );
            assert_error_contains(&evaluation.decision.errors, |error| {
                matches!(
                    error,
                    HighRiskProductionGateError::RuntimeDynamicOracleMissing
                )
            });
        } else {
            assert_eq!(
                evaluation.decision.kind,
                HighRiskProductionGateDecisionKind::ReportOnlyDryRunAccepted,
                "row {} should accept complete report-only scaffold proof",
                row.row_id
            );
            accepted += 1;
        }
        fs::remove_dir_all(root)?;
    }
    assert_eq!(accepted, 62);
    Ok(())
}

#[test]
fn runtime_dynamic_default_monitor_can_only_accept_when_runtime_oracle_is_proven(
) -> anyhow::Result<()> {
    let (mut request, root, _, _) = complete_proof("cursor.default_monitor")?;
    let rejected = evaluate_high_risk_production_gate(request.clone());
    assert_eq!(
        rejected.decision.kind,
        HighRiskProductionGateDecisionKind::ReportOnlyDryRunRejected
    );
    assert_error_contains(&rejected.decision.errors, |error| {
        matches!(
            error,
            HighRiskProductionGateError::RuntimeDynamicOracleMissing
        )
    });

    request.runtime_oracle_proven = true;
    let accepted = evaluate_high_risk_production_gate(request);
    assert_eq!(
        accepted.decision.kind,
        HighRiskProductionGateDecisionKind::ReportOnlyDryRunAccepted
    );
    assert!(accepted.runtime_dynamic_special_case);

    fs::remove_dir_all(root)?;
    Ok(())
}

#[test]
fn dry_run_gate_rejects_missing_or_incomplete_scaffold_proof() -> anyhow::Result<()> {
    let (request, root, _, _) = complete_proof("debug.disable_logs")?;

    let mut missing_plan = request.clone();
    missing_plan.proof = None;
    let evaluation = evaluate_high_risk_production_gate(missing_plan);
    assert_error_contains(&evaluation.decision.errors, |error| {
        matches!(error, HighRiskProductionGateError::MissingRecoveryPlan)
    });

    let mut missing_backup = request.clone();
    missing_backup
        .proof
        .as_mut()
        .expect("proof should exist")
        .backup_proof = None;
    let evaluation = evaluate_high_risk_production_gate(missing_backup);
    assert_error_contains(&evaluation.decision.errors, |error| {
        matches!(error, HighRiskProductionGateError::MissingBackupProof)
    });

    let mut missing_rollback = request.clone();
    missing_rollback
        .proof
        .as_mut()
        .expect("proof should exist")
        .rollback_proof = None;
    let evaluation = evaluate_high_risk_production_gate(missing_rollback);
    assert_error_contains(&evaluation.decision.errors, |error| {
        matches!(error, HighRiskProductionGateError::MissingRollbackProof)
    });

    let mut missing_confirmation = request.clone();
    missing_confirmation
        .proof
        .as_mut()
        .expect("proof should exist")
        .confirmation_token = None;
    let evaluation = evaluate_high_risk_production_gate(missing_confirmation);
    assert_error_contains(&evaluation.decision.errors, |error| {
        matches!(error, HighRiskProductionGateError::MissingConfirmationProof)
    });

    fs::remove_dir_all(root)?;
    Ok(())
}

#[test]
fn dry_run_gate_rejects_bad_confirmation_and_timeout_keep_apply() -> anyhow::Result<()> {
    let (mut wrong_token, root, _, _) = complete_proof("debug.damage_tracking")?;
    wrong_token
        .proof
        .as_mut()
        .expect("proof should exist")
        .confirmation_token = Some("wrong-token".to_string());
    let evaluation = evaluate_high_risk_production_gate(wrong_token);
    assert_error_contains(&evaluation.decision.errors, |error| {
        matches!(error, HighRiskProductionGateError::WrongConfirmationToken)
    });

    let (mut timed_out, timeout_root, _, _) = complete_proof("debug.damage_tracking")?;
    timed_out.now_unix_seconds = timed_out
        .proof
        .as_ref()
        .expect("proof should exist")
        .recovery_plan
        .confirmation_deadline_unix_seconds
        + 1;
    timed_out
        .proof
        .as_mut()
        .expect("proof should exist")
        .confirmation_token = None;
    let evaluation = evaluate_high_risk_production_gate(timed_out);
    assert_error_contains(&evaluation.decision.errors, |error| {
        matches!(
            error,
            HighRiskProductionGateError::TimeoutNoConfirmationForKeepApply
        )
    });

    fs::remove_dir_all(root)?;
    fs::remove_dir_all(timeout_root)?;
    Ok(())
}

#[test]
fn dry_run_gate_rejects_mismatched_row_setting_bucket_and_paths() -> anyhow::Result<()> {
    let (mut row_mismatch, row_root, _, _) = complete_proof("render.direct_scanout")?;
    row_mismatch
        .proof
        .as_mut()
        .expect("proof should exist")
        .recovery_plan
        .row_id = "debug.disable_logs".to_string();
    let evaluation = evaluate_high_risk_production_gate(row_mismatch);
    assert_error_contains(&evaluation.decision.errors, |error| {
        matches!(error, HighRiskProductionGateError::RowMismatch { .. })
    });

    let (mut setting_mismatch, setting_root, _, _) = complete_proof("render.direct_scanout")?;
    setting_mismatch.official_setting = "debug:disable_logs".to_string();
    let evaluation = evaluate_high_risk_production_gate(setting_mismatch);
    assert_error_contains(&evaluation.decision.errors, |error| {
        matches!(
            error,
            HighRiskProductionGateError::OfficialSettingMismatch { .. }
        )
    });

    let (mut bucket_mismatch, bucket_root, _, _) = complete_proof("render.direct_scanout")?;
    bucket_mismatch.bucket = HighRiskRecoveryBucket::DebugCrash;
    let evaluation = evaluate_high_risk_production_gate(bucket_mismatch);
    assert_error_contains(&evaluation.decision.errors, |error| {
        matches!(error, HighRiskProductionGateError::BucketMismatch { .. })
    });

    let (mut non_temp, non_temp_root, _, _) = complete_proof("render.direct_scanout")?;
    non_temp
        .proof
        .as_mut()
        .expect("proof should exist")
        .recovery_plan
        .target_config_path = PathBuf::from("/home/kyo/.config/hypr/hyprland.conf");
    let evaluation = evaluate_high_risk_production_gate(non_temp);
    assert_error_contains(&evaluation.decision.errors, |error| {
        matches!(
            error,
            HighRiskProductionGateError::RecoveryPlanInvalid(reason)
                if reason == "non-temp-target-path"
        )
    });

    fs::remove_dir_all(row_root)?;
    fs::remove_dir_all(setting_root)?;
    fs::remove_dir_all(bucket_root)?;
    fs::remove_dir_all(non_temp_root)?;
    Ok(())
}

#[test]
fn dry_run_gate_rejects_live_execution_enabled() -> anyhow::Result<()> {
    let (mut request, root, _, _) = complete_proof("cursor.invisible")?;
    request
        .proof
        .as_mut()
        .expect("proof should exist")
        .recovery_plan
        .live_execution_enabled = true;
    let evaluation = evaluate_high_risk_production_gate(request);
    assert_error_contains(&evaluation.decision.errors, |error| {
        matches!(error, HighRiskProductionGateError::LiveExecutionEnabled)
    });

    fs::remove_dir_all(root)?;
    Ok(())
}

#[test]
fn production_write_mode_refuses_all_rows_even_with_complete_scaffold_proof() -> anyhow::Result<()>
{
    let mut production_refused = 0;
    for row in blocked_pre_enablement_rows() {
        let (mut request, root, _, _) = complete_proof(row.row_id)?;
        request.mode = HighRiskProductionGateMode::ProductionWrite;
        if row.row_id == "cursor.default_monitor" {
            request.runtime_oracle_proven = true;
        }
        let evaluation = evaluate_high_risk_production_gate(request);
        assert_eq!(
            evaluation.decision.kind,
            HighRiskProductionGateDecisionKind::ProductionWriteRefused,
            "row {} should be refused in ProductionWrite mode",
            row.row_id
        );
        assert_error_contains(&evaluation.decision.errors, |error| {
            matches!(error, HighRiskProductionGateError::ProductionWriteDisabled)
        });
        production_refused += 1;
        fs::remove_dir_all(root)?;
    }
    assert_eq!(production_refused, 63);
    Ok(())
}

#[test]
fn current_write_allowlist_and_apply_path_still_refuse_all_blocked_rows() {
    assert_eq!(SAFE_WRITABLE_ROWS.len(), 340);

    let blocked_rows = blocked_pre_enablement_rows();
    for row in blocked_rows {
        if row.row_id == "cursor.default_monitor" {
            assert!(
                !is_safe_writable_setting(row.row_id),
                "{} must remain outside SAFE_WRITABLE_ROWS",
                row.row_id
            );
        } else {
            assert!(
                is_safe_writable_setting(row.row_id),
                "{} should now be allowlisted only through the high-risk gate",
                row.row_id
            );
            assert!(
                is_high_risk_gated_writable_setting(row.row_id),
                "{} should be classified as a high-risk gated writable row",
                row.row_id
            );
        }
    }

    let known_setting_ids: BTreeSet<String> = blocked_rows
        .iter()
        .map(|row| row.row_id.to_string())
        .collect();
    let temp_dir = temp_root("apply-refusal");
    fs::create_dir_all(&temp_dir).expect("temp backup root should be creatable");
    let backup_manager = BackupManager::new(temp_dir.clone());

    for row in blocked_rows {
        let source = temp_dir.join(format!("{}.conf", row.row_id.replace('.', "-")));
        let contents = format!(
            "{} = {}\n",
            config_key_from_official_setting(row.official_setting),
            valid_pre_enablement_example(row)
        );
        fs::write(&source, &contents).expect("temp config should be writable");
        let discovery = discovery_for(source.clone());
        let current_config = snapshot_for(&source, &contents);
        let result = apply_setting_change_with_backup_manager(
            known_setting_ids.clone(),
            &discovery,
            &current_config,
            row.row_id,
            &valid_pre_enablement_example(row),
            &backup_manager,
        );
        if row.row_id == "cursor.default_monitor" {
            assert!(
                matches!(result, Err(failure) if failure.failures.contains(&"NotAllowlisted".to_string())),
                "write flow should keep {} blocked before any write path",
                row.row_id
            );
        } else {
            assert!(
                matches!(result, Err(failure) if failure.failures.contains(&"MissingHighRiskProductionGateProof".to_string())),
                "write flow should reject {} without high-risk production gate proof",
                row.row_id
            );
        }
    }

    fs::remove_dir_all(temp_dir).expect("temp backup root should be removable");
}

#[test]
fn no_complete_dry_run_acceptance_touches_real_config_paths() -> anyhow::Result<()> {
    let (request, root, _, _) = complete_proof("render.direct_scanout")?;
    let proof = request.proof.as_ref().expect("proof should exist");
    assert!(proof.recovery_plan.target_config_path.starts_with(&root));
    assert!(proof.recovery_plan.backup_config_path.starts_with(&root));
    assert!(!proof
        .recovery_plan
        .target_config_path
        .starts_with("/home/kyo/.config/hypr"));
    assert!(!proof
        .recovery_plan
        .backup_config_path
        .starts_with("/home/kyo/.config/hypr"));

    let evaluation = evaluate_high_risk_production_gate(request);
    assert_eq!(
        evaluation.decision.kind,
        HighRiskProductionGateDecisionKind::ReportOnlyDryRunAccepted
    );

    fs::remove_dir_all(root)?;
    Ok(())
}

#[test]
fn screen_shader_track_remains_closed_by_default() {
    let report = fs::read_to_string(SCREEN_SHADER_CLOSURE_REPORT)
        .expect("screen shader closure report should exist");
    assert!(report.contains("\"screenShaderTrackClosedForNow\": true"));
    assert!(report.contains("\"nextScreenShaderWorkPolicy\""));
}

#[test]
fn aggregate_reports_link_to_high_risk_production_gate_dry_run_reports() {
    for path in [
        DRY_RUN_REPORT,
        DRY_RUN_TESTS_REPORT,
        DRY_RUN_BLOCKERS_REPORT,
    ] {
        assert!(
            Path::new(path).exists(),
            "required dry-run gate report should exist: {path}"
        );
    }

    let unified =
        fs::read_to_string(UNIFIED_PIPELINE_REPORT).expect("unified pipeline report should exist");
    assert!(unified.contains("high-risk-production-gate-dry-run.v0.55.2.json"));
    assert!(unified.contains("high-risk-production-gate-dry-run-blockers.v0.55.2.json"));
}

#[test]
fn dry_run_gate_reports_record_required_counts_and_blockers() {
    let report = read_json(DRY_RUN_REPORT);
    assert_eq!(report["rowsAnalyzed"], 63);
    assert_eq!(report["rowsDryRunAccepted"], 62);
    assert_eq!(report["rowsDryRunRejected"], 1);
    assert_eq!(report["rowsProductionWriteRefused"], 63);
    assert_eq!(report["rowsEnabledThisSprint"], 0);
    assert_eq!(report["safeWritableRowsChanged"], false);
    assert_eq!(report["writeAllowlistChanged"], false);
    assert_eq!(report["countsBefore"]["writable"], 278);
    assert_eq!(report["countsAfter"]["blocked"], 63);
    assert_eq!(
        report["sourceModuleAdded"],
        "src/high_risk_production_gate.rs"
    );
    assert_eq!(
        report["testsAdded"],
        "tests/high_risk_production_gate_dry_run.rs"
    );
    assert_eq!(report["bucketCoverage"]["display/render"], 23);
    assert_eq!(report["bucketCoverage"]["cursor/input"], 18);
    assert_eq!(report["bucketCoverage"]["debug/crash"], 22);

    let rows = report["rowEvaluations"]
        .as_array()
        .expect("row evaluations should be an array");
    assert_eq!(rows.len(), 63);
    for row in rows {
        assert_eq!(
            row["productionWriteStatus"],
            "refused-production-write-disabled"
        );
        assert_eq!(row["isSafeWritableSetting"], false);
        assert_eq!(row["enabledThisSprint"], false);
        assert!(row["exactRemainingBlocker"]
            .as_str()
            .expect("blocker should be a string")
            .contains("explicit high-risk enablement approval missing"));
    }
    let default_monitor = rows
        .iter()
        .find(|row| row["rowId"] == "cursor.default_monitor")
        .expect("cursor.default_monitor should be classified");
    assert_eq!(
        default_monitor["dryRunGateStatus"],
        "rejected-runtime-dynamic-oracle-missing"
    );
    assert_eq!(default_monitor["runtimeDynamicSpecialCase"], true);

    let blockers = read_json(DRY_RUN_BLOCKERS_REPORT);
    assert_eq!(blockers["rowsStillBlocked"], 63);
    assert_eq!(
        blockers["blockerCategories"]["dynamicRuntimeState"]
            .as_array()
            .expect("dynamic runtime state should be an array")
            .len(),
        1
    );
    assert_eq!(
        blockers["blockerCategories"]["missingExplicitApproval"]
            .as_array()
            .expect("missing approval should be an array")
            .len(),
        63
    );

    let tests = read_json(DRY_RUN_TESTS_REPORT);
    assert!(
        tests["tests"]
            .as_array()
            .expect("test report should list tests")
            .len()
            >= 12
    );
}
