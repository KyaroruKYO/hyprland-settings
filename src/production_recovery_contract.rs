use std::fs;
use std::path::PathBuf;

use crate::config_parser::{parse_hyprland_config_file, ParseStatus};
use crate::production_backup_contract::FixtureBackupContractProof;
use crate::write_target_candidate::WriteTargetCandidate;

pub const PRODUCTION_RECOVERY_CONTRACT_ENABLED: bool = true;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ProductionRecoveryContract {
    pub backup_exists_required: bool,
    pub backup_verified_before_write_required: bool,
    pub restore_exact_backup_bytes_required: bool,
    pub reread_restored_file_required: bool,
    pub report_rollback_success_or_failure_required: bool,
    pub hyprland_reload_allowed: bool,
    pub production_enabled: bool,
}

impl ProductionRecoveryContract {
    pub fn user_facing_lines(&self) -> Vec<String> {
        vec![
            "Recovery".to_string(),
            "Rollback/recovery must be implemented before real writes.".to_string(),
            "If verification fails in a future version, the app will restore the backup."
                .to_string(),
            "If verification fails, the app must restore the exact backup bytes.".to_string(),
            "The app will reread the restored file before reporting recovery success.".to_string(),
            "This pilot must never reload Hyprland automatically.".to_string(),
            "If recovery fails, the app will report the failure and leave the backup available."
                .to_string(),
            "Recovery approval is staged; real recovery is still not active yet.".to_string(),
        ]
    }
}

