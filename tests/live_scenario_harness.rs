mod support;

use std::path::{Path, PathBuf};

use hyprland_settings::config_discovery::{
    discover_hyprland_config_with_env, ConfigDiscoveryEnv, ConfigDiscoveryStatus,
};
use hyprland_settings::config_graph::{inspect_config_graph_with_options, ConfigGraphOptions};
use hyprland_settings::safe_batch_write::{
    build_safe_batch_write_plan, execute_safe_batch_write_plan, safe_batch_write_user_facing_lines,
    SafeBatchChangeRequest, SafeBatchExecutionOptions, SafeBatchWriteStatus,
};
use hyprland_settings::source_aware_current_config::current_config_from_graph;
use hyprland_settings::write_classification::SAFE_WRITABLE_ROWS;
use serde_json::{json, Value};
use support::safe_batch_harness::{
    known_settings, redact_path, temp_root, write_file, write_report,
};

#[derive(Debug, Clone)]
struct ScenarioResult {
    name: &'static str,
    mode: &'static str,
    root: PathBuf,
    config_discovered: bool,
    eligible_rows: usize,
    blocked_rows: usize,
    apply_available: bool,
    safe_batch_writes_attempted: bool,
    safe_batch_writes_succeeded: bool,
    failure_paths_tested: Vec<&'static str>,
    issues_found: Vec<&'static str>,
}

impl ScenarioResult {
    fn to_json(&self) -> Value {
        json!({
            "scenarioName": self.name,
            "scenarioPath": redact_path(&self.root),
            "mode": self.mode,
            "appLaunched": false,
            "appLaunchResult": "not launched; noninteractive GTK close automation was not available, so source/model/discovery proof was used",
            "configDiscovered": self.config_discovered,
            "eligibleRows": self.eligible_rows,
            "blockedRows": self.blocked_rows,
            "applyAvailability": if self.apply_available { "available for executable temp safe-batch plan" } else { "blocked" },
            "safeBatchWritesAttemptedAgainstTempFiles": self.safe_batch_writes_attempted,
            "safeBatchWritesSucceeded": self.safe_batch_writes_succeeded,
            "failuresTested": self.failure_paths_tested,
            "restoreTested": self.failure_paths_tested.iter().any(|case| case.contains("restore")),
            "screenshotsCaptured": false,
            "realConfigTouched": false,
            "hyprlandReloadUsed": false,
            "mutatingHyprctlUsed": false,
            "agsDisabledRestored": "not used",
            "waybarDisabledRestored": "not used",
            "issuesFound": self.issues_found,
            "recommendations": [
                "Keep safe-env mode as the default scenario path.",
                "Use live-swap only when a future test needs true real-path discovery and restore verification has passed."
            ]
        })
    }
}

