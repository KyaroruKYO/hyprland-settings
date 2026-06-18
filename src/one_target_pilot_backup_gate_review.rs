use crate::guarded_write_review::PRODUCTION_WRITE_TARGET_REVIEW_ENABLED;
use crate::one_target_pilot_manual_review::production_write_path_remains_disabled;
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

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum BackupGateCandidateDecision {
    PassedForUserApprovalRequest,
    NeedsRevision,
    Rejected,
    Blocked,
}

impl BackupGateCandidateDecision {
    pub fn label(self) -> &'static str {
        match self {
            Self::PassedForUserApprovalRequest => "passed_for_user_approval_request",
            Self::NeedsRevision => "needs_revision",
            Self::Rejected => "rejected",
            Self::Blocked => "blocked",
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct BackupGateCandidateReview {
    pub backup_gate_candidate_present: bool,
    pub current_backup_gate_value: bool,
    pub candidate_gate_remains_false: bool,
    pub pre_enable_audit_gate_already_true: bool,
    pub backup_contract_exists: bool,
    pub backup_path_policy_exists: bool,
    pub collision_policy_exists: bool,
    pub fixture_exact_copy_proof_exists: bool,
    pub fixture_misuse_protection_exists: bool,
    pub user_config_backup_created: bool,
    pub production_backup_active: bool,
    pub apply_connected: bool,
    pub decision: BackupGateCandidateDecision,
    pub ready_to_ask_user_for_explicit_approval: bool,
    pub gate_flipped: bool,
    pub writes_enabled: bool,
}

pub fn one_target_pilot_backup_gate_candidate_review() -> BackupGateCandidateReview {
    BackupGateCandidateReview {
        backup_gate_candidate_present: true,
        current_backup_gate_value: PRODUCTION_BACKUP_CONTRACT_ENABLED,
        candidate_gate_remains_false: !PRODUCTION_BACKUP_CONTRACT_ENABLED,
        pre_enable_audit_gate_already_true: PRODUCTION_ONE_TARGET_PRE_ENABLE_AUDIT_PASSED,
        backup_contract_exists: true,
        backup_path_policy_exists: true,
        collision_policy_exists: true,
        fixture_exact_copy_proof_exists: true,
        fixture_misuse_protection_exists: true,
        user_config_backup_created: false,
        production_backup_active: false,
        apply_connected: false,
        decision: BackupGateCandidateDecision::PassedForUserApprovalRequest,
        ready_to_ask_user_for_explicit_approval: false,
        gate_flipped: PRODUCTION_BACKUP_CONTRACT_ENABLED,
        writes_enabled: false,
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct BackupContractMaturityReview {
    pub exact_target_file_backup_required: bool,
    pub same_directory_policy_represented: bool,
    pub timestamped_backup_path_represented: bool,
    pub collision_safe_backup_path_represented: bool,
    pub byte_equality_proof_represented: bool,
    pub backup_before_write_required: bool,
    pub no_write_without_backup_proof_required: bool,
    pub fixture_only_proof_helpers_reject_non_temp_misuse: bool,
    pub target_exclusions_preserved_by_backup_contract: bool,
    pub backup_contract_implies_write_activation: bool,
    pub backup_contract_implies_verification_activation: bool,
    pub backup_contract_implies_recovery_activation: bool,
    pub user_config_backup_created: bool,
}

pub fn one_target_pilot_backup_contract_maturity_review() -> BackupContractMaturityReview {
    BackupContractMaturityReview {
        exact_target_file_backup_required: true,
        same_directory_policy_represented: true,
        timestamped_backup_path_represented: true,
        collision_safe_backup_path_represented: true,
        byte_equality_proof_represented: true,
        backup_before_write_required: true,
        no_write_without_backup_proof_required: true,
        fixture_only_proof_helpers_reject_non_temp_misuse: true,
        target_exclusions_preserved_by_backup_contract: true,
        backup_contract_implies_write_activation: false,
        backup_contract_implies_verification_activation: false,
        backup_contract_implies_recovery_activation: false,
        user_config_backup_created: false,
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct BackupSafetyBoundaryReview {
    pub enables_apply_writes: bool,
    pub enables_real_target_selection: bool,
    pub enables_one_target_pilot: bool,
    pub enables_reread_verification: bool,
    pub enables_recovery: bool,
    pub enables_advanced_confirmation: bool,
    pub enables_high_risk_approval: bool,
    pub allows_hyprland_reload: bool,
    pub allows_mutating_hyprctl: bool,
    pub allows_script_execution: bool,
    pub allows_lua_execution: bool,
    pub allows_profile_switching: bool,
    pub allows_mode_switching: bool,
    pub future_meaning: &'static str,
}

pub fn one_target_pilot_backup_safety_boundary_review() -> BackupSafetyBoundaryReview {
    BackupSafetyBoundaryReview {
        enables_apply_writes: false,
        enables_real_target_selection: false,
        enables_one_target_pilot: false,
        enables_reread_verification: false,
        enables_recovery: false,
        enables_advanced_confirmation: false,
        enables_high_risk_approval: false,
        allows_hyprland_reload: false,
        allows_mutating_hyprctl: false,
        allows_script_execution: false,
        allows_lua_execution: false,
        allows_profile_switching: false,
        allows_mode_switching: false,
        future_meaning:
            "A later backup gate approval may only allow the backup contract to become active for the approved one-target path; it still would not enable writes or Apply.",
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct FutureBackupGateApprovalScope {
    pub allowed_gate: &'static str,
    pub only_allowed_gate_change: &'static str,
    pub gates_that_must_remain_false: Vec<&'static str>,
    pub writes_remain_disabled: bool,
    pub apply_remains_disconnected: bool,
    pub target_selection_remains_inactive: bool,
    pub meaning_if_later_approved: &'static str,
    pub not_meaning_if_later_approved: Vec<&'static str>,
}

pub fn one_target_pilot_future_backup_gate_approval_scope() -> FutureBackupGateApprovalScope {
    FutureBackupGateApprovalScope {
        allowed_gate: "PRODUCTION_BACKUP_CONTRACT_ENABLED",
        only_allowed_gate_change:
            "already approved in the backup gate sprint; no further backup gate change is pending",
        gates_that_must_remain_false: vec![
            "PRODUCTION_ONE_TARGET_WRITE_PILOT_ENABLED",
            "PRODUCTION_WRITE_TARGET_SELECTION_READY",
            "PRODUCTION_WRITE_TARGET_REVIEW_ENABLED",
            "PRODUCTION_WRITE_REVIEW_WALKTHROUGH_CAN_WRITE",
            "PRODUCTION_RECOVERY_CONTRACT_ENABLED",
            "PRODUCTION_ADVANCED_CONFIRMATION_ENABLED",
            "PRODUCTION_HIGH_RISK_APPROVAL_ENABLED",
        ],
        writes_remain_disabled: true,
        apply_remains_disconnected: true,
        target_selection_remains_inactive: true,
        meaning_if_later_approved:
            "The production backup contract is approved as a prerequisite for the approved one-target path.",
        not_meaning_if_later_approved: vec![
            "writes are enabled",
            "Apply can write",
            "the one-target pilot is active",
            "verification execution is active",
            "recovery is active",
            "target selection is active",
        ],
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct BackupGateRemainingBlocker {
    pub blocker_id: &'static str,
    pub description: &'static str,
    pub blocks_user_approval_request: bool,
    pub blocks_production_activation: bool,
    pub required_next_proof: &'static str,
}

pub fn one_target_pilot_backup_gate_remaining_blockers() -> Vec<BackupGateRemainingBlocker> {
    vec![
        blocker(
            "explicit-user-approval-needed-for-backup-gate-sprint",
            "The backup gate approval prompt has been completed.",
            false,
            true,
            "No further user approval is needed for the backup gate itself; later gates still require separate approval.",
        ),
        blocker(
            "production-backup-gate-approved-but-non-executing",
            "The backup contract gate is approved, but no production backup creation is reachable while write-execution gates are false.",
            false,
            true,
            "Verification, recovery, target review, target selection, and pilot gates remain separately staged.",
        ),
        blocker(
            "production-backup-implementation-not-active",
            "Production backups are not active and no user config backups were created.",
            false,
            true,
            "Production backup activation proof in a later staged sprint.",
        ),
        blocker(
            "production-verification-gate-approved-but-non-executing",
            "The verification contract gate is approved, but no production verification execution is reachable while write-execution gates are false.",
            false,
            true,
            "Recovery, target review, target selection, and pilot gates remain separately staged.",
        ),
        blocker(
            "production-recovery-gate-not-approved",
            "Production recovery remains inactive.",
            false,
            true,
            "Separate recovery gate review and approval.",
        ),
        blocker(
            "production-write-target-review-not-active",
            "Guarded target review remains production-disabled.",
            false,
            true,
            "Target review gate approval after backup, verification, and recovery gates.",
        ),
        blocker(
            "production-target-selection-not-active",
            "Real write-target selection remains inactive.",
            false,
            true,
            "Target selection gate approval after all prerequisite contracts.",
        ),
        blocker(
            "one-target-pilot-not-active",
            "The one-target pilot gate remains false.",
            false,
            true,
            "Pilot gate approval after all prior stages pass.",
        ),
        blocker(
            "apply-integration-not-approved",
            "Apply integration is not approved for the pilot path.",
            false,
            true,
            "Explicit Apply integration boundary approval in a separate sprint.",
        ),
    ]
}

fn blocker(
    blocker_id: &'static str,
    description: &'static str,
    blocks_user_approval_request: bool,
    blocks_production_activation: bool,
    required_next_proof: &'static str,
) -> BackupGateRemainingBlocker {
    BackupGateRemainingBlocker {
        blocker_id,
        description,
        blocks_user_approval_request,
        blocks_production_activation,
        required_next_proof,
    }
}

pub fn one_target_pilot_backup_gate_inventory_verification() -> Vec<OneTargetPilotGateSnapshotItem>
{
    one_target_pilot_gate_inventory_snapshot()
}

pub fn backup_gate_candidate_current_staged_state_is_preserved() -> bool {
    PRODUCTION_ONE_TARGET_PRE_ENABLE_AUDIT_PASSED
        && PRODUCTION_BACKUP_CONTRACT_ENABLED
        && !PRODUCTION_ONE_TARGET_WRITE_PILOT_ENABLED
        && !PRODUCTION_WRITE_TARGET_SELECTION_READY
        && !PRODUCTION_WRITE_TARGET_REVIEW_ENABLED
        && !PRODUCTION_WRITE_REVIEW_WALKTHROUGH_CAN_WRITE
        && PRODUCTION_VERIFICATION_CONTRACT_ENABLED
        && !PRODUCTION_RECOVERY_CONTRACT_ENABLED
        && !PRODUCTION_ADVANCED_CONFIRMATION_ENABLED
        && !PRODUCTION_HIGH_RISK_APPROVAL_ENABLED
        && production_write_path_remains_disabled()
}
