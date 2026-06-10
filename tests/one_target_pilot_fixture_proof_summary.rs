use hyprland_settings::one_target_pilot_pre_enable_audit::one_target_pilot_fixture_proof_summary;
use hyprland_settings::write_classification::SAFE_WRITABLE_ROWS;

#[test]
fn fixture_proof_summary_marks_path_fixture_proven_but_not_production_enabled() {
    let summary = one_target_pilot_fixture_proof_summary();

    assert!(summary.fixture_target_path_only);
    assert!(summary.normal_scalar_target_proof_exists);
    assert!(summary.backup_exact_copy_proof_exists);
    assert!(summary.backup_collision_proof_exists);
    assert!(summary.fixture_write_proof_exists);
    assert!(summary.reread_verification_proof_exists);
    assert!(summary.verification_failure_proof_exists);
    assert!(summary.recovery_restore_proof_exists);
    assert!(summary.restore_verification_proof_exists);
    assert!(summary.advanced_confirmation_exclusion_proof_exists);
    assert!(summary.high_risk_exclusion_proof_exists);
    assert!(!summary.real_user_config_touched);
    assert!(summary.fixture_path_is_proven_but_production_disabled());
    assert_eq!(SAFE_WRITABLE_ROWS.len(), 341);
}
