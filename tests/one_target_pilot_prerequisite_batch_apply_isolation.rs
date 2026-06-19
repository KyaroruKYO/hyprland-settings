use std::fs;

use hyprland_settings::one_target_pilot_manual_review::{
    all_write_execution_gates_remain_false, nonwriting_prerequisite_gates_are_true,
};
use hyprland_settings::write_classification::SAFE_WRITABLE_ROWS;

#[test]
fn write_flow_does_not_import_or_call_batch_gate_or_activation_helpers() {
    let write_flow =
        fs::read_to_string("src/write_flow.rs").expect("write flow source should read");

    for forbidden in [
        "one_target_pilot_nonwriting_prerequisite_batch_approval",
        "production recovery activation",
        "target review activation",
        "target selection execution",
        "one-target pilot activation",
        "walkthrough write activation",
        "advanced confirmation activation",
        "high-risk approval activation",
        "production backup activation",
        "production verification activation",
        "fixture_backup_exact_copy",
        "fixture_reread_verify_expected_value",
        "fixture_rollback_recovery",
        "gate flip",
        "PRODUCTION_RECOVERY_CONTRACT_ENABLED",
        "PRODUCTION_WRITE_TARGET_REVIEW_ENABLED",
        "PRODUCTION_WRITE_TARGET_SELECTION_READY",
    ] {
        assert!(
            !write_flow.contains(forbidden),
            "production write flow must not import or call {forbidden}"
        );
    }

    assert!(write_flow.contains("pub fn apply_setting_change("));
    assert!(write_flow.contains("apply_scalar_write_plan"));
    assert!(write_flow.contains("high_risk_write_policy"));
    assert!(nonwriting_prerequisite_gates_are_true());
    assert!(all_write_execution_gates_remain_false());
    assert_eq!(SAFE_WRITABLE_ROWS.len(), 341);
}
