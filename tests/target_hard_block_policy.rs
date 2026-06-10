use std::path::PathBuf;

use hyprland_settings::production_advanced_confirmation::{
    hard_block_policy, TargetManagementRiskInput, PRODUCTION_ADVANCED_CONFIRMATION_ENABLED,
};
use hyprland_settings::write_classification::SAFE_WRITABLE_ROWS;

#[test]
fn hard_block_policy_rejects_targets_confirmation_cannot_fix() {
    for mut input in [
        {
            let mut input =
                TargetManagementRiskInput::normal_scalar(PathBuf::from("/tmp/missing.conf"), 1);
            input.line_number = None;
            input
        },
        {
            let mut input =
                TargetManagementRiskInput::normal_scalar(PathBuf::from("/tmp/structured.conf"), 1);
            input.structured_non_scalar_target = true;
            input
        },
        {
            let mut input =
                TargetManagementRiskInput::normal_scalar(PathBuf::from("/tmp/unreadable.conf"), 1);
            input.target_readable = false;
            input
        },
        {
            let mut input =
                TargetManagementRiskInput::normal_scalar(PathBuf::from("/tmp/script.conf"), 1);
            input.requires_script_or_lua_execution = true;
            input
        },
        {
            let mut input =
                TargetManagementRiskInput::normal_scalar(PathBuf::from("/tmp/duplicate.conf"), 1);
            input.duplicate_target_ambiguity = true;
            input
        },
    ] {
        let policy = hard_block_policy(&input);
        assert!(policy.hard_blocked);
        assert!(!policy.advanced_confirmation_can_override);
        assert!(!policy.production_enabled);
        assert!(!policy.reasons.is_empty());
        input.generated_file = true;
        let still_hard_blocked = hard_block_policy(&input);
        assert!(still_hard_blocked.hard_blocked);
        assert!(!still_hard_blocked.advanced_confirmation_can_override);
    }

    assert!(!PRODUCTION_ADVANCED_CONFIRMATION_ENABLED);
    assert_eq!(SAFE_WRITABLE_ROWS.len(), 341);
}
