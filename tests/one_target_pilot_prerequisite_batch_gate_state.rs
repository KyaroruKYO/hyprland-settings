use hyprland_settings::guarded_write_review::PRODUCTION_WRITE_TARGET_REVIEW_ENABLED;
use hyprland_settings::one_target_pilot_manual_review::{
    all_write_execution_gates_remain_false, nonwriting_prerequisite_gates_are_true,
    production_write_path_remains_disabled,
};
use hyprland_settings::one_target_pilot_nonwriting_prerequisite_batch_approval::{
    one_target_pilot_nonwriting_prerequisite_batch_gate_inventory,
    one_target_pilot_nonwriting_prerequisite_batch_state_is_preserved,
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
fn prerequisite_batch_gate_state_has_six_true_prerequisites_and_four_false_execution_gates() {
    let gates = one_target_pilot_nonwriting_prerequisite_batch_gate_inventory();

    assert!(PRODUCTION_ONE_TARGET_PRE_ENABLE_AUDIT_PASSED);
    assert!(PRODUCTION_BACKUP_CONTRACT_ENABLED);
    assert!(PRODUCTION_VERIFICATION_CONTRACT_ENABLED);
    assert!(PRODUCTION_RECOVERY_CONTRACT_ENABLED);
    assert!(PRODUCTION_WRITE_TARGET_REVIEW_ENABLED);
    assert!(PRODUCTION_WRITE_TARGET_SELECTION_READY);
    assert!(!PRODUCTION_ONE_TARGET_WRITE_PILOT_ENABLED);
    assert!(!PRODUCTION_WRITE_REVIEW_WALKTHROUGH_CAN_WRITE);
    assert!(!PRODUCTION_ADVANCED_CONFIRMATION_ENABLED);
    assert!(!PRODUCTION_HIGH_RISK_APPROVAL_ENABLED);
    assert!(nonwriting_prerequisite_gates_are_true());
    assert!(all_write_execution_gates_remain_false());
    assert!(production_write_path_remains_disabled());
    assert!(one_target_pilot_nonwriting_prerequisite_batch_state_is_preserved());

    assert_eq!(
        gates.iter().filter(|gate| gate.current_value).count(),
        6,
        "only non-writing prerequisite gates may be true"
    );
    for expected_true in [
        "PRODUCTION_ONE_TARGET_PRE_ENABLE_AUDIT_PASSED",
        "PRODUCTION_BACKUP_CONTRACT_ENABLED",
        "PRODUCTION_VERIFICATION_CONTRACT_ENABLED",
        "PRODUCTION_RECOVERY_CONTRACT_ENABLED",
        "PRODUCTION_WRITE_TARGET_REVIEW_ENABLED",
        "PRODUCTION_WRITE_TARGET_SELECTION_READY",
    ] {
        assert!(gates
            .iter()
            .any(|gate| gate.gate_name == expected_true && gate.current_value));
    }
    for expected_false in [
        "PRODUCTION_ONE_TARGET_WRITE_PILOT_ENABLED",
        "PRODUCTION_WRITE_REVIEW_WALKTHROUGH_CAN_WRITE",
        "PRODUCTION_ADVANCED_CONFIRMATION_ENABLED",
        "PRODUCTION_HIGH_RISK_APPROVAL_ENABLED",
    ] {
        assert!(gates
            .iter()
            .any(|gate| gate.gate_name == expected_false && !gate.current_value));
    }
    assert_eq!(SAFE_WRITABLE_ROWS.len(), 341);
}
