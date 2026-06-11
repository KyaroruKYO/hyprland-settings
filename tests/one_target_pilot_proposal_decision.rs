use hyprland_settings::one_target_pilot_gate_flip_proposal_review::{
    one_target_pilot_apply_integration_boundary_review, one_target_pilot_proposal_decision_review,
    ProposalReviewDecision,
};
use hyprland_settings::one_target_write_pilot::PRODUCTION_ONE_TARGET_WRITE_PILOT_ENABLED;
use hyprland_settings::write_classification::SAFE_WRITABLE_ROWS;

#[test]
fn proposal_decision_passes_reviewed_draft_for_user_approval_request_only() {
    let decision = one_target_pilot_proposal_decision_review();
    let apply = one_target_pilot_apply_integration_boundary_review();

    assert_eq!(
        decision.decision,
        ProposalReviewDecision::PassedForUserApprovalRequest
    );
    assert_eq!(
        decision.decision.label(),
        "passed_for_user_approval_request"
    );
    assert!(decision.ready_to_ask_user_for_explicit_approval);
    assert!(!decision.gate_flip_executed);
    assert!(!decision.writes_enabled);
    assert!(decision.reviewed_draft_created);
    assert!(!decision.original_required_revisions.is_empty());
    assert!(decision
        .decision_scope
        .contains("not approved to flip gates"));
    assert!(apply.apply_integration_only_for_approved_one_target_path);
    assert!(apply.high_risk_policy_remains_enforced);
    assert!(apply.blocked_target_classes_remain_blocked);
    assert!(apply.session_selected_config_does_not_automatically_affect_writes);
    assert!(apply.real_write_target_must_be_normal_scalar_file_occurrence);
    assert!(apply.backup_verify_recover_wrap_any_write);
    assert!(apply.errors_block_write_or_report_failure_safely);
    assert!(!apply.broad_apply_changes_allowed);
    assert!(!PRODUCTION_ONE_TARGET_WRITE_PILOT_ENABLED);
    assert_eq!(SAFE_WRITABLE_ROWS.len(), 341);
}
