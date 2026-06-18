use std::fs;

use hyprland_settings::guarded_write_review::PRODUCTION_WRITE_TARGET_REVIEW_ENABLED;
use hyprland_settings::one_target_write_pilot::PRODUCTION_ONE_TARGET_WRITE_PILOT_ENABLED;
use hyprland_settings::production_backup_contract::PRODUCTION_BACKUP_CONTRACT_ENABLED;
use hyprland_settings::production_recovery_contract::PRODUCTION_RECOVERY_CONTRACT_ENABLED;
use hyprland_settings::production_verification_contract::PRODUCTION_VERIFICATION_CONTRACT_ENABLED;
use hyprland_settings::write_classification::SAFE_WRITABLE_ROWS;
use hyprland_settings::write_enablement_readiness::PRODUCTION_WRITE_TARGET_SELECTION_READY;
use hyprland_settings::write_review_walkthrough::PRODUCTION_WRITE_REVIEW_WALKTHROUGH_CAN_WRITE;

#[test]
fn all_recovery_related_production_gates_remain_false() {
    assert!(PRODUCTION_RECOVERY_CONTRACT_ENABLED);
    assert!(PRODUCTION_BACKUP_CONTRACT_ENABLED);
    assert!(PRODUCTION_VERIFICATION_CONTRACT_ENABLED);
    assert!(!PRODUCTION_ONE_TARGET_WRITE_PILOT_ENABLED);
    assert!(PRODUCTION_WRITE_TARGET_SELECTION_READY);
    assert!(PRODUCTION_WRITE_TARGET_REVIEW_ENABLED);
    assert!(!PRODUCTION_WRITE_REVIEW_WALKTHROUGH_CAN_WRITE);
    assert_eq!(SAFE_WRITABLE_ROWS.len(), 341);
}

#[test]
fn write_flow_does_not_import_or_call_recovery_contracts_or_fixture_helpers() {
    let write_flow =
        fs::read_to_string("src/write_flow.rs").expect("write flow source should read");

    for forbidden in [
        "production_recovery_contract",
        "RecoveryRestoreOperation",
        "RecoveryReport",
        "fixture_restore_backup_bytes",
        "fixture_verify_restored_file",
        "production_backup_contract",
        "production_verification_contract",
        "write_target_fixture_proof",
        "fixture_backup_exact_copy",
        "fixture_reread_verify_expected_value",
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
