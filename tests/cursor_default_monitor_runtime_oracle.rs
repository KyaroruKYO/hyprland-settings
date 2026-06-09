use std::collections::BTreeSet;
use std::fs;
use std::path::PathBuf;

use hyprland_settings::blocked_row_pre_enablement::blocked_pre_enablement_row;
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
use hyprland_settings::monitor_name_oracle::{
    parse_hyprctl_monitors_snapshot, validate_monitor_name_candidate, MockMonitorNameOracle,
    MonitorNameCandidate, MonitorNameOracle, MonitorNameOracleError, MonitorNameSnapshot,
    MonitorNameSnapshotSource, MonitorNameValidationStatus, ReadOnlyHyprctlMonitorNameOracle,
};
use hyprland_settings::pending_change::{
    stage_pending_change, stage_pending_change_with_sources, PendingChangeValueSources,
};
use hyprland_settings::write_classification::{
    high_risk_write_policy, is_high_risk_gated_writable_setting, is_safe_writable_setting,
    safe_writable_value_kind, ScalarWriteValueKind, SAFE_WRITABLE_ROWS,
};
use hyprland_settings::write_flow::{
    apply_setting_change_with_backup_manager,
    apply_setting_change_with_backup_manager_and_production_gate,
};

const ORACLE_RESEARCH_REPORT: &str =
    "data/reports/cursor-default-monitor-runtime-oracle-research.v0.55.2.json";

