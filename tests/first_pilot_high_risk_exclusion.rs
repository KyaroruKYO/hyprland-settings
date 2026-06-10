use std::path::PathBuf;

use hyprland_settings::one_target_write_pilot::PRODUCTION_ONE_TARGET_WRITE_PILOT_ENABLED;
use hyprland_settings::production_advanced_confirmation::TargetManagementRiskInput;
use hyprland_settings::production_high_risk_approval::first_pilot_high_risk_exclusion;
use hyprland_settings::write_classification::SAFE_WRITABLE_ROWS;

fn target() -> TargetManagementRiskInput {
    TargetManagementRiskInput::normal_scalar(PathBuf::from("/tmp/hyprland.conf"), 2)
}

#[test]
fn first_pilot_excludes_high_risk_and_hard_blocked_targets() {
    let normal = first_pilot_high_risk_exclusion("windows.snap.enabled", &target());
    assert!(normal.normal_non_high_risk_scalar_eligible);
    assert!(!normal.production_gate_enabled);

    let high_risk = first_pilot_high_risk_exclusion("render.direct_scanout", &target());
    assert!(high_risk.high_risk_excluded);
    assert!(!high_risk.normal_non_high_risk_scalar_eligible);

    for mut risky in [
        {
            let mut target = target();
            target.generated_file = true;
            target
        },
        {
            let mut target = target();
            target.script_managed_file = true;
            target
        },
        {
            let mut target = target();
            target.symlink_target = true;
            target
        },
        {
            let mut target = target();
            target.line_number = None;
            target
        },
        {
            let mut target = target();
            target.structured_non_scalar_target = true;
            target
        },
        {
            let mut target = target();
            target.duplicate_target_ambiguity = true;
            target
        },
    ] {
        risky.high_risk_setting = true;
        let policy = first_pilot_high_risk_exclusion("debug.manual_crash", &risky);
        assert!(policy.high_risk_excluded);
        assert!(!policy.normal_non_high_risk_scalar_eligible);
        if risky.line_number.is_none()
            || risky.structured_non_scalar_target
            || risky.duplicate_target_ambiguity
        {
            assert!(policy.hard_blocked);
            assert!(policy
                .reasons
                .iter()
                .any(|reason| reason == "High-risk approval cannot override this block."));
        }
    }

    assert!(!PRODUCTION_ONE_TARGET_WRITE_PILOT_ENABLED);
    assert_eq!(SAFE_WRITABLE_ROWS.len(), 341);
}
