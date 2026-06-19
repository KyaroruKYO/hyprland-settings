use std::fs;
use std::path::{Path, PathBuf};
use std::time::{SystemTime, UNIX_EPOCH};

use hyprland_settings::future_capability::{
    assess_hyprland_version_migration, current_v0552_data_bundle, disabled_migration_review,
    disabled_missing_default_insertion_review, disabled_profile_switch_review,
    duplicate_occurrence_model, high_risk_recovery_review, replace_duplicate_occurrence_safe_env,
    runtime_action_policy, structured_family_model, switch_profile_symlink_safe_env,
    validate_structured_edit_candidate, DuplicateReplacementOptions, DuplicateReplacementRequest,
    DuplicateReplacementStatus, MockWatchdog, MockWatchdogState, RuntimeAction,
    RuntimeDryRunExecutor,
};
use hyprland_settings::missing_default_insertion::{
    build_missing_default_insertion_plan, MissingDefaultInsertionRequest,
};

fn temp_root(label: &str) -> PathBuf {
    let stamp = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .expect("time should work")
        .as_nanos();
    let root = std::env::temp_dir().join(format!(
        "hyprland-settings-future-capability-{label}-{}-{stamp}",
        std::process::id()
    ));
    fs::create_dir_all(&root).expect("temp root should be created");
    root
}

fn write_file(path: &Path, contents: &str) {
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent).expect("parent should exist");
    }
    fs::write(path, contents).expect("fixture should write");
}

#[test]
fn missing_default_review_scaffold_stays_disabled_for_production() {
    let root = temp_root("missing-review");
    let config = root.join("hyprland.conf");
    write_file(&config, "decoration:blur:enabled = true\n");
    let plan = build_missing_default_insertion_plan(MissingDefaultInsertionRequest {
        setting_id: "misc.disable_splash_rendering".to_string(),
        proposed_value: "true".to_string(),
        target_path: config,
        backup_stamp: "fixture".to_string(),
    });
    let review = disabled_missing_default_insertion_review(&plan);

    assert!(!review.production_apply_enabled);
    assert!(review
        .user_copy
        .contains("does not add new config lines yet"));
    assert!(review
        .required_gates
        .iter()
        .any(|gate| gate.contains("production UI approval")));
}

#[test]
fn duplicate_occurrence_model_lists_same_file_and_source_layer_occurrences() {
    let root = temp_root("duplicate-model");
    let root_conf = root.join("hyprland.conf");
    let sourced_conf = root.join("appearance.conf");
    write_file(&root_conf, "decoration:blur:enabled = true\n");
    write_file(&sourced_conf, "decoration:blur:enabled = false\n");

    let model = duplicate_occurrence_model(
        "decoration.blur.enabled",
        &[(root_conf.clone(), 0), (sourced_conf.clone(), 1)],
    )
    .expect("duplicate model should build");

    assert_eq!(model.occurrences.len(), 2);
    assert!(!model.selector_enabled);
    assert!(!model.production_write_enabled);
    assert!(model
        .occurrences
        .iter()
        .any(|occurrence| occurrence.path == root_conf && occurrence.source_depth == 0));
    assert!(model
        .occurrences
        .iter()
        .any(|occurrence| occurrence.path == sourced_conf && occurrence.source_depth == 1));
}

#[test]
fn duplicate_replacement_safe_env_replaces_exact_selected_line_and_verifies() {
    let root = temp_root("duplicate-replace");
    let config = root.join("hyprland.conf");
    write_file(
        &config,
        "decoration:blur:enabled = true\nmisc:disable_splash_rendering = false\n",
    );
    let model = duplicate_occurrence_model("decoration.blur.enabled", &[(config.clone(), 0)])
        .expect("model should build");
    let occurrence = model.occurrences[0].clone();

    let report = replace_duplicate_occurrence_safe_env(
        &DuplicateReplacementRequest {
            occurrence,
            expected_old_value: "true".to_string(),
            proposed_value: "false".to_string(),
            backup_stamp: "fixture".to_string(),
        },
        &DuplicateReplacementOptions::default(),
    );

    assert_eq!(report.status, DuplicateReplacementStatus::Succeeded);
    assert!(report.backup_bytes_equal);
    assert!(report.exact_line_replaced);
    assert!(report.reread_verified);
    assert!(!report.production_write_enabled);
    assert!(!report.real_config_touched);
    assert!(fs::read_to_string(config)
        .expect("config should read")
        .starts_with("decoration:blur:enabled = false\n"));
}

