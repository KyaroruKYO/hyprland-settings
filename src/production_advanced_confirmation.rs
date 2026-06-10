use std::path::PathBuf;

use crate::write_target_candidate::WriteTargetCandidate;

pub const PRODUCTION_ADVANCED_CONFIRMATION_ENABLED: bool = false;

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub enum TargetManagementRiskFlag {
    GeneratedFile,
    ScriptManagedFile,
    ScriptReferencedFile,
    SymlinkManagedFile,
    SymlinkTarget,
    UnknownManagementState,
    HighRiskSetting,
    MissingLineNumber,
    StructuredNonScalarTarget,
    DuplicateTargetAmbiguity,
    UnreadableTarget,
    RequiresScriptOrLuaExecution,
}

impl TargetManagementRiskFlag {
    pub fn friendly_label(self) -> &'static str {
        match self {
            Self::GeneratedFile => "This file appears to be generated.",
            Self::ScriptManagedFile => "This file may be changed by scripts.",
            Self::ScriptReferencedFile => "This file is referenced by scripts.",
            Self::SymlinkManagedFile => "This file is managed through a symlink.",
            Self::SymlinkTarget => "This target is reached through a symlink.",
            Self::UnknownManagementState => "The app cannot prove how this file is managed.",
            Self::HighRiskSetting => "This setting needs separate high-risk approval.",
            Self::MissingLineNumber => {
                "This setting needs a normal scalar line before it can be written safely."
            }
            Self::StructuredNonScalarTarget => {
                "Structured settings are not part of the first pilot."
            }
            Self::DuplicateTargetAmbiguity => {
                "This setting appears in more than one possible save location."
            }
            Self::UnreadableTarget => "This target could not be read.",
            Self::RequiresScriptOrLuaExecution => {
                "The app would need to execute script or Lua logic to understand this target."
            }
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TargetManagementRiskLevel {
    SafeForFirstPilot,
    RequiresAdvancedConfirmationLater,
    BlockedForFirstPilot,
    HardBlocked,
    Unknown,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct TargetManagementRiskInput {
    pub target_path: PathBuf,
    pub line_number: Option<usize>,
    pub generated_file: bool,
    pub script_managed_file: bool,
    pub script_referenced_file: bool,
    pub symlink_managed_file: bool,
    pub symlink_target: bool,
    pub unknown_management_state: bool,
    pub high_risk_setting: bool,
    pub structured_non_scalar_target: bool,
    pub duplicate_target_ambiguity: bool,
    pub target_readable: bool,
    pub requires_script_or_lua_execution: bool,
}

impl TargetManagementRiskInput {
    pub fn normal_scalar(target_path: impl Into<PathBuf>, line_number: usize) -> Self {
        Self {
            target_path: target_path.into(),
            line_number: Some(line_number),
            generated_file: false,
            script_managed_file: false,
            script_referenced_file: false,
            symlink_managed_file: false,
            symlink_target: false,
            unknown_management_state: false,
            high_risk_setting: false,
            structured_non_scalar_target: false,
            duplicate_target_ambiguity: false,
            target_readable: true,
            requires_script_or_lua_execution: false,
        }
    }

    pub fn from_candidate(
        candidate: &WriteTargetCandidate,
        high_risk_setting: bool,
        structured_non_scalar_target: bool,
        duplicate_target_ambiguity: bool,
        target_readable: bool,
    ) -> Self {
        Self {
            target_path: candidate.file_path.clone(),
            line_number: candidate.line_number,
            generated_file: candidate.generated_or_script_managed,
            script_managed_file: candidate.generated_or_script_managed,
            script_referenced_file: false,
            symlink_managed_file: candidate.symlink_managed,
            symlink_target: candidate.resolved_path.is_some(),
            unknown_management_state: false,
            high_risk_setting,
            structured_non_scalar_target,
            duplicate_target_ambiguity,
            target_readable,
            requires_script_or_lua_execution: false,
        }
    }

    pub fn risk_flags(&self) -> Vec<TargetManagementRiskFlag> {
        let mut flags = Vec::new();
        if self.generated_file {
            flags.push(TargetManagementRiskFlag::GeneratedFile);
        }
        if self.script_managed_file {
            flags.push(TargetManagementRiskFlag::ScriptManagedFile);
        }
        if self.script_referenced_file {
            flags.push(TargetManagementRiskFlag::ScriptReferencedFile);
        }
        if self.symlink_managed_file {
            flags.push(TargetManagementRiskFlag::SymlinkManagedFile);
        }
        if self.symlink_target {
            flags.push(TargetManagementRiskFlag::SymlinkTarget);
        }
        if self.unknown_management_state {
            flags.push(TargetManagementRiskFlag::UnknownManagementState);
        }
        if self.high_risk_setting {
            flags.push(TargetManagementRiskFlag::HighRiskSetting);
        }
        if self.line_number.is_none() {
            flags.push(TargetManagementRiskFlag::MissingLineNumber);
        }
        if self.structured_non_scalar_target {
            flags.push(TargetManagementRiskFlag::StructuredNonScalarTarget);
        }
        if self.duplicate_target_ambiguity {
            flags.push(TargetManagementRiskFlag::DuplicateTargetAmbiguity);
        }
        if !self.target_readable {
            flags.push(TargetManagementRiskFlag::UnreadableTarget);
        }
        if self.requires_script_or_lua_execution {
            flags.push(TargetManagementRiskFlag::RequiresScriptOrLuaExecution);
        }
        flags
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct TargetManagementRiskClassification {
    pub target_path: PathBuf,
    pub risk_flags: Vec<TargetManagementRiskFlag>,
    pub risk_level: TargetManagementRiskLevel,
    pub reasons: Vec<String>,
    pub advanced_confirmation_can_help_later: bool,
    pub eligible_for_first_pilot: bool,
    pub production_enabled: bool,
}

pub fn classify_target_management_risk(
    input: &TargetManagementRiskInput,
) -> TargetManagementRiskClassification {
    let flags = input.risk_flags();
    let hard_blocked = flags.iter().any(|flag| {
        matches!(
            flag,
            TargetManagementRiskFlag::MissingLineNumber
                | TargetManagementRiskFlag::StructuredNonScalarTarget
                | TargetManagementRiskFlag::UnreadableTarget
                | TargetManagementRiskFlag::RequiresScriptOrLuaExecution
                | TargetManagementRiskFlag::DuplicateTargetAmbiguity
        )
    });
    let requires_advanced = flags.iter().any(|flag| {
        matches!(
            flag,
            TargetManagementRiskFlag::GeneratedFile
                | TargetManagementRiskFlag::ScriptManagedFile
                | TargetManagementRiskFlag::ScriptReferencedFile
                | TargetManagementRiskFlag::SymlinkManagedFile
                | TargetManagementRiskFlag::SymlinkTarget
        )
    });
    let blocked_for_pilot = flags
        .iter()
        .any(|flag| matches!(flag, TargetManagementRiskFlag::HighRiskSetting));
    let unknown = flags
        .iter()
        .any(|flag| matches!(flag, TargetManagementRiskFlag::UnknownManagementState));

    let risk_level = if hard_blocked {
        TargetManagementRiskLevel::HardBlocked
    } else if blocked_for_pilot {
        TargetManagementRiskLevel::BlockedForFirstPilot
    } else if requires_advanced {
        TargetManagementRiskLevel::RequiresAdvancedConfirmationLater
    } else if unknown {
        TargetManagementRiskLevel::Unknown
    } else {
        TargetManagementRiskLevel::SafeForFirstPilot
    };
    let eligible_for_first_pilot = risk_level == TargetManagementRiskLevel::SafeForFirstPilot;

    TargetManagementRiskClassification {
        target_path: input.target_path.clone(),
        risk_flags: flags.clone(),
        risk_level,
        reasons: flags
            .iter()
            .map(|flag| flag.friendly_label().to_string())
            .collect(),
        advanced_confirmation_can_help_later: requires_advanced && !hard_blocked,
        eligible_for_first_pilot,
        production_enabled: PRODUCTION_ADVANCED_CONFIRMATION_ENABLED,
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ProductionAdvancedConfirmationPolicy {
    pub target_path: PathBuf,
    pub risk_flags: Vec<TargetManagementRiskFlag>,
    pub confirmation_required: bool,
    pub confirmation_unavailable_in_production: bool,
    pub acknowledgement_text: Vec<String>,
    pub blocked_conditions: Vec<String>,
    pub hard_blocked_conditions: Vec<String>,
    pub production_enabled: bool,
    pub fixture_only: bool,
}

impl ProductionAdvancedConfirmationPolicy {
    pub fn user_facing_lines(&self) -> Vec<String> {
        let mut lines = vec![
            "Advanced confirmation".to_string(),
            "Some files may be changed by scripts, generated by tools, or managed through symlinks."
                .to_string(),
            "These targets are excluded from the first production write pilot.".to_string(),
        ];
        lines.extend(self.acknowledgement_text.clone());
        if !self.hard_blocked_conditions.is_empty() {
            lines.push("This target is not eligible for the first write pilot.".to_string());
        }
        lines.push("Advanced confirmation is not active yet.".to_string());
        lines.push("Real writing is not active yet.".to_string());
        lines.push("Apply behavior has not changed.".to_string());
        lines
    }
}

pub fn production_advanced_confirmation_policy(
    input: &TargetManagementRiskInput,
) -> ProductionAdvancedConfirmationPolicy {
    let classification = classify_target_management_risk(input);
    let hard_block = hard_block_policy(input);
    let acknowledgement_text = acknowledgement_text_for_flags(&classification.risk_flags);

    ProductionAdvancedConfirmationPolicy {
        target_path: input.target_path.clone(),
        risk_flags: classification.risk_flags,
        confirmation_required: classification.advanced_confirmation_can_help_later,
        confirmation_unavailable_in_production: !PRODUCTION_ADVANCED_CONFIRMATION_ENABLED,
        acknowledgement_text,
        blocked_conditions: classification.reasons,
        hard_blocked_conditions: hard_block.reasons,
        production_enabled: PRODUCTION_ADVANCED_CONFIRMATION_ENABLED,
        fixture_only: true,
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct TargetHardBlockPolicy {
    pub target_path: PathBuf,
    pub hard_blocked: bool,
    pub reasons: Vec<String>,
    pub proof_required_later: Vec<String>,
    pub advanced_confirmation_can_override: bool,
    pub production_enabled: bool,
}

pub fn hard_block_policy(input: &TargetManagementRiskInput) -> TargetHardBlockPolicy {
    let hard_flags = input
        .risk_flags()
        .into_iter()
        .filter(|flag| {
            matches!(
                flag,
                TargetManagementRiskFlag::MissingLineNumber
                    | TargetManagementRiskFlag::StructuredNonScalarTarget
                    | TargetManagementRiskFlag::UnreadableTarget
                    | TargetManagementRiskFlag::RequiresScriptOrLuaExecution
                    | TargetManagementRiskFlag::DuplicateTargetAmbiguity
            )
        })
        .collect::<Vec<_>>();

    TargetHardBlockPolicy {
        target_path: input.target_path.clone(),
        hard_blocked: !hard_flags.is_empty(),
        reasons: hard_flags
            .iter()
            .map(|flag| flag.friendly_label().to_string())
            .collect(),
        proof_required_later: vec![
            "normal scalar line proof".to_string(),
            "readable target proof".to_string(),
            "no script or Lua execution required".to_string(),
            "duplicate target ambiguity resolved".to_string(),
        ],
        advanced_confirmation_can_override: false,
        production_enabled: PRODUCTION_ADVANCED_CONFIRMATION_ENABLED,
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct FirstPilotExclusionPolicy {
    pub target_path: PathBuf,
    pub eligible_for_first_pilot: bool,
    pub excluded_reasons: Vec<String>,
    pub production_gate_enabled: bool,
}

impl FirstPilotExclusionPolicy {
    pub fn user_facing_lines(&self) -> Vec<String> {
        if self.eligible_for_first_pilot {
            vec![
                "One existing scalar line in one normal config file".to_string(),
                "Normal scalar target can be reviewed for the first pilot later.".to_string(),
                "Production pilot gate remains false.".to_string(),
            ]
        } else {
            let mut lines = vec![
                "This target is not eligible for the first write pilot.".to_string(),
                "The first production write pilot only supports normal scalar targets.".to_string(),
            ];
            lines.extend(self.excluded_reasons.clone());
            lines.push("Production pilot gate remains false.".to_string());
            lines
        }
    }
}

pub fn first_pilot_exclusion_policy(
    input: &TargetManagementRiskInput,
) -> FirstPilotExclusionPolicy {
    let classification = classify_target_management_risk(input);
    let excluded_reasons = if classification.eligible_for_first_pilot {
        Vec::new()
    } else {
        classification.reasons
    };

    FirstPilotExclusionPolicy {
        target_path: input.target_path.clone(),
        eligible_for_first_pilot: classification.eligible_for_first_pilot,
        excluded_reasons,
        production_gate_enabled:
            crate::one_target_write_pilot::PRODUCTION_ONE_TARGET_WRITE_PILOT_ENABLED,
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RecommendationRiskExplanation {
    pub target_path: PathBuf,
    pub blocked_reason: String,
    pub advanced_confirmation_inactive_reason: Option<String>,
    pub hard_block_reason: Option<String>,
    pub real_target_selection_active: bool,
}

pub fn recommendation_risk_explanation_for_candidate(
    candidate: &WriteTargetCandidate,
    high_risk_setting: bool,
    structured_non_scalar_target: bool,
    duplicate_target_ambiguity: bool,
    target_readable: bool,
) -> RecommendationRiskExplanation {
    let input = TargetManagementRiskInput::from_candidate(
        candidate,
        high_risk_setting,
        structured_non_scalar_target,
        duplicate_target_ambiguity,
        target_readable,
    );
    let classification = classify_target_management_risk(&input);
    let hard_block = hard_block_policy(&input);

    RecommendationRiskExplanation {
        target_path: candidate.file_path.clone(),
        blocked_reason: if classification.reasons.is_empty() {
            "This location is not safe for automatic writes yet.".to_string()
        } else {
            classification.reasons.join(" ")
        },
        advanced_confirmation_inactive_reason: classification
            .advanced_confirmation_can_help_later
            .then(|| "Advanced confirmation is not active yet.".to_string()),
        hard_block_reason: hard_block
            .hard_blocked
            .then(|| "Advanced confirmation cannot override this block.".to_string()),
        real_target_selection_active: false,
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct AdvancedConfirmationReadinessMapping {
    pub risk_policy_complete: bool,
    pub hard_block_policy_complete: bool,
    pub first_pilot_exclusion_policy_complete: bool,
    pub risky_target_ui_copy_complete: bool,
    pub fixture_risk_policy_proof_passed: bool,
    pub production_advanced_confirmation_enabled: bool,
}

impl AdvancedConfirmationReadinessMapping {
    pub fn ready_for_production(&self) -> bool {
        self.risk_policy_complete
            && self.hard_block_policy_complete
            && self.first_pilot_exclusion_policy_complete
            && self.risky_target_ui_copy_complete
            && self.fixture_risk_policy_proof_passed
            && self.production_advanced_confirmation_enabled
    }
}

pub fn current_advanced_confirmation_readiness_mapping() -> AdvancedConfirmationReadinessMapping {
    AdvancedConfirmationReadinessMapping {
        risk_policy_complete: false,
        hard_block_policy_complete: false,
        first_pilot_exclusion_policy_complete: false,
        risky_target_ui_copy_complete: false,
        fixture_risk_policy_proof_passed: false,
        production_advanced_confirmation_enabled: PRODUCTION_ADVANCED_CONFIRMATION_ENABLED,
    }
}

pub fn disabled_advanced_confirmation_ui_lines() -> Vec<String> {
    vec![
        "Advanced confirmation".to_string(),
        "Some files may be changed by scripts, generated by tools, or managed through symlinks."
            .to_string(),
        "These targets are excluded from the first production write pilot.".to_string(),
        "Advanced confirmation is not active yet.".to_string(),
        "Real writing is not active yet.".to_string(),
        "Apply behavior has not changed.".to_string(),
    ]
}

fn acknowledgement_text_for_flags(flags: &[TargetManagementRiskFlag]) -> Vec<String> {
    let mut lines = Vec::new();
    if flags.contains(&TargetManagementRiskFlag::ScriptManagedFile)
        || flags.contains(&TargetManagementRiskFlag::ScriptReferencedFile)
    {
        lines.push("I understand this file may be changed by scripts.".to_string());
    }
    if flags.contains(&TargetManagementRiskFlag::GeneratedFile) {
        lines.push("I understand this file appears to be generated.".to_string());
    }
    if flags.contains(&TargetManagementRiskFlag::SymlinkManagedFile)
        || flags.contains(&TargetManagementRiskFlag::SymlinkTarget)
    {
        lines.push("I understand this file is symlink-managed.".to_string());
    }
    if !lines.is_empty() {
        lines.push("I understand writing here may be overwritten outside the app.".to_string());
        lines.push(
            "I understand the app must back up, write, reread, verify, and recover safely."
                .to_string(),
        );
    }
    lines
}
