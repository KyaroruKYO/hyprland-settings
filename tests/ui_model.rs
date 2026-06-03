use std::path::Path;

use anyhow::Result;
use hyprland_settings::config_discovery::{ConfigDiscovery, ConfigDiscoveryStatus};
use hyprland_settings::config_parser::parse_hyprland_config_text;
use hyprland_settings::current_config::{CurrentConfigSnapshot, CurrentValueSourceStatus};
use hyprland_settings::export::ExportBundle;
use hyprland_settings::metadata::resolve_metadata_path_with_env;
use hyprland_settings::ui::model::UiProjection;
use hyprland_settings::validation::validate_bundle;

fn load_projection() -> Result<UiProjection> {
    load_projection_with_current_config(CurrentConfigSnapshot::read_unavailable(
        "test fixture has no live config",
    ))
}

fn load_projection_with_current_config(
    current_config: CurrentConfigSnapshot,
) -> Result<UiProjection> {
    let resolution = resolve_metadata_path_with_env(None, None)?;
    let bundle = ExportBundle::load(Path::new(&resolution.export_dir))?;
    let summary = validate_bundle(&bundle)?;
    Ok(UiProjection::from_bundle(
        &bundle,
        &summary,
        ConfigDiscovery {
            status: ConfigDiscoveryStatus::Missing,
            attempted_paths: Vec::new(),
        },
        current_config,
    ))
}

#[test]
fn ui_projection_has_expected_tab_count() -> Result<()> {
    let projection = load_projection()?;

    assert_eq!(projection.tabs.len(), 12);

    Ok(())
}

#[test]
fn ui_projection_defaults_to_appearance_tab_data() -> Result<()> {
    let projection = load_projection()?;
    let appearance = projection
        .tabs
        .iter()
        .find(|tab| tab.id == "appearance")
        .expect("appearance tab missing");

    assert_eq!(appearance.label, "Appearance");
    assert_eq!(appearance.row_count, 48);
    assert!(projection
        .settings
        .iter()
        .any(|setting| setting.tab_id == "appearance"));

    Ok(())
}

#[test]
fn ui_projection_has_expected_summary_counts() -> Result<()> {
    let projection = load_projection()?;

    assert_eq!(projection.summary.inventory_rows, 341);
    assert_eq!(projection.summary.official_scalar_covered, 341);
    assert_eq!(projection.summary.official_scalar_total, 341);
    assert_eq!(projection.summary.read_allowlist_rows, 232);
    assert_eq!(projection.summary.non_read_rows, 109);
    assert_eq!(projection.summary.structured_family_count, 7);

    Ok(())
}

#[test]
fn ui_projection_preserves_write_safety_metadata() -> Result<()> {
    let projection = load_projection()?;

    assert_eq!(projection.active_write_candidates.len(), 1);
    let candidate = &projection.active_write_candidates[0];
    assert_eq!(candidate.row_id, "windows.snap.enabled");
    assert_eq!(candidate.target_mode, "pending-change-only");
    assert!(!candidate.executable);
    assert!(!candidate.command_generation_allowed);

    Ok(())
}

#[test]
fn ui_projection_selected_tab_rows_are_read_only_metadata() -> Result<()> {
    let projection = load_projection()?;
    let appearance_rows: Vec<_> = projection
        .settings
        .iter()
        .filter(|setting| setting.tab_id == "appearance")
        .collect();

    assert_eq!(appearance_rows.len(), 48);
    let forbidden_command = ["hypr", "ctl"].concat();
    for setting in appearance_rows {
        assert!(!setting.label.is_empty());
        assert!(!setting.official_setting.is_empty());
        assert!(!setting.row_id.is_empty());
        assert!(!setting.read_support.is_empty());
        assert!(!setting.write_support.is_empty());
        assert!(!setting.label.contains(forbidden_command.as_str()));
        assert!(!setting
            .official_setting
            .contains(forbidden_command.as_str()));
        assert!(!setting.description.contains(forbidden_command.as_str()));
        assert!(!setting.read_support.contains(forbidden_command.as_str()));
        assert!(!setting.write_support.contains(forbidden_command.as_str()));
    }

    Ok(())
}

