use std::fs;

use hyprland_settings::guarded_write_review::PRODUCTION_WRITE_TARGET_REVIEW_ENABLED;
use hyprland_settings::one_target_write_pilot::PRODUCTION_ONE_TARGET_WRITE_PILOT_ENABLED;
use hyprland_settings::write_classification::SAFE_WRITABLE_ROWS;
use hyprland_settings::write_enablement_readiness::PRODUCTION_WRITE_TARGET_SELECTION_READY;
use hyprland_settings::write_review_walkthrough::PRODUCTION_WRITE_REVIEW_WALKTHROUGH_CAN_WRITE;

#[test]
fn all_production_target_selection_gates_remain_false() {
    assert!(!PRODUCTION_WRITE_TARGET_SELECTION_READY);
    assert!(!PRODUCTION_WRITE_TARGET_REVIEW_ENABLED);
    assert!(!PRODUCTION_WRITE_REVIEW_WALKTHROUGH_CAN_WRITE);
    assert!(!PRODUCTION_ONE_TARGET_WRITE_PILOT_ENABLED);
    assert_eq!(SAFE_WRITABLE_ROWS.len(), 341);
}

#[test]
fn write_flow_does_not_import_architecture_pilot_or_fixture_modules() {
    let write_flow =
        fs::read_to_string("src/write_flow.rs").expect("write flow source should read");

    for forbidden in [
        "production_target_selection_architecture",
        "ProductionTargetSelectionArchitecture",
        "minimum_production_target_selection_architecture",
        "one_target_write_pilot",
        "OneTargetWritePilot",
        "one_target_write_pilot_for_candidate",
        "write_review_walkthrough",
        "WriteReviewWalkthrough",
        "guarded_write_review",
        "GuardedWriteTargetReview",
        "write_target_fixture_proof",
        "FixtureTargetWriteProof",
        "prove_fixture_target_write",
        "write_backup_plan",
        "WriteBackupPlan",
        "write_verification_plan",
        "WriteVerificationPlan",
    ] {
        assert!(
            !write_flow.contains(forbidden),
            "production write flow must not import or call {forbidden}"
        );
    }

    assert!(write_flow.contains("pub fn apply_setting_change("));
    assert!(write_flow.contains("apply_scalar_write_plan"));
}
