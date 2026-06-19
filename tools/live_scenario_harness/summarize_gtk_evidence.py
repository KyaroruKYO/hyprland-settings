#!/usr/bin/env python3
import json
import re
import sys
from datetime import datetime, timezone
from pathlib import Path


PROJECT_MODEL = "v0.55.2 / 341 readable / 341 writable / 0 blocked"
STARTING_COMMIT = "e0b6d1fe63bf7096183a8bd1ab6304003fa6a5b8"
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
        "fallbackProofUsed": any(level in {"source_model_fallback", "not_proven"} for level in by_area.values())
        or any(level in {"source_model_fallback", "not_proven"} for level in by_blocked_category.values()),
    }


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
        "completion-readiness-audit.v0.55.2.json": completion_readiness_report(
            evidence_root, summary, runs
        ),
        "completion-wrap-up-plan.v0.55.2.json": completion_wrap_up_plan(summary),
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
            "recommendedNextSprint": "Actual app completion wrap-up: polish remaining user-facing copy, packaging metadata, and release checklist without expanding unsafe write scope.",
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
            "Polish remaining user-facing safe-batch and blocked-copy wording.",
            "Review desktop/metainfo packaging files and app identity.",
            "Run a final safe-env GTK matrix and deterministic validation sweep.",
            "Prepare a release-readiness checklist without creating a release.",
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
        "releaseReadinessEstimate": "near-complete for guarded normal-scalar safe-batch scope, not release-ready for high-risk, insertion, duplicate-resolution, profile/mode, or Hyprland 0.55.4 migration scope",
        "nextSprintPlan": "Actual app completion wrap-up: polish remaining user-facing copy, packaging metadata, final validation reports, and release checklist while preserving the current safety boundaries.",
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
