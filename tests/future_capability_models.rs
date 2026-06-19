use std::fs;
use std::path::{Path, PathBuf};
use std::time::{SystemTime, UNIX_EPOCH};

use hyprland_settings::future_capability::{
    assess_hyprland_version_migration, current_v0552_data_bundle, disabled_migration_review,
    disabled_missing_default_insertion_review, disabled_profile_switch_review,
    disabled_profile_switch_selection_review, duplicate_occurrence_model,
    duplicate_occurrence_review, high_risk_recovery_review, high_risk_recovery_workflow,
    migration_comparison_review, replace_duplicate_occurrence_safe_env, runtime_action_policy,
    runtime_action_review, structured_family_model, structured_family_review,
    switch_profile_symlink_safe_env, validate_structured_edit_candidate,
    DuplicateOccurrenceReviewState, DuplicateReplacementOptions, DuplicateReplacementRequest,
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
fn duplicate_occurrence_review_tracks_no_selection_and_selected_disabled_state() {
    let root = temp_root("duplicate-review");
    let root_conf = root.join("hyprland.conf");
    let sourced_conf = root.join("appearance.conf");
    write_file(&root_conf, "decoration:blur:enabled = true\n");
    write_file(&sourced_conf, "decoration:blur:enabled = false\n");
    let model = duplicate_occurrence_model(
        "decoration.blur.enabled",
        &[(root_conf.clone(), 0), (sourced_conf.clone(), 1)],
    )
    .expect("duplicate model should build");

    let no_selection = duplicate_occurrence_review(&model, None, Some("false".to_string()));
    assert_eq!(
        no_selection.state,
        DuplicateOccurrenceReviewState::NoOccurrenceSelected
    );
    assert!(!no_selection.apply_enabled);
    assert!(!no_selection.production_write_enabled);
    assert!(!no_selection.write_execution_attempted);
    assert!(no_selection
        .review_lines
        .iter()
        .any(|line| line.contains("will not auto-choose")));

    let selected = duplicate_occurrence_review(&model, Some(1), Some("true".to_string()));
    assert_eq!(
        selected.state,
        DuplicateOccurrenceReviewState::OccurrenceSelectedProductionDisabled
    );
    assert_eq!(selected.selected_path.as_ref(), Some(&sourced_conf));
    assert_eq!(selected.selected_line_number, Some(1));
    assert_eq!(
        selected.selected_raw_line.as_deref(),
        Some("decoration:blur:enabled = false")
    );
    assert_eq!(selected.selected_current_value.as_deref(), Some("false"));
    assert_eq!(selected.proposed_value.as_deref(), Some("true"));
    assert_eq!(selected.source_depth, Some(1));
    assert!(!selected.apply_enabled);
    assert!(!selected.production_write_enabled);
    assert!(!selected.write_execution_attempted);
}

#[test]
fn duplicate_occurrence_review_rejects_stale_selection_without_writing() {
    let root = temp_root("duplicate-review-invalid");
    let config = root.join("hyprland.conf");
    write_file(&config, "decoration:blur:enabled = true\n");
    let model = duplicate_occurrence_model("decoration.blur.enabled", &[(config, 0)])
        .expect("duplicate model should build");

    let review = duplicate_occurrence_review(&model, Some(99), Some("false".to_string()));

    assert_eq!(
        review.state,
        DuplicateOccurrenceReviewState::InvalidSelection
    );
    assert!(!review.apply_enabled);
    assert!(!review.production_write_enabled);
    assert!(!review.write_execution_attempted);
    assert!(review.selected_path.is_none());
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
fn high_risk_recovery_workflow_records_rollback_proof_without_runtime() {
    let pending = MockWatchdog::arm("session", "token", 10);
    let pending_workflow = high_risk_recovery_workflow("render.direct_scanout", &pending);
    assert_eq!(pending_workflow.state, MockWatchdogState::Pending);
    assert!(!pending_workflow.confirmation_enabled);
    assert!(!pending_workflow.revert_enabled);
    assert!(!pending_workflow.production_write_enabled);
    assert!(!pending_workflow.real_runtime_enabled);
    assert!(pending_workflow.rollback_proof.backup_before_write_required);
    assert!(pending_workflow.rollback_proof.reread_after_write_required);
    assert!(pending_workflow.rollback_proof.restore_on_timeout_required);
    assert!(
        pending_workflow
            .rollback_proof
            .reread_after_restore_required
    );
    assert!(!pending_workflow.rollback_proof.real_runtime_enabled);

    let mut failed = MockWatchdog::arm("session-fail", "token", 10);
    failed.tick(11, false);
    let failed_workflow = high_risk_recovery_workflow("decoration.screen_shader", &failed);
    assert_eq!(failed_workflow.state, MockWatchdogState::RecoveryFailed);
    assert!(failed_workflow
        .review_lines
        .iter()
        .any(|line| line.contains("recovery failure")));
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
fn structured_family_review_keeps_repeated_bind_entries_disabled_and_lossless() {
    let root = temp_root("structured-review");
    let config = root.join("hyprland.conf");
    write_file(
        &config,
        "# keep this comment before binds\nbind = SUPER, Return, exec, foot\nbind = SUPER, Q, killactive\n",
    );
    let model = structured_family_model(&config, "hl.bind").expect("model should build");

    let review =
        structured_family_review(&model, Some("bind = SUPER, Space, exec, wofi --show drun"));

    assert_eq!(review.family_id, "hl.bind");
    assert_eq!(review.entries.len(), 2);
    assert!(review.proposed_edit.as_ref().expect("candidate").accepted);
    assert!(!review.editor_enabled);
    assert!(!review.production_write_enabled);
    assert!(review.raw_line_preservation_required);
    assert!(review.comments_order_preservation_required);
    assert_eq!(
        review.first_safe_env_write_candidate.as_deref(),
        Some("hl.bind single-line replacement after lossless render proof")
    );

    let invalid = structured_family_review(&model, Some("monitor = eDP-1,preferred,auto,1"));
    assert!(!invalid.proposed_edit.as_ref().expect("candidate").accepted);
    assert!(invalid
        .invalid_input_reasons
        .iter()
        .any(|reason| reason.contains("must start with bind")));
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

#[cfg(unix)]
#[test]
fn profile_switch_selection_review_tracks_selected_target_but_keeps_disabled() {
    let root = temp_root("profile-selection-review");
    let symlink = root.join("profiles/current.conf");
    let desktop = root.join("profiles/desktop.conf");
    let gaming = root.join("profiles/gaming.conf");

    let no_selection =
        disabled_profile_switch_selection_review(&symlink, Some(desktop.clone()), None, None);
    assert!(no_selection.selected_target_profile.is_none());
    assert!(!no_selection.confirmation_enabled);
    assert!(!no_selection.production_switch_enabled);
    assert!(!no_selection.reload_after_switch_enabled);
    assert!(no_selection
        .review_lines
        .iter()
        .any(|line| line.contains("No target profile")));

    let selected = disabled_profile_switch_selection_review(
        &symlink,
        Some(desktop.clone()),
        Some(desktop.clone()),
        Some(gaming.clone()),
    );
    assert_eq!(selected.symlink_path, symlink);
    assert_eq!(selected.current_profile.as_ref(), Some(&desktop));
    assert_eq!(selected.resolved_current_target.as_ref(), Some(&desktop));
    assert_eq!(selected.selected_target_profile.as_ref(), Some(&gaming));
    assert!(!selected.confirmation_enabled);
    assert!(!selected.production_switch_enabled);
    assert!(!selected.reload_after_switch_enabled);
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
fn runtime_action_review_records_policy_and_log_without_execution() {
    let reload_review = runtime_action_review(RuntimeAction::Reload);
    assert!(!reload_review.policy.allowlisted_for_real_execution);
    assert!(reload_review.policy.dry_run_allowed);
    assert!(reload_review.dry_run_result.would_mutate_runtime);
    assert!(!reload_review.production_execution_enabled);
    assert!(!reload_review.real_command_executed);
    assert!(!reload_review.dry_run_result.real_command_executed);
    assert_eq!(reload_review.execution_log.len(), 1);

    let status_review = runtime_action_review(RuntimeAction::Status {
        query: "version".to_string(),
    });
    assert!(!status_review.policy.allowlisted_for_real_execution);
    assert!(status_review.policy.dry_run_allowed);
    assert!(!status_review.dry_run_result.would_mutate_runtime);
    assert!(!status_review.production_execution_enabled);
    assert!(!status_review.real_command_executed);
    assert!(!status_review.dry_run_result.real_command_executed);
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

#[test]
fn migration_comparison_review_keeps_0552_default_until_trusted_export_proof_exists() {
    let blocked = migration_comparison_review("0.55.4", false);
    assert_eq!(blocked.current_default.version, "0.55.2");
    assert_eq!(blocked.requested_version, "0.55.4");
    assert!(blocked.requested_bundle.is_none());
    assert!(!blocked.trusted_source_requirement_met);
    assert!(!blocked.migration_enabled);
    assert!(!blocked.production_default_changed);
    assert!(blocked
        .missing_proof
        .iter()
        .any(|proof| proof.contains("trusted official export")));
    assert!(blocked
        .missing_proof
        .iter()
        .any(|proof| proof.contains("GTK safe-env evidence")));

    let current = migration_comparison_review("0.55.2", true);
    assert_eq!(current.current_default.version, "0.55.2");
    assert!(current.requested_bundle.is_some());
    assert!(current.trusted_source_requirement_met);
    assert!(!current.migration_enabled);
    assert!(!current.production_default_changed);
    assert!(current.missing_proof.is_empty());
}
