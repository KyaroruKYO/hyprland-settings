use crate::one_target_pilot_live_visual_smoke::{
    GateFlipProposalDraft, LiveVisualSmokeReviewItem, LiveVisualSmokeReviewStatus,
    VisualGateFlipProposalDecision, VisualGateFlipProposalReadiness, VisualSmokeItemResult,
};
use crate::one_target_pilot_manual_review::{
    all_production_gates_remain_false, one_target_pilot_remaining_blockers,
    OneTargetPilotRemainingBlocker,
};
use crate::one_target_pilot_pre_enable_audit::{
    one_target_pilot_gate_inventory_snapshot, OneTargetPilotGateSnapshotItem,
};

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct FocusedVisualSmokePlan {
    pub safe_launch_command: &'static str,
    pub read_only_method: &'static str,
    pub app_window_evidence_strategy: &'static str,
    pub screens_to_inspect: Vec<&'static str>,
    pub navigation_path: Vec<&'static str>,
    pub search_terms: Vec<&'static str>,
    pub expected_copy: Vec<&'static str>,
    pub expected_disabled_controls: Vec<&'static str>,
    pub forbidden_actions: Vec<&'static str>,
    pub cleanup_steps: Vec<&'static str>,
}

pub fn one_target_pilot_focused_visual_smoke_plan() -> FocusedVisualSmokePlan {
    FocusedVisualSmokePlan {
        safe_launch_command: "bounded cargo run --quiet; no write subcommands",
        read_only_method: "navigate Dashboard, Config, category, and detail pane only; never click Apply",
        app_window_evidence_strategy:
            "prefer cropped app-window-only screenshot; delete temporary image if it contains local paths or unrelated desktop content",
        screens_to_inspect: vec![
            "Dashboard",
            "Config page",
            "Config file section",
            "Connected files section",
            "Profiles section",
            "Future changes section",
            "normal settings category",
            "safe scalar row detail pane",
            "production review section",
        ],
        navigation_path: vec![
            "open Dashboard",
            "open Config",
            "inspect Config file, Connected files, Profiles, and Future changes",
            "open a normal settings category",
            "open one scalar row detail pane",
            "inspect production review copy and disabled controls",
            "close the app",
        ],
        search_terms: vec!["blur", "gaps", "border"],
        expected_copy: vec![
            "Write review walkthrough",
            "Production write enablement",
            "First production write pilot",
            "Production backup and verification",
            "Recovery",
            "Advanced confirmation",
            "High-risk approval",
            "Final pre-enable audit",
            "Manual smoke review",
            "Live visual smoke review",
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
            "Do not click unclear controls.",
            "Do not interact with profile or mode switching controls except to inspect disabled state.",
            "Do not reload Hyprland.",
            "Do not run mutating hyprctl.",
            "Do not create backups or restores for user config files.",
        ],
        cleanup_steps: vec![
            "delete temporary screenshots that include unrelated desktop content or local config paths",
            "close or interrupt the app after inspection",
        ],
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct FocusedAppWindowEvidence {
    pub attempted: bool,
    pub captured: bool,
    pub retained: bool,
    pub safe_to_commit: bool,
    pub evidence_reference: &'static str,
    pub cleanup_status: &'static str,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct FocusedVisualSmokeResult {
    pub review_performed: bool,
    pub app_launched: bool,
    pub app_window_only_evidence: FocusedAppWindowEvidence,
    pub status: LiveVisualSmokeReviewStatus,
    pub screens_inspected: Vec<LiveVisualSmokeReviewItem>,
    pub expected_copy_results: Vec<LiveVisualSmokeReviewItem>,
    pub disabled_control_results: Vec<LiveVisualSmokeReviewItem>,
    pub unsafe_actions_avoided: Vec<LiveVisualSmokeReviewItem>,
    pub warnings_observed: Vec<&'static str>,
    pub review_passed: bool,
    pub review_failed: bool,
    pub review_inconclusive: bool,
    pub proposal_allowed: bool,
}

pub fn one_target_pilot_focused_visual_smoke_result() -> FocusedVisualSmokeResult {
    FocusedVisualSmokeResult {
        review_performed: true,
        app_launched: true,
        app_window_only_evidence: FocusedAppWindowEvidence {
            attempted: true,
            captured: true,
            retained: false,
            safe_to_commit: false,
            evidence_reference:
                "temporary app-window crop was inspected locally, then deleted because it included local config paths",
            cleanup_status: "all temporary focused-smoke screenshots were deleted and are not committed",
        },
        status: LiveVisualSmokeReviewStatus::Passed,
        screens_inspected: vec![
            focused_item(
                "Dashboard",
                VisualSmokeItemResult::Passed,
                "live accessibility tree confirmed Dashboard before navigation",
                None,
                Some("focused live read-only review"),
            ),
            focused_item(
                "Config page",
                VisualSmokeItemResult::Passed,
                "live accessibility tree confirmed Config route and Config file section",
                None,
                Some("focused live read-only review"),
            ),
            focused_item(
                "Connected files section",
                VisualSmokeItemResult::Passed,
                "live accessibility tree confirmed Connected files and connected-file read-only copy",
                None,
                Some("focused live read-only review"),
            ),
            focused_item(
                "Profiles section",
                VisualSmokeItemResult::Passed,
                "live accessibility tree confirmed Profiles and disabled planned profile control",
                None,
                Some("focused live read-only review"),
            ),
            focused_item(
                "Future changes section",
                VisualSmokeItemResult::Passed,
                "live accessibility tree confirmed Future changes section",
                None,
                Some("focused live read-only review"),
            ),
            focused_item(
                "normal settings category",
                VisualSmokeItemResult::Passed,
                "Appearance category opened and displayed 48 rows",
                None,
                Some("focused live read-only review"),
            ),
            focused_item(
                "setting detail pane",
                VisualSmokeItemResult::Passed,
                "Appearance Blur Enabled detail pane opened read-only",
                None,
                Some("focused live read-only review"),
            ),
            focused_item(
                "production review section",
                VisualSmokeItemResult::Passed,
                "detail pane showed write review, backup, verification, recovery, advanced confirmation, high-risk, audit, and smoke-review copy",
                None,
                Some("focused live read-only review"),
            ),
        ],
        expected_copy_results: vec![
            focused_item(
                "Write review walkthrough",
                VisualSmokeItemResult::Passed,
                "visible in detail-pane production review section",
                None,
                None,
            ),
            focused_item(
                "Production write enablement",
                VisualSmokeItemResult::Passed,
                "visible in detail-pane production review section",
                None,
                None,
            ),
            focused_item(
                "First production write pilot",
                VisualSmokeItemResult::Passed,
                "visible in detail-pane production review section",
                None,
                None,
            ),
            focused_item(
                "Production backup and verification",
                VisualSmokeItemResult::Passed,
                "visible in detail-pane production review section",
                None,
                None,
            ),
            focused_item(
                "Recovery",
                VisualSmokeItemResult::Passed,
                "visible in detail-pane production review section",
                None,
                None,
            ),
            focused_item(
                "Advanced confirmation",
                VisualSmokeItemResult::Passed,
                "visible in detail-pane production review section",
                None,
                None,
            ),
            focused_item(
                "High-risk approval",
                VisualSmokeItemResult::Passed,
                "visible in detail-pane production review section",
                None,
                None,
            ),
            focused_item(
                "Final pre-enable audit",
                VisualSmokeItemResult::Passed,
                "visible in detail-pane production review section",
                None,
                None,
            ),
            focused_item(
                "Manual smoke review",
                VisualSmokeItemResult::Passed,
                "visible in detail-pane production review section",
                None,
                None,
            ),
            focused_item(
                "Live visual smoke review",
                VisualSmokeItemResult::Passed,
                "visible in detail-pane production review section",
                None,
                None,
            ),
            focused_item(
                "Real writing is not active yet.",
                VisualSmokeItemResult::Passed,
                "visible in detail-pane production review section",
                None,
                None,
            ),
            focused_item(
                "Apply behavior has not changed.",
                VisualSmokeItemResult::Passed,
                "visible in detail-pane production review section",
                None,
                None,
            ),
        ],
        disabled_control_results: vec![
            focused_item(
                "Choose review mode",
                VisualSmokeItemResult::Passed,
                "Config page control was present and insensitive",
                None,
                Some("AT-SPI state: sensitive=false, enabled=false"),
            ),
            focused_item(
                "Profile switching planned",
                VisualSmokeItemResult::Passed,
                "Config page control was present and insensitive",
                None,
                Some("AT-SPI state: sensitive=false, enabled=false"),
            ),
            focused_item(
                "Review save location",
                VisualSmokeItemResult::Passed,
                "detail-pane production review control was present and insensitive",
                None,
                Some("AT-SPI state: sensitive=false, enabled=false"),
            ),
            focused_item(
                "Production enablement is disabled",
                VisualSmokeItemResult::Passed,
                "detail-pane production review control was present and insensitive",
                None,
                Some("AT-SPI state: sensitive=false, enabled=false"),
            ),
            focused_item(
                "target decisions preview-only",
                VisualSmokeItemResult::Passed,
                "detail-pane production review control was present and insensitive",
                None,
                Some("AT-SPI state: sensitive=false, enabled=false"),
            ),
        ],
        unsafe_actions_avoided: vec![
            focused_item(
                "Apply avoided",
                VisualSmokeItemResult::Passed,
                "Apply button was not clicked during the focused review",
                None,
                None,
            ),
            focused_item(
                "No real write target selected",
                VisualSmokeItemResult::Passed,
                "no save-location or target decision control was clicked",
                None,
                None,
            ),
            focused_item(
                "Profile and mode controls not activated",
                VisualSmokeItemResult::Passed,
                "planned profile/review controls were inspected only for disabled state",
                None,
                None,
            ),
            focused_item(
                "Runtime/config mutation avoided",
                VisualSmokeItemResult::Passed,
                "no reload, mode switch, mutating hyprctl, backup, restore, or config write was run",
                None,
                None,
            ),
        ],
        warnings_observed: vec![
            "Adwaita warning about unsupported GtkSettings dark-theme flag",
            "Vulkan driver conformance warning from the environment",
            "AT-SPI dbind warning while querying accessibility tree; review evidence was still obtained",
        ],
        review_passed: true,
        review_failed: false,
        review_inconclusive: false,
        proposal_allowed: true,
    }
}

fn focused_item(
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

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct FocusedVisualPassCriteria {
    pub dashboard_confirmed: bool,
    pub config_page_confirmed: bool,
    pub connected_files_confirmed: bool,
    pub normal_category_confirmed: bool,
    pub detail_pane_confirmed: bool,
    pub production_review_copy_confirmed: bool,
    pub disabled_production_controls_confirmed: bool,
    pub unsafe_actions_avoided: bool,
    pub all_gates_remain_false: bool,
}

impl FocusedVisualPassCriteria {
    pub fn passes(&self) -> bool {
        self.dashboard_confirmed
            && self.config_page_confirmed
            && self.connected_files_confirmed
            && self.normal_category_confirmed
            && self.detail_pane_confirmed
            && self.production_review_copy_confirmed
            && self.disabled_production_controls_confirmed
            && self.unsafe_actions_avoided
            && self.all_gates_remain_false
    }
}

pub fn one_target_pilot_focused_visual_pass_criteria(
    result: &FocusedVisualSmokeResult,
) -> FocusedVisualPassCriteria {
    let screen_passed = |label: &str| {
        result
            .screens_inspected
            .iter()
            .any(|item| item.item_label == label && item.result == VisualSmokeItemResult::Passed)
    };
    FocusedVisualPassCriteria {
        dashboard_confirmed: screen_passed("Dashboard"),
        config_page_confirmed: screen_passed("Config page"),
        connected_files_confirmed: screen_passed("Connected files section"),
        normal_category_confirmed: screen_passed("normal settings category"),
        detail_pane_confirmed: screen_passed("setting detail pane"),
        production_review_copy_confirmed: screen_passed("production review section")
            && result
                .expected_copy_results
                .iter()
                .all(|item| item.result == VisualSmokeItemResult::Passed),
        disabled_production_controls_confirmed: result
            .disabled_control_results
            .iter()
            .all(|item| item.result == VisualSmokeItemResult::Passed),
        unsafe_actions_avoided: result
            .unsafe_actions_avoided
            .iter()
            .all(|item| item.result == VisualSmokeItemResult::Passed),
        all_gates_remain_false: all_production_gates_remain_false(),
    }
}

pub fn one_target_pilot_focused_visual_gate_flip_proposal_readiness(
    result: &FocusedVisualSmokeResult,
) -> VisualGateFlipProposalReadiness {
    let criteria = one_target_pilot_focused_visual_pass_criteria(result);
    let ready = result.status == LiveVisualSmokeReviewStatus::Passed
        && result.review_passed
        && result.proposal_allowed
        && criteria.passes();

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
                "focused live read-only visual smoke review passed",
                "Config page, normal category, detail pane, production copy, and disabled controls were confirmed",
                "separate future proposal is still required before any gate can flip",
                "no production activation happens in this sprint",
            ]
        } else {
            vec![
                "focused visual smoke review did not satisfy every pass criterion",
                "a proposal draft must not be created from failed or inconclusive visual evidence",
                "all production gates remain false",
            ]
        },
        remaining_blockers: one_target_pilot_focused_visual_remaining_blockers(result),
        proposal_scope_if_ready: ready.then_some(
            "draft-only future proposal for the narrow normal scalar one-target pilot gate set",
        ),
        proposal_scope_if_not_ready:
            "repeat focused read-only visual review until the missing visual evidence is confirmed",
    }
}

