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
    for target in [
        "MissingDefaultDetail",
        "GeneratedBlockedDetail",
        "ScriptManagedBlockedDetail",
        "SymlinkManagedBlockedDetail",
        "GeneratedConnectedFileDetail",
        "ScriptManagedConnectedFileDetail",
        "SymlinkConnectedFileDetail",
        "ProfileModeDetail",
        "HighRiskDetail",
        "DisplayRenderRiskDetail",
        "ProfileModeSwitchDetail",
    ] {
        assert!(
            collector.contains(&format!("\"{target}\"")),
            "collector should allowlist {target}"
        );
    }
    assert!(collector.contains("duplicateBlockedReasonTextCollected"));
    assert!(collector.contains("blockedCategoryDetailNavigationAttempted"));
    assert!(collector.contains("blockedCategoryDetailNavigationSucceeded"));
    assert!(collector.contains("blockedCategoryReasonTextCollected"));
    assert!(collector.contains("blockedCategoryExpectedTextCollected"));
    assert!(collector.contains("connectedFileDetailNavigationAttempted"));
    assert!(collector.contains("connectedFileDetailNavigationSucceeded"));
    assert!(collector.contains("connectedFileGeneratedDetailCollected"));
    assert!(collector.contains("connectedFileScriptManagedDetailCollected"));
    assert!(collector.contains("connectedFileSymlinkDetailCollected"));
    assert!(collector.contains("profileModeDetailCollected"));
    assert!(collector.contains("APPROVAL_CARD_ASSERTIONS"));
    assert!(collector.contains("approvalCardAssertionMethod"));
    assert!(collector.contains("approvalCardAssertions"));
    assert!(collector.contains("ACTIVATION_DECISION_ASSERTIONS"));
    assert!(collector.contains("activationDecisionAssertionMethod"));
    assert!(collector.contains("activationDecisionAssertions"));
    assert!(collector.contains("ACTIVATION_PATH_ASSERTIONS"));
    assert!(collector.contains("activationPathAssertionMethod"));
    assert!(collector.contains("activationPathAssertions"));
    assert!(collector.contains("ACTIVATION_CONTROL_ASSERTIONS"));
    assert!(collector.contains("activationControlAssertions"));
    assert!(collector.contains("activationControlsAllExecutorUnwiredFound"));
    assert!(collector.contains("ACTIVATION_FORM_ASSERTIONS"));
    assert!(collector.contains("activationFormAssertions"));
    assert!(collector.contains("activationFormsAllExecutorUnwiredFound"));
    for expected in [
        "Source/include approval review",
        "Duplicate approval review",
        "Source/include production activation decision",
        "Duplicate production activation decision",
        "Source/include production activation path",
        "Duplicate production activation path",
        "Source/include production activation control",
        "Duplicate production activation control",
        "Source/include activation request form",
        "Duplicate activation request form",
        "Executor wiring: Unwired",
        "Structured hl.bind approval review",
        "Profile/mode approval review",
        "High-risk/display approval review",
        "Hyprland 0.55.4 migration review",
        "Production source/include insertion",
        "Production duplicate writes",
        "Production structured writes",
        "Production profile switching",
        "Production high-risk/display writes",
        "Production migration activation",
    ] {
        assert!(
            collector.contains(expected),
            "collector should assert approval-card text: {expected}"
        );
    }
    assert!(collector.contains("\"proofSurface\""));
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
        "data/reports/gtk-safe-env-blocked-category-detail-proof.v0.55.2.json",
        "data/reports/gtk-safe-env-connected-file-detail-proof.v0.55.2.json",
        "data/reports/gtk-safe-env-disabled-approval-card-proof.v0.55.2.json",
        "data/reports/completion-readiness-audit.v0.55.2.json",
        "data/reports/final-app-completion-wrap-up.v0.55.2.json",
        "data/reports/autonomous-safe-scope-continuation.v0.55.2.json",
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
        assert!(value["blockedCategoryDetailNavigationAttempted"].is_boolean());
        assert!(value["blockedCategoryDetailNavigationSucceeded"].is_boolean());
        assert!(value["blockedCategoryReasonTextCollected"].is_boolean());
        assert!(value["proofLevelByUiArea"].is_object());
        assert!(value["proofLevelByBlockedCategory"].is_object());
        assert!(value["proofSurfaceByBlockedCategory"].is_object());
        assert!(value["blockedCategoryResults"].is_object());
    }
}

