use hyprland_settings::one_target_pilot_gate_flip_proposal_review::one_target_pilot_target_scope_review;
use hyprland_settings::write_classification::SAFE_WRITABLE_ROWS;

#[test]
fn proposal_target_scope_is_normal_scalar_only_and_preserves_all_exclusions() {
    let review = one_target_pilot_target_scope_review();

    assert!(review.allowed_target.contains("non-high-risk scalar"));
    assert!(review.one_non_high_risk_scalar_line);
    assert!(review.one_normal_file);
    assert!(review.exact_line_number_known);
    assert!(review.no_ambiguity);
    assert!(review.generated_targets_excluded);
    assert!(review.script_managed_targets_excluded);
    assert!(review.script_referenced_targets_excluded);
    assert!(review.symlink_managed_targets_excluded);
    assert!(review.symlink_targets_excluded);
    assert!(review.high_risk_rows_excluded);
    assert!(review.structured_targets_excluded);
    assert!(review.missing_line_targets_excluded);
    assert!(review.duplicate_ambiguous_targets_excluded);
    assert!(review.unknown_management_state_excluded);
    assert!(review.script_or_lua_required_targets_excluded);
    assert_eq!(SAFE_WRITABLE_ROWS.len(), 341);
}
