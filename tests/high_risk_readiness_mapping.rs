use hyprland_settings::one_target_pilot_readiness::current_one_target_pilot_readiness_mapping;
use hyprland_settings::production_high_risk_approval::{
    current_high_risk_readiness_mapping, PRODUCTION_HIGH_RISK_APPROVAL_ENABLED,
};
use hyprland_settings::write_classification::SAFE_WRITABLE_ROWS;

#[test]
fn readiness_mapping_tracks_high_risk_prerequisites_as_incomplete() {
    let high_risk = current_high_risk_readiness_mapping();
    assert!(!high_risk.approval_boundary_complete);
    assert!(!high_risk.classification_integration_complete);
    assert!(!high_risk.approval_state_model_complete);
    assert!(!high_risk.warning_copy_complete);
    assert!(!high_risk.first_pilot_exclusion_proof_complete);
    assert!(!high_risk.production_high_risk_approval_enabled);
    assert!(!high_risk.ready_for_production());

    let pilot = current_one_target_pilot_readiness_mapping();
    assert!(!pilot.high_risk_approval_boundary_complete);
    assert!(!pilot.high_risk_classification_integration_complete);
    assert!(!pilot.high_risk_approval_state_model_complete);
    assert!(!pilot.high_risk_warning_copy_complete);
    assert!(!pilot.high_risk_first_pilot_exclusion_proof_complete);
    assert!(!pilot.production_high_risk_approval_enabled);
    assert!(!pilot.is_ready_for_production());
    assert!(!PRODUCTION_HIGH_RISK_APPROVAL_ENABLED);
    assert_eq!(SAFE_WRITABLE_ROWS.len(), 341);
}
