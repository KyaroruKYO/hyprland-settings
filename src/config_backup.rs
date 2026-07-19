use std::env;
use std::fs::{self, File, OpenOptions};
use std::io::Write;
use std::os::unix::fs::{MetadataExt, OpenOptionsExt, PermissionsExt};
use std::path::{Component, Path, PathBuf};
use std::time::{SystemTime, UNIX_EPOCH};

use anyhow::{anyhow, Context, Result};
use tempfile::Builder;

use crate::durable_fs::{
    capture_file_precondition, hardened_atomic_replace, verify_file_precondition,
    DurableWriteReceipt, FilePrecondition,
};

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ConfigBackup {
    pub source_path: PathBuf,
    pub backup_path: PathBuf,
    pub byte_len: usize,
    pub source_sha256: String,
    pub created_unix_nanos: u128,
    pub source_precondition: FilePrecondition,
    pub backup_precondition: FilePrecondition,
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

    /// Convenience for fixture callers. Production write flows use
    /// `create_backup_from_precondition` so the backup cannot silently accept
    /// bytes newer than the reviewed snapshot.
    pub fn create_backup(&self, source_path: impl AsRef<Path>) -> Result<ConfigBackup> {
        let precondition = capture_file_precondition(source_path.as_ref())
            .map_err(|error| anyhow!("failed to read source config: {error}"))?;
        self.create_backup_from_precondition(&precondition)
    }

    pub fn create_backup_from_precondition(
        &self,
        source: &FilePrecondition,
    ) -> Result<ConfigBackup> {
        verify_file_precondition(source)
            .map_err(|error| anyhow!("source changed before backup: {error}"))?;
        self.prepare_backup_root()?;

        let source_name = source
            .requested_path
            .file_name()
            .and_then(|name| name.to_str())
            .unwrap_or("hyprland.conf");
        let prefix = format!("{}.backup-", sanitize_file_name(source_name));
        let mut temporary = Builder::new()
            .prefix(&prefix)
            .suffix(".bak")
            .tempfile_in(&self.backup_root)
            .with_context(|| {
                format!(
                    "failed to create exclusive backup in {}",
                    self.backup_root.display()
                )
            })?;
        temporary
            .as_file()
            .set_permissions(fs::Permissions::from_mode(0o600))
            .context("failed to set backup mode to 0600")?;
        temporary
            .write_all(&source.bytes)
            .context("failed to write backup bytes")?;
        temporary.flush().context("failed to flush backup")?;
        temporary
            .as_file()
            .sync_all()
            .context("failed to sync backup")?;
        let (_file, backup_path) = temporary
            .keep()
            .map_err(|error| anyhow!("failed to persist exclusive backup: {}", error.error))?;
        sync_directory(&self.backup_root)?;

        let backup_precondition = match verify_created_backup(&backup_path, source) {
            Ok(precondition) => precondition,
            Err(error) => {
                cleanup_failed_backup(&backup_path, &self.backup_root);
                return Err(error);
            }
        };

        Ok(ConfigBackup {
            source_path: source.requested_path.clone(),
            backup_path,
            byte_len: source.bytes.len(),
            source_sha256: source.sha256.clone(),
            created_unix_nanos: SystemTime::now().duration_since(UNIX_EPOCH)?.as_nanos(),
            source_precondition: source.clone(),
            backup_precondition,
        })
    }

    pub fn create_backup_at_path_from_precondition(
        &self,
        source: &FilePrecondition,
        backup_path: impl AsRef<Path>,
    ) -> Result<ConfigBackup> {
        let backup_path = backup_path.as_ref();
        verify_file_precondition(source)
            .map_err(|error| anyhow!("source changed before backup: {error}"))?;
        self.prepare_backup_root()?;
        if backup_path.parent() != Some(self.backup_root.as_path()) {
            return Err(anyhow!("backup path is outside the configured backup root"));
        }
        let mut file = OpenOptions::new()
            .write(true)
            .create_new(true)
            .mode(0o600)
            .open(backup_path)
            .with_context(|| {
                format!(
                    "failed to create exclusive backup {}",
                    backup_path.display()
                )
            })?;
        file.write_all(&source.bytes)?;
        file.flush()?;
        file.sync_all()?;
        sync_directory(&self.backup_root)?;
        let backup_precondition = match verify_created_backup(backup_path, source) {
            Ok(precondition) => precondition,
            Err(error) => {
                cleanup_failed_backup(backup_path, &self.backup_root);
                return Err(error);
            }
        };
        Ok(ConfigBackup {
            source_path: source.requested_path.clone(),
            backup_path: backup_path.to_path_buf(),
            byte_len: source.bytes.len(),
            source_sha256: source.sha256.clone(),
            created_unix_nanos: SystemTime::now().duration_since(UNIX_EPOCH)?.as_nanos(),
            source_precondition: source.clone(),
            backup_precondition,
        })
    }

    pub fn rollback(
        &self,
        backup: &ConfigBackup,
        expected_current_bytes: &[u8],
    ) -> Result<DurableWriteReceipt> {
        verify_file_precondition(&backup.backup_precondition)
            .map_err(|error| anyhow!("backup identity or bytes changed: {error}"))?;
        if backup.backup_precondition.bytes != backup.source_precondition.bytes
            || backup.backup_precondition.sha256 != backup.source_sha256
            || backup.backup_precondition.metadata.mode != 0o600
        {
            return Err(anyhow!("backup integrity verification failed"));
        }

        let current = capture_file_precondition(&backup.source_path)
            .map_err(|error| anyhow!("restore target validation failed: {error}"))?;
        if current.bytes != expected_current_bytes {
            return Err(anyhow!(
                "restore target changed after the recovery operation was prepared; refusing to overwrite intervening bytes"
            ));
        }
        if current.canonical_path != backup.source_precondition.canonical_path
            || current.canonical_parent != backup.source_precondition.canonical_parent
            || current.metadata.uid != backup.source_precondition.metadata.uid
            || current.metadata.gid != backup.source_precondition.metadata.gid
            || current.metadata.mode != backup.source_precondition.metadata.mode
        {
            return Err(anyhow!(
                "restore target identity or metadata no longer matches the original target"
            ));
        }

        let receipt = hardened_atomic_replace(&current, &backup.backup_precondition.bytes)
            .map_err(|error| anyhow!("hardened restore failed: {error}"))?;
        let restored = capture_file_precondition(&backup.source_path)
            .map_err(|error| anyhow!("post-restore verification failed: {error}"))?;
        if restored.bytes != backup.source_precondition.bytes
            || restored.metadata.mode != backup.source_precondition.metadata.mode
            || restored.metadata.uid != backup.source_precondition.metadata.uid
            || restored.metadata.gid != backup.source_precondition.metadata.gid
        {
            return Err(anyhow!(
                "post-restore bytes or metadata verification failed"
            ));
        }
        Ok(receipt)
    }

    /// Reconstruct a verified receipt for older recovery-plan formats that
    /// persist only paths and the known-good bytes. The backup must still be a
    /// non-symlink regular file with mode 0600 and exact expected contents.
    pub fn load_existing_for_restore(
        &self,
        source_path: impl AsRef<Path>,
        backup_path: impl AsRef<Path>,
        expected_original_bytes: &[u8],
    ) -> Result<ConfigBackup> {
        let current = capture_file_precondition(source_path.as_ref())
            .map_err(|error| anyhow!("restore target validation failed: {error}"))?;
        let backup_precondition = capture_file_precondition(backup_path.as_ref())
            .map_err(|error| anyhow!("backup validation failed: {error}"))?;
        if backup_precondition.metadata.mode != 0o600
            || backup_precondition.bytes != expected_original_bytes
        {
            return Err(anyhow!(
                "existing backup mode or bytes do not match the recovery plan"
            ));
        }
        let mut source_precondition = current;
        source_precondition.bytes = expected_original_bytes.to_vec();
        source_precondition.sha256 = crate::durable_fs::content_sha256(expected_original_bytes);
        source_precondition.metadata.byte_len = expected_original_bytes.len() as u64;
        Ok(ConfigBackup {
            source_path: source_precondition.requested_path.clone(),
            backup_path: backup_precondition.requested_path.clone(),
            byte_len: expected_original_bytes.len(),
            source_sha256: source_precondition.sha256.clone(),
            created_unix_nanos: 0,
            source_precondition,
            backup_precondition,
        })
    }

    fn prepare_backup_root(&self) -> Result<()> {
        reject_symlink_components(&self.backup_root)?;
        fs::create_dir_all(&self.backup_root).with_context(|| {
            format!(
                "failed to create backup directory {}",
                self.backup_root.display()
            )
        })?;
        reject_symlink_components(&self.backup_root)?;
        let metadata = fs::symlink_metadata(&self.backup_root).with_context(|| {
            format!(
                "failed to inspect backup directory {}",
                self.backup_root.display()
            )
        })?;
        if metadata.file_type().is_symlink() || !metadata.file_type().is_dir() {
            return Err(anyhow!(
                "backup root must be a real directory, not a symlink: {}",
                self.backup_root.display()
            ));
        }
        let effective_uid = unsafe { libc::geteuid() };
        if metadata.uid() != effective_uid {
            return Err(anyhow!(
                "backup root must be owned by the current user (expected uid {effective_uid}, found {})",
                metadata.uid()
            ));
        }
        fs::set_permissions(&self.backup_root, fs::Permissions::from_mode(0o700))
            .context("failed to enforce backup directory mode 0700")?;
        let verified_mode = fs::symlink_metadata(&self.backup_root)?
            .permissions()
            .mode()
            & 0o7777;
        if verified_mode != 0o700 {
            return Err(anyhow!("backup directory mode is not 0700"));
        }
        Ok(())
    }
}

