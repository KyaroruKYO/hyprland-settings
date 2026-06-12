use hyprland_settings::one_target_pilot_backup_gate_review::{
    one_target_pilot_backup_gate_remaining_blockers,
    one_target_pilot_future_backup_gate_approval_scope,
};
use hyprland_settings::write_classification::SAFE_WRITABLE_ROWS;

#[test]
fn backup_gate_scope_records_approval_and_keeps_writes_disabled() {
    let scope = one_target_pilot_future_backup_gate_approval_scope();

    assert_eq!(scope.allowed_gate, "PRODUCTION_BACKUP_CONTRACT_ENABLED");
    assert_eq!(
        scope.only_allowed_gate_change,
        "already approved in the backup gate sprint; no further backup gate change is pending"
    );
    for expected in [
        "PRODUCTION_ONE_TARGET_WRITE_PILOT_ENABLED",
        "PRODUCTION_WRITE_TARGET_SELECTION_READY",
        "PRODUCTION_WRITE_TARGET_REVIEW_ENABLED",
        "PRODUCTION_WRITE_REVIEW_WALKTHROUGH_CAN_WRITE",
        "PRODUCTION_VERIFICATION_CONTRACT_ENABLED",
        "PRODUCTION_RECOVERY_CONTRACT_ENABLED",
        "PRODUCTION_ADVANCED_CONFIRMATION_ENABLED",
        "PRODUCTION_HIGH_RISK_APPROVAL_ENABLED",
    ] {
        assert!(
            scope.gates_that_must_remain_false.contains(&expected),
            "missing gate from future false list: {expected}"
        );
    }
    assert!(scope.writes_remain_disabled);
    assert!(scope.apply_remains_disconnected);
    assert!(scope.target_selection_remains_inactive);
    assert!(scope
        .not_meaning_if_later_approved
        .contains(&"Apply can write"));
    assert_eq!(SAFE_WRITABLE_ROWS.len(), 341);
}

#[test]
fn remaining_blockers_keep_production_activation_blocked_after_backup_approval() {
    let blockers = one_target_pilot_backup_gate_remaining_blockers();
    let ids = blockers
        .iter()
        .map(|blocker| blocker.blocker_id)
        .collect::<Vec<_>>();

    for expected in [
        "explicit-user-approval-needed-for-backup-gate-sprint",
        "production-backup-gate-approved-but-non-executing",
        "production-backup-implementation-not-active",
        "production-verification-gate-not-approved",
        "production-recovery-gate-not-approved",
        "production-write-target-review-not-active",
        "production-target-selection-not-active",
        "one-target-pilot-not-active",
        "apply-integration-not-approved",
    ] {
        assert!(ids.contains(&expected), "missing blocker: {expected}");
    }
    assert!(blockers
        .iter()
        .all(|blocker| blocker.blocks_production_activation
            && !blocker.required_next_proof.is_empty()));
    assert_eq!(SAFE_WRITABLE_ROWS.len(), 341);
}
