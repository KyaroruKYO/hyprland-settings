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
fn disabled_production_write_review_ui_has_required_copy_and_disabled_controls() {
    let source = fs::read_to_string("src/ui/window.rs").expect("window source should read");
    let section = source_slice(
        &source,
        "fn append_pre_apply_review_scaffold",
        "fn append_user_facing_write_reason",
    );

    for copy in [
        "Write review",
        "Recommended save location",
        "Backup",
        "The app will back up this exact file before saving changes.",
        "Verification",
        "The app will reread the file to confirm the value.",
        "Safety",
        "Advanced confirmation would be required before writing here.",
        "Safe batch writing is guarded by backup, verification, and recovery checks.",
        "Apply writes only when every selected setting has a safe target.",
        "Safe batch write is available for normal settings.",
        "Review save location",
    ] {
        assert!(
            section.contains(copy)
                || fs::read_to_string("src/guarded_write_review.rs")
                    .expect("review source should read")
                    .contains(copy)
                || fs::read_to_string("src/write_backup_plan.rs")
                    .expect("backup source should read")
                    .contains(copy)
                || fs::read_to_string("src/write_verification_plan.rs")
                    .expect("verification source should read")
                    .contains(copy)
                || fs::read_to_string("src/write_advanced_confirmation.rs")
                    .expect("advanced source should read")
                    .contains(copy),
            "missing disabled review copy: {copy}"
        );
    }

    assert!(section.contains("review_button.set_sensitive(false)"));
    assert!(section.contains("button.set_sensitive(false)"));
    assert!(!section.contains("connect_toggled"));
    assert!(!section.contains("connect_clicked(move"));
    assert_eq!(SAFE_WRITABLE_ROWS.len(), 341);
}
