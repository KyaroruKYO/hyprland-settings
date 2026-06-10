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
fn disabled_production_enablement_ui_scaffold_has_not_ready_copy() {
    let source = fs::read_to_string("src/ui/window.rs").expect("window source should read");
    let section = source_slice(
        &source,
        "fn append_pre_apply_review_scaffold",
        "fn append_user_facing_write_reason",
    );
    let readiness_source = fs::read_to_string("src/write_enablement_readiness.rs")
        .expect("readiness source should read");

    for expected in [
        "Production write enablement",
        "Status: Not ready",
        "Production write-target selection is not ready yet.",
        "The app can preview the review flow, but cannot write through it.",
        "Before enabling writes, exact backup, reread verification, recovery, and advanced confirmation must be complete.",
        "Required before enabling",
        "Real write-target selection is not active yet.",
        "Production enablement is disabled",
    ] {
        assert!(
            section.contains(expected) || readiness_source.contains(expected),
            "missing production enablement UI copy: {expected}"
        );
    }

    assert!(section.contains("enablement_button.set_sensitive(false)"));
    assert!(!section.contains("connect_toggled"));
    assert!(!section.contains("connect_clicked(move"));
    assert_eq!(SAFE_WRITABLE_ROWS.len(), 341);
}
