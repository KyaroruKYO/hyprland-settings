//! Shared fail-closed filesystem primitives for active-config writes.
//!
//! Callers must capture a [`FilePrecondition`] while building their review
//! model and pass that same precondition to the commit. The commit rejects any
//! byte, inode, path, file-type, ownership, or mode drift before replacement.

use std::ffi::CString;
use std::fs::{self, File, OpenOptions};
use std::io::{Read, Write};
use std::os::unix::ffi::OsStrExt;
use std::os::unix::fs::{MetadataExt, OpenOptionsExt, PermissionsExt};
use std::path::{Component, Path, PathBuf};

use sha2::{Digest, Sha256};
use tempfile::Builder;
use thiserror::Error;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct FileMetadataSnapshot {
    pub device: u64,
    pub inode: u64,
    pub uid: u32,
    pub gid: u32,
    pub mode: u32,
    pub byte_len: u64,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct FilePrecondition {
    pub requested_path: PathBuf,
    pub canonical_path: PathBuf,
    pub canonical_parent: PathBuf,
    pub parent_device: u64,
    pub parent_inode: u64,
    pub bytes: Vec<u8>,
    pub sha256: String,
    pub metadata: FileMetadataSnapshot,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct DurableWriteReceipt {
    pub target_path: PathBuf,
    pub before_sha256: String,
    pub after_sha256: String,
    pub bytes_written: usize,
    pub target_mode: u32,
    pub target_uid: u32,
    pub target_gid: u32,
    pub file_synced: bool,
    pub parent_directory_synced: bool,
    pub final_bytes_verified: bool,
    pub final_metadata_verified: bool,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum DurableWriteTestFault {
    None,
    FailBeforeCommit,
    FailCommit,
    ReplaceTargetImmediatelyBeforeCommit,
    FailPostCommitVerification,
}

#[derive(Debug, Error)]
pub enum DurableFsError {
    #[error("OnDiskDriftDetected: {0}")]
    OnDiskDriftDetected(String),
    #[error("TargetIdentityChanged: {0}")]
    TargetIdentityChanged(String),
    #[error("TargetNotRegularFile: {0}")]
    TargetNotRegularFile(String),
    #[error("TargetSymlinkRejected: {0}")]
    TargetSymlinkRejected(String),
    #[error("ParentPathSymlinkRejected: {0}")]
    ParentPathSymlinkRejected(String),
    #[error("ParentIdentityChanged: {0}")]
    ParentIdentityChanged(String),
    #[error("OwnershipCannotBePreserved: {0}")]
    OwnershipCannotBePreserved(String),
    #[error("TemporaryWriteFailed: {0}")]
    TemporaryWriteFailed(String),
    #[error("CommitFailed: {0}")]
    CommitFailed(String),
    #[error("PostWriteVerificationFailed: {0}")]
    PostWriteVerificationFailed(String),
    #[error("RestoreFailed: {0}")]
    RestoreFailed(String),
    #[error("FilesystemOperationFailed: {0}")]
    FilesystemOperationFailed(String),
}

impl DurableFsError {
    pub fn user_message(&self) -> &'static str {
        match self {
            Self::OnDiskDriftDetected(_)
            | Self::TargetIdentityChanged(_)
            | Self::TargetSymlinkRejected(_)
            | Self::ParentIdentityChanged(_)
            | Self::ParentPathSymlinkRejected(_) => {
                "The config changed on disk after this edit was prepared. Nothing was written. Reload or reread the setting before saving again."
            }
            _ => "The config write did not complete. Nothing was marked saved.",
        }
    }

    pub fn code(&self) -> &'static str {
        match self {
            Self::OnDiskDriftDetected(_) => "OnDiskDriftDetected",
            Self::TargetIdentityChanged(_) => "TargetIdentityChanged",
            Self::TargetNotRegularFile(_) => "TargetNotRegularFile",
            Self::TargetSymlinkRejected(_) => "TargetSymlinkRejected",
            Self::ParentPathSymlinkRejected(_) => "ParentPathSymlinkRejected",
            Self::ParentIdentityChanged(_) => "ParentIdentityChanged",
            Self::OwnershipCannotBePreserved(_) => "OwnershipCannotBePreserved",
            Self::TemporaryWriteFailed(_) => "TemporaryWriteFailed",
            Self::CommitFailed(_) => "CommitFailed",
            Self::PostWriteVerificationFailed(_) => "PostWriteVerificationFailed",
            Self::RestoreFailed(_) => "RestoreFailed",
            Self::FilesystemOperationFailed(_) => "FilesystemOperationFailed",
        }
    }
}

pub fn content_sha256(bytes: &[u8]) -> String {
    format!("{:x}", Sha256::digest(bytes))
}

pub fn capture_file_precondition(
    path: impl AsRef<Path>,
) -> Result<FilePrecondition, DurableFsError> {
    let path = path.as_ref();
    reject_symlink_components(path)?;
    let metadata = fs::symlink_metadata(path).map_err(|error| {
        DurableFsError::FilesystemOperationFailed(format!(
            "failed to inspect {}: {error}",
            path.display()
        ))
    })?;
    if metadata.file_type().is_symlink() {
        return Err(DurableFsError::TargetSymlinkRejected(
            path.display().to_string(),
        ));
    }
    if !metadata.file_type().is_file() {
        return Err(DurableFsError::TargetNotRegularFile(
            path.display().to_string(),
        ));
    }
    let canonical_path = fs::canonicalize(path).map_err(|error| {
        DurableFsError::FilesystemOperationFailed(format!(
            "failed to canonicalize {}: {error}",
            path.display()
        ))
    })?;
    let parent = path.parent().ok_or_else(|| {
        DurableFsError::FilesystemOperationFailed(format!(
            "target {} has no parent",
            path.display()
        ))
    })?;
    let canonical_parent = fs::canonicalize(parent).map_err(|error| {
        DurableFsError::FilesystemOperationFailed(format!(
            "failed to canonicalize parent {}: {error}",
            parent.display()
        ))
    })?;
    let parent_metadata = fs::symlink_metadata(parent).map_err(|error| {
        DurableFsError::FilesystemOperationFailed(format!(
            "failed to inspect parent {}: {error}",
            parent.display()
        ))
    })?;
    if !parent_metadata.file_type().is_dir() || parent_metadata.file_type().is_symlink() {
        return Err(DurableFsError::ParentPathSymlinkRejected(
            parent.display().to_string(),
        ));
    }
    if canonical_path.parent() != Some(canonical_parent.as_path()) {
        return Err(DurableFsError::TargetIdentityChanged(format!(
            "{} did not resolve beneath its expected parent",
            path.display()
        )));
    }

    let mut file = open_read_no_follow(path)?;
    let opened_metadata = file.metadata().map_err(|error| {
        DurableFsError::FilesystemOperationFailed(format!(
            "failed to inspect opened target {}: {error}",
            path.display()
        ))
    })?;
    if opened_metadata.dev() != metadata.dev() || opened_metadata.ino() != metadata.ino() {
        return Err(DurableFsError::TargetIdentityChanged(format!(
            "{} changed while its precondition was captured",
            path.display()
        )));
    }
    let mut bytes = Vec::new();
    file.read_to_end(&mut bytes).map_err(|error| {
        DurableFsError::FilesystemOperationFailed(format!(
            "failed to read {}: {error}",
            path.display()
        ))
    })?;
    let final_metadata = file.metadata().map_err(|error| {
        DurableFsError::FilesystemOperationFailed(format!(
            "failed to re-inspect opened target {}: {error}",
            path.display()
        ))
    })?;
    if opened_metadata.dev() != final_metadata.dev()
        || opened_metadata.ino() != final_metadata.ino()
        || opened_metadata.len() != final_metadata.len()
    {
        return Err(DurableFsError::TargetIdentityChanged(format!(
            "{} changed while its bytes were read",
            path.display()
        )));
    }

    Ok(FilePrecondition {
        requested_path: path.to_path_buf(),
        canonical_path,
        canonical_parent,
        parent_device: parent_metadata.dev(),
        parent_inode: parent_metadata.ino(),
        sha256: content_sha256(&bytes),
        metadata: metadata_snapshot(&final_metadata),
        bytes,
    })
}

pub fn verify_file_precondition(expected: &FilePrecondition) -> Result<(), DurableFsError> {
    let actual = capture_file_precondition(&expected.requested_path)?;
    if actual.canonical_path != expected.canonical_path
        || actual.canonical_parent != expected.canonical_parent
        || actual.metadata.device != expected.metadata.device
        || actual.metadata.inode != expected.metadata.inode
    {
        return Err(DurableFsError::TargetIdentityChanged(format!(
            "{} no longer identifies the file used to prepare the write",
            expected.requested_path.display()
        )));
    }
    if actual.parent_device != expected.parent_device
        || actual.parent_inode != expected.parent_inode
    {
        return Err(DurableFsError::ParentIdentityChanged(format!(
            "parent directory changed for {}",
            expected.requested_path.display()
        )));
    }
    if actual.metadata.uid != expected.metadata.uid
        || actual.metadata.gid != expected.metadata.gid
        || actual.metadata.mode != expected.metadata.mode
    {
        return Err(DurableFsError::OnDiskDriftDetected(format!(
            "metadata changed for {}",
            expected.requested_path.display()
        )));
    }
    if actual.bytes != expected.bytes || actual.sha256 != expected.sha256 {
        return Err(DurableFsError::OnDiskDriftDetected(format!(
            "bytes changed for {}",
            expected.requested_path.display()
        )));
    }
    Ok(())
}

pub fn hardened_atomic_replace(
    expected: &FilePrecondition,
    replacement: &[u8],
) -> Result<DurableWriteReceipt, DurableFsError> {
    hardened_atomic_replace_with_fault(expected, replacement, DurableWriteTestFault::None)
}

#[doc(hidden)]
pub fn hardened_atomic_replace_with_fault(
    expected: &FilePrecondition,
    replacement: &[u8],
    fault: DurableWriteTestFault,
) -> Result<DurableWriteReceipt, DurableFsError> {
    verify_file_precondition(expected)?;
    let parent = &expected.canonical_parent;
    let mut temporary = Builder::new()
        .prefix(".hyprland-settings-write-")
        .suffix(".tmp")
        .tempfile_in(parent)
        .map_err(|error| {
            DurableFsError::TemporaryWriteFailed(format!(
                "failed to create exclusive temporary file in {}: {error}",
                parent.display()
            ))
        })?;

    temporary
        .as_file()
        .set_permissions(fs::Permissions::from_mode(expected.metadata.mode))
        .map_err(|error| {
            DurableFsError::TemporaryWriteFailed(format!(
                "failed to set temporary file mode: {error}"
            ))
        })?;
    temporary.write_all(replacement).map_err(|error| {
        DurableFsError::TemporaryWriteFailed(format!("failed to write temporary file: {error}"))
    })?;
    temporary.flush().map_err(|error| {
        DurableFsError::TemporaryWriteFailed(format!("failed to flush temporary file: {error}"))
    })?;
    temporary.as_file().sync_all().map_err(|error| {
        DurableFsError::TemporaryWriteFailed(format!("failed to sync temporary file: {error}"))
    })?;

    let temporary_metadata = temporary.as_file().metadata().map_err(|error| {
        DurableFsError::TemporaryWriteFailed(format!("failed to inspect temporary file: {error}"))
    })?;
    if temporary_metadata.uid() != expected.metadata.uid
        || temporary_metadata.gid() != expected.metadata.gid
    {
        return Err(DurableFsError::OwnershipCannotBePreserved(format!(
            "temporary uid:gid {}:{} does not match target {}:{}",
            temporary_metadata.uid(),
            temporary_metadata.gid(),
            expected.metadata.uid,
            expected.metadata.gid
        )));
    }

    // The second check is intentionally adjacent to rename. It catches edits
    // that occur while the staged bytes are being prepared.
    verify_file_precondition(expected)?;
    if fault == DurableWriteTestFault::FailBeforeCommit {
        return Err(DurableFsError::CommitFailed(
            "injected failure before rename".to_string(),
        ));
    }
    if fault == DurableWriteTestFault::FailCommit {
        return Err(DurableFsError::CommitFailed(
            "injected atomic exchange failure".to_string(),
        ));
    }

    if fault == DurableWriteTestFault::ReplaceTargetImmediatelyBeforeCommit {
        fs::write(
            &expected.canonical_path,
            b"# injected concurrent external edit immediately before commit\n",
        )
        .map_err(|error| {
            DurableFsError::FilesystemOperationFailed(format!(
                "failed to inject immediate pre-commit drift: {error}"
            ))
        })?;
    }

    // Atomic exchange closes the final verify/rename race. The displaced
    // path must still contain the exact inode and bytes captured by the plan;
    // otherwise it is exchanged straight back before a drift error returns.
    let temporary_path = temporary.path().to_path_buf();
    exchange_paths(&temporary_path, &expected.canonical_path)?;
    let displaced = capture_file_precondition(&temporary_path);
    let verification = displaced
        .and_then(|actual| verify_displaced_target(expected, &actual))
        .and_then(|()| sync_directory(parent))
        .and_then(|()| {
            if fault == DurableWriteTestFault::FailPostCommitVerification {
                Err(DurableFsError::PostWriteVerificationFailed(
                    "injected post-commit verification failure".to_string(),
                ))
            } else {
                verify_replacement(expected, replacement)
            }
        });

    if let Err(error) = verification {
        exchange_paths(&temporary_path, &expected.canonical_path).map_err(|restore_error| {
            DurableFsError::RestoreFailed(format!(
                "{error}; atomic exchange restore also failed: {restore_error}"
            ))
        })?;
        sync_directory(parent).map_err(|restore_error| {
            DurableFsError::RestoreFailed(format!(
                "{error}; restored target but directory sync failed: {restore_error}"
            ))
        })?;
        return Err(error);
    }

    if let Err(remove_error) = fs::remove_file(&temporary_path) {
        exchange_paths(&temporary_path, &expected.canonical_path).map_err(|restore_error| {
            DurableFsError::RestoreFailed(format!(
                "failed to remove displaced target: {remove_error}; atomic restore also failed: {restore_error}"
            ))
        })?;
        sync_directory(parent).map_err(|restore_error| {
            DurableFsError::RestoreFailed(format!(
                "failed to remove displaced target: {remove_error}; restored target but directory sync failed: {restore_error}"
            ))
        })?;
        return Err(DurableFsError::CommitFailed(format!(
            "failed to remove displaced target after verified exchange: {remove_error}"
        )));
    }
    drop(temporary);

    Ok(DurableWriteReceipt {
        target_path: expected.requested_path.clone(),
        before_sha256: expected.sha256.clone(),
        after_sha256: content_sha256(replacement),
        bytes_written: replacement.len(),
        target_mode: expected.metadata.mode,
        target_uid: expected.metadata.uid,
        target_gid: expected.metadata.gid,
        file_synced: true,
        parent_directory_synced: true,
        final_bytes_verified: true,
        final_metadata_verified: true,
    })
}

fn verify_displaced_target(
    expected: &FilePrecondition,
    displaced: &FilePrecondition,
) -> Result<(), DurableFsError> {
    if displaced.metadata.device != expected.metadata.device
        || displaced.metadata.inode != expected.metadata.inode
    {
        return Err(DurableFsError::TargetIdentityChanged(format!(
            "{} changed at the atomic commit boundary",
            expected.requested_path.display()
        )));
    }
    if displaced.metadata.uid != expected.metadata.uid
        || displaced.metadata.gid != expected.metadata.gid
        || displaced.metadata.mode != expected.metadata.mode
        || displaced.bytes != expected.bytes
        || displaced.sha256 != expected.sha256
    {
        return Err(DurableFsError::OnDiskDriftDetected(format!(
            "{} changed at the atomic commit boundary",
            expected.requested_path.display()
        )));
    }
    Ok(())
}

fn exchange_paths(left: &Path, right: &Path) -> Result<(), DurableFsError> {
    let left = CString::new(left.as_os_str().as_bytes()).map_err(|_| {
        DurableFsError::CommitFailed("temporary path contains an interior NUL".to_string())
    })?;
    let right = CString::new(right.as_os_str().as_bytes()).map_err(|_| {
        DurableFsError::CommitFailed("target path contains an interior NUL".to_string())
    })?;
    let result = unsafe {
        libc::renameat2(
            libc::AT_FDCWD,
            left.as_ptr(),
            libc::AT_FDCWD,
            right.as_ptr(),
            libc::RENAME_EXCHANGE,
        )
    };
    if result == 0 {
        Ok(())
    } else {
        Err(DurableFsError::CommitFailed(format!(
            "atomic rename exchange failed: {}",
            std::io::Error::last_os_error()
        )))
    }
}

pub fn restore_original_after_failed_verification(
    original: &FilePrecondition,
    expected_committed_bytes: &[u8],
) -> Result<DurableWriteReceipt, DurableFsError> {
    restore_original_after_failed_commit(original, expected_committed_bytes)
}

fn restore_original_after_failed_commit(
    original: &FilePrecondition,
    expected_committed_bytes: &[u8],
) -> Result<DurableWriteReceipt, DurableFsError> {
    let committed = capture_file_precondition(&original.canonical_path)?;
    if committed.bytes != expected_committed_bytes {
        return Err(DurableFsError::RestoreFailed(format!(
            "refusing restore because committed bytes changed at {}",
            original.canonical_path.display()
        )));
    }
    let receipt = hardened_atomic_replace(&committed, &original.bytes)?;
    let restored = capture_file_precondition(&original.canonical_path)?;
    if restored.bytes != original.bytes
        || restored.metadata.mode != original.metadata.mode
        || restored.metadata.uid != original.metadata.uid
        || restored.metadata.gid != original.metadata.gid
    {
        return Err(DurableFsError::RestoreFailed(format!(
            "restored bytes or metadata did not match {}",
            original.requested_path.display()
        )));
    }
    Ok(receipt)
}

fn verify_replacement(
    original: &FilePrecondition,
    replacement: &[u8],
) -> Result<(), DurableFsError> {
    let actual = capture_file_precondition(&original.canonical_path)?;
    if actual.bytes != replacement {
        return Err(DurableFsError::PostWriteVerificationFailed(format!(
            "final bytes differ at {}",
            original.requested_path.display()
        )));
    }
    if actual.metadata.mode != original.metadata.mode
        || actual.metadata.uid != original.metadata.uid
        || actual.metadata.gid != original.metadata.gid
    {
        return Err(DurableFsError::PostWriteVerificationFailed(format!(
            "final metadata differs at {}",
            original.requested_path.display()
        )));
    }
    let requested = capture_file_precondition(&original.requested_path)?;
    if requested.canonical_path != original.canonical_path
        || requested.metadata.device != actual.metadata.device
        || requested.metadata.inode != actual.metadata.inode
    {
        return Err(DurableFsError::TargetIdentityChanged(format!(
            "{} no longer resolves to the committed target",
            original.requested_path.display()
        )));
    }
    Ok(())
}

fn sync_directory(path: &Path) -> Result<(), DurableFsError> {
    File::open(path)
        .and_then(|directory| directory.sync_all())
        .map_err(|error| {
            DurableFsError::FilesystemOperationFailed(format!(
                "failed to sync directory {}: {error}",
                path.display()
            ))
        })
}

fn open_read_no_follow(path: &Path) -> Result<File, DurableFsError> {
    OpenOptions::new()
        .read(true)
        .custom_flags(libc::O_NOFOLLOW)
        .open(path)
        .map_err(|error| {
            DurableFsError::FilesystemOperationFailed(format!(
                "failed to open {} without following symlinks: {error}",
                path.display()
            ))
        })
}

fn metadata_snapshot(metadata: &fs::Metadata) -> FileMetadataSnapshot {
    FileMetadataSnapshot {
        device: metadata.dev(),
        inode: metadata.ino(),
        uid: metadata.uid(),
        gid: metadata.gid(),
        mode: metadata.mode() & 0o7777,
        byte_len: metadata.len(),
    }
}

fn reject_symlink_components(path: &Path) -> Result<(), DurableFsError> {
    let mut current = PathBuf::new();
    for component in path.components() {
        match component {
            Component::RootDir | Component::Prefix(_) => current.push(component.as_os_str()),
            Component::CurDir => continue,
            Component::ParentDir => {
                return Err(DurableFsError::ParentPathSymlinkRejected(format!(
                    "parent traversal is not allowed in {}",
                    path.display()
                )))
            }
            Component::Normal(part) => current.push(part),
        }
        if current.as_os_str().is_empty() || !current.exists() {
            continue;
        }
        let metadata = fs::symlink_metadata(&current).map_err(|error| {
            DurableFsError::FilesystemOperationFailed(format!(
                "failed to inspect path component {}: {error}",
                current.display()
            ))
        })?;
        if metadata.file_type().is_symlink() {
            return Err(if current == path {
                DurableFsError::TargetSymlinkRejected(current.display().to_string())
            } else {
                DurableFsError::ParentPathSymlinkRejected(current.display().to_string())
            });
        }
    }
    Ok(())
}