fn verify_created_backup(path: &Path, source: &FilePrecondition) -> Result<FilePrecondition> {
    let backup = capture_file_precondition(path)
        .map_err(|error| anyhow!("failed to verify backup: {error}"))?;
    if backup.bytes != source.bytes
        || backup.sha256 != source.sha256
        || backup.metadata.mode != 0o600
        || backup.metadata.uid != unsafe { libc::geteuid() }
    {
        return Err(anyhow!(
            "backup bytes, hash, mode, or ownership verification failed for {}",
            path.display()
        ));
    }
    Ok(backup)
}

fn cleanup_failed_backup(path: &Path, parent: &Path) {
    let _ = fs::remove_file(path);
    let _ = sync_directory(parent);
}

fn reject_symlink_components(path: &Path) -> Result<()> {
    let mut current = PathBuf::new();
    for component in path.components() {
        match component {
            Component::RootDir | Component::Prefix(_) => current.push(component.as_os_str()),
            Component::CurDir => continue,
            Component::ParentDir => {
                return Err(anyhow!(
                    "backup root may not contain parent traversal: {}",
                    path.display()
                ));
            }
            Component::Normal(part) => current.push(part),
        }
        if current.as_os_str().is_empty() || !current.exists() {
            continue;
        }
        if fs::symlink_metadata(&current)?.file_type().is_symlink() {
            return Err(anyhow!(
                "backup root may not contain symlink components: {}",
                current.display()
            ));
        }
    }
    Ok(())
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

fn sync_directory(path: &Path) -> Result<()> {
    File::open(path)
        .and_then(|directory| directory.sync_all())
        .with_context(|| format!("failed to sync backup directory {}", path.display()))
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
