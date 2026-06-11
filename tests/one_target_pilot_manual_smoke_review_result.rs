use hyprland_settings::one_target_pilot_manual_review::{
    all_manual_smoke_review_item_results, one_target_pilot_manual_smoke_review_result,
    ManualSmokeReviewItemResult,
};
use hyprland_settings::write_classification::SAFE_WRITABLE_ROWS;

#[test]
fn manual_smoke_review_result_represents_all_outcomes_and_blockers() {
    let all_results = all_manual_smoke_review_item_results();
    for expected in [
        ManualSmokeReviewItemResult::Passed,
        ManualSmokeReviewItemResult::Failed,
        ManualSmokeReviewItemResult::NotReviewed,
        ManualSmokeReviewItemResult::NotApplicable,
        ManualSmokeReviewItemResult::SourceProven,
        ManualSmokeReviewItemResult::FixtureProven,
        ManualSmokeReviewItemResult::ManualOnly,
    ] {
        assert!(all_results.contains(&expected));
        assert!(!expected.label().is_empty());
    }

    let review = one_target_pilot_manual_smoke_review_result();
    assert!(!review.live_gtk_visual_smoke_performed);
    assert!(!review.gate_flip_proposal_ready);
    assert!(!review.production_activation_ready);
    assert!(review
        .items
        .iter()
        .any(|item| item.result == ManualSmokeReviewItemResult::SourceProven));
    assert!(review
        .items
        .iter()
        .any(|item| item.result == ManualSmokeReviewItemResult::FixtureProven));
    assert!(review
        .items
        .iter()
        .any(|item| item.result == ManualSmokeReviewItemResult::ManualOnly));
    assert!(review
        .items
        .iter()
        .any(|item| item.result == ManualSmokeReviewItemResult::NotReviewed));
    assert!(review
        .items
        .iter()
        .any(|item| item.blocking_reason.is_some() && item.blocks_gate_flip_proposal_readiness));
    assert_eq!(SAFE_WRITABLE_ROWS.len(), 341);
}
