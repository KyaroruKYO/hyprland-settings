use std::path::PathBuf;

use hyprland_settings::write_classification::SAFE_WRITABLE_ROWS;
use hyprland_settings::write_target_candidate::WriteTargetCandidate;
use hyprland_settings::write_verification_plan::{
    fixture_verification_passed, planned_reread_verification, WriteVerificationStatus,
};

fn candidate() -> WriteTargetCandidate {
    WriteTargetCandidate {
        label: "Current profile".to_string(),
        file_path: PathBuf::from("/tmp/current.conf"),
        resolved_path: None,
        line_number: Some(4),
        safe: true,
        generated_or_script_managed: false,
        symlink_managed: false,
        requires_advanced_confirmation: false,
        backup_required: true,
        fixture_only: true,
    }
}

#[test]
fn reread_verification_plan_represents_expected_value_and_disabled_production() {
    let plan = planned_reread_verification(&candidate(), "general.layout", "master");

    assert_eq!(plan.target_file_path, PathBuf::from("/tmp/current.conf"));
    assert_eq!(plan.setting_id, "general.layout");
    assert_eq!(plan.expected_value, "master");
    assert_eq!(
        plan.verification_status,
        WriteVerificationStatus::ProductionDisabled
    );
    assert!(plan.fixture_only);
    assert!(plan.production_disabled);
    assert!(plan
        .user_facing_lines()
        .iter()
        .any(|line| line == "The app will reread the file to confirm the value."));

    let passed = fixture_verification_passed(&plan);
    assert_eq!(
        passed.verification_status,
        WriteVerificationStatus::PassedInFixture
    );
    assert_eq!(SAFE_WRITABLE_ROWS.len(), 341);
}

#[test]
fn all_verification_statuses_are_represented() {
    let statuses = [
        WriteVerificationStatus::NotRun,
        WriteVerificationStatus::Planned,
        WriteVerificationStatus::PassedInFixture,
        WriteVerificationStatus::FailedInFixture,
        WriteVerificationStatus::ProductionDisabled,
    ];

    assert_eq!(statuses.len(), 5);
    assert!(statuses.iter().any(|status| status.label() == "planned"));
}
