use hyprland_settings::one_target_pilot_manual_review::{
    one_target_pilot_gate_flip_proposal_readiness, GateFlipProposalDecision, ManualReviewCompletion,
};
use hyprland_settings::one_target_pilot_pre_enable_audit::PRODUCTION_ONE_TARGET_PRE_ENABLE_AUDIT_PASSED;
use hyprland_settings::write_classification::SAFE_WRITABLE_ROWS;

#[test]
fn gate_flip_proposal_readiness_is_a_no_go_without_flipping_gates() {
    let readiness = one_target_pilot_gate_flip_proposal_readiness();

    assert_eq!(
        readiness.decision,
        GateFlipProposalDecision::NotReadyForGateFlipProposal
    );
    assert_eq!(
        readiness.manual_review_completion,
        ManualReviewCompletion::Partial
    );
    assert!(!readiness.ready_for_gate_flip_proposal);
    assert!(!readiness.ready_for_production_implementation_sprint);
    assert!(!readiness.production_activation_ready);
    assert!(PRODUCTION_ONE_TARGET_PRE_ENABLE_AUDIT_PASSED);
    assert!(!readiness.reasons.is_empty());
    assert!(!readiness.remaining_blockers.is_empty());
    assert!(readiness
        .recommended_next_sprint
        .contains("controlled live read-only visual smoke review"));
    assert_eq!(SAFE_WRITABLE_ROWS.len(), 341);
}