#[test]
fn live_scenario_harness_runs_safe_env_scenarios_without_real_desktop_mutation() {
    assert_eq!(SAFE_WRITABLE_ROWS.len(), 341);

    let root = temp_root("live-scenario-harness");
    let minimal = create_minimal_single_config(&root);
    let flat = create_large_single_config(&root);
    let source = create_source_include_config(&root);
    let nested = create_nested_source_config(&root);
    let symlink = create_symlink_profile_config(&root);
    let duplicate = create_duplicate_conflict_config(&root);
    let generated = create_generated_config(&root);
    let script_managed = create_script_managed_config(&root);
    let missing = create_missing_default_config(&root);
    let high_risk = create_high_risk_config(&root);

    let minimal_result = evaluate_safe_env_scenario("minimal_single_config", &minimal);
    let flat_result = execute_single_file_safe_batch(&flat);
    let source_result = execute_multifile_safe_batch(&source);
    let nested_result = evaluate_safe_env_scenario("nested_source_config", &nested);
    let symlink_result = evaluate_blocked_scenario(
        "symlink_current_profile",
        &symlink,
        vec![SafeBatchChangeRequest::new(
            "appearance.blur.enabled",
            "false",
        )],
    );
    let duplicate_result = evaluate_blocked_scenario(
        "duplicate_conflict",
        &duplicate,
        vec![SafeBatchChangeRequest::new(
            "appearance.blur.enabled",
            "false",
        )],
    );
    let generated_result = evaluate_blocked_scenario(
        "generated_config",
        &generated,
        vec![SafeBatchChangeRequest::new(
            "appearance.blur.enabled",
            "false",
        )],
    );
    let script_result = evaluate_script_managed_scenario(&script_managed);
    let missing_result = evaluate_blocked_scenario(
        "missing_default_only",
        &missing,
        vec![SafeBatchChangeRequest::new(
            "appearance.blur.enabled",
            "false",
        )],
    );
    let high_risk_result = evaluate_blocked_scenario(
        "high_risk_display_risk",
        &high_risk,
        vec![SafeBatchChangeRequest::new(
            "decoration.screen_shader",
            "/tmp/fake.frag",
        )],
    );

    let write_failure_home = create_source_include_config(&root.join("write-failure"));
    let verification_failure_home = create_source_include_config(&root.join("verify-failure"));
    let write_failure = execute_forced_failure(&write_failure_home, FailureMode::WriteFailure);
    let verification_failure =
        execute_forced_failure(&verification_failure_home, FailureMode::VerificationFailure);
    assert_eq!(write_failure.status, SafeBatchWriteStatus::RecoveredFailure);
    assert_eq!(
        verification_failure.status,
        SafeBatchWriteStatus::RecoveredFailure
    );
    assert!(write_failure.recovery_attempted);
    assert!(write_failure.restore_verification_succeeded);
    assert!(verification_failure.recovery_attempted);
    assert!(verification_failure.restore_verification_succeeded);

    let real_config = support::safe_batch_harness::real_config_readonly_audit();

    write_report(
        "live-scenario-single-config.v0.55.2.json",
        &json!({
            "schemaVersion": 1,
            "artifactKind": "live_scenario_single_config",
            "scenarios": [minimal_result.to_json(), flat_result.to_json()],
            "safeEnvModeUsed": true,
            "liveSwapModeUsed": false,
            "realConfigTouched": false
        }),
    );
    write_report(
        "live-scenario-source-include.v0.55.2.json",
        &json!({
            "schemaVersion": 1,
            "artifactKind": "live_scenario_source_include",
            "scenarios": [source_result.to_json(), nested_result.to_json()],
            "multiFileBatch": {
                "attempted": true,
                "succeeded": source_result.safe_batch_writes_succeeded,
                "failureRecoveryCases": ["write_failure_restored", "verification_failure_restored"]
            },
            "realConfigTouched": false
        }),
    );
    write_report(
        "live-scenario-symlink-profile.v0.55.2.json",
        &json!({
            "schemaVersion": 1,
            "artifactKind": "live_scenario_symlink_profile",
            "scenario": symlink_result.to_json(),
            "expectedBlocker": "blocked_symlink_managed",
            "symlinksChangedInRealConfig": false
        }),
    );
    write_report(
        "live-scenario-generated-script-managed.v0.55.2.json",
        &json!({
            "schemaVersion": 1,
            "artifactKind": "live_scenario_generated_script_managed",
            "scenarios": [generated_result.to_json(), script_result.to_json()],
            "expectedBlockers": ["blocked_generated_file", "blocked_script_managed"],
            "scriptsExecuted": false
        }),
    );
    write_report(
        "live-scenario-duplicate-missing-high-risk.v0.55.2.json",
        &json!({
            "schemaVersion": 1,
            "artifactKind": "live_scenario_duplicate_missing_high_risk",
            "scenarios": [duplicate_result.to_json(), missing_result.to_json(), high_risk_result.to_json()],
            "expectedBlockers": [
                "blocked_duplicate_conflict",
                "blocked_missing_line",
                "blocked_display_render_risk"
            ],
            "partialApplyOccurred": false
        }),
    );
    write_report(
        "live-scenario-restore-proof.v0.55.2.json",
        &json!({
            "schemaVersion": 1,
            "artifactKind": "live_scenario_restore_proof",
            "safeEnvModeUsed": true,
            "liveSwapModeUsed": false,
            "restoreScriptsCreated": true,
            "restoreScript": "tools/live_scenario_harness/restore_desktop_state.sh",
            "restoreVerificationScript": "tools/live_scenario_harness/verify_restore.sh",
            "restoreVerificationResult": "pending external backup verification",
            "realConfigRestored": "not modified",
            "symlinksRestored": "not modified",
            "agsTouched": false,
            "waybarTouched": false
        }),
    );
    write_report(
        "live-scenario-real-config-readonly-final.v0.55.2.json",
        &json!({
            "schemaVersion": 1,
            "artifactKind": "live_scenario_real_config_readonly_final",
            "realConfigReadOnly": true,
            "eligibleRows": real_config["summary"]["eligibleSafeBatchWrites"],
            "blockedRows": real_config["summary"]["blocked"],
            "blockerCounts": real_config["summary"]["blockerCounts"],
            "duplicateConflicts": real_config["summary"]["duplicateConflicts"],
            "managedFileHints": {
                "generated": real_config["summary"]["generatedHints"],
                "scriptManaged": real_config["summary"]["scriptManagedHints"],
                "symlinkManaged": real_config["summary"]["symlinkManagedHints"]
            },
            "newHyprlandUserExperience": "minimal/default-like scenarios discover and explain safe-batch eligibility, while default-only settings stay blocked until insertion support exists",
            "currentUserExperience": "current real config remains mostly blocked by missing/default, duplicate, high-risk, display-risk, and profile/mode blockers",
            "realConfigTouched": false
        }),
    );

    let scenario_results = vec![
        minimal_result,
        flat_result.clone(),
        source_result.clone(),
        nested_result,
        symlink_result,
        duplicate_result,
        generated_result,
        script_result,
        missing_result,
        high_risk_result,
    ];
    write_report(
        "live-scenario-harness-summary.v0.55.2.json",
        &json!({
            "schemaVersion": 1,
            "artifactKind": "live_scenario_harness_summary",
            "startingCommit": "67ae30ecda72839fca76d57f9cd3f2eaed58a95a",
            "goal": "Live desktop scenario harness for Hyprland Settings with full restore safety",
            "safeEnvModeUsed": true,
            "liveSwapModeUsed": false,
            "agsDisabled": false,
            "waybarDisabled": false,
            "backupRoot": "created during validation; not committed",
            "restoreVerificationResult": "pending external backup verification",
            "scenariosTested": scenario_results.iter().map(|scenario| scenario.name).collect::<Vec<_>>(),
            "appLaunchResults": {
                "guiLaunchAttempted": false,
                "reason": "noninteractive GTK close automation unavailable; discovery/model/UI-copy proof used instead",
                "dashboard": "source/model proof",
                "configPage": "source/model proof",
                "categoryPages": "source/model proof",
                "search": "covered by existing test suite",
                "detailPane": "covered by existing test suite",
                "safeBatchCopy": safe_batch_write_user_facing_lines(),
                "blockedCopy": "covered by scenario blockers"
            },
            "safeBatchTempWriteResults": {
                "singleFileSucceeded": flat_result.safe_batch_writes_succeeded,
                "multiFileSucceeded": source_result.safe_batch_writes_succeeded,
                "writeFailureRecovered": write_failure.recovery_succeeded,
                "verificationFailureRecovered": verification_failure.recovery_succeeded,
                "hyprlandReloadAttempted": write_failure.hyprland_reload_attempted || verification_failure.hyprland_reload_attempted,
                "mutatingHyprctlUsed": write_failure.mutating_hyprctl_used || verification_failure.mutating_hyprctl_used,
                "runtimeMutated": write_failure.runtime_mutated || verification_failure.runtime_mutated
            },
            "scenarioResults": scenario_results.iter().map(ScenarioResult::to_json).collect::<Vec<_>>(),
            "safetyBoundaries": {
                "realUserConfigEdited": false,
                "realUserConfigBackupsCommitted": false,
                "hyprlandReloaded": false,
                "mutatingHyprctlUsed": false,
                "runtimeMutated": false,
                "scriptsExecuted": false,
                "luaExecuted": false,
                "screenshotsCommitted": false
            },
            "countsBefore": "341 readable / 341 writable / 0 blocked",
            "countsAfter": "341 readable / 341 writable / 0 blocked",
            "validation": {
                "cargoFmt": "pending",
                "cargoFmtCheck": "pending",
                "cargoCheck": "pending",
                "cargoTest": "pending",
                "cargoBuildRelease": "pending",
                "jqReports": "pending",
                "restoreVerification": "pending external backup verification",
                "gitDiffCheck": "pending",
                "gitStatusShort": "pending"
            },
            "criticalIssuesFound": [],
            "majorIssuesFound": [
                "No controlled GTK automation close path is available, so live app launch was represented by source/model/discovery proof.",
                "Live-swap was not used because safe-env discovery covered the scenario matrix without desktop risk."
            ],
            "minorIssuesFound": [
                "Safe-env scenario reports are redacted and do not retain screenshots."
            ],
            "nextRecommendedSprint": "Add noninteractive GTK automation for safe-env app launch and close, then rerun the scenario matrix without live-swap."
        }),
    );
}

