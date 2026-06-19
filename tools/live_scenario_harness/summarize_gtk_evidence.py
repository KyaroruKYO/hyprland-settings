#!/usr/bin/env python3
import json
import re
import sys
from datetime import datetime, timezone
from pathlib import Path


PROJECT_MODEL = "v0.55.2 / 341 readable / 341 writable / 0 blocked"
STARTING_COMMIT = "2338d9e1328320e60413e8e2625bd231d59cc4b3"
PROOF_LEVELS = {
    "live_gtk_atspi_proof",
    "safe_env_model_proof",
    "source_model_fallback",
    "not_proven",
}
OLD_EVIDENCE_RE = re.compile(r"/tmp/hyprland-settings-gtk-automation/20260618_[0-9]+")


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
            "pyatspiAvailable": bool(accessibility.get("pyatspiAvailable")),
        },
        "textSample": redact((accessibility.get("textAfterNavigation") or accessibility.get("text") or [])[:30]),
    }


def aggregate(runs):
    def any_run(predicate):
        return any(predicate(run) for run in runs)

    by_area = {
        "Dashboard": proof(any_run(lambda run: "dashboard" in all_terms(run))),
        "Config": proof(any_run(lambda run: "config" in all_terms(run))),
        "Appearance": proof(any_run(lambda run: "appearance" in all_terms(run))),
        "Display": proof(any_run(lambda run: "display" in all_terms(run))),
        "Search": proof(any_run(lambda run: "search" in all_terms(run))),
        "settingRow": proof(any_run(lambda run: "FirstBlockedSettingRow" in run["name"] and run["accessibility"]["navigationAttempted"]), fallback=True),
        "detailPane": proof(any_run(lambda run: run["accessibility"]["detailPaneTextCollected"])),
        "blockedReason": proof(any_run(lambda run: run["accessibility"]["blockedReasonTextCollected"])),
        "duplicateBlockedCopy": proof(any_run(lambda run: run["scenario"] == "duplicate_conflict" and "duplicate" in all_terms(run)), fallback=True),
        "generatedScriptSymlinkBlockedCopy": proof(any_run(lambda run: run["scenario"] in {"generated_config", "script_managed_config", "symlink_current_profile"} and any(term in all_terms(run) for term in ["generated", "script", "symlink"])), fallback=True),
        "highRiskDisplayRisk": proof(any_run(lambda run: run["scenario"] == "high_risk_display_risk" and ("display" in all_terms(run) or "high-risk" in all_terms(run))), fallback=True),
    }

    return {
        "appLaunchAttempted": bool(runs),
        "appLaunchSucceeded": any_run(lambda run: run["probe"]["appLaunchSucceeded"]),
        "appBinaryRebuiltBeforeProbe": all(run["probe"]["appBinaryRebuiltBeforeProbe"] for run in runs) if runs else False,
        "accessibilityInspectionAttempted": any_run(lambda run: run["accessibility"]["attempted"]),
        "accessibilityInspectionSucceeded": any_run(lambda run: run["accessibility"]["succeeded"]),
        "uiTextTreeCollected": any_run(lambda run: bool(run["textSample"])),
        "navigationAttempted": any_run(lambda run: run["accessibility"]["navigationAttempted"]),
        "navigationSucceeded": any_run(lambda run: run["accessibility"]["navigationSucceeded"]),
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
        "fallbackProofUsed": any(level in {"source_model_fallback", "not_proven"} for level in by_area.values()),
    }


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
        "appLaunchAttempted": summary["appLaunchAttempted"],
        "appLaunchSucceeded": summary["appLaunchSucceeded"],
        "appBinaryRebuiltBeforeProbe": summary["appBinaryRebuiltBeforeProbe"],
        "accessibilityInspectionAttempted": summary["accessibilityInspectionAttempted"],
        "accessibilityInspectionSucceeded": summary["accessibilityInspectionSucceeded"],
        "uiTextTreeCollected": summary["uiTextTreeCollected"],
        "navigationAttempted": summary["navigationAttempted"],
        "navigationSucceeded": summary["navigationSucceeded"],
        "closeAttempted": bool(runs),
        "closeSucceeded": summary["closeSucceeded"],
        "scenarioResults": runs,
        "proofLevelByScenario": {
            run["name"]: "live_gtk_atspi_proof" if run["accessibility"]["succeeded"] else "not_proven"
            for run in runs
        },
        "proofLevelByUiArea": summary["proofLevelByUiArea"],
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
    for key in ["duplicateBlockedCopy", "generatedScriptSymlinkBlockedCopy"]:
        if summary["proofLevelByUiArea"].get(key) != "live_gtk_atspi_proof":
            found.append(f"{key} still uses fallback proof or remains incomplete.")
    return found


def recommended_fixes(summary):
    fixes = []
    if summary["proofLevelByUiArea"].get("settingRow") != "live_gtk_atspi_proof":
        fixes.append("Add a stronger non-Apply row activation action or widget role for setting rows.")
    if summary["proofLevelByUiArea"].get("detailPane") != "live_gtk_atspi_proof":
        fixes.append("Expose selected row detail pane text more consistently through AT-SPI.")
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
    }
    for name, data in reports.items():
        (reports_dir / name).write_text(json.dumps(data, indent=2, sort_keys=True) + "\n")


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
