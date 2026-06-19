use std::collections::BTreeSet;
use std::fs;
use std::path::{Path, PathBuf};
use std::time::{SystemTime, UNIX_EPOCH};

use hyprland_settings::config_graph::{
    inspect_config_graph, inspect_config_graph_with_options, ConfigGraphOptions, SourceFollowPolicy,
};
use hyprland_settings::current_config::CurrentValueSourceStatus;
use hyprland_settings::safe_batch_write::{
    build_safe_batch_write_plan, SafeBatchChangeRequest, SafeBatchEligibility,
};
use hyprland_settings::source_aware_current_config::{
    current_config_from_graph, source_aware_mapping_report,
};
use hyprland_settings::write_classification::SAFE_WRITABLE_ROWS;

fn temp_root(label: &str) -> PathBuf {
    let stamp = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .expect("time should work")
        .as_nanos();
    let root = std::env::temp_dir().join(format!(
        "hyprland-settings-source-aware-{label}-{}-{stamp}",
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

#[test]
fn source_aware_mapping_maps_root_sourced_nested_relative_and_tilde_files() {
    let root = temp_root("mapping");
    let home = root.join("home");
    let config_dir = home.join(".config/hypr");
    let main = config_dir.join("hyprland.conf");
    let sourced = config_dir.join("sourced.conf");
    let nested = config_dir.join("nested.conf");
    let tilde = home.join("tilde.conf");
    write_file(
        &main,
        "source = sourced.conf\nsource = ~/tilde.conf\ndecoration:shadow:enabled = false\n",
    );
    write_file(
        &sourced,
        "source = nested.conf\ndecoration:blur:enabled = true\n",
    );
    write_file(&nested, "animations:enabled = true\n");
    write_file(&tilde, "misc:disable_splash_rendering = false\n");

    let graph = inspect_config_graph_with_options(
        &main,
        ConfigGraphOptions {
            home_dir: Some(home.clone()),
            script_dirs: Vec::new(),
            max_depth: 16,
            source_follow_policy: SourceFollowPolicy::ReviewAll,
        },
    );
    let snapshot = current_config_from_graph(&graph);
    let report = source_aware_mapping_report(&graph, &snapshot);
    assert!(report.root_file_mapped);
    assert_eq!(report.readable_files_mapped, 4);

    let root_value = snapshot.value_for("decoration.shadow.enabled");
    assert_eq!(root_value.status, CurrentValueSourceStatus::Configured);
    assert_eq!(root_value.source_path.as_deref(), Some(main.as_path()));
    assert_eq!(root_value.line_number, Some(3));
    assert_eq!(
        root_value.raw_line.as_deref(),
        Some("decoration:shadow:enabled = false")
    );

    let sourced_value = snapshot.value_for("decoration.blur.enabled");
    assert_eq!(sourced_value.status, CurrentValueSourceStatus::Configured);
    assert_eq!(
        sourced_value.source_path.as_deref(),
        Some(sourced.as_path())
    );
    assert_eq!(sourced_value.line_number, Some(2));

    let nested_value = snapshot.value_for("animations.enabled");
    assert_eq!(nested_value.source_path.as_deref(), Some(nested.as_path()));
    assert_eq!(nested_value.line_number, Some(1));

    let tilde_value = snapshot.value_for("misc.disable_splash_rendering");
    assert_eq!(tilde_value.source_path.as_deref(), Some(tilde.as_path()));
    assert_eq!(tilde_value.line_number, Some(1));
}

#[test]
fn source_aware_safe_batch_can_plan_exact_sourced_normal_scalar() {
    let root = temp_root("safe-batch-source");
    let main = root.join("hyprland.conf");
    let sourced = root.join("appearance.conf");
    write_file(&main, "source = appearance.conf\n");
    write_file(&sourced, "decoration:blur:enabled = true\n");
    let graph = inspect_config_graph(&main);
    let snapshot = current_config_from_graph(&graph);
    let plan = build_safe_batch_write_plan(
        "source-aware-plan",
        &known_settings(),
        &snapshot,
        &graph,
        vec![SafeBatchChangeRequest::new(
            "appearance.blur.enabled",
            "false",
        )],
        "source-aware",
    );
    assert!(plan.can_execute, "{:?}", plan.cannot_execute_reasons);
    assert_eq!(plan.eligible_changes.len(), 1);
    let change = &plan.eligible_changes[0];
    assert_eq!(change.target_path, sourced);
    assert_eq!(change.line_number, 1);
    assert_eq!(change.old_value, "true");
    assert_eq!(change.original_raw_line, "decoration:blur:enabled = true");
}

#[test]
fn source_aware_duplicates_across_root_and_sourced_files_still_block_apply() {
    let root = temp_root("duplicates");
    let main = root.join("hyprland.conf");
    let sourced = root.join("appearance.conf");
    write_file(
        &main,
        "source = appearance.conf\ndecoration:blur:enabled = false\n",
    );
    write_file(&sourced, "decoration:blur:enabled = true\n");
    let graph = inspect_config_graph(&main);
    let snapshot = current_config_from_graph(&graph);
    let value = snapshot.value_for("decoration.blur.enabled");
    assert_eq!(value.status, CurrentValueSourceStatus::DuplicateConflict);
    assert_eq!(value.duplicate_lines.len(), 2);

    let plan = build_safe_batch_write_plan(
        "source-aware-duplicate",
        &known_settings(),
        &snapshot,
        &graph,
        vec![SafeBatchChangeRequest::new(
            "appearance.blur.enabled",
            "false",
        )],
        "source-aware",
    );
    assert!(!plan.can_execute);
    assert!(plan
        .blocked_changes
        .iter()
        .any(|change| change.reason == SafeBatchEligibility::BlockedDuplicateConflict));
}

#[test]
fn missing_generated_script_and_symlink_connected_files_do_not_become_writable() {
    let root = temp_root("managed");
    let main = root.join("hyprland.conf");
    let generated = root.join("generated.conf");
    let script_managed = root.join("script-managed.conf");
    let scripts = root.join("scripts");
    let script = scripts.join("manage.sh");
    let symlink_target = root.join("profile-target.conf");
    let symlink = root.join("current.conf");

    write_file(
        &main,
        "source = missing.conf\nsource = generated.conf\nsource = script-managed.conf\nsource = current.conf\n",
    );
    write_file(
        &generated,
        "# generated by fixture\ndecoration:shadow:enabled = false\n",
    );
    write_file(&script_managed, "misc:disable_splash_rendering = false\n");
    write_file(
        &script,
        &format!("cp something {}\n", script_managed.display()),
    );
    write_file(&symlink_target, "animations:enabled = true\n");
    std::os::unix::fs::symlink(&symlink_target, &symlink).expect("symlink should create");

    let graph = inspect_config_graph_with_options(
        &main,
        ConfigGraphOptions {
            home_dir: Some(root.clone()),
            script_dirs: vec![scripts],
            max_depth: 16,
            source_follow_policy: SourceFollowPolicy::ReviewAll,
        },
    );
    assert_eq!(graph.unreadable_file_count, 1);
    let snapshot = current_config_from_graph(&graph);

    for (row_id, reason) in [
        (
            "appearance.shadow.enabled",
            SafeBatchEligibility::BlockedGeneratedFile,
        ),
        (
            "misc.disable_splash_rendering",
            SafeBatchEligibility::BlockedScriptManaged,
        ),
        (
            "animations.enabled",
            SafeBatchEligibility::BlockedSymlinkManaged,
        ),
    ] {
        let plan = build_safe_batch_write_plan(
            format!("source-aware-{}", reason.label()),
            &known_settings(),
            &snapshot,
            &graph,
            vec![SafeBatchChangeRequest::new(row_id, "true")],
            "source-aware",
        );
        assert!(!plan.can_execute, "{row_id} should stay blocked");
        assert!(plan
            .blocked_changes
            .iter()
            .any(|change| change.reason == reason));
    }

    let missing_plan = build_safe_batch_write_plan(
        "source-aware-default",
        &known_settings(),
        &snapshot,
        &graph,
        vec![SafeBatchChangeRequest::new(
            "appearance.blur.enabled",
            "false",
        )],
        "source-aware",
    );
    assert!(!missing_plan.can_execute);
    assert!(missing_plan
        .blocked_changes
        .iter()
        .any(|change| change.reason == SafeBatchEligibility::BlockedMissingLine));
}
