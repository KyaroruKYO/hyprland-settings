use hyprland_settings::one_target_pilot_readiness::current_one_target_pilot_readiness_mapping;
use hyprland_settings::production_advanced_confirmation::{
    current_advanced_confirmation_readiness_mapping, PRODUCTION_ADVANCED_CONFIRMATION_ENABLED,
};
use hyprland_settings::write_classification::SAFE_WRITABLE_ROWS;

#[test]
fn readiness_mapping_tracks_advanced_confirmation_prerequisites_as_incomplete() {
    let advanced = current_advanced_confirmation_readiness_mapping();
    assert!(!advanced.risk_policy_complete);
    assert!(!advanced.hard_block_policy_complete);
    assert!(!advanced.first_pilot_exclusion_policy_complete);
    assert!(!advanced.risky_target_ui_copy_complete);
    assert!(!advanced.fixture_risk_policy_proof_passed);
    assert!(!advanced.production_advanced_confirmation_enabled);
    assert!(!advanced.ready_for_production());

    let pilot = current_one_target_pilot_readiness_mapping();
    assert!(!pilot.advanced_confirmation_risk_policy_complete);
    assert!(!pilot.hard_block_policy_complete);
    assert!(!pilot.first_pilot_exclusion_policy_complete);
    assert!(!pilot.risky_target_ui_copy_complete);
    assert!(!pilot.fixture_risk_policy_proof_passed);
    assert!(!pilot.production_advanced_confirmation_enabled);
    assert!(!pilot.is_ready_for_production());
    assert!(!PRODUCTION_ADVANCED_CONFIRMATION_ENABLED);
    assert_eq!(SAFE_WRITABLE_ROWS.len(), 341);
}
