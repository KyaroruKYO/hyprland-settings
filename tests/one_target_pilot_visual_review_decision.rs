use hyprland_settings::one_target_pilot_live_visual_smoke::{
    one_target_pilot_live_visual_smoke_review_result,
    one_target_pilot_visual_gate_flip_proposal_readiness, LiveVisualSmokeReviewStatus,
    VisualGateFlipProposalDecision,
};
use hyprland_settings::one_target_pilot_pre_enable_audit::PRODUCTION_ONE_TARGET_PRE_ENABLE_AUDIT_PASSED;
use hyprland_settings::write_classification::SAFE_WRITABLE_ROWS;

#[test]
fn visual_review_decision_blocks_proposal_when_review_is_inconclusive() {
    let result = one_target_pilot_live_visual_smoke_review_result();
    let readiness = one_target_pilot_visual_gate_flip_proposal_readiness(&result);

    assert_eq!(result.status, LiveVisualSmokeReviewStatus::Inconclusive);
    assert_eq!(
        readiness.decision,
        VisualGateFlipProposalDecision::NotReadyForSeparateGateFlipProposal
    );
    assert!(!readiness.ready_for_separate_gate_flip_proposal);
    assert!(!readiness.production_activation_ready);
    assert!(readiness
        .reasons
        .iter()
        .any(|reason| reason.contains("inconclusive")));
    assert!(PRODUCTION_ONE_TARGET_PRE_ENABLE_AUDIT_PASSED);
    assert_eq!(SAFE_WRITABLE_ROWS.len(), 341);
}
