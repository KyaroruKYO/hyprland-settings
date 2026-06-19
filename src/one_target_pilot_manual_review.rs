use crate::guarded_write_review::PRODUCTION_WRITE_TARGET_REVIEW_ENABLED;
use crate::one_target_pilot_pre_enable_audit::{
    one_target_pilot_gate_inventory_snapshot, OneTargetPilotGateSnapshotItem,
    PRODUCTION_ONE_TARGET_PRE_ENABLE_AUDIT_PASSED,
};
use crate::one_target_write_pilot::PRODUCTION_ONE_TARGET_WRITE_PILOT_ENABLED;
use crate::production_advanced_confirmation::PRODUCTION_ADVANCED_CONFIRMATION_ENABLED;
use crate::production_backup_contract::PRODUCTION_BACKUP_CONTRACT_ENABLED;
use crate::production_high_risk_approval::PRODUCTION_HIGH_RISK_APPROVAL_ENABLED;
use crate::production_recovery_contract::PRODUCTION_RECOVERY_CONTRACT_ENABLED;
use crate::production_verification_contract::PRODUCTION_VERIFICATION_CONTRACT_ENABLED;
use crate::write_enablement_readiness::PRODUCTION_WRITE_TARGET_SELECTION_READY;
use crate::write_review_walkthrough::PRODUCTION_WRITE_REVIEW_WALKTHROUGH_CAN_WRITE;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ManualSmokeReviewItemResult {
    Passed,
    Failed,
    NotReviewed,
    NotApplicable,
    SourceProven,
    FixtureProven,
    ManualOnly,
}

impl ManualSmokeReviewItemResult {
    pub fn label(self) -> &'static str {
        match self {
            Self::Passed => "passed",
            Self::Failed => "failed",
            Self::NotReviewed => "not reviewed",
            Self::NotApplicable => "not applicable",
            Self::SourceProven => "source-proven",
            Self::FixtureProven => "fixture-proven",
            Self::ManualOnly => "manual-only",
        }
    }
}

pub fn all_manual_smoke_review_item_results() -> Vec<ManualSmokeReviewItemResult> {
    vec![
        ManualSmokeReviewItemResult::Passed,
        ManualSmokeReviewItemResult::Failed,
        ManualSmokeReviewItemResult::NotReviewed,
        ManualSmokeReviewItemResult::NotApplicable,
        ManualSmokeReviewItemResult::SourceProven,
        ManualSmokeReviewItemResult::FixtureProven,
        ManualSmokeReviewItemResult::ManualOnly,
    ]
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ManualSmokeReviewItem {
    pub item_label: &'static str,
    pub result: ManualSmokeReviewItemResult,
    pub evidence: &'static str,
    pub blocking_reason: Option<&'static str>,
    pub blocks_gate_flip_proposal_readiness: bool,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct OneTargetPilotManualSmokeReviewResult {
    pub review_method: &'static str,
    pub live_gtk_visual_smoke_performed: bool,
    pub items: Vec<ManualSmokeReviewItem>,
    pub gate_flip_proposal_ready: bool,
    pub production_activation_ready: bool,
}

impl OneTargetPilotManualSmokeReviewResult {
    pub fn source_or_fixture_proven_labels(&self) -> Vec<&'static str> {
        self.items
            .iter()
            .filter(|item| {
                matches!(
                    item.result,
                    ManualSmokeReviewItemResult::SourceProven
                        | ManualSmokeReviewItemResult::FixtureProven
                )
            })
            .map(|item| item.item_label)
            .collect()
    }

    pub fn not_reviewed_labels(&self) -> Vec<&'static str> {
        self.items
            .iter()
            .filter(|item| {
                matches!(
                    item.result,
                    ManualSmokeReviewItemResult::NotReviewed
                        | ManualSmokeReviewItemResult::ManualOnly
                )
            })
            .map(|item| item.item_label)
            .collect()
    }
}

