use hyprland_settings::production_recovery_contract::{
    production_recovery_prerequisite_contract, PRODUCTION_RECOVERY_CONTRACT_ENABLED,
};
use hyprland_settings::write_classification::SAFE_WRITABLE_ROWS;

#[test]
fn recovery_prerequisite_contract_requires_restore_without_reload_and_is_approved_nonexecuting() {
    let contract = production_recovery_prerequisite_contract();

    assert!(contract.backup_exists_required);
    assert!(contract.backup_verified_before_write_required);
    assert!(contract.restore_exact_backup_bytes_required);
    assert!(contract.reread_restored_file_required);
    assert!(contract.report_rollback_success_or_failure_required);
    assert!(!contract.hyprland_reload_allowed);
    assert!(contract.production_enabled);
    assert!(PRODUCTION_RECOVERY_CONTRACT_ENABLED);
    assert!(contract
        .user_facing_lines()
        .iter()
        .any(|line| line == "This pilot must never reload Hyprland automatically."));
    assert_eq!(SAFE_WRITABLE_ROWS.len(), 341);
}