pub fn production_recovery_prerequisite_contract() -> ProductionRecoveryContract {
    ProductionRecoveryContract {
        backup_exists_required: true,
        backup_verified_before_write_required: true,
        restore_exact_backup_bytes_required: true,
        reread_restored_file_required: true,
        report_rollback_success_or_failure_required: true,
        hyprland_reload_allowed: false,
        production_enabled: PRODUCTION_RECOVERY_CONTRACT_ENABLED,
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum RecoveryTriggerCondition {
    WriteFailedAfterBackup,
    WriteSucceededVerificationFailed,
    ExpectedSettingMissingAfterWrite,
    ExpectedValueMismatchAfterWrite,
    TargetUnreadableAfterWrite,
    BackupIntegrityMissingBeforeWrite,
    BackupRestoreFailed,
    UserCancellationBeforeWrite,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum RecoveryTriggerAction {
    ShouldRestoreBackup,
    ShouldNotRestoreBackup,
    ShouldReportFailureOnly,
    ShouldBlockBeforeWriteBegins,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RecoveryTriggerDecision {
    pub condition: RecoveryTriggerCondition,
    pub action: RecoveryTriggerAction,
    pub production_enabled: bool,
    pub fixture_only_recovery_allowed: bool,
    pub hyprland_reload_allowed: bool,
}

impl RecoveryTriggerDecision {
    pub fn should_restore_backup(&self) -> bool {
        self.action == RecoveryTriggerAction::ShouldRestoreBackup
    }
}

pub fn recovery_trigger_decision(condition: RecoveryTriggerCondition) -> RecoveryTriggerDecision {
    let action = match condition {
        RecoveryTriggerCondition::WriteFailedAfterBackup
        | RecoveryTriggerCondition::WriteSucceededVerificationFailed
        | RecoveryTriggerCondition::ExpectedSettingMissingAfterWrite
        | RecoveryTriggerCondition::ExpectedValueMismatchAfterWrite
        | RecoveryTriggerCondition::TargetUnreadableAfterWrite => {
            RecoveryTriggerAction::ShouldRestoreBackup
        }
        RecoveryTriggerCondition::BackupIntegrityMissingBeforeWrite => {
            RecoveryTriggerAction::ShouldBlockBeforeWriteBegins
        }
        RecoveryTriggerCondition::BackupRestoreFailed => {
            RecoveryTriggerAction::ShouldReportFailureOnly
        }
        RecoveryTriggerCondition::UserCancellationBeforeWrite => {
            RecoveryTriggerAction::ShouldNotRestoreBackup
        }
    };

    RecoveryTriggerDecision {
        condition,
        action,
        production_enabled: PRODUCTION_RECOVERY_CONTRACT_ENABLED,
        fixture_only_recovery_allowed: true,
        hyprland_reload_allowed: false,
    }
}

pub fn all_recovery_trigger_decisions() -> Vec<RecoveryTriggerDecision> {
    vec![
        recovery_trigger_decision(RecoveryTriggerCondition::WriteFailedAfterBackup),
        recovery_trigger_decision(RecoveryTriggerCondition::WriteSucceededVerificationFailed),
        recovery_trigger_decision(RecoveryTriggerCondition::ExpectedSettingMissingAfterWrite),
        recovery_trigger_decision(RecoveryTriggerCondition::ExpectedValueMismatchAfterWrite),
        recovery_trigger_decision(RecoveryTriggerCondition::TargetUnreadableAfterWrite),
        recovery_trigger_decision(RecoveryTriggerCondition::BackupIntegrityMissingBeforeWrite),
        recovery_trigger_decision(RecoveryTriggerCondition::BackupRestoreFailed),
        recovery_trigger_decision(RecoveryTriggerCondition::UserCancellationBeforeWrite),
    ]
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum RestoreOperationStatus {
    NotRun,
    Planned,
    PassedInFixture,
    FailedInFixture,
    ProductionDisabled,
    BlockedBeforeWrite,
}

impl RestoreOperationStatus {
    pub fn label(self) -> &'static str {
        match self {
            Self::NotRun => "not run",
            Self::Planned => "planned",
            Self::PassedInFixture => "passed in fixture",
            Self::FailedInFixture => "failed in fixture",
            Self::ProductionDisabled => "production disabled",
            Self::BlockedBeforeWrite => "blocked before write",
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RecoveryRestoreOperation {
    pub target_file_path: PathBuf,
    pub backup_file_path: PathBuf,
    pub backup_byte_length: Option<u64>,
    pub target_byte_length_before_restore: Option<u64>,
    pub restore_status: RestoreOperationStatus,
    pub writes_exact_backup_bytes: bool,
    pub modifies_only_target_file: bool,
    pub follows_symlink_target: bool,
    pub generated_script_symlink_targets_blocked: bool,
    pub production_enabled: bool,
    pub fixture_only: bool,
}

pub fn planned_recovery_restore_operation(
    candidate: &WriteTargetCandidate,
    backup_path: impl Into<PathBuf>,
    backup_byte_length: Option<u64>,
) -> RecoveryRestoreOperation {
    let blocked = candidate.generated_or_script_managed || candidate.symlink_managed;
    RecoveryRestoreOperation {
        target_file_path: candidate.file_path.clone(),
        backup_file_path: backup_path.into(),
        backup_byte_length,
        target_byte_length_before_restore: None,
        restore_status: if blocked {
            RestoreOperationStatus::BlockedBeforeWrite
        } else {
            RestoreOperationStatus::ProductionDisabled
        },
        writes_exact_backup_bytes: true,
        modifies_only_target_file: true,
        follows_symlink_target: false,
        generated_script_symlink_targets_blocked: blocked,
        production_enabled: PRODUCTION_RECOVERY_CONTRACT_ENABLED,
        fixture_only: true,
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum FixtureRestoreError {
    NonFixturePath,
    BlockedTarget,
    ReadBackupFailed(String),
    ReadTargetFailed(String),
    WriteTargetFailed(String),
}

pub fn fixture_restore_backup_bytes(
    operation: &RecoveryRestoreOperation,
) -> Result<RecoveryRestoreOperation, FixtureRestoreError> {
    if !operation.target_file_path.starts_with(std::env::temp_dir())
        || !operation.backup_file_path.starts_with(std::env::temp_dir())
    {
        return Err(FixtureRestoreError::NonFixturePath);
    }
    if operation.generated_script_symlink_targets_blocked {
        return Err(FixtureRestoreError::BlockedTarget);
    }

    let backup = fs::read(&operation.backup_file_path)
        .map_err(|error| FixtureRestoreError::ReadBackupFailed(error.to_string()))?;
    let before = fs::metadata(&operation.target_file_path)
        .map_err(|error| FixtureRestoreError::ReadTargetFailed(error.to_string()))?
        .len();
    fs::write(&operation.target_file_path, &backup)
        .map_err(|error| FixtureRestoreError::WriteTargetFailed(error.to_string()))?;

    Ok(RecoveryRestoreOperation {
        backup_byte_length: Some(backup.len() as u64),
        target_byte_length_before_restore: Some(before),
        restore_status: RestoreOperationStatus::PassedInFixture,
        ..operation.clone()
    })
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum RestoreVerificationStatus {
    NotRun,
    Planned,
    PassedInFixture,
    FailedInFixture,
    ProductionDisabled,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RestoreVerification {
    pub target_file_path: PathBuf,
    pub restored_backup_path: PathBuf,
    pub reread_method: String,
    pub expected_restored_value: Option<String>,
    pub observed_restored_value: Option<String>,
    pub restore_verification_status: RestoreVerificationStatus,
    pub failure_reasons: Vec<String>,
    pub bytes_match_backup: bool,
    pub production_enabled: bool,
    pub fixture_only: bool,
}

pub fn planned_restore_verification(
    target_file_path: impl Into<PathBuf>,
    restored_backup_path: impl Into<PathBuf>,
    expected_restored_value: Option<String>,
) -> RestoreVerification {
    RestoreVerification {
        target_file_path: target_file_path.into(),
        restored_backup_path: restored_backup_path.into(),
        reread_method:
            "reread exact target file, compare bytes with backup, and optionally parse original scalar value"
                .to_string(),
        expected_restored_value,
        observed_restored_value: None,
        restore_verification_status: RestoreVerificationStatus::ProductionDisabled,
        failure_reasons: vec![
            "restored file bytes did not match backup bytes".to_string(),
            "restored scalar value did not match original value".to_string(),
            "restored file could not be reread".to_string(),
        ],
        bytes_match_backup: false,
        production_enabled: PRODUCTION_RECOVERY_CONTRACT_ENABLED,
        fixture_only: true,
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum FixtureRestoreVerificationError {
    NonFixturePath,
    ReadTargetFailed(String),
    ReadBackupFailed(String),
    ParseTargetFailed(String),
}

pub fn fixture_verify_restored_file(
    verification: &RestoreVerification,
    setting_id: Option<&str>,
) -> Result<RestoreVerification, FixtureRestoreVerificationError> {
    if !verification
        .target_file_path
        .starts_with(std::env::temp_dir())
        || !verification
            .restored_backup_path
            .starts_with(std::env::temp_dir())
    {
        return Err(FixtureRestoreVerificationError::NonFixturePath);
    }

    let target = fs::read(&verification.target_file_path)
        .map_err(|error| FixtureRestoreVerificationError::ReadTargetFailed(error.to_string()))?;
    let backup = fs::read(&verification.restored_backup_path)
        .map_err(|error| FixtureRestoreVerificationError::ReadBackupFailed(error.to_string()))?;
    let bytes_match_backup = target == backup;
    let observed_restored_value = if let Some(setting_id) = setting_id {
        let parsed =
            parse_hyprland_config_file(&verification.target_file_path).map_err(|error| {
                FixtureRestoreVerificationError::ParseTargetFailed(error.to_string())
            })?;
        parsed
            .scalar_records()
            .filter(|record| {
                record.status == ParseStatus::Scalar
                    && record.normalized_setting_id.as_deref() == Some(setting_id)
            })
            .filter_map(|record| record.raw_value.clone())
            .last()
    } else {
        None
    };
    let value_matches = verification
        .expected_restored_value
        .as_ref()
        .map(|expected| observed_restored_value.as_ref() == Some(expected))
        .unwrap_or(true);
    let status = if bytes_match_backup && value_matches {
        RestoreVerificationStatus::PassedInFixture
    } else {
        RestoreVerificationStatus::FailedInFixture
    };

    Ok(RestoreVerification {
        observed_restored_value,
        restore_verification_status: status,
        bytes_match_backup,
        ..verification.clone()
    })
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum RecoveryReportStatus {
    RecoveryAttempted,
    RecoverySkipped,
    RecoverySucceeded,
    RecoveryFailed,
    ProductionDisabled,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RecoveryReport {
    pub recovery_attempted: bool,
    pub recovery_skipped: bool,
    pub trigger_reason: RecoveryTriggerCondition,
    pub backup_path: PathBuf,
    pub target_path: PathBuf,
    pub restore_status: RestoreOperationStatus,
    pub restore_verification_status: RestoreVerificationStatus,
    pub status: RecoveryReportStatus,
    pub user_facing_summary: String,
    pub safe_next_action: String,
    pub production_enabled: bool,
    pub fixture_only: bool,
}

pub fn recovery_report_for(
    trigger: &RecoveryTriggerDecision,
    restore: &RecoveryRestoreOperation,
    verification: &RestoreVerification,
) -> RecoveryReport {
    let status = match (
        trigger.action,
        restore.restore_status,
        verification.restore_verification_status,
    ) {
        (RecoveryTriggerAction::ShouldNotRestoreBackup, _, _) => {
            RecoveryReportStatus::RecoverySkipped
        }
        (
            _,
            RestoreOperationStatus::PassedInFixture,
            RestoreVerificationStatus::PassedInFixture,
        ) => RecoveryReportStatus::RecoverySucceeded,
        (_, RestoreOperationStatus::FailedInFixture, _)
        | (_, _, RestoreVerificationStatus::FailedInFixture) => {
            RecoveryReportStatus::RecoveryFailed
        }
        _ if !PRODUCTION_RECOVERY_CONTRACT_ENABLED => RecoveryReportStatus::ProductionDisabled,
        _ => RecoveryReportStatus::RecoveryAttempted,
    };
    let user_facing_summary = match status {
        RecoveryReportStatus::RecoverySucceeded => {
            "Recovery restored the backup in fixture proof.".to_string()
        }
        RecoveryReportStatus::RecoveryFailed => {
            "If recovery fails, the app will report the failure and leave the backup available."
                .to_string()
        }
        RecoveryReportStatus::RecoverySkipped => {
            "Recovery was skipped because no write had started.".to_string()
        }
        RecoveryReportStatus::ProductionDisabled | RecoveryReportStatus::RecoveryAttempted => {
            "Recovery approval is staged; real recovery is still not active yet.".to_string()
        }
    };
    let safe_next_action = match status {
        RecoveryReportStatus::RecoverySucceeded => {
            "Review the restored file before trying again.".to_string()
        }
        RecoveryReportStatus::RecoveryFailed => {
            "Keep the backup file and report the failure.".to_string()
        }
        RecoveryReportStatus::RecoverySkipped => "No restore is needed.".to_string(),
        RecoveryReportStatus::ProductionDisabled | RecoveryReportStatus::RecoveryAttempted => {
            "Real writing remains disabled.".to_string()
        }
    };

    RecoveryReport {
        recovery_attempted: matches!(
            status,
            RecoveryReportStatus::RecoveryAttempted
                | RecoveryReportStatus::RecoverySucceeded
                | RecoveryReportStatus::RecoveryFailed
        ),
        recovery_skipped: status == RecoveryReportStatus::RecoverySkipped,
        trigger_reason: trigger.condition,
        backup_path: restore.backup_file_path.clone(),
        target_path: restore.target_file_path.clone(),
        restore_status: restore.restore_status,
        restore_verification_status: verification.restore_verification_status,
        status,
        user_facing_summary,
        safe_next_action,
        production_enabled: PRODUCTION_RECOVERY_CONTRACT_ENABLED,
        fixture_only: true,
    }
}

pub fn planned_restore_from_backup_proof(
    candidate: &WriteTargetCandidate,
    backup: &FixtureBackupContractProof,
) -> RecoveryRestoreOperation {
    planned_recovery_restore_operation(
        candidate,
        backup.backup_path.clone(),
        Some(backup.backup_metadata.byte_len),
    )
}
