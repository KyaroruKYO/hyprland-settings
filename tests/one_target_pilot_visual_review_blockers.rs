use hyprland_settings::one_target_pilot_live_visual_smoke::{
    one_target_pilot_live_visual_smoke_review_result,
    one_target_pilot_visual_review_remaining_blockers,
};
use hyprland_settings::write_classification::SAFE_WRITABLE_ROWS;

#[test]
fn visual_review_blockers_keep_production_blockers_after_inconclusive_review() {
    let result = one_target_pilot_live_visual_smoke_review_result();
    let blockers = one_target_pilot_visual_review_remaining_blockers(&result);
    let ids = blockers
        .iter()
        .map(|blocker| blocker.blocker_id)
        .collect::<Vec<_>>();

    for expected in [
        "live-visual-smoke-inconclusive",
        "manual-smoke-source-only",
        "production-backup-inactive",
        "production-write-inactive",
        "production-reread-verification-inactive",
        "production-recovery-inactive",
        "apply-integration-not-approved",
        "all-production-gates-false",
    ] {
        assert!(ids.contains(&expected), "missing blocker: {expected}");
    }

    assert!(blockers
        .iter()
        .all(|blocker| !blocker.required_next_proof.is_empty()));
    assert_eq!(SAFE_WRITABLE_ROWS.len(), 341);
}
