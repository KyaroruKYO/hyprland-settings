use std::fs;

use hyprland_settings::write_classification::SAFE_WRITABLE_ROWS;

#[test]
fn disabled_production_backup_and_verification_copy_exists_without_handlers() {
    let source = fs::read_to_string("src/ui/window.rs").expect("window source should read");
    let model = fs::read_to_string("src/one_target_pilot_readiness.rs")
        .expect("readiness model source should read");
    let section = source
        .split("Production backup and verification")
        .nth(1)
        .and_then(|section| section.split("Review save location").next())
        .expect("production backup and verification UI copy should exist");

    for copy in [
        "The app will back up this exact file before saving changes.",
        "The backup must match the original file before any write can continue.",
        "The app will reread the file to confirm the value.",
        "If verification fails, the app must not report the change as complete.",
        "Rollback/recovery must be implemented before real writes.",
        "Backup contract approval is staged; backup creation is still blocked until write execution gates are approved.",
        "Production verification is not active yet.",
        "Real writing is not active yet.",
        "Apply behavior has not changed.",
    ] {
        assert!(
            model.contains(copy),
            "expected disabled UI copy missing: {copy}"
        );
    }

    assert!(source.contains("current_one_target_pilot_readiness_mapping"));
    assert!(!section.contains("connect_clicked"));
    assert!(!section.contains("apply_setting_change("));
    assert_eq!(SAFE_WRITABLE_ROWS.len(), 341);
}
