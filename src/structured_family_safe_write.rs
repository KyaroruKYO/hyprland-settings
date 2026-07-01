#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum StructuredFamilySafeWriteScaffoldStatus {
    Implemented,
    Inert,
    Unwired,
    RejectedByDefault,
    NotApproved,
}

impl StructuredFamilySafeWriteScaffoldStatus {
    pub fn as_str(self) -> &'static str {
        match self {
            Self::Implemented => "StructuredFamilySafeWriteScaffoldImplemented",
            Self::Inert => "StructuredFamilySafeWriteScaffoldInert",
            Self::Unwired => "StructuredFamilySafeWriteScaffoldUnwired",
            Self::RejectedByDefault => "StructuredFamilySafeWriteScaffoldRejectedByDefault",
            Self::NotApproved => "StructuredFamilySafeWriteScaffoldNotApproved",
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum StructuredFamilySafeWriteRejectionReason {
    ExecutorWiringNotApproved,
    RealWriteScopeNotApproved,
    RealConfigTargetNotApproved,
    BackupExecutionNotApproved,
    RestoreExecutionNotApproved,
    RollbackExecutionNotApproved,
    HyprlandReloadNotApproved,
    RuntimeMutationNotApproved,
    FirstRealConfigWriteNotApproved,
    GuiRealWriteControlsNotApproved,
    UnsupportedOrNotProvenRecord,
    BlockedPlan,
    ActivationSubsetNotSelected,
    ProductionReadinessNotApproved,
}

impl StructuredFamilySafeWriteRejectionReason {
    pub fn as_str(self) -> &'static str {
        match self {
            Self::ExecutorWiringNotApproved => "ExecutorWiringNotApproved",
            Self::RealWriteScopeNotApproved => "RealWriteScopeNotApproved",
            Self::RealConfigTargetNotApproved => "RealConfigTargetNotApproved",
            Self::BackupExecutionNotApproved => "BackupExecutionNotApproved",
            Self::RestoreExecutionNotApproved => "RestoreExecutionNotApproved",
            Self::RollbackExecutionNotApproved => "RollbackExecutionNotApproved",
            Self::HyprlandReloadNotApproved => "HyprlandReloadNotApproved",
            Self::RuntimeMutationNotApproved => "RuntimeMutationNotApproved",
            Self::FirstRealConfigWriteNotApproved => "FirstRealConfigWriteNotApproved",
            Self::GuiRealWriteControlsNotApproved => "GuiRealWriteControlsNotApproved",
            Self::UnsupportedOrNotProvenRecord => "UnsupportedOrNotProvenRecord",
            Self::BlockedPlan => "BlockedPlan",
            Self::ActivationSubsetNotSelected => "ActivationSubsetNotSelected",
            Self::ProductionReadinessNotApproved => "ProductionReadinessNotApproved",
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct StructuredFamilySafeWritePlan {
    pub plan_id: &'static str,
    pub scaffold_status: StructuredFamilySafeWriteScaffoldStatus,
    pub review_only: bool,
    pub executable: bool,
    pub default_rejection_reasons: Vec<StructuredFamilySafeWriteRejectionReason>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct StructuredFamilySafeWritePreflight {
    pub status: StructuredFamilySafeWriteScaffoldStatus,
    pub passed: bool,
    pub rejection_reasons: Vec<StructuredFamilySafeWriteRejectionReason>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct StructuredFamilySafeWriteTargetPolicy {
    pub status: StructuredFamilySafeWriteScaffoldStatus,
    pub real_config_target_enabled: bool,
    pub rejection_reasons: Vec<StructuredFamilySafeWriteRejectionReason>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct StructuredFamilySafeWriteBackupPlan {
    pub status: StructuredFamilySafeWriteScaffoldStatus,
    pub backup_creation_enabled: bool,
    pub restore_execution_enabled: bool,
    pub rejection_reasons: Vec<StructuredFamilySafeWriteRejectionReason>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct StructuredFamilySafeWriteRollbackPlan {
    pub status: StructuredFamilySafeWriteScaffoldStatus,
    pub rollback_execution_enabled: bool,
    pub rejection_reasons: Vec<StructuredFamilySafeWriteRejectionReason>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct StructuredFamilySafeWriteApprovalState {
    pub status: StructuredFamilySafeWriteScaffoldStatus,
    pub executor_wiring_approved: bool,
    pub real_write_scope_approved: bool,
    pub first_real_config_write_approved: bool,
    pub gui_real_write_controls_approved: bool,
    pub rejection_reasons: Vec<StructuredFamilySafeWriteRejectionReason>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct StructuredFamilySafeWriteExecutionReceipt {
    pub status: StructuredFamilySafeWriteScaffoldStatus,
    pub executed: bool,
    pub real_config_touched: bool,
    pub backup_created: bool,
    pub restore_executed: bool,
    pub rollback_executed: bool,
    pub reload_run: bool,
    pub runtime_mutated: bool,
    pub rejection_reasons: Vec<StructuredFamilySafeWriteRejectionReason>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct StructuredFamilySafeWriteAuditRecord {
    pub status: StructuredFamilySafeWriteScaffoldStatus,
    pub summary: &'static str,
    pub execution_recorded: bool,
    pub rejection_reasons: Vec<StructuredFamilySafeWriteRejectionReason>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct StructuredFamilySafeWriteEmergencyStop {
    pub status: StructuredFamilySafeWriteScaffoldStatus,
    pub reason: StructuredFamilySafeWriteRejectionReason,
}

pub fn structured_family_safe_write_default_rejection_reasons(
) -> Vec<StructuredFamilySafeWriteRejectionReason> {
    vec![
        StructuredFamilySafeWriteRejectionReason::ExecutorWiringNotApproved,
        StructuredFamilySafeWriteRejectionReason::RealWriteScopeNotApproved,
        StructuredFamilySafeWriteRejectionReason::RealConfigTargetNotApproved,
        StructuredFamilySafeWriteRejectionReason::BackupExecutionNotApproved,
        StructuredFamilySafeWriteRejectionReason::RestoreExecutionNotApproved,
        StructuredFamilySafeWriteRejectionReason::RollbackExecutionNotApproved,
        StructuredFamilySafeWriteRejectionReason::HyprlandReloadNotApproved,
        StructuredFamilySafeWriteRejectionReason::RuntimeMutationNotApproved,
        StructuredFamilySafeWriteRejectionReason::FirstRealConfigWriteNotApproved,
        StructuredFamilySafeWriteRejectionReason::GuiRealWriteControlsNotApproved,
        StructuredFamilySafeWriteRejectionReason::UnsupportedOrNotProvenRecord,
        StructuredFamilySafeWriteRejectionReason::BlockedPlan,
        StructuredFamilySafeWriteRejectionReason::ActivationSubsetNotSelected,
        StructuredFamilySafeWriteRejectionReason::ProductionReadinessNotApproved,
    ]
}

pub fn build_safe_write_plan_scaffold() -> StructuredFamilySafeWritePlan {
    StructuredFamilySafeWritePlan {
        plan_id: "structured-family-safe-write-scaffold",
        scaffold_status: StructuredFamilySafeWriteScaffoldStatus::Inert,
        review_only: true,
        executable: false,
        default_rejection_reasons: structured_family_safe_write_default_rejection_reasons(),
    }
}

pub fn validate_safe_write_preflight_scaffold(
    _plan: &StructuredFamilySafeWritePlan,
) -> StructuredFamilySafeWritePreflight {
    StructuredFamilySafeWritePreflight {
        status: StructuredFamilySafeWriteScaffoldStatus::RejectedByDefault,
        passed: false,
        rejection_reasons: structured_family_safe_write_default_rejection_reasons(),
    }
}

pub fn validate_safe_write_target_policy_scaffold(
    _plan: &StructuredFamilySafeWritePlan,
) -> StructuredFamilySafeWriteTargetPolicy {
    StructuredFamilySafeWriteTargetPolicy {
        status: StructuredFamilySafeWriteScaffoldStatus::NotApproved,
        real_config_target_enabled: false,
        rejection_reasons: structured_family_safe_write_default_rejection_reasons(),
    }
}

pub fn prepare_safe_write_backup_plan_scaffold(
    _plan: &StructuredFamilySafeWritePlan,
) -> StructuredFamilySafeWriteBackupPlan {
    StructuredFamilySafeWriteBackupPlan {
        status: StructuredFamilySafeWriteScaffoldStatus::NotApproved,
        backup_creation_enabled: false,
        restore_execution_enabled: false,
        rejection_reasons: structured_family_safe_write_default_rejection_reasons(),
    }
}

pub fn prepare_safe_write_rollback_plan_scaffold(
    _plan: &StructuredFamilySafeWritePlan,
) -> StructuredFamilySafeWriteRollbackPlan {
    StructuredFamilySafeWriteRollbackPlan {
        status: StructuredFamilySafeWriteScaffoldStatus::NotApproved,
        rollback_execution_enabled: false,
        rejection_reasons: structured_family_safe_write_default_rejection_reasons(),
    }
}

pub fn verify_manual_approval_state_scaffold(
    _plan: &StructuredFamilySafeWritePlan,
) -> StructuredFamilySafeWriteApprovalState {
    StructuredFamilySafeWriteApprovalState {
        status: StructuredFamilySafeWriteScaffoldStatus::NotApproved,
        executor_wiring_approved: false,
        real_write_scope_approved: false,
        first_real_config_write_approved: false,
        gui_real_write_controls_approved: false,
        rejection_reasons: structured_family_safe_write_default_rejection_reasons(),
    }
}

pub fn execute_safe_write_scaffold(
    _plan: &StructuredFamilySafeWritePlan,
) -> StructuredFamilySafeWriteExecutionReceipt {
    StructuredFamilySafeWriteExecutionReceipt {
        status: StructuredFamilySafeWriteScaffoldStatus::RejectedByDefault,
        executed: false,
        real_config_touched: false,
        backup_created: false,
        restore_executed: false,
        rollback_executed: false,
        reload_run: false,
        runtime_mutated: false,
        rejection_reasons: structured_family_safe_write_default_rejection_reasons(),
    }
}

pub fn verify_safe_write_result_scaffold(
    receipt: &StructuredFamilySafeWriteExecutionReceipt,
) -> StructuredFamilySafeWritePreflight {
    let mut rejection_reasons = receipt.rejection_reasons.clone();
    if receipt.executed {
        rejection_reasons.push(StructuredFamilySafeWriteRejectionReason::BlockedPlan);
    }
    StructuredFamilySafeWritePreflight {
        status: StructuredFamilySafeWriteScaffoldStatus::RejectedByDefault,
        passed: false,
        rejection_reasons,
    }
}

pub fn emit_safe_write_audit_record_scaffold(
    receipt: &StructuredFamilySafeWriteExecutionReceipt,
) -> StructuredFamilySafeWriteAuditRecord {
    StructuredFamilySafeWriteAuditRecord {
        status: StructuredFamilySafeWriteScaffoldStatus::RejectedByDefault,
        summary: "structured-family safe-write scaffold rejected by default; no execution occurred",
        execution_recorded: false,
        rejection_reasons: receipt.rejection_reasons.clone(),
    }
}

pub fn emergency_stop_reason_scaffold() -> StructuredFamilySafeWriteEmergencyStop {
    StructuredFamilySafeWriteEmergencyStop {
        status: StructuredFamilySafeWriteScaffoldStatus::RejectedByDefault,
        reason: StructuredFamilySafeWriteRejectionReason::ExecutorWiringNotApproved,
    }
}
