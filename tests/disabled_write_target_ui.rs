use std::fs;

use hyprland_settings::write_classification::SAFE_WRITABLE_ROWS;

fn source_slice<'a>(source: &'a str, start: &str, end: &str) -> &'a str {
    let start = source
        .find(start)
        .expect("source start marker should exist");
    let end = source[start..]
        .find(end)
        .map(|offset| start + offset)
        .expect("source end marker should exist");
    &source[start..end]
}

#[test]
fn save_location_ui_shape_is_disabled_and_future_only() {
    let source = fs::read_to_string("src/ui/window.rs").expect("window source should read");
    let section = source_slice(
        &source,
        "fn append_pre_apply_review_scaffold",
        "fn append_user_facing_write_reason",
    );
    let recommendation_source = fs::read_to_string("src/write_target_recommendation.rs")
        .expect("recommendation source should read");

    assert!(section.contains("gtk::CheckButton::with_label"));
    assert!(section.contains("button.set_sensitive(false)"));
    assert!(recommendation_source.contains("Recommended save location"));
    assert!(recommendation_source.contains("Other possible locations"));
    assert!(recommendation_source.contains("Blocked locations"));
    assert!(recommendation_source.contains("production_disabled: true"));
    assert!(!section.contains("connect_toggled"));
    assert!(!section.contains("write_target_path"));
    assert_eq!(SAFE_WRITABLE_ROWS.len(), 341);
}
