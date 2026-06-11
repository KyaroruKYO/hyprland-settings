use crate::one_target_pilot_manual_review::{
    all_production_gates_remain_false, one_target_pilot_remaining_blockers,
    OneTargetPilotRemainingBlocker,
};
use crate::one_target_pilot_pre_enable_audit::{
    one_target_pilot_gate_inventory_snapshot, OneTargetPilotGateSnapshotItem,
};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum VisualSmokeItemResult {
    Passed,
    Failed,
    NotSeen,
    Inconclusive,
    NotApplicable,
}

impl VisualSmokeItemResult {
    pub fn label(self) -> &'static str {
        match self {
            Self::Passed => "passed",
            Self::Failed => "failed",
            Self::NotSeen => "not seen",
            Self::Inconclusive => "inconclusive",
            Self::NotApplicable => "not applicable",
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum LiveVisualSmokeReviewStatus {
    NotPerformed,
    Passed,
    Failed,
    Inconclusive,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct LiveVisualSmokeReviewPlan {
    pub launch_command: &'static str,
    pub screens_to_inspect: Vec<&'static str>,
    pub settings_path_to_inspect: &'static str,
    pub search_terms: Vec<&'static str>,
    pub expected_visible_copy: Vec<&'static str>,
    pub expected_disabled_controls: Vec<&'static str>,
    pub forbidden_actions: Vec<&'static str>,
    pub exit_cleanup_steps: Vec<&'static str>,
}

pub fn one_target_pilot_live_visual_smoke_review_plan() -> LiveVisualSmokeReviewPlan {
    LiveVisualSmokeReviewPlan {
        launch_command: "timeout-bounded cargo run --quiet with no CLI write subcommand",
        screens_to_inspect: vec![
            "Dashboard",
            "Config page",
            "Connected files section",
            "Profiles section",
            "Future changes section",
            "normal settings category",
            "setting detail pane",
            "production review section if reachable",
        ],
        settings_path_to_inspect:
            "normal category, known safe scalar row, detail pane production review section",
        search_terms: vec!["general.layout", "gaps", "border"],
        expected_visible_copy: vec![
            "Write review walkthrough",
            "Production write enablement",
            "First production write pilot",
            "Production backup and verification",
            "Recovery",
            "Advanced confirmation",
            "High-risk approval",
            "Final pre-enable audit",
            "Manual smoke review",
            "Real writing is not active yet.",
            "Apply behavior has not changed.",
        ],
        expected_disabled_controls: vec![
            "Choose review mode",
            "Profile switching planned",
            "Review save location",
            "Production enablement is disabled",
            "target decisions preview-only",
        ],
        forbidden_actions: vec![
            "Do not click Apply.",
            "Do not select or confirm a real write target.",
            "Do not run mode-switch scripts.",
            "Do not reload Hyprland.",
            "Do not run mutating hyprctl.",
            "Do not create backups or restores for user config files.",
        ],
        exit_cleanup_steps: vec![
            "Close or interrupt the app after inspection.",
            "Delete any temporary screenshot that captures unrelated desktop content.",
        ],
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct LiveVisualSmokeReviewItem {
    pub item_label: &'static str,
    pub result: VisualSmokeItemResult,
    pub evidence: &'static str,
    pub blocking_reason: Option<&'static str>,
    pub screenshot_or_manual_reference: Option<&'static str>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct LiveVisualSmokeReviewResult {
    pub visual_review_performed: bool,
    pub app_launched: bool,
    pub status: LiveVisualSmokeReviewStatus,
    pub screens_inspected: Vec<LiveVisualSmokeReviewItem>,
    pub expected_copy_results: Vec<LiveVisualSmokeReviewItem>,
    pub disabled_control_results: Vec<LiveVisualSmokeReviewItem>,
    pub unsafe_actions_avoided: Vec<LiveVisualSmokeReviewItem>,
    pub warnings_observed: Vec<&'static str>,
    pub visual_review_passed: bool,
    pub visual_review_failed: bool,
    pub visual_review_inconclusive: bool,
    pub screenshot_retained: bool,
}

pub fn one_target_pilot_live_visual_smoke_review_result() -> LiveVisualSmokeReviewResult {
    LiveVisualSmokeReviewResult {
        visual_review_performed: true,
        app_launched: true,
        status: LiveVisualSmokeReviewStatus::Inconclusive,
        screens_inspected: vec![
            review_item(
                "Dashboard",
                VisualSmokeItemResult::Passed,
                "bounded live launch showed the Hyprland Settings Dashboard window",
                None,
                Some("manual observation; temporary full-screen screenshot was deleted"),
            ),
            review_item(
                "Config page",
                VisualSmokeItemResult::NotSeen,
                "no interactive navigation evidence was captured for the Config page",
                Some("Config page still needs live visual confirmation"),
                None,
            ),
            review_item(
                "Connected files section",
                VisualSmokeItemResult::NotSeen,
                "connected file UI remains source-proven only in this sprint",
                Some("connected files section was not visually confirmed"),
                None,
            ),
            review_item(
                "Profiles section",
                VisualSmokeItemResult::NotSeen,
                "profile UI was not opened; no profile action was triggered",
                Some("profile section was not visually confirmed"),
                None,
            ),
            review_item(
                "Future changes section",
                VisualSmokeItemResult::NotSeen,
                "future changes copy remains source-proven only",
                Some("future changes section was not visually confirmed"),
                None,
            ),
            review_item(
                "normal settings category",
                VisualSmokeItemResult::NotSeen,
                "Dashboard category cards were visible, but no category was opened",
                Some("normal category detail list still needs live confirmation"),
                None,
            ),
            review_item(
                "setting detail pane",
                VisualSmokeItemResult::NotSeen,
                "no setting row was selected during the live review",
                Some("detail pane production review section was not visually confirmed"),
                None,
            ),
        ],
        expected_copy_results: vec![
            review_item(
                "Dashboard",
                VisualSmokeItemResult::Passed,
                "Dashboard title and category cards were visible",
                None,
                Some("manual observation"),
            ),
            review_item(
                "Manual smoke review",
                VisualSmokeItemResult::NotSeen,
                "the production review detail section was not reached",
                Some("manual smoke copy remains source-proven, not visually confirmed"),
                None,
            ),
            review_item(
                "Real writing is not active yet.",
                VisualSmokeItemResult::NotSeen,
                "the production review detail section was not reached",
                Some("real-writing inactive copy remains source-proven"),
                None,
            ),
            review_item(
                "Apply behavior has not changed.",
                VisualSmokeItemResult::NotSeen,
                "the production review detail section was not reached",
                Some("Apply unchanged copy remains source-proven"),
                None,
            ),
        ],
        disabled_control_results: vec![
            review_item(
                "Review save location",
                VisualSmokeItemResult::NotSeen,
                "the production review detail section was not reached",
                Some("disabled save-location control still needs live visual confirmation"),
                None,
            ),
            review_item(
                "Production enablement is disabled",
                VisualSmokeItemResult::NotSeen,
                "the production review detail section was not reached",
                Some("disabled production enablement control still needs live visual confirmation"),
                None,
            ),
            review_item(
                "target decisions preview-only",
                VisualSmokeItemResult::NotSeen,
                "the production review detail section was not reached",
                Some("disabled target decision control still needs live visual confirmation"),
                None,
            ),
        ],
        unsafe_actions_avoided: vec![
            review_item(
                "Apply avoided",
                VisualSmokeItemResult::Passed,
                "no setting row was selected and Apply was not clicked",
                None,
                None,
            ),
            review_item(
                "No real write target selected",
                VisualSmokeItemResult::Passed,
                "no save-location or target decision control was clicked",
                None,
                None,
            ),
            review_item(
                "Runtime/config mutation avoided",
                VisualSmokeItemResult::Passed,
                "normal app launch only; no mode-switch scripts, reloads, or mutating hyprctl were run",
                None,
                None,
            ),
            review_item(
                "Screenshot not retained",
                VisualSmokeItemResult::Passed,
                "temporary screenshot captured unrelated desktop content and was deleted",
                None,
                None,
            ),
        ],
        warnings_observed: vec![
            "Adwaita warning about unsupported GtkSettings dark-theme flag",
            "Vulkan driver conformance warning from the environment",
        ],
        visual_review_passed: false,
        visual_review_failed: false,
        visual_review_inconclusive: true,
        screenshot_retained: false,
    }
}

fn review_item(
    item_label: &'static str,
    result: VisualSmokeItemResult,
    evidence: &'static str,
    blocking_reason: Option<&'static str>,
    screenshot_or_manual_reference: Option<&'static str>,
) -> LiveVisualSmokeReviewItem {
    LiveVisualSmokeReviewItem {
        item_label,
        result,
        evidence,
        blocking_reason,
        screenshot_or_manual_reference,
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum VisualGateFlipProposalDecision {
    ReadyForSeparateGateFlipProposal,
    NotReadyForSeparateGateFlipProposal,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct VisualGateFlipProposalReadiness {
    pub decision: VisualGateFlipProposalDecision,
    pub ready_for_separate_gate_flip_proposal: bool,
    pub production_activation_ready: bool,
    pub reasons: Vec<&'static str>,
    pub remaining_blockers: Vec<OneTargetPilotRemainingBlocker>,
    pub proposal_scope_if_ready: Option<&'static str>,
    pub proposal_scope_if_not_ready: &'static str,
}

pub fn one_target_pilot_visual_gate_flip_proposal_readiness(
    result: &LiveVisualSmokeReviewResult,
) -> VisualGateFlipProposalReadiness {
    let ready = result.status == LiveVisualSmokeReviewStatus::Passed
        && result.visual_review_passed
        && all_production_gates_remain_false();
    VisualGateFlipProposalReadiness {
        decision: if ready {
            VisualGateFlipProposalDecision::ReadyForSeparateGateFlipProposal
        } else {
            VisualGateFlipProposalDecision::NotReadyForSeparateGateFlipProposal
        },
        ready_for_separate_gate_flip_proposal: ready,
        production_activation_ready: false,
        reasons: if ready {
            vec![
                "live visual smoke review passed",
                "separate proposal would still be required",
                "no gate is flipped by this review model",
            ]
        } else {
            vec![
                "live visual smoke review was inconclusive",
                "detail-pane production review copy was not visually confirmed",
                "disabled production controls were not visually confirmed",
                "all production gates remain false",
            ]
        },
        remaining_blockers: one_target_pilot_visual_review_remaining_blockers(result),
        proposal_scope_if_ready: ready.then_some(
            "draft-only future proposal for the narrow normal scalar one-target pilot gate set",
        ),
        proposal_scope_if_not_ready:
            "repeat live read-only visual review with detail-pane and disabled-control evidence",
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct GateFlipProposalDraft {
    pub draft_only: bool,
    pub no_gate_flipped: bool,
    pub requires_user_approval: bool,
    pub requires_separate_sprint: bool,
    pub exact_gates_proposed_for_future_flip: Vec<&'static str>,
    pub target_class_allowed: &'static str,
    pub target_classes_excluded: Vec<&'static str>,
    pub stop_conditions: Vec<&'static str>,
}

pub fn one_target_pilot_gate_flip_proposal_draft(
    result: &LiveVisualSmokeReviewResult,
) -> Option<GateFlipProposalDraft> {
    if result.status != LiveVisualSmokeReviewStatus::Passed || !result.visual_review_passed {
        return None;
    }

    Some(GateFlipProposalDraft {
        draft_only: true,
        no_gate_flipped: true,
        requires_user_approval: true,
        requires_separate_sprint: true,
        exact_gates_proposed_for_future_flip: vec![
            "PRODUCTION_ONE_TARGET_PRE_ENABLE_AUDIT_PASSED",
            "PRODUCTION_ONE_TARGET_WRITE_PILOT_ENABLED",
        ],
        target_class_allowed: "one existing non-high-risk scalar line in one normal config file",
        target_classes_excluded: vec![
            "generated targets",
            "script-managed targets",
            "script-referenced targets",
            "symlink-managed targets",
            "symlink targets",
            "high-risk rows",
            "structured targets",
            "missing-line targets",
            "duplicate or ambiguous targets",
        ],
        stop_conditions: vec![
            "any production gate remains unapproved",
            "backup, verification, or recovery proof is missing",
            "Apply integration is not explicitly approved",
            "manual visual smoke evidence is incomplete",
        ],
    })
}

pub fn one_target_pilot_visual_review_remaining_blockers(
    result: &LiveVisualSmokeReviewResult,
) -> Vec<OneTargetPilotRemainingBlocker> {
    let mut blockers = one_target_pilot_remaining_blockers();
    if result.status == LiveVisualSmokeReviewStatus::Passed && result.visual_review_passed {
        blockers.retain(|blocker| blocker.blocker_id != "manual-smoke-source-only");
    }
    if result.status != LiveVisualSmokeReviewStatus::Passed || !result.visual_review_passed {
        blockers.insert(
            0,
            OneTargetPilotRemainingBlocker {
                blocker_id: "live-visual-smoke-inconclusive",
                description:
                    "Live visual smoke launched the app but did not confirm detail-pane production review copy or disabled controls.",
                blocks_gate_flip_proposal: true,
                blocks_production_activation: true,
                required_next_proof:
                    "Repeat read-only visual review with Config page, detail pane, expected copy, and disabled-control evidence.",
            },
        );
    }
    blockers
}

pub fn one_target_pilot_live_visual_gate_inventory_verification(
) -> Vec<OneTargetPilotGateSnapshotItem> {
    one_target_pilot_gate_inventory_snapshot()
}

pub fn disabled_live_visual_smoke_review_ui_lines() -> Vec<String> {
    vec![
        "Live visual smoke review".to_string(),
        "The visual review is recorded, but this sprint does not enable writes.".to_string(),
        "A separate future proposal is still required before any gate can flip.".to_string(),
        "All production write gates are still disabled.".to_string(),
        "Real writing is not active yet.".to_string(),
        "Apply behavior has not changed.".to_string(),
    ]
}
