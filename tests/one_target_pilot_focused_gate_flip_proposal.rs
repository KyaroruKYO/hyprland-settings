use hyprland_settings::one_target_pilot_focused_visual_smoke::{
    one_target_pilot_focused_gate_flip_proposal_draft,
    one_target_pilot_focused_visual_gate_flip_proposal_readiness,
    one_target_pilot_focused_visual_smoke_result,
};
use hyprland_settings::one_target_pilot_live_visual_smoke::{
    one_target_pilot_gate_flip_proposal_draft, one_target_pilot_live_visual_smoke_review_result,
    VisualGateFlipProposalDecision,
};
use hyprland_settings::one_target_write_pilot::PRODUCTION_ONE_TARGET_WRITE_PILOT_ENABLED;
use hyprland_settings::write_classification::SAFE_WRITABLE_ROWS;

#[test]
fn focused_review_pass_allows_draft_only_gate_flip_proposal_without_flipping_gate() {
    let result = one_target_pilot_focused_visual_smoke_result();
    let readiness = one_target_pilot_focused_visual_gate_flip_proposal_readiness(&result);
    let draft = one_target_pilot_focused_gate_flip_proposal_draft(&result)
        .expect("passing focused review should allow a draft proposal");

    assert_eq!(
        readiness.decision,
        VisualGateFlipProposalDecision::ReadyForSeparateGateFlipProposal
    );
    assert!(readiness.ready_for_separate_gate_flip_proposal);
    assert!(!readiness.production_activation_ready);
    assert!(readiness
        .reasons
        .iter()
        .any(|reason| reason.contains("separate future proposal")));
    assert!(draft.draft_only);
    assert!(draft.no_gate_flipped);
    assert!(draft.requires_user_approval);
    assert!(draft.requires_separate_sprint);
    assert!(draft
        .exact_gates_proposed_for_future_flip
        .contains(&"PRODUCTION_ONE_TARGET_WRITE_PILOT_ENABLED"));
    assert!(draft.target_class_allowed.contains("non-high-risk scalar"));
    assert!(draft.target_classes_excluded.contains(&"high-risk rows"));
    assert!(draft
        .stop_conditions
        .iter()
        .any(|condition| condition.contains("reread verification fails")));
    assert!(!PRODUCTION_ONE_TARGET_WRITE_PILOT_ENABLED);
    assert_eq!(SAFE_WRITABLE_ROWS.len(), 341);
}

#[test]
fn proposal_draft_is_not_created_when_review_is_inconclusive() {
    let inconclusive = one_target_pilot_live_visual_smoke_review_result();

    assert!(one_target_pilot_gate_flip_proposal_draft(&inconclusive).is_none());
    assert_eq!(SAFE_WRITABLE_ROWS.len(), 341);
}
