use crate::one_target_pilot_focused_visual_smoke::{
    one_target_pilot_focused_visual_gate_flip_proposal_readiness,
    one_target_pilot_focused_visual_gate_inventory_verification,
    one_target_pilot_focused_visual_smoke_result,
};
use crate::one_target_pilot_pre_enable_audit::OneTargetPilotGateSnapshotItem;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ProposalReviewDecision {
    PassedForUserApprovalRequest,
    NeedsRevision,
    Rejected,
    Blocked,
}

impl ProposalReviewDecision {
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
pub struct ProposalArtifactReview {
    pub proposal_artifact_present: bool,
    pub proposal_artifact_parsed_readable: bool,
    pub proposal_is_draft_only: bool,
    pub proposal_says_no_gate_flipped: bool,
    pub proposal_requires_user_approval: bool,
    pub proposal_requires_separate_sprint: bool,
    pub proposal_target_class_is_narrow: bool,
    pub proposal_exclusions_are_complete_in_original: bool,
    pub proposal_stop_conditions_are_complete: bool,
    pub proposal_rollback_conditions_are_represented: bool,
    pub proposal_proof_references_are_present: bool,
    pub proposal_implementation_scope_is_not_mixed_with_review: bool,
    pub original_revision_reasons: Vec<&'static str>,
}

pub fn one_target_pilot_proposal_artifact_review() -> ProposalArtifactReview {
    ProposalArtifactReview {
        proposal_artifact_present: true,
        proposal_artifact_parsed_readable: true,
        proposal_is_draft_only: true,
        proposal_says_no_gate_flipped: true,
        proposal_requires_user_approval: true,
        proposal_requires_separate_sprint: true,
        proposal_target_class_is_narrow: true,
        proposal_exclusions_are_complete_in_original: false,
        proposal_stop_conditions_are_complete: false,
        proposal_rollback_conditions_are_represented: true,
        proposal_proof_references_are_present: true,
        proposal_implementation_scope_is_not_mixed_with_review: true,
        original_revision_reasons: vec![
            "original draft should explicitly exclude unknown management state",
            "original draft should explicitly exclude targets requiring script or Lua execution",
            "original draft should explicitly require restored byte and scalar-value verification",
            "original draft should present future gate flips as staged rather than simultaneous",
        ],
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ProposalConsistencyReview {
    pub focused_visual_smoke_proof_referenced: bool,
    pub manual_smoke_evidence_referenced: bool,
    pub fixture_write_proof_referenced: bool,
    pub backup_proof_referenced: bool,
    pub verification_proof_referenced: bool,
    pub recovery_proof_referenced: bool,
    pub advanced_confirmation_proof_referenced: bool,
    pub high_risk_boundary_proof_referenced: bool,
    pub target_management_risk_policy_referenced: bool,
    pub apply_isolation_proof_referenced: bool,
    pub staged_gate_state_referenced: bool,
    pub enables_writes_in_this_sprint: bool,
    pub treats_draft_as_gate_flip_approval: bool,
    pub changes_apply_behavior: bool,
}

pub fn one_target_pilot_proposal_consistency_review() -> ProposalConsistencyReview {
    ProposalConsistencyReview {
        focused_visual_smoke_proof_referenced: true,
        manual_smoke_evidence_referenced: true,
        fixture_write_proof_referenced: true,
        backup_proof_referenced: true,
        verification_proof_referenced: true,
        recovery_proof_referenced: true,
        advanced_confirmation_proof_referenced: true,
        high_risk_boundary_proof_referenced: true,
        target_management_risk_policy_referenced: true,
        apply_isolation_proof_referenced: true,
        staged_gate_state_referenced: true,
        enables_writes_in_this_sprint: false,
        treats_draft_as_gate_flip_approval: false,
        changes_apply_behavior: false,
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct FutureGateListReview {
    pub proposed_future_gates: Vec<&'static str>,
    pub gates_that_must_remain_false_now: Vec<&'static str>,
    pub staged_flip_recommendation: Vec<&'static str>,
    pub gates_needing_more_proof_before_flip: Vec<&'static str>,
    pub gates_that_should_not_flip_together_without_more_proof: Vec<&'static str>,
    pub pre_enable_gate_true_and_write_gates_false: bool,
}

pub fn one_target_pilot_future_gate_list_review() -> FutureGateListReview {
    let proposed_future_gates = vec![
        "PRODUCTION_ONE_TARGET_PRE_ENABLE_AUDIT_PASSED",
        "PRODUCTION_WRITE_TARGET_SELECTION_READY",
        "PRODUCTION_WRITE_TARGET_REVIEW_ENABLED",
        "PRODUCTION_BACKUP_CONTRACT_ENABLED",
        "PRODUCTION_VERIFICATION_CONTRACT_ENABLED",
        "PRODUCTION_RECOVERY_CONTRACT_ENABLED",
        "PRODUCTION_ONE_TARGET_WRITE_PILOT_ENABLED",
        "PRODUCTION_WRITE_REVIEW_WALKTHROUGH_CAN_WRITE",
    ];
    FutureGateListReview {
        gates_that_must_remain_false_now: vec![
            "PRODUCTION_ONE_TARGET_WRITE_PILOT_ENABLED",
            "PRODUCTION_WRITE_TARGET_SELECTION_READY",
            "PRODUCTION_WRITE_TARGET_REVIEW_ENABLED",
            "PRODUCTION_WRITE_REVIEW_WALKTHROUGH_CAN_WRITE",
            "PRODUCTION_BACKUP_CONTRACT_ENABLED",
            "PRODUCTION_VERIFICATION_CONTRACT_ENABLED",
            "PRODUCTION_RECOVERY_CONTRACT_ENABLED",
            "PRODUCTION_ADVANCED_CONFIRMATION_ENABLED",
            "PRODUCTION_HIGH_RISK_APPROVAL_ENABLED",
        ],
        staged_flip_recommendation: vec![
            "1. Pre-enable audit gate is already approved.",
            "2. Activate backup contract only after production backup implementation proof.",
            "3. Activate verification contract only after production reread implementation proof.",
            "4. Activate recovery contract only after production restore and restore-verification proof.",
            "5. Activate write target review only after the normal scalar target path is proven.",
            "6. Mark target selection ready only for the approved normal scalar class.",
            "7. Enable the one-target pilot gate only after all prior stages pass.",
            "8. Allow walkthrough writes only after Apply integration is approved.",
        ],
        gates_needing_more_proof_before_flip: proposed_future_gates
            .iter()
            .copied()
            .filter(|gate| *gate != "PRODUCTION_ONE_TARGET_PRE_ENABLE_AUDIT_PASSED")
            .collect(),
        gates_that_should_not_flip_together_without_more_proof: proposed_future_gates
            .iter()
            .copied()
            .filter(|gate| *gate != "PRODUCTION_ONE_TARGET_PRE_ENABLE_AUDIT_PASSED")
            .collect(),
        proposed_future_gates,
        pre_enable_gate_true_and_write_gates_false: true,
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct TargetScopeReview {
    pub allowed_target: &'static str,
    pub one_non_high_risk_scalar_line: bool,
    pub one_normal_file: bool,
    pub exact_line_number_known: bool,
    pub no_ambiguity: bool,
    pub generated_targets_excluded: bool,
    pub script_managed_targets_excluded: bool,
    pub script_referenced_targets_excluded: bool,
    pub symlink_managed_targets_excluded: bool,
    pub symlink_targets_excluded: bool,
    pub high_risk_rows_excluded: bool,
    pub structured_targets_excluded: bool,
    pub missing_line_targets_excluded: bool,
    pub duplicate_ambiguous_targets_excluded: bool,
    pub unknown_management_state_excluded: bool,
    pub script_or_lua_required_targets_excluded: bool,
}

pub fn one_target_pilot_target_scope_review() -> TargetScopeReview {
    TargetScopeReview {
        allowed_target:
            "one existing non-high-risk scalar line in one normal config file with exact line number and no ambiguity",
        one_non_high_risk_scalar_line: true,
        one_normal_file: true,
        exact_line_number_known: true,
        no_ambiguity: true,
        generated_targets_excluded: true,
        script_managed_targets_excluded: true,
        script_referenced_targets_excluded: true,
        symlink_managed_targets_excluded: true,
        symlink_targets_excluded: true,
        high_risk_rows_excluded: true,
        structured_targets_excluded: true,
        missing_line_targets_excluded: true,
        duplicate_ambiguous_targets_excluded: true,
        unknown_management_state_excluded: true,
        script_or_lua_required_targets_excluded: true,
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct BackupVerificationRecoveryReview {
    pub exact_backup_before_write_required: bool,
    pub backup_byte_equality_proof_required: bool,
    pub write_only_after_backup_proof_required: bool,
    pub reread_target_after_write_required: bool,
    pub expected_value_verification_required: bool,
    pub restore_backup_on_write_failure_after_backup_required: bool,
    pub restore_backup_on_verification_failure_required: bool,
    pub reread_restored_file_required: bool,
    pub restored_bytes_and_value_verification_required: bool,
    pub report_recovery_failure_without_hiding_backup_required: bool,
    pub automatic_hyprland_reload_allowed: bool,
    pub mutating_hyprctl_allowed: bool,
}

pub fn one_target_pilot_backup_verification_recovery_review() -> BackupVerificationRecoveryReview {
    BackupVerificationRecoveryReview {
        exact_backup_before_write_required: true,
        backup_byte_equality_proof_required: true,
        write_only_after_backup_proof_required: true,
        reread_target_after_write_required: true,
        expected_value_verification_required: true,
        restore_backup_on_write_failure_after_backup_required: true,
        restore_backup_on_verification_failure_required: true,
        reread_restored_file_required: true,
        restored_bytes_and_value_verification_required: true,
        report_recovery_failure_without_hiding_backup_required: true,
        automatic_hyprland_reload_allowed: false,
        mutating_hyprctl_allowed: false,
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ApplyIntegrationBoundaryReview {
    pub apply_integration_only_for_approved_one_target_path: bool,
    pub high_risk_policy_remains_enforced: bool,
    pub blocked_target_classes_remain_blocked: bool,
    pub session_selected_config_does_not_automatically_affect_writes: bool,
    pub real_write_target_must_be_normal_scalar_file_occurrence: bool,
    pub backup_verify_recover_wrap_any_write: bool,
    pub errors_block_write_or_report_failure_safely: bool,
    pub broad_apply_changes_allowed: bool,
}

pub fn one_target_pilot_apply_integration_boundary_review() -> ApplyIntegrationBoundaryReview {
    ApplyIntegrationBoundaryReview {
        apply_integration_only_for_approved_one_target_path: true,
        high_risk_policy_remains_enforced: true,
        blocked_target_classes_remain_blocked: true,
        session_selected_config_does_not_automatically_affect_writes: true,
        real_write_target_must_be_normal_scalar_file_occurrence: true,
        backup_verify_recover_wrap_any_write: true,
        errors_block_write_or_report_failure_safely: true,
        broad_apply_changes_allowed: false,
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ProposalDecisionReview {
    pub decision: ProposalReviewDecision,
    pub ready_to_ask_user_for_explicit_approval: bool,
    pub gate_flip_executed: bool,
    pub writes_enabled: bool,
    pub reviewed_draft_created: bool,
    pub original_required_revisions: Vec<&'static str>,
    pub decision_scope: &'static str,
}

pub fn one_target_pilot_proposal_decision_review() -> ProposalDecisionReview {
    let artifact = one_target_pilot_proposal_artifact_review();
    ProposalDecisionReview {
        decision: ProposalReviewDecision::PassedForUserApprovalRequest,
        ready_to_ask_user_for_explicit_approval: true,
        gate_flip_executed: false,
        writes_enabled: false,
        reviewed_draft_created: true,
        original_required_revisions: artifact.original_revision_reasons,
        decision_scope:
            "Reviewed draft only; ready to ask user for explicit approval in a later sprint, not approved to flip gates.",
    }
}

pub fn one_target_pilot_proposal_gate_inventory_verification() -> Vec<OneTargetPilotGateSnapshotItem>
{
    one_target_pilot_focused_visual_gate_inventory_verification()
}

pub fn one_target_pilot_proposal_reviewed_draft_required() -> bool {
    !one_target_pilot_proposal_artifact_review()
        .original_revision_reasons
        .is_empty()
}

pub fn one_target_pilot_proposal_review_references_focused_readiness() -> bool {
    let result = one_target_pilot_focused_visual_smoke_result();
    one_target_pilot_focused_visual_gate_flip_proposal_readiness(&result)
        .ready_for_separate_gate_flip_proposal
}
