use hyprland_settings::guarded_write_review::PRODUCTION_WRITE_TARGET_REVIEW_ENABLED;
use hyprland_settings::one_target_pilot_gate_flip_proposal_review::one_target_pilot_future_gate_list_review;
use hyprland_settings::one_target_pilot_pre_enable_audit::PRODUCTION_ONE_TARGET_PRE_ENABLE_AUDIT_PASSED;
use hyprland_settings::one_target_write_pilot::PRODUCTION_ONE_TARGET_WRITE_PILOT_ENABLED;
use hyprland_settings::production_advanced_confirmation::PRODUCTION_ADVANCED_CONFIRMATION_ENABLED;
use hyprland_settings::production_backup_contract::PRODUCTION_BACKUP_CONTRACT_ENABLED;
use hyprland_settings::production_high_risk_approval::PRODUCTION_HIGH_RISK_APPROVAL_ENABLED;
use hyprland_settings::production_recovery_contract::PRODUCTION_RECOVERY_CONTRACT_ENABLED;
use hyprland_settings::production_verification_contract::PRODUCTION_VERIFICATION_CONTRACT_ENABLED;
use hyprland_settings::write_classification::SAFE_WRITABLE_ROWS;
use hyprland_settings::write_enablement_readiness::PRODUCTION_WRITE_TARGET_SELECTION_READY;
use hyprland_settings::write_review_walkthrough::PRODUCTION_WRITE_REVIEW_WALKTHROUGH_CAN_WRITE;

#[test]
fn proposal_gate_list_is_future_only_staged_with_pre_enable_gate_approved() {
    let review = one_target_pilot_future_gate_list_review();

    assert!(review
        .proposed_future_gates
        .contains(&"PRODUCTION_ONE_TARGET_PRE_ENABLE_AUDIT_PASSED"));
    assert!(review
        .proposed_future_gates
        .contains(&"PRODUCTION_ONE_TARGET_WRITE_PILOT_ENABLED"));
    assert!(review
        .gates_that_must_remain_false_now
        .contains(&"PRODUCTION_ADVANCED_CONFIRMATION_ENABLED"));
    assert!(review
        .gates_that_must_remain_false_now
        .contains(&"PRODUCTION_HIGH_RISK_APPROVAL_ENABLED"));
    assert_eq!(review.staged_flip_recommendation.len(), 8);
    assert_eq!(review.gates_needing_more_proof_before_flip.len(), 7);
    assert_eq!(
        review
            .gates_that_should_not_flip_together_without_more_proof
            .len(),
        7
    );
    assert!(review.pre_enable_gate_true_and_write_gates_false);
    assert!(PRODUCTION_ONE_TARGET_PRE_ENABLE_AUDIT_PASSED);
    assert!(!PRODUCTION_ONE_TARGET_WRITE_PILOT_ENABLED);
    assert!(!PRODUCTION_WRITE_TARGET_SELECTION_READY);
    assert!(!PRODUCTION_WRITE_TARGET_REVIEW_ENABLED);
    assert!(!PRODUCTION_WRITE_REVIEW_WALKTHROUGH_CAN_WRITE);
    assert!(!PRODUCTION_BACKUP_CONTRACT_ENABLED);
    assert!(!PRODUCTION_VERIFICATION_CONTRACT_ENABLED);
    assert!(!PRODUCTION_RECOVERY_CONTRACT_ENABLED);
    assert!(!PRODUCTION_ADVANCED_CONFIRMATION_ENABLED);
    assert!(!PRODUCTION_HIGH_RISK_APPROVAL_ENABLED);
    assert_eq!(SAFE_WRITABLE_ROWS.len(), 341);
}
