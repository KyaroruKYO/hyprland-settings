use std::path::Path;

use anyhow::Result;
use hyprland_settings::export::ExportBundle;
use hyprland_settings::metadata::resolve_metadata_path_with_env;
use hyprland_settings::ui::model::UiProjection;
use hyprland_settings::validation::validate_bundle;

fn load_projection() -> Result<UiProjection> {
    let resolution = resolve_metadata_path_with_env(None, None)?;
    let bundle = ExportBundle::load(Path::new(&resolution.export_dir))?;
    let summary = validate_bundle(&bundle)?;
    Ok(UiProjection::from_bundle(&bundle, &summary))
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
