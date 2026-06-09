use std::path::Path;

use anyhow::Result;
use hyprland_settings::config_discovery::{ConfigDiscovery, ConfigDiscoveryStatus};
use hyprland_settings::current_config::CurrentConfigSnapshot;
use hyprland_settings::export::ExportBundle;
use hyprland_settings::metadata::resolve_metadata_path_with_env;
use hyprland_settings::search::{search_projection, SearchRank};
use hyprland_settings::ui::model::UiProjection;
use hyprland_settings::validation::validate_bundle;

fn load_projection() -> Result<UiProjection> {
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
        CurrentConfigSnapshot::read_unavailable("test fixture has no live config"),
    ))
}

#[test]
fn empty_search_returns_selected_tab_rows() -> Result<()> {
    let projection = load_projection()?;
    let results = search_projection(&projection, "appearance", "   ");

    assert!(!results.is_searching);
    assert_eq!(results.results.len(), 48);
    assert!(results
        .results
        .iter()
        .all(|result| result.setting.tab_id == "appearance"));

    Ok(())
}

#[test]
fn search_by_exact_row_id_finds_row() -> Result<()> {
    let projection = load_projection()?;
    let results = search_projection(&projection, "appearance", "windows.snap.enabled");

    assert!(results.is_searching);
    assert_eq!(results.results[0].setting.row_id, "windows.snap.enabled");
    assert_eq!(results.results[0].rank, Some(SearchRank::ExactKey));

    Ok(())
}

#[test]
fn search_by_official_setting_finds_row() -> Result<()> {
    let projection = load_projection()?;
    let results = search_projection(&projection, "appearance", "animations.enabled");

    assert_eq!(results.results[0].setting.row_id, "animations.enabled");
    assert_eq!(results.results[0].rank, Some(SearchRank::ExactKey));

    Ok(())
}

#[test]
fn search_by_label_or_description_finds_expected_rows() -> Result<()> {
    let projection = load_projection()?;
    let blur_results = search_projection(&projection, "appearance", "blur");
    let workspace_results = search_projection(&projection, "appearance", "workspace wraparound");

    assert!(blur_results
        .results
        .iter()
        .any(|result| result.setting.row_id == "appearance.blur.enabled"));
    assert!(workspace_results
        .results
        .iter()
        .any(|result| result.setting.row_id == "animations.workspace_wraparound"));

    Ok(())
}

#[test]
fn search_is_case_insensitive() -> Result<()> {
    let projection = load_projection()?;
    let lower = search_projection(&projection, "appearance", "snap enabled");
    let mixed = search_projection(&projection, "appearance", "SnAp EnAbLeD");

    let lower_ids: Vec<_> = lower
        .results
        .iter()
        .map(|result| result.setting.row_id.as_str())
        .collect();
    let mixed_ids: Vec<_> = mixed
        .results
        .iter()
        .map(|result| result.setting.row_id.as_str())
        .collect();

    assert_eq!(lower_ids, mixed_ids);
    assert!(lower_ids.contains(&"windows.snap.enabled"));

    Ok(())
}

#[test]
fn search_results_are_deterministic() -> Result<()> {
    let projection = load_projection()?;
    let first = search_projection(&projection, "appearance", "snap");
    let second = search_projection(&projection, "appearance", "snap");

    let first_ids: Vec<_> = first
        .results
        .iter()
        .map(|result| result.setting.row_id.as_str())
        .collect();
    let second_ids: Vec<_> = second
        .results
        .iter()
        .map(|result| result.setting.row_id.as_str())
        .collect();

    assert_eq!(first_ids, second_ids);

    Ok(())
}
