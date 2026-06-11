use hyprland_settings::one_target_pilot_live_visual_smoke::{
    one_target_pilot_live_visual_smoke_review_result, LiveVisualSmokeReviewStatus,
    VisualSmokeItemResult,
};
use hyprland_settings::write_classification::SAFE_WRITABLE_ROWS;

#[test]
fn live_visual_smoke_review_result_represents_current_inconclusive_review() {
    let result = one_target_pilot_live_visual_smoke_review_result();

    assert!(result.visual_review_performed);
    assert!(result.app_launched);
    assert_eq!(result.status, LiveVisualSmokeReviewStatus::Inconclusive);
    assert!(!result.visual_review_passed);
    assert!(!result.visual_review_failed);
    assert!(result.visual_review_inconclusive);
    assert!(!result.screenshot_retained);
    assert!(
        result
            .screens_inspected
            .iter()
            .any(|item| item.item_label == "Dashboard"
                && item.result == VisualSmokeItemResult::Passed)
    );
    assert!(result
        .screens_inspected
        .iter()
        .any(|item| item.result == VisualSmokeItemResult::NotSeen));
    assert!(result
        .expected_copy_results
        .iter()
        .any(|item| item.item_label == "Manual smoke review"
            && item.result == VisualSmokeItemResult::NotSeen));
    assert!(result
        .disabled_control_results
        .iter()
        .any(|item| item.item_label == "Review save location"
            && item.result == VisualSmokeItemResult::NotSeen));
    assert!(result
        .unsafe_actions_avoided
        .iter()
        .any(|item| item.item_label == "Apply avoided"
            && item.result == VisualSmokeItemResult::Passed));
    assert!(!result.warnings_observed.is_empty());
    assert_eq!(SAFE_WRITABLE_ROWS.len(), 341);
}
