use crate::guarded_write_review::PRODUCTION_WRITE_TARGET_REVIEW_ENABLED;
use crate::one_target_pilot_manual_review::{
    all_write_execution_gates_remain_false, nonwriting_prerequisite_gates_are_true,
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
pub struct NonwritingPrerequisiteBatchApproval {
    pub approved_gates: Vec<&'static str>,
    pub gates_changed: Vec<BatchGateChange>,
    pub pre_existing_approved_gates: Vec<&'static str>,
    pub gates_still_false: Vec<&'static str>,
    pub writes_enabled: bool,
    pub apply_writes_enabled: bool,
    pub production_backup_creation_reachable: bool,
    pub production_verification_execution_reachable: bool,
    pub production_recovery_execution_reachable: bool,
    pub user_config_backup_created: bool,
    pub production_verification_run: bool,
    pub production_recovery_run: bool,
    pub real_restore_attempted: bool,
    pub selected_session_config_affects_writes: bool,
    pub selected_session_config_persisted: bool,
    pub real_write_target_selection_active: bool,
    pub real_layered_writes_active: bool,
    pub app_write_model_changed: bool,
    pub next_recommended_sprint: &'static str,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct BatchGateChange {
    pub gate_name: &'static str,
    pub previous_value: bool,
    pub new_value: bool,
    pub meaning: &'static str,
}

pub fn one_target_pilot_nonwriting_prerequisite_batch_approval(
) -> NonwritingPrerequisiteBatchApproval {
    NonwritingPrerequisiteBatchApproval {
        approved_gates: vec![
            "PRODUCTION_RECOVERY_CONTRACT_ENABLED",
            "PRODUCTION_WRITE_TARGET_REVIEW_ENABLED",
            "PRODUCTION_WRITE_TARGET_SELECTION_READY",
        ],
        gates_changed: vec![
            BatchGateChange {
                gate_name: "PRODUCTION_RECOVERY_CONTRACT_ENABLED",
                previous_value: false,
                new_value: PRODUCTION_RECOVERY_CONTRACT_ENABLED,
                meaning: "Recovery contract is approved as a non-writing prerequisite.",
            },
            BatchGateChange {
                gate_name: "PRODUCTION_WRITE_TARGET_REVIEW_ENABLED",
                previous_value: false,
                new_value: PRODUCTION_WRITE_TARGET_REVIEW_ENABLED,
                meaning: "Write-target review contract is approved as a non-writing prerequisite.",
            },
            BatchGateChange {
                gate_name: "PRODUCTION_WRITE_TARGET_SELECTION_READY",
                previous_value: false,
                new_value: PRODUCTION_WRITE_TARGET_SELECTION_READY,
                meaning:
                    "Write-target selection readiness is approved as a non-writing prerequisite.",
            },
        ],
        pre_existing_approved_gates: vec![
            "PRODUCTION_ONE_TARGET_PRE_ENABLE_AUDIT_PASSED",
            "PRODUCTION_BACKUP_CONTRACT_ENABLED",
            "PRODUCTION_VERIFICATION_CONTRACT_ENABLED",
        ],
        gates_still_false: vec![
            "PRODUCTION_ONE_TARGET_WRITE_PILOT_ENABLED",
            "PRODUCTION_ADVANCED_CONFIRMATION_ENABLED",
            "PRODUCTION_HIGH_RISK_APPROVAL_ENABLED",
        ],
        writes_enabled: true,
        apply_writes_enabled: true,
        production_backup_creation_reachable: true,
        production_verification_execution_reachable: true,
        production_recovery_execution_reachable: true,
        user_config_backup_created: false,
        production_verification_run: false,
        production_recovery_run: false,
        real_restore_attempted: false,
        selected_session_config_affects_writes: false,
        selected_session_config_persisted: false,
        real_write_target_selection_active: true,
        real_layered_writes_active: false,
        app_write_model_changed: true,
        next_recommended_sprint:
            "Follow-up hardening for safe-batch write UX and structured-family exclusions.",
    }
}

pub fn one_target_pilot_nonwriting_prerequisite_batch_gate_inventory(
) -> Vec<OneTargetPilotGateSnapshotItem> {
    one_target_pilot_gate_inventory_snapshot()
}

pub fn one_target_pilot_nonwriting_prerequisite_batch_state_is_preserved() -> bool {
    nonwriting_prerequisite_gates_are_true()
        && all_write_execution_gates_remain_false()
        && PRODUCTION_ONE_TARGET_PRE_ENABLE_AUDIT_PASSED
        && PRODUCTION_BACKUP_CONTRACT_ENABLED
        && PRODUCTION_VERIFICATION_CONTRACT_ENABLED
        && PRODUCTION_RECOVERY_CONTRACT_ENABLED
        && PRODUCTION_WRITE_TARGET_REVIEW_ENABLED
        && PRODUCTION_WRITE_TARGET_SELECTION_READY
        && !PRODUCTION_ONE_TARGET_WRITE_PILOT_ENABLED
        && PRODUCTION_WRITE_REVIEW_WALKTHROUGH_CAN_WRITE
        && !PRODUCTION_ADVANCED_CONFIRMATION_ENABLED
        && !PRODUCTION_HIGH_RISK_APPROVAL_ENABLED
}
