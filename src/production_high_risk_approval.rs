use crate::production_advanced_confirmation::{
    classify_target_management_risk, hard_block_policy, TargetManagementRiskInput,
    TargetManagementRiskLevel,
};
use crate::write_classification::{high_risk_write_policy, HighRiskWritePolicy};

pub const PRODUCTION_HIGH_RISK_APPROVAL_ENABLED: bool = false;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum HighRiskApprovalCategory {
    NotHighRisk,
    HighRiskApprovableLater,
    HighRiskRequiresSeparatePolicy,
    HardBlockedHighRisk,
    UnknownHighRiskStatus,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum HighRiskApprovalState {
    NotRequired,
    RequiredButUnavailable,
    RequestedButDisabled,
    ApprovedInFixtureOnly,
    Rejected,
    Expired,
    ProductionDisabled,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ProductionHighRiskApprovalBoundary {
    pub row_id: String,
    pub official_setting_id: String,
    pub policy: Option<HighRiskWritePolicy>,
    pub risk_category: HighRiskApprovalCategory,
    pub approval_required: bool,
    pub approval_currently_unavailable: bool,
    pub first_pilot_eligible: bool,
    pub production_enabled: bool,
    pub fixture_only: bool,
}

impl ProductionHighRiskApprovalBoundary {
    pub fn user_facing_lines(&self) -> Vec<String> {
        let mut lines = disabled_high_risk_approval_ui_lines();
        if self.approval_required {
            lines.push("This setting needs separate high-risk approval.".to_string());
            lines.push("High-risk approval cannot override this block.".to_string());
        }
        lines
    }
}

pub fn high_risk_approval_boundary(
    row_id: impl Into<String>,
    official_setting_id: impl Into<String>,
    target: Option<&TargetManagementRiskInput>,
    unknown_high_risk_status: bool,
) -> ProductionHighRiskApprovalBoundary {
    let row_id = row_id.into();
    let official_setting_id = official_setting_id.into();
    let policy = high_risk_write_policy(&row_id);
    let hard_blocked = target.is_some_and(|input| hard_block_policy(input).hard_blocked);
    let high_risk = policy.is_some();

    let risk_category = if unknown_high_risk_status {
        HighRiskApprovalCategory::UnknownHighRiskStatus
    } else if hard_blocked && high_risk {
        HighRiskApprovalCategory::HardBlockedHighRisk
    } else if let Some(policy) = policy {
        if policy.approval_gate.contains("explicit-high-risk") {
            HighRiskApprovalCategory::HighRiskRequiresSeparatePolicy
        } else {
            HighRiskApprovalCategory::HighRiskApprovableLater
        }
    } else {
        HighRiskApprovalCategory::NotHighRisk
    };

    let approval_required = matches!(
        risk_category,
        HighRiskApprovalCategory::HighRiskApprovableLater
            | HighRiskApprovalCategory::HighRiskRequiresSeparatePolicy
            | HighRiskApprovalCategory::HardBlockedHighRisk
            | HighRiskApprovalCategory::UnknownHighRiskStatus
    );
    let first_pilot_eligible = !approval_required
        && target
            .map(|input| {
                classify_target_management_risk(input).risk_level
                    == TargetManagementRiskLevel::SafeForFirstPilot
            })
            .unwrap_or(true);

    ProductionHighRiskApprovalBoundary {
        row_id,
        official_setting_id,
        policy,
        risk_category,
        approval_required,
        approval_currently_unavailable: approval_required && !PRODUCTION_HIGH_RISK_APPROVAL_ENABLED,
        first_pilot_eligible,
        production_enabled: PRODUCTION_HIGH_RISK_APPROVAL_ENABLED,
        fixture_only: true,
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct HighRiskApprovalStateModel {
    pub state: HighRiskApprovalState,
    pub can_persist: bool,
    pub affects_apply: bool,
    pub can_make_first_pilot_eligible: bool,
    pub production_enabled: bool,
}

pub fn high_risk_approval_state_model(state: HighRiskApprovalState) -> HighRiskApprovalStateModel {
    HighRiskApprovalStateModel {
        state,
        can_persist: false,
        affects_apply: false,
        can_make_first_pilot_eligible: false,
        production_enabled: PRODUCTION_HIGH_RISK_APPROVAL_ENABLED,
    }
}

pub fn all_high_risk_approval_states() -> Vec<HighRiskApprovalState> {
    vec![
        HighRiskApprovalState::NotRequired,
        HighRiskApprovalState::RequiredButUnavailable,
        HighRiskApprovalState::RequestedButDisabled,
        HighRiskApprovalState::ApprovedInFixtureOnly,
        HighRiskApprovalState::Rejected,
        HighRiskApprovalState::Expired,
        HighRiskApprovalState::ProductionDisabled,
    ]
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct FirstPilotHighRiskExclusion {
    pub row_id: String,
    pub high_risk_excluded: bool,
    pub hard_blocked: bool,
    pub normal_non_high_risk_scalar_eligible: bool,
    pub reasons: Vec<String>,
    pub production_gate_enabled: bool,
}

pub fn first_pilot_high_risk_exclusion(
    row_id: impl Into<String>,
    target: &TargetManagementRiskInput,
) -> FirstPilotHighRiskExclusion {
    let row_id = row_id.into();
    let boundary = high_risk_approval_boundary(&row_id, &row_id, Some(target), false);
    let target_classification = classify_target_management_risk(target);
    let hard_block = hard_block_policy(target);
    let high_risk_excluded = boundary.approval_required;
    let normal_non_high_risk_scalar_eligible =
        !high_risk_excluded && target_classification.eligible_for_first_pilot;
    let mut reasons = target_classification.reasons;
    if high_risk_excluded {
        reasons.push("This setting needs separate high-risk approval.".to_string());
        reasons.push("High-risk approval is not active yet.".to_string());
    }
    if hard_block.hard_blocked {
        reasons.push("High-risk approval cannot override this block.".to_string());
    }

    FirstPilotHighRiskExclusion {
        row_id,
        high_risk_excluded,
        hard_blocked: hard_block.hard_blocked,
        normal_non_high_risk_scalar_eligible,
        reasons,
        production_gate_enabled:
            crate::one_target_write_pilot::PRODUCTION_ONE_TARGET_WRITE_PILOT_ENABLED,
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct HighRiskReadinessMapping {
    pub approval_boundary_complete: bool,
    pub classification_integration_complete: bool,
    pub approval_state_model_complete: bool,
    pub warning_copy_complete: bool,
    pub first_pilot_exclusion_proof_complete: bool,
    pub production_high_risk_approval_enabled: bool,
}

impl HighRiskReadinessMapping {
    pub fn ready_for_production(&self) -> bool {
        self.approval_boundary_complete
            && self.classification_integration_complete
            && self.approval_state_model_complete
            && self.warning_copy_complete
            && self.first_pilot_exclusion_proof_complete
            && self.production_high_risk_approval_enabled
    }
}

pub fn current_high_risk_readiness_mapping() -> HighRiskReadinessMapping {
    HighRiskReadinessMapping {
        approval_boundary_complete: false,
        classification_integration_complete: false,
        approval_state_model_complete: false,
        warning_copy_complete: false,
        first_pilot_exclusion_proof_complete: false,
        production_high_risk_approval_enabled: PRODUCTION_HIGH_RISK_APPROVAL_ENABLED,
    }
}

pub fn disabled_high_risk_approval_ui_lines() -> Vec<String> {
    vec![
        "High-risk approval".to_string(),
        "Some settings need extra review before they can ever be written.".to_string(),
        "High-risk rows are excluded from the first production write pilot.".to_string(),
        "High-risk approval is not active yet.".to_string(),
        "Real writing is not active yet.".to_string(),
        "Apply behavior has not changed.".to_string(),
    ]
}

pub fn future_high_risk_acknowledgement_text() -> Vec<String> {
    vec![
        "I understand this setting may affect session stability.".to_string(),
        "I understand this setting may require manual recovery if misconfigured.".to_string(),
        "I understand the app must back up, write, reread, verify, and recover safely.".to_string(),
        "I understand high-risk approval does not override hard blocks.".to_string(),
    ]
}
