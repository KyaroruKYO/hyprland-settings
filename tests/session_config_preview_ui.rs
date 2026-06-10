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
fn config_page_contains_active_session_preview_ui_copy() {
    let source = fs::read_to_string("src/ui/window.rs").expect("window source should read");
    let model_source =
        fs::read_to_string("src/session_config_preview.rs").expect("model source should read");
    let section = source_slice(
        &source,
        "fn config_file_selection_section",
        "fn config_selection_scaffold_lines",
    );

    for copy in [
        "Use for this session preview",
        "Using this config for this app session only. This is not saved.",
        "This config is being reread for display only. Apply behavior has not changed.",
        "Clear session preview",
        "Session preview",
        "Values read for preview:",
        "Settings with multiple locations:",
        "Connected files are not included in this session preview.",
        "Choose where to save changes in a future version.",
    ] {
        assert!(
            section.contains(copy) || model_source.contains(copy),
            "missing session preview UI copy: {copy}"
        );
    }

    assert!(section.contains("use_preview_for_session_read_only()"));
    assert!(section.contains("build_session_config_preview"));
    assert_eq!(SAFE_WRITABLE_ROWS.len(), 341);
}

#[test]
fn session_preview_ui_does_not_call_production_write_or_projection_rebuilds() {
    let source = fs::read_to_string("src/ui/window.rs").expect("window source should read");
    let section = source_slice(
        &source,
        "fn config_file_selection_section",
        "fn config_selection_scaffold_lines",
    );

    for forbidden in [
        "UiProjection::from_bundle",
        "CurrentConfigSnapshot::from_discovery",
        "discover_hyprland_config",
        "apply_setting_change",
        "write_target_path",
    ] {
        assert!(
            !section.contains(forbidden),
            "session preview UI must not mutate production model or write flow: {forbidden}"
        );
    }
}
