use hyprland_settings::one_target_pilot_nonwriting_prerequisite_batch_approval::one_target_pilot_nonwriting_prerequisite_batch_approval;
use hyprland_settings::write_classification::SAFE_WRITABLE_ROWS;

#[test]
fn batch_approval_represents_three_nonwriting_prerequisite_gate_changes() {
    let approval = one_target_pilot_nonwriting_prerequisite_batch_approval();

    assert_eq!(
        approval.approved_gates,
        vec![
            "PRODUCTION_RECOVERY_CONTRACT_ENABLED",
            "PRODUCTION_WRITE_TARGET_REVIEW_ENABLED",
            "PRODUCTION_WRITE_TARGET_SELECTION_READY"
        ]
    );
    assert_eq!(approval.gates_changed.len(), 3);
    assert!(approval
        .gates_changed
        .iter()
        .all(|gate| !gate.previous_value
            && gate.new_value
            && gate.meaning.contains("prerequisite")));
    assert!(approval
        .pre_existing_approved_gates
        .contains(&"PRODUCTION_ONE_TARGET_PRE_ENABLE_AUDIT_PASSED"));
    assert!(approval
        .pre_existing_approved_gates
        .contains(&"PRODUCTION_BACKUP_CONTRACT_ENABLED"));
    assert!(approval
        .pre_existing_approved_gates
        .contains(&"PRODUCTION_VERIFICATION_CONTRACT_ENABLED"));
    assert!(approval
        .gates_still_false
        .contains(&"PRODUCTION_ONE_TARGET_WRITE_PILOT_ENABLED"));
    assert!(approval
        .gates_still_false
        .contains(&"PRODUCTION_WRITE_REVIEW_WALKTHROUGH_CAN_WRITE"));
    assert!(approval
        .gates_still_false
        .contains(&"PRODUCTION_ADVANCED_CONFIRMATION_ENABLED"));
    assert!(approval
        .gates_still_false
        .contains(&"PRODUCTION_HIGH_RISK_APPROVAL_ENABLED"));
    assert!(!approval.writes_enabled);
    assert!(!approval.apply_writes_enabled);
    assert!(!approval.production_backup_creation_reachable);
    assert!(!approval.production_verification_execution_reachable);
    assert!(!approval.production_recovery_execution_reachable);
    assert!(!approval.user_config_backup_created);
    assert!(!approval.production_verification_run);
    assert!(!approval.production_recovery_run);
    assert!(!approval.real_restore_attempted);
    assert!(!approval.selected_session_config_affects_writes);
    assert!(!approval.selected_session_config_persisted);
    assert!(!approval.real_write_target_selection_active);
    assert!(!approval.real_layered_writes_active);
    assert!(!approval.app_write_model_changed);
    assert_eq!(
        approval.next_recommended_sprint,
        "Manual approval boundary for the first real one-target write pilot."
    );
    assert_eq!(SAFE_WRITABLE_ROWS.len(), 341);
}
