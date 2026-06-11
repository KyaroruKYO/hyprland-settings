use hyprland_settings::one_target_pilot_focused_visual_smoke::focused_visual_smoke_ui_decision;
use hyprland_settings::write_classification::SAFE_WRITABLE_ROWS;

#[test]
fn focused_visual_smoke_ui_is_deferred_to_avoid_denser_detail_pane() {
    let decision = focused_visual_smoke_ui_decision();

    assert!(!decision.ui_added);
    assert!(decision.model_report_only);
    assert!(!decision.controls_added);
    assert!(!decision.handlers_added);
    assert!(decision.reason.contains("model, report, and review log"));
    assert_eq!(SAFE_WRITABLE_ROWS.len(), 341);
}
