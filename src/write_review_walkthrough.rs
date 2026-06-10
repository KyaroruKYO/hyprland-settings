use crate::config_layered_values::LayeredSettingValues;
use crate::guarded_write_review::GuardedWriteTargetReview;
use crate::session_value_projection::SessionValueProjection;
use crate::write_advanced_confirmation::WriteAdvancedConfirmation;
use crate::write_backup_plan::WriteBackupPlan;
use crate::write_target_candidate::WriteTargetCandidate;
use crate::write_target_recommendation::WriteTargetRecommendation;
use crate::write_verification_plan::WriteVerificationPlan;

pub const PRODUCTION_WRITE_REVIEW_WALKTHROUGH_CAN_WRITE: bool = false;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct WriteReviewWalkthrough {
    pub steps: Vec<WriteReviewWalkthroughStep>,
    pub safety: WriteReviewWalkthroughSafetyFlags,
    pub target_decision: WriteReviewTargetDecisionState,
}

impl WriteReviewWalkthrough {
    pub fn user_facing_lines(&self) -> Vec<String> {
        let mut lines = vec![
            "Write review walkthrough".to_string(),
            "This walkthrough shows what the app would check before writing.".to_string(),
        ];
        for step in &self.steps {
            lines.push(format!("{}: {}", step.title, step.friendly_summary));
            for detail in &step.details {
                lines.push(detail.clone());
            }
        }
        lines.push("Target decisions are preview-only right now.".to_string());
        lines.push("Real save-location selection is not active yet.".to_string());
        lines.push("Real writing is not active yet.".to_string());
        lines.push("Apply behavior has not changed.".to_string());
        lines
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct WriteReviewWalkthroughStep {
    pub title: String,
    pub status: WriteReviewWalkthroughStepStatus,
    pub friendly_summary: String,
    pub details: Vec<String>,
    pub enabled: bool,
    pub completed: bool,
    pub blocking_reason: Option<String>,
}

impl WriteReviewWalkthroughStep {
    fn new(
        title: impl Into<String>,
        status: WriteReviewWalkthroughStepStatus,
        friendly_summary: impl Into<String>,
        details: Vec<String>,
    ) -> Self {
        let enabled = false;
        let completed = matches!(
            status,
            WriteReviewWalkthroughStepStatus::ReviewOnly
                | WriteReviewWalkthroughStepStatus::FixtureProofOnly
        );
        let blocking_reason = matches!(
            status,
            WriteReviewWalkthroughStepStatus::Blocked
                | WriteReviewWalkthroughStepStatus::ProductionDisabled
                | WriteReviewWalkthroughStepStatus::NotAvailable
        )
        .then(|| status.label().to_string());

        Self {
            title: title.into(),
            status,
            friendly_summary: friendly_summary.into(),
            details,
            enabled,
            completed,
            blocking_reason,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum WriteReviewWalkthroughStepStatus {
    NotAvailable,
    Ready,
    ReviewOnly,
    Blocked,
    RequiresConfirmationLater,
    FixtureProofOnly,
    ProductionDisabled,
}

impl WriteReviewWalkthroughStepStatus {
    pub fn label(self) -> &'static str {
        match self {
            Self::NotAvailable => "not available",
            Self::Ready => "ready",
            Self::ReviewOnly => "review only",
            Self::Blocked => "blocked",
            Self::RequiresConfirmationLater => "requires confirmation later",
            Self::FixtureProofOnly => "fixture proof only",
            Self::ProductionDisabled => "production disabled",
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct WriteReviewWalkthroughSafetyFlags {
    pub read_only: bool,
    pub production_disabled: bool,
    pub affects_apply: bool,
    pub affects_writes: bool,
    pub persists_selection: bool,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum WriteReviewTargetDecision {
    RecommendedTargetAccepted,
    AlternateTargetRequested,
    BlockedTargetRequested,
    AdvancedConfirmationNeeded,
    DecisionNotActive,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct WriteReviewTargetDecisionState {
    pub decision: WriteReviewTargetDecision,
    pub decision_enabled: bool,
    pub production_disabled: bool,
    pub selected_target: Option<WriteTargetCandidate>,
}

impl WriteReviewTargetDecisionState {
    pub fn disabled(decision: WriteReviewTargetDecision) -> Self {
        Self {
            decision,
            decision_enabled: false,
            production_disabled: true,
            selected_target: None,
        }
    }
}

pub fn build_write_review_walkthrough(
    session_projection: Option<&SessionValueProjection>,
    layered_values: Option<&LayeredSettingValues>,
    recommendation: Option<&WriteTargetRecommendation>,
    guarded_review: Option<&GuardedWriteTargetReview>,
    backup_plan: Option<&WriteBackupPlan>,
    advanced_confirmation: Option<&WriteAdvancedConfirmation>,
    verification_plan: Option<&WriteVerificationPlan>,
) -> WriteReviewWalkthrough {
    let mut steps = Vec::new();

    steps.push(session_step(session_projection));
    steps.push(layered_step(layered_values));
    steps.push(recommendation_step(recommendation));
    steps.push(backup_step(backup_plan));
    steps.push(advanced_step(advanced_confirmation));
    steps.push(verification_step(verification_plan));
    steps.push(production_gate_step(guarded_review));

    WriteReviewWalkthrough {
        steps,
        safety: WriteReviewWalkthroughSafetyFlags {
            read_only: true,
            production_disabled: true,
            affects_apply: false,
            affects_writes: false,
            persists_selection: false,
        },
        target_decision: WriteReviewTargetDecisionState::disabled(
            WriteReviewTargetDecision::DecisionNotActive,
        ),
    }
}

fn session_step(projection: Option<&SessionValueProjection>) -> WriteReviewWalkthroughStep {
    if let Some(projection) = projection {
        WriteReviewWalkthroughStep::new(
            "Compare values",
            WriteReviewWalkthroughStepStatus::ReviewOnly,
            format!(
                "Status: {}",
                projection.comparison_status.user_facing_label()
            ),
            projection.user_facing_lines(),
        )
    } else {
        WriteReviewWalkthroughStep::new(
            "Compare values",
            WriteReviewWalkthroughStepStatus::NotAvailable,
            "Session preview comparison is not available yet.",
            vec!["Active and session values cannot be compared yet.".to_string()],
        )
    }
}

fn layered_step(layered: Option<&LayeredSettingValues>) -> WriteReviewWalkthroughStep {
    if let Some(layered) = layered {
        let summary = if layered.controlled_in_more_than_one_place {
            "This setting is controlled in more than one place."
        } else {
            "This setting has one detected location."
        };
        WriteReviewWalkthroughStep::new(
            "Review where this setting is defined",
            WriteReviewWalkthroughStepStatus::ReviewOnly,
            summary,
            layered.display_lines(),
        )
    } else {
        WriteReviewWalkthroughStep::new(
            "Review where this setting is defined",
            WriteReviewWalkthroughStepStatus::NotAvailable,
            "Setting locations are not available yet.",
            vec!["No layered setting details are available.".to_string()],
        )
    }
}

fn recommendation_step(
    recommendation: Option<&WriteTargetRecommendation>,
) -> WriteReviewWalkthroughStep {
    if let Some(recommendation) = recommendation {
        let summary = recommendation
            .recommended_target
            .as_ref()
            .map(|target| format!("Recommended save location: {}", target.label))
            .unwrap_or_else(|| "Recommended save location is not available yet.".to_string());
        WriteReviewWalkthroughStep::new(
            "Recommended save location",
            WriteReviewWalkthroughStepStatus::ProductionDisabled,
            summary,
            recommendation.user_facing_lines(),
        )
    } else {
        WriteReviewWalkthroughStep::new(
            "Recommended save location",
            WriteReviewWalkthroughStepStatus::NotAvailable,
            "No save location recommendation is available yet.",
            vec!["Real save-location selection is not active yet.".to_string()],
        )
    }
}

fn backup_step(plan: Option<&WriteBackupPlan>) -> WriteReviewWalkthroughStep {
    if let Some(plan) = plan {
        WriteReviewWalkthroughStep::new(
            "Backup planned",
            WriteReviewWalkthroughStepStatus::FixtureProofOnly,
            "Backup planned for future.",
            plan.user_facing_lines(),
        )
    } else {
        WriteReviewWalkthroughStep::new(
            "Backup planned",
            WriteReviewWalkthroughStepStatus::NotAvailable,
            "Backup planning is not available yet.",
            vec!["The app will back up this exact file before saving changes.".to_string()],
        )
    }
}

fn advanced_step(confirmation: Option<&WriteAdvancedConfirmation>) -> WriteReviewWalkthroughStep {
    if let Some(confirmation) = confirmation {
        let status = if confirmation.requires_confirmation {
            WriteReviewWalkthroughStepStatus::RequiresConfirmationLater
        } else {
            WriteReviewWalkthroughStepStatus::ReviewOnly
        };
        WriteReviewWalkthroughStep::new(
            "Safety warning",
            status,
            if confirmation.requires_confirmation {
                "Advanced confirmation would be required before writing here."
            } else {
                "No advanced confirmation warning was detected for this target."
            },
            confirmation.user_facing_lines(),
        )
    } else {
        WriteReviewWalkthroughStep::new(
            "Safety warning",
            WriteReviewWalkthroughStepStatus::NotAvailable,
            "Safety warning details are not available yet.",
            vec!["Advanced confirmation is not active yet.".to_string()],
        )
    }
}

fn verification_step(plan: Option<&WriteVerificationPlan>) -> WriteReviewWalkthroughStep {
    if let Some(plan) = plan {
        WriteReviewWalkthroughStep::new(
            "Verification planned",
            WriteReviewWalkthroughStepStatus::ProductionDisabled,
            "Verification planned for future.",
            plan.user_facing_lines(),
        )
    } else {
        WriteReviewWalkthroughStep::new(
            "Verification planned",
            WriteReviewWalkthroughStepStatus::NotAvailable,
            "Verification planning is not available yet.",
            vec!["The app will reread the file to confirm the value.".to_string()],
        )
    }
}

fn production_gate_step(review: Option<&GuardedWriteTargetReview>) -> WriteReviewWalkthroughStep {
    let mut details = vec![
        "Real writing is not active yet.".to_string(),
        "Apply behavior has not changed.".to_string(),
    ];
    if let Some(review) = review {
        details.extend(review.user_facing_lines());
    }
    WriteReviewWalkthroughStep::new(
        "Status",
        WriteReviewWalkthroughStepStatus::ProductionDisabled,
        "Real writing is not active yet.",
        details,
    )
}
