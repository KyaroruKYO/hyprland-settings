use std::fs;

use hyprland_settings::production_high_risk_approval::{
    disabled_high_risk_approval_ui_lines, future_high_risk_acknowledgement_text,
};
use hyprland_settings::write_classification::SAFE_WRITABLE_ROWS;

#[test]
fn disabled_high_risk_warning_copy_exists_without_handlers() {
    let window = fs::read_to_string("src/ui/window.rs").expect("window source should read");
    let section = window
        .split("Production backup and verification")
        .nth(1)
        .and_then(|section| section.split("Review save location").next())
        .expect("production review section should exist");
    let lines = disabled_high_risk_approval_ui_lines();

    for copy in [
        "High-risk approval",
        "Some settings need extra review before they can ever be written.",
        "High-risk rows are excluded from the first production write pilot.",
        "High-risk approval is not active yet.",
        "Real writing is not active yet.",
        "Apply behavior has not changed.",
    ] {
        assert!(lines.iter().any(|line| line == copy));
    }
    for copy in [
        "I understand this setting may affect session stability.",
        "I understand this setting may require manual recovery if misconfigured.",
        "I understand the app must back up, write, reread, verify, and recover safely.",
        "I understand high-risk approval does not override hard blocks.",
    ] {
        assert!(future_high_risk_acknowledgement_text()
            .iter()
            .any(|line| line == copy));
    }
    assert!(window.contains("disabled_high_risk_approval_ui_lines"));
    assert!(!section.contains("connect_clicked"));
    assert!(!section.contains("apply_setting_change("));
    assert_eq!(SAFE_WRITABLE_ROWS.len(), 341);
}
