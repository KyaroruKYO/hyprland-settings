use std::fmt;
use std::path::PathBuf;

use crate::blocked_row_pre_enablement::{blocked_pre_enablement_row, blocked_pre_enablement_rows};
use crate::high_risk_persisted_recovery::{
    accept_confirmation_token, require_valid_recovery_plan, HighRiskRecoveryBucket,
    HighRiskRecoveryDecision, HighRiskRecoveryPlan, HighRiskRecoveryPlanError, TempBackupProof,
    TempRestoreProof,
};
use crate::write_classification::{is_high_risk_gated_writable_setting, is_safe_writable_setting};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum HighRiskProductionGateMode {
    ReportOnlyDryRun,
    ProductionWrite,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct HighRiskProductionGateRequest {
    pub mode: HighRiskProductionGateMode,
    pub row_id: String,
    pub official_setting: String,
    pub bucket: HighRiskRecoveryBucket,
    pub requested_keep_apply: bool,
    pub now_unix_seconds: u64,
    pub proof: Option<HighRiskProductionGateProof>,
    pub runtime_oracle_proven: bool,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct HighRiskProductionGateProof {
    pub recovery_plan: HighRiskRecoveryPlan,
    pub backup_proof: Option<TempBackupProof>,
    pub rollback_proof: Option<TempRestoreProof>,
    pub confirmation_token: Option<String>,
    pub explicit_high_risk_approval: bool,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct HighRiskProductionGateDecision {
    pub kind: HighRiskProductionGateDecisionKind,
    pub errors: Vec<HighRiskProductionGateError>,
}

impl HighRiskProductionGateDecision {
    pub fn accepted(&self) -> bool {
        matches!(
            self.kind,
            HighRiskProductionGateDecisionKind::ReportOnlyDryRunAccepted
                | HighRiskProductionGateDecisionKind::ProductionWriteAccepted
        ) && self.errors.is_empty()
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum HighRiskProductionGateDecisionKind {
    ReportOnlyDryRunAccepted,
    ReportOnlyDryRunRejected,
    ProductionWriteAccepted,
    ProductionWriteRefused,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct HighRiskProductionGateEvaluation {
    pub row_id: String,
    pub bucket: HighRiskRecoveryBucket,
    pub mode: HighRiskProductionGateMode,
    pub decision: HighRiskProductionGateDecision,
    pub runtime_dynamic_special_case: bool,
    pub is_safe_writable_setting: bool,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum HighRiskProductionGateError {
    MissingRecoveryPlan,
    MissingBackupProof,
    MissingRollbackProof,
    MissingConfirmationProof,
    WrongConfirmationToken,
    TimeoutNoConfirmationForKeepApply,
    NonHighRiskRow(String),
    RowMismatch {
        expected: String,
        actual: String,
    },
    OfficialSettingMismatch {
        expected: String,
        actual: String,
    },
    BucketMismatch {
        expected: HighRiskRecoveryBucket,
        actual: HighRiskRecoveryBucket,
    },
    RecoveryPlanInvalid(String),
    BackupTargetMismatch {
        expected: PathBuf,
        actual: PathBuf,
    },
    BackupPathMismatch {
        expected: PathBuf,
        actual: PathBuf,
    },
    RollbackTargetMismatch {
        expected: PathBuf,
        actual: PathBuf,
    },
    RollbackPathMismatch {
        expected: PathBuf,
        actual: PathBuf,
    },
    RollbackParserRereadMissing,
    LiveExecutionEnabled,
    RuntimeDynamicOracleMissing,
    MissingExplicitHighRiskApproval,
    ProductionWriteDisabled,
}

impl fmt::Display for HighRiskProductionGateError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::MissingRecoveryPlan => write!(formatter, "missing recovery plan"),
            Self::MissingBackupProof => write!(formatter, "missing backup proof"),
            Self::MissingRollbackProof => write!(formatter, "missing rollback proof"),
            Self::MissingConfirmationProof => write!(formatter, "missing confirmation proof"),
            Self::WrongConfirmationToken => write!(formatter, "wrong confirmation token"),
            Self::TimeoutNoConfirmationForKeepApply => {
                write!(formatter, "timeout/no confirmation cannot keep/apply")
            }
            Self::NonHighRiskRow(row_id) => write!(formatter, "non-high-risk row: {row_id}"),
            Self::RowMismatch { expected, actual } => {
                write!(formatter, "row mismatch; expected {expected}, got {actual}")
            }
            Self::OfficialSettingMismatch { expected, actual } => write!(
                formatter,
                "official setting mismatch; expected {expected}, got {actual}"
            ),
            Self::BucketMismatch { expected, actual } => write!(
                formatter,
                "bucket mismatch; expected {}, got {}",
                expected.as_str(),
                actual.as_str()
            ),
            Self::RecoveryPlanInvalid(error) => write!(formatter, "recovery plan invalid: {error}"),
            Self::BackupTargetMismatch { expected, actual } => write!(
                formatter,
                "backup target mismatch; expected {}, got {}",
                expected.display(),
                actual.display()
            ),
            Self::BackupPathMismatch { expected, actual } => write!(
                formatter,
                "backup path mismatch; expected {}, got {}",
                expected.display(),
                actual.display()
            ),
            Self::RollbackTargetMismatch { expected, actual } => write!(
                formatter,
                "rollback target mismatch; expected {}, got {}",
                expected.display(),
                actual.display()
            ),
            Self::RollbackPathMismatch { expected, actual } => write!(
                formatter,
                "rollback path mismatch; expected {}, got {}",
                expected.display(),
                actual.display()
            ),
            Self::RollbackParserRereadMissing => {
                write!(formatter, "rollback proof did not include parser reread")
            }
            Self::LiveExecutionEnabled => write!(formatter, "live execution is enabled"),
            Self::RuntimeDynamicOracleMissing => {
                write!(formatter, "runtime-dynamic oracle proof is missing")
            }
            Self::MissingExplicitHighRiskApproval => {
                write!(formatter, "explicit high-risk approval is missing")
            }
            Self::ProductionWriteDisabled => write!(
                formatter,
                "production write mode is disabled for this high-risk request"
            ),
        }
    }
}

impl std::error::Error for HighRiskProductionGateError {}

pub fn high_risk_production_gate_rows() -> Vec<HighRiskProductionGateEvaluation> {
    blocked_pre_enablement_rows()
        .iter()
        .map(|row| {
            let bucket = HighRiskRecoveryBucket::from(row.bucket);
            let request = HighRiskProductionGateRequest {
                mode: HighRiskProductionGateMode::ProductionWrite,
                row_id: row.row_id.to_string(),
                official_setting: row.official_setting.to_string(),
                bucket,
                requested_keep_apply: true,
                now_unix_seconds: 0,
                proof: None,
                runtime_oracle_proven: false,
            };
            evaluate_high_risk_production_gate(request)
        })
        .collect()
}

pub fn evaluate_high_risk_production_gate(
    request: HighRiskProductionGateRequest,
) -> HighRiskProductionGateEvaluation {
    let expected = blocked_pre_enablement_row(&request.row_id);
    let runtime_dynamic_special_case = request.row_id == "cursor.default_monitor";
    let mut errors = Vec::new();

    if expected.is_none() {
        errors.push(HighRiskProductionGateError::NonHighRiskRow(
            request.row_id.clone(),
        ));
    }

    if let Some(row) = expected {
        let expected_bucket = HighRiskRecoveryBucket::from(row.bucket);
        if request.official_setting != row.official_setting {
            errors.push(HighRiskProductionGateError::OfficialSettingMismatch {
                expected: row.official_setting.to_string(),
                actual: request.official_setting.clone(),
            });
        }
        if request.bucket != expected_bucket {
            errors.push(HighRiskProductionGateError::BucketMismatch {
                expected: expected_bucket,
                actual: request.bucket,
            });
        }
    }

    if request.mode == HighRiskProductionGateMode::ProductionWrite
        && !is_high_risk_gated_writable_setting(&request.row_id)
    {
        errors.push(HighRiskProductionGateError::ProductionWriteDisabled);
        return HighRiskProductionGateEvaluation {
            row_id: request.row_id.clone(),
            bucket: request.bucket,
            mode: request.mode,
            decision: HighRiskProductionGateDecision {
                kind: HighRiskProductionGateDecisionKind::ProductionWriteRefused,
                errors,
            },
            runtime_dynamic_special_case,
            is_safe_writable_setting: is_safe_writable_setting(&request.row_id),
        };
    }

    match &request.proof {
        Some(proof) => {
            validate_gate_proof(&request, proof, &mut errors);
            if request.mode == HighRiskProductionGateMode::ProductionWrite
                && !proof.explicit_high_risk_approval
            {
                errors.push(HighRiskProductionGateError::MissingExplicitHighRiskApproval);
                errors.push(HighRiskProductionGateError::ProductionWriteDisabled);
            }
        }
        None => errors.push(HighRiskProductionGateError::MissingRecoveryPlan),
    }

    let kind = match (request.mode, errors.is_empty()) {
        (HighRiskProductionGateMode::ReportOnlyDryRun, true) => {
            HighRiskProductionGateDecisionKind::ReportOnlyDryRunAccepted
        }
        (HighRiskProductionGateMode::ReportOnlyDryRun, false) => {
            HighRiskProductionGateDecisionKind::ReportOnlyDryRunRejected
        }
        (HighRiskProductionGateMode::ProductionWrite, true) => {
            HighRiskProductionGateDecisionKind::ProductionWriteAccepted
        }
        (HighRiskProductionGateMode::ProductionWrite, false) => {
            HighRiskProductionGateDecisionKind::ProductionWriteRefused
        }
    };

    HighRiskProductionGateEvaluation {
        row_id: request.row_id.clone(),
        bucket: request.bucket,
        mode: request.mode,
        decision: HighRiskProductionGateDecision { kind, errors },
        runtime_dynamic_special_case,
        is_safe_writable_setting: is_safe_writable_setting(&request.row_id),
    }
}

fn validate_gate_proof(
    request: &HighRiskProductionGateRequest,
    proof: &HighRiskProductionGateProof,
    errors: &mut Vec<HighRiskProductionGateError>,
) {
    let plan = &proof.recovery_plan;
    if let Err(error) = require_valid_recovery_plan(plan) {
        errors.push(HighRiskProductionGateError::RecoveryPlanInvalid(
            recovery_plan_error_label(error),
        ));
    }
    if plan.row_id != request.row_id {
        errors.push(HighRiskProductionGateError::RowMismatch {
            expected: request.row_id.clone(),
            actual: plan.row_id.clone(),
        });
    }
    if plan.official_setting != request.official_setting {
        errors.push(HighRiskProductionGateError::OfficialSettingMismatch {
            expected: request.official_setting.clone(),
            actual: plan.official_setting.clone(),
        });
    }
    if plan.bucket != request.bucket {
        errors.push(HighRiskProductionGateError::BucketMismatch {
            expected: request.bucket,
            actual: plan.bucket,
        });
    }
    if plan.live_execution_enabled {
        errors.push(HighRiskProductionGateError::LiveExecutionEnabled);
    }

    match &proof.backup_proof {
        Some(backup) => {
            if backup.target_config_path != plan.target_config_path {
                errors.push(HighRiskProductionGateError::BackupTargetMismatch {
                    expected: plan.target_config_path.clone(),
                    actual: backup.target_config_path.clone(),
                });
            }
            if backup.backup_config_path != plan.backup_config_path {
                errors.push(HighRiskProductionGateError::BackupPathMismatch {
                    expected: plan.backup_config_path.clone(),
                    actual: backup.backup_config_path.clone(),
                });
            }
            if !backup.backup_matches_target_before_mutation {
                errors.push(HighRiskProductionGateError::MissingBackupProof);
            }
        }
        None => errors.push(HighRiskProductionGateError::MissingBackupProof),
    }

    match &proof.rollback_proof {
        Some(rollback) => {
            if rollback.target_config_path != plan.target_config_path {
                errors.push(HighRiskProductionGateError::RollbackTargetMismatch {
                    expected: plan.target_config_path.clone(),
                    actual: rollback.target_config_path.clone(),
                });
            }
            if rollback.backup_config_path != plan.backup_config_path {
                errors.push(HighRiskProductionGateError::RollbackPathMismatch {
                    expected: plan.backup_config_path.clone(),
                    actual: rollback.backup_config_path.clone(),
                });
            }
            if !rollback.restore_written || !rollback.parser_reread_succeeded {
                errors.push(HighRiskProductionGateError::MissingRollbackProof);
            }
            if rollback.restored_value.is_none() {
                errors.push(HighRiskProductionGateError::RollbackParserRereadMissing);
            }
        }
        None => errors.push(HighRiskProductionGateError::MissingRollbackProof),
    }

    if request.requested_keep_apply {
        match proof.confirmation_token.as_deref() {
            Some(token) => {
                if accept_confirmation_token(plan, token) != Ok(HighRiskRecoveryDecision::KeepApply)
                {
                    errors.push(HighRiskProductionGateError::WrongConfirmationToken);
                }
            }
            None if request.now_unix_seconds >= plan.confirmation_deadline_unix_seconds => {
                errors.push(HighRiskProductionGateError::TimeoutNoConfirmationForKeepApply);
            }
            None => errors.push(HighRiskProductionGateError::MissingConfirmationProof),
        }
    }

    if request.row_id == "cursor.default_monitor" && !request.runtime_oracle_proven {
        errors.push(HighRiskProductionGateError::RuntimeDynamicOracleMissing);
    }
}

fn recovery_plan_error_label(error: HighRiskRecoveryPlanError) -> String {
    match error {
        HighRiskRecoveryPlanError::TargetPathNotTemp(_) => "non-temp-target-path".to_string(),
        HighRiskRecoveryPlanError::BackupPathNotTemp(_) => "non-temp-backup-path".to_string(),
        HighRiskRecoveryPlanError::PlanPathNotTemp(_) => "non-temp-plan-path".to_string(),
        HighRiskRecoveryPlanError::LiveExecutionDisabled => "live-execution-disabled".to_string(),
        other => other.to_string(),
    }
}