#[test]
fn duplicate_replacement_restores_after_verification_failure() {
    let root = temp_root("duplicate-restore");
    let config = root.join("hyprland.conf");
    let original = "decoration:blur:enabled = true\n";
    write_file(&config, original);
    let occurrence = duplicate_occurrence_model("decoration.blur.enabled", &[(config.clone(), 0)])
        .expect("model should build")
        .occurrences[0]
        .clone();

    let report = replace_duplicate_occurrence_safe_env(
        &DuplicateReplacementRequest {
            occurrence,
            expected_old_value: "true".to_string(),
            proposed_value: "false".to_string(),
            backup_stamp: "fixture".to_string(),
        },
        &DuplicateReplacementOptions {
            force_verification_failure: true,
            ..DuplicateReplacementOptions::default()
        },
    );

    assert_eq!(report.status, DuplicateReplacementStatus::RecoveredFailure);
    assert!(report.restore_attempted);
    assert!(report.restore_succeeded);
    assert_eq!(
        fs::read_to_string(config).expect("config should read"),
        original
    );
}

#[test]
fn high_risk_mock_watchdog_handles_confirm_timeout_revert_and_failure() {
    let mut confirmed = MockWatchdog::arm("session", "token", 10);
    assert_eq!(confirmed.state, MockWatchdogState::Pending);
    assert_eq!(confirmed.confirm("wrong"), MockWatchdogState::Pending);
    assert_eq!(confirmed.confirm("token"), MockWatchdogState::Confirmed);
    assert!(!confirmed.real_runtime_enabled);

    let mut reverted = MockWatchdog::arm("session-2", "token", 10);
    assert_eq!(reverted.tick(11, true), MockWatchdogState::Reverted);

    let mut failed = MockWatchdog::arm("session-3", "token", 10);
    assert_eq!(failed.tick(11, false), MockWatchdogState::RecoveryFailed);
}

#[test]
fn high_risk_recovery_review_stays_non_mutating_and_disabled() {
    let mut watchdog = MockWatchdog::arm("session", "token", 10);
    watchdog.tick(11, true);

    let review = high_risk_recovery_review("render.direct_scanout", &watchdog);

    assert_eq!(review.state, MockWatchdogState::Reverted);
    assert!(!review.production_write_enabled);
    assert!(!review.real_runtime_enabled);
    assert!(review
        .review_lines
        .iter()
        .any(|line| line.contains("does not reload Hyprland")));
}

#[test]
fn structured_family_model_keeps_bind_entries_read_only() {
    let root = temp_root("structured");
    let config = root.join("hyprland.conf");
    write_file(
        &config,
        "bind = SUPER, Return, exec, foot\nmonitor = eDP-1,preferred,auto,1\n",
    );

    let model = structured_family_model(&config, "hl.bind").expect("model should build");

    assert_eq!(model.family_id, "hl.bind");
    assert_eq!(model.entries.len(), 1);
    assert_eq!(model.entries[0].parsed_key, "bind");
    assert!(!model.editor_enabled);
    assert!(!model.production_write_enabled);
    assert!(!model.lossless_render_proven);
}

#[test]
fn structured_edit_candidate_blocks_invalid_input_and_keeps_production_disabled() {
    let accepted =
        validate_structured_edit_candidate("hl.bind", "bind = SUPER, Return, exec, foot");
    assert!(accepted.accepted);
    assert!(!accepted.production_write_enabled);
    assert!(accepted.lossless_render_required);

    let rejected =
        validate_structured_edit_candidate("hl.bind", "monitor = eDP-1,preferred,auto,1");
    assert!(!rejected.accepted);
    assert!(!rejected.production_write_enabled);
    assert!(rejected
        .errors
        .iter()
        .any(|error| error.contains("must start with bind")));

    let multiline = validate_structured_edit_candidate("hl.bind", "bind = A\nbind = B");
    assert!(!multiline.accepted);
    assert!(multiline
        .errors
        .iter()
        .any(|error| error.contains("single-line")));
}

