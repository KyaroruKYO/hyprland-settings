use std::path::PathBuf;

use hyprland_settings::guarded_write_review::{
    build_guarded_write_target_review, FixtureProofStatus, GuardedWriteReviewStatus,
    PRODUCTION_WRITE_TARGET_REVIEW_ENABLED,
};
use hyprland_settings::write_classification::SAFE_WRITABLE_ROWS;
use hyprland_settings::write_target_candidate::WriteTargetCandidate;
use hyprland_settings::write_target_recommendation::recommend_write_targets;

fn candidate(label: &str, safe: bool) -> WriteTargetCandidate {
    WriteTargetCandidate {
        label: label.to_string(),
        file_path: PathBuf::from(format!("/tmp/{label}.conf")),
        resolved_path: None,
        line_number: Some(3),
        safe,
        generated_or_script_managed: !safe,
        symlink_managed: false,
        requires_advanced_confirmation: !safe,
        backup_required: true,
        fixture_only: true,
    }
}

#[test]
fn guarded_review_model_represents_candidates_gates_and_nonwriting_approval() {
    let candidates = vec![
        candidate("Current profile", true),
        candidate("Generated file", false),
    ];
    let recommendation = recommend_write_targets(&candidates);
    let review = build_guarded_write_target_review(
        "appearance.blur.enabled",
        "decoration.blur.enabled",
        "true",
        Some("false".to_string()),
        Some("true".to_string()),
        &recommendation,
        recommendation.recommended_target.clone(),
        true,
        FixtureProofStatus::Passed,
    );

    assert!(PRODUCTION_WRITE_TARGET_REVIEW_ENABLED);
    assert_eq!(
        review.review_status,
        GuardedWriteReviewStatus::ReadyForReview
    );
    assert!(review.required_gates.target_selected);
    assert!(review.required_gates.exact_backup_planned);
    assert!(review.required_gates.reread_verification_planned);
    assert!(review.required_gates.high_risk_policy_satisfied);
    assert!(review.required_gates.fixture_proof_passed);
    assert!(review.required_gates.production_write_integration_allowed);
    assert!(review.production_enabled);
    assert_eq!(review.blocked_candidates.len(), 1);
    assert_eq!(review.active_value.as_deref(), Some("false"));
    assert_eq!(review.session_preview_value.as_deref(), Some("true"));
    assert!(review
        .user_facing_lines()
        .iter()
        .any(|line| line == "Real writing is not active yet."));
    assert!(review
        .user_facing_lines()
        .iter()
        .any(|line| line == "Write review approval is staged; Apply still cannot write."));
    assert_eq!(SAFE_WRITABLE_ROWS.len(), 341);
}

#[test]
fn all_guarded_review_statuses_are_represented() {
    let statuses = [
        GuardedWriteReviewStatus::NotAvailable,
        GuardedWriteReviewStatus::ReadyForReview,
        GuardedWriteReviewStatus::Blocked,
        GuardedWriteReviewStatus::RequiresAdvancedConfirmation,
        GuardedWriteReviewStatus::FixtureProofOnly,
        GuardedWriteReviewStatus::ProductionDisabled,
    ];

    assert_eq!(statuses.len(), 6);
    assert!(statuses
        .iter()
        .any(|status| status.label() == "production disabled"));
}
