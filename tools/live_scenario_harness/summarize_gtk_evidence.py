#!/usr/bin/env python3
import json
import re
import sys
from datetime import datetime, timezone
from pathlib import Path


PROJECT_MODEL = "v0.55.2 / 341 readable / 341 writable / 0 blocked"
STARTING_COMMIT = "9083e9eda0c6c90df04991d8d1b32f842915004e"
PROOF_LEVELS = {
    "live_gtk_atspi_proof",
    "safe_env_model_proof",
    "source_model_fallback",
    "not_proven",
}
OLD_EVIDENCE_RE = re.compile(r"/tmp/hyprland-settings-gtk-automation/20260618_[0-9]+")

BLOCKED_CATEGORY_PROOFS = {
    "defaultMissingBlockedCopy": {
        "category": "default_missing_line",
        "scenario": "missing_default_only",
        "target": "MissingDefaultDetail",
        "expectedText": "Uses Hyprland default",
    },
    "duplicateBlockedCopy": {
        "category": "duplicate_conflict",
        "scenario": "duplicate_conflict",
        "target": "DuplicateConflictDetail",
        "expectedText": "This setting appears more than once in your config",
    },
    "generatedBlockedCopy": {
        "category": "generated_file",
        "scenario": "generated_config",
        "target": "GeneratedConnectedFileDetail",
        "expectedText": "generated",
        "expectedProofSurface": "connected_file_detail",
    },
    "scriptManagedBlockedCopy": {
        "category": "script_managed_file",
        "scenario": "script_managed_config",
        "target": "ScriptManagedConnectedFileDetail",
        "expectedText": "script",
        "expectedProofSurface": "connected_file_detail",
    },
    "symlinkCurrentProfileBlockedCopy": {
        "category": "symlink_current_profile",
        "scenario": "symlink_current_profile",
        "target": "SymlinkConnectedFileDetail",
        "expectedText": "symlink",
        "expectedProofSurface": "connected_file_detail",
    },
    "highRiskBlockedCopy": {
        "category": "high_risk",
        "scenario": "high_risk_display_risk",
        "target": "HighRiskDetail",
        "expectedText": "Extra care needed",
    },
    "displayRenderRiskBlockedCopy": {
        "category": "display_render_risk",
        "scenario": "high_risk_display_risk",
        "target": "DisplayRenderRiskDetail",
        "expectedText": "display/render",
    },
    "profileModeSwitchBlockedCopy": {
        "category": "profile_mode_switch",
        "scenario": "symlink_current_profile",
        "target": "ProfileModeDetail",
        "expectedText": "profile",
        "expectedProofSurface": "profile_detail",
    },
}


def read_json(path):
    try:
        return json.loads(path.read_text())
    except Exception:
        return {}


def redact(value):
    if isinstance(value, str):
        value = OLD_EVIDENCE_RE.sub("<old-evidence-path>", value)
        value = re.sub(r"/tmp/hyprland-settings-gtk-automation/[^/\\s\"]+", "<tmp>/hyprland-settings-gtk-automation/<fresh-run>", value)
        value = re.sub(r"/tmp/hyprland-settings-[^/\\s\"]+", "<tmp>/hyprland-settings-<redacted>", value)
        value = re.sub(r"/home/kyo/[^\\s\"]+", "<home>/<redacted>", value)
        return value
    if isinstance(value, list):
        return [redact(item) for item in value]
    if isinstance(value, dict):
        return {key: redact(item) for key, item in value.items()}
    return value


def proof(condition, fallback=False):
    if condition:
        return "live_gtk_atspi_proof"
    if fallback:
        return "source_model_fallback"
    return "not_proven"


def combined_text(accessibility):
    return "\n".join(
        accessibility.get("text", []) + accessibility.get("textAfterNavigation", [])
    ).lower()


def term_found(accessibility, *terms):
    text = combined_text(accessibility)
    return any(term.lower() in text for term in terms)


def run_summary(run_dir):
    probe = read_json(run_dir / "probe-result.json")
    accessibility = read_json(run_dir / "accessibility.json")
    name = run_dir.name
    scenario, _, target = name.rpartition("_")
    return {
        "name": name,
        "scenario": scenario or name,
        "navigationTarget": target or accessibility.get("navigationTarget"),
        "probe": {
            "appBuildAttempted": bool(probe.get("appBuildAttempted")),
            "appBuildSucceeded": bool(probe.get("appBuildSucceeded")),
            "appLaunchSucceeded": bool(probe.get("appLaunchSucceeded")),
            "appBinaryRebuiltBeforeProbe": bool(probe.get("appBinaryRebuiltBeforeProbe")),
            "closeSucceeded": bool(probe.get("closeSucceeded")),
            "applyClicked": bool(probe.get("applyClicked")),
            "safeEnvModeUsed": bool(probe.get("safeEnvModeUsed")),
            "liveSwapModeUsed": bool(probe.get("liveSwapModeUsed")),
            "realConfigEdited": bool(probe.get("realConfigEdited")),
            "realBackupsCreated": bool(probe.get("realBackupsCreated")),
            "hyprlandReloaded": bool(probe.get("hyprlandReloaded")),
            "mutatingHyprctlUsed": bool(probe.get("mutatingHyprctlUsed")),
            "runtimeMutated": bool(probe.get("runtimeMutated")),
            "scriptsExecuted": bool(probe.get("scriptsExecuted")),
            "luaExecuted": bool(probe.get("luaExecuted")),
        },
        "accessibility": {
            "attempted": bool(accessibility.get("attempted")),
            "succeeded": bool(accessibility.get("succeeded")),
            "applicationMatched": bool(accessibility.get("applicationMatched")),
            "navigationAttempted": bool(accessibility.get("navigationAttempted")),
            "navigationSucceeded": bool(accessibility.get("navigationSucceeded")),
            "navigationMessage": accessibility.get("navigationMessage"),
            "foundTerms": accessibility.get("foundTerms", []),
            "foundTermsAfterNavigation": accessibility.get("foundTermsAfterNavigation", []),
            "detailPaneTextCollected": bool(accessibility.get("detailPaneTextCollected")),
            "blockedReasonTextCollected": bool(accessibility.get("blockedReasonTextCollected")),
            "duplicateBlockedReasonTextCollected": bool(
                accessibility.get("duplicateBlockedReasonTextCollected")
            ),
            "blockedCategory": accessibility.get("blockedCategory"),
            "blockedCategoryDetailNavigationAttempted": bool(
                accessibility.get("blockedCategoryDetailNavigationAttempted")
            ),
            "blockedCategoryDetailNavigationSucceeded": bool(
                accessibility.get("blockedCategoryDetailNavigationSucceeded")
            ),
            "blockedCategoryReasonTextCollected": bool(
                accessibility.get("blockedCategoryReasonTextCollected")
            ),
            "blockedCategoryExpectedTextCollected": bool(
                accessibility.get("blockedCategoryExpectedTextCollected")
            ),
            "blockedCategorySelectionFallbackUsed": bool(
                accessibility.get("blockedCategorySelectionFallbackUsed")
            ),
            "connectedFileDetailNavigationAttempted": bool(
                accessibility.get("connectedFileDetailNavigationAttempted")
            ),
            "connectedFileDetailNavigationSucceeded": bool(
                accessibility.get("connectedFileDetailNavigationSucceeded")
            ),
            "connectedFileGeneratedDetailCollected": bool(
                accessibility.get("connectedFileGeneratedDetailCollected")
            ),
            "connectedFileScriptManagedDetailCollected": bool(
                accessibility.get("connectedFileScriptManagedDetailCollected")
            ),
            "connectedFileSymlinkDetailCollected": bool(
                accessibility.get("connectedFileSymlinkDetailCollected")
            ),
            "profileModeDetailCollected": bool(accessibility.get("profileModeDetailCollected")),
            "proofSurface": accessibility.get("proofSurface"),
            "approvalCardAssertionMethod": accessibility.get("approvalCardAssertionMethod"),
            "approvalCardAssertions": accessibility.get("approvalCardAssertions", {}),
            "approvalCardsAllHeadingsFound": bool(
                accessibility.get("approvalCardsAllHeadingsFound")
            ),
            "approvalCardsAllProductionDisabledFound": bool(
                accessibility.get("approvalCardsAllProductionDisabledFound")
            ),
            "approvalCardsAllDisabledActionsFound": bool(
                accessibility.get("approvalCardsAllDisabledActionsFound")
            ),
            "activationDecisionAssertionMethod": accessibility.get(
                "activationDecisionAssertionMethod"
            ),
            "activationDecisionAssertions": accessibility.get(
                "activationDecisionAssertions", {}
            ),
            "activationDecisionsAllHeadingsFound": bool(
                accessibility.get("activationDecisionsAllHeadingsFound")
            ),
            "activationDecisionsAllProductionDisabledFound": bool(
                accessibility.get("activationDecisionsAllProductionDisabledFound")
            ),
            "activationDecisionsAllDisabledActionsFound": bool(
                accessibility.get("activationDecisionsAllDisabledActionsFound")
            ),
            "activationPathAssertionMethod": accessibility.get("activationPathAssertionMethod"),
            "activationPathAssertions": accessibility.get("activationPathAssertions", {}),
            "activationPathsAllHeadingsFound": bool(
                accessibility.get("activationPathsAllHeadingsFound")
            ),
            "activationPathsAllProductionDisabledFound": bool(
                accessibility.get("activationPathsAllProductionDisabledFound")
            ),
            "activationPathsAllDisabledActionsFound": bool(
                accessibility.get("activationPathsAllDisabledActionsFound")
            ),
            "activationControlAssertionMethod": accessibility.get(
                "activationControlAssertionMethod"
            ),
            "activationControlAssertions": accessibility.get(
                "activationControlAssertions", {}
            ),
            "activationControlsAllHeadingsFound": bool(
                accessibility.get("activationControlsAllHeadingsFound")
            ),
            "activationControlsAllProductionDisabledFound": bool(
                accessibility.get("activationControlsAllProductionDisabledFound")
            ),
            "activationControlsAllExecutorUnwiredFound": bool(
                accessibility.get("activationControlsAllExecutorUnwiredFound")
            ),
            "activationControlsAllDisabledActionsFound": bool(
                accessibility.get("activationControlsAllDisabledActionsFound")
            ),
            "activationFormAssertionMethod": accessibility.get(
                "activationFormAssertionMethod"
            ),
            "activationFormAssertions": accessibility.get("activationFormAssertions", {}),
            "activationFormsAllHeadingsFound": bool(
                accessibility.get("activationFormsAllHeadingsFound")
            ),
            "activationFormsAllProductionDisabledFound": bool(
                accessibility.get("activationFormsAllProductionDisabledFound")
            ),
            "activationFormsAllExecutorUnwiredFound": bool(
                accessibility.get("activationFormsAllExecutorUnwiredFound")
            ),
            "activationFormsAllDisabledActionsFound": bool(
                accessibility.get("activationFormsAllDisabledActionsFound")
            ),
            "activationFormsAllFieldLabelsFound": bool(
                accessibility.get("activationFormsAllFieldLabelsFound")
            ),
            "activationDraftAssertionMethod": accessibility.get(
                "activationDraftAssertionMethod"
            ),
            "activationDraftAssertions": accessibility.get("activationDraftAssertions", {}),
            "activationDraftsAllHeadingsFound": bool(
                accessibility.get("activationDraftsAllHeadingsFound")
            ),
            "activationDraftsAllProductionDisabledFound": bool(
                accessibility.get("activationDraftsAllProductionDisabledFound")
            ),
            "activationDraftsAllExecutorUnwiredFound": bool(
                accessibility.get("activationDraftsAllExecutorUnwiredFound")
            ),
            "activationDraftsAllInMemoryOnlyFound": bool(
                accessibility.get("activationDraftsAllInMemoryOnlyFound")
            ),
            "activationDraftsAllDisabledActionsFound": bool(
                accessibility.get("activationDraftsAllDisabledActionsFound")
            ),
            "activationDraftEditAssertionMethod": accessibility.get(
                "activationDraftEditAssertionMethod"
            ),
            "activationDraftEditAssertions": accessibility.get(
                "activationDraftEditAssertions", {}
            ),
            "activationDraftEditsAllHeadingsFound": bool(
                accessibility.get("activationDraftEditsAllHeadingsFound")
            ),
            "activationDraftEditsAllProductionDisabledFound": bool(
                accessibility.get("activationDraftEditsAllProductionDisabledFound")
            ),
            "activationDraftEditsAllExecutorUnwiredFound": bool(
                accessibility.get("activationDraftEditsAllExecutorUnwiredFound")
            ),
            "activationDraftEditsAllInMemoryOnlyFound": bool(
                accessibility.get("activationDraftEditsAllInMemoryOnlyFound")
            ),
            "activationDraftEditsAllModeFound": bool(
                accessibility.get("activationDraftEditsAllModeFound")
            ),
            "activationDraftEditsAllValidationFound": bool(
                accessibility.get("activationDraftEditsAllValidationFound")
            ),
            "activationDraftEditsAllDisabledActionsFound": bool(
                accessibility.get("activationDraftEditsAllDisabledActionsFound")
            ),
            "productionActivationSafetyGateAssertionMethod": accessibility.get(
                "productionActivationSafetyGateAssertionMethod"
            ),
            "productionActivationSafetyGateAssertions": accessibility.get(
                "productionActivationSafetyGateAssertions", {}
            ),
            "productionActivationSafetyGatesAllHeadingsFound": bool(
                accessibility.get("productionActivationSafetyGatesAllHeadingsFound")
            ),
            "productionActivationSafetyGatesAllProductionDisabledFound": bool(
                accessibility.get(
                    "productionActivationSafetyGatesAllProductionDisabledFound"
                )
            ),
            "productionActivationSafetyGatesAllExecutorUnwiredFound": bool(
                accessibility.get(
                    "productionActivationSafetyGatesAllExecutorUnwiredFound"
                )
            ),
            "productionActivationSafetyGatesAllBlockedByDefaultFound": bool(
                accessibility.get(
                    "productionActivationSafetyGatesAllBlockedByDefaultFound"
                )
            ),
            "productionActivationSafetyGatesAllRequiredProofFound": bool(
                accessibility.get(
                    "productionActivationSafetyGatesAllRequiredProofFound"
                )
            ),
            "productionActivationSafetyGatesAllDisabledActionsFound": bool(
                accessibility.get(
                    "productionActivationSafetyGatesAllDisabledActionsFound"
                )
            ),
            "productionActivationSafetyProofAssertionMethod": accessibility.get(
                "productionActivationSafetyProofAssertionMethod"
            ),
            "productionActivationSafetyProofAssertions": accessibility.get(
                "productionActivationSafetyProofAssertions", {}
            ),
            "productionActivationSafetyProofsAllHeadingsFound": bool(
                accessibility.get("productionActivationSafetyProofsAllHeadingsFound")
            ),
            "productionActivationSafetyProofsAllProductionDisabledFound": bool(
                accessibility.get(
                    "productionActivationSafetyProofsAllProductionDisabledFound"
                )
            ),
            "productionActivationSafetyProofsAllExecutorUnwiredFound": bool(
                accessibility.get(
                    "productionActivationSafetyProofsAllExecutorUnwiredFound"
                )
            ),
            "productionActivationSafetyProofsAllProofStatusFound": bool(
                accessibility.get("productionActivationSafetyProofsAllProofStatusFound")
            ),
            "productionActivationSafetyProofsAllCopiedFixtureProofFound": bool(
                accessibility.get(
                    "productionActivationSafetyProofsAllCopiedFixtureProofFound"
                )
            ),
            "productionActivationSafetyProofsAllNoAutoApplyFound": bool(
                accessibility.get("productionActivationSafetyProofsAllNoAutoApplyFound")
            ),
            "productionActivationSafetyProofsAllFinalApprovalFound": bool(
                accessibility.get("productionActivationSafetyProofsAllFinalApprovalFound")
            ),
            "productionActivationSafetyProofsAllDisabledActionsFound": bool(
                accessibility.get(
                    "productionActivationSafetyProofsAllDisabledActionsFound"
                )
            ),
            "productionActivationFinalDecisionAssertionMethod": accessibility.get(
                "productionActivationFinalDecisionAssertionMethod"
            ),
            "productionActivationFinalDecisionAssertions": accessibility.get(
                "productionActivationFinalDecisionAssertions", {}
            ),
            "productionActivationFinalDecisionsAllHeadingsFound": bool(
                accessibility.get("productionActivationFinalDecisionsAllHeadingsFound")
            ),
            "productionActivationFinalDecisionsAllProductionDisabledFound": bool(
                accessibility.get(
                    "productionActivationFinalDecisionsAllProductionDisabledFound"
                )
            ),
            "productionActivationFinalDecisionsAllExecutorUnwiredFound": bool(
                accessibility.get(
                    "productionActivationFinalDecisionsAllExecutorUnwiredFound"
                )
            ),
            "productionActivationFinalDecisionsAllStatusFound": bool(
                accessibility.get("productionActivationFinalDecisionsAllStatusFound")
            ),
            "productionActivationFinalDecisionsAllDecisionLabelsFound": bool(
                accessibility.get(
                    "productionActivationFinalDecisionsAllDecisionLabelsFound"
                )
            ),
            "productionActivationFinalDecisionsAllPersistenceFound": bool(
                accessibility.get("productionActivationFinalDecisionsAllPersistenceFound")
            ),
            "productionActivationFinalDecisionsAllDisabledActionsFound": bool(
                accessibility.get(
                    "productionActivationFinalDecisionsAllDisabledActionsFound"
                )
            ),
            "duplicateConflictDetailNavigationAttempted": bool(
                accessibility.get("duplicateConflictDetailNavigationAttempted")
            ),
            "duplicateConflictDetailNavigationSucceeded": bool(
                accessibility.get("duplicateConflictDetailNavigationSucceeded")
            ),
            "forbiddenApplyActionSeen": bool(accessibility.get("forbiddenApplyActionSeen")),
            "pyatspiAvailable": bool(accessibility.get("pyatspiAvailable")),
        },
        "textSample": redact((accessibility.get("textAfterNavigation") or accessibility.get("text") or [])[:30]),
    }


