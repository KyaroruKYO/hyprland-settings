use hyprland_settings::guarded_write_review::PRODUCTION_WRITE_TARGET_REVIEW_ENABLED;
use hyprland_settings::one_target_pilot_pre_enable_audit::{
    one_target_pilot_gate_inventory_snapshot, PRODUCTION_ONE_TARGET_PRE_ENABLE_AUDIT_PASSED,
};
use hyprland_settings::one_target_write_pilot::PRODUCTION_ONE_TARGET_WRITE_PILOT_ENABLED;
use hyprland_settings::production_advanced_confirmation::PRODUCTION_ADVANCED_CONFIRMATION_ENABLED;
use hyprland_settings::production_backup_contract::PRODUCTION_BACKUP_CONTRACT_ENABLED;
use hyprland_settings::production_high_risk_approval::PRODUCTION_HIGH_RISK_APPROVAL_ENABLED;
use hyprland_settings::production_recovery_contract::PRODUCTION_RECOVERY_CONTRACT_ENABLED;
use hyprland_settings::production_verification_contract::PRODUCTION_VERIFICATION_CONTRACT_ENABLED;
use hyprland_settings::write_classification::SAFE_WRITABLE_ROWS;
use hyprland_settings::write_enablement_readiness::PRODUCTION_WRITE_TARGET_SELECTION_READY;
use hyprland_settings::write_review_walkthrough::PRODUCTION_WRITE_REVIEW_WALKTHROUGH_CAN_WRITE;

#[test]
fn gate_inventory_snapshot_lists_all_production_gates_as_false() {
    assert!(!PRODUCTION_ONE_TARGET_PRE_ENABLE_AUDIT_PASSED);
    assert!(!PRODUCTION_HIGH_RISK_APPROVAL_ENABLED);
    assert!(!PRODUCTION_ADVANCED_CONFIRMATION_ENABLED);
    assert!(!PRODUCTION_RECOVERY_CONTRACT_ENABLED);
    assert!(!PRODUCTION_BACKUP_CONTRACT_ENABLED);
    assert!(!PRODUCTION_VERIFICATION_CONTRACT_ENABLED);
    assert!(!PRODUCTION_ONE_TARGET_WRITE_PILOT_ENABLED);
    assert!(!PRODUCTION_WRITE_TARGET_SELECTION_READY);
    assert!(!PRODUCTION_WRITE_TARGET_REVIEW_ENABLED);
    assert!(!PRODUCTION_WRITE_REVIEW_WALKTHROUGH_CAN_WRITE);

    let snapshot = one_target_pilot_gate_inventory_snapshot();
    let names = snapshot
        .iter()
        .map(|gate| gate.gate_name)
        .collect::<Vec<_>>();
    for expected in [
        "PRODUCTION_ONE_TARGET_PRE_ENABLE_AUDIT_PASSED",
        "PRODUCTION_ONE_TARGET_WRITE_PILOT_ENABLED",
        "PRODUCTION_WRITE_TARGET_SELECTION_READY",
        "PRODUCTION_WRITE_TARGET_REVIEW_ENABLED",
        "PRODUCTION_WRITE_REVIEW_WALKTHROUGH_CAN_WRITE",
        "PRODUCTION_BACKUP_CONTRACT_ENABLED",
        "PRODUCTION_VERIFICATION_CONTRACT_ENABLED",
        "PRODUCTION_RECOVERY_CONTRACT_ENABLED",
        "PRODUCTION_ADVANCED_CONFIRMATION_ENABLED",
        "PRODUCTION_HIGH_RISK_APPROVAL_ENABLED",
    ] {
        assert!(
            names.contains(&expected),
            "missing gate snapshot: {expected}"
        );
    }

    assert!(snapshot.iter().all(|gate| !gate.current_value));
    assert!(snapshot.iter().all(|gate| !gate.would_allow.is_empty()
        && !gate.required_proof_before_flip.is_empty()
        && !gate.current_blocking_reason.is_empty()));
    assert_eq!(SAFE_WRITABLE_ROWS.len(), 341);
}
