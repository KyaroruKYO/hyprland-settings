#![recursion_limit = "256"]

mod support;

use std::fs;
use std::process::Command;

use serde_json::json;
use support::safe_batch_harness::write_report;

const STARTING_COMMIT: &str = "c8143b7e8b7abb3499f60d8e08b557257003f9b4";

#[test]
fn gtk_automation_scripts_exist_and_are_safe_env_by_default() {
    let scripts = [
        "tools/live_scenario_harness/gtk_automation_probe.sh",
        "tools/live_scenario_harness/run_gtk_safe_env_scenario.sh",
        "tools/live_scenario_harness/close_app_window.sh",
    ];
    for script in scripts {
        assert!(fs::metadata(script).is_ok(), "{script} should exist");
        let status = Command::new("bash")
            .arg("-n")
            .arg(script)
            .status()
            .expect("bash -n should run");
        assert!(status.success(), "{script} should pass bash -n");
    }

    let launcher = fs::read_to_string("tools/live_scenario_harness/gtk_automation_probe.sh")
        .expect("probe should read");
    assert!(launcher.contains("HOME=\"$scenario_home\""));
    assert!(launcher.contains("XDG_CONFIG_HOME=\"$scenario_home/.config\""));
    assert!(launcher.contains("timeout \"$timeout_seconds\" target/debug/hyprland-settings"));
    assert!(launcher.contains("\"liveSwapModeUsed\": false"));
    assert!(launcher.contains("\"applyClicked\": false"));
    assert!(launcher.contains("\"realConfigEdited\": false"));
    assert!(!launcher.contains("hyprctl reload"));
    assert!(!launcher.contains("pkill -x ags"));
    assert!(!launcher.contains("pkill -x waybar"));
}

#[test]
fn gtk_automation_python_collector_exists_and_reports_capability() {
    let collector = "tools/live_scenario_harness/collect_accessibility_tree.py";
    assert!(fs::metadata(collector).is_ok());
    let status = Command::new("python3")
        .arg("-m")
        .arg("py_compile")
        .arg(collector)
        .status()
        .expect("python py_compile should run");
    assert!(status.success());

    let source = fs::read_to_string(collector).expect("collector should read");
    assert!(source.contains("pyatspi"));
    assert!(source.contains("org.a11y.Bus"));
    assert!(source.contains("\"succeeded\""));
}

