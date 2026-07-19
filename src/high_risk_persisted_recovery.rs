use std::fmt;
use std::fs::{self, OpenOptions};
use std::io::Write;
use std::os::unix::fs::OpenOptionsExt;
use std::path::{Path, PathBuf};

use serde::{Deserialize, Serialize};

use crate::blocked_row_pre_enablement::{
    blocked_pre_enablement_row, blocked_pre_enablement_rows, BlockedRowBucket,
};
use crate::config_backup::BackupManager;
use crate::config_parser::parse_hyprland_config_file;
use crate::high_risk_recovery::ensure_dry_run_target_path;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum HighRiskRecoveryBucket {
    DisplayRender,
    CursorInput,
    DebugCrash,
}

impl HighRiskRecoveryBucket {
    pub fn as_str(self) -> &'static str {
        match self {
            Self::DisplayRender => "display/render",
            Self::CursorInput => "cursor/input",
            Self::DebugCrash => "debug/crash",
        }
    }

    pub fn recovery_model(self) -> &'static str {
        match self {
            Self::DisplayRender => {
                "display-render-persisted-dead-man-watchdog-with-backup-rollback-and-output-independent-confirmation"
            }
            Self::CursorInput => {
                "cursor-input-persisted-dead-man-watchdog-with-keyboard-token-confirmation-and-pointer-independent-rollback"
            }
            Self::DebugCrash => {
                "debug-crash-persisted-dead-man-watchdog-with-external-process-rollback"
            }
        }
    }

    pub fn must_not_depend_on(self) -> &'static [&'static str] {
        match self {
            Self::DisplayRender => &[
                "visible compositor output",
                "screen remains readable",
                "app UI visibility",
                "Hyprland keybinds inside the affected compositor session",
            ],
            Self::CursorInput => &[
                "pointer visibility",
                "mouse input",
                "app UI",
                "Hyprland keybinds",
                "normal pointer focus or cursor warping behavior",
            ],
            Self::DebugCrash => &[
                "process that may be disrupted",
                "active compositor health",
                "debug/logging subsystem being modified",
                "app UI running inside the affected session",
                "Hyprland keybinds",
            ],
        }
    }
}

