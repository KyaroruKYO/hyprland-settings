use hyprland_settings::one_target_pilot_manual_review::{
    one_target_pilot_gate_inventory_verification, pre_enable_backup_and_verification_gates_are_true,
};
use hyprland_settings::write_classification::SAFE_WRITABLE_ROWS;

#[test]
fn gate_inventory_verification_keeps_pre_enable_backup_and_verification_gates_true() {
    let gates = one_target_pilot_gate_inventory_verification();
    let gate_names = gates.iter().map(|gate| gate.gate_name).collect::<Vec<_>>();

    for expected in [
        "PRODUCTION_ONE_TARGET_PRE_ENABLE_AUDIT_PASSED",
        "PRODUCTION_ONE_TARGET_WRITE_PILOT_ENABLED",
        "PRODUCTION_WRITE_TARGET_SELECTION_READY",
        "PRODUCTION_WRITE_TARGET_REVIEW_ENABLED",
        "PRODUCTION_WRITE_REVIEW_WALKTHROUGH_CAN_WRITE",
        "PRODUCTION_BACKUP_CONTRACT_ENABLED",
        "PRODUCTION_VERIFICATION_CONTRACT_ENABLED",
        "PRODUCTION_RECOVERY_CONTRACT_ENABLED",
        "PRODUCTION_ADVANCED_CONFIRMATION_ENABLED",
        "PRODUCTION_HIGH_RISK_APPROVAL_ENABLED",
    ] {
        assert!(gate_names.contains(&expected), "missing gate: {expected}");
    }

    assert!(pre_enable_backup_and_verification_gates_are_true());
    assert!(gates
        .iter()
        .all(|gate| !gate.required_proof_before_flip.is_empty()
            && !gate.current_blocking_reason.is_empty()));
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