pub fn one_target_pilot_focused_gate_flip_proposal_draft(
    result: &FocusedVisualSmokeResult,
) -> Option<GateFlipProposalDraft> {
    if !one_target_pilot_focused_visual_gate_flip_proposal_readiness(result)
        .ready_for_separate_gate_flip_proposal
    {
        return None;
    }

    Some(GateFlipProposalDraft {
        draft_only: true,
        no_gate_flipped: true,
        requires_user_approval: true,
        requires_separate_sprint: true,
        exact_gates_proposed_for_future_flip: vec![
            "PRODUCTION_ONE_TARGET_PRE_ENABLE_AUDIT_PASSED",
            "PRODUCTION_WRITE_TARGET_SELECTION_READY",
            "PRODUCTION_WRITE_TARGET_REVIEW_ENABLED",
            "PRODUCTION_BACKUP_CONTRACT_ENABLED",
            "PRODUCTION_VERIFICATION_CONTRACT_ENABLED",
            "PRODUCTION_RECOVERY_CONTRACT_ENABLED",
            "PRODUCTION_ONE_TARGET_WRITE_PILOT_ENABLED",
            "PRODUCTION_WRITE_REVIEW_WALKTHROUGH_CAN_WRITE",
        ],
        target_class_allowed:
            "one existing non-high-risk scalar line in one normal config file with exact line number and no ambiguity",
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
            "any production gate remains unapproved in the future sprint",
            "backup cannot be created and byte-verified before write",
            "write target is not a normal scalar file line",
            "reread verification fails after write",
            "recovery restore or restore verification fails",
            "Apply integration changes outside the approved one-target scope",
        ],
    })
}

