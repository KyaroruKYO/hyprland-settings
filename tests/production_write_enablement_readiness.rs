use hyprland_settings::write_classification::SAFE_WRITABLE_ROWS;
use hyprland_settings::write_enablement_readiness::{
    current_production_write_enablement_readiness, ProductionWriteEnablementStatus,
    PRODUCTION_WRITE_TARGET_SELECTION_READY,
};

#[test]
fn readiness_model_represents_safe_batch_write_enablement() {
    let readiness = current_production_write_enablement_readiness();

    assert_eq!(readiness.status, ProductionWriteEnablementStatus::Ready);
    assert!(readiness.is_ready());
    assert!(readiness.production_apply_integration_allowed);
    assert!(readiness.real_write_target_selection_active);
    assert!(!readiness.real_layered_writes_active);
    assert!(PRODUCTION_WRITE_TARGET_SELECTION_READY);

    let gate_ids = readiness
        .gates
        .iter()
        .map(|gate| gate.id)
        .collect::<Vec<_>>();
    for expected in [
        "production_write_review_gate",
        "target_selection_ui",
        "exact_backup",
        "backup_path_policy",
        "generated_script_confirmation",
        "symlink_target_policy",
        "reread_verification",
        "rollback_recovery",
        "high_risk_policy",
        "fixture_proof",
        "manual_smoke_review",
        "production_apply_integration",
    ] {
        assert!(
            gate_ids.contains(&expected),
            "missing readiness gate: {expected}"
        );
    }
    assert!(readiness.gates.iter().all(|gate| gate.satisfied));
    assert!(readiness.gates.iter().all(|gate| gate.production_enabling));
    assert_eq!(SAFE_WRITABLE_ROWS.len(), 341);
}

#[test]
fn readiness_copy_explains_safe_batch_status_and_exclusions() {
    let copy = current_production_write_enablement_readiness()
        .user_facing_lines()
        .join("\n");

    for expected in [
        "Status: Ready for safe batch writes",
        "Safe batch write is available for normal settings.",
        "Some settings are blocked because they need extra safety review.",
        "The app will back up files before writing.",
        "The app will check the result after writing.",
        "If something fails, the app will restore the backup.",
        "Required before enabling: Exact backup implementation",
        "Required before enabling: Generated/script-managed confirmation",
        "Required before enabling: Reread verification",
        "Required before enabling: Rollback/recovery plan",
        "High-risk settings remain blocked.",
        "Generated, script-managed, symlink-managed, duplicate, missing-line, and structured settings remain blocked.",
    ] {
        assert!(copy.contains(expected), "missing readiness copy: {expected}");
    }
}
