use std::fs;

use hyprland_settings::one_target_pilot_gate_flip_proposal_review::{
    one_target_pilot_proposal_artifact_review, one_target_pilot_proposal_reviewed_draft_required,
};
use hyprland_settings::write_classification::SAFE_WRITABLE_ROWS;

#[test]
fn proposal_review_model_records_draft_artifact_status_and_required_clarifications() {
    let review = one_target_pilot_proposal_artifact_review();
    let draft_doc = fs::read_to_string("docs/ONE-TARGET-PILOT-GATE-FLIP-PROPOSAL-DRAFT.md")
        .expect("proposal draft markdown should exist");
    let draft_json =
        fs::read_to_string("data/reports/one-target-pilot-gate-flip-proposal-draft.v0.55.2.json")
            .expect("proposal draft json should exist");

    assert!(review.proposal_artifact_present);
    assert!(review.proposal_artifact_parsed_readable);
    assert!(review.proposal_is_draft_only);
    assert!(review.proposal_says_no_gate_flipped);
    assert!(review.proposal_requires_user_approval);
    assert!(review.proposal_requires_separate_sprint);
    assert!(review.proposal_target_class_is_narrow);
    assert!(!review.proposal_exclusions_are_complete_in_original);
    assert!(!review.proposal_stop_conditions_are_complete);
    assert!(review.proposal_rollback_conditions_are_represented);
    assert!(review.proposal_proof_references_are_present);
    assert!(review.proposal_implementation_scope_is_not_mixed_with_review);
    assert!(review
        .original_revision_reasons
        .iter()
        .any(|reason| reason.contains("unknown management state")));
    assert!(review
        .original_revision_reasons
        .iter()
        .any(|reason| reason.contains("restored byte")));
    assert!(draft_doc.contains("draft only"));
    assert!(draft_doc.contains("No gate was flipped"));
    assert!(draft_json.contains("\"draftOnly\": true"));
    assert!(one_target_pilot_proposal_reviewed_draft_required());
    assert_eq!(SAFE_WRITABLE_ROWS.len(), 341);
}