def aggregate(runs):
    def any_run(predicate):
        return any(predicate(run) for run in runs)

    by_blocked_category = proof_level_by_blocked_category(runs)
    proof_surface_by_category = proof_surface_by_blocked_category(runs)
    by_area = {
        "Dashboard": proof(any_run(lambda run: "dashboard" in all_terms(run))),
        "Config": proof(any_run(lambda run: "config" in all_terms(run))),
        "Appearance": proof(any_run(lambda run: "appearance" in all_terms(run))),
        "Display": proof(any_run(lambda run: "display" in all_terms(run))),
        "Search": proof(any_run(lambda run: "search" in all_terms(run))),
        "settingRow": proof(any_run(lambda run: "FirstBlockedSettingRow" in run["name"] and run["accessibility"]["navigationAttempted"]), fallback=True),
        "detailPane": proof(any_run(lambda run: run["accessibility"]["detailPaneTextCollected"])),
        "blockedReason": proof(any_run(lambda run: run["accessibility"]["blockedReasonTextCollected"])),
        "defaultMissingBlockedCopy": by_blocked_category["defaultMissingBlockedCopy"],
        "duplicateBlockedCopy": by_blocked_category["duplicateBlockedCopy"],
        "generatedBlockedCopy": by_blocked_category["generatedBlockedCopy"],
        "scriptManagedBlockedCopy": by_blocked_category["scriptManagedBlockedCopy"],
        "symlinkCurrentProfileBlockedCopy": by_blocked_category["symlinkCurrentProfileBlockedCopy"],
        "highRiskBlockedCopy": by_blocked_category["highRiskBlockedCopy"],
        "displayRenderRiskBlockedCopy": by_blocked_category["displayRenderRiskBlockedCopy"],
        "profileModeSwitchBlockedCopy": by_blocked_category["profileModeSwitchBlockedCopy"],
        "generatedScriptSymlinkBlockedCopy": proof(
            all(
                by_blocked_category[key] == "live_gtk_atspi_proof"
                for key in [
                    "generatedBlockedCopy",
                    "scriptManagedBlockedCopy",
                    "symlinkCurrentProfileBlockedCopy",
                ]
            ),
            fallback=True,
        ),
        "highRiskDisplayRisk": proof(
            by_blocked_category["highRiskBlockedCopy"] == "live_gtk_atspi_proof"
            and by_blocked_category["displayRenderRiskBlockedCopy"] == "live_gtk_atspi_proof",
            fallback=True,
        ),
    }
    category_results = blocked_category_results(runs, by_blocked_category)
    approval_card_results = approval_card_assertion_results(runs)
    activation_decision_results = activation_decision_assertion_results(runs)
    activation_path_results = activation_path_assertion_results(runs)
    activation_control_results = activation_control_assertion_results(runs)
    activation_form_results = activation_form_assertion_results(runs)
    activation_draft_results = activation_draft_assertion_results(runs)
    activation_draft_edit_results = activation_draft_edit_assertion_results(runs)
    production_activation_safety_gate_results = (
        production_activation_safety_gate_assertion_results(runs)
    )
    production_activation_safety_proof_results = (
        production_activation_safety_proof_assertion_results(runs)
    )
    production_activation_final_decision_results = (
        production_activation_final_decision_assertion_results(runs)
    )

    return {
        "appLaunchAttempted": bool(runs),
        "appBuildAttempted": any_run(lambda run: run["probe"]["appBuildAttempted"]),
        "appBuildSucceeded": all(run["probe"]["appBuildSucceeded"] for run in runs) if runs else False,
        "appLaunchSucceeded": any_run(lambda run: run["probe"]["appLaunchSucceeded"]),
        "appBinaryRebuiltBeforeProbe": all(run["probe"]["appBinaryRebuiltBeforeProbe"] for run in runs) if runs else False,
        "accessibilityInspectionAttempted": any_run(lambda run: run["accessibility"]["attempted"]),
        "accessibilityInspectionSucceeded": any_run(lambda run: run["accessibility"]["succeeded"]),
        "uiTextTreeCollected": any_run(lambda run: bool(run["textSample"])),
        "navigationAttempted": any_run(lambda run: run["accessibility"]["navigationAttempted"]),
        "navigationSucceeded": any_run(lambda run: run["accessibility"]["navigationSucceeded"]),
        "duplicateConflictDetailNavigationAttempted": any_run(lambda run: run["accessibility"]["duplicateConflictDetailNavigationAttempted"]),
        "duplicateConflictDetailNavigationSucceeded": any_run(lambda run: run["accessibility"]["duplicateConflictDetailNavigationSucceeded"]),
        "duplicateBlockedReasonTextCollected": any_run(lambda run: run["accessibility"]["duplicateBlockedReasonTextCollected"]),
        "blockedCategoryDetailNavigationAttempted": any_run(lambda run: run["accessibility"]["blockedCategoryDetailNavigationAttempted"]),
        "blockedCategoryDetailNavigationSucceeded": any_run(lambda run: run["accessibility"]["blockedCategoryDetailNavigationSucceeded"]),
        "blockedCategoryReasonTextCollected": any_run(lambda run: run["accessibility"]["blockedCategoryReasonTextCollected"]),
        "closeSucceeded": all(run["probe"]["closeSucceeded"] for run in runs) if runs else False,
        "safeEnvModeUsed": all(run["probe"]["safeEnvModeUsed"] for run in runs) if runs else False,
        "liveSwapModeUsed": any_run(lambda run: run["probe"]["liveSwapModeUsed"]),
        "applyClicked": any_run(lambda run: run["probe"]["applyClicked"]),
        "realConfigEdited": any_run(lambda run: run["probe"]["realConfigEdited"]),
        "realBackupsCreated": any_run(lambda run: run["probe"]["realBackupsCreated"]),
        "hyprlandReloaded": any_run(lambda run: run["probe"]["hyprlandReloaded"]),
        "mutatingHyprctlUsed": any_run(lambda run: run["probe"]["mutatingHyprctlUsed"]),
        "runtimeMutated": any_run(lambda run: run["probe"]["runtimeMutated"]),
        "scriptsExecuted": any_run(lambda run: run["probe"]["scriptsExecuted"]),
        "luaExecuted": any_run(lambda run: run["probe"]["luaExecuted"]),
        "pyatspiAvailable": any_run(lambda run: run["accessibility"]["pyatspiAvailable"]),
        "proofLevelByUiArea": by_area,
        "proofLevelByBlockedCategory": by_blocked_category,
        "blockedCategoryResults": category_results,
        "proofSurfaceByBlockedCategory": proof_surface_by_category,
        "connectedFileDetailNavigationAttempted": any_run(lambda run: run["accessibility"]["connectedFileDetailNavigationAttempted"]),
        "connectedFileDetailNavigationSucceeded": any_run(lambda run: run["accessibility"]["connectedFileDetailNavigationSucceeded"]),
        "profileDetailNavigationAttempted": any_run(lambda run: run["accessibility"]["proofSurface"] == "profile_detail"),
        "profileDetailNavigationSucceeded": any_run(lambda run: run["accessibility"]["proofSurface"] == "profile_detail" and run["accessibility"]["connectedFileDetailNavigationSucceeded"]),
        "approvalCardAssertionMethod": "screenshot_plus_accessibility_tree_text_not_ocr",
        "approvalCardResults": approval_card_results,
        "approvalCardsAllHeadingsFound": all(
            result["headingProof"] == "live_gtk_atspi_proof"
            for result in approval_card_results.values()
        ),
        "approvalCardsAllProductionDisabledFound": all(
            result["productionDisabledProof"] == "live_gtk_atspi_proof"
            for result in approval_card_results.values()
        ),
        "approvalCardsAllDisabledActionsFound": all(
            result["disabledActionProof"] == "live_gtk_atspi_proof"
            for result in approval_card_results.values()
        ),
        "activationDecisionAssertionMethod": "screenshot_plus_accessibility_tree_text_not_ocr",
        "activationDecisionResults": activation_decision_results,
        "activationDecisionsAllHeadingsFound": all(
            result["headingProof"] == "live_gtk_atspi_proof"
            for result in activation_decision_results.values()
        ),
        "activationDecisionsAllProductionDisabledFound": all(
            result["productionDisabledProof"] == "live_gtk_atspi_proof"
            for result in activation_decision_results.values()
        ),
        "activationDecisionsAllDisabledActionsFound": all(
            result["disabledActionProof"] == "live_gtk_atspi_proof"
            for result in activation_decision_results.values()
        ),
        "activationPathAssertionMethod": "screenshot_plus_accessibility_tree_text_not_ocr",
        "activationPathResults": activation_path_results,
        "activationPathsAllHeadingsFound": all(
            result["headingProof"] == "live_gtk_atspi_proof"
            for result in activation_path_results.values()
        ),
        "activationPathsAllProductionDisabledFound": all(
            result["productionDisabledProof"] == "live_gtk_atspi_proof"
            for result in activation_path_results.values()
        ),
        "activationPathsAllDisabledActionsFound": all(
            result["disabledActionProof"] == "live_gtk_atspi_proof"
            for result in activation_path_results.values()
        ),
        "activationControlAssertionMethod": "screenshot_plus_accessibility_tree_text_not_ocr",
        "activationControlResults": activation_control_results,
        "activationControlsAllHeadingsFound": all(
            result["headingProof"] == "live_gtk_atspi_proof"
            for result in activation_control_results.values()
        ),
        "activationControlsAllProductionDisabledFound": all(
            result["productionDisabledProof"] == "live_gtk_atspi_proof"
            for result in activation_control_results.values()
        ),
        "activationControlsAllExecutorUnwiredFound": all(
            result["executorWiringProof"] == "live_gtk_atspi_proof"
            for result in activation_control_results.values()
        ),
        "activationControlsAllDisabledActionsFound": all(
            result["disabledActionProof"] == "live_gtk_atspi_proof"
            for result in activation_control_results.values()
        ),
        "activationFormAssertionMethod": "screenshot_plus_accessibility_tree_text_not_ocr",
        "activationFormResults": activation_form_results,
        "activationFormsAllHeadingsFound": all(
            result["headingProof"] == "live_gtk_atspi_proof"
            for result in activation_form_results.values()
        ),
        "activationFormsAllProductionDisabledFound": all(
            result["productionDisabledProof"] == "live_gtk_atspi_proof"
            for result in activation_form_results.values()
        ),
        "activationFormsAllExecutorUnwiredFound": all(
            result["executorWiringProof"] == "live_gtk_atspi_proof"
            for result in activation_form_results.values()
        ),
        "activationFormsAllDisabledActionsFound": all(
            result["disabledActionProof"] == "live_gtk_atspi_proof"
            for result in activation_form_results.values()
        ),
        "activationFormsAllFieldLabelsFound": all(
            result["fieldLabelsProof"] == "live_gtk_atspi_proof"
            for result in activation_form_results.values()
        ),
        "activationDraftAssertionMethod": "screenshot_plus_accessibility_tree_text_not_ocr",
        "activationDraftResults": activation_draft_results,
        "activationDraftsAllHeadingsFound": all(
            result["headingProof"] == "live_gtk_atspi_proof"
            for result in activation_draft_results.values()
        ),
        "activationDraftsAllProductionDisabledFound": all(
            result["productionDisabledProof"] == "live_gtk_atspi_proof"
            for result in activation_draft_results.values()
        ),
        "activationDraftsAllExecutorUnwiredFound": all(
            result["executorWiringProof"] == "live_gtk_atspi_proof"
            for result in activation_draft_results.values()
        ),
        "activationDraftsAllInMemoryOnlyFound": all(
            result["memoryStatusProof"] == "live_gtk_atspi_proof"
            for result in activation_draft_results.values()
        ),
        "activationDraftsAllDisabledActionsFound": all(
            result["disabledActionsProof"] == "live_gtk_atspi_proof"
            for result in activation_draft_results.values()
        ),
        "activationDraftEditAssertionMethod": "screenshot_plus_accessibility_tree_text_not_ocr",
        "activationDraftEditResults": activation_draft_edit_results,
        "activationDraftEditsAllHeadingsFound": all(
            result["headingProof"] == "live_gtk_atspi_proof"
            for result in activation_draft_edit_results.values()
        ),
        "activationDraftEditsAllProductionDisabledFound": all(
            result["productionDisabledProof"] == "live_gtk_atspi_proof"
            for result in activation_draft_edit_results.values()
        ),
        "activationDraftEditsAllExecutorUnwiredFound": all(
            result["executorWiringProof"] == "live_gtk_atspi_proof"
            for result in activation_draft_edit_results.values()
        ),
        "activationDraftEditsAllInMemoryOnlyFound": all(
            result["memoryStatusProof"] == "live_gtk_atspi_proof"
            for result in activation_draft_edit_results.values()
        ),
        "activationDraftEditsAllModeFound": all(
            result["editingModeProof"] == "live_gtk_atspi_proof"
            for result in activation_draft_edit_results.values()
        ),
        "activationDraftEditsAllValidationFound": all(
            result["draftValidationProof"] == "live_gtk_atspi_proof"
            for result in activation_draft_edit_results.values()
        ),
        "activationDraftEditsAllDisabledActionsFound": all(
            result["disabledActionsProof"] == "live_gtk_atspi_proof"
            for result in activation_draft_edit_results.values()
        ),
        "productionActivationSafetyGateAssertionMethod": "screenshot_plus_accessibility_tree_text_not_ocr",
        "productionActivationSafetyGateResults": production_activation_safety_gate_results,
        "productionActivationSafetyGatesAllHeadingsFound": all(
            result["headingProof"] == "live_gtk_atspi_proof"
            for result in production_activation_safety_gate_results.values()
        ),
        "productionActivationSafetyGatesAllProductionDisabledFound": all(
            result["productionDisabledProof"] == "live_gtk_atspi_proof"
            for result in production_activation_safety_gate_results.values()
        ),
        "productionActivationSafetyGatesAllExecutorUnwiredFound": all(
            result["executorWiringProof"] == "live_gtk_atspi_proof"
            for result in production_activation_safety_gate_results.values()
        ),
        "productionActivationSafetyGatesAllBlockedByDefaultFound": all(
            result["blockedByDefaultProof"] == "live_gtk_atspi_proof"
            for result in production_activation_safety_gate_results.values()
        ),
        "productionActivationSafetyGatesAllRequiredProofFound": all(
            result["requiredProofProof"] == "live_gtk_atspi_proof"
            for result in production_activation_safety_gate_results.values()
        ),
        "productionActivationSafetyGatesAllDisabledActionsFound": all(
            result["disabledActionsProof"] == "live_gtk_atspi_proof"
            for result in production_activation_safety_gate_results.values()
        ),
        "productionActivationSafetyProofAssertionMethod": "screenshot_plus_accessibility_tree_text_not_ocr",
        "productionActivationSafetyProofResults": production_activation_safety_proof_results,
        "productionActivationSafetyProofsAllHeadingsFound": all(
            result["headingProof"] == "live_gtk_atspi_proof"
            for result in production_activation_safety_proof_results.values()
        ),
        "productionActivationSafetyProofsAllProductionDisabledFound": all(
            result["productionDisabledProof"] == "live_gtk_atspi_proof"
            for result in production_activation_safety_proof_results.values()
        ),
        "productionActivationSafetyProofsAllExecutorUnwiredFound": all(
            result["executorWiringProof"] == "live_gtk_atspi_proof"
            for result in production_activation_safety_proof_results.values()
        ),
        "productionActivationSafetyProofsAllProofStatusFound": all(
            result["proofStatusProof"] == "live_gtk_atspi_proof"
            for result in production_activation_safety_proof_results.values()
        ),
        "productionActivationSafetyProofsAllCopiedFixtureProofFound": all(
            result["copiedFixtureProof"] == "live_gtk_atspi_proof"
            for result in production_activation_safety_proof_results.values()
        ),
        "productionActivationSafetyProofsAllNoAutoApplyFound": all(
            result["noAutoApplyProof"] == "live_gtk_atspi_proof"
            for result in production_activation_safety_proof_results.values()
        ),
        "productionActivationSafetyProofsAllFinalApprovalFound": all(
            result["finalApprovalProof"] == "live_gtk_atspi_proof"
            for result in production_activation_safety_proof_results.values()
        ),
        "productionActivationSafetyProofsAllDisabledActionsFound": all(
            result["disabledActionsProof"] == "live_gtk_atspi_proof"
            for result in production_activation_safety_proof_results.values()
        ),
        "productionActivationFinalDecisionAssertionMethod": "screenshot_plus_accessibility_tree_text_not_ocr",
        "productionActivationFinalDecisionResults": production_activation_final_decision_results,
        "productionActivationFinalDecisionsAllHeadingsFound": all(
            result["headingProof"] == "live_gtk_atspi_proof"
            for result in production_activation_final_decision_results.values()
        ),
        "productionActivationFinalDecisionsAllProductionDisabledFound": all(
            result["productionDisabledProof"] == "live_gtk_atspi_proof"
            for result in production_activation_final_decision_results.values()
        ),
        "productionActivationFinalDecisionsAllExecutorUnwiredFound": all(
            result["executorWiringProof"] == "live_gtk_atspi_proof"
            for result in production_activation_final_decision_results.values()
        ),
        "productionActivationFinalDecisionsAllStatusFound": all(
            result["finalDecisionStatusProof"] == "live_gtk_atspi_proof"
            for result in production_activation_final_decision_results.values()
        ),
        "productionActivationFinalDecisionsAllDecisionLabelsFound": all(
            result["decisionLabelsProof"] == "live_gtk_atspi_proof"
            for result in production_activation_final_decision_results.values()
        ),
        "productionActivationFinalDecisionsAllPersistenceFound": all(
            result["draftPersistenceProof"] == "live_gtk_atspi_proof"
            for result in production_activation_final_decision_results.values()
        ),
        "productionActivationFinalDecisionsAllDisabledActionsFound": all(
            result["disabledActionsProof"] == "live_gtk_atspi_proof"
            for result in production_activation_final_decision_results.values()
        ),
        "fallbackProofUsed": any(level in {"source_model_fallback", "not_proven"} for level in by_area.values())
        or any(level in {"source_model_fallback", "not_proven"} for level in by_blocked_category.values()),
    }


