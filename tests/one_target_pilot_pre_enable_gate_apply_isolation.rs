use std::fs;

use hyprland_settings::one_target_pilot_manual_review::{
    pre_enable_and_backup_gates_are_true, production_write_path_remains_disabled,
};
use hyprland_settings::write_classification::SAFE_WRITABLE_ROWS;

#[test]
fn write_flow_does_not_import_gate_approval_or_activation_helpers() {
    let write_flow =
        fs::read_to_string("src/write_flow.rs").expect("write flow source should read");

    for forbidden in [
        "one_target_pilot_gate_flip_proposal_review",
        "one_target_pilot_focused_visual_smoke",
        "one_target_pilot_pre_enable_audit",
        "one_target_pilot_proposal_decision_review",
        "gate approval",
        "gate flip",
        "proposal draft",
        "backup contract activation",
        "verification contract activation",
        "recovery contract activation",
        "target selection activation",
        "PRODUCTION_ONE_TARGET_PRE_ENABLE_AUDIT_PASSED",
        "PRODUCTION_BACKUP_CONTRACT_ENABLED = true",
        "PRODUCTION_VERIFICATION_CONTRACT_ENABLED = true",
        "PRODUCTION_RECOVERY_CONTRACT_ENABLED = true",
        "PRODUCTION_WRITE_TARGET_SELECTION_READY = true",
    ] {
        assert!(
            !write_flow.contains(forbidden),
            "production write flow must not import or call {forbidden}"
        );
    }

    assert!(write_flow.contains("pub fn apply_setting_change("));
    assert!(write_flow.contains("apply_scalar_write_plan"));
    assert!(write_flow.contains("high_risk_write_policy"));
    assert!(pre_enable_and_backup_gates_are_true());
    assert!(production_write_path_remains_disabled());
    assert_eq!(SAFE_WRITABLE_ROWS.len(), 341);
}
