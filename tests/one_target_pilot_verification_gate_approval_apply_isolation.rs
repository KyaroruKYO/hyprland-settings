use std::fs;

use hyprland_settings::one_target_pilot_manual_review::{
    pre_enable_backup_and_verification_gates_are_true, production_write_path_remains_disabled,
};
use hyprland_settings::write_classification::SAFE_WRITABLE_ROWS;

#[test]
fn write_flow_does_not_import_or_call_verification_gate_approval_or_activation_helpers() {
    let write_flow =
        fs::read_to_string("src/write_flow.rs").expect("write flow source should read");

    for forbidden in [
        "one_target_pilot_verification_gate_approval",
        "verification gate approval",
        "production verification activation",
        "fixture_reread_verify_expected_value",
        "production_verification_contract_for_candidate",
        "planned_reread_verification",
        "gate flip",
        "recovery activation",
        "target selection activation",
        "backup activation",
        "PRODUCTION_VERIFICATION_CONTRACT_ENABLED",
    ] {
        assert!(
            !write_flow.contains(forbidden),
            "production write flow must not import or call {forbidden}"
        );
    }

    assert!(write_flow.contains("pub fn apply_setting_change("));
    assert!(write_flow.contains("apply_scalar_write_plan"));
    assert!(write_flow.contains("high_risk_write_policy"));
    assert!(pre_enable_backup_and_verification_gates_are_true());
    assert!(production_write_path_remains_disabled());
    assert_eq!(SAFE_WRITABLE_ROWS.len(), 341);
}
