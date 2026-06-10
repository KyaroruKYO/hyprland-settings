use std::fs;
use std::path::{Path, PathBuf};

use hyprland_settings::config_graph::{
    inspect_config_graph_with_options, ConfigGraphOptions, SourceFollowPolicy,
};
use hyprland_settings::config_layered_values::layered_values_for_setting;
use hyprland_settings::current_config::{CurrentConfigSnapshot, CurrentValueProjection};
use hyprland_settings::session_value_projection::{
    compare_active_and_session_values, SessionValueComparisonStatus,
};
use hyprland_settings::write_classification::SAFE_WRITABLE_ROWS;

fn temp_fixture(name: &str) -> PathBuf {
    let root = std::env::temp_dir().join(format!(
        "hyprland-settings-session-value-{name}-{}",
        std::process::id()
    ));
    let _ = fs::remove_dir_all(&root);
    fs::create_dir_all(&root).expect("fixture root should be created");
    root
}

fn write_file(path: &Path, content: &str) {
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent).expect("fixture parent should be created");
    }
    fs::write(path, content).expect("fixture file should be written");
}

fn graph_for(path: &Path, root: &Path) -> hyprland_settings::config_graph::ConfigGraphSummary {
    inspect_config_graph_with_options(
        path,
        ConfigGraphOptions {
            home_dir: Some(root.to_path_buf()),
            script_dirs: Vec::new(),
            max_depth: 16,
            source_follow_policy: SourceFollowPolicy::ReviewAll,
        },
    )
}

#[test]
fn active_and_session_values_can_be_compared_read_only() {
    let root = temp_fixture("different");
    let active_config = root.join("active.conf");
    let session_config = root.join("session.conf");
    write_file(&active_config, "decoration:blur:enabled = true\n");
    write_file(&session_config, "decoration:blur:enabled = false\n");

    let active = CurrentConfigSnapshot::from_parsed(
        hyprland_settings::config_parser::parse_hyprland_config_file(&active_config)
            .expect("active fixture should parse"),
    );
    let graph = graph_for(&session_config, &root);
    let layered = layered_values_for_setting(&graph, "decoration.blur.enabled");
    let projection = compare_active_and_session_values(
        "appearance.blur.enabled",
        "decoration.blur.enabled",
        &active.value_for("decoration.blur.enabled"),
        &layered,
    );

    assert_eq!(
        projection.comparison_status,
        SessionValueComparisonStatus::Different
    );
    assert_eq!(projection.active_value.as_deref(), Some("true"));
    assert_eq!(projection.session_preview_value.as_deref(), Some("false"));
    assert!(projection.read_only);
    assert!(!projection.affects_writes);
    assert!(projection
        .user_facing_lines()
        .iter()
        .any(|line| line == "Apply behavior has not changed."));
}

#[test]
fn comparison_statuses_cover_same_missing_unreadable_and_unknown() {
    let root = temp_fixture("statuses");
    let config = root.join("hyprland.conf");
    write_file(&config, "general:layout = dwindle\n");
    let graph = graph_for(&config, &root);
    let layered = layered_values_for_setting(&graph, "general.layout");
    let active = CurrentConfigSnapshot::from_parsed(
        hyprland_settings::config_parser::parse_hyprland_config_file(&config)
            .expect("fixture should parse"),
    );

    let same = compare_active_and_session_values(
        "windows.layout",
        "general.layout",
        &active.value_for("general.layout"),
        &layered,
    );
    assert_eq!(same.comparison_status, SessionValueComparisonStatus::Same);

    let missing_active = compare_active_and_session_values(
        "windows.layout",
        "general.layout",
        &CurrentValueProjection::not_configured(),
        &layered,
    );
    assert_eq!(
        missing_active.comparison_status,
        SessionValueComparisonStatus::MissingInActiveConfig
    );

    let missing_session = compare_active_and_session_values(
        "windows.layout",
        "general.layout",
        &active.value_for("general.layout"),
        &layered_values_for_setting(&graph, "misc.disable_hyprland_logo"),
    );
    assert_eq!(
        missing_session.comparison_status,
        SessionValueComparisonStatus::MissingInSessionPreview
    );

    let unreadable = compare_active_and_session_values(
        "windows.layout",
        "general.layout",
        &CurrentValueProjection::read_unavailable("test read error".to_string()),
        &layered,
    );
    assert_eq!(
        unreadable.comparison_status,
        SessionValueComparisonStatus::Unreadable
    );

    let unknown = compare_active_and_session_values(
        "windows.layout",
        "general.layout",
        &CurrentValueProjection::not_configured(),
        &layered_values_for_setting(&graph, "misc.disable_hyprland_logo"),
    );
    assert_eq!(
        unknown.comparison_status,
        SessionValueComparisonStatus::Unknown
    );
    assert_eq!(SAFE_WRITABLE_ROWS.len(), 341);
}

#[test]
fn detail_ui_session_projection_scaffold_is_read_only() {
    let source = fs::read_to_string("src/ui/window.rs").expect("window source should read");
    let start = source
        .find("fn append_session_value_projection_summary")
        .expect("session projection helper should exist");
    let end = source[start..]
        .find("fn append_pre_apply_review_scaffold")
        .map(|offset| start + offset)
        .expect("next helper should exist");
    let section = &source[start..end];

    for copy in [
        "Session preview comparison",
        "Active config value",
        "Session preview value",
        "Apply behavior has not changed.",
        "Session source",
    ] {
        assert!(
            section.contains(copy)
                || fs::read_to_string("src/session_value_projection.rs")
                    .expect("projection source should read")
                    .contains(copy),
            "missing session comparison copy: {copy}"
        );
    }

    for forbidden in [
        "apply_setting_change",
        "UiProjection::from_bundle",
        "CurrentConfigSnapshot::from_discovery",
        "write_target_path",
    ] {
        assert!(
            !section.contains(forbidden),
            "session projection helper must stay read-only: {forbidden}"
        );
    }
}
