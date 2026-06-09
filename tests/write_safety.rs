use std::collections::BTreeSet;
use std::fs;
use std::path::PathBuf;
use std::time::{SystemTime, UNIX_EPOCH};

use anyhow::Result;
use hyprland_settings::config_backup::BackupManager;
use hyprland_settings::config_parser::parse_hyprland_config_text;
use hyprland_settings::current_config::{CurrentConfigSnapshot, CurrentValueProjection};
use hyprland_settings::pending_change::{stage_pending_change, ACTIVE_PENDING_CHANGE_SETTING};
use hyprland_settings::write_safety::{
    review_write_plan, WriteGateFailure, WritePlanAction, WritePlanRequest,
};

fn temp_root(name: &str) -> Result<PathBuf> {
    let stamp = SystemTime::now().duration_since(UNIX_EPOCH)?.as_nanos();
    let root = std::env::temp_dir().join(format!(
        "hyprland-settings-write-safety-{name}-{}-{stamp}",
        std::process::id()
    ));
    fs::create_dir_all(&root)?;
    Ok(root)
}

fn known_ids(ids: &[&str]) -> BTreeSet<String> {
    ids.iter().map(|id| id.to_string()).collect()
}

fn current_value_for(path: &str, setting_id: &str, contents: &str) -> CurrentValueProjection {
    let parsed = parse_hyprland_config_text(path, contents);
    CurrentConfigSnapshot::from_parsed(parsed).value_for(setting_id)
}

#[test]
fn valid_plan_for_windows_snap_enabled_is_approved() -> Result<()> {
    let root = temp_root("valid")?;
    let source = root.join("hyprland.conf");
    fs::write(&source, "general:snap:enabled = false\n")?;
    let backup = BackupManager::new(root.join("backups")).create_backup(&source)?;
    let current = current_value_for(
        source.to_str().expect("utf-8 fixture path"),
        "general.snap.enabled",
        "general:snap:enabled = false\n",
    );
    let pending = stage_pending_change(ACTIVE_PENDING_CHANGE_SETTING, &current, "true");

    let review = review_write_plan(WritePlanRequest {
        known_setting_ids: known_ids(&[ACTIVE_PENDING_CHANGE_SETTING]),
        detected_config_path: source.clone(),
        current_value: current,
        pending_change: pending,
        backup: Some(backup),
    });

    assert!(review.is_approved());
    let plan = review.plan.expect("valid review should produce plan");
    assert_eq!(plan.setting_id, ACTIVE_PENDING_CHANGE_SETTING);
    assert_eq!(plan.target_path, source);
    assert_eq!(plan.action, WritePlanAction::ReplaceLine { line_number: 1 });
    assert_eq!(plan.old_value.as_deref(), Some("false"));
    assert_eq!(plan.proposed_value, "true");

    fs::remove_dir_all(root)?;
    Ok(())
}

#[test]
fn valid_plan_for_validator_backed_numeric_row_is_approved() -> Result<()> {
    let root = temp_root("valid-numeric")?;
    let source = root.join("hyprland.conf");
    fs::write(&source, "decoration:blur:size = 8\n")?;
    let backup = BackupManager::new(root.join("backups")).create_backup(&source)?;
    let current = current_value_for(
        source.to_str().expect("utf-8 fixture path"),
        "decoration.blur.size",
        "decoration:blur:size = 8\n",
    );
    let pending = stage_pending_change("appearance.blur.size", &current, "10");

    let review = review_write_plan(WritePlanRequest {
        known_setting_ids: known_ids(&["appearance.blur.size"]),
        detected_config_path: source.clone(),
        current_value: current,
        pending_change: pending,
        backup: Some(backup),
    });

    assert!(review.is_approved());
    let plan = review.plan.expect("valid review should produce plan");
    assert_eq!(plan.setting_id, "appearance.blur.size");
    assert_eq!(plan.target_path, source);
    assert_eq!(plan.action, WritePlanAction::ReplaceLine { line_number: 1 });
    assert_eq!(plan.old_value.as_deref(), Some("8"));
    assert_eq!(plan.proposed_value, "10");

    fs::remove_dir_all(root)?;
    Ok(())
}

