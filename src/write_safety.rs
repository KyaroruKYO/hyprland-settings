use std::collections::BTreeSet;
use std::path::PathBuf;

use crate::config_backup::ConfigBackup;
use crate::current_config::{CurrentValueProjection, CurrentValueSourceStatus};
use crate::pending_change::{
    PendingChange, PendingChangeValidation, ACTIVE_PENDING_CHANGE_SETTING,
};

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
    pub source_line_number: usize,
    pub old_value: String,
    pub proposed_value: String,
    pub backup_path: PathBuf,
    pub rollback: RollbackPlan,
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

pub fn review_write_plan(request: WritePlanRequest) -> WriteReview {
    let mut failures = Vec::new();
    let setting_id = request.pending_change.setting_id.as_str();

    if !request.known_setting_ids.contains(setting_id) {
        failures.push(WriteGateFailure::UnknownSetting);
    }
    if setting_id != ACTIVE_PENDING_CHANGE_SETTING {
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
    if request.current_value.status == CurrentValueSourceStatus::DuplicateConflict {
        failures.push(WriteGateFailure::DuplicateConflict);
    }

    let Some(source_path) = request.current_value.source_path.clone() else {
        failures.push(WriteGateFailure::MissingCurrentSource);
        return WriteReview {
            plan: None,
            failures,
        };
    };
    let Some(source_line_number) = request.current_value.line_number else {
        failures.push(WriteGateFailure::MissingCurrentSource);
        return WriteReview {
            plan: None,
            failures,
        };
    };
    if source_path != request.detected_config_path {
        failures.push(WriteGateFailure::TargetMismatch);
    }

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

    let plan = WritePlan {
        setting_id: setting_id.to_string(),
        target_path: request.detected_config_path.clone(),
        source_line_number,
        old_value: request.current_value.raw_value.clone().unwrap_or_default(),
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
