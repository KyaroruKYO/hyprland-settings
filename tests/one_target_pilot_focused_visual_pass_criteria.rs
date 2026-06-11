use hyprland_settings::one_target_pilot_focused_visual_smoke::{
    one_target_pilot_focused_visual_pass_criteria, one_target_pilot_focused_visual_smoke_result,
};
use hyprland_settings::one_target_pilot_live_visual_smoke::VisualSmokeItemResult;
use hyprland_settings::one_target_pilot_manual_review::all_production_gates_remain_false;
use hyprland_settings::write_classification::SAFE_WRITABLE_ROWS;

#[test]
fn focused_visual_review_passes_only_when_all_required_criteria_are_confirmed() {
    let result = one_target_pilot_focused_visual_smoke_result();
    let criteria = one_target_pilot_focused_visual_pass_criteria(&result);

    assert!(criteria.dashboard_confirmed);
    assert!(criteria.config_page_confirmed);
    assert!(criteria.connected_files_confirmed);
    assert!(criteria.normal_category_confirmed);
    assert!(criteria.detail_pane_confirmed);
    assert!(criteria.production_review_copy_confirmed);
    assert!(criteria.disabled_production_controls_confirmed);
    assert!(criteria.unsafe_actions_avoided);
    assert!(criteria.all_gates_remain_false);
    assert!(criteria.passes());

    for expected in [
        "Dashboard",
        "Config page",
        "Connected files section",
        "normal settings category",
        "setting detail pane",
        "production review section",
    ] {
        assert!(result.screens_inspected.iter().any(|item| {
            item.item_label == expected && item.result == VisualSmokeItemResult::Passed
        }));
    }

    assert!(all_production_gates_remain_false());
    assert_eq!(SAFE_WRITABLE_ROWS.len(), 341);
}