fn temp_root(label: &str) -> PathBuf {
    std::env::temp_dir().join(format!(
        "hyprland-settings-cursor-default-monitor-{label}-{}-{}",
        std::process::id(),
        std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .expect("system time should be after unix epoch")
            .as_nanos()
    ))
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

fn current_value(
    path: &PathBuf,
    value: &str,
) -> hyprland_settings::current_config::CurrentValueProjection {
    snapshot_for(path, &format!("cursor:default_monitor = {value}\n"))
        .value_for("cursor.default_monitor")
}

fn complete_default_monitor_proof(
    target_path: PathBuf,
    proposed: &str,
    previous: &str,
) -> anyhow::Result<HighRiskProductionGateProof> {
    let row = blocked_pre_enablement_row("cursor.default_monitor").expect("row should exist");
    let backup_path = target_path.with_extension("recovery-backup");
    fs::write(
        &target_path,
        format!("cursor:default_monitor = {previous}\n"),
    )?;
    let oracle = MockMonitorNameOracle::from_names(["DP-1", "eDP-1"])?;
    let monitor_name_oracle_proof =
        oracle.validate_candidate(MonitorNameCandidate::new(proposed))?;
    assert!(monitor_name_oracle_proof.accepted());
    let plan = create_temp_recovery_plan(
        row.row_id,
        proposed,
        Some(previous.to_string()),
        target_path.clone(),
        backup_path,
        1_700_000_000,
        None,
        60,
    )?;
    let backup_proof = create_temp_config_backup(&plan)?;
    fs::write(
        &target_path,
        format!("cursor:default_monitor = {proposed}\n"),
    )?;
    let rollback_proof = restore_temp_config_from_backup(&plan)?;
    Ok(HighRiskProductionGateProof {
        recovery_plan: plan.clone(),
        backup_proof: Some(backup_proof),
        rollback_proof: Some(rollback_proof),
        confirmation_token: Some(plan.confirmation_token.as_str().to_string()),
        explicit_high_risk_approval: true,
        monitor_name_oracle_proof: Some(monitor_name_oracle_proof),
    })
}

#[test]
fn fixture_monitor_oracle_accepts_current_monitor_names() {
    let snapshot = MonitorNameSnapshot::from_names(
        MonitorNameSnapshotSource::Fixture,
        ["DP-1", "eDP-1", "HDMI-A-1"],
    )
    .expect("fixture names should parse");
    assert_eq!(
        validate_monitor_name_candidate("DP-1", &snapshot).status,
        MonitorNameValidationStatus::Accepted
    );
    assert_eq!(
        validate_monitor_name_candidate("eDP-1", &snapshot).status,
        MonitorNameValidationStatus::Accepted
    );
}

#[test]
fn fixture_monitor_oracle_rejects_missing_stale_and_unsafe_monitor_names() {
    let snapshot = MonitorNameSnapshot::from_names(MonitorNameSnapshotSource::Fixture, ["DP-1"])
        .expect("fixture names should parse");
    assert_eq!(
        validate_monitor_name_candidate("", &snapshot).status,
        MonitorNameValidationStatus::Missing
    );
    assert_eq!(
        validate_monitor_name_candidate("HDMI-A-1", &snapshot).status,
        MonitorNameValidationStatus::Stale
    );
    assert_eq!(
        validate_monitor_name_candidate("DP-1; hyprctl reload", &snapshot).status,
        MonitorNameValidationStatus::UnsafeSyntax
    );
    assert_eq!(
        validate_monitor_name_candidate("../DP-1", &snapshot).status,
        MonitorNameValidationStatus::UnsafeSyntax
    );
}

#[test]
fn hyprctl_monitors_fixture_parser_extracts_monitor_names_without_live_query() {
    let fixture = "\
Monitor eDP-1 (ID 0):
\t1920x1080@60.00000 at 0x0
Monitor DP-1 (ID 1):
\t2560x1440@144.00000 at 1920x0
";
    let snapshot = parse_hyprctl_monitors_snapshot(fixture, MonitorNameSnapshotSource::Fixture)
        .expect("fixture should parse");
    assert_eq!(
        snapshot.monitor_names(),
        &["DP-1".to_string(), "eDP-1".to_string()]
    );
}

#[test]
fn monitor_snapshot_parser_rejects_empty_malformed_and_unsafe_snapshots() {
    assert!(matches!(
        parse_hyprctl_monitors_snapshot(
            "active workspace: 1\n",
            MonitorNameSnapshotSource::Fixture
        ),
        Err(MonitorNameOracleError::EmptySnapshot)
    ));
    assert!(matches!(
        parse_hyprctl_monitors_snapshot("Monitor \n", MonitorNameSnapshotSource::Fixture),
        Err(MonitorNameOracleError::MalformedMonitorLine(_))
    ));
    assert!(matches!(
        parse_hyprctl_monitors_snapshot(
            "Monitor ../DP-1 (ID 0):\n",
            MonitorNameSnapshotSource::Fixture
        ),
        Err(MonitorNameOracleError::UnsafeMonitorName(_))
    ));
}

#[test]
fn duplicate_monitor_names_are_deduplicated_deterministically() {
    let snapshot = parse_hyprctl_monitors_snapshot(
        "Monitor DP-1 (ID 0):\nMonitor eDP-1 (ID 1):\nMonitor DP-1 (ID 2):\n",
        MonitorNameSnapshotSource::Fixture,
    )
    .expect("fixture should parse");
    assert_eq!(
        snapshot.monitor_names(),
        &["DP-1".to_string(), "eDP-1".to_string()]
    );
}

#[test]
fn stale_monitor_names_reject_after_snapshot_refresh() {
    let old_snapshot = MonitorNameSnapshot::from_names(MonitorNameSnapshotSource::Mock, ["DP-1"])
        .expect("old fixture should parse");
    let new_snapshot = MonitorNameSnapshot::from_names(MonitorNameSnapshotSource::Mock, ["eDP-1"])
        .expect("new fixture should parse");
    assert_eq!(
        validate_monitor_name_candidate("DP-1", &old_snapshot).status,
        MonitorNameValidationStatus::Accepted
    );
    assert_eq!(
        validate_monitor_name_candidate("DP-1", &new_snapshot).status,
        MonitorNameValidationStatus::Stale
    );
}

#[test]
fn read_only_hyprctl_adapter_is_non_mutating_and_fixture_parsed() {
    let adapter = ReadOnlyHyprctlMonitorNameOracle::new("hyprctl");
    assert_eq!(adapter.command_args(), &["monitors"]);
    let snapshot = adapter
        .parse_read_only_output("Monitor DP-1 (ID 0):\n\tmake: Example\n\tmodel: Panel\n")
        .expect("read-only fixture output should parse");
    assert_eq!(
        snapshot.source(),
        MonitorNameSnapshotSource::ReadOnlyHyprctl
    );
    assert_eq!(snapshot.monitor_names(), &["DP-1".to_string()]);
}

#[test]
fn cursor_default_monitor_is_gated_writable_and_not_generic_freeform() {
    assert_eq!(SAFE_WRITABLE_ROWS.len(), 341);
    assert!(is_safe_writable_setting("cursor.default_monitor"));
    assert!(is_high_risk_gated_writable_setting(
        "cursor.default_monitor"
    ));
    assert_eq!(
        safe_writable_value_kind("cursor.default_monitor"),
        Some(ScalarWriteValueKind::MonitorName)
    );
    assert!(high_risk_write_policy("cursor.default_monitor")
        .expect("policy should exist")
        .review_warning
        .contains("runtime monitor-name oracle proof"));

    let path = PathBuf::from("/tmp/cursor-default-monitor.conf");
    let current = current_value(&path, "eDP-1");
    let missing_oracle = stage_pending_change("cursor.default_monitor", &current, "DP-1");
    assert!(!missing_oracle.can_be_applied());
    assert!(
        format!("{:?}", missing_oracle.validation).contains("runtime monitor-name oracle proof")
    );

    let sources = PendingChangeValueSources {
        monitor_names: vec!["DP-1".to_string(), "eDP-1".to_string()],
    };
    let valid =
        stage_pending_change_with_sources("cursor.default_monitor", &current, "DP-1", &sources);
    assert!(valid.can_be_applied());

    let stale =
        stage_pending_change_with_sources("cursor.default_monitor", &current, "HDMI-A-1", &sources);
    assert!(!stale.can_be_applied());

    let unsafe_name = stage_pending_change_with_sources(
        "cursor.default_monitor",
        &current,
        "DP-1; hyprctl reload",
        &sources,
    );
    assert!(!unsafe_name.can_be_applied());
}

#[test]
fn production_gate_requires_monitor_name_oracle_proof() -> anyhow::Result<()> {
    let root = temp_root("gate");
    fs::create_dir_all(&root)?;
    let target_path = root.join("hyprland.conf");
    let mut proof = complete_default_monitor_proof(target_path, "DP-1", "eDP-1")?;
    let row = blocked_pre_enablement_row("cursor.default_monitor").expect("row should exist");

    let accepted = evaluate_high_risk_production_gate(HighRiskProductionGateRequest {
        mode: HighRiskProductionGateMode::ProductionWrite,
        row_id: row.row_id.to_string(),
        official_setting: row.official_setting.to_string(),
        bucket: row.bucket.into(),
        requested_keep_apply: true,
        now_unix_seconds: proof.recovery_plan.created_unix_seconds + 1,
        runtime_oracle_proven: true,
        proof: Some(proof.clone()),
    });
    assert_eq!(
        accepted.decision.kind,
        HighRiskProductionGateDecisionKind::ProductionWriteAccepted
    );

    proof.monitor_name_oracle_proof = None;
    let rejected = evaluate_high_risk_production_gate(HighRiskProductionGateRequest {
        mode: HighRiskProductionGateMode::ProductionWrite,
        row_id: row.row_id.to_string(),
        official_setting: row.official_setting.to_string(),
        bucket: row.bucket.into(),
        requested_keep_apply: true,
        now_unix_seconds: proof.recovery_plan.created_unix_seconds + 1,
        runtime_oracle_proven: false,
        proof: Some(proof),
    });
    assert_eq!(
        rejected.decision.kind,
        HighRiskProductionGateDecisionKind::ProductionWriteRefused
    );
    assert!(rejected
        .decision
        .errors
        .iter()
        .any(|error| error.to_string().contains("runtime-dynamic oracle proof")));

    fs::remove_dir_all(root)?;
    Ok(())
}

#[test]
fn valid_oracle_and_gate_proof_accepts_temp_fixture_write() -> anyhow::Result<()> {
    let root = temp_root("write");
    fs::create_dir_all(&root)?;
    let target_path = root.join("hyprland.conf");
    let baseline = "cursor:default_monitor = eDP-1\n";
    fs::write(&target_path, baseline)?;
    let current_config = snapshot_for(&target_path, baseline);
    let backup_manager = BackupManager::new(root.join("backups"));
    let known_setting_ids: BTreeSet<String> = SAFE_WRITABLE_ROWS
        .iter()
        .map(|row| row.row_id.to_string())
        .collect();
    let proof = complete_default_monitor_proof(target_path.clone(), "DP-1", "eDP-1")?;

    let default_path = apply_setting_change_with_backup_manager(
        known_setting_ids.clone(),
        &discovery_for(target_path.clone()),
        &current_config,
        "cursor.default_monitor",
        "DP-1",
        &backup_manager,
    )
    .expect_err("missing production proof should fail");
    assert_eq!(
        default_path.failures,
        vec!["InvalidProposedValue".to_string()]
    );

    let outcome = apply_setting_change_with_backup_manager_and_production_gate(
        known_setting_ids,
        &discovery_for(target_path.clone()),
        &current_config,
        "cursor.default_monitor",
        "DP-1",
        &backup_manager,
        Some(proof),
    )
    .expect("valid oracle and high-risk gate proof should allow temp write");

    assert!(outcome.target_path.starts_with(&root));
    assert_eq!(outcome.verified_value.as_deref(), Some("DP-1"));
    assert_eq!(
        fs::read_to_string(&target_path)?,
        "cursor:default_monitor = DP-1\n"
    );
    fs::remove_dir_all(root)?;
    Ok(())
}

#[test]
fn cursor_default_monitor_research_report_records_remaining_oracle_gap() {
    let report: serde_json::Value = serde_json::from_slice(
        &fs::read(ORACLE_RESEARCH_REPORT).expect("oracle research report should exist"),
    )
    .expect("oracle research report should parse");
    assert_eq!(report["rowId"], "cursor.default_monitor");
    assert_eq!(report["fixtureOracleImplemented"], true);
    assert_eq!(
        report["runtimeOracleNeeded"],
        "runtime monitor-name allowlist/readback oracle"
    );
    assert_eq!(
        report["proofStillMissing"],
        "live/runtime monitor-name source adapter proof and stale-name refresh proof remain missing; row remains blocked"
    );
}
