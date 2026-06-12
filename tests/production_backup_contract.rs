use std::path::PathBuf;

use hyprland_settings::production_backup_contract::{
    production_backup_contract_for_candidate, ProductionBackupFixtureProofStatus,
    PRODUCTION_BACKUP_CONTRACT_ENABLED,
};
use hyprland_settings::write_classification::SAFE_WRITABLE_ROWS;
use hyprland_settings::write_target_candidate::WriteTargetCandidate;

fn candidate() -> WriteTargetCandidate {
    WriteTargetCandidate {
        label: "Main config".to_string(),
        file_path: PathBuf::from("/tmp/hyprland.conf"),
        resolved_path: Some(PathBuf::from("/tmp/hyprland.conf")),
        line_number: Some(7),
        safe: true,
        generated_or_script_managed: false,
        symlink_managed: false,
        requires_advanced_confirmation: false,
        backup_required: true,
        fixture_only: true,
    }
}

#[test]
fn production_backup_contract_represents_exact_backup_requirements_disabled() {
    let contract = production_backup_contract_for_candidate(&candidate(), "20260610T090000Z");

    assert_eq!(
        contract.target_file_path,
        PathBuf::from("/tmp/hyprland.conf")
    );
    assert_eq!(
        contract.resolved_target_path,
        Some(PathBuf::from("/tmp/hyprland.conf"))
    );
    assert_eq!(contract.backup_directory, PathBuf::from("/tmp"));
    assert!(contract.backup_filename_policy.contains("20260610T090000Z"));
    assert!(contract
        .timestamp_policy
        .contains("UTC timestamp supplied by the production write review"));
    assert!(contract.collision_policy.contains("append .1"));
    assert!(contract
        .original_file_metadata_to_record
        .contains(&"original byte length".to_string()));
    assert!(contract
        .backup_file_metadata_to_record
        .contains(&"byte equality with original".to_string()));
    assert!(contract
        .backup_verification_requirement
        .contains("exactly match original bytes"));
    assert_eq!(
        contract.fixture_only_proof_status,
        ProductionBackupFixtureProofStatus::NotRun
    );
    assert!(contract.production_enabled);
    assert!(PRODUCTION_BACKUP_CONTRACT_ENABLED);
    assert!(contract
        .user_facing_lines()
        .iter()
        .any(|line| line == "Backup contract approval is staged; backup creation is still blocked until write execution gates are approved."));
    assert_eq!(SAFE_WRITABLE_ROWS.len(), 341);
}
