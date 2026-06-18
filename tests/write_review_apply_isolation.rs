use std::fs;

use hyprland_settings::guarded_write_review::PRODUCTION_WRITE_TARGET_REVIEW_ENABLED;
use hyprland_settings::write_classification::SAFE_WRITABLE_ROWS;

#[test]
fn guarded_write_review_is_explicitly_disabled_for_production() {
    assert!(PRODUCTION_WRITE_TARGET_REVIEW_ENABLED);
    assert_eq!(SAFE_WRITABLE_ROWS.len(), 341);
}

#[test]
fn production_apply_flow_does_not_call_guarded_review_modules() {
    let write_flow =
        fs::read_to_string("src/write_flow.rs").expect("write flow source should read");

    for forbidden in [
        "guarded_write_review",
        "GuardedWriteTargetReview",
        "build_guarded_write_target_review",
        "write_backup_plan",
        "WriteBackupPlan",
        "build_exact_backup_plan",
        "write_advanced_confirmation",
        "WriteAdvancedConfirmation",
        "write_verification_plan",
        "WriteVerificationPlan",
        "planned_reread_verification",
        "write_target_fixture_proof",
        "FixtureTargetWriteProof",
        "FixtureTargetWriteProofRequest",
        "prove_fixture_target_write",
        "PRODUCTION_WRITE_TARGET_REVIEW_ENABLED",
    ] {
        assert!(
            !write_flow.contains(forbidden),
            "production write flow must not import or call guarded review module: {forbidden}"
        );
    }

    assert!(write_flow.contains("pub fn apply_setting_change("));
    assert!(write_flow.contains("apply_scalar_write_plan"));
}
