use hyprland_settings::one_target_pilot_focused_visual_smoke::{
    one_target_pilot_focused_visual_remaining_blockers,
    one_target_pilot_focused_visual_smoke_result,
};
use hyprland_settings::write_classification::SAFE_WRITABLE_ROWS;

#[test]
fn focused_visual_pass_removes_visual_blocker_but_keeps_production_activation_blockers() {
    let result = one_target_pilot_focused_visual_smoke_result();
    let blockers = one_target_pilot_focused_visual_remaining_blockers(&result);
    let ids = blockers
        .iter()
        .map(|blocker| blocker.blocker_id)
        .collect::<Vec<_>>();

    assert!(!ids.contains(&"focused-visual-smoke-incomplete"));
    assert!(!ids.contains(&"manual-smoke-source-only"));
    assert!(ids.contains(&"production-backup-inactive"));
    assert!(ids.contains(&"production-write-inactive"));
    assert!(ids.contains(&"production-reread-verification-inactive"));
    assert!(ids.contains(&"production-recovery-inactive"));
    assert!(ids.contains(&"apply-integration-not-approved"));
    assert!(ids.contains(&"all-write-activation-gates-false"));
    assert!(ids.contains(&"gate-flip-proposal-draft-not-executed"));
    assert!(blockers
        .iter()
        .all(|blocker| !blocker.required_next_proof.is_empty()));
    assert_eq!(SAFE_WRITABLE_ROWS.len(), 341);
}
