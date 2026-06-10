use std::path::PathBuf;

use hyprland_settings::production_verification_contract::{
    all_production_verification_statuses, production_verification_contract_for_candidate,
    ProductionVerificationStatus, PRODUCTION_VERIFICATION_CONTRACT_ENABLED,
};
use hyprland_settings::write_classification::SAFE_WRITABLE_ROWS;
use hyprland_settings::write_target_candidate::WriteTargetCandidate;

fn candidate() -> WriteTargetCandidate {
    WriteTargetCandidate {
        label: "Main config".to_string(),
        file_path: PathBuf::from("/tmp/hyprland.conf"),
        resolved_path: None,
        line_number: Some(5),
        safe: true,
        generated_or_script_managed: false,
        symlink_managed: false,
        requires_advanced_confirmation: false,
        backup_required: true,
        fixture_only: true,
    }
}

#[test]
fn production_verification_contract_represents_reread_requirements_disabled() {
    let contract =
        production_verification_contract_for_candidate(&candidate(), "general.layout", "master");

    assert_eq!(
        contract.target_file_path,
        PathBuf::from("/tmp/hyprland.conf")
    );
    assert_eq!(contract.setting_id, "general.layout");
    assert_eq!(contract.expected_value, "master");
    assert_eq!(contract.expected_line_number, Some(5));
    assert!(contract
        .reread_parser_method
        .contains("reread exact target file"));
    assert!(contract
        .normalization_policy
        .contains("existing Hyprland scalar"));
    assert!(contract
        .failure_reasons
        .contains(&"observed value did not match expected value".to_string()));
    assert_eq!(
        contract.fixture_only_proof_status,
        ProductionVerificationStatus::NotRun
    );
    assert!(!contract.production_enabled);
    assert!(!PRODUCTION_VERIFICATION_CONTRACT_ENABLED);
    assert!(contract
        .user_facing_lines()
        .iter()
        .any(|line| line == "Production verification is not active yet."));

    let statuses = all_production_verification_statuses();
    assert!(statuses.contains(&ProductionVerificationStatus::NotRun));
    assert!(statuses.contains(&ProductionVerificationStatus::Planned));
    assert!(statuses.contains(&ProductionVerificationStatus::PassedInFixture));
    assert!(statuses.contains(&ProductionVerificationStatus::FailedInFixture));
    assert!(statuses.contains(&ProductionVerificationStatus::ProductionDisabled));
    assert!(statuses.contains(&ProductionVerificationStatus::WouldRequireRollback));
    assert_eq!(SAFE_WRITABLE_ROWS.len(), 341);
}
