use hyprland_settings::one_target_pilot_verification_gate_approval::{
    one_target_pilot_verification_gate_approval_state, one_target_pilot_verification_gate_meaning,
    one_target_pilot_verification_gate_readiness_change,
};
use hyprland_settings::production_backup_contract::PRODUCTION_BACKUP_CONTRACT_ENABLED;
use hyprland_settings::production_verification_contract::PRODUCTION_VERIFICATION_CONTRACT_ENABLED;
use hyprland_settings::write_classification::SAFE_WRITABLE_ROWS;

#[test]
fn verification_gate_approval_represents_single_gate_change_without_execution() {
    let state = one_target_pilot_verification_gate_approval_state();
    let readiness = one_target_pilot_verification_gate_readiness_change();
    let meaning = one_target_pilot_verification_gate_meaning();

    assert_eq!(
        state.approved_gate,
        "PRODUCTION_VERIFICATION_CONTRACT_ENABLED"
    );
    assert!(!state.previous_value);
    assert!(state.new_value);
    assert!(state.pre_enable_audit_remains_true);
    assert!(state.backup_contract_remains_true);
    assert!(state.verification_contract_gate_approved);
    assert!(!state.production_backup_creation_reachable);
    assert!(!state.production_verification_execution_reachable);
    assert!(!state.writes_enabled);
    assert!(!state.apply_behavior_changed);
    assert!(!state.user_config_backup_created);
    assert!(!state.production_verification_run);
    assert!(!state.real_restore_attempted);
    assert!(!state.selected_session_config_affects_writes);
    assert!(!state.selected_session_config_persisted);
    assert!(!state.real_write_target_selection_active);
    assert!(!state.real_layered_writes_active);
    assert!(!state.app_write_model_changed);

    assert!(PRODUCTION_BACKUP_CONTRACT_ENABLED);
    assert!(PRODUCTION_VERIFICATION_CONTRACT_ENABLED);
    assert!(readiness.verification_contract_allowed_as_prerequisite);
    assert!(!readiness.production_backup_creation_reachable);
    assert!(!readiness.production_verification_execution_reachable);
    assert_eq!(
        readiness.next_recommended_gate,
        "PRODUCTION_RECOVERY_CONTRACT_ENABLED"
    );
    assert!(meaning.meaning.contains("prerequisite"));
    for forbidden_meaning in [
        "writes are enabled",
        "Apply can write",
        "real verification is run",
        "real backups are created",
        "production backup creation is reachable",
        "recovery is active",
        "target selection is active",
    ] {
        assert!(meaning.non_meanings.contains(&forbidden_meaning));
    }
    assert_eq!(SAFE_WRITABLE_ROWS.len(), 341);
}
