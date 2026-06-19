use hyprland_settings::write_classification::SAFE_WRITABLE_ROWS;
use hyprland_settings::write_enablement_readiness::disabled_walkthrough_manual_smoke_checklist;

#[test]
fn manual_smoke_checklist_exists_and_covers_disabled_walkthrough_review() {
    let checklist = disabled_walkthrough_manual_smoke_checklist();
    let lines = checklist.user_facing_lines().join("\n");

    for expected in [
        "Launch the app.",
        "Open a normal settings category.",
        "Select a setting controlled in more than one place.",
        "Inspect the setting detail pane.",
        "Safe batch write",
        "This walkthrough shows what the app would check before writing.",
        "Recommended save location",
        "Backup planned",
        "Verification planned",
        "Safe batch write is available for normal settings.",
        "The app will back up files before writing.",
        "The app will check the result after writing.",
        "If something fails, the app will restore the backup.",
        "Disabled: Target decisions are preview-only",
        "Disabled: Review save location",
        "Disabled: Production enablement is disabled",
        "Must not happen: No config file is edited.",
        "Must not happen: No Hyprland reload is run.",
        "Must not happen: No mutating hyprctl command is run.",
        "Must not happen: Apply writes only eligible safe-batch scalar settings.",
    ] {
        assert!(
            lines.contains(expected),
            "missing checklist text: {expected}"
        );
    }

    assert!(checklist.screenshot_automation.contains("not required"));
    assert_eq!(SAFE_WRITABLE_ROWS.len(), 341);
}
