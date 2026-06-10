use hyprland_settings::production_target_selection_architecture::production_apply_integration_boundary;
use hyprland_settings::write_classification::SAFE_WRITABLE_ROWS;

#[test]
fn apply_integration_boundary_forbids_unsafe_shortcuts() {
    let boundary = production_apply_integration_boundary();

    assert!(!boundary.production_apply_may_call_target_selection_after_all_gates);
    assert!(boundary.production_apply_must_not_call_fixture_proof);
    assert!(boundary.production_apply_must_not_call_walkthrough_directly);
    assert!(boundary.production_apply_must_not_skip_backup);
    assert!(boundary.production_apply_must_not_skip_reread_verification);
    assert!(boundary.production_apply_must_not_bypass_high_risk_policy);

    let lines = boundary.report_lines().join("\n");
    assert!(lines.contains("Production Apply may only call target selection after all gates pass."));
    assert!(lines.contains("Production Apply must not call fixture proof."));
    assert!(lines.contains("Production Apply must not skip backup."));
    assert!(lines.contains("Production Apply must not bypass high-risk policy."));
    assert_eq!(SAFE_WRITABLE_ROWS.len(), 341);
}