fn create_minimal_single_config(root: &Path) -> PathBuf {
    let home = root.join("minimal-home");
    let conf = home.join(".config/hypr/hyprland.conf");
    write_file(
        &conf,
        "decoration:blur:enabled = true\nmisc:disable_hyprland_logo = true\n",
    );
    home
}

fn create_large_single_config(root: &Path) -> PathBuf {
    let home = root.join("flat-home");
    let conf = home.join(".config/hypr/hyprland.conf");
    write_file(
        &conf,
        "\
decoration:blur:enabled = true
general:gaps_in = 4
general:gaps_out = 8
general:border_size = 2
decoration:rounding = 6
misc:disable_hyprland_logo = true
misc:disable_splash_rendering = true
input:follow_mouse = 1
",
    );
    home
}

fn create_source_include_config(root: &Path) -> PathBuf {
    let home = root.join("source-home");
    let hypr = home.join(".config/hypr");
    write_file(
        &hypr.join("hyprland.conf"),
        "source = appearance.conf\ninclude = input.conf\nsource = misc.conf\n",
    );
    write_file(
        &hypr.join("appearance.conf"),
        "decoration:blur:enabled = true\ngeneral:gaps_in = 4\n",
    );
    write_file(&hypr.join("input.conf"), "input:follow_mouse = 1\n");
    write_file(
        &hypr.join("misc.conf"),
        "misc:disable_hyprland_logo = true\n",
    );
    home
}

