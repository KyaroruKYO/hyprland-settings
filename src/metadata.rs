use std::fmt;
use std::path::{Path, PathBuf};

use anyhow::{anyhow, Result};

const ENV_EXPORT_DIR: &str = "HYPRLAND_SETTINGS_EXPORT_DIR";
const EXPORT_DIR_SUFFIX: &str = "hyprland-settings/exports/hyprland-0.55.2";
const MANIFEST_FILE: &str = "hyprland-settings-export-manifest.v0.55.2.json";
const PROJECT_RELATIVE_EXPORT_DIR: &str = "data/exports/hyprland-0.55.2";
const SYSTEM_EXPORT_DIR: &str = "/usr/share/hyprland-settings/exports/hyprland-0.55.2";
const TEMPORARY_DEVELOPMENT_FALLBACK: &str = "/home/kyo/.config/hypr/docs/exports/hyprland-0.55.2";

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct MetadataPathResolution {
    pub export_dir: PathBuf,
    pub source: MetadataPathSource,
    pub attempted_paths: Vec<PathBuf>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum MetadataPathSource {
    CliOverride,
    EnvironmentOverride,
    ProjectRelative,
    XdgUserData,
    LocalShareUserData,
    SystemData,
    TemporaryDevelopmentFallback,
}

impl fmt::Display for MetadataPathSource {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let label = match self {
            Self::CliOverride => "CLI override",
            Self::EnvironmentOverride => "environment override",
            Self::ProjectRelative => "project-relative data",
            Self::XdgUserData => "XDG user data",
            Self::LocalShareUserData => "local-share user data",
            Self::SystemData => "system data",
            Self::TemporaryDevelopmentFallback => "temporary development fallback",
        };
        f.write_str(label)
    }
}

pub fn resolve_metadata_path(cli_override: Option<PathBuf>) -> Result<MetadataPathResolution> {
    let env_override = std::env::var_os(ENV_EXPORT_DIR).map(PathBuf::from);
    resolve_metadata_path_with_env(cli_override, env_override)
}

pub fn resolve_metadata_path_with_env(
    cli_override: Option<PathBuf>,
    env_override: Option<PathBuf>,
) -> Result<MetadataPathResolution> {
    if let Some(path) = cli_override {
        return resolve_explicit_path(path, MetadataPathSource::CliOverride);
    }

    if let Some(path) = env_override {
        return resolve_explicit_path(path, MetadataPathSource::EnvironmentOverride);
    }

    let mut attempted_paths = Vec::new();
    for (source, path) in automatic_metadata_paths() {
        attempted_paths.push(path.clone());
        if contains_manifest(&path) {
            return Ok(MetadataPathResolution {
                export_dir: path,
                source,
                attempted_paths,
            });
        }
    }

    Err(missing_metadata_error(&attempted_paths))
}

pub fn automatic_metadata_paths() -> Vec<(MetadataPathSource, PathBuf)> {
    let mut paths = vec![(
        MetadataPathSource::ProjectRelative,
        PathBuf::from(PROJECT_RELATIVE_EXPORT_DIR),
    )];

    if let Some(xdg_data_home) = std::env::var_os("XDG_DATA_HOME") {
        paths.push((
            MetadataPathSource::XdgUserData,
            PathBuf::from(xdg_data_home).join(EXPORT_DIR_SUFFIX),
        ));
    }

    if let Some(home) = std::env::var_os("HOME") {
        paths.push((
            MetadataPathSource::LocalShareUserData,
            PathBuf::from(home)
                .join(".local")
                .join("share")
                .join(EXPORT_DIR_SUFFIX),
        ));
    }

    paths.extend([
        (
            MetadataPathSource::SystemData,
            PathBuf::from(SYSTEM_EXPORT_DIR),
        ),
        (
            MetadataPathSource::TemporaryDevelopmentFallback,
            PathBuf::from(TEMPORARY_DEVELOPMENT_FALLBACK),
        ),
    ]);

    paths
}

fn resolve_explicit_path(
    path: PathBuf,
    source: MetadataPathSource,
) -> Result<MetadataPathResolution> {
    let attempted_paths = vec![path.clone()];
    if contains_manifest(&path) {
        Ok(MetadataPathResolution {
            export_dir: path,
            source,
            attempted_paths,
        })
    } else {
        Err(missing_metadata_error(&attempted_paths))
    }
}

fn contains_manifest(path: &Path) -> bool {
    path.join(MANIFEST_FILE).is_file()
}

fn missing_metadata_error(attempted_paths: &[PathBuf]) -> anyhow::Error {
    let attempted = attempted_paths
        .iter()
        .map(|path| path.display().to_string())
        .collect::<Vec<_>>()
        .join(", ");
    anyhow!("export metadata manifest not found; attempted paths: {attempted}")
}
