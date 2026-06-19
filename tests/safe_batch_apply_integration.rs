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
use hyprland_settings::safe_batch_write::{SafeBatchChangeRequest, SafeBatchExecutionOptions};
use hyprland_settings::write_classification::SAFE_WRITABLE_ROWS;
use hyprland_settings::write_flow::apply_safe_batch_setting_changes_with_graph_and_options;

fn temp_root(label: &str) -> PathBuf {
    let stamp = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .expect("time should work")
        .as_nanos();
    let root = std::env::temp_dir().join(format!(
        "hyprland-settings-safe-batch-apply-{label}-{}-{stamp}",
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

fn graph_for(files: Vec<ConfigGraphFile>, root_path: PathBuf) -> ConfigGraphSummary {
    ConfigGraphSummary {
        root_path,
        connected_file_count: files.len(),
        unreadable_file_count: 0,
        multi_file: files.len() > 1,
        has_profile_hints: false,
        has_mode_hints: false,
        has_theme_hints: false,
        has_generated_hints: false,
        has_script_managed_hints: false,
        files,
        source_references: Vec::new(),
        unreadable_files: Vec::new(),
        cycles: Vec::new(),
        unsupported_sources: Vec::new(),
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
            path: PathBuf::from("/tmp/safe-batch-apply-fixture.conf"),
            scalar_count: map.len(),
            structured_count: 0,
            unsupported_count: 0,
        },
        values: map,
        structured_records: Vec::new(),
        unsupported_records: Vec::new(),
    }
}

#[test]
fn apply_can_write_only_when_safe_batch_plan_is_executable() {
    let root = temp_root("success");
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

    let report = apply_safe_batch_setting_changes_with_graph_and_options(
        known_settings(),
        &current,
        &graph,
        vec![
            SafeBatchChangeRequest::new("appearance.blur.enabled", "false"),
            SafeBatchChangeRequest::new("appearance.shadow.enabled", "true"),
        ],
        SafeBatchExecutionOptions {
            backup_timestamp: "apply-success".to_string(),
            ..SafeBatchExecutionOptions::default()
        },
    )
    .expect("eligible batch should apply");

    assert_eq!(report.verified_changes.len(), 2);
    assert!(fs::read_to_string(&config).unwrap().contains("false"));
    assert!(!report.hyprland_reload_attempted);
    assert!(!report.mutating_hyprctl_used);
    assert!(!report.runtime_mutated);
}

#[test]
fn apply_blocks_high_risk_generated_ambiguous_missing_line_and_structured_targets() {
    let root = temp_root("blocked");
    let config = root.join("hyprland.conf");
    let generated = root.join("generated.conf");
    write_file(&config, "decoration:blur:enabled = true\n");
    write_file(&generated, "decoration:shadow:enabled = false\n");
    let current = snapshot(vec![
        (
            "decoration.blur.enabled",
            "true",
            &config,
            1,
            "decoration:blur:enabled = true",
            CurrentValueStatus::DuplicateConflict,
        ),
        (
            "decoration.shadow.enabled",
            "false",
            &generated,
            1,
            "decoration:shadow:enabled = false",
            CurrentValueStatus::Configured,
        ),
    ]);
    let graph = graph_for(
        vec![
            graph_file(&config, Vec::new()),
            graph_file(&generated, vec![ConfigManagementHintKind::GeneratedFile]),
        ],
        config.clone(),
    );

    let failure = apply_safe_batch_setting_changes_with_graph_and_options(
        known_settings(),
        &current,
        &graph,
        vec![
            SafeBatchChangeRequest::new("render.direct_scanout", "1"),
            SafeBatchChangeRequest::new("cursor.invisible", "1"),
            SafeBatchChangeRequest::new("appearance.shadow.enabled", "true"),
            SafeBatchChangeRequest::new("appearance.blur.enabled", "false"),
            SafeBatchChangeRequest::new("appearance.blur.special", "true"),
            SafeBatchChangeRequest::new("hl.monitor", "DP-1,preferred,auto,1"),
        ],
        SafeBatchExecutionOptions::default(),
    )
    .expect_err("blocked categories must reject apply");

    assert!(failure
        .failures
        .iter()
        .any(|failure| failure.contains("blocked_display_render_risk")));
    assert!(failure
        .failures
        .iter()
        .any(|failure| failure.contains("blocked_high_risk")));
    assert!(failure
        .failures
        .iter()
        .any(|failure| failure.contains("blocked_generated_file")));
    assert!(failure
        .failures
        .iter()
        .any(|failure| failure.contains("blocked_duplicate_conflict")));
    assert!(failure
        .failures
        .iter()
        .any(|failure| failure.contains("blocked_missing_line")));
    assert!(failure
        .failures
        .iter()
        .any(|failure| failure.contains("blocked_structured_family")));
    assert_eq!(
        fs::read_to_string(&config).unwrap(),
        "decoration:blur:enabled = true\n"
    );
}

#[test]
fn apply_blocks_backup_failure_and_does_not_hide_restore_failure() {
    let root = temp_root("proof-failure");
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

    let backup_failure = apply_safe_batch_setting_changes_with_graph_and_options(
        known_settings(),
        &current,
        &graph,
        vec![SafeBatchChangeRequest::new(
            "appearance.blur.enabled",
            "false",
        )],
        SafeBatchExecutionOptions {
            backup_timestamp: "backup-proof-failure".to_string(),
            force_backup_verification_failure_for: Some(config.clone()),
            ..SafeBatchExecutionOptions::default()
        },
    )
    .expect_err("backup proof failure must block apply");
    assert!(backup_failure
        .failures
        .iter()
        .any(|failure| failure.contains("backup byte equality verification failed")));

    let restore_failure = apply_safe_batch_setting_changes_with_graph_and_options(
        known_settings(),
        &current,
        &graph,
        vec![SafeBatchChangeRequest::new(
            "appearance.blur.enabled",
            "false",
        )],
        SafeBatchExecutionOptions {
            backup_timestamp: "restore-proof-failure".to_string(),
            force_verification_failure_for: Some("appearance.blur.enabled".to_string()),
            force_restore_failure: true,
            ..SafeBatchExecutionOptions::default()
        },
    )
    .expect_err("restore failure must be reported");
    assert!(restore_failure
        .failures
        .iter()
        .any(|failure| failure.contains("forced restore failure")));
}