#[test]
fn rejects_unknown_setting() {
    let current = CurrentValueProjection::not_configured();
    let pending = stage_pending_change("unknown.setting", &current, "true");

    let review = review_write_plan(WritePlanRequest {
        known_setting_ids: BTreeSet::new(),
        detected_config_path: PathBuf::from("/tmp/hyprland.conf"),
        current_value: current,
        pending_change: pending,
        backup: None,
    });

    assert!(review.failures.contains(&WriteGateFailure::UnknownSetting));
}

#[test]
fn rejects_unsafe_setting() {
    let current = current_value_for(
        "/tmp/hyprland.conf",
        "cursor.default_monitor",
        "cursor:default_monitor = HDMI-A-1\n",
    );
    let pending = stage_pending_change("cursor.default_monitor", &current, "DP-1");

    let review = review_write_plan(WritePlanRequest {
        known_setting_ids: known_ids(&["cursor.default_monitor"]),
        detected_config_path: PathBuf::from("/tmp/hyprland.conf"),
        current_value: current,
        pending_change: pending,
        backup: None,
    });

    assert!(review.failures.contains(&WriteGateFailure::NotAllowlisted));
}

#[test]
fn rejects_missing_backup() {
    let current = current_value_for(
        "/tmp/hyprland.conf",
        "general.snap.enabled",
        "general:snap:enabled = false\n",
    );
    let pending = stage_pending_change(ACTIVE_PENDING_CHANGE_SETTING, &current, "true");

    let review = review_write_plan(WritePlanRequest {
        known_setting_ids: known_ids(&[ACTIVE_PENDING_CHANGE_SETTING]),
        detected_config_path: PathBuf::from("/tmp/hyprland.conf"),
        current_value: current,
        pending_change: pending,
        backup: None,
    });

    assert!(review.failures.contains(&WriteGateFailure::MissingBackup));
}

#[test]
fn rejects_duplicate_conflict() {
    let current = current_value_for(
        "/tmp/hyprland.conf",
        "general.snap.enabled",
        "general:snap:enabled = false\ngeneral:snap:enabled = true\n",
    );
    let pending = stage_pending_change(ACTIVE_PENDING_CHANGE_SETTING, &current, "false");

    let review = review_write_plan(WritePlanRequest {
        known_setting_ids: known_ids(&[ACTIVE_PENDING_CHANGE_SETTING]),
        detected_config_path: PathBuf::from("/tmp/hyprland.conf"),
        current_value: current,
        pending_change: pending,
        backup: None,
    });

    assert!(review
        .failures
        .contains(&WriteGateFailure::DuplicateConflict));
}

#[test]
fn rejects_structured_family() {
    let current = CurrentValueProjection::not_configured();
    let pending = stage_pending_change("hl.monitor", &current, "monitor = ,preferred,auto,1");

    let review = review_write_plan(WritePlanRequest {
        known_setting_ids: known_ids(&["hl.monitor"]),
        detected_config_path: PathBuf::from("/tmp/hyprland.conf"),
        current_value: current,
        pending_change: pending,
        backup: None,
    });

    assert!(review
        .failures
        .contains(&WriteGateFailure::StructuredFamilyRejected));
}

#[test]
fn rejects_invalid_value() {
    let current = current_value_for(
        "/tmp/hyprland.conf",
        "general.snap.enabled",
        "general:snap:enabled = false\n",
    );
    let pending = stage_pending_change(ACTIVE_PENDING_CHANGE_SETTING, &current, "maybe");

    let review = review_write_plan(WritePlanRequest {
        known_setting_ids: known_ids(&[ACTIVE_PENDING_CHANGE_SETTING]),
        detected_config_path: PathBuf::from("/tmp/hyprland.conf"),
        current_value: current,
        pending_change: pending,
        backup: None,
    });

    assert!(review
        .failures
        .iter()
        .any(|failure| matches!(failure, WriteGateFailure::InvalidProposedValue(_))));
}