#[cfg(unix)]
#[test]
fn profile_switch_safe_env_switches_and_restores_temp_symlink_only() {
    use std::os::unix::fs::symlink;

    let root = temp_root("profile-switch");
    let profiles = root.join("profiles");
    fs::create_dir_all(&profiles).expect("profiles should create");
    let desktop = profiles.join("desktop.conf");
    let gaming = profiles.join("gaming.conf");
    write_file(&desktop, "general:layout = dwindle\n");
    write_file(&gaming, "general:layout = master\n");
    let current = profiles.join("current.conf");
    symlink(&desktop, &current).expect("current symlink should create");

    let report = switch_profile_symlink_safe_env(&root, &current, &gaming, false);

    assert_eq!(
        report.status,
        hyprland_settings::future_capability::ProfileSwitchStatus::Succeeded
    );
    assert_eq!(
        fs::read_link(&current).expect("current should read"),
        desktop
    );
    assert!(!report.production_switch_enabled);
    assert!(!report.real_config_touched);
    assert!(!report.runtime_touched);
}

#[cfg(unix)]
#[test]
fn profile_switch_review_and_forced_restore_failure_stay_disabled() {
    use std::os::unix::fs::symlink;

    let root = temp_root("profile-failure");
    let profiles = root.join("profiles");
    fs::create_dir_all(&profiles).expect("profiles should create");
    let desktop = profiles.join("desktop.conf");
    let gaming = profiles.join("gaming.conf");
    write_file(&desktop, "general:layout = dwindle\n");
    write_file(&gaming, "general:layout = master\n");
    let current = profiles.join("current.conf");
    symlink(&desktop, &current).expect("current symlink should create");

    let review = disabled_profile_switch_review(&current, Some(desktop.clone()), &gaming);
    assert!(!review.production_switch_enabled);
    assert!(!review.reload_after_switch_enabled);
    assert!(review
        .review_lines
        .iter()
        .any(|line| line.contains("Real profile files")));

    let report = switch_profile_symlink_safe_env(&root, &current, &gaming, true);
    assert_eq!(
        report.status,
        hyprland_settings::future_capability::ProfileSwitchStatus::RestoreFailed
    );
    assert!(!report.production_switch_enabled);
    assert!(!report.real_config_touched);
    assert!(!report.runtime_touched);
}

#[test]
fn runtime_boundary_is_dry_run_only_and_never_executes_commands() {
    let mut executor = RuntimeDryRunExecutor::default();
    let reload = executor.evaluate(RuntimeAction::Reload);
    let status = executor.evaluate(RuntimeAction::Status {
        query: "version".to_string(),
    });

    assert!(reload.would_mutate_runtime);
    assert!(!reload.accepted_by_allowlist);
    assert!(!reload.real_command_executed);
    assert!(!reload.production_runtime_enabled);
    assert!(!status.would_mutate_runtime);
    assert!(status.accepted_by_allowlist);
    assert!(!status.real_command_executed);
    assert_eq!(executor.recorded_actions.len(), 2);

    let reload_policy = runtime_action_policy(RuntimeAction::Reload);
    assert!(!reload_policy.allowlisted_for_real_execution);
    assert!(reload_policy.dry_run_allowed);
    assert!(!reload_policy.production_runtime_enabled);
    assert!(reload_policy.reason.contains("disabled"));
}

#[test]
fn hyprland_0554_migration_assessment_keeps_0552_default_without_trusted_export() {
    let assessment = assess_hyprland_version_migration("0.55.4", false);

    assert_eq!(assessment.current_default_version, "0.55.2");
    assert_eq!(assessment.requested_version, "0.55.4");
    assert!(!assessment.migration_activated);
    assert!(!assessment.production_default_changed);
    assert!(assessment
        .blockers
        .iter()
        .any(|blocker| blocker.contains("trusted official export")));

    let bundle = current_v0552_data_bundle();
    assert_eq!(bundle.version, "0.55.2");
    assert_eq!(bundle.readable_rows, 341);
    assert_eq!(bundle.writable_rows, 341);
    assert_eq!(bundle.blocked_rows, 0);
    assert!(bundle.default_model);

    let review = disabled_migration_review("0.55.4");
    assert_eq!(review.current_default.version, "0.55.2");
    assert!(!review.migration_enabled);
    assert!(review
        .review_lines
        .iter()
        .any(|line| line.contains("newer runtime package is not enough")));
}
