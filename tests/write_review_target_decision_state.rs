use hyprland_settings::write_classification::SAFE_WRITABLE_ROWS;
use hyprland_settings::write_review_walkthrough::{
    WriteReviewTargetDecision, WriteReviewTargetDecisionState,
};

#[test]
fn disabled_target_decision_state_represents_future_decisions_without_selection() {
    for decision in [
        WriteReviewTargetDecision::RecommendedTargetAccepted,
        WriteReviewTargetDecision::AlternateTargetRequested,
        WriteReviewTargetDecision::BlockedTargetRequested,
        WriteReviewTargetDecision::AdvancedConfirmationNeeded,
        WriteReviewTargetDecision::DecisionNotActive,
    ] {
        let state = WriteReviewTargetDecisionState::disabled(decision);
        assert_eq!(state.decision, decision);
        assert!(!state.decision_enabled);
        assert!(state.production_disabled);
        assert!(state.selected_target.is_none());
    }
    assert_eq!(SAFE_WRITABLE_ROWS.len(), 341);
}
