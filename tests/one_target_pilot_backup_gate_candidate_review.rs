use hyprland_settings::one_target_pilot_backup_gate_review::{
    one_target_pilot_backup_gate_candidate_review, BackupGateCandidateDecision,
};
use hyprland_settings::one_target_pilot_pre_enable_audit::PRODUCTION_ONE_TARGET_PRE_ENABLE_AUDIT_PASSED;
use hyprland_settings::production_backup_contract::PRODUCTION_BACKUP_CONTRACT_ENABLED;
use hyprland_settings::write_classification::SAFE_WRITABLE_ROWS;

#[test]
fn backup_gate_candidate_review_is_review_only_and_ready_for_user_approval_request() {
    let review = one_target_pilot_backup_gate_candidate_review();

    assert!(review.backup_gate_candidate_present);
    assert!(review.current_backup_gate_value);
    assert!(!review.candidate_gate_remains_false);
    assert!(review.pre_enable_audit_gate_already_true);
    assert!(review.backup_contract_exists);
    assert!(review.backup_path_policy_exists);
    assert!(review.collision_policy_exists);
    assert!(review.fixture_exact_copy_proof_exists);
    assert!(review.fixture_misuse_protection_exists);
    assert!(!review.user_config_backup_created);
    assert!(!review.production_backup_active);
    assert!(!review.apply_connected);
    assert_eq!(
        review.decision,
        BackupGateCandidateDecision::PassedForUserApprovalRequest
    );
    assert_eq!(review.decision.label(), "passed_for_user_approval_request");
    assert!(!review.ready_to_ask_user_for_explicit_approval);
    assert!(review.gate_flipped);
    assert!(!review.writes_enabled);
    assert!(PRODUCTION_ONE_TARGET_PRE_ENABLE_AUDIT_PASSED);
    assert!(PRODUCTION_BACKUP_CONTRACT_ENABLED);
    assert_eq!(SAFE_WRITABLE_ROWS.len(), 341);
}
