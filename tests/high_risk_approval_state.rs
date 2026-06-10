use hyprland_settings::production_high_risk_approval::{
    all_high_risk_approval_states, high_risk_approval_state_model, HighRiskApprovalState,
    PRODUCTION_HIGH_RISK_APPROVAL_ENABLED,
};
use hyprland_settings::write_classification::SAFE_WRITABLE_ROWS;

#[test]
fn high_risk_approval_states_are_non_persistent_and_non_applying() {
    for state in [
        HighRiskApprovalState::NotRequired,
        HighRiskApprovalState::RequiredButUnavailable,
        HighRiskApprovalState::RequestedButDisabled,
        HighRiskApprovalState::ApprovedInFixtureOnly,
        HighRiskApprovalState::Rejected,
        HighRiskApprovalState::Expired,
        HighRiskApprovalState::ProductionDisabled,
    ] {
        assert!(all_high_risk_approval_states().contains(&state));
        let model = high_risk_approval_state_model(state);
        assert!(!model.can_persist);
        assert!(!model.affects_apply);
        assert!(!model.can_make_first_pilot_eligible);
        assert!(!model.production_enabled);
    }

    assert!(!PRODUCTION_HIGH_RISK_APPROVAL_ENABLED);
    assert_eq!(SAFE_WRITABLE_ROWS.len(), 341);
}
