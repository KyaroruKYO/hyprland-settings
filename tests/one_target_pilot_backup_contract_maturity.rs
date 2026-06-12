use hyprland_settings::one_target_pilot_backup_gate_review::one_target_pilot_backup_contract_maturity_review;
use hyprland_settings::production_backup_contract::PRODUCTION_BACKUP_CONTRACT_ENABLED;
use hyprland_settings::write_classification::SAFE_WRITABLE_ROWS;

#[test]
fn backup_contract_maturity_requires_exact_verified_fixture_only_backup() {
    let review = one_target_pilot_backup_contract_maturity_review();

    assert!(review.exact_target_file_backup_required);
    assert!(review.same_directory_policy_represented);
    assert!(review.timestamped_backup_path_represented);
    assert!(review.collision_safe_backup_path_represented);
    assert!(review.byte_equality_proof_represented);
    assert!(review.backup_before_write_required);
    assert!(review.no_write_without_backup_proof_required);
    assert!(review.fixture_only_proof_helpers_reject_non_temp_misuse);
    assert!(review.target_exclusions_preserved_by_backup_contract);
    assert!(!review.backup_contract_implies_write_activation);
    assert!(!review.backup_contract_implies_verification_activation);
    assert!(!review.backup_contract_implies_recovery_activation);
    assert!(!review.user_config_backup_created);
    assert!(PRODUCTION_BACKUP_CONTRACT_ENABLED);
    assert_eq!(SAFE_WRITABLE_ROWS.len(), 341);
}