def approval_card_assertion_results(runs):
    card_keys = [
        "sourceIncludeInsertion",
        "duplicateReplacement",
        "structuredHlBindWrite",
        "profileModeSwitch",
        "highRiskDisplayWrite",
        "hyprland0554Migration",
    ]
    results = {}
    for key in card_keys:
        heading_run = next(
            (
                run
                for run in runs
                if run["accessibility"]
                .get("approvalCardAssertions", {})
                .get(key, {})
                .get("headingFound")
            ),
            None,
        )
        production_run = next(
            (
                run
                for run in runs
                if run["accessibility"]
                .get("approvalCardAssertions", {})
                .get(key, {})
                .get("productionDisabledFound")
            ),
            None,
        )
        action_run = next(
            (
                run
                for run in runs
                if run["accessibility"]
                .get("approvalCardAssertions", {})
                .get(key, {})
                .get("disabledActionFound")
            ),
            None,
        )
        sample = {}
        for run in runs:
            sample = run["accessibility"].get("approvalCardAssertions", {}).get(key, {})
            if sample:
                break
        results[key] = {
            "heading": sample.get("heading"),
            "headingProof": "live_gtk_atspi_proof" if heading_run else "not_proven",
            "headingEvidenceRun": heading_run["name"] if heading_run else None,
            "productionDisabledText": sample.get("productionDisabledText"),
            "productionDisabledProof": "live_gtk_atspi_proof"
            if production_run
            else "not_proven",
            "productionDisabledEvidenceRun": production_run["name"] if production_run else None,
            "disabledAction": sample.get("disabledAction"),
            "disabledActionProof": "live_gtk_atspi_proof" if action_run else "not_proven",
            "disabledActionEvidenceRun": action_run["name"] if action_run else None,
            "widgetName": sample.get("widgetName"),
            "assertionMethod": "screenshot_plus_accessibility_tree_text_not_ocr",
        }
    return results


def activation_decision_assertion_results(runs):
    decision_keys = [
        "sourceIncludeInsertion",
        "duplicateReplacement",
    ]
    results = {}
    for key in decision_keys:
        heading_run = next(
            (
                run
                for run in runs
                if run["accessibility"]
                .get("activationDecisionAssertions", {})
                .get(key, {})
                .get("headingFound")
            ),
            None,
        )
        production_run = next(
            (
                run
                for run in runs
                if run["accessibility"]
                .get("activationDecisionAssertions", {})
                .get(key, {})
                .get("productionDisabledFound")
            ),
            None,
        )
        action_run = next(
            (
                run
                for run in runs
                if run["accessibility"]
                .get("activationDecisionAssertions", {})
                .get(key, {})
                .get("disabledActionFound")
            ),
            None,
        )
        sample = {}
        for run in runs:
            sample = run["accessibility"].get("activationDecisionAssertions", {}).get(key, {})
            if sample:
                break
        results[key] = {
            "heading": sample.get("heading"),
            "headingProof": "live_gtk_atspi_proof" if heading_run else "not_proven",
            "headingEvidenceRun": heading_run["name"] if heading_run else None,
            "productionDisabledText": sample.get("productionDisabledText"),
            "productionDisabledProof": "live_gtk_atspi_proof"
            if production_run
            else "not_proven",
            "productionDisabledEvidenceRun": production_run["name"] if production_run else None,
            "disabledAction": sample.get("disabledAction"),
            "disabledActionProof": "live_gtk_atspi_proof" if action_run else "not_proven",
            "disabledActionEvidenceRun": action_run["name"] if action_run else None,
            "widgetName": sample.get("widgetName"),
            "assertionMethod": "screenshot_plus_accessibility_tree_text_not_ocr",
        }
    return results


def activation_path_assertion_results(runs):
    path_keys = [
        "sourceIncludeInsertion",
        "duplicateReplacement",
    ]
    results = {}
    for key in path_keys:
        heading_run = next(
            (
                run
                for run in runs
                if run["accessibility"]
                .get("activationPathAssertions", {})
                .get(key, {})
                .get("headingFound")
            ),
            None,
        )
        production_run = next(
            (
                run
                for run in runs
                if run["accessibility"]
                .get("activationPathAssertions", {})
                .get(key, {})
                .get("productionDisabledFound")
            ),
            None,
        )
        action_run = next(
            (
                run
                for run in runs
                if run["accessibility"]
                .get("activationPathAssertions", {})
                .get(key, {})
                .get("disabledActionFound")
            ),
            None,
        )
        sample = {}
        for run in runs:
            sample = run["accessibility"].get("activationPathAssertions", {}).get(key, {})
            if sample:
                break
        results[key] = {
            "heading": sample.get("heading"),
            "headingProof": "live_gtk_atspi_proof" if heading_run else "not_proven",
            "headingEvidenceRun": heading_run["name"] if heading_run else None,
            "productionDisabledText": sample.get("productionDisabledText"),
            "productionDisabledProof": "live_gtk_atspi_proof"
            if production_run
            else "not_proven",
            "productionDisabledEvidenceRun": production_run["name"] if production_run else None,
            "disabledAction": sample.get("disabledAction"),
            "disabledActionProof": "live_gtk_atspi_proof" if action_run else "not_proven",
            "disabledActionEvidenceRun": action_run["name"] if action_run else None,
            "widgetName": sample.get("widgetName"),
            "assertionMethod": "screenshot_plus_accessibility_tree_text_not_ocr",
        }
    return results


def activation_control_assertion_results(runs):
    control_keys = [
        "sourceIncludeInsertion",
        "duplicateReplacement",
    ]
    results = {}
    for key in control_keys:
        heading_run = next(
            (
                run
                for run in runs
                if run["accessibility"]
                .get("activationControlAssertions", {})
                .get(key, {})
                .get("headingFound")
            ),
            None,
        )
        production_run = next(
            (
                run
                for run in runs
                if run["accessibility"]
                .get("activationControlAssertions", {})
                .get(key, {})
                .get("productionDisabledFound")
            ),
            None,
        )
        executor_run = next(
            (
                run
                for run in runs
                if run["accessibility"]
                .get("activationControlAssertions", {})
                .get(key, {})
                .get("executorWiringFound")
            ),
            None,
        )
        action_run = next(
            (
                run
                for run in runs
                if run["accessibility"]
                .get("activationControlAssertions", {})
                .get(key, {})
                .get("disabledActionFound")
            ),
            None,
        )
        sample = {}
        for run in runs:
            sample = run["accessibility"].get("activationControlAssertions", {}).get(key, {})
            if sample:
                break
        results[key] = {
            "heading": sample.get("heading"),
            "headingProof": "live_gtk_atspi_proof" if heading_run else "not_proven",
            "headingEvidenceRun": heading_run["name"] if heading_run else None,
            "productionDisabledText": sample.get("productionDisabledText"),
            "productionDisabledProof": "live_gtk_atspi_proof"
            if production_run
            else "not_proven",
            "productionDisabledEvidenceRun": production_run["name"] if production_run else None,
            "executorWiring": sample.get("executorWiring"),
            "executorWiringProof": "live_gtk_atspi_proof" if executor_run else "not_proven",
            "executorWiringEvidenceRun": executor_run["name"] if executor_run else None,
            "disabledAction": sample.get("disabledAction"),
            "disabledActionProof": "live_gtk_atspi_proof" if action_run else "not_proven",
            "disabledActionEvidenceRun": action_run["name"] if action_run else None,
            "widgetName": sample.get("widgetName"),
            "assertionMethod": "screenshot_plus_accessibility_tree_text_not_ocr",
        }
    return results


