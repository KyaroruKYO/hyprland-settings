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
fn walkthrough_discoverability_copy_is_scoped_to_layered_settings() {
    let source = fs::read_to_string("src/ui/window.rs").expect("window source should read");
    let section = source_slice(
        &source,
        "fn append_pre_apply_review_scaffold",
        "fn append_user_facing_write_reason",
    );

    assert!(section.contains("Write review walkthrough"));
    assert!(section.contains("Shown when a setting is controlled in more than one place."));
    assert!(section.contains("This walkthrough shows what the app would check before writing."));
    assert!(section.contains("if !layered.controlled_in_more_than_one_place"));
    assert!(section.contains("return;"));
    assert!(!section.contains("apply_setting_change("));
    assert_eq!(SAFE_WRITABLE_ROWS.len(), 341);
}