#[test]
fn gtk_safe_env_reports_are_generated_with_safe_defaults() {
    let capability = json!({
        "schemaVersion": 1,
        "artifactKind": "gtk_safe_env_automation_capability",
        "generatedAt": "2026-06-18T21:15:00-07:00",
        "startingCommit": STARTING_COMMIT,
        "projectModel": "v0.55.2 / 341 readable / 341 writable / 0 blocked",
        "projectDataMigratedToHyprland0554": false,
        "pyatspiAvailable": python_pyatspi_available(),
        "runningHyprlandVersion": "0.55.4",
        "installedHyprlandPackageVersion": "0.55.4-1",
        "rustPackageVersion": "1:1.96.0-1",
        "pythonAtspiPackageVersion": "2.58.2-1",
        "gtk4PackageVersion": "1:4.22.4-1",
        "libadwaitaPackageVersion": "1:1.9.1-1",
        "cargoCheckAfterSystemUpgrade": "passed",
        "safeEnvModeUsed": true,
        "liveSwapModeUsed": false,
        "agsTouched": false,
        "waybarTouched": false,
        "realConfigEdited": false,
        "realBackupsCreated": false,
        "hyprlandReloaded": false,
        "mutatingHyprctlUsed": false,
        "runtimeMutated": false,
        "scriptsExecuted": false,
        "luaExecuted": false,
        "screenshotsCommitted": false,
        "availableTools": {
            "gdbus": command_available("gdbus"),
            "busctl": command_available("busctl"),
            "python": command_available("python"),
            "python3": command_available("python3"),
            "ydotool": command_available("ydotool"),
            "wtype": command_available("wtype"),
            "grim": command_available("grim"),
            "jq": command_available("jq")
        },
        "appLaunchAttempted": true,
        "appLaunchSucceeded": true,
        "accessibilityInspectionAttempted": true,
        "accessibilityInspectionSucceeded": true,
        "uiTextTreeCollected": true,
        "navigationAttempted": true,
        "navigationSucceeded": true,
        "closeAttempted": true,
        "closeSucceeded": true,
        "fallbackProofUsed": true,
        "evidence": {
            "minimalInitial": "/tmp/hyprland-settings-gtk-automation/20260618_205721",
            "configNavigation": "/tmp/hyprland-settings-gtk-automation/20260618_205954",
            "appearanceNavigation": "/tmp/hyprland-settings-gtk-automation/20260618_205927",
            "duplicateAppearanceNavigation": "/tmp/hyprland-settings-gtk-automation/20260618_210021",
            "displayNavigation": "/tmp/hyprland-settings-gtk-automation/20260618_210114"
        },
        "issuesFound": [
            "Live GTK probe is run outside cargo test so the full test suite remains deterministic.",
            "Search and deeper detail-pane blocker explanations were not fully navigated through AT-SPI in this sprint.",
            "Duplicate-specific blocked copy was not exposed in the row-list accessibility text; source/model proof remains the fallback."
        ],
        "recommendedFixes": [
            "Add stable accessibility names for search, detail panes, and blocked-reason expanders.",
            "Add a non-Apply detail-pane action target to collect duplicate/missing/generated/symlink blocker copy through AT-SPI."
        ]
    });

    let scenario_matrix = json!({
        "schemaVersion": 1,
        "artifactKind": "gtk_safe_env_scenario_matrix",
        "generatedAt": "2026-06-18T21:15:00-07:00",
        "startingCommit": STARTING_COMMIT,
        "projectModel": "v0.55.2 / 341 readable / 341 writable / 0 blocked",
        "safeEnvModeUsed": true,
        "liveSwapModeUsed": false,
        "scenarioResults": {
            "minimal_single_config": {
                "gtkLaunch": "passed",
                "accessibilityTree": "passed",
                "dashboardText": "collected",
                "configNavigation": "passed",
                "appearanceNavigation": "passed",
                "close": "passed"
            },
            "large_single_config": {
                "gtkLaunch": "not rerun through GTK in this sprint",
                "fallbackProof": "covered by previous safe-env harness/source model"
            },
            "source_include_config": {
                "gtkLaunch": "not rerun through GTK in this sprint",
                "fallbackProof": "covered by previous safe-env harness/source-aware mapping tests"
            },
            "nested_source_config": {
                "gtkLaunch": "not rerun through GTK in this sprint",
                "fallbackProof": "covered by previous safe-env harness/source-aware mapping tests"
            },
            "symlink_current_profile": {
                "gtkLaunch": "not rerun through GTK in this sprint",
                "fallbackProof": "covered by previous safe-env harness/source model"
            },
            "duplicate_conflict": {
                "gtkLaunch": "passed",
                "accessibilityTree": "passed",
                "appearanceNavigation": "passed",
                "duplicateSpecificText": "not exposed in row-list AT-SPI text"
            },
            "generated_config": {
                "gtkLaunch": "not rerun through GTK in this sprint",
                "fallbackProof": "covered by previous safe-env harness/source model"
            },
            "script_managed_config": {
                "gtkLaunch": "not rerun through GTK in this sprint",
                "fallbackProof": "covered by previous safe-env harness/source model"
            },
            "missing_default_only": {
                "gtkLaunch": "represented by minimal config Appearance navigation",
                "defaultText": "collected"
            },
            "high_risk_display_risk": {
                "gtkLaunch": "passed",
                "accessibilityTree": "passed",
                "displayNavigation": "passed",
                "riskText": "Extra care needed collected"
            },
            "real_current_config_readonly": {
                "gtkLaunch": "not launched against real config in this sprint",
                "safety": "real config not mutated"
            }
        },
        "appLaunchAttempted": true,
        "appLaunchSucceeded": true,
        "accessibilityInspectionAttempted": true,
        "accessibilityInspectionSucceeded": true,
        "closeAttempted": true,
        "closeSucceeded": true,
        "applyClicked": false,
        "realConfigEdited": false,
        "hyprlandReloaded": false,
        "mutatingHyprctlUsed": false
    });

    let navigation = json!({
        "schemaVersion": 1,
        "artifactKind": "gtk_safe_env_ui_navigation",
        "generatedAt": "2026-06-18T21:15:00-07:00",
        "startingCommit": STARTING_COMMIT,
        "projectModel": "v0.55.2 / 341 readable / 341 writable / 0 blocked",
        "safeEnvModeUsed": true,
        "liveSwapModeUsed": false,
        "navigationAttempted": true,
        "navigationSucceeded": true,
        "dashboard": "collected from initial AT-SPI tree",
        "configPage": "collected after Config navigation",
        "categoryPage": {
            "appearance": "collected after Appearance navigation",
            "display": "collected after Display navigation"
        },
        "search": "not proven through AT-SPI navigation in this sprint",
        "detailPane": "placeholder text collected; specific row detail not opened through AT-SPI",
        "safeBatchCopy": [
            "Config page AT-SPI text collected backup-before-save copy.",
            "Full safe-batch wording remains source/model proof."
        ],
        "blockedCopy": [
            "Uses Hyprland default was collected from Appearance and Display AT-SPI text.",
            "Extra care needed was collected from Display AT-SPI text.",
            "Duplicate/generated/script/symlink-specific detail copy remains source/model fallback because row details were not opened."
        ],
        "collectedTerms": {
            "initial": ["appearance", "config", "dashboard", "display", "hyprland", "settings"],
            "configNavigation": ["appearance", "apply", "config", "dashboard", "display", "hyprland", "settings"],
            "appearanceNavigation": ["appearance", "config", "dashboard", "default", "display", "hyprland", "settings"],
            "displayNavigation": ["appearance", "config", "dashboard", "default", "display", "hyprland", "settings"]
        },
        "applyAvoided": true
    });

    let ux = json!({
        "schemaVersion": 1,
        "artifactKind": "gtk_safe_env_user_experience",
        "generatedAt": "2026-06-18T21:15:00-07:00",
        "startingCommit": STARTING_COMMIT,
        "projectModel": "v0.55.2 / 341 readable / 341 writable / 0 blocked",
        "newHyprlandUser": "GTK proof: simple config launches, Dashboard/Config/Appearance are discoverable, and default rows show Uses Hyprland default.",
        "sourcedConfigUser": "source/model fallback: connected files map exact source targets; GTK source/include scenario was not rerun in this sprint.",
        "profileSymlinkUser": "source/model fallback: symlink/current-profile writes stay blocked; GTK symlink scenario was not rerun in this sprint.",
        "generatedScriptManagedConfigUser": "source/model fallback: generated and script-managed configs stay blocked; GTK generated/script scenario was not rerun in this sprint.",
        "currentUser": "read-only audit remains source-aware and blocked where unsafe; real config was not launched or mutated in this sprint.",
        "safeBatchWordingUnderstandable": true,
        "blockedReasonsUnderstandable": "partially proven through AT-SPI for default/display-risk copy; deeper blocked reasons need stable detail-pane automation",
        "oneTargetWordingVisibleInSafeBatchPath": false,
        "appCloseCleanly": true,
        "applyAvoided": true
    });

    let summary = json!({
        "schemaVersion": 1,
        "artifactKind": "gtk_safe_env_automation_summary",
        "generatedAt": "2026-06-18T21:15:00-07:00",
        "startingCommit": STARTING_COMMIT,
        "projectModel": "v0.55.2 / 341 readable / 341 writable / 0 blocked",
        "projectDataMigratedToHyprland0554": false,
        "pyatspiAvailable": python_pyatspi_available(),
        "runningHyprlandVersion": "0.55.4",
        "installedHyprlandPackageVersion": "0.55.4-1",
        "cargoCheckAfterSystemUpgrade": "passed",
        "safeEnvModeUsed": true,
        "liveSwapModeUsed": false,
        "agsTouched": false,
        "waybarTouched": false,
        "realConfigEdited": false,
        "realBackupsCreated": false,
        "hyprlandReloaded": false,
        "mutatingHyprctlUsed": false,
        "runtimeMutated": false,
        "scriptsExecuted": false,
        "luaExecuted": false,
        "screenshotsCommitted": false,
        "appLaunchAttempted": true,
        "appLaunchSucceeded": true,
        "accessibilityInspectionAttempted": true,
        "accessibilityInspectionSucceeded": true,
        "uiTextTreeCollected": true,
        "navigationAttempted": true,
        "navigationSucceeded": true,
        "closeAttempted": true,
        "closeSucceeded": true,
        "fallbackProofUsed": true,
        "scenarioResults": scenario_matrix["scenarioResults"],
        "uiEvidence": {
            "dashboard": "collected",
            "configPage": "collected",
            "appearanceCategory": "collected",
            "displayCategory": "collected",
            "search": "not proven through AT-SPI navigation",
            "detailPane": "placeholder collected, row-specific details not opened",
            "safeBatchCopy": "partial AT-SPI plus source/model proof",
            "blockedCopy": "default/display-risk partial AT-SPI plus source/model proof",
            "duplicateConflictCopy": "source/model fallback",
            "generatedScriptSymlinkCopy": "source/model fallback"
        },
        "issuesFound": [
            "Search and row-specific detail navigation need stable AT-SPI names/actions.",
            "Duplicate/generated/script/symlink blocked copy was not collected from live UI detail panes in this sprint."
        ],
        "recommendedFixes": [
            "Add accessible labels/actions for search, setting rows, and blocked-reason detail panes.",
            "Extend collect_accessibility_tree.py with safe row-detail navigation that explicitly refuses Apply."
        ],
        "countsBefore": "341 readable / 341 writable / 0 blocked",
        "countsAfter": "341 readable / 341 writable / 0 blocked",
        "validation": {
            "bashScripts": "pending",
            "cargoFmt": "pending",
            "cargoFmtCheck": "pending",
            "cargoCheck": "pending",
            "cargoTest": "pending",
            "cargoBuildRelease": "pending",
            "jqReports": "pending",
            "gitDiffCheck": "pending",
            "gitStatusShort": "pending"
        }
    });

    write_report(
        "gtk-safe-env-automation-capability.v0.55.2.json",
        &capability,
    );
    write_report(
        "gtk-safe-env-scenario-matrix.v0.55.2.json",
        &scenario_matrix,
    );
    write_report("gtk-safe-env-ui-navigation.v0.55.2.json", &navigation);
    write_report("gtk-safe-env-user-experience.v0.55.2.json", &ux);
    write_report("gtk-safe-env-automation-summary.v0.55.2.json", &summary);
}

fn command_available(command: &str) -> bool {
    Command::new("sh")
        .arg("-c")
        .arg(format!("command -v {command} >/dev/null 2>&1"))
        .status()
        .map(|status| status.success())
        .unwrap_or(false)
}

fn python_pyatspi_available() -> bool {
    Command::new("python3")
        .arg("-c")
        .arg("import pyatspi")
        .status()
        .map(|status| status.success())
        .unwrap_or(false)
}
