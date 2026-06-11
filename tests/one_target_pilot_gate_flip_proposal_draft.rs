use hyprland_settings::one_target_pilot_live_visual_smoke::{
    one_target_pilot_gate_flip_proposal_draft, one_target_pilot_live_visual_smoke_review_result,
};
use hyprland_settings::one_target_write_pilot::PRODUCTION_ONE_TARGET_WRITE_PILOT_ENABLED;
use hyprland_settings::write_classification::SAFE_WRITABLE_ROWS;

#[test]
fn gate_flip_proposal_draft_is_not_created_for_inconclusive_review() {
    let result = one_target_pilot_live_visual_smoke_review_result();
    let draft = one_target_pilot_gate_flip_proposal_draft(&result);

    assert!(draft.is_none());
    assert!(!PRODUCTION_ONE_TARGET_WRITE_PILOT_ENABLED);
    assert_eq!(SAFE_WRITABLE_ROWS.len(), 341);
}
