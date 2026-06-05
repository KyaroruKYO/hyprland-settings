use std::collections::BTreeSet;
use std::fs;
use std::path::PathBuf;
use std::time::{SystemTime, UNIX_EPOCH};

use anyhow::Result;
use hyprland_settings::config_backup::BackupManager;
use hyprland_settings::config_parser::parse_hyprland_config_text;
use hyprland_settings::current_config::{CurrentConfigSnapshot, CurrentValueProjection};
use hyprland_settings::pending_change::{stage_pending_change, ACTIVE_PENDING_CHANGE_SETTING};
use hyprland_settings::write_pilot::apply_windows_snap_enabled_plan;
use hyprland_settings::write_safety::{review_write_plan, WriteGateFailure, WritePlanRequest};

fn temp_root(name: &str) -> Result<PathBuf> {
    let stamp = SystemTime::now().duration_since(UNIX_EPOCH)?.as_nanos();
    let root = std::env::temp_dir().join(format!(
        "hyprland-settings-write-pilot-{name}-{}-{stamp}",
        std::process::id()
    ));
    fs::create_dir_all(&root)?;
    Ok(root)
}

fn known_ids() -> BTreeSet<String> {
    [ACTIVE_PENDING_CHANGE_SETTING]
        .into_iter()
        .map(str::to_string)
        .collect()
}

fn current_value_for(path: &PathBuf, setting_id: &str, contents: &str) -> CurrentValueProjection {
    let parsed = parse_hyprland_config_text(path, contents);
    CurrentConfigSnapshot::from_parsed(parsed).value_for(setting_id)
}

#[test]
fn write_pilot_replaces_existing_windows_snap_line() -> Result<()> {
    let root = temp_root("replace")?;
    let source = root.join("hyprland.conf");
    fs::write(
        &source,
        "general:gaps_in = 5\ngeneral:snap:enabled = false # keep comment\n",
    )?;
    let backup = BackupManager::new(root.join("backups")).create_backup(&source)?;
    let current = current_value_for(
        &source,
        "general.snap.enabled",
        &fs::read_to_string(&source)?,
    );
    let pending = stage_pending_change(ACTIVE_PENDING_CHANGE_SETTING, &current, "true");
    let review = review_write_plan(WritePlanRequest {
        known_setting_ids: known_ids(),
        detected_config_path: source.clone(),
        current_value: current,
        pending_change: pending,
        backup: Some(backup),
    });
    let plan = review.plan.expect("valid plan expected");

    let result = apply_windows_snap_enabled_plan(&plan)?;

    assert_eq!(result.verified_value.as_deref(), Some("true"));
    assert_eq!(
        fs::read_to_string(&source)?,
        "general:gaps_in = 5\ngeneral:snap:enabled = true # keep comment\n"
    );

    fs::remove_dir_all(root)?;
    Ok(())
}

#[test]
fn write_pilot_appends_missing_windows_snap_setting() -> Result<()> {
    let root = temp_root("append")?;
    let source = root.join("hyprland.conf");
    fs::write(&source, "general:gaps_in = 5")?;
    let backup = BackupManager::new(root.join("backups")).create_backup(&source)?;
    let current = CurrentValueProjection::not_configured();
    let pending = stage_pending_change(ACTIVE_PENDING_CHANGE_SETTING, &current, "false");
    let review = review_write_plan(WritePlanRequest {
        known_setting_ids: known_ids(),
        detected_config_path: source.clone(),
        current_value: current,
        pending_change: pending,
        backup: Some(backup),
    });
    let plan = review.plan.expect("append plan expected");

    let result = apply_windows_snap_enabled_plan(&plan)?;

    assert_eq!(result.verified_value.as_deref(), Some("false"));
    assert_eq!(
        fs::read_to_string(&source)?,
        "general:gaps_in = 5\ngeneral:snap:enabled = false\n"
    );

    fs::remove_dir_all(root)?;
    Ok(())
}

#[test]
fn backup_is_required_before_write_pilot_plan() -> Result<()> {
    let root = temp_root("backup-required")?;
    let source = root.join("hyprland.conf");
    fs::write(&source, "general:snap:enabled = false\n")?;
    let current = current_value_for(
        &source,
        "general.snap.enabled",
        &fs::read_to_string(&source)?,
    );
    let pending = stage_pending_change(ACTIVE_PENDING_CHANGE_SETTING, &current, "true");

    let review = review_write_plan(WritePlanRequest {
        known_setting_ids: known_ids(),
        detected_config_path: source,
        current_value: current,
        pending_change: pending,
        backup: None,
    });

    assert!(review.failures.contains(&WriteGateFailure::MissingBackup));

    fs::remove_dir_all(root)?;
    Ok(())
}

#[test]
fn rollback_restores_after_write_pilot() -> Result<()> {
    let root = temp_root("rollback")?;
    let source = root.join("hyprland.conf");
    fs::write(&source, "general:snap:enabled = false\n")?;
    let manager = BackupManager::new(root.join("backups"));
    let backup = manager.create_backup(&source)?;
    let current = current_value_for(
        &source,
        "general.snap.enabled",
        &fs::read_to_string(&source)?,
    );
    let pending = stage_pending_change(ACTIVE_PENDING_CHANGE_SETTING, &current, "true");
    let review = review_write_plan(WritePlanRequest {
        known_setting_ids: known_ids(),
        detected_config_path: source.clone(),
        current_value: current,
        pending_change: pending,
        backup: Some(backup.clone()),
    });

    apply_windows_snap_enabled_plan(&review.plan.expect("valid plan expected"))?;
    manager.rollback(&backup)?;

    assert_eq!(
        fs::read_to_string(&source)?,
        "general:snap:enabled = false\n"
    );

    fs::remove_dir_all(root)?;
    Ok(())
}

#[test]
fn invalid_value_is_rejected_before_write_pilot() -> Result<()> {
    let root = temp_root("invalid")?;
    let source = root.join("hyprland.conf");
    fs::write(&source, "general:snap:enabled = false\n")?;
    let current = current_value_for(
        &source,
        "general.snap.enabled",
        &fs::read_to_string(&source)?,
    );
    let pending = stage_pending_change(ACTIVE_PENDING_CHANGE_SETTING, &current, "maybe");
    let review = review_write_plan(WritePlanRequest {
        known_setting_ids: known_ids(),
        detected_config_path: source,
        current_value: current,
        pending_change: pending,
        backup: None,
    });

    assert!(review
        .failures
        .iter()
        .any(|failure| matches!(failure, WriteGateFailure::InvalidProposedValue(_))));

    fs::remove_dir_all(root)?;
    Ok(())
}

#[test]
fn no_other_setting_gets_writable_plan() {
    let current = CurrentValueProjection::not_configured();
    let pending = stage_pending_change("input.kb_layout", &current, "us");
    let review = review_write_plan(WritePlanRequest {
        known_setting_ids: ["input.kb_layout"]
            .into_iter()
            .map(str::to_string)
            .collect(),
        detected_config_path: PathBuf::from("/tmp/hyprland.conf"),
        current_value: current,
        pending_change: pending,
        backup: None,
    });

    assert!(review.failures.contains(&WriteGateFailure::NotAllowlisted));
    assert!(review.plan.is_none());
}
