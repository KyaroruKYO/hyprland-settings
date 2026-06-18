use std::fs;

use hyprland_settings::one_target_pilot_backup_gate_review::{
    backup_gate_candidate_current_staged_state_is_preserved,
    one_target_pilot_backup_gate_inventory_verification,
};
use hyprland_settings::write_classification::SAFE_WRITABLE_ROWS;

#[test]
fn backup_gate_review_preserves_current_staged_gate_inventory() {
    let gates = one_target_pilot_backup_gate_inventory_verification();

    assert!(backup_gate_candidate_current_staged_state_is_preserved());
    assert_eq!(
        gates.iter().filter(|gate| gate.current_value).count(),
        3,
        "only the pre-enable audit, backup, and verification gates should be true"
    );
    assert!(gates.iter().any(|gate| gate.gate_name
        == "PRODUCTION_ONE_TARGET_PRE_ENABLE_AUDIT_PASSED"
        && gate.current_value));
    assert!(gates
        .iter()
        .any(|gate| gate.gate_name == "PRODUCTION_BACKUP_CONTRACT_ENABLED" && gate.current_value));
    assert!(gates.iter().any(
        |gate| gate.gate_name == "PRODUCTION_VERIFICATION_CONTRACT_ENABLED" && gate.current_value
    ));
    assert!(gates
        .iter()
        .filter(|gate| !matches!(
            gate.gate_name,
            "PRODUCTION_ONE_TARGET_PRE_ENABLE_AUDIT_PASSED"
                | "PRODUCTION_BACKUP_CONTRACT_ENABLED"
                | "PRODUCTION_VERIFICATION_CONTRACT_ENABLED"
        ))
        .all(|gate| !gate.current_value));
    assert_eq!(SAFE_WRITABLE_ROWS.len(), 341);
}

#[test]
fn write_flow_does_not_import_or_call_backup_gate_review_or_activation_helpers() {
    let write_flow =
        fs::read_to_string("src/write_flow.rs").expect("write flow source should read");

    for forbidden in [
        "one_target_pilot_backup_gate_review",
        "one_target_pilot_backup_gate_candidate_review",
        "one_target_pilot_backup_contract_maturity_review",
        "one_target_pilot_backup_safety_boundary_review",
        "one_target_pilot_future_backup_gate_approval_scope",
        "one_target_pilot_backup_gate_remaining_blockers",
        "backup gate review",
        "backup gate candidate",
        "backup activation",
        "fixture_backup_exact_copy",
        "production backup contract activation",
        "gate flip",
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