#[test]
fn gtk_harness_records_screenshot_level_disabled_approval_card_assertions() {
    let summarizer = fs::read_to_string("tools/live_scenario_harness/summarize_gtk_evidence.py")
        .expect("summarizer should read");
    for expected in [
        "approval_card_assertion_results",
        "gtk-safe-env-disabled-approval-card-proof.v0.55.2.json",
        "screenshot_plus_accessibility_tree_text_not_ocr",
        "approvalCardsAllHeadingsFound",
        "approvalCardsAllProductionDisabledFound",
        "approvalCardsAllDisabledActionsFound",
        "activation_decision_assertion_results",
        "activationDecisionsAllHeadingsFound",
        "activationDecisionsAllProductionDisabledFound",
        "activationDecisionsAllDisabledActionsFound",
        "activation_path_assertion_results",
        "activationPathsAllHeadingsFound",
        "activationPathsAllProductionDisabledFound",
        "activationPathsAllDisabledActionsFound",
        "activation_control_assertion_results",
        "activationControlsAllHeadingsFound",
        "activationControlsAllProductionDisabledFound",
        "activationControlsAllExecutorUnwiredFound",
        "activationControlsAllDisabledActionsFound",
        "activation_form_assertion_results",
        "activationFormsAllHeadingsFound",
        "activationFormsAllProductionDisabledFound",
        "activationFormsAllExecutorUnwiredFound",
        "activationFormsAllDisabledActionsFound",
        "sourceIncludeInsertion",
        "duplicateReplacement",
        "structuredHlBindWrite",
        "profileModeSwitch",
        "highRiskDisplayWrite",
        "hyprland0554Migration",
    ] {
        assert!(
            summarizer.contains(expected),
            "summarizer should preserve disabled approval card assertion evidence: {expected}"
        );
    }
}

