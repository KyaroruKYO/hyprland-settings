use std::collections::{BTreeMap, BTreeSet};
use std::fs;
use std::path::PathBuf;

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
};
use hyprland_settings::high_risk_production_gate::{
    evaluate_high_risk_production_gate, HighRiskProductionGateDecisionKind,
    HighRiskProductionGateMode, HighRiskProductionGateProof, HighRiskProductionGateRequest,
};
use hyprland_settings::pending_change::stage_pending_change;
use hyprland_settings::write_classification::{
    high_risk_write_policy, is_high_risk_gated_writable_setting, is_safe_writable_setting,
    SAFE_WRITABLE_ROWS,
};
use hyprland_settings::write_flow::{
    apply_setting_change_with_backup_manager,
    apply_setting_change_with_backup_manager_and_production_gate,
};

fn temp_root(label: &str) -> PathBuf {
    std::env::temp_dir().join(format!(
        "hyprland-settings-accepted-high-risk-{label}-{}-{}",
        std::process::id(),
        std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .expect("system time should be after unix epoch")
            .as_nanos()
    ))
}

fn config_key(row_id: &str) -> String {
    row_id.replace('.', ":")
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

fn complete_production_proof(
    row_id: &str,
) -> anyhow::Result<(
    HighRiskProductionGateRequest,
    PathBuf,
    CurrentConfigSnapshot,
    String,
)> {
    let row = blocked_pre_enablement_rows()
        .iter()
        .find(|row| row.row_id == row_id)
        .expect("row should exist");
    let proposed = valid_pre_enablement_example(row);
    let previous = if proposed == "true" || proposed == "1" || proposed == "0" {
        "false".to_string()
    } else {
        proposed.clone()
    };
    let root = temp_root(row_id.replace('.', "-").as_str());
    fs::create_dir_all(&root)?;
    let target_path = root.join("hyprland.conf");
    let backup_path = root.join("hyprland.conf.recovery-backup");
    let baseline = format!("{} = {previous}\n", config_key(row_id));
    fs::write(&target_path, &baseline)?;
    let snapshot = snapshot_for(&target_path, &baseline);

    let plan = create_temp_recovery_plan(
        row.row_id,
        &proposed,
        Some(previous),
        target_path.clone(),
        backup_path,
        1_700_000_000,
        None,
        60,
    )?;
    let backup_proof = create_temp_config_backup(&plan)?;
    fs::write(
        &target_path,
        format!("{} = {proposed}\n", config_key(row_id)),
    )?;
    let rollback_proof = restore_temp_config_from_backup(&plan)?;

    let proof = HighRiskProductionGateProof {
        recovery_plan: plan.clone(),
        backup_proof: Some(backup_proof),
        rollback_proof: Some(rollback_proof),
        confirmation_token: Some(plan.confirmation_token.as_str().to_string()),
        explicit_high_risk_approval: true,
        monitor_name_oracle_proof: None,
    };
    let request = HighRiskProductionGateRequest {
        mode: HighRiskProductionGateMode::ProductionWrite,
        row_id: row.row_id.to_string(),
        official_setting: row.official_setting.to_string(),
        bucket: row.bucket.into(),
        requested_keep_apply: true,
        now_unix_seconds: plan.created_unix_seconds + 1,
        proof: Some(proof),
        runtime_oracle_proven: false,
    };
    Ok((request, root, snapshot, proposed))
}

#[test]
fn accepted_high_risk_rows_are_gated_writable_and_default_monitor_is_oracle_gated() {
    let rows = blocked_pre_enablement_rows();
    assert_eq!(rows.len(), 63);
    assert_eq!(SAFE_WRITABLE_ROWS.len(), 341);

    let mut counts = BTreeMap::new();
    for row in rows {
        assert!(
            is_safe_writable_setting(row.row_id),
            "{} should now be write-allowlisted",
            row.row_id
        );
        assert!(
            is_high_risk_gated_writable_setting(row.row_id),
            "{} should be gated high-risk writable",
            row.row_id
        );
        assert!(
            high_risk_write_policy(row.row_id).is_some(),
            "{} should project high-risk UI/recovery warning metadata",
            row.row_id
        );
        *counts.entry(row.bucket.as_str()).or_insert(0usize) += 1;
    }

    assert_eq!(counts.get("display/render"), Some(&23));
    assert_eq!(counts.get("cursor/input"), Some(&18));
    assert_eq!(counts.get("debug/crash"), Some(&22));
    assert!(is_safe_writable_setting("cursor.default_monitor"));
    assert!(is_high_risk_gated_writable_setting(
        "cursor.default_monitor"
    ));
}

#[test]
fn accepted_high_risk_rows_use_source_backed_pre_enablement_validation() {
    for row in blocked_pre_enablement_rows()
        .iter()
        .filter(|row| row.row_id != "cursor.default_monitor")
    {
        let path = PathBuf::from(format!("/tmp/{}.conf", row.row_id));
        let current = snapshot_for(
            &path,
            &format!(
                "{} = {}\n",
                config_key(row.row_id),
                valid_pre_enablement_example(row)
            ),
        )
        .value_for(row.official_setting);
        let valid = stage_pending_change(row.row_id, &current, valid_pre_enablement_example(row));
        assert!(
            valid.can_be_applied(),
            "valid value rejected for {}",
            row.row_id
        );

        let invalid = stage_pending_change(row.row_id, &current, "not-a-source-backed-value");
        assert!(
            !invalid.can_be_applied(),
            "invalid value should reject for {}",
            row.row_id
        );
    }
}

#[test]
fn production_gate_accepts_complete_approved_proof_for_62_rows() -> anyhow::Result<()> {
    let mut accepted = 0;
    for row in blocked_pre_enablement_rows()
        .iter()
        .filter(|row| row.row_id != "cursor.default_monitor")
    {
        let (request, root, _, _) = complete_production_proof(row.row_id)?;
        let evaluation = evaluate_high_risk_production_gate(request);
        assert_eq!(
            evaluation.decision.kind,
            HighRiskProductionGateDecisionKind::ProductionWriteAccepted,
            "{} should be accepted by the production gate with explicit approval and complete proof",
            row.row_id
        );
        accepted += 1;
        fs::remove_dir_all(root)?;
    }
    assert_eq!(accepted, 62);
    Ok(())
}

#[test]
fn default_apply_path_rejects_enabled_high_risk_rows_without_gate_proof() -> anyhow::Result<()> {
    let row = blocked_pre_enablement_rows()
        .iter()
        .find(|row| row.row_id == "debug.disable_logs")
        .expect("row should exist");
    let root = temp_root("default-apply-rejects");
    fs::create_dir_all(&root)?;
    let source = root.join("hyprland.conf");
    fs::write(&source, "debug:disable_logs = false\n")?;
    let snapshot = snapshot_for(&source, "debug:disable_logs = false\n");
    let backup_manager = BackupManager::new(root.join("backups"));

    let result = apply_setting_change_with_backup_manager(
        known_ids(),
        &discovery_for(source),
        &snapshot,
        row.row_id,
        &valid_pre_enablement_example(row),
        &backup_manager,
    );
    assert!(matches!(
        result,
        Err(failure)
            if failure
                .failures
                .contains(&"MissingHighRiskProductionGateProof".to_string())
    ));

    fs::remove_dir_all(root)?;
    Ok(())
}

#[test]
fn valid_gated_temp_fixture_write_path_succeeds_without_touching_real_config() -> anyhow::Result<()>
{
    let row_id = "debug.disable_logs";
    let (request, root, snapshot, proposed) = complete_production_proof(row_id)?;
    let proof = request.proof.clone().expect("proof should exist");
    let target_path = proof.recovery_plan.target_config_path.clone();
    assert!(target_path.starts_with(std::env::temp_dir()));
    assert!(!target_path.starts_with("/home/kyo/.config/hypr"));

    let backup_manager = BackupManager::new(root.join("apply-backups"));
    let outcome = apply_setting_change_with_backup_manager_and_production_gate(
        known_ids(),
        &discovery_for(target_path.clone()),
        &snapshot,
        row_id,
        &proposed,
        &backup_manager,
        Some(proof),
    )
    .map_err(|failure| anyhow::anyhow!("{failure:?}"))?;

    assert_eq!(outcome.setting_id, row_id);
    assert_eq!(outcome.target_path, target_path);
    assert_eq!(outcome.verified_value.as_deref(), Some(proposed.as_str()));
    assert!(fs::read_to_string(&outcome.target_path)?
        .contains(&format!("{} = {proposed}", config_key(row_id))));

    fs::remove_dir_all(root)?;
    Ok(())
}

#[test]
fn incomplete_gate_proof_rejects_production_write() -> anyhow::Result<()> {
    let (mut request, root, _, _) = complete_production_proof("debug.disable_logs")?;
    request
        .proof
        .as_mut()
        .expect("proof should exist")
        .rollback_proof = None;
    let evaluation = evaluate_high_risk_production_gate(request);
    assert_eq!(
        evaluation.decision.kind,
        HighRiskProductionGateDecisionKind::ProductionWriteRefused
    );
    assert!(evaluation
        .decision
        .errors
        .iter()
        .any(|error| error.to_string().contains("rollback proof")));
    fs::remove_dir_all(root)?;
    Ok(())
}

#[test]
fn aggregate_reports_record_341_writable_and_zero_blocked_rows() {
    let coverage: serde_json::Value = serde_json::from_slice(
        &fs::read("data/reports/scalar-read-write-coverage.v0.55.2.json")
            .expect("coverage report should exist"),
    )
    .expect("coverage report should parse");
    assert_eq!(coverage["counts"]["readableRows"], 341);
    assert_eq!(coverage["counts"]["writableRows"], 341);
    assert_eq!(coverage["counts"]["blockedWriteRows"], 0);

    let pipeline: serde_json::Value = serde_json::from_slice(
        &fs::read("data/reports/all-341-unified-pipeline.v0.55.2.json")
            .expect("pipeline report should exist"),
    )
    .expect("pipeline report should parse");
    assert_eq!(pipeline["counts"]["writableRows"], 341);
    assert_eq!(pipeline["counts"]["blockedRows"], 0);
}