pub fn one_target_pilot_manual_smoke_review_result() -> OneTargetPilotManualSmokeReviewResult {
    OneTargetPilotManualSmokeReviewResult {
        review_method:
            "source/report review only; no live GTK visual smoke was launched in this sprint",
        live_gtk_visual_smoke_performed: false,
        items: vec![
            review_item(
                "Open the app.",
                ManualSmokeReviewItemResult::ManualOnly,
                "not launched; source/UI copy was reviewed instead",
                Some("live GTK manual smoke review was not performed"),
                true,
            ),
            review_item(
                "Open a normal settings category.",
                ManualSmokeReviewItemResult::SourceProven,
                "normal category pages and detail panes remain wired in source and existing tests",
                None,
                false,
            ),
            review_item(
                "Select a known safe scalar row.",
                ManualSmokeReviewItemResult::SourceProven,
                "one-target pilot design requires one existing scalar line in one normal file",
                None,
                false,
            ),
            review_item(
                "Confirm the row has one current scalar value.",
                ManualSmokeReviewItemResult::SourceProven,
                "pilot target constraints require one scalar setting and one existing scalar line",
                None,
                false,
            ),
            review_item(
                "Confirm the target file is a normal config file.",
                ManualSmokeReviewItemResult::SourceProven,
                "target-risk policy excludes generated, script, symlink, structured, and ambiguous targets",
                None,
                false,
            ),
            review_item(
                "Confirm generated/script/symlink exclusions.",
                ManualSmokeReviewItemResult::SourceProven,
                "first-pilot exclusion and target-risk policies exclude generated, script-managed, script-referenced, symlink-managed, and symlink targets",
                None,
                false,
            ),
            review_item(
                "Confirm high-risk exclusion.",
                ManualSmokeReviewItemResult::SourceProven,
                "high-risk approval boundary excludes all high-risk rows from the first pilot",
                None,
                false,
            ),
            review_item(
                "Confirm exact line number is known.",
                ManualSmokeReviewItemResult::SourceProven,
                "one-target pilot candidate eligibility requires candidate.line_number to be present",
                None,
                false,
            ),
            review_item(
                "Confirm backup contract is shown as required.",
                ManualSmokeReviewItemResult::SourceProven,
                "disabled production review UI includes production backup and verification copy",
                None,
                false,
            ),
            review_item(
                "Confirm verification contract is shown as required.",
                ManualSmokeReviewItemResult::SourceProven,
                "disabled production review UI includes reread verification copy",
                None,
                false,
            ),
            review_item(
                "Confirm recovery contract is shown as required.",
                ManualSmokeReviewItemResult::SourceProven,
                "readiness UI copy includes recovery and rollback behavior",
                None,
                false,
            ),
            review_item(
                "Confirm fixture proof remains temporary-only.",
                ManualSmokeReviewItemResult::FixtureProven,
                "fixture proof summary represents temporary-only backup, write, verification, and recovery proof",
                None,
                false,
            ),
            review_item(
                "Confirm advanced confirmation is inactive.",
                ManualSmokeReviewItemResult::SourceProven,
                "PRODUCTION_ADVANCED_CONFIRMATION_ENABLED remains false",
                None,
                false,
            ),
            review_item(
                "Confirm high-risk approval is inactive.",
                ManualSmokeReviewItemResult::SourceProven,
                "PRODUCTION_HIGH_RISK_APPROVAL_ENABLED remains false",
                None,
                false,
            ),
            review_item(
                "Confirm real writing is not active.",
                ManualSmokeReviewItemResult::SourceProven,
                "all production write gates remain false and Apply is disconnected from pilot review models",
                None,
                false,
            ),
            review_item(
                "Confirm Apply behavior has not changed.",
                ManualSmokeReviewItemResult::SourceProven,
                "write_flow.rs still owns apply_setting_change and apply_scalar_write_plan",
                None,
                false,
            ),
            review_item(
                "Visually inspect rendered GTK detail pane.",
                ManualSmokeReviewItemResult::NotReviewed,
                "not launched in this sprint to avoid any chance of interacting with Apply or runtime state",
                Some("live visual inspection remains manual-only evidence"),
                true,
            ),
        ],
        gate_flip_proposal_ready: false,
        production_activation_ready: false,
    }
}