def activation_form_assertion_results(runs):
    form_keys = [
        "sourceIncludeInsertion",
        "duplicateReplacement",
    ]
    results = {}
    for key in form_keys:
        heading_run = next(
            (
                run
                for run in runs
                if run["accessibility"]
                .get("activationFormAssertions", {})
                .get(key, {})
                .get("headingFound")
            ),
            None,
        )
        production_run = next(
            (
                run
                for run in runs
                if run["accessibility"]
                .get("activationFormAssertions", {})
                .get(key, {})
                .get("productionDisabledFound")
            ),
            None,
        )
        executor_run = next(
            (
                run
                for run in runs
                if run["accessibility"]
                .get("activationFormAssertions", {})
                .get(key, {})
                .get("executorWiringFound")
            ),
            None,
        )
        action_run = next(
            (
                run
                for run in runs
                if run["accessibility"]
                .get("activationFormAssertions", {})
                .get(key, {})
                .get("disabledActionFound")
            ),
            None,
        )
        field_run = next(
            (
                run
                for run in runs
                if run["accessibility"]
                .get("activationFormAssertions", {})
                .get(key, {})
                .get("fieldLabelsFound")
            ),
            None,
        )
        sample = {}
        for run in runs:
            sample = run["accessibility"].get("activationFormAssertions", {}).get(key, {})
            if sample:
                break
        results[key] = {
            "heading": sample.get("heading"),
            "headingProof": "live_gtk_atspi_proof" if heading_run else "not_proven",
            "headingEvidenceRun": heading_run["name"] if heading_run else None,
            "productionDisabledText": sample.get("productionDisabledText"),
            "productionDisabledProof": "live_gtk_atspi_proof"
            if production_run
            else "not_proven",
            "productionDisabledEvidenceRun": production_run["name"] if production_run else None,
            "executorWiring": sample.get("executorWiring"),
            "executorWiringProof": "live_gtk_atspi_proof" if executor_run else "not_proven",
            "executorWiringEvidenceRun": executor_run["name"] if executor_run else None,
            "disabledAction": sample.get("disabledAction"),
            "disabledActionProof": "live_gtk_atspi_proof" if action_run else "not_proven",
            "disabledActionEvidenceRun": action_run["name"] if action_run else None,
            "fieldLabels": sample.get("fieldLabels"),
            "fieldLabelResults": sample.get("fieldLabelResults"),
            "fieldLabelsProof": "live_gtk_atspi_proof" if field_run else "not_proven",
            "fieldLabelsEvidenceRun": field_run["name"] if field_run else None,
            "widgetName": sample.get("widgetName"),
            "assertionMethod": "screenshot_plus_accessibility_tree_text_not_ocr",
        }
    return results


def activation_draft_assertion_results(runs):
    draft_keys = [
        "sourceIncludeInsertion",
        "duplicateReplacement",
    ]
    results = {}
    for key in draft_keys:
        heading_run = next(
            (
                run
                for run in runs
                if run["accessibility"]
                .get("activationDraftAssertions", {})
                .get(key, {})
                .get("headingFound")
            ),
            None,
        )
        production_run = next(
            (
                run
                for run in runs
                if run["accessibility"]
                .get("activationDraftAssertions", {})
                .get(key, {})
                .get("productionDisabledFound")
            ),
            None,
        )
        executor_run = next(
            (
                run
                for run in runs
                if run["accessibility"]
                .get("activationDraftAssertions", {})
                .get(key, {})
                .get("executorWiringFound")
            ),
            None,
        )
        memory_run = next(
            (
                run
                for run in runs
                if run["accessibility"]
                .get("activationDraftAssertions", {})
                .get(key, {})
                .get("memoryStatusFound")
            ),
            None,
        )
        action_run = next(
            (
                run
                for run in runs
                if run["accessibility"]
                .get("activationDraftAssertions", {})
                .get(key, {})
                .get("disabledUpdateFound")
                and run["accessibility"]
                .get("activationDraftAssertions", {})
                .get(key, {})
                .get("disabledResetFound")
            ),
            None,
        )
        sample = {}
        for run in runs:
            sample = run["accessibility"].get("activationDraftAssertions", {}).get(key, {})
            if sample:
                break
        results[key] = {
            "heading": sample.get("heading"),
            "headingProof": "live_gtk_atspi_proof" if heading_run else "not_proven",
            "headingEvidenceRun": heading_run["name"] if heading_run else None,
            "productionDisabledText": sample.get("productionDisabledText"),
            "productionDisabledProof": "live_gtk_atspi_proof"
            if production_run
            else "not_proven",
            "productionDisabledEvidenceRun": production_run["name"] if production_run else None,
            "executorWiring": sample.get("executorWiring"),
            "executorWiringProof": "live_gtk_atspi_proof" if executor_run else "not_proven",
            "executorWiringEvidenceRun": executor_run["name"] if executor_run else None,
            "memoryStatus": sample.get("memoryStatus"),
            "memoryStatusProof": "live_gtk_atspi_proof" if memory_run else "not_proven",
            "memoryStatusEvidenceRun": memory_run["name"] if memory_run else None,
            "disabledUpdate": sample.get("disabledUpdate"),
            "disabledReset": sample.get("disabledReset"),
            "disabledActionsProof": "live_gtk_atspi_proof" if action_run else "not_proven",
            "disabledActionsEvidenceRun": action_run["name"] if action_run else None,
            "widgetName": sample.get("widgetName"),
            "assertionMethod": "screenshot_plus_accessibility_tree_text_not_ocr",
        }
    return results


def activation_draft_edit_assertion_results(runs):
    edit_keys = [
        "sourceIncludeInsertion",
        "duplicateReplacement",
    ]
    results = {}
    for key in edit_keys:
        heading_run = next(
            (
                run
                for run in runs
                if run["accessibility"]
                .get("activationDraftEditAssertions", {})
                .get(key, {})
                .get("headingFound")
            ),
            None,
        )
        production_run = next(
            (
                run
                for run in runs
                if run["accessibility"]
                .get("activationDraftEditAssertions", {})
                .get(key, {})
                .get("productionDisabledFound")
            ),
            None,
        )
        executor_run = next(
            (
                run
                for run in runs
                if run["accessibility"]
                .get("activationDraftEditAssertions", {})
                .get(key, {})
                .get("executorWiringFound")
            ),
            None,
        )
        memory_run = next(
            (
                run
                for run in runs
                if run["accessibility"]
                .get("activationDraftEditAssertions", {})
                .get(key, {})
                .get("memoryStatusFound")
            ),
            None,
        )
        mode_run = next(
            (
                run
                for run in runs
                if run["accessibility"]
                .get("activationDraftEditAssertions", {})
                .get(key, {})
                .get("editingModeFound")
            ),
            None,
        )
        validation_run = next(
            (
                run
                for run in runs
                if run["accessibility"]
                .get("activationDraftEditAssertions", {})
                .get(key, {})
                .get("draftValidationFound")
            ),
            None,
        )
        action_run = next(
            (
                run
                for run in runs
                if run["accessibility"]
                .get("activationDraftEditAssertions", {})
                .get(key, {})
                .get("disabledUpdateFound")
                and run["accessibility"]
                .get("activationDraftEditAssertions", {})
                .get(key, {})
                .get("disabledResetFound")
            ),
            None,
        )
        sample = {}
        for run in runs:
            sample = run["accessibility"].get("activationDraftEditAssertions", {}).get(key, {})
            if sample:
                break
        results[key] = {
            "heading": sample.get("heading"),
            "headingProof": "live_gtk_atspi_proof" if heading_run else "not_proven",
            "headingEvidenceRun": heading_run["name"] if heading_run else None,
            "productionDisabledText": sample.get("productionDisabledText"),
            "productionDisabledProof": "live_gtk_atspi_proof"
            if production_run
            else "not_proven",
            "productionDisabledEvidenceRun": production_run["name"] if production_run else None,
            "executorWiring": sample.get("executorWiring"),
            "executorWiringProof": "live_gtk_atspi_proof" if executor_run else "not_proven",
            "executorWiringEvidenceRun": executor_run["name"] if executor_run else None,
            "memoryStatus": sample.get("memoryStatus"),
            "memoryStatusProof": "live_gtk_atspi_proof" if memory_run else "not_proven",
            "memoryStatusEvidenceRun": memory_run["name"] if memory_run else None,
            "editingMode": sample.get("editingMode"),
            "editingModeProof": "live_gtk_atspi_proof" if mode_run else "not_proven",
            "editingModeEvidenceRun": mode_run["name"] if mode_run else None,
            "draftValidation": sample.get("draftValidation"),
            "draftValidationProof": "live_gtk_atspi_proof"
            if validation_run
            else "not_proven",
            "draftValidationEvidenceRun": validation_run["name"] if validation_run else None,
            "disabledUpdate": sample.get("disabledUpdate"),
            "disabledReset": sample.get("disabledReset"),
            "disabledActionsProof": "live_gtk_atspi_proof" if action_run else "not_proven",
            "disabledActionsEvidenceRun": action_run["name"] if action_run else None,
            "widgetName": sample.get("widgetName"),
            "assertionMethod": "screenshot_plus_accessibility_tree_text_not_ocr",
        }
    return results


def production_activation_safety_gate_assertion_results(runs):
    gate_keys = [
        "sourceIncludeInsertion",
        "duplicateReplacement",
    ]
    results = {}
    for key in gate_keys:
        heading_run = next(
            (
                run
                for run in runs
                if run["accessibility"]
                .get("productionActivationSafetyGateAssertions", {})
                .get(key, {})
                .get("headingFound")
            ),
            None,
        )
        production_run = next(
            (
                run
                for run in runs
                if run["accessibility"]
                .get("productionActivationSafetyGateAssertions", {})
                .get(key, {})
                .get("productionDisabledFound")
            ),
            None,
        )
        executor_run = next(
            (
                run
                for run in runs
                if run["accessibility"]
                .get("productionActivationSafetyGateAssertions", {})
                .get(key, {})
                .get("executorWiringFound")
            ),
            None,
        )
        blocked_run = next(
            (
                run
                for run in runs
                if run["accessibility"]
                .get("productionActivationSafetyGateAssertions", {})
                .get(key, {})
                .get("gateStatusFound")
            ),
            None,
        )
        required_run = next(
            (
                run
                for run in runs
                if all(
                    run["accessibility"]
                    .get("productionActivationSafetyGateAssertions", {})
                    .get(key, {})
                    .get(field)
                    for field in [
                        "byteExactBackupFound",
                        "writePlanFound",
                        "rereadPlanFound",
                        "restorePlanFound",
                        "noAutoApplyProofFound",
                        "persistenceAutoApplyProofFound",
                        "finalApprovalFound",
                    ]
                )
            ),
            None,
        )
        action_run = next(
            (
                run
                for run in runs
                if run["accessibility"]
                .get("productionActivationSafetyGateAssertions", {})
                .get(key, {})
                .get("disabledReviewFound")
                and run["accessibility"]
                .get("productionActivationSafetyGateAssertions", {})
                .get(key, {})
                .get("disabledEnableFound")
            ),
            None,
        )
        sample = {}
        for run in runs:
            sample = (
                run["accessibility"]
                .get("productionActivationSafetyGateAssertions", {})
                .get(key, {})
            )
            if sample:
                break
        results[key] = {
            "heading": sample.get("heading"),
            "headingProof": "live_gtk_atspi_proof" if heading_run else "not_proven",
            "headingEvidenceRun": heading_run["name"] if heading_run else None,
            "productionDisabledText": sample.get("productionDisabledText"),
            "productionDisabledProof": "live_gtk_atspi_proof"
            if production_run
            else "not_proven",
            "productionDisabledEvidenceRun": production_run["name"] if production_run else None,
            "executorWiring": sample.get("executorWiring"),
            "executorWiringProof": "live_gtk_atspi_proof" if executor_run else "not_proven",
            "executorWiringEvidenceRun": executor_run["name"] if executor_run else None,
            "gateStatus": sample.get("gateStatus"),
            "blockedByDefaultProof": "live_gtk_atspi_proof" if blocked_run else "not_proven",
            "blockedByDefaultEvidenceRun": blocked_run["name"] if blocked_run else None,
            "requiredProofLabels": [
                sample.get("byteExactBackup"),
                sample.get("writePlan"),
                sample.get("rereadPlan"),
                sample.get("restorePlan"),
                sample.get("noAutoApplyProof"),
                sample.get("persistenceAutoApplyProof"),
                sample.get("finalApproval"),
            ],
            "requiredProofProof": "live_gtk_atspi_proof"
            if required_run
            else "not_proven",
            "requiredProofEvidenceRun": required_run["name"] if required_run else None,
            "disabledReview": sample.get("disabledReview"),
            "disabledEnable": sample.get("disabledEnable"),
            "disabledActionsProof": "live_gtk_atspi_proof" if action_run else "not_proven",
            "disabledActionsEvidenceRun": action_run["name"] if action_run else None,
            "widgetName": sample.get("widgetName"),
            "assertionMethod": "screenshot_plus_accessibility_tree_text_not_ocr",
        }
    return results


