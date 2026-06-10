use std::path::PathBuf;

use hyprland_settings::one_target_write_pilot::PRODUCTION_ONE_TARGET_WRITE_PILOT_ENABLED;
use hyprland_settings::production_advanced_confirmation::{
    first_pilot_exclusion_policy, TargetManagementRiskInput,
};
use hyprland_settings::write_classification::SAFE_WRITABLE_ROWS;

fn normal() -> TargetManagementRiskInput {
    TargetManagementRiskInput::normal_scalar(PathBuf::from("/tmp/hyprland.conf"), 3)
}

#[test]
fn first_pilot_excludes_all_risky_target_classes() {
    let normal_policy = first_pilot_exclusion_policy(&normal());
    assert!(normal_policy.eligible_for_first_pilot);
    assert!(!normal_policy.production_gate_enabled);

    let risky_inputs = [
        {
            let mut input = normal();
            input.generated_file = true;
            input
        },
        {
            let mut input = normal();
            input.script_managed_file = true;
            input
        },
        {
            let mut input = normal();
            input.script_referenced_file = true;
            input
        },
        {
            let mut input = normal();
            input.symlink_managed_file = true;
            input
        },
        {
            let mut input = normal();
            input.symlink_target = true;
            input
        },
        {
            let mut input = normal();
            input.high_risk_setting = true;
            input
        },
        {
            let mut input = normal();
            input.structured_non_scalar_target = true;
            input
        },
        {
            let mut input = normal();
            input.line_number = None;
            input
        },
        {
            let mut input = normal();
            input.duplicate_target_ambiguity = true;
            input
        },
    ];

    for input in risky_inputs {
        let policy = first_pilot_exclusion_policy(&input);
        assert!(!policy.eligible_for_first_pilot);
        assert!(!policy.excluded_reasons.is_empty());
        assert!(!policy.production_gate_enabled);
    }

    assert!(!PRODUCTION_ONE_TARGET_WRITE_PILOT_ENABLED);
    assert_eq!(SAFE_WRITABLE_ROWS.len(), 341);
}
