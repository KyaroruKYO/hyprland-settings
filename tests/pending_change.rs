use std::path::Path;

use hyprland_settings::config_parser::parse_hyprland_config_text;
use hyprland_settings::current_config::{CurrentConfigSnapshot, CurrentValueProjection};
use hyprland_settings::pending_change::{
    stage_pending_change, PendingChangeValidation, ACTIVE_PENDING_CHANGE_SETTING,
};

fn current_value_for(setting_id: &str, config: &str) -> CurrentValueProjection {
    let parsed = parse_hyprland_config_text("/tmp/pending-change.conf", config);
    CurrentConfigSnapshot::from_parsed(parsed).value_for(setting_id)
}

#[test]
fn valid_pending_change_for_windows_snap_enabled_can_be_staged() {
    let current = current_value_for("general.snap.enabled", "general:snap:enabled = false\n");

    let change = stage_pending_change(ACTIVE_PENDING_CHANGE_SETTING, &current, "true");

    assert_eq!(change.validation, PendingChangeValidation::Valid);
    assert!(change.can_be_applied());
    assert_eq!(change.old_parsed_value.as_deref(), Some("false"));
    assert_eq!(change.proposed_value, "true");
    assert_eq!(
        change.source.as_ref().map(|source| source.line_number),
        Some(1)
    );
}

#[test]
fn invalid_pending_change_for_windows_snap_enabled_is_rejected() {
    let current = current_value_for("general.snap.enabled", "general:snap:enabled = false\n");

    let change = stage_pending_change(ACTIVE_PENDING_CHANGE_SETTING, &current, "maybe");

    assert_eq!(
        change.validation,
        PendingChangeValidation::Invalid {
            reason: "windows.snap.enabled requires a boolean value".to_string(),
        }
    );
    assert!(!change.can_be_applied());
}

#[test]
fn only_windows_snap_enabled_can_be_staged() {
    let current = current_value_for("animations.enabled", "animations:enabled = true\n");

    let change = stage_pending_change("animations.enabled", &current, "false");

    assert_eq!(
        change.validation,
        PendingChangeValidation::NotAllowed {
            reason: "setting is not pending-change allowlisted".to_string(),
        }
    );
    assert_eq!(
        change.non_editable_reason.as_deref(),
        Some("only windows.snap.enabled can be staged")
    );
}

#[test]
fn pending_change_preserves_missing_old_value_for_absent_setting() {
    let current = CurrentValueProjection::not_configured();

    let change = stage_pending_change(ACTIVE_PENDING_CHANGE_SETTING, &current, "false");

    assert_eq!(change.validation, PendingChangeValidation::Valid);
    assert_eq!(change.old_parsed_value, None);
    assert_eq!(change.source, None);
}

#[test]
fn no_write_executor_exists_in_rust_project() {
    let root = Path::new(env!("CARGO_MANIFEST_DIR"));

    assert!(!root.join("src/write_executor.rs").exists());
    assert!(!root.join("src/write_runtime_executor.rs").exists());
    assert!(!root.join("writeRuntimeExecutor.ts").exists());
}
