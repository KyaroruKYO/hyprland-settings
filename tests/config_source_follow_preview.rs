use std::fs;
use std::path::{Path, PathBuf};

use hyprland_settings::config_graph::{
    inspect_config_graph_with_options, ConfigGraphOptions, SourceFollowPolicy,
};
use hyprland_settings::config_selection::{ConfigSelectionState, SourceFollowChoice};
use hyprland_settings::write_classification::SAFE_WRITABLE_ROWS;

fn temp_fixture(name: &str) -> PathBuf {
    let root = std::env::temp_dir().join(format!(
        "hyprland-settings-source-follow-{name}-{}",
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

fn inspect(root: &Path, source_follow_policy: SourceFollowPolicy) -> usize {
    inspect_config_graph_with_options(
        root,
        ConfigGraphOptions {
            home_dir: Some(root.parent().unwrap_or(root).to_path_buf()),
            script_dirs: Vec::new(),
            max_depth: 16,
            source_follow_policy,
        },
    )
    .connected_file_count
}

#[test]
fn review_all_connected_files_previews_connected_graph() {
    let root = temp_fixture("all");
    let config = root.join("hyprland.conf");
    let profile = root.join("profiles/desktop.conf");
    write_file(&config, "source = profiles/desktop.conf\n");
    write_file(&profile, "general:layout = dwindle\n");

    assert_eq!(inspect(&config, SourceFollowPolicy::ReviewAll), 2);
    assert_eq!(SAFE_WRITABLE_ROWS.len(), 341);
}

#[test]
fn only_this_file_previews_only_selected_file_without_active_config_change() {
    let root = temp_fixture("only");
    let config = root.join("hyprland.conf");
    let profile = root.join("profiles/desktop.conf");
    write_file(&config, "source = profiles/desktop.conf\n");
    write_file(&profile, "general:layout = dwindle\n");

    assert_eq!(inspect(&config, SourceFollowPolicy::OnlyRoot), 1);

    let state = ConfigSelectionState::auto_detected("/tmp/active.conf")
        .preview_manual_config(&config, SourceFollowChoice::OnlySelectedFile);
    assert_eq!(
        state.write_target_path(),
        Some(&PathBuf::from("/tmp/active.conf"))
    );
    assert_eq!(
        state.preview().source_follow_choice,
        SourceFollowChoice::OnlySelectedFile
    );
}

#[test]
fn cancel_clears_preview_without_affecting_write_target() {
    let state = ConfigSelectionState::auto_detected("/tmp/active.conf")
        .preview_manual_config(
            "/tmp/selected.conf",
            SourceFollowChoice::ReviewAllConnectedFiles,
        )
        .cancel_preview();

    assert_eq!(state.preview().selected_for_review, None);
    assert_eq!(
        state.write_target_path(),
        Some(&PathBuf::from("/tmp/active.conf"))
    );
}
