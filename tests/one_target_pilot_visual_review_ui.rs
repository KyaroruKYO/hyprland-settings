use std::fs;

use hyprland_settings::one_target_pilot_live_visual_smoke::disabled_live_visual_smoke_review_ui_lines;
use hyprland_settings::write_classification::SAFE_WRITABLE_ROWS;

#[test]
fn live_visual_smoke_review_copy_is_disabled_and_future_only() {
    let lines = disabled_live_visual_smoke_review_ui_lines();
    for expected in [
        "Live visual smoke review",
        "The visual review is recorded, but this sprint does not enable writes.",
        "A separate future proposal is still required before any gate can flip.",
        "All production write gates are still disabled.",
        "Real writing is not active yet.",
        "Apply behavior has not changed.",
    ] {
        assert!(lines.iter().any(|line| line == expected));
    }

    let window_source = fs::read_to_string("src/ui/window.rs").expect("window source should read");
    assert!(window_source.contains("disabled_live_visual_smoke_review_ui_lines"));
    assert!(window_source.contains("set_sensitive(false)"));
    assert!(!window_source.contains("live_visual_smoke_review_approved"));
    assert!(!window_source.contains("gate_flip_button"));
    assert_eq!(SAFE_WRITABLE_ROWS.len(), 341);
}
