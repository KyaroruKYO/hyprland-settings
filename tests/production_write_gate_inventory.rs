use hyprland_settings::guarded_write_review::PRODUCTION_WRITE_TARGET_REVIEW_ENABLED;
use hyprland_settings::one_target_write_pilot::PRODUCTION_ONE_TARGET_WRITE_PILOT_ENABLED;
use hyprland_settings::production_target_selection_architecture::production_write_gate_inventory;
use hyprland_settings::write_classification::SAFE_WRITABLE_ROWS;
use hyprland_settings::write_enablement_readiness::PRODUCTION_WRITE_TARGET_SELECTION_READY;
use hyprland_settings::write_review_walkthrough::PRODUCTION_WRITE_REVIEW_WALKTHROUGH_CAN_WRITE;

#[test]
fn code_gate_inventory_lists_safe_batch_gate_true_and_unsafe_gates_false() {
    assert!(PRODUCTION_WRITE_TARGET_SELECTION_READY);
    assert!(PRODUCTION_WRITE_TARGET_REVIEW_ENABLED);
    assert!(PRODUCTION_WRITE_REVIEW_WALKTHROUGH_CAN_WRITE);
    assert!(!PRODUCTION_ONE_TARGET_WRITE_PILOT_ENABLED);

    let inventory = production_write_gate_inventory();
    let gate_names = inventory
        .iter()
        .map(|gate| gate.gate_name)
        .collect::<Vec<_>>();
    for gate in [
        "PRODUCTION_WRITE_TARGET_SELECTION_READY",
        "PRODUCTION_WRITE_TARGET_REVIEW_ENABLED",
        "PRODUCTION_WRITE_REVIEW_WALKTHROUGH_CAN_WRITE",
        "PRODUCTION_ONE_TARGET_WRITE_PILOT_ENABLED",
    ] {
        assert!(
            gate_names.contains(&gate),
            "missing gate inventory item: {gate}"
        );
    }
    assert!(inventory.iter().any(|gate| gate.gate_name
        == "PRODUCTION_WRITE_TARGET_SELECTION_READY"
        && gate.current_value
        && !gate.must_remain_false_now));
    assert!(inventory.iter().any(|gate| gate.gate_name
        == "PRODUCTION_WRITE_TARGET_REVIEW_ENABLED"
        && gate.current_value
        && !gate.must_remain_false_now));
    assert!(inventory.iter().any(|gate| gate.gate_name
        == "PRODUCTION_WRITE_REVIEW_WALKTHROUGH_CAN_WRITE"
        && gate.current_value
        && !gate.must_remain_false_now
        && gate.would_allow.contains("safe-batch")));
    assert!(inventory
        .iter()
        .filter(|gate| !matches!(
            gate.gate_name,
            "PRODUCTION_WRITE_TARGET_SELECTION_READY"
                | "PRODUCTION_WRITE_TARGET_REVIEW_ENABLED"
                | "PRODUCTION_WRITE_REVIEW_WALKTHROUGH_CAN_WRITE"
        ))
        .all(|gate| !gate.current_value && gate.must_remain_false_now));
    assert!(inventory
        .iter()
        .all(|gate| !gate.would_allow.is_empty() && !gate.required_proof_before_flip.is_empty()));
    assert_eq!(SAFE_WRITABLE_ROWS.len(), 341);
}
