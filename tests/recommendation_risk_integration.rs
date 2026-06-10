use std::path::PathBuf;

use hyprland_settings::production_advanced_confirmation::recommendation_risk_explanation_for_candidate;
use hyprland_settings::write_classification::SAFE_WRITABLE_ROWS;
use hyprland_settings::write_target_candidate::WriteTargetCandidate;
use hyprland_settings::write_target_recommendation::recommend_write_targets;

fn candidate(
    label: &str,
    generated: bool,
    symlink: bool,
    line_number: Option<usize>,
) -> WriteTargetCandidate {
    WriteTargetCandidate {
        label: label.to_string(),
        file_path: PathBuf::from(format!("/tmp/{label}.conf")),
        resolved_path: symlink.then(|| PathBuf::from(format!("/tmp/{label}-target.conf"))),
        line_number,
        safe: !generated && !symlink && line_number.is_some(),
        generated_or_script_managed: generated,
        symlink_managed: symlink,
        requires_advanced_confirmation: generated || symlink,
        backup_required: true,
        fixture_only: true,
    }
}

#[test]
fn recommendations_include_risk_policy_explanations_without_enabling_selection() {
    let candidates = vec![
        candidate("main", false, false, Some(1)),
        candidate("generated", true, false, Some(2)),
        candidate("symlink", false, true, Some(3)),
        candidate("missing_line", false, false, None),
    ];
    let recommendation = recommend_write_targets(&candidates);

    assert!(recommendation.recommended_target.is_some());
    assert_eq!(recommendation.blocked_targets.len(), 3);
    let reasons = recommendation
        .blocked_targets
        .iter()
        .map(|target| target.reason.as_str())
        .collect::<Vec<_>>()
        .join("\n");
    assert!(reasons.contains("This file may be changed by scripts."));
    assert!(reasons.contains("This file appears to be generated."));
    assert!(reasons.contains("This file is managed through a symlink."));
    assert!(reasons.contains("This target is reached through a symlink."));
    assert!(reasons.contains("This setting needs a normal scalar line"));
    assert!(reasons.contains("Advanced confirmation is not active yet."));
    assert!(reasons.contains("Advanced confirmation cannot override this block."));
    assert!(recommendation.production_disabled);

    let high_risk = recommendation_risk_explanation_for_candidate(
        &candidate("high_risk", false, false, Some(4)),
        true,
        false,
        false,
        true,
    );
    assert!(high_risk
        .blocked_reason
        .contains("This setting needs separate high-risk approval."));
    assert!(!high_risk.real_target_selection_active);
    assert_eq!(SAFE_WRITABLE_ROWS.len(), 341);
}