fn review_item(
    item_label: &'static str,
    result: ManualSmokeReviewItemResult,
    evidence: &'static str,
    blocking_reason: Option<&'static str>,
    blocks_gate_flip_proposal_readiness: bool,
) -> ManualSmokeReviewItem {
    ManualSmokeReviewItem {
        item_label,
        result,
        evidence,
        blocking_reason,
        blocks_gate_flip_proposal_readiness,
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SafeSmokeReviewEvidence {
    pub requirement: &'static str,
    pub evidence: &'static str,
    pub result: ManualSmokeReviewItemResult,
}

pub fn one_target_pilot_safe_smoke_review_evidence() -> Vec<SafeSmokeReviewEvidence> {
    vec![
        evidence("normal scalar row", "one-target pilot model constrains the path to one existing scalar line"),
        evidence("normal file", "target-risk classification marks only a normal non-generated, non-script, non-symlink file as safe for the first pilot"),
        evidence("generated/script/symlink exclusions", "first-pilot exclusion policy blocks generated, script-managed, script-referenced, symlink-managed, and symlink targets"),
        evidence("high-risk exclusion", "high-risk approval boundary excludes high-risk rows and keeps approval inactive"),
        evidence("line number known", "candidate eligibility requires a present line number"),
        evidence("backup contract", "production backup contract and disabled UI copy require exact-file backup"),
        evidence("verification contract", "production verification contract and disabled UI copy require reread verification"),
        evidence("recovery contract", "production recovery contract and disabled UI copy require exact-byte restore and reread"),
        evidence("advanced confirmation inactive", "PRODUCTION_ADVANCED_CONFIRMATION_ENABLED is false"),
        evidence("high-risk approval inactive", "PRODUCTION_HIGH_RISK_APPROVAL_ENABLED is false"),
        evidence("real writing inactive", "all production write gates remain false"),
        evidence("Apply unchanged", "write_flow.rs still contains apply_setting_change and apply_scalar_write_plan and does not import manual review models"),
    ]
}

fn evidence(requirement: &'static str, evidence: &'static str) -> SafeSmokeReviewEvidence {
    SafeSmokeReviewEvidence {
        requirement,
        evidence,
        result: ManualSmokeReviewItemResult::SourceProven,
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum GateFlipProposalDecision {
    ReadyForGateFlipProposal,
    NotReadyForGateFlipProposal,
    ReadyForProductionImplementationSprint,
    NotReadyForProductionImplementationSprint,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ManualReviewCompletion {
    Complete,
    Partial,
    Missing,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct GateFlipProposalReadiness {
    pub decision: GateFlipProposalDecision,
    pub manual_review_completion: ManualReviewCompletion,
    pub ready_for_gate_flip_proposal: bool,
    pub ready_for_production_implementation_sprint: bool,
    pub production_activation_ready: bool,
    pub reasons: Vec<&'static str>,
    pub remaining_blockers: Vec<OneTargetPilotRemainingBlocker>,
    pub recommended_next_sprint: &'static str,
}

pub fn one_target_pilot_gate_flip_proposal_readiness() -> GateFlipProposalReadiness {
    let blockers = one_target_pilot_remaining_blockers();
    GateFlipProposalReadiness {
        decision: GateFlipProposalDecision::NotReadyForGateFlipProposal,
        manual_review_completion: ManualReviewCompletion::Partial,
        ready_for_gate_flip_proposal: false,
        ready_for_production_implementation_sprint: false,
        production_activation_ready: false,
        reasons: vec![
            "live GTK visual smoke was not performed",
            "manual smoke review remains source-only and partial",
            "production backup/write/reread/recovery are inactive",
            "Apply integration is not approved",
            "all write activation gates remain false",
        ],
        remaining_blockers: blockers,
        recommended_next_sprint:
            "Run a controlled live read-only visual smoke review and draft a separate gate-flip proposal only if that review passes.",
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct OneTargetPilotRemainingBlocker {
    pub blocker_id: &'static str,
    pub description: &'static str,
    pub blocks_gate_flip_proposal: bool,
    pub blocks_production_activation: bool,
    pub required_next_proof: &'static str,
}

pub fn one_target_pilot_remaining_blockers() -> Vec<OneTargetPilotRemainingBlocker> {
    vec![
        blocker(
            "manual-smoke-source-only",
            "Manual smoke review was source-reviewed, not visually run in GTK.",
            true,
            true,
            "Controlled read-only visual inspection without clicking Apply.",
        ),
        blocker(
            "production-backup-inactive",
            "Production exact-file backup implementation remains inactive.",
            false,
            true,
            "Production backup implementation and exact-byte verification proof.",
        ),
        blocker(
            "production-write-inactive",
            "Production write implementation for the one-target pilot remains inactive.",
            true,
            true,
            "Separate gate-flip proposal and production write integration review.",
        ),
        blocker(
            "production-reread-verification-inactive",
            "Production reread verification remains inactive.",
            false,
            true,
            "Production reread verification implementation proof.",
        ),
        blocker(
            "production-recovery-inactive",
            "Production recovery remains inactive.",
            false,
            true,
            "Production exact-byte restore and restored-file verification proof.",
        ),
        blocker(
            "apply-integration-not-approved",
            "Apply integration is not approved for the pilot path.",
            true,
            true,
            "Explicit Apply integration boundary approval in a separate sprint.",
        ),
        blocker(
            "all-write-activation-gates-false",
            "All production write activation gates remain false.",
            true,
            true,
            "Separate proposal documenting the exact gate set and proof for any flip.",
        ),
        blocker(
            "release-gate-flip-proposal-not-created",
            "No separate future gate-flip proposal has been created.",
            true,
            true,
            "A dedicated proposal sprint that still does not combine gate flip with unrelated work.",
        ),
    ]
}

fn blocker(
    blocker_id: &'static str,
    description: &'static str,
    blocks_gate_flip_proposal: bool,
    blocks_production_activation: bool,
    required_next_proof: &'static str,
) -> OneTargetPilotRemainingBlocker {
    OneTargetPilotRemainingBlocker {
        blocker_id,
        description,
        blocks_gate_flip_proposal,
        blocks_production_activation,
        required_next_proof,
    }
}

pub fn one_target_pilot_gate_inventory_verification() -> Vec<OneTargetPilotGateSnapshotItem> {
    one_target_pilot_gate_inventory_snapshot()
}

pub fn all_production_gates_remain_false() -> bool {
    !PRODUCTION_ONE_TARGET_PRE_ENABLE_AUDIT_PASSED
        && !PRODUCTION_ONE_TARGET_WRITE_PILOT_ENABLED
        && !PRODUCTION_WRITE_TARGET_SELECTION_READY
        && !PRODUCTION_WRITE_TARGET_REVIEW_ENABLED
        && !PRODUCTION_WRITE_REVIEW_WALKTHROUGH_CAN_WRITE
        && !PRODUCTION_BACKUP_CONTRACT_ENABLED
        && !PRODUCTION_VERIFICATION_CONTRACT_ENABLED
        && !PRODUCTION_RECOVERY_CONTRACT_ENABLED
        && !PRODUCTION_ADVANCED_CONFIRMATION_ENABLED
        && !PRODUCTION_HIGH_RISK_APPROVAL_ENABLED
}

pub fn all_write_activation_gates_remain_false() -> bool {
    all_write_execution_gates_remain_false()
}

pub fn all_write_execution_gates_remain_false() -> bool {
    !PRODUCTION_ONE_TARGET_WRITE_PILOT_ENABLED
        && !PRODUCTION_ADVANCED_CONFIRMATION_ENABLED
        && !PRODUCTION_HIGH_RISK_APPROVAL_ENABLED
}

pub fn safe_batch_write_execution_gate_is_true() -> bool {
    PRODUCTION_WRITE_REVIEW_WALKTHROUGH_CAN_WRITE
}

pub fn only_pre_enable_audit_gate_is_true() -> bool {
    PRODUCTION_ONE_TARGET_PRE_ENABLE_AUDIT_PASSED
        && !PRODUCTION_BACKUP_CONTRACT_ENABLED
        && all_write_execution_gates_remain_false()
}

pub fn pre_enable_and_backup_gates_are_true() -> bool {
    PRODUCTION_ONE_TARGET_PRE_ENABLE_AUDIT_PASSED
        && PRODUCTION_BACKUP_CONTRACT_ENABLED
        && all_write_execution_gates_remain_false()
}

pub fn pre_enable_backup_and_verification_gates_are_true() -> bool {
    PRODUCTION_ONE_TARGET_PRE_ENABLE_AUDIT_PASSED
        && PRODUCTION_BACKUP_CONTRACT_ENABLED
        && PRODUCTION_VERIFICATION_CONTRACT_ENABLED
        && all_write_execution_gates_remain_false()
}

pub fn nonwriting_prerequisite_gates_are_true() -> bool {
    PRODUCTION_ONE_TARGET_PRE_ENABLE_AUDIT_PASSED
        && PRODUCTION_BACKUP_CONTRACT_ENABLED
        && PRODUCTION_VERIFICATION_CONTRACT_ENABLED
        && PRODUCTION_RECOVERY_CONTRACT_ENABLED
        && PRODUCTION_WRITE_TARGET_REVIEW_ENABLED
        && PRODUCTION_WRITE_TARGET_SELECTION_READY
        && safe_batch_write_execution_gate_is_true()
        && all_write_execution_gates_remain_false()
}

pub fn production_write_path_remains_disabled() -> bool {
    false
}

pub fn disabled_manual_smoke_review_ui_lines() -> Vec<String> {
    vec![
        "Manual smoke review".to_string(),
        "Manual review is represented, but this sprint does not enable writes.".to_string(),
        "A separate future proposal is required before any gate can flip.".to_string(),
        "All production write gates are still disabled.".to_string(),
        "Real writing is not active yet.".to_string(),
        "Apply behavior has not changed.".to_string(),
    ]
}
