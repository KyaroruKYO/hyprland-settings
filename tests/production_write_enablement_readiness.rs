use hyprland_settings::write_classification::SAFE_WRITABLE_ROWS;
use hyprland_settings::write_enablement_readiness::{
    current_production_write_enablement_readiness, ProductionWriteEnablementStatus,
    PRODUCTION_WRITE_TARGET_SELECTION_READY,
};

#[test]
fn readiness_model_represents_all_required_gates_as_not_ready() {
    let readiness = current_production_write_enablement_readiness();

    assert_eq!(readiness.status, ProductionWriteEnablementStatus::NotReady);
    assert!(!readiness.is_ready());
    assert!(!readiness.production_apply_integration_allowed);
    assert!(!readiness.real_write_target_selection_active);
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
    assert!(readiness.gates.iter().all(|gate| !gate.satisfied));
    assert!(readiness.gates.iter().all(|gate| gate.production_enabling));
    assert_eq!(SAFE_WRITABLE_ROWS.len(), 341);
}

#[test]
fn readiness_copy_explains_preview_only_status() {
    let copy = current_production_write_enablement_readiness()
        .user_facing_lines()
        .join("\n");

    for expected in [
        "Target-selection approval is staged; real selection is still not active yet.",
        "The app can preview the review flow, but cannot write through it.",
        "Before enabling writes, exact backup, reread verification, recovery, and advanced confirmation must be complete.",
        "Required before enabling: Exact backup implementation",
        "Required before enabling: Generated/script-managed confirmation",
        "Required before enabling: Reread verification",
        "Required before enabling: Rollback/recovery plan",
        "Real write-target selection is not active yet.",
        "Apply behavior has not changed.",
    ] {
        assert!(copy.contains(expected), "missing readiness copy: {expected}");
    }
}
