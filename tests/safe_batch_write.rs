use std::collections::{BTreeMap, BTreeSet};
use std::fs;
use std::path::{Path, PathBuf};
use std::time::{SystemTime, UNIX_EPOCH};

use hyprland_settings::config_graph::{
    ConfigDetectionConfidence, ConfigGraphFile, ConfigGraphSummary, ConfigManagementHint,
    ConfigManagementHintKind,
};
use hyprland_settings::current_config::{
    CurrentConfigLoadStatus, CurrentConfigSnapshot, CurrentValue, CurrentValueStatus,
};
use hyprland_settings::safe_batch_write::{
    build_safe_batch_write_plan, execute_safe_batch_write_plan, safe_batch_write_user_facing_lines,
    SafeBatchChangeRequest, SafeBatchEligibility, SafeBatchExecutionOptions, SafeBatchWriteStatus,
};
use hyprland_settings::write_classification::SAFE_WRITABLE_ROWS;

fn temp_root(label: &str) -> PathBuf {
    let stamp = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .expect("time should work")
        .as_nanos();
    let root = std::env::temp_dir().join(format!(
        "hyprland-settings-safe-batch-{label}-{}-{stamp}",
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

fn known_settings() -> BTreeSet<String> {
    SAFE_WRITABLE_ROWS
        .iter()
        .map(|row| row.row_id.to_string())
        .collect()
}

fn graph_for(files: Vec<ConfigGraphFile>, root_path: PathBuf) -> ConfigGraphSummary {
    ConfigGraphSummary {
        root_path,
        connected_file_count: files.len(),
        unreadable_file_count: 0,
        multi_file: files.len() > 1,
        has_profile_hints: false,
        has_mode_hints: false,
        has_theme_hints: false,
        has_generated_hints: files.iter().any(|file| {
            file.hints
                .iter()
                .any(|hint| hint.kind == ConfigManagementHintKind::GeneratedFile)
        }),
        has_script_managed_hints: files.iter().any(|file| {
            file.hints.iter().any(|hint| {
                matches!(
                    hint.kind,
                    ConfigManagementHintKind::ScriptManaged
                        | ConfigManagementHintKind::ScriptReferenced
                        | ConfigManagementHintKind::SymlinkManaged
                )
            })
        }),
        files,
        source_references: Vec::new(),
        unreadable_files: Vec::new(),
        cycles: Vec::new(),
        unsupported_sources: Vec::new(),
    }
}

fn graph_file(path: &Path, hints: Vec<ConfigManagementHintKind>) -> ConfigGraphFile {
    ConfigGraphFile {
        path: path.to_path_buf(),
        resolved_path: fs::canonicalize(path).ok(),
        source_depth: 0,
        readable: true,
        is_symlink: hints.contains(&ConfigManagementHintKind::SymlinkManaged),
        symlink_target: None,
        hints: hints
            .into_iter()
            .map(|kind| ConfigManagementHint {
                kind,
                confidence: ConfigDetectionConfidence::Confirmed,
                evidence: "fixture metadata".to_string(),
            })
            .collect(),
    }
}

fn snapshot(
    values: Vec<(&str, &str, &Path, usize, &str, CurrentValueStatus)>,
) -> CurrentConfigSnapshot {
    let mut map = BTreeMap::new();
    for (setting_id, raw_value, source_path, line_number, raw_line, status) in values {
        map.insert(
            setting_id.to_string(),
            CurrentValue {
                setting_id: setting_id.to_string(),
                raw_value: raw_value.to_string(),
                source_path: source_path.to_path_buf(),
                line_number,
                raw_line: raw_line.to_string(),
                duplicate_lines: matches!(status, CurrentValueStatus::DuplicateConflict)
                    .then_some(vec![line_number, line_number + 1])
                    .unwrap_or_default(),
                status,
                warning: None,
            },
        );
    }
    CurrentConfigSnapshot {
        status: CurrentConfigLoadStatus::Loaded {
            path: PathBuf::from("/tmp/safe-batch-fixture.conf"),
            scalar_count: map.len(),
            structured_count: 0,
            unsupported_count: 0,
        },
        values: map,
        structured_records: Vec::new(),
        unsupported_records: Vec::new(),
    }
}

fn plan_for(
    current_config: &CurrentConfigSnapshot,
    graph: &ConfigGraphSummary,
    changes: Vec<SafeBatchChangeRequest>,
) -> hyprland_settings::safe_batch_write::SafeBatchWritePlan {
    build_safe_batch_write_plan(
        "test-safe-batch",
        &known_settings(),
        current_config,
        graph,
        changes,
        "fixture-timestamp",
    )
}

#[test]
fn safe_batch_with_two_normal_scalar_changes_in_one_file_succeeds() {
    let root = temp_root("one-file-success");
    let config = root.join("hyprland.conf");
    write_file(
        &config,
        "decoration:blur:enabled = true\ndecoration:shadow:enabled = false\n",
    );
    let current = snapshot(vec![
        (
            "decoration.blur.enabled",
            "true",
            &config,
            1,
            "decoration:blur:enabled = true",
            CurrentValueStatus::Configured,
        ),
        (
            "decoration.shadow.enabled",
            "false",
            &config,
            2,
            "decoration:shadow:enabled = false",
            CurrentValueStatus::Configured,
        ),
    ]);
    let graph = graph_for(vec![graph_file(&config, Vec::new())], config.clone());
    let plan = plan_for(
        &current,
        &graph,
        vec![
            SafeBatchChangeRequest::new("appearance.blur.enabled", "false"),
            SafeBatchChangeRequest::new("appearance.shadow.enabled", "true"),
        ],
    );

    assert!(plan.can_execute);
    assert_eq!(plan.eligible_changes.len(), 2);
    assert_eq!(plan.target_files.len(), 1);

    let report = execute_safe_batch_write_plan(
        &plan,
        &SafeBatchExecutionOptions {
            backup_timestamp: "one-file".to_string(),
            ..SafeBatchExecutionOptions::default()
        },
    );

    assert_eq!(report.status, SafeBatchWriteStatus::Succeeded);
    assert_eq!(report.backups.len(), 1);
    assert!(report.backups[0].bytes_equal);
    assert!(!report.hyprland_reload_attempted);
    assert!(!report.mutating_hyprctl_used);
    assert!(!report.runtime_mutated);
    let updated = fs::read_to_string(&config).expect("updated config should read");
    assert!(updated.contains("decoration:blur:enabled = false"));
    assert!(updated.contains("decoration:shadow:enabled = true"));
}

#[test]
fn safe_batch_with_normal_scalar_changes_across_two_files_succeeds() {
    let root = temp_root("two-file-success");
    let primary = root.join("hyprland.conf");
    let included = root.join("appearance.conf");
    write_file(&primary, "decoration:blur:enabled = true\n");
    write_file(&included, "decoration:shadow:enabled = false\n");
    let current = snapshot(vec![
        (
            "decoration.blur.enabled",
            "true",
            &primary,
            1,
            "decoration:blur:enabled = true",
            CurrentValueStatus::Configured,
        ),
        (
            "decoration.shadow.enabled",
            "false",
            &included,
            1,
            "decoration:shadow:enabled = false",
            CurrentValueStatus::Configured,
        ),
    ]);
    let graph = graph_for(
        vec![
            graph_file(&primary, Vec::new()),
            graph_file(&included, Vec::new()),
        ],
        primary.clone(),
    );
    let plan = plan_for(
        &current,
        &graph,
        vec![
            SafeBatchChangeRequest::new("appearance.blur.enabled", "false"),
            SafeBatchChangeRequest::new("appearance.shadow.enabled", "true"),
        ],
    );

    assert!(plan.can_execute);
    assert_eq!(plan.target_files.len(), 2);
    let report = execute_safe_batch_write_plan(
        &plan,
        &SafeBatchExecutionOptions {
            backup_timestamp: "two-file".to_string(),
            ..SafeBatchExecutionOptions::default()
        },
    );

    assert_eq!(report.status, SafeBatchWriteStatus::Succeeded);
    assert_eq!(report.backups.len(), 2);
    assert!(fs::read_to_string(&primary).unwrap().contains("false"));
    assert!(fs::read_to_string(&included).unwrap().contains("true"));
}

#[test]
fn safe_batch_inserts_missing_default_normal_scalar_into_explicit_root_file() {
    let root = temp_root("missing-insertion-success");
    let config = root.join("hyprland.conf");
    write_file(&config, "decoration:blur:enabled = true\n");
    let current = snapshot(vec![(
        "decoration.blur.enabled",
        "true",
        &config,
        1,
        "decoration:blur:enabled = true",
        CurrentValueStatus::Configured,
    )]);
    let graph = graph_for(vec![graph_file(&config, Vec::new())], config.clone());
    let plan = plan_for(
        &current,
        &graph,
        vec![SafeBatchChangeRequest::new(
            "misc.disable_splash_rendering",
            "true",
        )],
    );

    assert!(plan.can_execute, "{:?}", plan.cannot_execute_reasons);
    assert!(plan.eligible_changes.is_empty());
    assert_eq!(plan.insertion_changes.len(), 1);
    assert_eq!(plan.target_files.len(), 1);
    assert_eq!(
        plan.insertion_changes[0].insertion_line,
        "misc:disable_splash_rendering = true"
    );
    assert!(plan.insertion_changes[0]
        .review_copy
        .contains("insert this missing/default setting"));

    let report = execute_safe_batch_write_plan(
        &plan,
        &SafeBatchExecutionOptions {
            backup_timestamp: "missing-insertion".to_string(),
            ..SafeBatchExecutionOptions::default()
        },
    );

    assert_eq!(report.status, SafeBatchWriteStatus::Succeeded);
    assert_eq!(
        report.verified_changes,
        vec!["misc.disable_splash_rendering".to_string()]
    );
    assert_eq!(report.backups.len(), 1);
    assert!(report.backups[0].bytes_equal);
    assert!(!report.hyprland_reload_attempted);
    assert!(!report.mutating_hyprctl_used);
    assert!(!report.runtime_mutated);
    let updated = fs::read_to_string(&config).expect("updated config should read");
    assert!(updated.contains("# Added by Hyprland Settings safe-batch missing/default insertion"));
    assert!(updated.contains("misc:disable_splash_rendering = true"));
}

#[test]
fn safe_batch_missing_default_insertion_restores_after_verification_failure() {
    let root = temp_root("missing-insertion-restore");
    let config = root.join("hyprland.conf");
    let original = "decoration:blur:enabled = true\n";
    write_file(&config, original);
    let current = snapshot(vec![(
        "decoration.blur.enabled",
        "true",
        &config,
        1,
        "decoration:blur:enabled = true",
        CurrentValueStatus::Configured,
    )]);
    let graph = graph_for(vec![graph_file(&config, Vec::new())], config.clone());
    let plan = plan_for(
        &current,
        &graph,
        vec![SafeBatchChangeRequest::new(
            "misc.disable_splash_rendering",
            "true",
        )],
    );

    let report = execute_safe_batch_write_plan(
        &plan,
        &SafeBatchExecutionOptions {
            backup_timestamp: "missing-verify-failure".to_string(),
            force_verification_failure_for: Some("misc.disable_splash_rendering".to_string()),
            ..SafeBatchExecutionOptions::default()
        },
    );

    assert_eq!(report.status, SafeBatchWriteStatus::RecoveredFailure);
    assert!(report.recovery_attempted);
    assert!(report.recovery_succeeded);
    assert!(report.restore_verification_succeeded);
    assert_eq!(
        fs::read_to_string(&config).expect("config should read"),
        original
    );
}

#[test]
fn safe_batch_missing_default_insertion_blocks_managed_duplicate_and_ambiguous_targets() {
    let root = temp_root("missing-insertion-blocked");
    let normal = root.join("hyprland.conf");
    let generated = root.join("generated.conf");
    let script = root.join("script.conf");
    let symlink = root.join("current.conf");
    write_file(&normal, "decoration:blur:enabled = true\n");
    write_file(&generated, "decoration:blur:enabled = true\n");
    write_file(&script, "decoration:blur:enabled = true\n");
    write_file(&symlink, "decoration:blur:enabled = true\n");
    let current = snapshot(vec![(
        "decoration.blur.enabled",
        "true",
        &normal,
        1,
        "decoration:blur:enabled = true",
        CurrentValueStatus::Configured,
    )]);

    for (target, hints, reason) in [
        (
            generated.clone(),
            vec![ConfigManagementHintKind::GeneratedFile],
            SafeBatchEligibility::BlockedGeneratedFile,
        ),
        (
            script.clone(),
            vec![ConfigManagementHintKind::ScriptManaged],
            SafeBatchEligibility::BlockedScriptManaged,
        ),
        (
            symlink.clone(),
            vec![ConfigManagementHintKind::SymlinkManaged],
            SafeBatchEligibility::BlockedSymlinkManaged,
        ),
    ] {
        let graph = graph_for(vec![graph_file(&target, hints)], target.clone());
        let plan = plan_for(
            &current,
            &graph,
            vec![SafeBatchChangeRequest::new(
                "misc.disable_splash_rendering",
                "true",
            )],
        );
        assert!(!plan.can_execute);
        assert!(plan.insertion_changes.is_empty());
        assert!(plan
            .blocked_changes
            .iter()
            .any(|change| change.reason == reason));
    }

    let duplicate_config = root.join("duplicate.conf");
    write_file(&duplicate_config, "misc:disable_splash_rendering = false\n");
    let duplicate_graph = graph_for(
        vec![graph_file(&duplicate_config, Vec::new())],
        duplicate_config,
    );
    let duplicate_plan = plan_for(
        &current,
        &duplicate_graph,
        vec![SafeBatchChangeRequest::new(
            "misc.disable_splash_rendering",
            "true",
        )],
    );
    assert!(!duplicate_plan.can_execute);
    assert!(duplicate_plan.insertion_changes.is_empty());
    assert!(duplicate_plan
        .blocked_changes
        .iter()
        .any(|change| change.reason == SafeBatchEligibility::BlockedDuplicateConflict));

    let included = root.join("appearance.conf");
    write_file(&included, "decoration:shadow:enabled = false\n");
    let ambiguous_graph = graph_for(
        vec![
            graph_file(&normal, Vec::new()),
            graph_file(&included, Vec::new()),
        ],
        normal.clone(),
    );
    let ambiguous_plan = plan_for(
        &current,
        &ambiguous_graph,
        vec![SafeBatchChangeRequest::new(
            "misc.disable_splash_rendering",
            "true",
        )],
    );
    assert!(!ambiguous_plan.can_execute);
    assert!(ambiguous_plan
        .blocked_changes
        .iter()
        .any(|change| change.reason == SafeBatchEligibility::BlockedMissingLine));
}

#[test]
fn blocked_categories_prevent_batch_execution_and_partial_apply_by_default() {
    let root = temp_root("blocked");
    let normal = root.join("normal.conf");
    let generated = root.join("generated.conf");
    let script = root.join("script.conf");
    let symlink_managed = root.join("current.conf");
    write_file(&normal, "decoration:blur:enabled = true\n");
    write_file(&generated, "decoration:shadow:enabled = false\n");
    write_file(&script, "decoration:blur:ignore_opacity = true\n");
    write_file(
        &symlink_managed,
        "decoration:blur:new_optimizations = true\n",
    );
    let current = snapshot(vec![
        (
            "decoration.blur.enabled",
            "true",
            &normal,
            1,
            "decoration:blur:enabled = true",
            CurrentValueStatus::Configured,
        ),
        (
            "decoration.shadow.enabled",
            "false",
            &generated,
            1,
            "decoration:shadow:enabled = false",
            CurrentValueStatus::Configured,
        ),
        (
            "decoration.blur.ignore_opacity",
            "true",
            &script,
            1,
            "decoration:blur:ignore_opacity = true",
            CurrentValueStatus::Configured,
        ),
        (
            "decoration.blur.new_optimizations",
            "true",
            &symlink_managed,
            1,
            "decoration:blur:new_optimizations = true",
            CurrentValueStatus::Configured,
        ),
        (
            "decoration.blur.popups",
            "false",
            &normal,
            2,
            "decoration:blur:popups = false",
            CurrentValueStatus::DuplicateConflict,
        ),
    ]);
    let graph = graph_for(
        vec![
            graph_file(&normal, Vec::new()),
            graph_file(&generated, vec![ConfigManagementHintKind::GeneratedFile]),
            graph_file(&script, vec![ConfigManagementHintKind::ScriptManaged]),
            graph_file(
                &symlink_managed,
                vec![ConfigManagementHintKind::SymlinkManaged],
            ),
        ],
        normal.clone(),
    );
    let plan = plan_for(
        &current,
        &graph,
        vec![
            SafeBatchChangeRequest::new("appearance.blur.enabled", "false"),
            SafeBatchChangeRequest::new("render.direct_scanout", "1"),
            SafeBatchChangeRequest::new("cursor.invisible", "1"),
            SafeBatchChangeRequest::new("appearance.shadow.enabled", "true"),
            SafeBatchChangeRequest::new("appearance.blur.ignore_opacity", "false"),
            SafeBatchChangeRequest::new("appearance.blur.new_optimizations", "false"),
            SafeBatchChangeRequest::new("appearance.blur.popups", "true"),
            SafeBatchChangeRequest::new("appearance.blur.special", "true"),
            SafeBatchChangeRequest::new("hl.monitor", "DP-1,preferred,auto,1"),
            SafeBatchChangeRequest::new("runtime.pointer", "on"),
            SafeBatchChangeRequest::new("profile.mode_switch", "gaming"),
        ],
    );

    assert!(!plan.can_execute);
    assert!(plan
        .blocked_changes
        .iter()
        .any(|change| change.reason == SafeBatchEligibility::BlockedDisplayRenderRisk));
    assert!(plan
        .blocked_changes
        .iter()
        .any(|change| change.reason == SafeBatchEligibility::BlockedHighRisk));
    assert!(plan
        .blocked_changes
        .iter()
        .any(|change| change.reason == SafeBatchEligibility::BlockedGeneratedFile));
    assert!(plan
        .blocked_changes
        .iter()
        .any(|change| change.reason == SafeBatchEligibility::BlockedScriptManaged));
    assert!(plan
        .blocked_changes
        .iter()
        .any(|change| change.reason == SafeBatchEligibility::BlockedSymlinkManaged));
    assert!(plan
        .blocked_changes
        .iter()
        .any(|change| change.reason == SafeBatchEligibility::BlockedDuplicateConflict));
    assert!(plan
        .blocked_changes
        .iter()
        .any(|change| change.reason == SafeBatchEligibility::BlockedMissingLine));
    assert!(plan
        .blocked_changes
        .iter()
        .any(|change| change.reason == SafeBatchEligibility::BlockedStructuredFamily));
    assert!(plan
        .blocked_changes
        .iter()
        .any(|change| change.reason == SafeBatchEligibility::BlockedRuntimeOnly));
    assert!(plan
        .blocked_changes
        .iter()
        .any(|change| change.reason == SafeBatchEligibility::BlockedProfileModeSwitch));

    let report = execute_safe_batch_write_plan(&plan, &SafeBatchExecutionOptions::default());
    assert_eq!(report.status, SafeBatchWriteStatus::Blocked);
    assert_eq!(
        fs::read_to_string(&normal).expect("normal fixture should read"),
        "decoration:blur:enabled = true\n"
    );
}

#[test]
fn write_failure_and_verification_failure_restore_all_touched_files() {
    let root = temp_root("recovery");
    let first = root.join("first.conf");
    let second = root.join("second.conf");
    write_file(&first, "decoration:blur:enabled = true\n");
    write_file(&second, "decoration:shadow:enabled = false\n");
    let current = snapshot(vec![
        (
            "decoration.blur.enabled",
            "true",
            &first,
            1,
            "decoration:blur:enabled = true",
            CurrentValueStatus::Configured,
        ),
        (
            "decoration.shadow.enabled",
            "false",
            &second,
            1,
            "decoration:shadow:enabled = false",
            CurrentValueStatus::Configured,
        ),
    ]);
    let graph = graph_for(
        vec![
            graph_file(&first, Vec::new()),
            graph_file(&second, Vec::new()),
        ],
        first.clone(),
    );
    let plan = plan_for(
        &current,
        &graph,
        vec![
            SafeBatchChangeRequest::new("appearance.blur.enabled", "false"),
            SafeBatchChangeRequest::new("appearance.shadow.enabled", "true"),
        ],
    );

    let write_failure = execute_safe_batch_write_plan(
        &plan,
        &SafeBatchExecutionOptions {
            backup_timestamp: "write-failure".to_string(),
            fail_after_writing_target: Some(first.clone()),
            ..SafeBatchExecutionOptions::default()
        },
    );
    assert_eq!(write_failure.status, SafeBatchWriteStatus::RecoveredFailure);
    assert!(write_failure.recovery_attempted);
    assert!(write_failure.recovery_succeeded);
    assert_eq!(
        fs::read_to_string(&first).unwrap(),
        "decoration:blur:enabled = true\n"
    );
    assert_eq!(
        fs::read_to_string(&second).unwrap(),
        "decoration:shadow:enabled = false\n"
    );

    let verification_failure = execute_safe_batch_write_plan(
        &plan,
        &SafeBatchExecutionOptions {
            backup_timestamp: "verification-failure".to_string(),
            force_verification_failure_for: Some("appearance.shadow.enabled".to_string()),
            ..SafeBatchExecutionOptions::default()
        },
    );
    assert_eq!(
        verification_failure.status,
        SafeBatchWriteStatus::RecoveredFailure
    );
    assert!(verification_failure.restore_verification_succeeded);
    assert_eq!(
        fs::read_to_string(&first).unwrap(),
        "decoration:blur:enabled = true\n"
    );
    assert_eq!(
        fs::read_to_string(&second).unwrap(),
        "decoration:shadow:enabled = false\n"
    );
}

#[test]
fn backup_verification_failure_blocks_before_write_and_restore_failure_is_reported() {
    let root = temp_root("failure-reporting");
    let config = root.join("hyprland.conf");
    write_file(&config, "decoration:blur:enabled = true\n");
    let current = snapshot(vec![(
        "decoration.blur.enabled",
        "true",
        &config,
        1,
        "decoration:blur:enabled = true",
        CurrentValueStatus::Configured,
    )]);
    let graph = graph_for(vec![graph_file(&config, Vec::new())], config.clone());
    let plan = plan_for(
        &current,
        &graph,
        vec![SafeBatchChangeRequest::new(
            "appearance.blur.enabled",
            "false",
        )],
    );

    let backup_failure = execute_safe_batch_write_plan(
        &plan,
        &SafeBatchExecutionOptions {
            backup_timestamp: "backup-failure".to_string(),
            force_backup_verification_failure_for: Some(config.clone()),
            ..SafeBatchExecutionOptions::default()
        },
    );
    assert_eq!(backup_failure.status, SafeBatchWriteStatus::Blocked);
    assert!(!backup_failure.recovery_attempted);
    assert_eq!(
        fs::read_to_string(&config).unwrap(),
        "decoration:blur:enabled = true\n"
    );

    let restore_failure = execute_safe_batch_write_plan(
        &plan,
        &SafeBatchExecutionOptions {
            backup_timestamp: "restore-failure".to_string(),
            force_verification_failure_for: Some("appearance.blur.enabled".to_string()),
            force_restore_failure: true,
            ..SafeBatchExecutionOptions::default()
        },
    );
    assert_eq!(
        restore_failure.status,
        SafeBatchWriteStatus::UnrecoveredFailure
    );
    assert!(restore_failure.recovery_attempted);
    assert!(!restore_failure.recovery_succeeded);
    assert!(restore_failure
        .failures
        .iter()
        .any(|failure| failure.contains("forced restore failure")));
}

#[test]
fn safe_batch_user_copy_uses_batch_language_and_preserves_safe_writable_count() {
    let lines = safe_batch_write_user_facing_lines();
    assert!(lines
        .iter()
        .any(|line| line == "Safe batch write is available for normal settings."));
    assert!(lines
        .iter()
        .any(|line| line == "The app will back up files before writing."));
    assert!(lines
        .iter()
        .any(|line| line == "The app will check the result after writing."));
    assert!(lines
        .iter()
        .any(|line| line == "If something fails, the app will restore the backup."));
    assert!(!lines.iter().any(|line| line.contains("one-target")));
    assert_eq!(SAFE_WRITABLE_ROWS.len(), 341);
}
