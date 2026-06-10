use std::fs;
use std::path::{Path, PathBuf};

use hyprland_settings::config_graph::{
    inspect_config_graph_with_options, ConfigDetectionConfidence, ConfigGraphOptions,
    ConfigManagementHintKind, SourceFollowPolicy,
};
use hyprland_settings::write_classification::SAFE_WRITABLE_ROWS;

fn temp_fixture(name: &str) -> PathBuf {
    let root = std::env::temp_dir().join(format!(
        "hyprland-settings-config-graph-{name}-{}",
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

fn inspect(
    root: &Path,
    home: &Path,
    script_dirs: Vec<PathBuf>,
) -> hyprland_settings::config_graph::ConfigGraphSummary {
    inspect_config_graph_with_options(
        root,
        ConfigGraphOptions {
            home_dir: Some(home.to_path_buf()),
            script_dirs,
            max_depth: 16,
            source_follow_policy: SourceFollowPolicy::ReviewAll,
        },
    )
}

fn file_has_hint(
    summary: &hyprland_settings::config_graph::ConfigGraphSummary,
    file_name: &str,
    hint_kind: ConfigManagementHintKind,
) -> bool {
    summary.files.iter().any(|file| {
        file.path
            .file_name()
            .and_then(|name| name.to_str())
            .is_some_and(|name| name == file_name)
            && file.hints.iter().any(|hint| hint.kind == hint_kind)
    })
}

#[test]
fn single_file_config_has_no_connected_files() {
    let root = temp_fixture("single");
    let config = root.join("hyprland.conf");
    write_file(&config, "general:layout = dwindle\n");

    let summary = inspect(&config, &root, Vec::new());

    assert_eq!(summary.connected_file_count, 1);
    assert!(!summary.multi_file);
    assert_eq!(summary.unreadable_file_count, 0);
    assert!(summary.source_references.is_empty());
    assert_eq!(SAFE_WRITABLE_ROWS.len(), 341);
}

#[test]
fn source_include_config_records_profile_file() {
    let root = temp_fixture("source");
    let config = root.join("hyprland.conf");
    let desktop = root.join("profiles/desktop.conf");
    write_file(&config, "source = profiles/desktop.conf\n");
    write_file(&desktop, "decoration:blur:enabled = true\n");

    let summary = inspect(&config, &root, Vec::new());

    assert_eq!(summary.connected_file_count, 2);
    assert!(summary.multi_file);
    assert_eq!(summary.source_references.len(), 1);
    assert!(summary.has_profile_hints);
    assert!(summary.has_mode_hints);
    assert!(file_has_hint(
        &summary,
        "desktop.conf",
        ConfigManagementHintKind::DesktopProfile
    ));
}

#[test]
fn relative_source_path_resolves_from_containing_file() {
    let root = temp_fixture("relative");
    let config = root.join("nested/hyprland.conf");
    let desktop = root.join("nested/profiles/desktop.conf");
    write_file(&config, "source=./profiles/desktop.conf\n");
    write_file(&desktop, "general:gaps_in = 5\n");

    let summary = inspect(&config, &root, Vec::new());
    let target = summary.source_references[0]
        .resolved_target
        .as_ref()
        .expect("relative source should resolve");

    assert_eq!(target, &desktop);
    assert!(summary.files.iter().any(|file| file.path == desktop));
}

#[test]
fn tilde_source_path_uses_configured_home_directory() {
    let root = temp_fixture("tilde");
    let config = root.join(".config/hypr/hyprland.conf");
    let desktop = root.join(".config/hypr/profiles/desktop.conf");
    write_file(&config, "source = ~/.config/hypr/profiles/desktop.conf\n");
    write_file(&desktop, "general:gaps_out = 10\n");

    let summary = inspect(&config, &root, Vec::new());
    let target = summary.source_references[0]
        .resolved_target
        .as_ref()
        .expect("tilde source should resolve");

    assert_eq!(target, &desktop);
    assert_eq!(summary.connected_file_count, 2);
}

#[test]
fn symlinked_current_profile_records_symlink_and_target_hints() {
    let root = temp_fixture("symlink");
    let config = root.join("hyprland.conf");
    let current = root.join("modes/current.conf");
    let desktop = root.join("modes/desktop.conf");
    write_file(&config, "source = modes/current.conf\n");
    write_file(&desktop, "misc:disable_hyprland_logo = false\n");
    std::os::unix::fs::symlink("desktop.conf", &current)
        .expect("fixture symlink should be created");

    let summary = inspect(&config, &root, Vec::new());
    let current_file = summary
        .files
        .iter()
        .find(|file| file.path == current)
        .expect("current symlink should be recorded");

    assert!(current_file.is_symlink);
    assert_eq!(current_file.symlink_target.as_ref(), Some(&desktop));
    assert!(file_has_hint(
        &summary,
        "current.conf",
        ConfigManagementHintKind::CurrentProfile
    ));
    assert!(file_has_hint(
        &summary,
        "current.conf",
        ConfigManagementHintKind::SymlinkManaged
    ));
}

#[test]
fn missing_source_target_is_recorded_without_panic() {
    let root = temp_fixture("missing");
    let config = root.join("hyprland.conf");
    write_file(&config, "source = missing.conf\n");

    let summary = inspect(&config, &root, Vec::new());

    assert_eq!(summary.connected_file_count, 2);
    assert_eq!(summary.unreadable_file_count, 1);
    assert!(summary
        .unreadable_files
        .iter()
        .any(|issue| issue.path.ends_with("missing.conf")));
}

#[test]
fn generated_marker_adds_generated_file_hint() {
    let root = temp_fixture("generated");
    let config = root.join("hyprland.conf");
    let generated = root.join("profiles/generated.conf");
    write_file(&config, "source = profiles/generated.conf\n");
    write_file(
        &generated,
        "# generated by profile manager\n# do not edit\ninput:kb_layout = us\n",
    );

    let summary = inspect(&config, &root, Vec::new());

    assert!(summary.has_generated_hints);
    assert!(file_has_hint(
        &summary,
        "generated.conf",
        ConfigManagementHintKind::GeneratedFile
    ));
}

#[test]
fn script_managed_hint_reads_scripts_without_executing_them() {
    let root = temp_fixture("script");
    let config = root.join("hyprland.conf");
    let current = root.join("modes/current.conf");
    let desktop = root.join("modes/desktop.conf");
    let scripts = root.join("scripts");
    write_file(&config, "source = modes/current.conf\n");
    write_file(&desktop, "general:layout = dwindle\n");
    std::os::unix::fs::symlink("desktop.conf", &current)
        .expect("fixture symlink should be created");
    write_file(
        &scripts.join("switch-mode.sh"),
        "#!/bin/sh\nln -sf desktop.conf current.conf\n# hyprctl reload would happen elsewhere\n",
    );

    let summary = inspect(&config, &root, vec![scripts]);

    assert!(summary.has_script_managed_hints);
    let current_file = summary
        .files
        .iter()
        .find(|file| file.path == current)
        .expect("current config should be recorded");
    assert!(current_file.hints.iter().any(|hint| {
        hint.kind == ConfigManagementHintKind::ScriptManaged
            && hint.confidence == ConfigDetectionConfidence::Likely
    }));
}

#[test]
fn source_cycles_are_recorded_without_infinite_recursion() {
    let root = temp_fixture("cycle");
    let a = root.join("a.conf");
    let b = root.join("b.conf");
    write_file(&a, "source = b.conf\n");
    write_file(&b, "source = a.conf\n");

    let summary = inspect(&a, &root, Vec::new());

    assert_eq!(summary.connected_file_count, 2);
    assert_eq!(summary.cycles.len(), 1);
    assert!(summary.cycles[0].message.contains("cycle"));
}
