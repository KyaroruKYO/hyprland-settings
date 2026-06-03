use std::env;
use std::fs::{self, OpenOptions};
use std::io::Write;
use std::path::{Path, PathBuf};
use std::time::{SystemTime, UNIX_EPOCH};

use anyhow::{anyhow, Context, Result};

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ConfigBackup {
    pub source_path: PathBuf,
    pub backup_path: PathBuf,
    pub byte_len: usize,
}

#[derive(Debug, Clone)]
pub struct BackupManager {
    pub backup_root: PathBuf,
}

impl BackupManager {
    pub fn new(backup_root: impl Into<PathBuf>) -> Self {
        Self {
            backup_root: backup_root.into(),
        }
    }

    pub fn default_user_backup_root() -> Result<PathBuf> {
        if let Some(xdg_state_home) = non_empty_env_path("XDG_STATE_HOME") {
            return Ok(xdg_state_home.join("hyprland-settings").join("backups"));
        }
        let home = non_empty_env_path("HOME")
            .ok_or_else(|| anyhow!("HOME is not set; cannot derive backup directory"))?;
        Ok(home
            .join(".local")
            .join("state")
            .join("hyprland-settings")
            .join("backups"))
    }

    pub fn create_backup(&self, source_path: impl AsRef<Path>) -> Result<ConfigBackup> {
        let source_path = source_path.as_ref();
        let source_bytes = fs::read(source_path)
            .with_context(|| format!("failed to read source config {}", source_path.display()))?;
        fs::create_dir_all(&self.backup_root).with_context(|| {
            format!(
                "failed to create backup directory {}",
                self.backup_root.display()
            )
        })?;

        let backup_path = self.unique_backup_path(source_path)?;
        let mut file = OpenOptions::new()
            .write(true)
            .create_new(true)
            .open(&backup_path)
            .with_context(|| format!("failed to create backup {}", backup_path.display()))?;
        file.write_all(&source_bytes)
            .with_context(|| format!("failed to write backup {}", backup_path.display()))?;
        file.sync_all()
            .with_context(|| format!("failed to sync backup {}", backup_path.display()))?;

        let backup_bytes = fs::read(&backup_path)
            .with_context(|| format!("failed to verify backup {}", backup_path.display()))?;
        if backup_bytes != source_bytes {
            return Err(anyhow!(
                "backup verification failed for {}",
                backup_path.display()
            ));
        }

        Ok(ConfigBackup {
            source_path: source_path.to_path_buf(),
            backup_path,
            byte_len: source_bytes.len(),
        })
    }

    pub fn rollback(&self, backup: &ConfigBackup) -> Result<()> {
        let backup_bytes = fs::read(&backup.backup_path)
            .with_context(|| format!("failed to read backup {}", backup.backup_path.display()))?;
        if backup_bytes.len() != backup.byte_len {
            return Err(anyhow!(
                "backup length changed for {}",
                backup.backup_path.display()
            ));
        }

        let parent = backup
            .source_path
            .parent()
            .ok_or_else(|| anyhow!("source path has no parent"))?;
        fs::create_dir_all(parent)?;
        let temp_path = parent.join(format!(
            ".{}.rollback-{}.tmp",
            backup
                .source_path
                .file_name()
                .and_then(|name| name.to_str())
                .unwrap_or("hyprland.conf"),
            unique_stamp()?
        ));
        {
            let mut file = OpenOptions::new()
                .write(true)
                .create_new(true)
                .open(&temp_path)
                .with_context(|| {
                    format!("failed to create rollback temp {}", temp_path.display())
                })?;
            file.write_all(&backup_bytes).with_context(|| {
                format!("failed to write rollback temp {}", temp_path.display())
            })?;
            file.sync_all()
                .with_context(|| format!("failed to sync rollback temp {}", temp_path.display()))?;
        }
        fs::rename(&temp_path, &backup.source_path).with_context(|| {
            format!(
                "failed to replace {} from rollback temp {}",
                backup.source_path.display(),
                temp_path.display()
            )
        })?;

        Ok(())
    }

    fn unique_backup_path(&self, source_path: &Path) -> Result<PathBuf> {
        let file_name = source_path
            .file_name()
            .and_then(|name| name.to_str())
            .unwrap_or("hyprland.conf");
        let sanitized = sanitize_file_name(file_name);
        for attempt in 0..100 {
            let candidate =
                self.backup_root
                    .join(format!("{}.{}.{}.bak", sanitized, unique_stamp()?, attempt));
            if !candidate.exists() {
                return Ok(candidate);
            }
        }
        Err(anyhow!(
            "failed to allocate unique backup path in {}",
            self.backup_root.display()
        ))
    }
}

fn sanitize_file_name(file_name: &str) -> String {
    file_name
        .chars()
        .map(|character| {
            if character.is_ascii_alphanumeric() || matches!(character, '.' | '-' | '_') {
                character
            } else {
                '_'
            }
        })
        .collect()
}

fn unique_stamp() -> Result<String> {
    let nanos = SystemTime::now().duration_since(UNIX_EPOCH)?.as_nanos();
    Ok(format!("{}-{nanos}", std::process::id()))
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
