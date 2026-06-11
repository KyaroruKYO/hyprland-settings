use hyprland_settings::one_target_pilot_manual_review::one_target_pilot_remaining_blockers;
use hyprland_settings::write_classification::SAFE_WRITABLE_ROWS;

#[test]
fn remaining_blocker_inventory_tracks_gate_flip_and_activation_blockers() {
    let blockers = one_target_pilot_remaining_blockers();
    let ids = blockers
        .iter()
        .map(|blocker| blocker.blocker_id)
        .collect::<Vec<_>>();

    for expected in [
        "manual-smoke-source-only",
        "production-backup-inactive",
        "production-write-inactive",
        "production-reread-verification-inactive",
        "production-recovery-inactive",
        "apply-integration-not-approved",
        "all-production-gates-false",
        "release-gate-flip-proposal-not-created",
    ] {
        assert!(ids.contains(&expected), "missing blocker: {expected}");
    }

    assert!(blockers
        .iter()
        .any(|blocker| blocker.blocks_gate_flip_proposal));
    assert!(blockers
        .iter()
        .all(|blocker| blocker.blocks_production_activation
            && !blocker.required_next_proof.is_empty()));
    assert_eq!(SAFE_WRITABLE_ROWS.len(), 341);
}