#[test]
fn gtk_evidence_matrix_covers_source_include_and_nested_source_scenarios() {
    let matrix = fs::read_to_string("tools/live_scenario_harness/run_gtk_evidence_matrix.sh")
        .expect("matrix script should read");
    assert!(matrix.contains("write_conf source_include_config"));
    assert!(matrix.contains("write_conf nested_source_config"));
    assert!(matrix.contains("source = profiles/current.conf"));
    assert!(matrix.contains("source = ../appearance/theme.conf"));
    assert!(matrix.contains("run_probe source_include_config Config"));
    assert!(matrix.contains("run_probe nested_source_config Config"));
    assert!(!matrix.contains("live-swap"));
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
        "defaultMissingBlockedCopy",
        "duplicateBlockedCopy",
        "generatedBlockedCopy",
        "scriptManagedBlockedCopy",
        "symlinkCurrentProfileBlockedCopy",
        "highRiskBlockedCopy",
        "displayRenderRiskBlockedCopy",
        "profileModeSwitchBlockedCopy",
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

    let blocked = summary["proofLevelByBlockedCategory"]
        .as_object()
        .expect("proofLevelByBlockedCategory should be an object");
    for required in [
        "defaultMissingBlockedCopy",
        "duplicateBlockedCopy",
        "generatedBlockedCopy",
        "scriptManagedBlockedCopy",
        "symlinkCurrentProfileBlockedCopy",
        "highRiskBlockedCopy",
        "displayRenderRiskBlockedCopy",
        "profileModeSwitchBlockedCopy",
    ] {
        let level = blocked
            .get(required)
            .and_then(Value::as_str)
            .unwrap_or_else(|| panic!("{required} blocked category proof level missing"));
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

    let surfaces = summary["proofSurfaceByBlockedCategory"]
        .as_object()
        .expect("proofSurfaceByBlockedCategory should be an object");
    for (key, expected) in [
        ("generatedBlockedCopy", "connected_file_detail"),
        ("scriptManagedBlockedCopy", "connected_file_detail"),
        ("symlinkCurrentProfileBlockedCopy", "connected_file_detail"),
        ("profileModeSwitchBlockedCopy", "profile_detail"),
    ] {
        let surface = surfaces
            .get(key)
            .and_then(Value::as_str)
            .unwrap_or_else(|| panic!("{key} proof surface missing"));
        assert!(
            matches!(
                surface,
                "connected_file_detail"
                    | "profile_detail"
                    | "setting_row_detail"
                    | "config_page_text"
                    | "source_model_fallback"
                    | "not_proven"
            ),
            "{key} has unexpected proof surface {surface}"
        );
        if summary["proofLevelByBlockedCategory"][key] == "live_gtk_atspi_proof" {
            assert_eq!(
                surface, expected,
                "{key} should use the dedicated detail surface"
            );
        }
    }
}

#[test]
fn blocked_category_copy_is_not_overclaimed_without_evidence() {
    let summary: Value = serde_json::from_str(
        &fs::read_to_string("data/reports/gtk-safe-env-automation-summary.v0.55.2.json")
            .expect("summary report should read"),
    )
    .expect("summary report should parse");
    let blocked_levels = summary["proofLevelByBlockedCategory"]
        .as_object()
        .expect("proofLevelByBlockedCategory should be an object");
    let blocked_results = summary["blockedCategoryResults"]
        .as_object()
        .expect("blockedCategoryResults should be an object");
    for (key, level_value) in blocked_levels {
        let level = level_value
            .as_str()
            .unwrap_or_else(|| panic!("{key} proof level should be a string"));
        let result = blocked_results
            .get(key)
            .unwrap_or_else(|| panic!("{key} blocked category result missing"));
        let expected_text_collected = result["expectedTextCollected"].as_bool().unwrap_or(false);
        let reason_text_collected = result["blockedReasonTextCollected"]
            .as_bool()
            .unwrap_or(false);
        if level == "live_gtk_atspi_proof" {
            assert!(
                expected_text_collected || reason_text_collected,
                "{key} must not claim live proof without collected blocker text"
            );
        }
    }
}

#[test]
fn gtk_automation_preserves_safe_writable_row_count() {
    assert_eq!(SAFE_WRITABLE_ROWS.len(), 341);
}

#[test]
fn completion_readiness_reports_exist_and_preserve_safety_boundaries() {
    let audit: Value = serde_json::from_str(
        &fs::read_to_string("data/reports/completion-readiness-audit.v0.55.2.json")
            .expect("completion readiness report should read"),
    )
    .expect("completion readiness report should parse");
    assert_eq!(audit["schemaVersion"], 1);
    assert_eq!(audit["artifactKind"], "completion_readiness_audit");
    assert_eq!(
        audit["projectModel"],
        "v0.55.2 / 341 readable / 341 writable / 0 blocked"
    );
    assert_eq!(audit["projectDataMigratedToHyprland0554"], false);
    assert_eq!(audit["coverageModel"]["safeWritableRowsLen"], 341);
    assert_eq!(audit["coverageModel"]["exportsUpdatedInThisSprint"], false);
    assert_eq!(audit["writeSafetyStatus"]["highRiskWritesEnabled"], false);
    assert_eq!(
        audit["writeSafetyStatus"]["displayRenderRiskyWritesEnabled"],
        false
    );
    assert_eq!(
        audit["writeSafetyStatus"]["generatedScriptSymlinkProfileWritesEnabled"],
        false
    );
    assert_eq!(
        audit["writeSafetyStatus"]["duplicateConflictsStillBlock"],
        true
    );
    assert_eq!(
        audit["writeSafetyStatus"]["missingDefaultInsertionStillBlocked"],
        true
    );
    assert_eq!(audit["safetyBoundaries"]["liveSwapModeUsed"], false);
    assert_eq!(audit["safetyBoundaries"]["realConfigEdited"], false);
    assert_eq!(audit["safetyBoundaries"]["hyprlandReloaded"], false);
    assert_eq!(audit["packagingStatus"]["releaseCreated"], false);
    assert_eq!(SAFE_WRITABLE_ROWS.len(), 341);

    let wrap_up: Value = serde_json::from_str(
        &fs::read_to_string("data/reports/completion-wrap-up-plan.v0.55.2.json")
            .expect("completion wrap-up plan should read"),
    )
    .expect("completion wrap-up plan should parse");
    assert_eq!(wrap_up["schemaVersion"], 1);
    assert_eq!(wrap_up["artifactKind"], "completion_wrap_up_plan");
    assert_eq!(
        wrap_up["projectModel"],
        "v0.55.2 / 341 readable / 341 writable / 0 blocked"
    );
    assert_eq!(wrap_up["projectDataMigratedToHyprland0554"], false);
    assert_eq!(wrap_up["liveSwapModeUsed"], false);
    assert_eq!(wrap_up["realConfigEdited"], false);
    assert_eq!(wrap_up["hyprlandReloaded"], false);
    assert_eq!(wrap_up["mutatingHyprctlUsed"], false);
    assert_eq!(
        wrap_up["whatCanBeFinishedSafelyNext"][0],
        "No obvious green-lane completion item remains open after the final safe-scope validation sweep."
    );
}

#[test]
fn final_completion_reports_exist_and_validate_safe_scope() {
    let final_wrap: Value = serde_json::from_str(
        &fs::read_to_string("data/reports/final-app-completion-wrap-up.v0.55.2.json")
            .expect("final wrap-up report should read"),
    )
    .expect("final wrap-up report should parse");
    assert_eq!(final_wrap["schemaVersion"], 1);
    assert_eq!(final_wrap["artifactKind"], "final_app_completion_wrap_up");
    assert_eq!(
        final_wrap["projectModel"],
        "v0.55.2 / 341 readable / 341 writable / 0 blocked"
    );
    assert_eq!(final_wrap["projectDataMigratedToHyprland0554"], false);
    assert_eq!(
        final_wrap["completionScope"]["completeMeans"],
        "The app is ready for guarded normal-scalar safe-batch use under the current v0.55.2 model, with clear UI copy, stable packaging metadata, final validation reports, and a release checklist."
    );
    assert_eq!(final_wrap["appIdentity"]["appName"], "Hyprland Settings");
    assert_eq!(final_wrap["appIdentity"]["repoName"], "hyprland-settings");
    assert_eq!(final_wrap["appIdentity"]["binaryName"], "hyprland-settings");
    assert_eq!(
        final_wrap["appIdentity"]["appId"],
        "io.github.kyarorukyo.hyprlandsettings"
    );
    assert_eq!(final_wrap["appIdentity"]["githubOwner"], "KyaroruKYO");
    assert_eq!(
        final_wrap["writeSafetyStatus"]["unsafeWritesExpanded"],
        false
    );
    assert_eq!(
        final_wrap["writeSafetyStatus"]["missingDefaultInsertionEnabled"],
        false
    );
    assert_eq!(
        final_wrap["writeSafetyStatus"]["duplicateAutoResolutionEnabled"],
        false
    );
    assert_eq!(
        final_wrap["writeSafetyStatus"]["highRiskWritesEnabled"],
        false
    );
    assert_eq!(
        final_wrap["writeSafetyStatus"]["displayRenderRiskWritesEnabled"],
        false
    );
    assert_eq!(
        final_wrap["writeSafetyStatus"]["profileModeSwitchingEnabled"],
        false
    );

    let release: Value = serde_json::from_str(
        &fs::read_to_string("data/reports/final-release-readiness-checklist.v0.55.2.json")
            .expect("release checklist should read"),
    )
    .expect("release checklist should parse");
    assert_eq!(release["schemaVersion"], 1);
    assert_eq!(release["artifactKind"], "final_release_readiness_checklist");
    assert_eq!(release["notARelease"], true);
    assert_eq!(release["releaseCreated"], false);
    assert_eq!(release["tagCreated"], false);
    assert_eq!(release["packageCreated"], false);
    assert_eq!(release["unsafeScopeStillBlocked"], true);
    assert_eq!(
        release["recommendedReleaseDecision"],
        "Ready to ask for explicit release-boundary approval for the guarded normal-scalar safe-batch v0.55.2 scope."
    );

    let safe_scope: Value = serde_json::from_str(
        &fs::read_to_string("data/reports/final-safe-scope-validation.v0.55.2.json")
            .expect("safe scope validation report should read"),
    )
    .expect("safe scope validation report should parse");
    assert_eq!(safe_scope["schemaVersion"], 1);
    assert_eq!(safe_scope["artifactKind"], "final_safe_scope_validation");
    assert_eq!(safe_scope["SAFE_WRITABLE_ROWS_len"], 341);
    assert_eq!(safe_scope["blockedUnsafeWrites"], true);
    assert_eq!(safe_scope["blockedGeneratedScriptSymlinkProfile"], true);
    assert_eq!(safe_scope["blockedMissingDefaultInsertion"], true);
    assert_eq!(safe_scope["blockedDuplicateResolution"], true);
    assert_eq!(safe_scope["blockedHighRiskDisplayRender"], true);
    assert_eq!(safe_scope["blockedStructuredFamily"], true);
    assert_eq!(safe_scope["blockedRuntimeMutation"], true);
    assert_eq!(safe_scope["realConfigTouched"], false);
    assert_eq!(safe_scope["runtimeTouched"], false);
    assert_eq!(SAFE_WRITABLE_ROWS.len(), 341);
}

#[test]
fn autonomous_safe_scope_continuation_report_marks_green_lane_complete() {
    let continuation: Value = serde_json::from_str(
        &fs::read_to_string("data/reports/autonomous-safe-scope-continuation.v0.55.2.json")
            .expect("autonomous continuation report should read"),
    )
    .expect("autonomous continuation report should parse");

    assert_eq!(continuation["schemaVersion"], 1);
    assert_eq!(
        continuation["artifactKind"],
        "autonomous_safe_scope_continuation"
    );
    assert_eq!(
        continuation["projectModel"],
        "v0.55.2 / 341 readable / 341 writable / 0 blocked"
    );
    assert_eq!(continuation["projectDataMigratedToHyprland0554"], false);
    assert_eq!(
        continuation["greenLaneStatus"]["remainingGreenLaneItems"]
            .as_array()
            .expect("remaining green-lane item list should exist")
            .len(),
        0
    );
    assert_eq!(
        continuation["redLaneActionsPerformed"]["releaseCreated"],
        false
    );
    assert_eq!(continuation["redLaneActionsPerformed"]["tagCreated"], false);
    assert_eq!(
        continuation["redLaneActionsPerformed"]["packageArtifactCreated"],
        false
    );
    assert_eq!(
        continuation["redLaneActionsPerformed"]["realUserConfigEdited"],
        false
    );
    assert_eq!(
        continuation["redLaneActionsPerformed"]["hyprland0554Migration"],
        false
    );
    assert_eq!(
        continuation["recommendedNextAction"],
        "Ask for explicit release-boundary approval before creating any release, tag, or package artifact."
    );
    assert_eq!(SAFE_WRITABLE_ROWS.len(), 341);
}

#[test]
fn public_metadata_matches_safe_scope_and_does_not_overclaim() {
    let readme = fs::read_to_string("README.md").expect("README should read");
    assert!(readme.contains("guarded safe-batch config writes"));
    assert!(readme.contains("Hyprland `0.55.2`"));
    assert!(readme.contains("Production Apply remains narrower"));
    assert!(readme.contains("It does not add missing config lines yet."));
    assert!(readme.contains("It does not auto-resolve duplicate settings."));
    assert!(readme.contains("It does not switch profiles"));
    assert!(readme.contains("It does not migrate this v0.55.2 data/model to Hyprland 0.55.4."));

    let cargo = fs::read_to_string("Cargo.toml").expect("Cargo.toml should read");
    assert!(cargo.contains("name = \"hyprland-settings\""));
    assert!(cargo.contains("description = \"GTK4/libadwaita Hyprland settings app with guarded safe-batch config writes\""));

    let desktop =
        fs::read_to_string("data/applications/io.github.kyarorukyo.hyprlandsettings.desktop")
            .expect("desktop file should read");
    assert!(desktop.contains("Name=Hyprland Settings"));
    assert!(desktop.contains("Exec=hyprland-settings"));
    assert!(desktop.contains("Icon=io.github.kyarorukyo.hyprlandsettings"));
    assert!(desktop.contains("guarded safe-batch"));

    let metainfo =
        fs::read_to_string("data/metainfo/io.github.kyarorukyo.hyprlandsettings.metainfo.xml")
            .expect("metainfo should read");
    assert!(metainfo.contains("<id>io.github.kyarorukyo.hyprlandsettings</id>"));
    assert!(metainfo.contains("https://github.com/KyaroruKYO/hyprland-settings"));
    assert!(metainfo.contains("guarded safe-batch writes"));
    assert!(metainfo.contains("The bundled metadata targets Hyprland 0.55.2."));
    assert!(!metainfo.contains("The current release is read-only."));
    assert!(!metainfo.contains("does not read live Hyprland config files"));
}

#[test]
fn active_user_facing_copy_uses_safe_batch_not_pilot_language() {
    let model = fs::read_to_string("src/ui/model.rs").expect("ui model should read");
    assert!(model.contains("guarded safe-batch writing"));
    assert!(!model.contains("This row is the only active write pilot"));

    let readiness = fs::read_to_string("src/one_target_pilot_readiness.rs")
        .expect("readiness source should read");
    assert!(readiness.contains("Final safe-batch readiness"));
    assert!(readiness
        .contains("Safe-batch writing is available only for eligible normal scalar settings."));
    assert!(readiness.contains("High-risk rows remain blocked from safe-batch writing."));
    assert!(readiness.contains("These targets remain blocked from safe-batch writing."));
    assert!(!readiness.contains("The first write pilot is not ready yet."));
    assert!(!readiness.contains("first production write pilot"));
    assert!(!readiness.contains("recovery execution is still blocked until the pilot is approved"));
}