impl From<BlockedRowBucket> for HighRiskRecoveryBucket {
    fn from(value: BlockedRowBucket) -> Self {
        match value {
            BlockedRowBucket::DisplayRender => Self::DisplayRender,
            BlockedRowBucket::CursorInput => Self::CursorInput,
            BlockedRowBucket::DebugCrash => Self::DebugCrash,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct HighRiskRecoveryPlanId(pub String);

impl HighRiskRecoveryPlanId {
    pub fn new(value: impl Into<String>) -> Self {
        Self(value.into())
    }

    pub fn as_str(&self) -> &str {
        &self.0
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct HighRiskRecoveryToken(pub String);

impl HighRiskRecoveryToken {
    pub fn new(value: impl Into<String>) -> Self {
        Self(value.into())
    }

    pub fn as_str(&self) -> &str {
        &self.0
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum HighRiskRecoveryAction {
    RestoreBackup,
    KeepProposedValue,
    RefuseLiveTarget,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum HighRiskRecoveryDecision {
    AwaitConfirmation,
    KeepApply,
    Rollback,
    RefuseLiveTarget,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum HighRiskRecoveryPlanStatus {
    Created,
    Armed,
    Confirmed,
    RolledBack,
    Refused,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct HighRiskRecoveryPlan {
    pub plan_id: HighRiskRecoveryPlanId,
    pub row_id: String,
    pub official_setting: String,
    pub bucket: HighRiskRecoveryBucket,
    pub recovery_model: String,
    pub proposed_value: String,
    pub previous_value: Option<String>,
    pub target_config_path: PathBuf,
    pub backup_config_path: PathBuf,
    pub created_unix_seconds: u64,
    pub confirmation_token: HighRiskRecoveryToken,
    pub timeout_seconds: u64,
    pub confirmation_deadline_unix_seconds: u64,
    pub rollback_action: HighRiskRecoveryAction,
    pub status: HighRiskRecoveryPlanStatus,
    pub temp_test_only: bool,
    pub live_execution_enabled: bool,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct HighRiskRecoveryPlanValidation {
    pub valid: bool,
    pub errors: Vec<HighRiskRecoveryPlanError>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum HighRiskRecoveryPlanError {
    MissingPlanId,
    MissingRowId,
    MissingOfficialSettingKey,
    MissingTargetPath,
    MissingBackupPath,
    MissingConfirmationToken,
    TimeoutMustBePositive,
    NonHighRiskRow(String),
    OfficialSettingMismatch {
        row_id: String,
        expected: String,
        actual: String,
    },
    BucketMismatch {
        row_id: String,
        expected: HighRiskRecoveryBucket,
        actual: HighRiskRecoveryBucket,
    },
    RecoveryModelMismatch {
        bucket: HighRiskRecoveryBucket,
        expected: String,
        actual: String,
    },
    TargetPathNotTemp(PathBuf),
    BackupPathNotTemp(PathBuf),
    PlanPathNotTemp(PathBuf),
    WrongConfirmationToken,
    LiveExecutionDisabled,
    Io(String),
    ParserReread(String),
}

impl fmt::Display for HighRiskRecoveryPlanError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::MissingPlanId => write!(formatter, "plan id is required"),
            Self::MissingRowId => write!(formatter, "row id is required"),
            Self::MissingOfficialSettingKey => {
                write!(formatter, "official setting key is required")
            }
            Self::MissingTargetPath => write!(formatter, "target config path is required"),
            Self::MissingBackupPath => write!(formatter, "backup config path is required"),
            Self::MissingConfirmationToken => write!(formatter, "confirmation token is required"),
            Self::TimeoutMustBePositive => write!(formatter, "timeout must be greater than zero"),
            Self::NonHighRiskRow(row_id) => write!(
                formatter,
                "row is not in the blocked high-risk pre-enablement set: {row_id}"
            ),
            Self::OfficialSettingMismatch {
                row_id,
                expected,
                actual,
            } => write!(
                formatter,
                "{row_id} official setting mismatch; expected {expected}, got {actual}"
            ),
            Self::BucketMismatch {
                row_id,
                expected,
                actual,
            } => write!(
                formatter,
                "{row_id} recovery bucket mismatch; expected {}, got {}",
                expected.as_str(),
                actual.as_str()
            ),
            Self::RecoveryModelMismatch {
                bucket,
                expected,
                actual,
            } => write!(
                formatter,
                "{} recovery model mismatch; expected {expected}, got {actual}",
                bucket.as_str()
            ),
            Self::TargetPathNotTemp(path) => {
                write!(
                    formatter,
                    "target path is not a temp/test path: {}",
                    path.display()
                )
            }
            Self::BackupPathNotTemp(path) => {
                write!(
                    formatter,
                    "backup path is not a temp/test path: {}",
                    path.display()
                )
            }
            Self::PlanPathNotTemp(path) => {
                write!(
                    formatter,
                    "plan path is not a temp/test path: {}",
                    path.display()
                )
            }
            Self::WrongConfirmationToken => write!(formatter, "confirmation token did not match"),
            Self::LiveExecutionDisabled => {
                write!(
                    formatter,
                    "live target execution is disabled for this scaffold"
                )
            }
            Self::Io(error) => write!(formatter, "{error}"),
            Self::ParserReread(error) => write!(formatter, "{error}"),
        }
    }
}

impl std::error::Error for HighRiskRecoveryPlanError {}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct HighRiskRecoveryBucketRequirements {
    pub bucket: HighRiskRecoveryBucket,
    pub recovery_model: &'static str,
    pub must_not_depend_on: &'static [&'static str],
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct HighRiskRecoveryRowClassification {
    pub row_id: String,
    pub official_setting: String,
    pub bucket: HighRiskRecoveryBucket,
    pub recovery_model: String,
    pub runtime_dynamic_special_case: bool,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct TempBackupProof {
    pub target_config_path: PathBuf,
    pub backup_config_path: PathBuf,
    pub bytes_copied: usize,
    pub backup_matches_target_before_mutation: bool,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct TempRestoreProof {
    pub target_config_path: PathBuf,
    pub backup_config_path: PathBuf,
    pub restore_written: bool,
    pub parser_reread_succeeded: bool,
    pub restored_value: Option<String>,
}

pub fn recovery_bucket_requirements(
    bucket: HighRiskRecoveryBucket,
) -> HighRiskRecoveryBucketRequirements {
    HighRiskRecoveryBucketRequirements {
        bucket,
        recovery_model: bucket.recovery_model(),
        must_not_depend_on: bucket.must_not_depend_on(),
    }
}

pub fn high_risk_recovery_rows() -> Vec<HighRiskRecoveryRowClassification> {
    blocked_pre_enablement_rows()
        .iter()
        .map(|row| {
            let bucket = HighRiskRecoveryBucket::from(row.bucket);
            HighRiskRecoveryRowClassification {
                row_id: row.row_id.to_string(),
                official_setting: row.official_setting.to_string(),
                bucket,
                recovery_model: bucket.recovery_model().to_string(),
                runtime_dynamic_special_case: row.row_id == "cursor.default_monitor",
            }
        })
        .collect()
}

pub fn deterministic_confirmation_token(
    row_id: &str,
    created_unix_seconds: u64,
) -> HighRiskRecoveryToken {
    HighRiskRecoveryToken::new(format!(
        "confirm-{}-{created_unix_seconds}",
        sanitize_token_part(row_id)
    ))
}

pub fn create_temp_recovery_plan(
    row_id: &str,
    proposed_value: impl Into<String>,
    previous_value: Option<String>,
    target_config_path: impl Into<PathBuf>,
    backup_config_path: impl Into<PathBuf>,
    created_unix_seconds: u64,
    confirmation_token: Option<HighRiskRecoveryToken>,
    timeout_seconds: u64,
) -> Result<HighRiskRecoveryPlan, HighRiskRecoveryPlanError> {
    let row = blocked_pre_enablement_row(row_id)
        .ok_or_else(|| HighRiskRecoveryPlanError::NonHighRiskRow(row_id.to_string()))?;
    let bucket = HighRiskRecoveryBucket::from(row.bucket);
    let token = confirmation_token
        .unwrap_or_else(|| deterministic_confirmation_token(row_id, created_unix_seconds));
    let plan = HighRiskRecoveryPlan {
        plan_id: HighRiskRecoveryPlanId::new(format!(
            "high-risk-recovery-{}-{created_unix_seconds}",
            sanitize_token_part(row_id)
        )),
        row_id: row.row_id.to_string(),
        official_setting: row.official_setting.to_string(),
        bucket,
        recovery_model: bucket.recovery_model().to_string(),
        proposed_value: proposed_value.into(),
        previous_value,
        target_config_path: target_config_path.into(),
        backup_config_path: backup_config_path.into(),
        created_unix_seconds,
        confirmation_token: token,
        timeout_seconds,
        confirmation_deadline_unix_seconds: created_unix_seconds + timeout_seconds,
        rollback_action: HighRiskRecoveryAction::RestoreBackup,
        status: HighRiskRecoveryPlanStatus::Created,
        temp_test_only: true,
        live_execution_enabled: false,
    };
    require_valid_recovery_plan(&plan)?;
    Ok(plan)
}

pub fn validate_recovery_plan(plan: &HighRiskRecoveryPlan) -> HighRiskRecoveryPlanValidation {
    let mut errors = Vec::new();

    if plan.plan_id.as_str().is_empty() {
        errors.push(HighRiskRecoveryPlanError::MissingPlanId);
    }
    if plan.row_id.trim().is_empty() {
        errors.push(HighRiskRecoveryPlanError::MissingRowId);
    }
    if plan.official_setting.trim().is_empty() {
        errors.push(HighRiskRecoveryPlanError::MissingOfficialSettingKey);
    }
    if plan.target_config_path.as_os_str().is_empty() {
        errors.push(HighRiskRecoveryPlanError::MissingTargetPath);
    }
    if plan.backup_config_path.as_os_str().is_empty() {
        errors.push(HighRiskRecoveryPlanError::MissingBackupPath);
    }
    if plan.confirmation_token.as_str().is_empty() {
        errors.push(HighRiskRecoveryPlanError::MissingConfirmationToken);
    }
    if plan.timeout_seconds == 0 {
        errors.push(HighRiskRecoveryPlanError::TimeoutMustBePositive);
    }

    if !plan.row_id.trim().is_empty() {
        match blocked_pre_enablement_row(&plan.row_id) {
            Some(row) => {
                if plan.official_setting != row.official_setting {
                    errors.push(HighRiskRecoveryPlanError::OfficialSettingMismatch {
                        row_id: plan.row_id.clone(),
                        expected: row.official_setting.to_string(),
                        actual: plan.official_setting.clone(),
                    });
                }
                let expected_bucket = HighRiskRecoveryBucket::from(row.bucket);
                if plan.bucket != expected_bucket {
                    errors.push(HighRiskRecoveryPlanError::BucketMismatch {
                        row_id: plan.row_id.clone(),
                        expected: expected_bucket,
                        actual: plan.bucket,
                    });
                }
                let expected_model = expected_bucket.recovery_model();
                if plan.recovery_model != expected_model {
                    errors.push(HighRiskRecoveryPlanError::RecoveryModelMismatch {
                        bucket: expected_bucket,
                        expected: expected_model.to_string(),
                        actual: plan.recovery_model.clone(),
                    });
                }
            }
            None => errors.push(HighRiskRecoveryPlanError::NonHighRiskRow(
                plan.row_id.clone(),
            )),
        }
    }

    if !plan.target_config_path.as_os_str().is_empty()
        && (!plan.temp_test_only || ensure_dry_run_target_path(&plan.target_config_path).is_err())
    {
        errors.push(HighRiskRecoveryPlanError::TargetPathNotTemp(
            plan.target_config_path.clone(),
        ));
    }
    if !plan.backup_config_path.as_os_str().is_empty()
        && (!plan.temp_test_only || ensure_dry_run_target_path(&plan.backup_config_path).is_err())
    {
        errors.push(HighRiskRecoveryPlanError::BackupPathNotTemp(
            plan.backup_config_path.clone(),
        ));
    }

    HighRiskRecoveryPlanValidation {
        valid: errors.is_empty(),
        errors,
    }
}

pub fn require_valid_recovery_plan(
    plan: &HighRiskRecoveryPlan,
) -> Result<(), HighRiskRecoveryPlanError> {
    let validation = validate_recovery_plan(plan);
    if validation.valid {
        Ok(())
    } else {
        Err(validation
            .errors
            .into_iter()
            .next()
            .expect("invalid recovery plan must have an error"))
    }
}

pub fn persist_recovery_plan(
    path: impl AsRef<Path>,
    plan: &HighRiskRecoveryPlan,
) -> Result<(), HighRiskRecoveryPlanError> {
    let path = path.as_ref();
    ensure_dry_run_target_path(path)
        .map_err(|_| HighRiskRecoveryPlanError::PlanPathNotTemp(path.to_path_buf()))?;
    require_valid_recovery_plan(plan)?;
    let bytes = serde_json::to_vec_pretty(plan)
        .map_err(|error| HighRiskRecoveryPlanError::Io(error.to_string()))?;
    atomic_write(path, &bytes)
}

pub fn load_recovery_plan(
    path: impl AsRef<Path>,
) -> Result<HighRiskRecoveryPlan, HighRiskRecoveryPlanError> {
    let path = path.as_ref();
    ensure_dry_run_target_path(path)
        .map_err(|_| HighRiskRecoveryPlanError::PlanPathNotTemp(path.to_path_buf()))?;
    let bytes = fs::read(path).map_err(|error| HighRiskRecoveryPlanError::Io(error.to_string()))?;
    let plan = serde_json::from_slice(&bytes)
        .map_err(|error| HighRiskRecoveryPlanError::Io(error.to_string()))?;
    require_valid_recovery_plan(&plan)?;
    Ok(plan)
}

pub fn create_temp_config_backup(
    plan: &HighRiskRecoveryPlan,
) -> Result<TempBackupProof, HighRiskRecoveryPlanError> {
    require_valid_recovery_plan(plan)?;
    let target_bytes = fs::read(&plan.target_config_path)
        .map_err(|error| HighRiskRecoveryPlanError::Io(error.to_string()))?;
    if let Some(parent) = plan.backup_config_path.parent() {
        fs::create_dir_all(parent)
            .map_err(|error| HighRiskRecoveryPlanError::Io(error.to_string()))?;
    }
    let mut backup_file = OpenOptions::new()
        .write(true)
        .create_new(true)
        .mode(0o600)
        .open(&plan.backup_config_path)
        .map_err(|error| HighRiskRecoveryPlanError::Io(error.to_string()))?;
    backup_file
        .write_all(&target_bytes)
        .and_then(|_| backup_file.sync_all())
        .map_err(|error| HighRiskRecoveryPlanError::Io(error.to_string()))?;
    let backup_bytes = fs::read(&plan.backup_config_path)
        .map_err(|error| HighRiskRecoveryPlanError::Io(error.to_string()))?;
    Ok(TempBackupProof {
        target_config_path: plan.target_config_path.clone(),
        backup_config_path: plan.backup_config_path.clone(),
        bytes_copied: target_bytes.len(),
        backup_matches_target_before_mutation: backup_bytes == target_bytes,
    })
}

pub fn restore_temp_config_from_backup(
    plan: &HighRiskRecoveryPlan,
) -> Result<TempRestoreProof, HighRiskRecoveryPlanError> {
    require_valid_recovery_plan(plan)?;
    let backup_manager = BackupManager::new(
        plan.backup_config_path
            .parent()
            .unwrap_or_else(|| Path::new(".")),
    );
    let backup_bytes = fs::read(&plan.backup_config_path)
        .map_err(|error| HighRiskRecoveryPlanError::Io(error.to_string()))?;
    let backup = backup_manager
        .load_existing_for_restore(
            &plan.target_config_path,
            &plan.backup_config_path,
            &backup_bytes,
        )
        .map_err(|error| HighRiskRecoveryPlanError::Io(error.to_string()))?;
    let expected_current_bytes = fs::read(&plan.target_config_path)
        .map_err(|error| HighRiskRecoveryPlanError::Io(error.to_string()))?;
    backup_manager
        .rollback(&backup, &expected_current_bytes)
        .map_err(|error| HighRiskRecoveryPlanError::Io(error.to_string()))?;

    let parsed = parse_hyprland_config_file(&plan.target_config_path)
        .map_err(|error| HighRiskRecoveryPlanError::ParserReread(error.to_string()))?;
    let restored_value = parsed
        .scalar_records()
        .filter(|record| record.normalized_setting_id.as_deref() == Some(&plan.official_setting))
        .last()
        .and_then(|record| record.raw_value.clone());

    Ok(TempRestoreProof {
        target_config_path: plan.target_config_path.clone(),
        backup_config_path: plan.backup_config_path.clone(),
        restore_written: true,
        parser_reread_succeeded: true,
        restored_value,
    })
}

pub fn accept_confirmation_token(
    plan: &HighRiskRecoveryPlan,
    token: &str,
) -> Result<HighRiskRecoveryDecision, HighRiskRecoveryPlanError> {
    require_valid_recovery_plan(plan)?;
    if plan.confirmation_token.as_str() == token {
        Ok(HighRiskRecoveryDecision::KeepApply)
    } else {
        Err(HighRiskRecoveryPlanError::WrongConfirmationToken)
    }
}

pub fn decide_recovery_action(
    plan: &HighRiskRecoveryPlan,
    now_unix_seconds: u64,
    confirmation_token: Option<&str>,
) -> Result<HighRiskRecoveryDecision, HighRiskRecoveryPlanError> {
    require_valid_recovery_plan(plan)?;
    if let Some(token) = confirmation_token {
        return accept_confirmation_token(plan, token);
    }
    if now_unix_seconds >= plan.confirmation_deadline_unix_seconds {
        Ok(HighRiskRecoveryDecision::Rollback)
    } else {
        Ok(HighRiskRecoveryDecision::AwaitConfirmation)
    }
}

pub fn refuse_live_target_execution(
    plan: &HighRiskRecoveryPlan,
) -> Result<HighRiskRecoveryDecision, HighRiskRecoveryPlanError> {
    require_valid_recovery_plan(plan)?;
    if plan.live_execution_enabled {
        Ok(HighRiskRecoveryDecision::RefuseLiveTarget)
    } else {
        Err(HighRiskRecoveryPlanError::LiveExecutionDisabled)
    }
}

fn sanitize_token_part(value: &str) -> String {
    value
        .chars()
        .map(|character| {
            if character.is_ascii_alphanumeric() {
                character
            } else {
                '-'
            }
        })
        .collect()
}

fn atomic_write(target: &Path, bytes: &[u8]) -> Result<(), HighRiskRecoveryPlanError> {
    let parent = target
        .parent()
        .ok_or_else(|| HighRiskRecoveryPlanError::Io("target path has no parent".to_string()))?;
    fs::create_dir_all(parent).map_err(|error| HighRiskRecoveryPlanError::Io(error.to_string()))?;
    let temp_path = parent.join(format!(
        ".{}.persisted-recovery.tmp",
        target
            .file_name()
            .and_then(|name| name.to_str())
            .unwrap_or("hyprland.conf")
    ));
    {
        let mut file = OpenOptions::new()
            .write(true)
            .create_new(true)
            .open(&temp_path)
            .map_err(|error| HighRiskRecoveryPlanError::Io(error.to_string()))?;
        file.write_all(bytes)
            .map_err(|error| HighRiskRecoveryPlanError::Io(error.to_string()))?;
        file.sync_all()
            .map_err(|error| HighRiskRecoveryPlanError::Io(error.to_string()))?;
    }
    fs::rename(&temp_path, target).map_err(|error| HighRiskRecoveryPlanError::Io(error.to_string()))
}
