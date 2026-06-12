use std::fs;

use hyprland_settings::one_target_pilot_backup_gate_approval::one_target_pilot_backup_gate_approval_state;
use hyprland_settings::write_classification::SAFE_WRITABLE_ROWS;

#[test]
fn backup_gate_approval_does_not_connect_apply_or_backup_activation_helpers() {
    let state = one_target_pilot_backup_gate_approval_state();
    let write_flow =
        fs::read_to_string("src/write_flow.rs").expect("write flow source should read");

    assert!(!state.apply_behavior_changed);
    assert!(!state.production_backup_creation_reachable);
    assert!(!state.user_config_backup_created);
    assert!(!state.writes_enabled);

    for forbidden in [
        "one_target_pilot_backup_gate_approval",
        "backup gate approval",
        "production backup activation",
        "fixture_backup_exact_copy",
        "choose_unique_backup_path",
        "backup_path_policy_for_target",
        "gate flip",
        "verification activation",
        "recovery activation",
        "target selection activation",
        "PRODUCTION_BACKUP_CONTRACT_ENABLED",
    ] {
        assert!(
            !write_flow.contains(forbidden),
            "production write flow must not import or call {forbidden}"
        );
    }

    assert!(write_flow.contains("pub fn apply_setting_change("));
    assert!(write_flow.contains("apply_scalar_write_plan"));
    assert!(write_flow.contains("high_risk_write_policy"));
    assert_eq!(SAFE_WRITABLE_ROWS.len(), 341);
}
