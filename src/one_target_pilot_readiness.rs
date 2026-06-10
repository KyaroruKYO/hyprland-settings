use crate::guarded_write_review::PRODUCTION_WRITE_TARGET_REVIEW_ENABLED;
use crate::one_target_write_pilot::PRODUCTION_ONE_TARGET_WRITE_PILOT_ENABLED;
use crate::production_backup_contract::PRODUCTION_BACKUP_CONTRACT_ENABLED;
use crate::production_recovery_contract::PRODUCTION_RECOVERY_CONTRACT_ENABLED;
use crate::production_verification_contract::PRODUCTION_VERIFICATION_CONTRACT_ENABLED;
use crate::write_enablement_readiness::PRODUCTION_WRITE_TARGET_SELECTION_READY;
use crate::write_review_walkthrough::PRODUCTION_WRITE_REVIEW_WALKTHROUGH_CAN_WRITE;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct OneTargetPilotReadinessMapping {
    pub backup_contract_complete: bool,
    pub backup_collision_policy_complete: bool,
    pub backup_integrity_check_complete: bool,
    pub reread_verification_complete: bool,
    pub verification_failure_behavior_complete: bool,
    pub recovery_contract_complete: bool,
    pub recovery_trigger_model_complete: bool,
    pub restore_operation_contract_complete: bool,
    pub restore_verification_contract_complete: bool,
    pub recovery_result_reporting_complete: bool,
    pub fixture_recovery_proof_passed: bool,
    pub advanced_confirmation_policy_complete: bool,
    pub manual_smoke_review_complete: bool,
    pub apply_integration_boundary_approved: bool,
    pub production_backup_enabled: bool,
    pub production_verification_enabled: bool,
    pub production_recovery_enabled: bool,
    pub pilot_gate_enabled: bool,
    pub target_selection_ready: bool,
    pub guarded_review_enabled: bool,
    pub walkthrough_can_write: bool,
}

impl OneTargetPilotReadinessMapping {
    pub fn is_ready_for_production(&self) -> bool {
        self.backup_contract_complete
            && self.backup_collision_policy_complete
            && self.backup_integrity_check_complete
            && self.reread_verification_complete
            && self.verification_failure_behavior_complete
            && self.recovery_contract_complete
            && self.recovery_trigger_model_complete
            && self.restore_operation_contract_complete
            && self.restore_verification_contract_complete
            && self.recovery_result_reporting_complete
            && self.fixture_recovery_proof_passed
            && self.advanced_confirmation_policy_complete
            && self.manual_smoke_review_complete
            && self.apply_integration_boundary_approved
            && self.production_backup_enabled
            && self.production_verification_enabled
            && self.production_recovery_enabled
            && self.pilot_gate_enabled
            && self.target_selection_ready
            && self.guarded_review_enabled
            && self.walkthrough_can_write
    }

    pub fn user_facing_lines(&self) -> Vec<String> {
        vec![
            "Production backup and verification".to_string(),
            "The app will back up this exact file before saving changes.".to_string(),
            "The backup must match the original file before any write can continue.".to_string(),
            "The app will reread the file to confirm the value.".to_string(),
            "If verification fails, the app must not report the change as complete.".to_string(),
            "Rollback/recovery must be implemented before real writes.".to_string(),
            "Recovery".to_string(),
            "If verification fails in a future version, the app will restore the backup."
                .to_string(),
            "The app will reread the restored file before reporting recovery success.".to_string(),
            "The app will not reload Hyprland automatically.".to_string(),
            "If recovery fails, the app will report the failure and leave the backup available."
                .to_string(),
            "Production recovery is not active yet.".to_string(),
            "Production backups are not active yet.".to_string(),
            "Production verification is not active yet.".to_string(),
            "Real writing is not active yet.".to_string(),
            "Apply behavior has not changed.".to_string(),
        ]
    }
}

pub fn current_one_target_pilot_readiness_mapping() -> OneTargetPilotReadinessMapping {
    OneTargetPilotReadinessMapping {
        backup_contract_complete: false,
        backup_collision_policy_complete: false,
        backup_integrity_check_complete: false,
        reread_verification_complete: false,
        verification_failure_behavior_complete: false,
        recovery_contract_complete: false,
        recovery_trigger_model_complete: false,
        restore_operation_contract_complete: false,
        restore_verification_contract_complete: false,
        recovery_result_reporting_complete: false,
        fixture_recovery_proof_passed: false,
        advanced_confirmation_policy_complete: false,
        manual_smoke_review_complete: false,
        apply_integration_boundary_approved: false,
        production_backup_enabled: PRODUCTION_BACKUP_CONTRACT_ENABLED,
        production_verification_enabled: PRODUCTION_VERIFICATION_CONTRACT_ENABLED,
        production_recovery_enabled: PRODUCTION_RECOVERY_CONTRACT_ENABLED,
        pilot_gate_enabled: PRODUCTION_ONE_TARGET_WRITE_PILOT_ENABLED,
        target_selection_ready: PRODUCTION_WRITE_TARGET_SELECTION_READY,
        guarded_review_enabled: PRODUCTION_WRITE_TARGET_REVIEW_ENABLED,
        walkthrough_can_write: PRODUCTION_WRITE_REVIEW_WALKTHROUGH_CAN_WRITE,
    }
}