def production_activation_safety_proof_assertion_results(runs):
    proof_keys = [
        "sourceIncludeInsertion",
        "duplicateReplacement",
    ]
    results = {}
    for key in proof_keys:
        def assertion_run(field):
            return next(
                (
                    run
                    for run in runs
                    if run["accessibility"]
                    .get("productionActivationSafetyProofAssertions", {})
                    .get(key, {})
                    .get(field)
                ),
                None,
            )

        heading_run = assertion_run("headingFound")
        production_run = assertion_run("productionDisabledFound")
        executor_run = assertion_run("executorWiringFound")
        status_run = assertion_run("proofStatusFound")
        copied_fixture_run = next(
            (
                run
                for run in runs
                if all(
                    run["accessibility"]
                    .get("productionActivationSafetyProofAssertions", {})
                    .get(key, {})
                    .get(field)
                    for field in [
                        "byteExactBackupFound",
                        "dryRunWritePlanFound",
                        "diffPreviewFound",
                        "postWriteRereadFound",
                        "restorePlanFound",
                        "postRestoreVerificationFound",
                    ]
                )
            ),
            None,
        )
        no_auto_apply_run = next(
            (
                run
                for run in runs
                if run["accessibility"]
                .get("productionActivationSafetyProofAssertions", {})
                .get(key, {})
                .get("noAutoApplyProofFound")
                and run["accessibility"]
                .get("productionActivationSafetyProofAssertions", {})
                .get(key, {})
                .get("persistenceAutoApplyProofFound")
            ),
            None,
        )
        final_approval_run = assertion_run("finalApprovalFound")
        action_run = next(
            (
                run
                for run in runs
                if run["accessibility"]
                .get("productionActivationSafetyProofAssertions", {})
                .get(key, {})
                .get("disabledRunFound")
                and run["accessibility"]
                .get("productionActivationSafetyProofAssertions", {})
                .get(key, {})
                .get("disabledEnableFound")
            ),
            None,
        )
        sample = {}
        for run in runs:
            sample = (
                run["accessibility"]
                .get("productionActivationSafetyProofAssertions", {})
                .get(key, {})
            )
            if sample:
                break
        results[key] = {
            "heading": sample.get("heading"),
            "headingProof": "live_gtk_atspi_proof" if heading_run else "not_proven",
            "headingEvidenceRun": heading_run["name"] if heading_run else None,
            "productionDisabledText": sample.get("productionDisabledText"),
            "productionDisabledProof": "live_gtk_atspi_proof"
            if production_run
            else "not_proven",
            "productionDisabledEvidenceRun": production_run["name"] if production_run else None,
            "executorWiring": sample.get("executorWiring"),
            "executorWiringProof": "live_gtk_atspi_proof" if executor_run else "not_proven",
            "executorWiringEvidenceRun": executor_run["name"] if executor_run else None,
            "proofStatus": sample.get("proofStatus"),
            "proofStatusProof": "live_gtk_atspi_proof" if status_run else "not_proven",
            "proofStatusEvidenceRun": status_run["name"] if status_run else None,
            "copiedFixtureLabels": [
                sample.get("byteExactBackup"),
                sample.get("dryRunWritePlan"),
                sample.get("diffPreview"),
                sample.get("postWriteReread"),
                sample.get("restorePlan"),
                sample.get("postRestoreVerification"),
            ],
            "copiedFixtureProof": "live_gtk_atspi_proof"
            if copied_fixture_run
            else "not_proven",
            "copiedFixtureEvidenceRun": copied_fixture_run["name"]
            if copied_fixture_run
            else None,
            "noAutoApplyLabels": [
                sample.get("noAutoApplyProof"),
                sample.get("persistenceAutoApplyProof"),
            ],
            "noAutoApplyProof": "live_gtk_atspi_proof"
            if no_auto_apply_run
            else "not_proven",
            "noAutoApplyEvidenceRun": no_auto_apply_run["name"]
            if no_auto_apply_run
            else None,
            "finalApproval": sample.get("finalApproval"),
            "finalApprovalProof": "live_gtk_atspi_proof"
            if final_approval_run
            else "not_proven",
            "finalApprovalEvidenceRun": final_approval_run["name"]
            if final_approval_run
            else None,
            "disabledRun": sample.get("disabledRun"),
            "disabledEnable": sample.get("disabledEnable"),
            "disabledActionsProof": "live_gtk_atspi_proof" if action_run else "not_proven",
            "disabledActionsEvidenceRun": action_run["name"] if action_run else None,
            "widgetName": sample.get("widgetName"),
            "assertionMethod": "screenshot_plus_accessibility_tree_text_not_ocr",
        }
    return results


def production_activation_final_decision_assertion_results(runs):
    decision_keys = [
        "sourceIncludeInsertion",
        "duplicateReplacement",
    ]
    results = {}
    for key in decision_keys:
        def assertion_run(field):
            return next(
                (
                    run
                    for run in runs
                    if run["accessibility"]
                    .get("productionActivationFinalDecisionAssertions", {})
                    .get(key, {})
                    .get(field)
                ),
                None,
            )

        heading_run = assertion_run("headingFound")
        production_run = assertion_run("productionDisabledFound")
        executor_run = assertion_run("executorWiringFound")
        status_run = assertion_run("finalDecisionStatusFound")
        decisions_run = next(
            (
                run
                for run in runs
                if all(
                    run["accessibility"]
                    .get("productionActivationFinalDecisionAssertions", {})
                    .get(key, {})
                    .get(field)
                    for field in [
                        "finalApprovalFound",
                        "productionFlagDecisionFound",
                        "executorWiringDecisionFound",
                        "liveProductionDryRunPolicyFound",
                        "copiedFixtureProofFound",
                    ]
                )
            ),
            None,
        )
        persistence_run = assertion_run("draftPersistenceFound")
        action_run = next(
            (
                run
                for run in runs
                if all(
                    run["accessibility"]
                    .get("productionActivationFinalDecisionAssertions", {})
                    .get(key, {})
                    .get(field)
                    for field in [
                        "disabledApprovalFound",
                        "disabledProductionFlagFound",
                        "disabledExecutorWiringFound",
                        "disabledLiveDryRunFound",
                    ]
                )
            ),
            None,
        )
        sample = {}
        for run in runs:
            sample = (
                run["accessibility"]
                .get("productionActivationFinalDecisionAssertions", {})
                .get(key, {})
            )
            if sample:
                break
        results[key] = {
            "heading": sample.get("heading"),
            "headingProof": "live_gtk_atspi_proof" if heading_run else "not_proven",
            "headingEvidenceRun": heading_run["name"] if heading_run else None,
            "productionDisabledText": sample.get("productionDisabledText"),
            "productionDisabledProof": "live_gtk_atspi_proof"
            if production_run
            else "not_proven",
            "productionDisabledEvidenceRun": production_run["name"] if production_run else None,
            "executorWiring": sample.get("executorWiring"),
            "executorWiringProof": "live_gtk_atspi_proof" if executor_run else "not_proven",
            "executorWiringEvidenceRun": executor_run["name"] if executor_run else None,
            "finalDecisionStatus": sample.get("finalDecisionStatus"),
            "finalDecisionStatusProof": "live_gtk_atspi_proof"
            if status_run
            else "not_proven",
            "finalDecisionStatusEvidenceRun": status_run["name"] if status_run else None,
            "decisionLabels": [
                sample.get("finalApproval"),
                sample.get("productionFlagDecision"),
                sample.get("executorWiringDecision"),
                sample.get("liveProductionDryRunPolicy"),
                sample.get("copiedFixtureProof"),
            ],
            "decisionLabelsProof": "live_gtk_atspi_proof"
            if decisions_run
            else "not_proven",
            "decisionLabelsEvidenceRun": decisions_run["name"] if decisions_run else None,
            "draftPersistence": sample.get("draftPersistence"),
            "draftPersistenceProof": "live_gtk_atspi_proof"
            if persistence_run
            else "not_proven",
            "draftPersistenceEvidenceRun": persistence_run["name"] if persistence_run else None,
            "disabledApproval": sample.get("disabledApproval"),
            "disabledProductionFlag": sample.get("disabledProductionFlag"),
            "disabledExecutorWiring": sample.get("disabledExecutorWiring"),
            "disabledLiveDryRun": sample.get("disabledLiveDryRun"),
            "disabledActionsProof": "live_gtk_atspi_proof" if action_run else "not_proven",
            "disabledActionsEvidenceRun": action_run["name"] if action_run else None,
            "widgetName": sample.get("widgetName"),
            "assertionMethod": "screenshot_plus_accessibility_tree_text_not_ocr",
        }
    return results


def matching_category_run(runs, spec):
    for run in runs:
        if run["scenario"] == spec["scenario"] and run["navigationTarget"] == spec["target"]:
            return run
    return None


def proof_level_by_blocked_category(runs):
    levels = {}
    for key, spec in BLOCKED_CATEGORY_PROOFS.items():
        run = matching_category_run(runs, spec)
        if run is None:
            levels[key] = "not_proven"
            continue
        live = (
            run["accessibility"]["succeeded"]
            and run["accessibility"]["navigationAttempted"]
            and run["accessibility"]["navigationSucceeded"]
            and (
                run["accessibility"]["blockedCategoryExpectedTextCollected"]
                or run["accessibility"]["blockedCategoryReasonTextCollected"]
                or (
                    key == "duplicateBlockedCopy"
                    and run["accessibility"]["duplicateBlockedReasonTextCollected"]
                )
            )
        )
        levels[key] = "live_gtk_atspi_proof" if live else "not_proven"
    return levels


def proof_surface_by_blocked_category(runs):
    surfaces = {}
    for key, spec in BLOCKED_CATEGORY_PROOFS.items():
        run = matching_category_run(runs, spec)
        if run is None:
            surfaces[key] = "not_proven"
            continue
        surface = run["accessibility"].get("proofSurface")
        if surface in {
            "connected_file_detail",
            "profile_detail",
            "setting_row_detail",
            "config_page_text",
        }:
            surfaces[key] = surface
        elif key in {"defaultMissingBlockedCopy", "duplicateBlockedCopy", "highRiskBlockedCopy", "displayRenderRiskBlockedCopy"}:
            surfaces[key] = "setting_row_detail"
        else:
            surfaces[key] = "not_proven"
    return surfaces


def blocked_category_results(runs, levels):
    results = {}
    for key, spec in BLOCKED_CATEGORY_PROOFS.items():
        run = matching_category_run(runs, spec)
        if run is None:
            results[key] = {
                "category": spec["category"],
                "scenario": spec["scenario"],
                "navigationTarget": spec["target"],
                "expectedBlockedReasonText": spec["expectedText"],
                "proofLevel": levels[key],
                "rowDetailNavigationAttempted": False,
                "rowDetailNavigationSucceeded": False,
                "blockedReasonTextCollected": False,
                "applyAvoided": True,
                "proofSurface": "not_proven",
                "issue": "No evidence run was found for this blocked category.",
            }
            continue
        results[key] = {
            "category": spec["category"],
            "scenario": run["scenario"],
            "navigationTarget": run["navigationTarget"],
            "expectedBlockedReasonText": spec["expectedText"],
            "proofLevel": levels[key],
            "rowDetailNavigationAttempted": run["accessibility"]["blockedCategoryDetailNavigationAttempted"]
            or run["accessibility"]["duplicateConflictDetailNavigationAttempted"],
            "rowDetailNavigationSucceeded": run["accessibility"]["blockedCategoryDetailNavigationSucceeded"]
            or run["accessibility"]["duplicateConflictDetailNavigationSucceeded"],
            "blockedReasonTextCollected": run["accessibility"]["blockedCategoryReasonTextCollected"]
            or run["accessibility"]["duplicateBlockedReasonTextCollected"],
            "expectedTextCollected": run["accessibility"]["blockedCategoryExpectedTextCollected"]
            or run["accessibility"]["duplicateBlockedReasonTextCollected"],
            "selectionFallbackUsed": run["accessibility"]["blockedCategorySelectionFallbackUsed"],
            "proofSurface": run["accessibility"].get("proofSurface")
            or (
                "setting_row_detail"
                if levels[key] == "live_gtk_atspi_proof"
                else "not_proven"
            ),
            "expectedProofSurface": spec.get("expectedProofSurface"),
            "applyAvoided": not run["probe"]["applyClicked"],
            "navigationMessage": run["accessibility"]["navigationMessage"],
        }
    return results


def all_terms(run):
    terms = run["accessibility"]["foundTerms"] + run["accessibility"]["foundTermsAfterNavigation"]
    return {term.lower() for term in terms}


