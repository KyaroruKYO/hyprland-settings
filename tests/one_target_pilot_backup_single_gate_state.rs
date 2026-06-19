use hyprland_settings::guarded_write_review::PRODUCTION_WRITE_TARGET_REVIEW_ENABLED;
use hyprland_settings::one_target_pilot_backup_gate_approval::{
    one_target_pilot_backup_gate_inventory_after,
    one_target_pilot_backup_gate_single_gate_state_is_preserved,
};
use hyprland_settings::one_target_pilot_pre_enable_audit::PRODUCTION_ONE_TARGET_PRE_ENABLE_AUDIT_PASSED;
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
fn pre_enable_backup_and_verification_gates_are_true_after_verification_approval() {
    let gates = one_target_pilot_backup_gate_inventory_after();

    assert!(PRODUCTION_ONE_TARGET_PRE_ENABLE_AUDIT_PASSED);
    assert!(PRODUCTION_BACKUP_CONTRACT_ENABLED);
    assert!(!PRODUCTION_ONE_TARGET_WRITE_PILOT_ENABLED);
    assert!(PRODUCTION_WRITE_TARGET_SELECTION_READY);
    assert!(PRODUCTION_WRITE_TARGET_REVIEW_ENABLED);
    assert!(PRODUCTION_WRITE_REVIEW_WALKTHROUGH_CAN_WRITE);
    assert!(PRODUCTION_VERIFICATION_CONTRACT_ENABLED);
    assert!(PRODUCTION_RECOVERY_CONTRACT_ENABLED);
    assert!(!PRODUCTION_ADVANCED_CONFIRMATION_ENABLED);
    assert!(!PRODUCTION_HIGH_RISK_APPROVAL_ENABLED);
    assert!(one_target_pilot_backup_gate_single_gate_state_is_preserved());
    assert_eq!(
        gates.iter().filter(|gate| gate.current_value).count(),
        7,
        "approved prerequisite gates plus the guarded safe-batch execution gate may be true"
    );
    assert!(gates.iter().any(|gate| gate.gate_name
        == "PRODUCTION_ONE_TARGET_PRE_ENABLE_AUDIT_PASSED"
        && gate.current_value));
    assert!(gates
        .iter()
        .any(|gate| gate.gate_name == "PRODUCTION_BACKUP_CONTRACT_ENABLED" && gate.current_value));
    assert!(gates.iter().any(
        |gate| gate.gate_name == "PRODUCTION_VERIFICATION_CONTRACT_ENABLED" && gate.current_value
    ));
    assert!(gates
        .iter()
        .filter(|gate| !matches!(
            gate.gate_name,
            "PRODUCTION_ONE_TARGET_PRE_ENABLE_AUDIT_PASSED"
                | "PRODUCTION_BACKUP_CONTRACT_ENABLED"
                | "PRODUCTION_VERIFICATION_CONTRACT_ENABLED"
                | "PRODUCTION_RECOVERY_CONTRACT_ENABLED"
                | "PRODUCTION_WRITE_TARGET_REVIEW_ENABLED"
                | "PRODUCTION_WRITE_TARGET_SELECTION_READY"
                | "PRODUCTION_WRITE_REVIEW_WALKTHROUGH_CAN_WRITE"
        ))
        .all(|gate| !gate.current_value));
    assert_eq!(SAFE_WRITABLE_ROWS.len(), 341);
}
