use std::fs;

use hyprland_settings::write_classification::SAFE_WRITABLE_ROWS;

fn source_slice<'a>(source: &'a str, start: &str, end: &str) -> &'a str {
    let start = source
        .find(start)
        .expect("source start marker should exist");
    let end = source[start..]
        .find(end)
        .map(|offset| start + offset)
        .expect("source end marker should exist");
    &source[start..end]
}

#[test]
fn selected_file_preview_panel_and_summary_copy_exist() {
    let source = fs::read_to_string("src/ui/window.rs").expect("window source should read");
    let section = source_slice(
        &source,
        "fn config_file_selection_section",
        "fn config_selection_scaffold_lines",
    );

    for copy in [
        "Selected file preview",
        "This file is only being reviewed.",
        "Connected files found:",
        "Unreadable files:",
        "Profile-style files:",
        "Script-managed hints:",
        "Generated-file hints:",
        "Cycles:",
        "Unsupported patterns:",
        "This file could not be read for preview.",
        "No changes were made.",
    ] {
        assert!(
            section.contains(copy),
            "missing preview inspection copy: {copy}"
        );
    }

    assert!(section.contains("inspect_config_graph_with_options"));
    assert!(section.contains("SourceFollowPolicy::ReviewAll"));
    assert!(section.contains("SourceFollowPolicy::OnlyRoot"));
    assert_eq!(SAFE_WRITABLE_ROWS.len(), 341);
}

#[test]
fn selected_preview_still_does_not_touch_write_or_app_data_paths() {
    let source = fs::read_to_string("src/ui/window.rs").expect("window source should read");
    let section = source_slice(
        &source,
        "fn config_file_selection_section",
        "fn config_selection_scaffold_lines",
    );

    for forbidden in [
        "UiProjection::from_bundle",
        "CurrentConfigSnapshot",
        "discover_hyprland_config",
        "apply_setting_change",
        "write_target_path",
    ] {
        assert!(
            !section.contains(forbidden),
            "preview inspection must not mutate app data or write flow: {forbidden}"
        );
    }
}