#[test]
fn ui_projection_marks_configured_current_value() -> Result<()> {
    let parsed =
        parse_hyprland_config_text("/tmp/current-values.conf", "animations:enabled = true\n");
    let projection =
        load_projection_with_current_config(CurrentConfigSnapshot::from_parsed(parsed))?;
    let detail = projection
        .detail_for_row("animations.enabled")
        .expect("animations.enabled detail should exist");

    assert_eq!(
        detail.current_value.status,
        CurrentValueSourceStatus::Configured
    );
    assert_eq!(detail.current_value.raw_value.as_deref(), Some("true"));
    assert_eq!(detail.current_value.line_number, Some(1));
    assert_eq!(
        detail.current_value.raw_line.as_deref(),
        Some("animations:enabled = true")
    );
    assert_eq!(detail.comparison.badge, "User configured");
    assert!(detail.comparison.detail.contains("user override present"));
    assert!(detail
        .comparison
        .detail
        .contains("official default value is not exported"));

    Ok(())
}

#[test]
fn ui_projection_marks_unconfigured_current_value_after_config_load() -> Result<()> {
    let parsed =
        parse_hyprland_config_text("/tmp/current-values.conf", "animations:enabled = true\n");
    let projection =
        load_projection_with_current_config(CurrentConfigSnapshot::from_parsed(parsed))?;
    let detail = projection
        .detail_for_row("windows.snap.enabled")
        .expect("windows.snap.enabled detail should exist");

    assert_eq!(
        detail.current_value.status,
        CurrentValueSourceStatus::NotConfigured
    );
    assert_eq!(detail.current_value.raw_value, None);
    assert_eq!(detail.comparison.badge, "Default");
    assert!(detail.comparison.detail.contains("no user override found"));

    Ok(())
}

#[test]
fn ui_projection_marks_duplicate_current_value_conflict() -> Result<()> {
    let parsed = parse_hyprland_config_text(
        "/tmp/current-values.conf",
        "animations:enabled = true\nanimations:enabled = false\n",
    );
    let projection =
        load_projection_with_current_config(CurrentConfigSnapshot::from_parsed(parsed))?;
    let detail = projection
        .detail_for_row("animations.enabled")
        .expect("animations.enabled detail should exist");

    assert_eq!(
        detail.current_value.status,
        CurrentValueSourceStatus::DuplicateConflict
    );
    assert_eq!(detail.current_value.raw_value.as_deref(), Some("false"));
    assert_eq!(detail.current_value.duplicate_lines, vec![1, 2]);
    assert_eq!(detail.comparison.badge, "Conflict");
    assert!(detail
        .comparison
        .detail
        .contains("duplicate user config entries"));

    Ok(())
}

#[test]
fn ui_projection_groups_structured_family_entries_read_only() -> Result<()> {
    let parsed = parse_hyprland_config_text(
        "/tmp/structured-ui.conf",
        r#"
monitor = ,preferred,auto,1
bind = SUPER,Return,exec,kitty
animation = windows,1,7,default
bezier = snappy,0.2,0.8,0.2,1.0
device {
  name = test-device
}
permission = /usr/bin/app, screencopy, allow
"#,
    );
    let projection =
        load_projection_with_current_config(CurrentConfigSnapshot::from_parsed(parsed))?;

    let monitor = projection
        .structured_families
        .iter()
        .find(|family| family.family_id == "hl.monitor")
        .expect("monitor family should be projected");
    assert_eq!(monitor.label, "Monitors");
    assert_eq!(monitor.entries.len(), 1);
    assert_eq!(monitor.entries[0].line_number, 2);
    assert!(monitor.entries[0].raw_line.contains("monitor ="));
    assert!(monitor.edit_status.contains("read-only"));

    let device = projection
        .structured_families
        .iter()
        .find(|family| family.family_id == "hl.device")
        .expect("device family should be projected");
    assert_eq!(device.entries.len(), 3);
    assert!(device.warning_count >= 1);

    let permission = projection
        .structured_families
        .iter()
        .find(|family| family.family_id == "hl.permission")
        .expect("permission family should be projected");
    assert_eq!(permission.entries.len(), 1);
    assert!(permission.entries[0].raw_line.contains("permission ="));

    Ok(())
}
