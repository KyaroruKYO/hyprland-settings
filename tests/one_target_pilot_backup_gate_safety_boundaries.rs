use hyprland_settings::one_target_pilot_backup_gate_review::one_target_pilot_backup_safety_boundary_review;
use hyprland_settings::write_classification::SAFE_WRITABLE_ROWS;

#[test]
fn backup_gate_candidate_does_not_enable_write_or_runtime_behavior() {
    let review = one_target_pilot_backup_safety_boundary_review();

    assert!(!review.enables_apply_writes);
    assert!(!review.enables_real_target_selection);
    assert!(!review.enables_one_target_pilot);
    assert!(!review.enables_reread_verification);
    assert!(!review.enables_recovery);
    assert!(!review.enables_advanced_confirmation);
    assert!(!review.enables_high_risk_approval);
    assert!(!review.allows_hyprland_reload);
    assert!(!review.allows_mutating_hyprctl);
    assert!(!review.allows_script_execution);
    assert!(!review.allows_lua_execution);
    assert!(!review.allows_profile_switching);
    assert!(!review.allows_mode_switching);
    assert!(review.future_meaning.contains("would not enable writes"));
    assert_eq!(SAFE_WRITABLE_ROWS.len(), 341);
}
