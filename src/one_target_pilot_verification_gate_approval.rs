use crate::guarded_write_review::PRODUCTION_WRITE_TARGET_REVIEW_ENABLED;
use crate::one_target_pilot_manual_review::{
    all_write_execution_gates_remain_false, pre_enable_backup_and_verification_gates_are_true,
};
use crate::one_target_pilot_pre_enable_audit::{
    one_target_pilot_gate_inventory_snapshot, OneTargetPilotGateSnapshotItem,
    PRODUCTION_ONE_TARGET_PRE_ENABLE_AUDIT_PASSED,
};
use crate::one_target_write_pilot::PRODUCTION_ONE_TARGET_WRITE_PILOT_ENABLED;
use crate::production_advanced_confirmation::PRODUCTION_ADVANCED_CONFIRMATION_ENABLED;
use crate::production_backup_contract::PRODUCTION_BACKUP_CONTRACT_ENABLED;
use crate::production_high_risk_approval::PRODUCTION_HIGH_RISK_APPROVAL_ENABLED;
use crate::production_recovery_contract::PRODUCTION_RECOVERY_CONTRACT_ENABLED;
use crate::production_verification_contract::PRODUCTION_VERIFICATION_CONTRACT_ENABLED;
use crate::write_enablement_readiness::PRODUCTION_WRITE_TARGET_SELECTION_READY;
use crate::write_review_walkthrough::PRODUCTION_WRITE_REVIEW_WALKTHROUGH_CAN_WRITE;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct VerificationGateApprovalState {
    pub approved_gate: &'static str,
    pub previous_value: bool,
    pub new_value: bool,
    pub pre_enable_audit_remains_true: bool,
    pub backup_contract_remains_true: bool,
    pub verification_contract_gate_approved: bool,
    pub production_backup_creation_reachable: bool,
    pub production_verification_execution_reachable: bool,
    pub writes_enabled: bool,
    pub apply_behavior_changed: bool,
    pub user_config_backup_created: bool,
    pub production_verification_run: bool,
    pub real_restore_attempted: bool,
    pub selected_session_config_affects_writes: bool,
    pub selected_session_config_persisted: bool,
    pub real_write_target_selection_active: bool,
    pub real_layered_writes_active: bool,
    pub app_write_model_changed: bool,
}

pub fn one_target_pilot_verification_gate_approval_state() -> VerificationGateApprovalState {
    VerificationGateApprovalState {
        approved_gate: "PRODUCTION_VERIFICATION_CONTRACT_ENABLED",
        previous_value: false,
        new_value: PRODUCTION_VERIFICATION_CONTRACT_ENABLED,
        pre_enable_audit_remains_true: PRODUCTION_ONE_TARGET_PRE_ENABLE_AUDIT_PASSED,
        backup_contract_remains_true: PRODUCTION_BACKUP_CONTRACT_ENABLED,
        verification_contract_gate_approved: PRODUCTION_VERIFICATION_CONTRACT_ENABLED,
        production_backup_creation_reachable: false,
        production_verification_execution_reachable: false,
        writes_enabled: false,
        apply_behavior_changed: false,
        user_config_backup_created: false,
        production_verification_run: false,
        real_restore_attempted: false,
        selected_session_config_affects_writes: false,
        selected_session_config_persisted: false,
        real_write_target_selection_active: false,
        real_layered_writes_active: false,
        app_write_model_changed: false,
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct VerificationGateReadinessChange {
    pub pre_enable_audit_gate_passed: bool,
    pub backup_contract_gate_approved: bool,
    pub verification_contract_gate_approved: bool,
    pub verification_contract_allowed_as_prerequisite: bool,
    pub production_backup_creation_reachable: bool,
    pub production_verification_execution_reachable: bool,
    pub recovery_gate_enabled: bool,
    pub target_review_gate_enabled: bool,
    pub target_selection_gate_enabled: bool,
    pub one_target_pilot_gate_enabled: bool,
    pub apply_integration_changed: bool,
    pub writes_enabled: bool,
    pub next_recommended_gate: &'static str,
}

pub fn one_target_pilot_verification_gate_readiness_change() -> VerificationGateReadinessChange {
    VerificationGateReadinessChange {
        pre_enable_audit_gate_passed: PRODUCTION_ONE_TARGET_PRE_ENABLE_AUDIT_PASSED,
        backup_contract_gate_approved: PRODUCTION_BACKUP_CONTRACT_ENABLED,
        verification_contract_gate_approved: PRODUCTION_VERIFICATION_CONTRACT_ENABLED,
        verification_contract_allowed_as_prerequisite: PRODUCTION_VERIFICATION_CONTRACT_ENABLED,
        production_backup_creation_reachable: false,
        production_verification_execution_reachable: false,
        recovery_gate_enabled: PRODUCTION_RECOVERY_CONTRACT_ENABLED,
        target_review_gate_enabled: PRODUCTION_WRITE_TARGET_REVIEW_ENABLED,
        target_selection_gate_enabled: PRODUCTION_WRITE_TARGET_SELECTION_READY,
        one_target_pilot_gate_enabled: PRODUCTION_ONE_TARGET_WRITE_PILOT_ENABLED,
        apply_integration_changed: false,
        writes_enabled: false,
        next_recommended_gate:
            "Manual approval boundary for the first real one-target write pilot.",
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct VerificationGateMeaning {
    pub meaning: &'static str,
    pub non_meanings: Vec<&'static str>,
}

pub fn one_target_pilot_verification_gate_meaning() -> VerificationGateMeaning {
    VerificationGateMeaning {
        meaning:
            "The production reread verification contract stage is approved as a prerequisite for the future one-target pilot path.",
        non_meanings: vec![
            "writes are enabled",
            "Apply can write",
            "real verification is run",
            "real backups are created",
            "production backup creation is reachable",
            "real recovery execution is active",
            "real target selection executes writes",
            "the one-target pilot is active",
            "walkthrough can write",
            "advanced confirmation is active",
            "high-risk approval is active",
        ],
    }
}

pub fn one_target_pilot_verification_gate_inventory_after() -> Vec<OneTargetPilotGateSnapshotItem> {
    one_target_pilot_gate_inventory_snapshot()
}

pub fn one_target_pilot_verification_gate_single_gate_state_is_preserved() -> bool {
    pre_enable_backup_and_verification_gates_are_true()
        && all_write_execution_gates_remain_false()
        && !PRODUCTION_ONE_TARGET_WRITE_PILOT_ENABLED
        && PRODUCTION_WRITE_TARGET_SELECTION_READY
        && PRODUCTION_WRITE_TARGET_REVIEW_ENABLED
        && PRODUCTION_WRITE_REVIEW_WALKTHROUGH_CAN_WRITE
        && PRODUCTION_RECOVERY_CONTRACT_ENABLED
        && !PRODUCTION_ADVANCED_CONFIRMATION_ENABLED
        && !PRODUCTION_HIGH_RISK_APPROVAL_ENABLED
}
