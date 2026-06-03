use std::path::{Path, PathBuf};

use anyhow::Result;
use hyprland_settings::export::ExportBundle;
use hyprland_settings::metadata::{
    automatic_metadata_paths, resolve_metadata_path_with_env, MetadataPathSource,
};
use hyprland_settings::validation::validate_bundle;

const PROJECT_EXPORT_DIR: &str = "data/exports/hyprland-0.55.2";

#[test]
fn cli_override_wins() -> Result<()> {
    let resolution = resolve_metadata_path_with_env(Some(PathBuf::from(PROJECT_EXPORT_DIR)), None)?;

    assert_eq!(resolution.source, MetadataPathSource::CliOverride);
    assert_eq!(resolution.export_dir, PathBuf::from(PROJECT_EXPORT_DIR));

    Ok(())
}

#[test]
fn project_relative_path_resolves() -> Result<()> {
    let resolution = resolve_metadata_path_with_env(None, None)?;

    assert_eq!(resolution.source, MetadataPathSource::ProjectRelative);
    assert_eq!(resolution.export_dir, PathBuf::from(PROJECT_EXPORT_DIR));

    Ok(())
}

#[test]
fn missing_metadata_reports_attempted_paths() {
    let missing = PathBuf::from("target/nonexistent-export-bundle-for-metadata-test");
    let error = resolve_metadata_path_with_env(Some(missing.clone()), None)
        .expect_err("missing explicit metadata path should fail");
    let error = error.to_string();

    assert!(error.contains("export metadata manifest not found"));
    assert!(error.contains(&missing.display().to_string()));
}

#[test]
fn resolver_does_not_use_live_config_paths() {
    let resolution = resolve_metadata_path_with_env(None, None).expect("metadata should resolve");
    let legacy_conf_name = ["hyprland", "conf"].join(".");
    let legacy_lua_name = ["hyprland", "lua"].join(".");
    for path in resolution
        .attempted_paths
        .iter()
        .chain(automatic_metadata_paths().iter().map(|(_, path)| path))
    {
        assert!(!path_has_component(path, &legacy_conf_name));
        assert!(!path_has_component(path, &legacy_lua_name));
        assert_ne!(path, Path::new(".config/hypr"));
    }
}

#[test]
fn resolved_project_relative_bundle_validates() -> Result<()> {
    let resolution = resolve_metadata_path_with_env(None, None)?;
    let bundle = ExportBundle::load(&resolution.export_dir)?;

    validate_bundle(&bundle)?;

    Ok(())
}

fn path_has_component(path: &Path, needle: &str) -> bool {
    path.components()
        .any(|component| component.as_os_str() == needle)
}
