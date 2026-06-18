use hyprland_settings::one_target_pilot_readiness::current_one_target_pilot_readiness_mapping;
use hyprland_settings::one_target_write_pilot::PRODUCTION_ONE_TARGET_WRITE_PILOT_ENABLED;
use hyprland_settings::production_backup_contract::PRODUCTION_BACKUP_CONTRACT_ENABLED;
use hyprland_settings::production_recovery_contract::PRODUCTION_RECOVERY_CONTRACT_ENABLED;
use hyprland_settings::production_verification_contract::PRODUCTION_VERIFICATION_CONTRACT_ENABLED;
use hyprland_settings::write_classification::SAFE_WRITABLE_ROWS;

#[test]
fn one_target_pilot_readiness_maps_verification_approved_and_keeps_write_execution_blocked() {
    let readiness = current_one_target_pilot_readiness_mapping();

    assert!(!readiness.backup_contract_complete);
    assert!(!readiness.backup_collision_policy_complete);
    assert!(!readiness.backup_integrity_check_complete);
    assert!(!readiness.reread_verification_complete);
    assert!(!readiness.verification_failure_behavior_complete);
    assert!(!readiness.recovery_contract_complete);
    assert!(!readiness.advanced_confirmation_policy_complete);
    assert!(!readiness.manual_smoke_review_complete);
    assert!(!readiness.apply_integration_boundary_approved);
    assert!(readiness.production_pre_enable_audit_passed);
    assert!(!readiness.is_ready_for_production());
    assert!(readiness.production_backup_enabled);
    assert!(readiness.production_verification_enabled);
    assert!(readiness.production_recovery_enabled);
    assert!(!readiness.pilot_gate_enabled);
    assert!(PRODUCTION_BACKUP_CONTRACT_ENABLED);
    assert!(PRODUCTION_VERIFICATION_CONTRACT_ENABLED);
    assert!(PRODUCTION_RECOVERY_CONTRACT_ENABLED);
    assert!(!PRODUCTION_ONE_TARGET_WRITE_PILOT_ENABLED);
    assert!(readiness
        .user_facing_lines()
        .iter()
        .any(|line| line == "Recovery contract approval is staged; recovery execution is still blocked until the pilot is approved."));
    assert_eq!(SAFE_WRITABLE_ROWS.len(), 341);
}
