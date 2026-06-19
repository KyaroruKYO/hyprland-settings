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
fn production_enablement_ui_scaffold_has_safe_batch_copy() {
    let source = fs::read_to_string("src/ui/window.rs").expect("window source should read");
    let section = source_slice(
        &source,
        "fn append_pre_apply_review_scaffold",
        "fn append_user_facing_write_reason",
    );
    let readiness_source = fs::read_to_string("src/write_enablement_readiness.rs")
        .expect("readiness source should read");
    let safe_batch_source =
        fs::read_to_string("src/safe_batch_write.rs").expect("safe batch source should read");

    for expected in [
        "Production write enablement",
        "Status: Ready for safe batch writes",
        "Safe batch write",
        "Safe batch write is available for normal settings.",
        "Some settings are blocked because they need extra safety review.",
        "The app will back up files before writing.",
        "The app will check the result after writing.",
        "If something fails, the app will restore the backup.",
        "Required before enabling",
        "High-risk settings remain blocked.",
        "Generated, script-managed, symlink-managed, duplicate, missing-line, and structured settings remain blocked.",
    ] {
        assert!(
            section.contains(expected)
                || readiness_source.contains(expected)
                || safe_batch_source.contains(expected),
            "missing production enablement UI copy: {expected}"
        );
    }

    assert!(section.contains("enablement_button.set_sensitive(false)"));
    assert!(!section.contains("connect_toggled"));
    assert!(!section.contains("connect_clicked(move"));
    assert_eq!(SAFE_WRITABLE_ROWS.len(), 341);
}