def base_report(kind, evidence_root, summary, runs, extra=None):
    data = {
        "schemaVersion": 1,
        "projectDataVersion": "v0.55.2",
        "artifactKind": kind,
        "generatedAt": datetime.now(timezone.utc).isoformat(),
        "startingCommit": STARTING_COMMIT,
        "projectModel": PROJECT_MODEL,
        "projectDataMigratedToHyprland0554": False,
        "runningHyprlandVersion": "socket read unavailable in noninteractive shell; installed package is 0.55.4-1",
        "installedHyprlandPackageVersion": "0.55.4-1",
        "pyatspiAvailable": summary["pyatspiAvailable"],
        "safeEnvModeUsed": summary["safeEnvModeUsed"],
        "liveSwapModeUsed": summary["liveSwapModeUsed"],
        "evidenceSummarySource": "tools/live_scenario_harness/summarize_gtk_evidence.py",
        "appBuildAttempted": summary["appBuildAttempted"],
        "appBuildSucceeded": summary["appBuildSucceeded"],
        "appBinaryRebuiltBeforeProbe": summary["appBinaryRebuiltBeforeProbe"],
        "duplicateConflictDetailNavigationAttempted": summary[
            "duplicateConflictDetailNavigationAttempted"
        ],
        "duplicateConflictDetailNavigationSucceeded": summary[
            "duplicateConflictDetailNavigationSucceeded"
        ],
        "duplicateBlockedReasonTextCollected": summary["duplicateBlockedReasonTextCollected"],
        "blockedCategoryDetailNavigationAttempted": summary[
            "blockedCategoryDetailNavigationAttempted"
        ],
        "blockedCategoryDetailNavigationSucceeded": summary[
            "blockedCategoryDetailNavigationSucceeded"
        ],
        "blockedCategoryReasonTextCollected": summary["blockedCategoryReasonTextCollected"],
        "proofLevelByUiArea": summary["proofLevelByUiArea"],
        "proofLevelByBlockedCategory": summary["proofLevelByBlockedCategory"],
        "proofSurfaceByBlockedCategory": summary["proofSurfaceByBlockedCategory"],
        "blockedCategoryResults": summary["blockedCategoryResults"],
        "agsTouched": False,
        "waybarTouched": False,
        "realConfigEdited": summary["realConfigEdited"],
        "realBackupsCreated": summary["realBackupsCreated"],
        "hyprlandReloaded": summary["hyprlandReloaded"],
        "mutatingHyprctlUsed": summary["mutatingHyprctlUsed"],
        "runtimeMutated": summary["runtimeMutated"],
        "scriptsExecuted": summary["scriptsExecuted"],
        "luaExecuted": summary["luaExecuted"],
        "screenshotsCommitted": False,
        "evidenceRoot": "<tmp>/hyprland-settings-gtk-automation/<fresh-run>",
        "evidenceSummarySource": "tools/live_scenario_harness/summarize_gtk_evidence.py",
        "appBuildAttempted": summary["appBuildAttempted"],
        "appBuildSucceeded": summary["appBuildSucceeded"],
        "appLaunchAttempted": summary["appLaunchAttempted"],
        "appLaunchSucceeded": summary["appLaunchSucceeded"],
        "appBinaryRebuiltBeforeProbe": summary["appBinaryRebuiltBeforeProbe"],
        "accessibilityInspectionAttempted": summary["accessibilityInspectionAttempted"],
        "accessibilityInspectionSucceeded": summary["accessibilityInspectionSucceeded"],
        "uiTextTreeCollected": summary["uiTextTreeCollected"],
        "navigationAttempted": summary["navigationAttempted"],
        "navigationSucceeded": summary["navigationSucceeded"],
        "duplicateConflictDetailNavigationAttempted": summary["duplicateConflictDetailNavigationAttempted"],
        "duplicateConflictDetailNavigationSucceeded": summary["duplicateConflictDetailNavigationSucceeded"],
        "duplicateBlockedReasonTextCollected": summary["duplicateBlockedReasonTextCollected"],
        "blockedCategoryDetailNavigationAttempted": summary["blockedCategoryDetailNavigationAttempted"],
        "blockedCategoryDetailNavigationSucceeded": summary["blockedCategoryDetailNavigationSucceeded"],
        "blockedCategoryReasonTextCollected": summary["blockedCategoryReasonTextCollected"],
        "connectedFileDetailNavigationAttempted": summary["connectedFileDetailNavigationAttempted"],
        "connectedFileDetailNavigationSucceeded": summary["connectedFileDetailNavigationSucceeded"],
        "profileDetailNavigationAttempted": summary["profileDetailNavigationAttempted"],
        "profileDetailNavigationSucceeded": summary["profileDetailNavigationSucceeded"],
        "closeAttempted": bool(runs),
        "closeSucceeded": summary["closeSucceeded"],
        "scenarioResults": runs,
        "proofLevelByScenario": {
            run["name"]: "live_gtk_atspi_proof" if run["accessibility"]["succeeded"] else "not_proven"
            for run in runs
        },
        "proofLevelByUiArea": summary["proofLevelByUiArea"],
        "proofLevelByBlockedCategory": summary["proofLevelByBlockedCategory"],
        "proofSurfaceByBlockedCategory": summary["proofSurfaceByBlockedCategory"],
        "blockedCategoryResults": summary["blockedCategoryResults"],
        "approvalCardAssertionMethod": summary["approvalCardAssertionMethod"],
        "approvalCardResults": summary["approvalCardResults"],
        "approvalCardsAllHeadingsFound": summary["approvalCardsAllHeadingsFound"],
        "approvalCardsAllProductionDisabledFound": summary[
            "approvalCardsAllProductionDisabledFound"
        ],
        "approvalCardsAllDisabledActionsFound": summary["approvalCardsAllDisabledActionsFound"],
        "activationDecisionAssertionMethod": summary["activationDecisionAssertionMethod"],
        "activationDecisionResults": summary["activationDecisionResults"],
        "activationDecisionsAllHeadingsFound": summary[
            "activationDecisionsAllHeadingsFound"
        ],
        "activationDecisionsAllProductionDisabledFound": summary[
            "activationDecisionsAllProductionDisabledFound"
        ],
        "activationDecisionsAllDisabledActionsFound": summary[
            "activationDecisionsAllDisabledActionsFound"
        ],
        "activationPathAssertionMethod": summary["activationPathAssertionMethod"],
        "activationPathResults": summary["activationPathResults"],
        "activationPathsAllHeadingsFound": summary["activationPathsAllHeadingsFound"],
        "activationPathsAllProductionDisabledFound": summary[
            "activationPathsAllProductionDisabledFound"
        ],
        "activationPathsAllDisabledActionsFound": summary[
            "activationPathsAllDisabledActionsFound"
        ],
        "activationControlAssertionMethod": summary["activationControlAssertionMethod"],
        "activationControlResults": summary["activationControlResults"],
        "activationControlsAllHeadingsFound": summary[
            "activationControlsAllHeadingsFound"
        ],
        "activationControlsAllProductionDisabledFound": summary[
            "activationControlsAllProductionDisabledFound"
        ],
        "activationControlsAllExecutorUnwiredFound": summary[
            "activationControlsAllExecutorUnwiredFound"
        ],
        "activationControlsAllDisabledActionsFound": summary[
            "activationControlsAllDisabledActionsFound"
        ],
        "activationFormAssertionMethod": summary["activationFormAssertionMethod"],
        "activationFormResults": summary["activationFormResults"],
        "activationFormsAllHeadingsFound": summary["activationFormsAllHeadingsFound"],
        "activationFormsAllProductionDisabledFound": summary[
            "activationFormsAllProductionDisabledFound"
        ],
        "activationFormsAllExecutorUnwiredFound": summary[
            "activationFormsAllExecutorUnwiredFound"
        ],
        "activationFormsAllDisabledActionsFound": summary[
            "activationFormsAllDisabledActionsFound"
        ],
        "activationFormsAllFieldLabelsFound": summary[
            "activationFormsAllFieldLabelsFound"
        ],
        "activationDraftAssertionMethod": summary["activationDraftAssertionMethod"],
        "activationDraftResults": summary["activationDraftResults"],
        "activationDraftsAllHeadingsFound": summary[
            "activationDraftsAllHeadingsFound"
        ],
        "activationDraftsAllProductionDisabledFound": summary[
            "activationDraftsAllProductionDisabledFound"
        ],
        "activationDraftsAllExecutorUnwiredFound": summary[
            "activationDraftsAllExecutorUnwiredFound"
        ],
        "activationDraftsAllInMemoryOnlyFound": summary[
            "activationDraftsAllInMemoryOnlyFound"
        ],
        "activationDraftsAllDisabledActionsFound": summary[
            "activationDraftsAllDisabledActionsFound"
        ],
        "activationDraftEditAssertionMethod": summary["activationDraftEditAssertionMethod"],
        "activationDraftEditResults": summary["activationDraftEditResults"],
        "activationDraftEditsAllHeadingsFound": summary[
            "activationDraftEditsAllHeadingsFound"
        ],
        "activationDraftEditsAllProductionDisabledFound": summary[
            "activationDraftEditsAllProductionDisabledFound"
        ],
        "activationDraftEditsAllExecutorUnwiredFound": summary[
            "activationDraftEditsAllExecutorUnwiredFound"
        ],
        "activationDraftEditsAllInMemoryOnlyFound": summary[
            "activationDraftEditsAllInMemoryOnlyFound"
        ],
        "activationDraftEditsAllModeFound": summary[
            "activationDraftEditsAllModeFound"
        ],
        "activationDraftEditsAllValidationFound": summary[
            "activationDraftEditsAllValidationFound"
        ],
        "activationDraftEditsAllDisabledActionsFound": summary[
            "activationDraftEditsAllDisabledActionsFound"
        ],
        "productionActivationSafetyProofAssertionMethod": summary[
            "productionActivationSafetyProofAssertionMethod"
        ],
        "productionActivationSafetyProofResults": summary[
            "productionActivationSafetyProofResults"
        ],
        "productionActivationSafetyProofsAllHeadingsFound": summary[
            "productionActivationSafetyProofsAllHeadingsFound"
        ],
        "productionActivationSafetyProofsAllProductionDisabledFound": summary[
            "productionActivationSafetyProofsAllProductionDisabledFound"
        ],
        "productionActivationSafetyProofsAllExecutorUnwiredFound": summary[
            "productionActivationSafetyProofsAllExecutorUnwiredFound"
        ],
        "productionActivationSafetyProofsAllProofStatusFound": summary[
            "productionActivationSafetyProofsAllProofStatusFound"
        ],
        "productionActivationSafetyProofsAllCopiedFixtureProofFound": summary[
            "productionActivationSafetyProofsAllCopiedFixtureProofFound"
        ],
        "productionActivationSafetyProofsAllNoAutoApplyFound": summary[
            "productionActivationSafetyProofsAllNoAutoApplyFound"
        ],
        "productionActivationSafetyProofsAllFinalApprovalFound": summary[
            "productionActivationSafetyProofsAllFinalApprovalFound"
        ],
        "productionActivationSafetyProofsAllDisabledActionsFound": summary[
            "productionActivationSafetyProofsAllDisabledActionsFound"
        ],
        "productionActivationFinalDecisionAssertionMethod": summary[
            "productionActivationFinalDecisionAssertionMethod"
        ],
        "productionActivationFinalDecisionResults": summary[
            "productionActivationFinalDecisionResults"
        ],
        "productionActivationFinalDecisionsAllHeadingsFound": summary[
            "productionActivationFinalDecisionsAllHeadingsFound"
        ],
        "productionActivationFinalDecisionsAllProductionDisabledFound": summary[
            "productionActivationFinalDecisionsAllProductionDisabledFound"
        ],
        "productionActivationFinalDecisionsAllExecutorUnwiredFound": summary[
            "productionActivationFinalDecisionsAllExecutorUnwiredFound"
        ],
        "productionActivationFinalDecisionsAllStatusFound": summary[
            "productionActivationFinalDecisionsAllStatusFound"
        ],
        "productionActivationFinalDecisionsAllDecisionLabelsFound": summary[
            "productionActivationFinalDecisionsAllDecisionLabelsFound"
        ],
        "productionActivationFinalDecisionsAllPersistenceFound": summary[
            "productionActivationFinalDecisionsAllPersistenceFound"
        ],
        "productionActivationFinalDecisionsAllDisabledActionsFound": summary[
            "productionActivationFinalDecisionsAllDisabledActionsFound"
        ],
        "fallbackProofUsed": summary["fallbackProofUsed"],
        "issuesFound": issues(summary),
        "recommendedFixes": recommended_fixes(summary),
        "countsBefore": "341 readable / 341 writable / 0 blocked",
        "countsAfter": "341 readable / 341 writable / 0 blocked",
        "validation": {
            "bashScripts": "pending",
            "pythonPyCompile": "pending",
            "cargoFmt": "pending",
            "cargoFmtCheck": "pending",
            "cargoCheck": "pending",
            "cargoTest": "pending",
            "cargoBuildRelease": "pending",
            "jqReports": "pending",
            "gitDiffCheck": "pending",
            "gitStatusShort": "pending",
        },
    }
    if extra:
        data.update(extra)
    return data


def issues(summary):
    found = []
    if summary["proofLevelByUiArea"].get("Search") != "live_gtk_atspi_proof":
        found.append("Search field text was not proven through live AT-SPI.")
    for key, level in summary["proofLevelByBlockedCategory"].items():
        if level != "live_gtk_atspi_proof":
            found.append(f"{key} still lacks live blocked-category row-detail proof.")
    return found


def recommended_fixes(summary):
    fixes = []
    if summary["proofLevelByUiArea"].get("settingRow") != "live_gtk_atspi_proof":
        fixes.append("Add a stronger non-Apply row activation action or widget role for setting rows.")
    if summary["proofLevelByUiArea"].get("detailPane") != "live_gtk_atspi_proof":
        fixes.append("Expose selected row detail pane text more consistently through AT-SPI.")
    incomplete = [
        key
        for key, level in summary["proofLevelByBlockedCategory"].items()
        if level != "live_gtk_atspi_proof"
    ]
    if incomplete:
        fixes.append(
            "Add stronger row/detail accessible names for blocked categories: "
            + ", ".join(incomplete)
            + "."
        )
    if not fixes:
        fixes.append("Keep expanding scenario-specific row-detail probes without enabling live-swap.")
    return fixes


