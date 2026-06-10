use std::fs;
use std::path::{Path, PathBuf};

use hyprland_settings::config_selection::{ConfigSelectionState, SourceFollowChoice};
use hyprland_settings::session_config_preview::{
    build_session_config_preview, SessionConfigPreviewReadStatus,
};
use hyprland_settings::write_classification::SAFE_WRITABLE_ROWS;

fn temp_fixture(name: &str) -> PathBuf {
    let root = std::env::temp_dir().join(format!(
        "hyprland-settings-session-preview-model-{name}-{}",
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

#[test]
fn session_preview_model_reads_selected_config_for_display_only() {
    let root = temp_fixture("readable");
    let config = root.join("hyprland.conf");
    write_file(
        &config,
        "general:layout = dwindle\ndecoration:blur:enabled = true\n",
    );

    let preview =
        build_session_config_preview(&config, SourceFollowChoice::ReviewAllConnectedFiles);

    assert!(preview.active);
    assert!(preview.clearable);
    assert!(!preview.persisted);
    assert!(!preview.affects_writes);
    assert_eq!(
        preview.read_status,
        SessionConfigPreviewReadStatus::Readable
    );
    assert_eq!(preview.scalar_value_count, 2);
    assert_eq!(preview.settings_with_multiple_locations, 0);
    assert_eq!(preview.readable_file_count, 1);
    assert_eq!(SAFE_WRITABLE_ROWS.len(), 341);
}

#[test]
fn session_selected_config_is_not_write_target_and_can_be_cleared() {
    let state = ConfigSelectionState::auto_detected("/tmp/active.conf")
        .preview_manual_config(
            "/tmp/session.conf",
            SourceFollowChoice::ReviewAllConnectedFiles,
        )
        .use_preview_for_session_read_only();

    assert_eq!(
        state.write_target_path(),
        Some(&PathBuf::from("/tmp/active.conf"))
    );
    assert!(state.preview().session_only);
    assert!(!state.preview().confirmed);

    let cleared = state.cancel_preview();
    assert_eq!(cleared.preview().session_read_only_config, None);
    assert_eq!(cleared.preview().selected_for_review, None);
    assert_eq!(
        cleared.write_target_path(),
        Some(&PathBuf::from("/tmp/active.conf"))
    );
}

#[test]
fn session_preview_unreadable_selected_config_reports_friendly_status() {
    let root = temp_fixture("missing");
    let missing = root.join("missing.conf");

    let preview =
        build_session_config_preview(&missing, SourceFollowChoice::ReviewAllConnectedFiles);
    let lines = preview.user_facing_lines();

    assert!(matches!(
        preview.read_status,
        SessionConfigPreviewReadStatus::Unreadable { .. }
    ));
    assert_eq!(preview.scalar_value_count, 0);
    assert!(lines
        .iter()
        .any(|line| line == "This file could not be read for preview."));
    assert!(lines.iter().any(|line| line == "No changes were made."));
}