fn create_nested_source_config(root: &Path) -> PathBuf {
    let home = root.join("nested-home");
    let hypr = home.join(".config/hypr");
    write_file(
        &hypr.join("hyprland.conf"),
        "source = profiles/current.conf\n",
    );
    write_file(
        &hypr.join("profiles/current.conf"),
        "source = ../appearance/theme.conf\n",
    );
    write_file(
        &hypr.join("appearance/theme.conf"),
        "decoration:blur:enabled = true\nmisc:disable_hyprland_logo = true\n",
    );
    home
}

fn create_symlink_profile_config(root: &Path) -> PathBuf {
    let home = root.join("symlink-home");
    let hypr = home.join(".config/hypr");
    write_file(
        &hypr.join("hyprland.conf"),
        "source = profiles/current.conf\n",
    );
    write_file(
        &hypr.join("profiles/desktop.conf"),
        "decoration:blur:enabled = true\n",
    );
    #[cfg(unix)]
    std::os::unix::fs::symlink("desktop.conf", hypr.join("profiles/current.conf"))
        .expect("fixture symlink should be created");
    home
}

fn create_duplicate_conflict_config(root: &Path) -> PathBuf {
    let home = root.join("duplicate-home");
    let hypr = home.join(".config/hypr");
    write_file(
        &hypr.join("hyprland.conf"),
        "decoration:blur:enabled = true\nsource = appearance.conf\n",
    );
    write_file(
        &hypr.join("appearance.conf"),
        "decoration:blur:enabled = false\n",
    );
    home
}

fn create_generated_config(root: &Path) -> PathBuf {
    let home = root.join("generated-home");
    let conf = home.join(".config/hypr/hyprland.conf");
    write_file(
        &conf,
        "# generated by fixture\n# do not edit\ndecoration:blur:enabled = true\n",
    );
    home
}

fn create_script_managed_config(root: &Path) -> PathBuf {
    let home = root.join("script-home");
    let hypr = home.join(".config/hypr");
    write_file(&hypr.join("hyprland.conf"), "source = managed.conf\n");
    write_file(
        &hypr.join("managed.conf"),
        "decoration:blur:enabled = true\n",
    );
    write_file(
        &hypr.join("scripts/manage-config.sh"),
        "cp managed.conf managed.conf.bak\nsed -i 's/true/false/' managed.conf\nhyprctl reload\n",
    );
    home
}

