use std::fs;
use std::path::{Path, PathBuf};

use crate::write_target_candidate::WriteTargetCandidate;

pub const PRODUCTION_BACKUP_CONTRACT_ENABLED: bool = true;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ProductionBackupContract {
    pub target_file_path: PathBuf,
    pub resolved_target_path: Option<PathBuf>,
    pub backup_directory: PathBuf,
    pub backup_filename_policy: String,
    pub timestamp_policy: String,
    pub collision_policy: String,
    pub original_file_metadata_to_record: Vec<String>,
    pub backup_file_metadata_to_record: Vec<String>,
    pub backup_verification_requirement: String,
    pub fixture_only_proof_status: ProductionBackupFixtureProofStatus,
    pub production_enabled: bool,
}

impl ProductionBackupContract {
    pub fn user_facing_lines(&self) -> Vec<String> {
        vec![
            "The app will back up this exact file before saving changes.".to_string(),
            "The backup must match the original file before any write can continue.".to_string(),
            "Backup contract approval is staged; backup creation is still blocked until write execution gates are approved.".to_string(),
        ]
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ProductionBackupFixtureProofStatus {
    NotRun,
    PassedInFixture,
    FailedInFixture,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct BackupPathPolicy {
    pub backup_directory: PathBuf,
    pub timestamp: String,
    pub first_candidate_path: PathBuf,
    pub collision_policy: String,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct BackupFileMetadata {
    pub path: PathBuf,
    pub byte_len: u64,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct FixtureBackupContractProof {
    pub target_path: PathBuf,
    pub backup_path: PathBuf,
    pub original_metadata: BackupFileMetadata,
    pub backup_metadata: BackupFileMetadata,
    pub bytes_equal: bool,
    pub fixture_only: bool,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum FixtureBackupContractError {
    NonFixturePath,
    MissingParentDirectory,
    ReadFailed(String),
    WriteFailed(String),
    MetadataFailed(String),
}

pub fn production_backup_contract_for_candidate(
    candidate: &WriteTargetCandidate,
    timestamp: impl Into<String>,
) -> ProductionBackupContract {
    let timestamp = timestamp.into();
    ProductionBackupContract {
        target_file_path: candidate.file_path.clone(),
        resolved_target_path: candidate.resolved_path.clone(),
        backup_directory: backup_directory_for(&candidate.file_path),
        backup_filename_policy: format!(
            "target filename plus .{timestamp}.bak suffix, with numeric collision suffixes"
        ),
        timestamp_policy: "UTC timestamp supplied by the production write review".to_string(),
        collision_policy: "If the backup path exists, append .1, .2, and so on before .bak"
            .to_string(),
        original_file_metadata_to_record: vec![
            "target path".to_string(),
            "resolved target path if any".to_string(),
            "original byte length".to_string(),
            "planned backup path".to_string(),
        ],
        backup_file_metadata_to_record: vec![
            "backup path".to_string(),
            "backup byte length".to_string(),
            "byte equality with original".to_string(),
        ],
        backup_verification_requirement:
            "Backup bytes must exactly match original bytes before any future write continues."
                .to_string(),
        fixture_only_proof_status: ProductionBackupFixtureProofStatus::NotRun,
        production_enabled: PRODUCTION_BACKUP_CONTRACT_ENABLED,
    }
}

pub fn backup_path_policy_for_target(
    target_file_path: impl AsRef<Path>,
    timestamp: impl Into<String>,
) -> BackupPathPolicy {
    let target_file_path = target_file_path.as_ref();
    let timestamp = timestamp.into();
    BackupPathPolicy {
        backup_directory: backup_directory_for(target_file_path),
        first_candidate_path: backup_path_with_counter(target_file_path, &timestamp, None),
        timestamp,
        collision_policy: "append numeric suffix before .bak without overwriting existing backups"
            .to_string(),
    }
}

pub fn choose_unique_backup_path(
    target_file_path: impl AsRef<Path>,
    timestamp: impl AsRef<str>,
) -> PathBuf {
    let target_file_path = target_file_path.as_ref();
    let timestamp = timestamp.as_ref();
    let first = backup_path_with_counter(target_file_path, timestamp, None);
    if !first.exists() {
        return first;
    }

    for counter in 1.. {
        let candidate = backup_path_with_counter(target_file_path, timestamp, Some(counter));
        if !candidate.exists() {
            return candidate;
        }
    }
    unreachable!("unbounded counter loop should always return a path")
}

pub fn fixture_backup_exact_copy(
    target_file_path: impl AsRef<Path>,
    timestamp: impl AsRef<str>,
) -> Result<FixtureBackupContractProof, FixtureBackupContractError> {
    let target_file_path = target_file_path.as_ref();
    if !target_file_path.starts_with(std::env::temp_dir()) {
        return Err(FixtureBackupContractError::NonFixturePath);
    }
    if target_file_path.parent().is_none() {
        return Err(FixtureBackupContractError::MissingParentDirectory);
    }

    let original = fs::read(target_file_path)
        .map_err(|error| FixtureBackupContractError::ReadFailed(error.to_string()))?;
    let backup_path = choose_unique_backup_path(target_file_path, timestamp.as_ref());
    fs::write(&backup_path, &original)
        .map_err(|error| FixtureBackupContractError::WriteFailed(error.to_string()))?;
    let backup = fs::read(&backup_path)
        .map_err(|error| FixtureBackupContractError::ReadFailed(error.to_string()))?;
    let original_metadata = metadata_for(target_file_path)?;
    let backup_metadata = metadata_for(&backup_path)?;

    Ok(FixtureBackupContractProof {
        target_path: target_file_path.to_path_buf(),
        backup_path,
        original_metadata,
        backup_metadata,
        bytes_equal: original == backup,
        fixture_only: true,
    })
}

fn backup_directory_for(target_file_path: &Path) -> PathBuf {
    target_file_path
        .parent()
        .map(Path::to_path_buf)
        .unwrap_or_else(|| PathBuf::from("."))
}

fn backup_path_with_counter(
    target_file_path: &Path,
    timestamp: &str,
    counter: Option<usize>,
) -> PathBuf {
    let directory = backup_directory_for(target_file_path);
    let file_name = target_file_path
        .file_name()
        .map(|name| name.to_string_lossy().into_owned())
        .unwrap_or_else(|| "config".to_string());
    let suffix = counter
        .map(|counter| format!(".{counter}"))
        .unwrap_or_default();
    directory.join(format!("{file_name}.{timestamp}{suffix}.bak"))
}

fn metadata_for(path: &Path) -> Result<BackupFileMetadata, FixtureBackupContractError> {
    let metadata = fs::metadata(path)
        .map_err(|error| FixtureBackupContractError::MetadataFailed(error.to_string()))?;
    Ok(BackupFileMetadata {
        path: path.to_path_buf(),
        byte_len: metadata.len(),
    })
}
