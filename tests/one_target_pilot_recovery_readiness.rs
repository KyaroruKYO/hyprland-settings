use hyprland_settings::one_target_pilot_readiness::current_one_target_pilot_readiness_mapping;
use hyprland_settings::production_recovery_contract::PRODUCTION_RECOVERY_CONTRACT_ENABLED;
use hyprland_settings::write_classification::SAFE_WRITABLE_ROWS;

#[test]
fn one_target_pilot_readiness_tracks_specific_recovery_prerequisites() {
    let readiness = current_one_target_pilot_readiness_mapping();

    assert!(!readiness.recovery_trigger_model_complete);
    assert!(!readiness.restore_operation_contract_complete);
    assert!(!readiness.restore_verification_contract_complete);
    assert!(!readiness.recovery_result_reporting_complete);
    assert!(!readiness.fixture_recovery_proof_passed);
    assert!(!readiness.production_recovery_enabled);
    assert!(!readiness.pilot_gate_enabled);
    assert!(!readiness.is_ready_for_production());
    assert!(!PRODUCTION_RECOVERY_CONTRACT_ENABLED);
    assert_eq!(SAFE_WRITABLE_ROWS.len(), 341);
}