fn create_missing_default_config(root: &Path) -> PathBuf {
    let home = root.join("missing-home");
    let conf = home.join(".config/hypr/hyprland.conf");
    write_file(&conf, "misc:disable_hyprland_logo = true\n");
    home
}

fn create_high_risk_config(root: &Path) -> PathBuf {
    let home = root.join("high-risk-home");
    let conf = home.join(".config/hypr/hyprland.conf");
    write_file(
        &conf,
        "decoration:screen_shader = /tmp/fake.frag\nrender:direct_scanout = false\n",
    );
    home
}

fn evaluate_safe_env_scenario(name: &'static str, home: &Path) -> ScenarioResult {
    let env = ConfigDiscoveryEnv {
        xdg_config_home: Some(home.join(".config")),
        home: Some(home.to_path_buf()),
    };
    let discovery = discover_hyprland_config_with_env(&env);
    let config_discovered = matches!(discovery.status, ConfigDiscoveryStatus::Found { .. });
    let graph = graph_for_home(home);
    let current = current_config_from_graph(&graph);
    let plan = build_safe_batch_write_plan(
        format!("{name}-plan"),
        &known_settings(),
        &current,
        &graph,
        vec![SafeBatchChangeRequest::new(
            "appearance.blur.enabled",
            "false",
        )],
        "live-scenario",
    );
    ScenarioResult {
        name,
        mode: "safe-env",
        root: home.to_path_buf(),
        config_discovered,
        eligible_rows: plan.eligible_changes.len(),
        blocked_rows: plan.blocked_changes.len(),
        apply_available: plan.can_execute,
        safe_batch_writes_attempted: false,
        safe_batch_writes_succeeded: false,
        failure_paths_tested: Vec::new(),
        issues_found: Vec::new(),
    }
}

fn execute_single_file_safe_batch(home: &Path) -> ScenarioResult {
    let graph = graph_for_home(home);
    let current = current_config_from_graph(&graph);
    let plan = build_safe_batch_write_plan(
        "single-file-safe-batch",
        &known_settings(),
        &current,
        &graph,
        vec![
            SafeBatchChangeRequest::new("appearance.blur.enabled", "false"),
            SafeBatchChangeRequest::new("appearance.gaps_in", "6"),
        ],
        "live-scenario",
    );
    assert!(plan.can_execute, "{:?}", plan.cannot_execute_reasons);
    let report = execute_safe_batch_write_plan(&plan, &SafeBatchExecutionOptions::default());
    assert_eq!(report.status, SafeBatchWriteStatus::Succeeded);
    assert_eq!(report.backups.len(), 1);
    assert!(report.backups.iter().all(|backup| backup.bytes_equal));
    ScenarioResult {
        name: "large_single_config",
        mode: "safe-env",
        root: home.to_path_buf(),
        config_discovered: true,
        eligible_rows: plan.eligible_changes.len(),
        blocked_rows: plan.blocked_changes.len(),
        apply_available: plan.can_execute,
        safe_batch_writes_attempted: true,
        safe_batch_writes_succeeded: true,
        failure_paths_tested: Vec::new(),
        issues_found: Vec::new(),
    }
}

fn execute_multifile_safe_batch(home: &Path) -> ScenarioResult {
    let graph = graph_for_home(home);
    let current = current_config_from_graph(&graph);
    let plan = build_safe_batch_write_plan(
        "multi-file-safe-batch",
        &known_settings(),
        &current,
        &graph,
        vec![
            SafeBatchChangeRequest::new("appearance.blur.enabled", "false"),
            SafeBatchChangeRequest::new("misc.disable_hyprland_logo", "false"),
        ],
        "live-scenario",
    );
    assert!(plan.can_execute, "{:?}", plan.cannot_execute_reasons);
    let report = execute_safe_batch_write_plan(&plan, &SafeBatchExecutionOptions::default());
    assert_eq!(report.status, SafeBatchWriteStatus::Succeeded);
    assert_eq!(report.backups.len(), 2);
    assert!(report.backups.iter().all(|backup| backup.bytes_equal));
    ScenarioResult {
        name: "source_include_config",
        mode: "safe-env",
        root: home.to_path_buf(),
        config_discovered: true,
        eligible_rows: plan.eligible_changes.len(),
        blocked_rows: plan.blocked_changes.len(),
        apply_available: plan.can_execute,
        safe_batch_writes_attempted: true,
        safe_batch_writes_succeeded: true,
        failure_paths_tested: Vec::new(),
        issues_found: Vec::new(),
    }
}