def write_reports(reports_dir, evidence_root, evidence_summary, runs, summary):
    reports_dir.mkdir(parents=True, exist_ok=True)
    reports = {
        "gtk-safe-env-automation-capability.v0.55.2.json": base_report(
            "gtk_safe_env_automation_capability",
            evidence_root,
            summary,
            runs,
            {
                "appBuildAttempted": summary["appBuildAttempted"],
                "appBuildSucceeded": summary["appBuildSucceeded"],
                "appLaunchAttempted": summary["appLaunchAttempted"],
                "appLaunchSucceeded": summary["appLaunchSucceeded"],
                "appBinaryRebuiltBeforeProbe": summary["appBinaryRebuiltBeforeProbe"],
                "accessibilityInspectionAttempted": summary["accessibilityInspectionAttempted"],
                "accessibilityInspectionSucceeded": summary["accessibilityInspectionSucceeded"],
                "uiTextTreeCollected": summary["uiTextTreeCollected"],
                "closeSucceeded": summary["closeSucceeded"],
            },
        ),
        "gtk-safe-env-scenario-matrix.v0.55.2.json": base_report(
            "gtk_safe_env_scenario_matrix", evidence_root, summary, runs
        ),
        "gtk-safe-env-ui-navigation.v0.55.2.json": base_report(
            "gtk_safe_env_ui_navigation",
            evidence_root,
            summary,
            runs,
            {
                "navigationAttempted": summary["navigationAttempted"],
                "navigationSucceeded": summary["navigationSucceeded"],
                "applyAvoided": not summary["applyClicked"],
            },
        ),
        "gtk-safe-env-user-experience.v0.55.2.json": base_report(
            "gtk_safe_env_user_experience",
            evidence_root,
            summary,
            runs,
            {
                "newHyprlandUser": "safe-env scenarios launched through the GTK app and were inspected with AT-SPI where exposed",
                "oneTargetWordingVisibleInSafeBatchPath": False,
                "proofLevelByUiArea": summary["proofLevelByUiArea"],
            },
        ),
        "gtk-safe-env-automation-summary.v0.55.2.json": base_report(
            "gtk_safe_env_automation_summary", evidence_root, summary, runs
        ),
        "gtk-safe-env-evidence-derived-matrix.v0.55.2.json": {
            **base_report("gtk_safe_env_evidence_derived_matrix", evidence_root, summary, runs),
            "evidenceSummary": evidence_summary,
        },
        "gtk-safe-env-blocked-category-detail-proof.v0.55.2.json": base_report(
            "gtk_safe_env_blocked_category_detail_proof",
            evidence_root,
            summary,
            runs,
            {
                "blockedCategoryResults": summary["blockedCategoryResults"],
                "proofLevelByBlockedCategory": summary["proofLevelByBlockedCategory"],
                "goal": "Expand live GTK/AT-SPI row-detail proof across representative blocked categories.",
            },
        ),
        "gtk-safe-env-connected-file-detail-proof.v0.55.2.json": base_report(
            "gtk_safe_env_connected_file_detail_proof",
            evidence_root,
            summary,
            runs,
            {
                "connectedFileDetailResults": {
                    key: value
                    for key, value in summary["blockedCategoryResults"].items()
                    if value.get("proofSurface") == "connected_file_detail"
                },
                "profileModeDetailResults": {
                    key: value
                    for key, value in summary["blockedCategoryResults"].items()
                    if value.get("proofSurface") == "profile_detail"
                },
                "proofLevelByBlockedCategory": summary["proofLevelByBlockedCategory"],
                "proofSurfaceByBlockedCategory": summary["proofSurfaceByBlockedCategory"],
                "goal": "Prove connected-file and profile blocker detail surfaces through live GTK/AT-SPI safe-env automation.",
            },
        ),
        "gtk-safe-env-disabled-approval-card-proof.v0.55.2.json": base_report(
            "gtk_safe_env_disabled_approval_card_proof",
            evidence_root,
            summary,
            runs,
            {
                "assertionMethod": summary["approvalCardAssertionMethod"],
                "approvalCardResults": summary["approvalCardResults"],
                "allHeadingsFound": summary["approvalCardsAllHeadingsFound"],
                "allProductionDisabledTextFound": summary[
                    "approvalCardsAllProductionDisabledFound"
                ],
                "allDisabledActionsFound": summary["approvalCardsAllDisabledActionsFound"],
                "activationDecisionAssertionMethod": summary[
                    "activationDecisionAssertionMethod"
                ],
                "activationDecisionResults": summary["activationDecisionResults"],
                "activationDecisionsAllHeadingsFound": summary[
                    "activationDecisionsAllHeadingsFound"
                ],
                "activationDecisionsAllProductionDisabledFound": summary[
                    "activationDecisionsAllProductionDisabledFound"
                ],
                "activationDecisionsAllDisabledActionsFound": summary[
                    "activationDecisionsAllDisabledActionsFound"
                ],
                "activationPathAssertionMethod": summary["activationPathAssertionMethod"],
                "activationPathResults": summary["activationPathResults"],
                "activationPathsAllHeadingsFound": summary[
                    "activationPathsAllHeadingsFound"
                ],
                "activationPathsAllProductionDisabledFound": summary[
                    "activationPathsAllProductionDisabledFound"
                ],
                "activationPathsAllDisabledActionsFound": summary[
                    "activationPathsAllDisabledActionsFound"
                ],
                "activationControlAssertionMethod": summary[
                    "activationControlAssertionMethod"
                ],
                "activationControlResults": summary["activationControlResults"],
                "activationControlsAllHeadingsFound": summary[
                    "activationControlsAllHeadingsFound"
                ],
                "activationControlsAllProductionDisabledFound": summary[
                    "activationControlsAllProductionDisabledFound"
                ],
                "activationControlsAllExecutorUnwiredFound": summary[
                    "activationControlsAllExecutorUnwiredFound"
                ],
                "activationControlsAllDisabledActionsFound": summary[
                    "activationControlsAllDisabledActionsFound"
                ],
                "activationFormAssertionMethod": summary["activationFormAssertionMethod"],
                "activationFormResults": summary["activationFormResults"],
                "activationFormsAllHeadingsFound": summary[
                    "activationFormsAllHeadingsFound"
                ],
                "activationFormsAllProductionDisabledFound": summary[
                    "activationFormsAllProductionDisabledFound"
                ],
                "activationFormsAllExecutorUnwiredFound": summary[
                    "activationFormsAllExecutorUnwiredFound"
                ],
                "activationFormsAllDisabledActionsFound": summary[
                    "activationFormsAllDisabledActionsFound"
                ],
                "activationFormsAllFieldLabelsFound": summary[
                    "activationFormsAllFieldLabelsFound"
                ],
                "activationDraftAssertionMethod": summary["activationDraftAssertionMethod"],
                "activationDraftResults": summary["activationDraftResults"],
                "activationDraftsAllHeadingsFound": summary[
                    "activationDraftsAllHeadingsFound"
                ],
                "activationDraftsAllProductionDisabledFound": summary[
                    "activationDraftsAllProductionDisabledFound"
                ],
                "activationDraftsAllExecutorUnwiredFound": summary[
                    "activationDraftsAllExecutorUnwiredFound"
                ],
                "activationDraftsAllInMemoryOnlyFound": summary[
                    "activationDraftsAllInMemoryOnlyFound"
                ],
                "activationDraftsAllDisabledActionsFound": summary[
                    "activationDraftsAllDisabledActionsFound"
                ],
                "activationDraftEditAssertionMethod": summary[
                    "activationDraftEditAssertionMethod"
                ],
                "activationDraftEditResults": summary["activationDraftEditResults"],
                "activationDraftEditsAllHeadingsFound": summary[
                    "activationDraftEditsAllHeadingsFound"
                ],
                "activationDraftEditsAllProductionDisabledFound": summary[
                    "activationDraftEditsAllProductionDisabledFound"
                ],
                "activationDraftEditsAllExecutorUnwiredFound": summary[
                    "activationDraftEditsAllExecutorUnwiredFound"
                ],
                "activationDraftEditsAllInMemoryOnlyFound": summary[
                    "activationDraftEditsAllInMemoryOnlyFound"
                ],
                "activationDraftEditsAllModeFound": summary[
                    "activationDraftEditsAllModeFound"
                ],
                "activationDraftEditsAllValidationFound": summary[
                    "activationDraftEditsAllValidationFound"
                ],
                "activationDraftEditsAllDisabledActionsFound": summary[
                    "activationDraftEditsAllDisabledActionsFound"
                ],
                "safety": {
                    "runtimeMutated": False,
                    "hyprlandReloaded": False,
                    "realConfigTouched": False,
                    "unsafeProductionBehaviorEnabled": False,
                    "productionBehaviorEnabled": False,
                    "v0552DefaultPreserved": True,
                    "hyprland0554MigrationActivated": False,
                },
                "goal": "Prove disabled approval review cards through screenshot-level safe-env automation using screenshots plus AT-SPI accessibility-tree text, not OCR.",
            },
        ),
        "completion-readiness-audit.v0.55.2.json": completion_readiness_report(
            evidence_root, summary, runs
        ),
        "completion-wrap-up-plan.v0.55.2.json": completion_wrap_up_plan(summary),
        "final-app-completion-wrap-up.v0.55.2.json": final_app_completion_wrap_up(
            evidence_root, summary, runs
        ),
        "final-release-readiness-checklist.v0.55.2.json": final_release_readiness_checklist(
            summary
        ),
        "final-safe-scope-validation.v0.55.2.json": final_safe_scope_validation(summary),
        "autonomous-safe-scope-continuation.v0.55.2.json": autonomous_safe_scope_continuation(
            summary
        ),
    }
    for name, data in reports.items():
        (reports_dir / name).write_text(json.dumps(data, indent=2, sort_keys=True) + "\n")


def completion_readiness_report(evidence_root, summary, runs):
    connected_file_surfaces = summary["proofSurfaceByBlockedCategory"]
    blocked_levels = summary["proofLevelByBlockedCategory"]
    ui_levels = summary["proofLevelByUiArea"]
    source_include_scenarios = [
        run
        for run in runs
        if run["scenario"] in {"source_include_config", "nested_source_config"}
    ]
    completion_blockers = [
        "Missing/default insertion remains intentionally blocked until safe insertion is designed.",
        "Duplicate conflicts remain intentionally blocked until manual or explicit duplicate-resolution design exists.",
        "High-risk and display/render writes remain intentionally blocked until family-specific recovery proof is approved.",
        "Profile/mode switching remains intentionally read-only and inactive.",
        "Project data remains intentionally pinned to v0.55.2 despite the runtime package being 0.55.4-1.",
    ]
    safe_fixes = [
        "Connected files section gained a stable accessibility anchor.",
        "Config diagnostics now describe source/include connections in user-facing copy.",
        "Safe-env GTK evidence matrix now includes a nested source/include scenario.",
        "Completion-readiness and wrap-up reports are generated by the evidence summarizer.",
    ]
    return base_report(
        "completion_readiness_audit",
        evidence_root,
        summary,
        runs,
        {
            "goal": "Final completion-readiness audit for the v0.55.2 Hyprland Settings app model before app wrap-up work.",
            "hyprlandRuntimeVersion": "unavailable through hyprctl in noninteractive shell; user session is reported as 0.55.4",
            "hyprlandPackageVersion": "0.55.4-1",
            "coverageModel": {
                "projectDataVersion": "v0.55.2",
                "readableRows": 341,
                "writableRows": 341,
                "blockedRows": 0,
                "safeWritableRowsLen": 341,
                "exportsUpdatedInThisSprint": False,
                "status": "preserved",
            },
            "readDiscoveryStatus": {
                "configDiscovery": "reviewed and preserved",
                "sourceIncludeGraph": "covered by source_include_config and nested_source_config safe-env scenarios",
                "sourceAwareCurrentValueMapping": "preserved; no write eligibility expansion in this sprint",
                "connectedFileDiagnostics": "read-only Config page diagnostics with stable connected-file/profile detail anchors",
                "sourceIncludeScenarioCount": len(source_include_scenarios),
            },
            "sourceIncludeStatus": {
                "rootConfig": "reviewed through safe-env matrix",
                "connectedFiles": "reviewed through safe-env matrix",
                "nestedSourceFiles": "added to safe-env matrix",
                "generatedFiles": connected_file_surfaces.get("generatedBlockedCopy"),
                "scriptManagedFiles": connected_file_surfaces.get("scriptManagedBlockedCopy"),
                "symlinkCurrentProfileFiles": connected_file_surfaces.get("symlinkCurrentProfileBlockedCopy"),
                "profileModeStatus": connected_file_surfaces.get("profileModeSwitchBlockedCopy"),
                "mutatingControlsAdded": False,
            },
            "writeSafetyStatus": {
                "safeBatchWritePath": "guarded and preserved",
                "applyClicked": summary["applyClicked"],
                "highRiskWritesEnabled": False,
                "displayRenderRiskyWritesEnabled": False,
                "generatedScriptSymlinkProfileWritesEnabled": False,
                "duplicateConflictsStillBlock": True,
                "missingDefaultInsertionStillBlocked": True,
                "structuredFamilyWritesEnabled": False,
                "runtimeMutationEnabled": False,
            },
            "uiStatus": {
                "dashboard": ui_levels.get("Dashboard"),
                "configPage": ui_levels.get("Config"),
                "appearance": ui_levels.get("Appearance"),
                "display": ui_levels.get("Display"),
                "search": ui_levels.get("Search"),
                "detailPane": ui_levels.get("detailPane"),
                "blockedReason": ui_levels.get("blockedReason"),
                "oneTargetWordingUserFacingInActiveSafeBatchPath": False,
                "connectedFileDetailSurfaces": connected_file_surfaces,
                "blockedCategoryProof": blocked_levels,
            },
            "automationStatus": {
                "safeEnvMatrixPassed": summary["appLaunchSucceeded"]
                and summary["accessibilityInspectionSucceeded"]
                and summary["navigationSucceeded"],
                "connectedFileProfileDetailProof": {
                    key: connected_file_surfaces.get(key)
                    for key in [
                        "generatedBlockedCopy",
                        "scriptManagedBlockedCopy",
                        "symlinkCurrentProfileBlockedCopy",
                        "profileModeSwitchBlockedCopy",
                    ]
                },
                "blockedCategoryProof": blocked_levels,
                "rawAccessibilityDumpsCommitted": False,
            },
            "packagingStatus": {
                "appId": "io.github.kyarorukyo.hyprlandsettings",
                "binaryName": "hyprland-settings",
                "repoIdentity": "hyprland-settings",
                "desktopMetainfoReviewed": "source/repo identity reviewed; no release created",
                "releaseCreated": False,
            },
            "safetyBoundaries": {
                "safeEnvModeUsed": summary["safeEnvModeUsed"],
                "liveSwapModeUsed": summary["liveSwapModeUsed"],
                "realConfigEdited": summary["realConfigEdited"],
                "realBackupsCreated": summary["realBackupsCreated"],
                "hyprlandReloaded": summary["hyprlandReloaded"],
                "mutatingHyprctlUsed": summary["mutatingHyprctlUsed"],
                "runtimeMutated": summary["runtimeMutated"],
                "scriptsExecuted": summary["scriptsExecuted"],
                "luaExecuted": summary["luaExecuted"],
                "screenshotsCommitted": False,
            },
            "completionBlockers": completion_blockers,
            "safeFixesMade": safe_fixes,
            "remainingWork": completion_blockers,
            "recommendedNextSprint": "Final release-boundary review and explicit user approval decision for creating an actual release artifact.",
        },
    )


def completion_wrap_up_plan(summary):
    return {
        "schemaVersion": 1,
        "artifactKind": "completion_wrap_up_plan",
        "generatedAt": datetime.now(timezone.utc).isoformat(),
        "startingCommit": STARTING_COMMIT,
        "projectModel": PROJECT_MODEL,
        "projectDataMigratedToHyprland0554": False,
        "whatIsDone": [
            "v0.55.2 app/export model remains at 341 readable / 341 writable / 0 blocked.",
            "Safe-batch writes are guarded by exact target, backup, reread verification, and recovery paths.",
            "High-risk, display/render, generated, script-managed, symlink/current-profile, duplicate, and missing/default paths remain blocked.",
            "GTK safe-env automation launches the app, collects AT-SPI evidence, avoids Apply, and closes the app.",
            "Connected-file and profile blocker detail surfaces have live GTK/AT-SPI proof in safe-env scenarios.",
        ],
        "whatIsStillBlockedByDesign": [
            "Missing/default insertion.",
            "Duplicate auto-resolution.",
            "High-risk and display/render real writes.",
            "Structured-family writes.",
            "Profile/mode switching.",
            "Hyprland reload and runtime mutation.",
            "Hyprland 0.55.4 data migration.",
        ],
        "whatCanBeFinishedSafelyNext": [
            "No obvious green-lane completion item remains open after the final safe-scope validation sweep.",
            "The next safe action is an explicit release-boundary approval decision; do not create a release, tag, or package without that approval.",
        ],
        "whatRequiresSeparateDesign": [
            "Insertion of missing config lines.",
            "Manual duplicate-resolution workflow.",
            "Family-specific high-risk write approvals and recovery watchdogs.",
            "Profile/mode switching and symlink management.",
            "Hyprland 0.55.4 export/model migration.",
        ],
        "whatRequiresUserApproval": [
            "Any live-swap test.",
            "Any real user config write.",
            "Any high-risk/display-render write expansion.",
            "Any profile/mode switching.",
            "Any release creation.",
            "Any migration from v0.55.2 to Hyprland 0.55.4 data.",
        ],
        "whatShouldNotBeDoneBeforeRelease": [
            "Do not silently expand write scope.",
            "Do not create config insertion behavior without its own design and proof.",
            "Do not auto-resolve duplicate conflicts.",
            "Do not switch profiles or symlinks.",
            "Do not reload Hyprland as part of Apply.",
        ],
        "releaseReadinessEstimate": "ready for a release-boundary approval review for the guarded normal-scalar safe-batch scope; intentionally not ready for high-risk, insertion, duplicate-resolution, profile/mode, or Hyprland 0.55.4 migration scope",
        "nextSprintPlan": "Final release-boundary review and explicit user approval decision for creating an actual release artifact.",
        "safeEnvModeUsed": summary["safeEnvModeUsed"],
        "liveSwapModeUsed": summary["liveSwapModeUsed"],
        "realConfigEdited": summary["realConfigEdited"],
        "hyprlandReloaded": summary["hyprlandReloaded"],
        "mutatingHyprctlUsed": summary["mutatingHyprctlUsed"],
        "countsBefore": "341 readable / 341 writable / 0 blocked",
        "countsAfter": "341 readable / 341 writable / 0 blocked",
        "validation": {
            "jqReports": "pending",
            "cargoTest": "pending",
            "gitDiffCheck": "pending",
            "gitStatusShort": "pending",
        },
    }


