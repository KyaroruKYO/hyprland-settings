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
    config_key_from_official_setting, finite_choice_options, SafeWritableRow, ScalarWriteValueKind,
    SAFE_WRITABLE_ROWS,
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
    if row.value_kind == ScalarWriteValueKind::SourceBacked {
        return match row.row_id {
            "input.kb_model" => "pc105",
            "input.kb_layout" => "us,de",
            "input.kb_variant" => "intl",
            "input.kb_options" => "grp:alt_shift_toggle,ctrl:nocaps",
            "input.kb_rules" => "evdev",
            _ => "us",
        };
    }
    if row.value_kind == ScalarWriteValueKind::MonitorName {
        return "";
    }
    if row.value_kind == ScalarWriteValueKind::Color {
        return "rgba(ffffffff)";
    }
    if row.value_kind == ScalarWriteValueKind::Gradient {
        return "rgba(ffffffff) rgba(000000ff) 45deg";
    }
    if row.value_kind == ScalarWriteValueKind::Vector2 {
        return "10,20";
    }
    if row.value_kind == ScalarWriteValueKind::NumericList {
        return "0.2 0.0 0.5 1 1.2 1.5";
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
    if row.value_kind == ScalarWriteValueKind::Boolean {
        return "true";
    }
    if row.value_kind == ScalarWriteValueKind::FiniteChoice {
        return finite_choice_options(row.row_id)
            .and_then(|options| options.get(1).or_else(|| options.first()))
            .map(|option| option.raw_value)
            .expect("finite choice row should have verified options");
    }
    if row.row_id == "input.pointer_sensitivity" {
        return "-0.25";
    }
    if row.value_kind == ScalarWriteValueKind::Percent {
        return "0.75";
    }
    if row.value_kind == ScalarWriteValueKind::Number {
        return "10";
    }
    "10"
}

fn existing_value_for(row: &SafeWritableRow) -> &'static str {
    if row.value_kind == ScalarWriteValueKind::SourceBacked {
        return match row.row_id {
            "input.kb_model" => "pc104",
            "input.kb_layout" => "us",
            "input.kb_variant" => "",
            "input.kb_options" => "ctrl:nocaps",
            "input.kb_rules" => "base",
            _ => "us",
        };
    }
    if row.value_kind == ScalarWriteValueKind::MonitorName {
        return "";
    }
    if row.value_kind == ScalarWriteValueKind::Color {
        return "rgba(000000ff)";
    }
    if row.value_kind == ScalarWriteValueKind::Gradient {
        return "rgba(000000ff)";
    }
    if row.value_kind == ScalarWriteValueKind::Vector2 {
        return "0 0";
    }
    if row.value_kind == ScalarWriteValueKind::NumericList {
        return "0.2 0.5 1";
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
    if row.value_kind == ScalarWriteValueKind::Boolean {
        return "false";
    }
    if row.value_kind == ScalarWriteValueKind::FiniteChoice {
        return finite_choice_options(row.row_id)
            .and_then(|options| options.first())
            .map(|option| option.raw_value)
            .expect("finite choice row should have verified options");
    }
    if row.row_id == "input.pointer_sensitivity" {
        return "0";
    }
    if row.value_kind == ScalarWriteValueKind::Percent {
        return "0.5";
    }
    if row.value_kind == ScalarWriteValueKind::Number {
        return "5";
    }
    "5"
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

#[test]
fn finite_choice_writer_roundtrips_every_verified_choice() -> Result<()> {
    for row in SAFE_WRITABLE_ROWS
        .iter()
        .filter(|row| row.value_kind == ScalarWriteValueKind::FiniteChoice)
    {
        let row_id = row.row_id;
        let config_key = config_key_from_official_setting(row.official_setting);
        let options = finite_choice_options(row_id).expect("finite choices should exist");

        for option in options {
            let root = temp_root(&format!("{row_id}-{}", option.raw_value))?;
            let source = root.join("hyprland.conf");
            fs::write(&source, format!("{config_key} = 0\n"))?;
            let contents = fs::read_to_string(&source)?;
            let backup = BackupManager::new(root.join("backups")).create_backup(&source)?;
            let current = current_value_for(&source, row.official_setting, &contents);
            let pending = stage_pending_change(row.row_id, &current, option.raw_value);
            let review = review_write_plan(WritePlanRequest {
                known_setting_ids: known_ids(),
                detected_config_path: source.clone(),
                current_value: current,
                pending_change: pending,
                backup: Some(backup),
            });

            let result = apply_scalar_write_plan(
                &review
                    .plan
                    .expect("verified finite choice should pass write safety"),
            )?;

            assert_eq!(result.verified_value.as_deref(), Some(option.raw_value));
            assert_eq!(
                fs::read_to_string(&source)?,
                format!("{config_key} = {}\n", option.raw_value),
                "{row_id} should roundtrip {} ({})",
                option.raw_value,
                option.label
            );
            fs::remove_dir_all(root)?;
        }
    }
    Ok(())
}
