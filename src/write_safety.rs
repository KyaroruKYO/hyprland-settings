use std::collections::BTreeSet;
use std::path::{Path, PathBuf};

use crate::config_backup::ConfigBackup;
use crate::current_config::{CurrentValueProjection, CurrentValueSourceStatus};
use crate::high_risk_recovery::{validate_watchdog_plan, HighRiskWatchdogPlan};
use crate::pending_change::{PendingChange, PendingChangeValidation};
use crate::write_classification::is_safe_writable_setting;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct WritePlanRequest {
    pub known_setting_ids: BTreeSet<String>,
    pub detected_config_path: PathBuf,
    pub current_value: CurrentValueProjection,
    pub pending_change: PendingChange,
    pub backup: Option<ConfigBackup>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct WriteReview {
    pub plan: Option<WritePlan>,
    pub failures: Vec<WriteGateFailure>,
}

impl WriteReview {
    pub fn is_approved(&self) -> bool {
        self.plan.is_some() && self.failures.is_empty()
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct WritePlan {
    pub setting_id: String,
    pub target_path: PathBuf,
    pub action: WritePlanAction,
    pub old_value: Option<String>,
    pub proposed_value: String,
    pub backup_path: PathBuf,
    pub rollback: RollbackPlan,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum WritePlanAction {
    ReplaceLine { line_number: usize },
    AppendSetting,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RollbackPlan {
    pub source_path: PathBuf,
    pub backup_path: PathBuf,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct WriteResult {
    pub plan: WritePlan,
    pub verified_value: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum WriteGateFailure {
    UnknownSetting,
    NotAllowlisted,
    InvalidProposedValue(String),
    MissingCurrentSource,
    DuplicateConflict,
    MissingBackup,
    BackupTargetMismatch,
    TargetMismatch,
    StructuredFamilyRejected,
}

pub const SCREEN_SHADER_PRODUCTION_GATE_PRIMITIVE_NAME: &str =
    "screen-shader-dry-run-gated-write-review";
pub const SCREEN_SHADER_GATED_SETTING_ID: &str = "decoration.screen_shader";

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct HighRiskGateProof {
    pub setting_id: String,
    pub recovery_bucket: String,
    pub watchdog_plan: HighRiskWatchdogPlan,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct GatedWriteReview {
    pub base_review: WriteReview,
    pub gate_required: bool,
    pub gate_proof_accepted: bool,
    pub failures: Vec<GatedWriteFailure>,
}

impl GatedWriteReview {
    pub fn is_approved(&self) -> bool {
        self.base_review.is_approved() && self.failures.is_empty()
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum GatedWriteFailure {
    BaseWriteRejected,
    MissingScreenShaderGateProof,
    GateProofForWrongSetting,
    GateProofForWrongRecoveryBucket,
    GateProofTargetMismatch,
    InvalidWatchdogPlan(String),
}

pub fn review_write_plan(request: WritePlanRequest) -> WriteReview {
    let mut failures = Vec::new();
    let setting_id = request.pending_change.setting_id.as_str();

    if !request.known_setting_ids.contains(setting_id) {
        failures.push(WriteGateFailure::UnknownSetting);
    }
    if !is_safe_writable_setting(setting_id) {
        failures.push(WriteGateFailure::NotAllowlisted);
    }
    if setting_id.starts_with("hl.") {
        failures.push(WriteGateFailure::StructuredFamilyRejected);
    }
    match &request.pending_change.validation {
        PendingChangeValidation::Valid => {}
        PendingChangeValidation::Invalid { reason }
        | PendingChangeValidation::NotAllowed { reason } => {
            failures.push(WriteGateFailure::InvalidProposedValue(reason.clone()));
        }
    }
    let action = match request.current_value.status {
        CurrentValueSourceStatus::Configured => {
            let Some(source_path) = request.current_value.source_path.clone() else {
                failures.push(WriteGateFailure::MissingCurrentSource);
                return WriteReview {
                    plan: None,
                    failures,
                };
            };
            let Some(line_number) = request.current_value.line_number else {
                failures.push(WriteGateFailure::MissingCurrentSource);
                return WriteReview {
                    plan: None,
                    failures,
                };
            };
            if source_path != request.detected_config_path {
                failures.push(WriteGateFailure::TargetMismatch);
            }
            Some(WritePlanAction::ReplaceLine { line_number })
        }
        CurrentValueSourceStatus::NotConfigured => Some(WritePlanAction::AppendSetting),
        CurrentValueSourceStatus::DuplicateConflict => {
            failures.push(WriteGateFailure::DuplicateConflict);
            None
        }
        CurrentValueSourceStatus::ReadUnavailable => {
            failures.push(WriteGateFailure::MissingCurrentSource);
            None
        }
    };
    let Some(backup) = request.backup else {
        failures.push(WriteGateFailure::MissingBackup);
        return WriteReview {
            plan: None,
            failures,
        };
    };
    if backup.source_path != request.detected_config_path {
        failures.push(WriteGateFailure::BackupTargetMismatch);
    }

    if !failures.is_empty() {
        return WriteReview {
            plan: None,
            failures,
        };
    }
    let action = action.expect("valid write review should have action");

    let plan = WritePlan {
        setting_id: setting_id.to_string(),
        target_path: request.detected_config_path.clone(),
        action,
        old_value: request.current_value.raw_value.clone(),
        proposed_value: request.pending_change.proposed_value,
        backup_path: backup.backup_path.clone(),
        rollback: RollbackPlan {
            source_path: request.detected_config_path,
            backup_path: backup.backup_path,
        },
    };

    WriteReview {
        plan: Some(plan),
        failures,
    }
}

pub fn screen_shader_requires_high_risk_gate(setting_id: &str) -> bool {
    setting_id == SCREEN_SHADER_GATED_SETTING_ID
}

pub fn review_screen_shader_gated_write_plan(
    base_review: WriteReview,
    gate_proof: Option<HighRiskGateProof>,
) -> GatedWriteReview {
    let gate_required = base_review
        .plan
        .as_ref()
        .is_some_and(|plan| screen_shader_requires_high_risk_gate(&plan.setting_id));
    let mut failures = Vec::new();
    let mut gate_proof_accepted = false;

    if !base_review.is_approved() {
        failures.push(GatedWriteFailure::BaseWriteRejected);
    }

    if gate_required {
        match (&base_review.plan, gate_proof) {
            (Some(plan), Some(proof)) => match validate_screen_shader_gate_proof(plan, &proof) {
                Ok(()) => gate_proof_accepted = true,
                Err(failure) => failures.push(failure),
            },
            (Some(_), None) => failures.push(GatedWriteFailure::MissingScreenShaderGateProof),
            (None, _) => {}
        }
    }

    GatedWriteReview {
        base_review,
        gate_required,
        gate_proof_accepted,
        failures,
    }
}

fn validate_screen_shader_gate_proof(
    plan: &WritePlan,
    proof: &HighRiskGateProof,
) -> Result<(), GatedWriteFailure> {
    if proof.setting_id != SCREEN_SHADER_GATED_SETTING_ID {
        return Err(GatedWriteFailure::GateProofForWrongSetting);
    }
    if proof.recovery_bucket != "display-render-recovery:screen-shader-gate-migration-design" {
        return Err(GatedWriteFailure::GateProofForWrongRecoveryBucket);
    }
    if proof.watchdog_plan.recovery.target_config_path != plan.target_path {
        return Err(GatedWriteFailure::GateProofTargetMismatch);
    }
    validate_watchdog_plan(&proof.watchdog_plan)
        .map_err(|error| GatedWriteFailure::InvalidWatchdogPlan(error.to_string()))?;
    require_existing_fixture_file(&proof.watchdog_plan.plan_path)?;
    require_existing_fixture_file(&proof.watchdog_plan.recovery.backup_path)?;
    Ok(())
}

fn require_existing_fixture_file(path: &Path) -> Result<(), GatedWriteFailure> {
    if path.exists() {
        Ok(())
    } else {
        Err(GatedWriteFailure::InvalidWatchdogPlan(format!(
            "required fixture watchdog artifact does not exist: {}",
            path.display()
        )))
    }
}
