use std::path::PathBuf;

use hyprland_settings::production_advanced_confirmation::TargetManagementRiskInput;
use hyprland_settings::production_high_risk_approval::{
    high_risk_approval_boundary, HighRiskApprovalCategory, PRODUCTION_HIGH_RISK_APPROVAL_ENABLED,
};
use hyprland_settings::write_classification::SAFE_WRITABLE_ROWS;

fn normal_target() -> TargetManagementRiskInput {
    TargetManagementRiskInput::normal_scalar(PathBuf::from("/tmp/hyprland.conf"), 12)
}

#[test]
fn high_risk_approval_boundary_represents_normal_and_high_risk_rows() {
    let normal = high_risk_approval_boundary(
        "windows.snap.enabled",
        "general:snap:enabled",
        Some(&normal_target()),
        false,
    );
    assert_eq!(normal.risk_category, HighRiskApprovalCategory::NotHighRisk);
    assert!(!normal.approval_required);
    assert!(normal.first_pilot_eligible);
    assert!(!normal.production_enabled);
    assert!(normal.fixture_only);

    let high_risk = high_risk_approval_boundary(
        "render.direct_scanout",
        "render:direct_scanout",
        Some(&normal_target()),
        false,
    );
    assert_eq!(
        high_risk.risk_category,
        HighRiskApprovalCategory::HighRiskRequiresSeparatePolicy
    );
    assert!(high_risk.policy.is_some());
    assert!(high_risk.approval_required);
    assert!(high_risk.approval_currently_unavailable);
    assert!(!high_risk.first_pilot_eligible);

    assert!(!PRODUCTION_HIGH_RISK_APPROVAL_ENABLED);
    assert_eq!(SAFE_WRITABLE_ROWS.len(), 341);
}