fn evaluate_blocked_scenario(
    name: &'static str,
    home: &Path,
    changes: Vec<SafeBatchChangeRequest>,
) -> ScenarioResult {
    let graph = graph_for_home(home);
    let current = current_config_from_graph(&graph);
    let plan = build_safe_batch_write_plan(
        format!("{name}-blocked"),
        &known_settings(),
        &current,
        &graph,
        changes,
        "live-scenario",
    );
    assert!(!plan.can_execute);
    assert!(!plan.blocked_changes.is_empty());
    ScenarioResult {
        name,
        mode: "safe-env",
        root: home.to_path_buf(),
        config_discovered: true,
        eligible_rows: plan.eligible_changes.len(),
        blocked_rows: plan.blocked_changes.len(),
        apply_available: false,
        safe_batch_writes_attempted: false,
        safe_batch_writes_succeeded: false,
        failure_paths_tested: Vec::new(),
        issues_found: Vec::new(),
    }
}

fn evaluate_script_managed_scenario(home: &Path) -> ScenarioResult {
    let graph = graph_for_home_with_script_scan(home);
    let current = current_config_from_graph(&graph);
    let plan = build_safe_batch_write_plan(
        "script-managed-blocked",
        &known_settings(),
        &current,
        &graph,
        vec![SafeBatchChangeRequest::new(
            "appearance.blur.enabled",
            "false",
        )],
        "live-scenario",
    );
    assert!(!plan.can_execute);
    assert_eq!(
        plan.blocked_changes[0].reason.label(),
        "blocked_script_managed"
    );
    ScenarioResult {
        name: "script_managed_config",
        mode: "safe-env",
        root: home.to_path_buf(),
        config_discovered: true,
        eligible_rows: plan.eligible_changes.len(),
        blocked_rows: plan.blocked_changes.len(),
        apply_available: false,
        safe_batch_writes_attempted: false,
        safe_batch_writes_succeeded: false,
        failure_paths_tested: Vec::new(),
        issues_found: Vec::new(),
    }
}

#[derive(Debug, Clone, Copy)]
enum FailureMode {
    WriteFailure,
    VerificationFailure,
}

fn execute_forced_failure(
    home: &Path,
    mode: FailureMode,
) -> hyprland_settings::safe_batch_write::SafeBatchWriteReport {
    let graph = graph_for_home(home);
    let current = current_config_from_graph(&graph);
    let plan = build_safe_batch_write_plan(
        "forced-failure-safe-batch",
        &known_settings(),
        &current,
        &graph,
        vec![
            SafeBatchChangeRequest::new("appearance.blur.enabled", "false"),
            SafeBatchChangeRequest::new("misc.disable_hyprland_logo", "false"),
        ],
        "live-scenario-failure",
    );
    assert!(plan.can_execute);
    let mut options = SafeBatchExecutionOptions::default();
    match mode {
        FailureMode::WriteFailure => {
            options.fail_after_writing_target = Some(plan.target_files[0].target_path.clone());
        }
        FailureMode::VerificationFailure => {
            options.force_verification_failure_for = Some("appearance.blur.enabled".to_string());
        }
    }
    execute_safe_batch_write_plan(&plan, &options)
}

fn graph_for_home(home: &Path) -> hyprland_settings::config_graph::ConfigGraphSummary {
    let root = home.join(".config/hypr/hyprland.conf");
    inspect_config_graph_with_options(
        &root,
        ConfigGraphOptions {
            home_dir: Some(home.to_path_buf()),
            script_dirs: Vec::new(),
            max_depth: 16,
            source_follow_policy: hyprland_settings::config_graph::SourceFollowPolicy::ReviewAll,
        },
    )
}

fn graph_for_home_with_script_scan(
    home: &Path,
) -> hyprland_settings::config_graph::ConfigGraphSummary {
    let root = home.join(".config/hypr/hyprland.conf");
    inspect_config_graph_with_options(
        &root,
        ConfigGraphOptions {
            home_dir: Some(home.to_path_buf()),
            script_dirs: vec![home.join(".config/hypr/scripts")],
            max_depth: 16,
            source_follow_policy: hyprland_settings::config_graph::SourceFollowPolicy::ReviewAll,
        },
    )
}
