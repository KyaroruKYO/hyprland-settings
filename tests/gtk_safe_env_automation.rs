use std::fs;
use std::process::Command;

use hyprland_settings::write_classification::SAFE_WRITABLE_ROWS;
use serde_json::Value;

const OLD_EVIDENCE_SUFFIXES: [&str; 5] = ["205721", "205954", "205927", "210021", "210114"];

#[test]
fn gtk_automation_scripts_exist_and_are_safe_env_by_default() {
    let scripts = [
        "tools/live_scenario_harness/gtk_automation_probe.sh",
        "tools/live_scenario_harness/run_gtk_safe_env_scenario.sh",
        "tools/live_scenario_harness/run_gtk_evidence_matrix.sh",
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
    assert!(launcher.contains("cargo build --quiet"));
    assert!(launcher.contains("appBuildAttempted"));
    assert!(launcher.contains("appBuildSucceeded"));
    assert!(launcher.contains("appBinaryRebuiltBeforeProbe"));
    assert!(launcher.contains("app_launch_attempted=\"false\""));
    assert!(launcher.contains("exit 0"));
    assert!(!launcher.contains("if [ ! -x target/debug/hyprland-settings ]"));
    assert!(launcher.contains("HOME=\"$scenario_home\""));
    assert!(launcher.contains("XDG_CONFIG_HOME=\"$scenario_home/.config\""));
    assert!(launcher.contains("timeout \"$timeout_seconds\" target/debug/hyprland-settings"));
    assert!(launcher.contains("\"liveSwapModeUsed\": false"));
    assert!(launcher.contains("\"applyClicked\": false"));
    assert!(launcher.contains("\"realConfigEdited\": false"));
    assert!(launcher.contains("\"realBackupsCreated\": false"));
    assert!(!launcher.contains("hyprctl reload"));
    assert!(!launcher.contains("pkill -x ags"));
    assert!(!launcher.contains("pkill -x waybar"));
}

#[test]
fn gtk_automation_python_collectors_are_safe_and_compilable() {
    for script in [
        "tools/live_scenario_harness/collect_accessibility_tree.py",
        "tools/live_scenario_harness/summarize_gtk_evidence.py",
    ] {
        assert!(fs::metadata(script).is_ok(), "{script} should exist");
        let status = Command::new("python3")
            .arg("-m")
            .arg("py_compile")
            .arg(script)
            .status()
            .expect("python py_compile should run");
        assert!(status.success(), "{script} should compile");
    }

    let collector = fs::read_to_string("tools/live_scenario_harness/collect_accessibility_tree.py")
        .expect("collector should read");
    assert!(collector.contains("pyatspi"));
    assert!(collector.contains("SAFE_NAVIGATION_TARGETS"));
    assert!(collector.contains("\"FirstBlockedSettingRow\""));
    assert!(collector.contains("\"DuplicateConflictDetail\""));
    assert!(collector.contains("duplicateBlockedReasonTextCollected"));
    assert!(collector.contains("refused to navigate to Apply"));
    assert!(collector.contains("refused to click node containing Apply"));
    assert!(collector.contains("\"applyRefused\""));
    assert!(!collector.contains("ydotool"));
    assert!(!collector.contains("wtype"));
}

#[test]
fn gtk_reports_are_evidence_derived_and_preserve_project_model() {
    let report_paths = [
        "data/reports/gtk-safe-env-automation-capability.v0.55.2.json",
        "data/reports/gtk-safe-env-scenario-matrix.v0.55.2.json",
        "data/reports/gtk-safe-env-ui-navigation.v0.55.2.json",
        "data/reports/gtk-safe-env-user-experience.v0.55.2.json",
        "data/reports/gtk-safe-env-automation-summary.v0.55.2.json",
        "data/reports/gtk-safe-env-evidence-derived-matrix.v0.55.2.json",
    ];
    for path in report_paths {
        let text = fs::read_to_string(path).unwrap_or_else(|error| panic!("{path}: {error}"));
        for suffix in OLD_EVIDENCE_SUFFIXES {
            let old_path = format!("/tmp/hyprland-settings-gtk-automation/20260618_{suffix}");
            assert!(
                !text.contains(&old_path),
                "{path} must not contain old hard-coded evidence path {old_path}"
            );
        }
        let value: Value = serde_json::from_str(&text).expect("report json should parse");
        assert_eq!(value["schemaVersion"], 1);
        assert_eq!(
            value["projectModel"],
            "v0.55.2 / 341 readable / 341 writable / 0 blocked"
        );
        assert_eq!(value["projectDataMigratedToHyprland0554"], false);
        assert_eq!(value["safeEnvModeUsed"], true);
        assert_eq!(value["liveSwapModeUsed"], false);
        assert_eq!(value["agsTouched"], false);
        assert_eq!(value["waybarTouched"], false);
        assert_eq!(value["realConfigEdited"], false);
        assert_eq!(value["realBackupsCreated"], false);
        assert_eq!(value["hyprlandReloaded"], false);
        assert_eq!(value["mutatingHyprctlUsed"], false);
        assert_eq!(value["runtimeMutated"], false);
        assert_eq!(value["scriptsExecuted"], false);
        assert_eq!(value["luaExecuted"], false);
        assert_eq!(value["screenshotsCommitted"], false);
        assert_eq!(
            value["evidenceSummarySource"],
            "tools/live_scenario_harness/summarize_gtk_evidence.py"
        );
        assert_eq!(value["appBuildAttempted"], true);
        assert_eq!(value["appBuildSucceeded"], true);
        assert_eq!(value["appBinaryRebuiltBeforeProbe"], true);
        assert!(value["duplicateConflictDetailNavigationAttempted"].is_boolean());
        assert!(value["duplicateConflictDetailNavigationSucceeded"].is_boolean());
        assert!(value["duplicateBlockedReasonTextCollected"].is_boolean());
        assert!(value["proofLevelByUiArea"].is_object());
    }
}

#[test]
fn gtk_reports_use_explicit_proof_level_labels() {
    let summary: Value = serde_json::from_str(
        &fs::read_to_string("data/reports/gtk-safe-env-automation-summary.v0.55.2.json")
            .expect("summary report should read"),
    )
    .expect("summary report should parse");
    let proof = summary["proofLevelByUiArea"]
        .as_object()
        .expect("proofLevelByUiArea should be an object");
    for required in [
        "Dashboard",
        "Config",
        "Appearance",
        "Display",
        "Search",
        "settingRow",
        "detailPane",
        "blockedReason",
        "duplicateBlockedCopy",
        "generatedScriptSymlinkBlockedCopy",
        "highRiskDisplayRisk",
    ] {
        let level = proof
            .get(required)
            .and_then(Value::as_str)
            .unwrap_or_else(|| panic!("{required} proof level missing"));
        assert!(
            matches!(
                level,
                "live_gtk_atspi_proof"
                    | "safe_env_model_proof"
                    | "source_model_fallback"
                    | "not_proven"
            ),
            "{required} has unexpected proof level {level}"
        );
    }
}

#[test]
fn duplicate_blocked_copy_is_not_overclaimed_without_evidence() {
    let summary: Value = serde_json::from_str(
        &fs::read_to_string("data/reports/gtk-safe-env-automation-summary.v0.55.2.json")
            .expect("summary report should read"),
    )
    .expect("summary report should parse");
    let duplicate_level = summary["proofLevelByUiArea"]["duplicateBlockedCopy"]
        .as_str()
        .expect("duplicate proof level should be a string");
    let duplicate_text_collected = summary["duplicateBlockedReasonTextCollected"]
        .as_bool()
        .expect("duplicateBlockedReasonTextCollected should be a boolean");
    if duplicate_text_collected {
        assert_eq!(duplicate_level, "live_gtk_atspi_proof");
    } else {
        assert_ne!(duplicate_level, "live_gtk_atspi_proof");
    }
}

#[test]
fn gtk_automation_preserves_safe_writable_row_count() {
    assert_eq!(SAFE_WRITABLE_ROWS.len(), 341);
}
