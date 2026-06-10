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
fn disabled_walkthrough_ui_has_screenshot_friendly_copy_and_no_handlers() {
    let source = fs::read_to_string("src/ui/window.rs").expect("window source should read");
    let section = source_slice(
        &source,
        "fn append_pre_apply_review_scaffold",
        "fn append_user_facing_write_reason",
    );
    let walkthrough_source = fs::read_to_string("src/write_review_walkthrough.rs")
        .expect("walkthrough source should read");
    let recommendation_source = fs::read_to_string("src/write_target_recommendation.rs")
        .expect("recommendation source should read");
    let guarded_source =
        fs::read_to_string("src/guarded_write_review.rs").expect("guarded source should read");
    let session_source = fs::read_to_string("src/session_value_projection.rs")
        .expect("session projection source should read");

    for copy in [
        "Write review walkthrough",
        "This walkthrough shows what the app would check before writing.",
        "Active config value",
        "Session preview value",
        "Recommended save location",
        "Other possible locations",
        "Blocked locations",
        "Backup planned",
        "Verification planned",
        "Target decisions are preview-only right now.",
        "Real save-location selection is not active yet.",
        "Real writing is not active yet.",
        "Apply behavior has not changed.",
    ] {
        assert!(
            section.contains(copy)
                || walkthrough_source.contains(copy)
                || recommendation_source.contains(copy)
                || guarded_source.contains(copy)
                || session_source.contains(copy),
            "missing walkthrough copy: {copy}"
        );
    }

    assert!(section.contains("decision_button.set_sensitive(false)"));
    assert!(section.contains("review_button.set_sensitive(false)"));
    assert!(!section.contains("connect_toggled"));
    assert!(!section.contains("connect_clicked(move"));
    assert_eq!(SAFE_WRITABLE_ROWS.len(), 341);
}