pub fn one_target_pilot_focused_visual_remaining_blockers(
    result: &FocusedVisualSmokeResult,
) -> Vec<OneTargetPilotRemainingBlocker> {
    let mut blockers = one_target_pilot_remaining_blockers();
    if result.status == LiveVisualSmokeReviewStatus::Passed && result.review_passed {
        blockers.retain(|blocker| blocker.blocker_id != "manual-smoke-source-only");
    } else {
        blockers.insert(
            0,
            OneTargetPilotRemainingBlocker {
                blocker_id: "focused-visual-smoke-incomplete",
                description:
                    "Focused live visual smoke review did not confirm every required screen, copy line, or disabled control.",
                blocks_gate_flip_proposal: true,
                blocks_production_activation: true,
                required_next_proof:
                    "Repeat focused read-only visual review with app-window-only or manual evidence for every pass criterion.",
            },
        );
    }

    if result.status == LiveVisualSmokeReviewStatus::Passed && result.review_passed {
        blockers.retain(|blocker| blocker.blocker_id != "release-gate-flip-proposal-not-created");
        blockers.push(OneTargetPilotRemainingBlocker {
            blocker_id: "gate-flip-proposal-draft-not-executed",
            description:
                "A separate proposal draft may exist, but no future gate-flip sprint has executed it.",
            blocks_gate_flip_proposal: false,
            blocks_production_activation: true,
            required_next_proof:
                "User-approved separate sprint that reviews and explicitly flips only approved gates.",
        });
    }

    blockers
}

pub fn one_target_pilot_focused_visual_gate_inventory_verification(
) -> Vec<OneTargetPilotGateSnapshotItem> {
    one_target_pilot_gate_inventory_snapshot()
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct FocusedVisualUiDecision {
    pub ui_added: bool,
    pub reason: &'static str,
    pub model_report_only: bool,
    pub controls_added: bool,
    pub handlers_added: bool,
}

pub fn focused_visual_smoke_ui_decision() -> FocusedVisualUiDecision {
    FocusedVisualUiDecision {
        ui_added: false,
        reason:
            "The production review pane already includes live visual smoke copy; this focused follow-up is recorded in model, report, and review log only to avoid making the detail pane denser.",
        model_report_only: true,
        controls_added: false,
        handlers_added: false,
    }
}
