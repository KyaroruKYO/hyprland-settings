use crate::write_target_candidate::WriteTargetCandidate;
use crate::write_target_recommendation::{BlockedWriteTarget, WriteTargetRecommendation};

pub const PRODUCTION_WRITE_TARGET_REVIEW_ENABLED: bool = true;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct GuardedWriteTargetReview {
    pub row_id: String,
    pub official_setting_id: String,
    pub proposed_value: String,
    pub active_value: Option<String>,
    pub session_preview_value: Option<String>,
    pub selected_candidate: Option<WriteTargetCandidate>,
    pub recommended_candidate: Option<WriteTargetCandidate>,
    pub other_candidates: Vec<WriteTargetCandidate>,
    pub blocked_candidates: Vec<BlockedWriteTarget>,
    pub review_status: GuardedWriteReviewStatus,
    pub required_gates: GuardedWriteReviewGates,
    pub production_enabled: bool,
    pub fixture_proof_status: FixtureProofStatus,
}

impl GuardedWriteTargetReview {
    pub fn user_facing_lines(&self) -> Vec<String> {
        let mut lines = vec![
            "Write review".to_string(),
            "Safe batch write is available for normal settings.".to_string(),
            "Some settings are blocked because they need extra safety review.".to_string(),
            "The app will back up files before writing.".to_string(),
            "The app will check the result after writing.".to_string(),
            "If something fails, the app will restore the backup.".to_string(),
        ];
        if let Some(candidate) = &self.recommended_candidate {
            lines.push(format!("Recommended save location: {}", candidate.label));
        }
        if !self.other_candidates.is_empty() {
            lines.push(format!(
                "Other possible locations: {}",
                self.other_candidates
                    .iter()
                    .map(|candidate| candidate.label.as_str())
                    .collect::<Vec<_>>()
                    .join(", ")
            ));
        }
        if !self.blocked_candidates.is_empty() {
            lines.push(format!(
                "Blocked locations: {}",
                self.blocked_candidates
                    .iter()
                    .map(|candidate| candidate.label.as_str())
                    .collect::<Vec<_>>()
                    .join(", ")
            ));
        }
        lines.push(format!("Review status: {}", self.review_status.label()));
        lines
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum GuardedWriteReviewStatus {
    NotAvailable,
    ReadyForReview,
    Blocked,
    RequiresAdvancedConfirmation,
    FixtureProofOnly,
    ProductionDisabled,
}

impl GuardedWriteReviewStatus {
    pub fn label(self) -> &'static str {
        match self {
            Self::NotAvailable => "not available",
            Self::ReadyForReview => "ready for review",
            Self::Blocked => "blocked",
            Self::RequiresAdvancedConfirmation => "requires advanced confirmation",
            Self::FixtureProofOnly => "fixture proof only",
            Self::ProductionDisabled => "production disabled",
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct GuardedWriteReviewGates {
    pub target_selected: bool,
    pub exact_backup_planned: bool,
    pub generated_script_managed_confirmation_resolved: bool,
    pub reread_verification_planned: bool,
    pub high_risk_policy_satisfied: bool,
    pub fixture_proof_passed: bool,
    pub production_write_integration_allowed: bool,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum FixtureProofStatus {
    NotRun,
    Passed,
    Failed,
}

pub fn build_guarded_write_target_review(
    row_id: impl Into<String>,
    official_setting_id: impl Into<String>,
    proposed_value: impl Into<String>,
    active_value: Option<String>,
    session_preview_value: Option<String>,
    recommendation: &WriteTargetRecommendation,
    selected_candidate: Option<WriteTargetCandidate>,
    high_risk_policy_satisfied: bool,
    fixture_proof_status: FixtureProofStatus,
) -> GuardedWriteTargetReview {
    let selected = selected_candidate.or_else(|| recommendation.recommended_target.clone());
    let generated_confirmation_resolved = selected
        .as_ref()
        .map(|candidate| !candidate.requires_advanced_confirmation)
        .unwrap_or(false);
    let fixture_proof_passed = fixture_proof_status == FixtureProofStatus::Passed;

    let review_status = if !PRODUCTION_WRITE_TARGET_REVIEW_ENABLED {
        GuardedWriteReviewStatus::ProductionDisabled
    } else if selected.is_none() {
        GuardedWriteReviewStatus::NotAvailable
    } else if !generated_confirmation_resolved {
        GuardedWriteReviewStatus::RequiresAdvancedConfirmation
    } else if !fixture_proof_passed {
        GuardedWriteReviewStatus::FixtureProofOnly
    } else if high_risk_policy_satisfied {
        GuardedWriteReviewStatus::ReadyForReview
    } else {
        GuardedWriteReviewStatus::Blocked
    };

    GuardedWriteTargetReview {
        row_id: row_id.into(),
        official_setting_id: official_setting_id.into(),
        proposed_value: proposed_value.into(),
        active_value,
        session_preview_value,
        selected_candidate: selected.clone(),
        recommended_candidate: recommendation.recommended_target.clone(),
        other_candidates: recommendation.other_possible_targets.clone(),
        blocked_candidates: recommendation.blocked_targets.clone(),
        review_status,
        required_gates: GuardedWriteReviewGates {
            target_selected: selected.is_some(),
            exact_backup_planned: selected.is_some(),
            generated_script_managed_confirmation_resolved: generated_confirmation_resolved,
            reread_verification_planned: selected.is_some(),
            high_risk_policy_satisfied,
            fixture_proof_passed,
            production_write_integration_allowed: PRODUCTION_WRITE_TARGET_REVIEW_ENABLED,
        },
        production_enabled: PRODUCTION_WRITE_TARGET_REVIEW_ENABLED,
        fixture_proof_status,
    }
}
