use std::path::PathBuf;

use hyprland_settings::production_advanced_confirmation::TargetManagementRiskInput;
use hyprland_settings::production_high_risk_approval::{
    high_risk_approval_boundary, HighRiskApprovalCategory,
};
use hyprland_settings::write_classification::SAFE_WRITABLE_ROWS;

fn target() -> TargetManagementRiskInput {
    TargetManagementRiskInput::normal_scalar(PathBuf::from("/tmp/hyprland.conf"), 4)
}

#[test]
fn high_risk_classification_categories_and_hard_blocks_are_represented() {
    let not_high_risk = high_risk_approval_boundary(
        "windows.snap.enabled",
        "general:snap:enabled",
        Some(&target()),
        false,
    );
    assert_eq!(
        not_high_risk.risk_category,
        HighRiskApprovalCategory::NotHighRisk
    );

    let approvable_later = high_risk_approval_boundary(
        "ecosystem.no_update_news",
        "ecosystem:no_update_news",
        Some(&target()),
        false,
    );
    assert_eq!(
        approvable_later.risk_category,
        HighRiskApprovalCategory::HighRiskApprovableLater
    );

    let separate_policy = high_risk_approval_boundary(
        "debug.manual_crash",
        "debug:manual_crash",
        Some(&target()),
        false,
    );
    assert_eq!(
        separate_policy.risk_category,
        HighRiskApprovalCategory::HighRiskRequiresSeparatePolicy
    );

    let mut missing_line = target();
    missing_line.line_number = None;
    let hard_blocked = high_risk_approval_boundary(
        "debug.manual_crash",
        "debug:manual_crash",
        Some(&missing_line),
        false,
    );
    assert_eq!(
        hard_blocked.risk_category,
        HighRiskApprovalCategory::HardBlockedHighRisk
    );

    let unknown = high_risk_approval_boundary("unknown.row", "unknown:row", None, true);
    assert_eq!(
        unknown.risk_category,
        HighRiskApprovalCategory::UnknownHighRiskStatus
    );
    assert!(!unknown.first_pilot_eligible);
    assert_eq!(SAFE_WRITABLE_ROWS.len(), 341);
}
