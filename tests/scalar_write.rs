use std::collections::BTreeSet;
use std::fs;
use std::path::PathBuf;
use std::time::{SystemTime, UNIX_EPOCH};

use anyhow::Result;
use hyprland_settings::config_backup::BackupManager;
use hyprland_settings::config_parser::parse_hyprland_config_text;
use hyprland_settings::current_config::{CurrentConfigSnapshot, CurrentValueProjection};
use hyprland_settings::pending_change::stage_pending_change;
use hyprland_settings::scalar_write::apply_scalar_write_plan;
use hyprland_settings::write_classification::{
    config_key_from_official_setting, SafeWritableRow, ScalarWriteValueKind, SAFE_WRITABLE_ROWS,
};
use hyprland_settings::write_safety::{review_write_plan, WritePlanRequest};

fn temp_root(name: &str) -> Result<PathBuf> {
    let stamp = SystemTime::now().duration_since(UNIX_EPOCH)?.as_nanos();
    let root = std::env::temp_dir().join(format!(
        "hyprland-settings-scalar-write-{name}-{}-{stamp}",
        std::process::id()
    ));
    fs::create_dir_all(&root)?;
    Ok(root)
}

fn known_ids() -> BTreeSet<String> {
    SAFE_WRITABLE_ROWS
        .iter()
        .map(|row| row.row_id.to_string())
        .collect()
}

fn current_value_for(path: &PathBuf, setting_id: &str, contents: &str) -> CurrentValueProjection {
    let parsed = parse_hyprland_config_text(path, contents);
    CurrentConfigSnapshot::from_parsed(parsed).value_for(setting_id)
}

fn valid_value_for(row: &SafeWritableRow) -> &'static str {
    if row.value_kind == ScalarWriteValueKind::Color {
        return "rgba(ffffffff)";
    }
    if row.value_kind == ScalarWriteValueKind::Gradient {
        return "rgba(ffffffff) rgba(000000ff) 45deg";
    }
    if row.value_kind == ScalarWriteValueKind::Vector2 {
        return "10,20";
    }
    if row.value_kind == ScalarWriteValueKind::LineSafeString {
        return "JetBrains Mono";
    }
    if row.value_kind == ScalarWriteValueKind::Path {
        return "~/.config/hypr/example.conf";
    }
    if row.value_kind == ScalarWriteValueKind::RegexString {
        return "^(Alacritty|kitty)$";
    }
    match row.row_id {
        "appearance.blur.enabled"
        | "appearance.shadow.enabled"
        | "animations.enabled"
        | "windows.snap.enabled" => "true",
        "appearance.blur.brightness"
        | "appearance.blur.contrast"
        | "appearance.active_opacity"
        | "appearance.inactive_opacity" => "0.75",
        "input.pointer_sensitivity" => "-0.25",
        _ => "10",
    }
}

fn existing_value_for(row: &SafeWritableRow) -> &'static str {
    if row.value_kind == ScalarWriteValueKind::Color {
        return "rgba(000000ff)";
    }
    if row.value_kind == ScalarWriteValueKind::Gradient {
        return "rgba(000000ff)";
    }
    if row.value_kind == ScalarWriteValueKind::Vector2 {
        return "0 0";
    }
    if row.value_kind == ScalarWriteValueKind::LineSafeString {
        return "Sans";
    }
    if row.value_kind == ScalarWriteValueKind::Path {
        return "./old";
    }
    if row.value_kind == ScalarWriteValueKind::RegexString {
        return "firefox";
    }
    match row.row_id {
        "appearance.blur.enabled"
        | "appearance.shadow.enabled"
        | "animations.enabled"
        | "windows.snap.enabled" => "false",
        "appearance.blur.brightness"
        | "appearance.blur.contrast"
        | "appearance.active_opacity"
        | "appearance.inactive_opacity" => "0.5",
        "input.pointer_sensitivity" => "0",
        _ => "5",
    }
}

#[test]
fn generic_scalar_writer_replaces_each_safe_writable_row() -> Result<()> {
    for row in SAFE_WRITABLE_ROWS {
        let root = temp_root(row.row_id)?;
        let source = root.join("hyprland.conf");
        let config_key = config_key_from_official_setting(row.official_setting);
        let proposed = valid_value_for(row);
        fs::write(
            &source,
            format!("{config_key} = {} # keep\n", existing_value_for(row)),
        )?;
        let contents = fs::read_to_string(&source)?;
        let backup = BackupManager::new(root.join("backups")).create_backup(&source)?;
        let current = current_value_for(&source, row.official_setting, &contents);
        let pending = stage_pending_change(row.row_id, &current, proposed);
        let review = review_write_plan(WritePlanRequest {
            known_setting_ids: known_ids(),
            detected_config_path: source.clone(),
            current_value: current,
            pending_change: pending,
            backup: Some(backup),
        });

        let result = apply_scalar_write_plan(&review.plan.expect("safe toggle should plan"))?;

        assert_eq!(result.verified_value.as_deref(), Some(proposed));
        assert_eq!(
            fs::read_to_string(&source)?,
            format!("{config_key} = {proposed} # keep\n")
        );
        fs::remove_dir_all(root)?;
    }
    Ok(())
}

#[test]
fn generic_scalar_writer_appends_missing_safe_writable_row() -> Result<()> {
    for row in SAFE_WRITABLE_ROWS {
        let root = temp_root(&format!("{}-append", row.row_id))?;
        let source = root.join("hyprland.conf");
        let config_key = config_key_from_official_setting(row.official_setting);
        let proposed = valid_value_for(row);
        fs::write(&source, "general:gaps_in = 5\n")?;
        let backup = BackupManager::new(root.join("backups")).create_backup(&source)?;
        let current = CurrentValueProjection::not_configured();
        let pending = stage_pending_change(row.row_id, &current, proposed);
        let review = review_write_plan(WritePlanRequest {
            known_setting_ids: known_ids(),
            detected_config_path: source.clone(),
            current_value: current,
            pending_change: pending,
            backup: Some(backup),
        });

        let result = apply_scalar_write_plan(&review.plan.expect("safe toggle should plan"))?;

        assert_eq!(result.verified_value.as_deref(), Some(proposed));
        assert!(fs::read_to_string(&source)?.contains(&format!("{config_key} = {proposed}")));
        fs::remove_dir_all(root)?;
    }
    Ok(())
}
