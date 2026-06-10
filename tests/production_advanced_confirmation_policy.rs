use std::path::PathBuf;

use hyprland_settings::production_advanced_confirmation::{
    production_advanced_confirmation_policy, TargetManagementRiskInput,
    PRODUCTION_ADVANCED_CONFIRMATION_ENABLED,
};
use hyprland_settings::write_classification::SAFE_WRITABLE_ROWS;

#[test]
fn production_advanced_confirmation_policy_is_unavailable_and_explanatory() {
    let mut input =
        TargetManagementRiskInput::normal_scalar(PathBuf::from("/tmp/generated.conf"), 7);
    input.generated_file = true;
    input.script_managed_file = true;
    input.symlink_managed_file = true;

    let policy = production_advanced_confirmation_policy(&input);

    assert!(policy.confirmation_required);
    assert!(policy.confirmation_unavailable_in_production);
    assert!(policy.fixture_only);
    assert!(!policy.production_enabled);
    assert!(!PRODUCTION_ADVANCED_CONFIRMATION_ENABLED);
    for copy in [
        "I understand this file may be changed by scripts.",
        "I understand this file appears to be generated.",
        "I understand this file is symlink-managed.",
        "I understand writing here may be overwritten outside the app.",
        "I understand the app must back up, write, reread, verify, and recover safely.",
    ] {
        assert!(policy.acknowledgement_text.iter().any(|line| line == copy));
    }
    assert!(policy
        .user_facing_lines()
        .iter()
        .any(|line| line == "Advanced confirmation is not active yet."));
    assert_eq!(SAFE_WRITABLE_ROWS.len(), 341);
}
