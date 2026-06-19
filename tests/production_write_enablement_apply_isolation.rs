use std::fs;

use hyprland_settings::guarded_write_review::PRODUCTION_WRITE_TARGET_REVIEW_ENABLED;
use hyprland_settings::write_classification::SAFE_WRITABLE_ROWS;
use hyprland_settings::write_enablement_readiness::PRODUCTION_WRITE_TARGET_SELECTION_READY;
use hyprland_settings::write_review_walkthrough::PRODUCTION_WRITE_REVIEW_WALKTHROUGH_CAN_WRITE;

#[test]
fn all_production_write_target_enablement_gates_remain_false() {
    assert!(PRODUCTION_WRITE_TARGET_REVIEW_ENABLED);
    assert!(PRODUCTION_WRITE_REVIEW_WALKTHROUGH_CAN_WRITE);
    assert!(PRODUCTION_WRITE_TARGET_SELECTION_READY);
    assert_eq!(SAFE_WRITABLE_ROWS.len(), 341);
}

#[test]
fn write_flow_does_not_import_or_call_enablement_or_walkthrough_modules() {
    let write_flow =
        fs::read_to_string("src/write_flow.rs").expect("write flow source should read");

    for forbidden in [
        "write_enablement_readiness",
        "ProductionWriteEnablementReadiness",
        "current_production_write_enablement_readiness",
        "write_review_walkthrough",
        "WriteReviewWalkthrough",
        "build_write_review_walkthrough",
        "guarded_write_review",
        "GuardedWriteTargetReview",
        "build_guarded_write_target_review",
        "write_backup_plan",
        "WriteBackupPlan",
        "build_exact_backup_plan",
        "write_verification_plan",
        "WriteVerificationPlan",
        "planned_reread_verification",
        "write_target_fixture_proof",
        "FixtureTargetWriteProof",
        "prove_fixture_target_write",
    ] {
        assert!(
            !write_flow.contains(forbidden),
            "production write flow must not import or call {forbidden}"
        );
    }

    assert!(write_flow.contains("pub fn apply_setting_change("));
    assert!(write_flow.contains("apply_scalar_write_plan"));
}
