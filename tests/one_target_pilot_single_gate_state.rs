use hyprland_settings::one_target_pilot_manual_review::{
    all_write_execution_gates_remain_false, pre_enable_and_backup_gates_are_true,
};
use hyprland_settings::one_target_pilot_pre_enable_audit::one_target_pilot_gate_inventory_snapshot;
use hyprland_settings::write_classification::SAFE_WRITABLE_ROWS;

#[test]
fn gate_inventory_has_two_true_prerequisite_gates_and_all_write_execution_gates_false() {
    let gates = one_target_pilot_gate_inventory_snapshot();

    assert!(pre_enable_and_backup_gates_are_true());
    assert!(all_write_execution_gates_remain_false());
    assert_eq!(
        gates.iter().filter(|gate| gate.current_value).count(),
        2,
        "only the pre-enable audit and backup gates may be true"
    );
    assert!(gates.iter().any(|gate| gate.gate_name
        == "PRODUCTION_ONE_TARGET_PRE_ENABLE_AUDIT_PASSED"
        && gate.current_value));
    assert!(gates
        .iter()
        .any(|gate| gate.gate_name == "PRODUCTION_BACKUP_CONTRACT_ENABLED" && gate.current_value));
    assert!(gates
        .iter()
        .filter(|gate| !matches!(
            gate.gate_name,
            "PRODUCTION_ONE_TARGET_PRE_ENABLE_AUDIT_PASSED" | "PRODUCTION_BACKUP_CONTRACT_ENABLED"
        ))
        .all(|gate| !gate.current_value));
    assert!(gates.iter().all(|gate| !gate.would_allow.is_empty()
        && !gate.required_proof_before_flip.is_empty()
        && !gate.current_blocking_reason.is_empty()));
    assert_eq!(SAFE_WRITABLE_ROWS.len(), 341);
}
