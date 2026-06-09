use std::env;
use std::fmt;
use std::fs::File;
use std::path::{Path, PathBuf};

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ConfigDiscovery {
    pub status: ConfigDiscoveryStatus,
    pub attempted_paths: Vec<PathBuf>,
}

impl ConfigDiscovery {
    pub fn summary(&self) -> String {
        match &self.status {
            ConfigDiscoveryStatus::Found { path, source } => {
                format!("Detected Hyprland config: {} ({source})", path.display())
            }
            ConfigDiscoveryStatus::Missing => {
                if self.attempted_paths.is_empty() {
                    "No Hyprland config path could be derived.".to_string()
                } else {
                    let attempted = self
                        .attempted_paths
                        .iter()
                        .map(|path| path.display().to_string())
                        .collect::<Vec<_>>()
                        .join(", ");
                    format!("No Hyprland config found. Attempted: {attempted}")
                }
            }
            ConfigDiscoveryStatus::Unreadable {
                path,
                error,
                source,
            } => format!(
                "Hyprland config exists but is not readable: {} ({source}): {error}",
                path.display()
            ),
            ConfigDiscoveryStatus::NotAFile { path, source } => {
                format!(
                    "Hyprland config path is not a regular file: {} ({source})",
                    path.display()
                )
            }
        }
    }

    pub fn live_read_status(&self) -> &'static str {
        "Discovery only. Config contents were not read."
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ConfigDiscoveryStatus {
    Found {
        path: PathBuf,
        source: ConfigPathSource,
    },
    Missing,
    Unreadable {
        path: PathBuf,
        source: ConfigPathSource,
        error: String,
    },
    NotAFile {
        path: PathBuf,
        source: ConfigPathSource,
    },
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ConfigPathSource {
    XdgConfigHome,
    HomeFallback,
}

impl fmt::Display for ConfigPathSource {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::XdgConfigHome => f.write_str("$XDG_CONFIG_HOME"),
            Self::HomeFallback => f.write_str("~/.config"),
        }
    }
}

#[derive(Debug, Clone, Default)]
pub struct ConfigDiscoveryEnv {
    pub xdg_config_home: Option<PathBuf>,
    pub home: Option<PathBuf>,
}

impl ConfigDiscoveryEnv {
    pub fn from_process() -> Self {
        Self {
            xdg_config_home: non_empty_env_path("XDG_CONFIG_HOME"),
            home: non_empty_env_path("HOME"),
        }
    }
}

pub fn discover_hyprland_config() -> ConfigDiscovery {
    discover_hyprland_config_with_env(&ConfigDiscoveryEnv::from_process())
}

pub fn discover_hyprland_config_with_env(env: &ConfigDiscoveryEnv) -> ConfigDiscovery {
    let candidates = config_candidates(env);
    let attempted_paths = candidates
        .iter()
        .map(|(_, path)| path.clone())
        .collect::<Vec<_>>();

    for (source, path) in candidates {
        if !path.exists() {
            continue;
        }
        if !path.is_file() {
            return ConfigDiscovery {
                status: ConfigDiscoveryStatus::NotAFile { path, source },
                attempted_paths,
            };
        }
        if let Err(error) = File::open(&path) {
            return ConfigDiscovery {
                status: ConfigDiscoveryStatus::Unreadable {
                    path,
                    source,
                    error: error.to_string(),
                },
                attempted_paths,
            };
        }
        return ConfigDiscovery {
            status: ConfigDiscoveryStatus::Found { path, source },
            attempted_paths,
        };
    }

    ConfigDiscovery {
        status: ConfigDiscoveryStatus::Missing,
        attempted_paths,
    }
}

pub fn config_candidates(env: &ConfigDiscoveryEnv) -> Vec<(ConfigPathSource, PathBuf)> {
    let mut candidates = Vec::new();
    if let Some(xdg_config_home) = &env.xdg_config_home {
        candidates.push((
            ConfigPathSource::XdgConfigHome,
            hyprland_conf_path(xdg_config_home),
        ));
    }
    if let Some(home) = &env.home {
        candidates.push((
            ConfigPathSource::HomeFallback,
            home.join(".config").join("hypr").join("hyprland.conf"),
        ));
    }
    candidates
}

fn hyprland_conf_path(config_home: &Path) -> PathBuf {
    config_home.join("hypr").join("hyprland.conf")
}

fn non_empty_env_path(key: &str) -> Option<PathBuf> {
    env::var_os(key).and_then(|value| {
        if value.is_empty() {
            None
        } else {
            Some(PathBuf::from(value))
        }
    })
}
