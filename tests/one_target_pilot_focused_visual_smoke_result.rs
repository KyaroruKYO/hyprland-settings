use hyprland_settings::one_target_pilot_focused_visual_smoke::{
    one_target_pilot_focused_visual_smoke_plan, one_target_pilot_focused_visual_smoke_result,
};
use hyprland_settings::one_target_pilot_live_visual_smoke::{
    LiveVisualSmokeReviewStatus, VisualSmokeItemResult,
};
use hyprland_settings::write_classification::SAFE_WRITABLE_ROWS;

#[test]
fn focused_visual_smoke_result_records_passed_live_review_and_evidence_handling() {
    let plan = one_target_pilot_focused_visual_smoke_plan();
    let result = one_target_pilot_focused_visual_smoke_result();

    assert!(plan.safe_launch_command.contains("cargo run"));
    assert!(plan.read_only_method.contains("never click Apply"));
    assert!(plan
        .app_window_evidence_strategy
        .contains("delete temporary image"));
    assert!(plan.screens_to_inspect.contains(&"Config page"));
    assert!(plan
        .screens_to_inspect
        .contains(&"safe scalar row detail pane"));
    assert!(plan.expected_copy.contains(&"Live visual smoke review"));
    assert!(plan
        .expected_disabled_controls
        .contains(&"Production enablement is disabled"));
    assert!(plan
        .forbidden_actions
        .iter()
        .any(|action| action.contains("Do not click Apply")));

    assert!(result.review_performed);
    assert!(result.app_launched);
    assert_eq!(result.status, LiveVisualSmokeReviewStatus::Passed);
    assert!(result.review_passed);
    assert!(!result.review_failed);
    assert!(!result.review_inconclusive);
    assert!(result.proposal_allowed);
    assert!(result.app_window_only_evidence.attempted);
    assert!(result.app_window_only_evidence.captured);
    assert!(!result.app_window_only_evidence.retained);
    assert!(!result.app_window_only_evidence.safe_to_commit);
    assert!(result
        .app_window_only_evidence
        .cleanup_status
        .contains("deleted"));
    assert!(result
        .warnings_observed
        .iter()
        .any(|warning| warning.contains("Vulkan")));
    assert!(result
        .screens_inspected
        .iter()
        .all(|item| item.result == VisualSmokeItemResult::Passed));
    assert_eq!(SAFE_WRITABLE_ROWS.len(), 341);
}
