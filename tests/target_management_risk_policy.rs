use std::path::PathBuf;

use hyprland_settings::production_advanced_confirmation::{
    classify_target_management_risk, TargetManagementRiskFlag, TargetManagementRiskInput,
    TargetManagementRiskLevel, PRODUCTION_ADVANCED_CONFIRMATION_ENABLED,
};
use hyprland_settings::write_classification::SAFE_WRITABLE_ROWS;

fn target() -> TargetManagementRiskInput {
    TargetManagementRiskInput::normal_scalar(PathBuf::from("/tmp/hyprland.conf"), 4)
}

#[test]
fn risk_classification_represents_safe_and_risky_target_classes() {
    let normal = classify_target_management_risk(&target());
    assert_eq!(
        normal.risk_level,
        TargetManagementRiskLevel::SafeForFirstPilot
    );
    assert!(normal.eligible_for_first_pilot);
    assert!(!normal.production_enabled);

    for (mut input, flag) in [
        {
            let mut input = target();
            input.generated_file = true;
            (input, TargetManagementRiskFlag::GeneratedFile)
        },
        {
            let mut input = target();
            input.script_managed_file = true;
            (input, TargetManagementRiskFlag::ScriptManagedFile)
        },
        {
            let mut input = target();
            input.script_referenced_file = true;
            (input, TargetManagementRiskFlag::ScriptReferencedFile)
        },
        {
            let mut input = target();
            input.symlink_managed_file = true;
            (input, TargetManagementRiskFlag::SymlinkManagedFile)
        },
        {
            let mut input = target();
            input.symlink_target = true;
            (input, TargetManagementRiskFlag::SymlinkTarget)
        },
    ] {
        let classified = classify_target_management_risk(&input);
        assert_eq!(
            classified.risk_level,
            TargetManagementRiskLevel::RequiresAdvancedConfirmationLater
        );
        assert!(classified.risk_flags.contains(&flag));
        assert!(classified.advanced_confirmation_can_help_later);
        assert!(!classified.eligible_for_first_pilot);
        assert!(!classified.production_enabled);
        input.generated_file = false;
    }

    let mut high_risk = target();
    high_risk.high_risk_setting = true;
    let high_risk = classify_target_management_risk(&high_risk);
    assert_eq!(
        high_risk.risk_level,
        TargetManagementRiskLevel::BlockedForFirstPilot
    );

    let mut duplicate = target();
    duplicate.duplicate_target_ambiguity = true;
    let duplicate = classify_target_management_risk(&duplicate);
    assert_eq!(duplicate.risk_level, TargetManagementRiskLevel::HardBlocked);

    assert!(!PRODUCTION_ADVANCED_CONFIRMATION_ENABLED);
    assert_eq!(SAFE_WRITABLE_ROWS.len(), 341);
}
