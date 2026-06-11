use std::fs;

use hyprland_settings::one_target_pilot_pre_enable_audit::disabled_pre_enable_audit_ui_lines;
use hyprland_settings::write_classification::SAFE_WRITABLE_ROWS;

#[test]
fn disabled_final_pre_enable_audit_copy_is_available_and_wired() {
    let lines = disabled_pre_enable_audit_ui_lines();
    for expected in [
        "Final pre-enable audit",
        "The first write pilot is not ready yet.",
        "The pre-enable audit stage is complete.",
        "The next gate still needs a separate review and approval.",
        "All production write gates are still disabled.",
        "Real writing is not active yet.",
        "Apply behavior has not changed.",
    ] {
        assert!(lines.iter().any(|line| line == expected));
    }

    let window_source = fs::read_to_string("src/ui/window.rs").expect("window source should read");
    assert!(window_source.contains("disabled_pre_enable_audit_ui_lines"));
    assert!(window_source.contains("set_sensitive(false)"));
    assert!(!window_source.contains("PRODUCTION_ONE_TARGET_WRITE_PILOT_ENABLED = true"));
    assert_eq!(SAFE_WRITABLE_ROWS.len(), 341);
}
