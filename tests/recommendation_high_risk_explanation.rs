use std::path::PathBuf;

use hyprland_settings::production_advanced_confirmation::recommendation_risk_explanation_for_candidate;
use hyprland_settings::write_classification::SAFE_WRITABLE_ROWS;
use hyprland_settings::write_target_candidate::WriteTargetCandidate;

fn candidate(line_number: Option<usize>) -> WriteTargetCandidate {
    WriteTargetCandidate {
        label: "High-risk target".to_string(),
        file_path: PathBuf::from("/tmp/high-risk.conf"),
        resolved_path: None,
        line_number,
        safe: line_number.is_some(),
        generated_or_script_managed: false,
        symlink_managed: false,
        requires_advanced_confirmation: false,
        backup_required: true,
        fixture_only: true,
    }
}

#[test]
fn recommendation_risk_explanation_includes_high_risk_inactive_copy() {
    let high_risk = recommendation_risk_explanation_for_candidate(
        &candidate(Some(5)),
        true,
        false,
        false,
        true,
    );
    assert!(high_risk
        .blocked_reason
        .contains("This setting needs separate high-risk approval."));
    assert_eq!(
        high_risk.high_risk_approval_inactive_reason.as_deref(),
        Some("High-risk approval is not active yet.")
    );
    assert!(high_risk.hard_block_reason.is_none());
    assert!(!high_risk.real_target_selection_active);

    let hard_block =
        recommendation_risk_explanation_for_candidate(&candidate(None), true, false, false, true);
    assert_eq!(
        hard_block.hard_block_reason.as_deref(),
        Some("Advanced confirmation cannot override this block.")
    );
    assert_eq!(SAFE_WRITABLE_ROWS.len(), 341);
}
