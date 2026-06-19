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
fn pre_apply_review_scaffold_has_backup_and_caution_copy() {
    let source = fs::read_to_string("src/ui/window.rs").expect("window source should read");
    let section = source_slice(
        &source,
        "fn append_pre_apply_review_scaffold",
        "fn append_user_facing_write_reason",
    );

    for copy in [
        "Pre-apply review",
        "Before this setting can be changed, choose where the app should save it.",
        "The app will back up the exact file before saving changes.",
        "Generated or script-managed files may require advanced confirmation.",
        "Safe batch writing is guarded by backup, verification, and recovery checks.",
        "Apply writes only when every selected setting has a safe target.",
        "Save location",
    ] {
        assert!(section.contains(copy), "missing scaffold copy: {copy}");
    }

    assert!(section.contains("recommend_write_targets"));
    assert!(!section.contains("apply_setting_change("));
    assert_eq!(SAFE_WRITABLE_ROWS.len(), 341);
}
