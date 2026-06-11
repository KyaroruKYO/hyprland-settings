use hyprland_settings::one_target_pilot_gate_flip_proposal_review::{
    one_target_pilot_proposal_consistency_review,
    one_target_pilot_proposal_review_references_focused_readiness,
};
use hyprland_settings::write_classification::SAFE_WRITABLE_ROWS;

#[test]
fn proposal_consistency_references_full_proof_chain_without_enabling_writes() {
    let review = one_target_pilot_proposal_consistency_review();

    assert!(review.focused_visual_smoke_proof_referenced);
    assert!(review.manual_smoke_evidence_referenced);
    assert!(review.fixture_write_proof_referenced);
    assert!(review.backup_proof_referenced);
    assert!(review.verification_proof_referenced);
    assert!(review.recovery_proof_referenced);
    assert!(review.advanced_confirmation_proof_referenced);
    assert!(review.high_risk_boundary_proof_referenced);
    assert!(review.target_management_risk_policy_referenced);
    assert!(review.apply_isolation_proof_referenced);
    assert!(review.all_gates_false_referenced);
    assert!(!review.enables_writes_in_this_sprint);
    assert!(!review.treats_draft_as_gate_flip_approval);
    assert!(!review.changes_apply_behavior);
    assert!(one_target_pilot_proposal_review_references_focused_readiness());
    assert_eq!(SAFE_WRITABLE_ROWS.len(), 341);
}