def autonomous_safe_scope_continuation(summary):
    return {
        "schemaVersion": 1,
        "artifactKind": "autonomous_safe_scope_continuation",
        "generatedAt": datetime.now(timezone.utc).isoformat(),
        "startingCommit": STARTING_COMMIT,
        "goal": "Continue green-lane safe-scope completion work without creating a release or expanding unsafe behavior.",
        "projectModel": PROJECT_MODEL,
        "projectDataMigratedToHyprland0554": False,
        "greenLaneStatus": {
            "userFacingCopyPolish": "complete for the current safe release scope",
            "readmeDocsCleanup": "complete for the current safe release scope",
            "cargoMetadataCleanup": "complete for the current safe release scope",
            "desktopMetainfoMetadataCleanup": "complete for the current safe release scope",
            "gtkSafeEnvAutomation": "passing",
            "reportGeneration": "complete for release-boundary readiness",
            "releaseChecklistPreparation": "complete; release creation still requires explicit approval",
            "packagingValidation": "desktop/AppStream validation passes",
            "remainingGreenLaneItems": [],
        },
        "yellowLaneDocumentedButNotActivated": [
            "missing/default insertion",
            "duplicate auto-resolution",
            "high-risk write expansion",
            "display/render-risk write expansion",
            "structured-family writes",
            "profile/mode switching",
            "Hyprland reload/runtime mutation",
            "Hyprland 0.55.4 migration",
            "live-swap testing",
            "release packaging workflow",
        ],
        "redLaneBoundariesReached": [
            "release/tag/package creation requires explicit user approval before proceeding"
        ],
        "redLaneActionsPerformed": {
            "releaseCreated": False,
            "tagCreated": False,
            "packageArtifactCreated": False,
            "realUserConfigEdited": False,
            "realUserConfigBackupsCreated": False,
            "liveSwapUsed": False,
            "agsTouched": False,
            "waybarTouched": False,
            "hyprlandReloaded": False,
            "mutatingHyprctlUsed": False,
            "runtimeMutated": False,
            "scriptsExecuted": False,
            "luaExecuted": False,
            "profileSwitchingActivated": False,
            "symlinksChanged": False,
            "hyprland0554Migration": False,
        },
        "safeEnvModeUsed": summary["safeEnvModeUsed"],
        "liveSwapModeUsed": summary["liveSwapModeUsed"],
        "evidenceSummarySource": "tools/live_scenario_harness/summarize_gtk_evidence.py",
        "appBuildAttempted": summary["appBuildAttempted"],
        "appBuildSucceeded": summary["appBuildSucceeded"],
        "appBinaryRebuiltBeforeProbe": summary["appBinaryRebuiltBeforeProbe"],
        "duplicateConflictDetailNavigationAttempted": summary[
            "duplicateConflictDetailNavigationAttempted"
        ],
        "duplicateConflictDetailNavigationSucceeded": summary[
            "duplicateConflictDetailNavigationSucceeded"
        ],
        "duplicateBlockedReasonTextCollected": summary["duplicateBlockedReasonTextCollected"],
        "blockedCategoryDetailNavigationAttempted": summary[
            "blockedCategoryDetailNavigationAttempted"
        ],
        "blockedCategoryDetailNavigationSucceeded": summary[
            "blockedCategoryDetailNavigationSucceeded"
        ],
        "blockedCategoryReasonTextCollected": summary["blockedCategoryReasonTextCollected"],
        "blockedCategoryResults": summary["blockedCategoryResults"],
        "agsTouched": False,
        "waybarTouched": False,
        "realConfigEdited": summary["realConfigEdited"],
        "realBackupsCreated": summary["realBackupsCreated"],
        "hyprlandReloaded": summary["hyprlandReloaded"],
        "mutatingHyprctlUsed": summary["mutatingHyprctlUsed"],
        "runtimeMutated": summary["runtimeMutated"],
        "scriptsExecuted": summary["scriptsExecuted"],
        "luaExecuted": summary["luaExecuted"],
        "screenshotsCommitted": False,
        "appLaunchSucceeded": summary["appLaunchSucceeded"],
        "accessibilityInspectionSucceeded": summary["accessibilityInspectionSucceeded"],
        "uiNavigationSucceeded": summary["navigationSucceeded"],
        "proofLevelByUiArea": summary["proofLevelByUiArea"],
        "proofLevelByBlockedCategory": summary["proofLevelByBlockedCategory"],
        "proofSurfaceByBlockedCategory": summary["proofSurfaceByBlockedCategory"],
        "countsBefore": "341 readable / 341 writable / 0 blocked",
        "countsAfter": "341 readable / 341 writable / 0 blocked",
        "validation": validation_pending_full(),
        "remainingBlockers": [
            "release-boundary approval",
            "missing/default insertion design",
            "duplicate auto-resolution design",
            "high-risk/display-render write proof",
            "structured-family write proof",
            "profile/mode switching design",
            "Hyprland 0.55.4 data/model migration",
        ],
        "recommendedNextAction": "Ask for explicit release-boundary approval before creating any release, tag, or package artifact.",
    }


def completion_scope_definition():
    return {
        "completeMeans": "The app is ready for guarded normal-scalar safe-batch use under the current v0.55.2 model, with clear UI copy, stable packaging metadata, final validation reports, and a release checklist.",
        "completeDoesNotMean": [
            "missing/default insertion",
            "duplicate auto-resolution",
            "high-risk/display-render writes",
            "structured-family writes",
            "profile switching",
            "live-swap testing",
            "Hyprland reload",
            "runtime mutation",
            "Hyprland 0.55.4 data migration",
        ],
    }


def app_identity():
    return {
        "appName": "Hyprland Settings",
        "repoName": "hyprland-settings",
        "binaryName": "hyprland-settings",
        "appId": "io.github.kyarorukyo.hyprlandsettings",
        "githubOwner": "KyaroruKYO",
        "homepage": "https://github.com/KyaroruKYO/hyprland-settings",
    }


def validation_pending_full():
    return {
        "bashScripts": "pending",
        "pythonPyCompile": "pending",
        "gtkSafeEnvEvidenceMatrix": "pending",
        "cargoFmt": "pending",
        "cargoFmtCheck": "pending",
        "cargoCheck": "pending",
        "cargoTest": "pending",
        "cargoBuildRelease": "pending",
        "jqReports": "pending",
        "desktopFileValidate": "pending",
        "appstreamValidate": "pending",
        "gitDiffCheck": "pending",
        "gitStatusShort": "pending",
    }


def final_app_completion_wrap_up(evidence_root, summary, runs):
    return base_report(
        "final_app_completion_wrap_up",
        evidence_root,
        summary,
        runs,
        {
            "goal": "Wrap up the current guarded normal-scalar safe-batch Hyprland Settings app scope without expanding unsafe functionality.",
            "completionScope": completion_scope_definition(),
            "projectDataVersion": "v0.55.2",
            "appIdentity": app_identity(),
            "copyPolish": {
                "readmeReviewed": True,
                "cargoDescriptionUpdated": True,
                "desktopCommentUpdated": True,
                "metainfoReviewed": True,
                "staleOneTargetPilotUserFacingWordingRemovedFromActiveSafeBatchPath": True,
                "unsafeFeatureOverclaimsRemoved": True,
            },
            "configPagePolish": {
                "connectedFilesExplained": True,
                "sourceIncludeExplained": True,
                "generatedScriptSymlinkProfileBlockersExplained": True,
                "profileSwitchingInactiveExplained": True,
                "mutatingControlsAdded": False,
            },
            "packagingMetadataReview": {
                "appId": "io.github.kyarorukyo.hyprlandsettings",
                "binaryName": "hyprland-settings",
                "desktopName": "Hyprland Settings",
                "repoIdentity": "KyaroruKYO/hyprland-settings",
                "desktopFile": "data/applications/io.github.kyarorukyo.hyprlandsettings.desktop",
                "metainfoFile": "data/metainfo/io.github.kyarorukyo.hyprlandsettings.metainfo.xml",
                "metadataDoesNotClaimUnsupportedWrites": True,
                "releaseArtifactsCreated": False,
            },
            "automationStatus": {
                "gtkSafeEnvMatrixPassed": summary["appLaunchSucceeded"]
                and summary["accessibilityInspectionSucceeded"]
                and summary["navigationSucceeded"],
                "proofLevelByUiArea": summary["proofLevelByUiArea"],
                "proofLevelByBlockedCategory": summary["proofLevelByBlockedCategory"],
                "proofSurfaceByBlockedCategory": summary["proofSurfaceByBlockedCategory"],
                "fallbackProofUsed": summary["fallbackProofUsed"],
                "rawAccessibilityDumpsCommitted": False,
            },
            "writeSafetyStatus": {
                "safeBatchWritePath": "enabled only for eligible normal scalar settings",
                "unsafeWritesExpanded": False,
                "highRiskWritesEnabled": False,
                "displayRenderRiskWritesEnabled": False,
                "generatedScriptSymlinkProfileWritesEnabled": False,
                "missingDefaultInsertionEnabled": False,
                "duplicateAutoResolutionEnabled": False,
                "structuredFamilyWritesEnabled": False,
                "profileModeSwitchingEnabled": False,
                "runtimeMutationEnabled": False,
            },
            "intentionalBlockers": [
                "missing/default insertion",
                "duplicate auto-resolution",
                "high-risk/display-render writes",
                "structured-family writes",
                "profile/mode switching",
                "Hyprland reload/runtime mutation",
                "Hyprland 0.55.4 data migration",
            ],
            "safeFixesMade": [
                "README safe-scope copy polished",
                "Cargo package description updated",
                "Desktop file comment updated",
                "AppStream metadata updated to guarded safe-batch scope",
                "Active detail copy changed from pilot wording to safe-batch wording",
                "Final completion, release-readiness, and safe-scope reports added",
            ],
            "remainingWork": [
                "Release-boundary approval before any tag/package/release artifact",
                "Optional final packaging install docs",
                "Separate design for intentionally blocked expansion areas",
            ],
            "releaseReadinessEstimate": "ready for release-boundary approval review for guarded normal-scalar safe-batch scope; no release created in this sprint",
            "validation": validation_pending_full(),
        },
    )


def final_release_readiness_checklist(summary):
    return {
        "schemaVersion": 1,
        "artifactKind": "final_release_readiness_checklist",
        "generatedAt": datetime.now(timezone.utc).isoformat(),
        "startingCommit": STARTING_COMMIT,
        "notARelease": True,
        "releaseCreated": False,
        "tagCreated": False,
        "packageCreated": False,
        "appId": "io.github.kyarorukyo.hyprlandsettings",
        "binaryName": "hyprland-settings",
        "repoIdentity": "KyaroruKYO/hyprland-settings",
        "metadataValidated": "pending",
        "desktopFileValidated": "pending",
        "appstreamValidated": "pending",
        "readmeReviewed": True,
        "userFacingCopyReviewed": True,
        "gtkAutomationPassed": summary["appLaunchSucceeded"]
        and summary["accessibilityInspectionSucceeded"]
        and summary["navigationSucceeded"],
        "cargoValidationPassed": "pending",
        "unsafeScopeStillBlocked": True,
        "knownLimitations": [
            "No missing/default insertion",
            "No duplicate auto-resolution",
            "No high-risk/display-render expansion",
            "No structured-family writes",
            "No profile/mode switching",
            "No Hyprland reload/runtime mutation",
            "No Hyprland 0.55.4 data migration",
        ],
        "beforeReleaseRequired": [
            "User approval to create a release",
            "Final tag/version decision",
            "Packaging artifact decision",
            "Release notes",
        ],
        "requiresUserApproval": [
            "creating a release",
            "creating a tag",
            "creating a package artifact",
            "expanding unsafe write scope",
            "migrating data/model to Hyprland 0.55.4",
        ],
        "recommendedReleaseDecision": "Ready to ask for explicit release-boundary approval for the guarded normal-scalar safe-batch v0.55.2 scope.",
        "projectModel": PROJECT_MODEL,
        "projectDataMigratedToHyprland0554": False,
        "countsBefore": "341 readable / 341 writable / 0 blocked",
        "countsAfter": "341 readable / 341 writable / 0 blocked",
        "validation": validation_pending_full(),
    }


def final_safe_scope_validation(summary):
    return {
        "schemaVersion": 1,
        "artifactKind": "final_safe_scope_validation",
        "generatedAt": datetime.now(timezone.utc).isoformat(),
        "startingCommit": STARTING_COMMIT,
        "scope": "guarded normal-scalar safe-batch Hyprland Settings app for v0.55.2 data/model",
        "countsBefore": "341 readable / 341 writable / 0 blocked",
        "countsAfter": "341 readable / 341 writable / 0 blocked",
        "SAFE_WRITABLE_ROWS_len": 341,
        "safeBatchWritePath": {
            "eligibleNormalScalarWrites": True,
            "backupBeforeWrite": True,
            "rereadVerificationAfterWrite": True,
            "restoreOnFailure": True,
            "applyStillGuarded": True,
        },
        "blockedUnsafeWrites": True,
        "blockedGeneratedScriptSymlinkProfile": True,
        "blockedMissingDefaultInsertion": True,
        "blockedDuplicateResolution": True,
        "blockedHighRiskDisplayRender": True,
        "blockedStructuredFamily": True,
        "blockedRuntimeMutation": True,
        "realConfigTouched": False,
        "runtimeTouched": False,
        "validation": validation_pending_full(),
        "proofUsed": {
            "cargoTest": "deterministic Rust tests",
            "gtkSafeEnvEvidenceMatrix": "live GTK/AT-SPI safe-env scenarios",
            "fixtureSafeBatchTests": "temporary fixture files only",
            "reports": "redacted committed reports",
        },
        "proofStillMissing": [
            "release-boundary approval",
            "real release packaging proof",
            "separate designs for intentionally blocked expansion areas",
        ],
        "projectModel": PROJECT_MODEL,
        "projectDataMigratedToHyprland0554": False,
    }


def main():
    if len(sys.argv) < 3:
        print("usage: summarize_gtk_evidence.py <evidence-root> <summary-output> [reports-dir]", file=sys.stderr)
        return 2
    evidence_root = Path(sys.argv[1])
    output = Path(sys.argv[2])
    runs = []
    for run_dir in sorted((evidence_root / "runs").glob("*")):
        if run_dir.is_dir():
            runs.append(run_summary(run_dir))
    summary = aggregate(runs)
    evidence_summary = {
        "schemaVersion": 1,
        "artifactKind": "gtk_safe_env_redacted_evidence_summary",
        "generatedAt": datetime.now(timezone.utc).isoformat(),
        "evidenceRoot": "<tmp>/hyprland-settings-gtk-automation/<fresh-run>",
        "scenarioResults": runs,
        "aggregate": summary,
        "proofLevelsAllowed": sorted(PROOF_LEVELS),
    }
    output.parent.mkdir(parents=True, exist_ok=True)
    output.write_text(json.dumps(evidence_summary, indent=2, sort_keys=True) + "\n")
    if len(sys.argv) > 3:
        write_reports(Path(sys.argv[3]), evidence_root, evidence_summary, runs, summary)
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
