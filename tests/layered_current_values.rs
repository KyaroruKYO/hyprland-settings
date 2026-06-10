use std::fs;
use std::path::{Path, PathBuf};

use hyprland_settings::config_graph::{
    inspect_config_graph_with_options, ConfigGraphOptions, SourceFollowPolicy,
};
use hyprland_settings::config_layered_values::layered_values_for_setting;
use hyprland_settings::write_classification::SAFE_WRITABLE_ROWS;

fn temp_fixture(name: &str) -> PathBuf {
    let root = std::env::temp_dir().join(format!(
        "hyprland-settings-layered-values-{name}-{}",
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
fn fixture_with_base_and_profile_values_produces_layered_records() {
    let root = temp_fixture("base-profile");
    let config = root.join("hyprland.conf");
    let desktop = root.join("profiles/desktop.conf");
    let gaming = root.join("profiles/gaming.conf");
    write_file(
        &config,
        "source = profiles/desktop.conf\nsource = profiles/gaming.conf\ndecoration:blur:enabled = false\n",
    );
    write_file(&desktop, "decoration:blur:enabled = true\n");
    write_file(&gaming, "decoration:blur:enabled = false\n");

    let graph = inspect_config_graph_with_options(
        &config,
        ConfigGraphOptions {
            home_dir: Some(root.clone()),
            script_dirs: Vec::new(),
            max_depth: 16,
            source_follow_policy: SourceFollowPolicy::ReviewAll,
        },
    );
    let layered = layered_values_for_setting(&graph, "decoration.blur.enabled");

    assert_eq!(layered.occurrences.len(), 3);
    assert!(layered.controlled_in_more_than_one_place);
    assert_eq!(layered.currently_active_value.as_deref(), Some("false"));
    let lines = layered.display_lines();
    assert!(lines
        .iter()
        .any(|line| line == "This setting is controlled in more than one place."));
    assert!(lines
        .iter()
        .any(|line| line.contains("Desktop profile: true")));
    assert!(lines
        .iter()
        .any(|line| line.contains("Gaming profile: false")));
    assert!(lines
        .iter()
        .any(|line| line == "Choose where to save changes in a future version."));
    assert_eq!(SAFE_WRITABLE_ROWS.len(), 341);
}

#[test]
fn structured_block_false_positive_does_not_become_scalar_layered_value() {
    let root = temp_fixture("structured");
    let config = root.join("hyprland.conf");
    let profile = root.join("profiles/desktop.conf");
    write_file(
        &config,
        "source = profiles/desktop.conf\nmonitor = eDP-1,preferred,auto,1\n",
    );
    write_file(&profile, "monitor = HDMI-A-1,preferred,auto,1\n");

    let graph = inspect_config_graph_with_options(
        &config,
        ConfigGraphOptions {
            home_dir: Some(root.clone()),
            script_dirs: Vec::new(),
            max_depth: 16,
            source_follow_policy: SourceFollowPolicy::ReviewAll,
        },
    );
    let layered = layered_values_for_setting(&graph, "hl.monitor");

    assert!(layered.occurrences.is_empty());
    assert!(!layered.controlled_in_more_than_one_place);
}
