use std::fs;

use hyprland_settings::write_classification::SAFE_WRITABLE_ROWS;

#[test]
fn disabled_recovery_copy_exists_without_handlers() {
    let window = fs::read_to_string("src/ui/window.rs").expect("window source should read");
    let readiness = fs::read_to_string("src/one_target_pilot_readiness.rs")
        .expect("readiness source should read");
    let section = window
        .split("Production backup and verification")
        .nth(1)
        .and_then(|section| section.split("Review save location").next())
        .expect("backup verification section should exist");

    for copy in [
        "Recovery",
        "If verification fails in a future version, the app will restore the backup.",
        "The app will reread the restored file before reporting recovery success.",
        "The app will not reload Hyprland automatically.",
        "If recovery fails, the app will report the failure and leave the backup available.",
        "Production recovery is not active yet.",
    ] {
        assert!(
            readiness.contains(copy),
            "expected recovery copy missing: {copy}"
        );
    }
    assert!(!section.contains("connect_clicked"));
    assert!(!section.contains("apply_setting_change("));
    assert_eq!(SAFE_WRITABLE_ROWS.len(), 341);
}
