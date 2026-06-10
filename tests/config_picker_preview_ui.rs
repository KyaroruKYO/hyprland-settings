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
fn config_picker_button_is_active_preview_flow() {
    let source = fs::read_to_string("src/ui/window.rs").expect("window source should read");
    let section_source = source_slice(
        &source,
        "fn config_file_selection_section",
        "fn update_config_selection_preview",
    );

    assert!(section_source.contains("gtk::Button::with_label(\"Choose Config File...\")"));
    assert!(section_source.contains("gtk::FileChooserNative::new"));
    assert!(section_source.contains("gtk::FileChooserAction::Open"));
    assert!(section_source.contains("gtk::ResponseType::Accept"));
    assert!(section_source.contains("preview_manual_config("));
    assert!(section_source.contains("SourceFollowChoice::ReviewAllConnectedFiles"));
}

#[test]
fn selected_preview_and_clear_copy_are_visible_in_source() {
    let source = fs::read_to_string("src/ui/window.rs").expect("window source should read");
    let section_source = source_slice(
        &source,
        "fn config_file_selection_section",
        "fn update_config_selection_preview",
    );

    for copy in [
        "Selected for review:",
        "Selected file preview",
        "This file is only being reviewed.",
        "This has not changed what the app will write.",
        "This selection is not saved yet.",
        "Choose how this preview should read connected files.",
        "Review all connected files",
        "Only this file",
        "Cancel",
        "Clear selected file",
        "Use for this session preview",
        "Using this config for this app session only. This is not saved.",
        "This config is being reread for display only. Apply behavior has not changed.",
    ] {
        assert!(
            section_source.contains(copy),
            "missing preview-only picker copy: {copy}"
        );
    }

    assert!(!section_source.contains("session_button.set_sensitive(false)"));
    assert!(section_source.contains("cancel_preview()"));
}

#[test]
fn picker_preview_does_not_rebuild_model_discovery_or_write_flow() {
    let source = fs::read_to_string("src/ui/window.rs").expect("window source should read");
    let section_source = source_slice(
        &source,
        "fn config_file_selection_section",
        "fn update_config_selection_preview",
    );

    for forbidden in [
        "UiProjection::from_bundle",
        "CurrentConfigSnapshot",
        "ConfigDiscoveryStatus::Found",
        "discover_hyprland_config",
        "apply_setting_change",
        "write_target_path",
    ] {
        assert!(
            !section_source.contains(forbidden),
            "picker preview section must not change app data or write flow: {forbidden}"
        );
    }
}

#[test]
fn picker_preview_report_preserves_final_counts_and_boundaries() {
    let report: serde_json::Value = serde_json::from_slice(
        &fs::read("data/reports/config-picker-preview-flow.v0.55.2.json")
            .expect("report should exist"),
    )
    .expect("report should parse");

    assert_eq!(report["countsBefore"]["readableRows"], 341);
    assert_eq!(report["countsBefore"]["writableRows"], 341);
    assert_eq!(report["countsBefore"]["blockedRows"], 0);
    assert_eq!(report["countsAfter"]["readableRows"], 341);
    assert_eq!(report["countsAfter"]["writableRows"], 341);
    assert_eq!(report["countsAfter"]["blockedRows"], 0);
    assert_eq!(
        report["pickerPreviewBehavior"]["selectedConfigAffectsWrites"],
        false
    );
    assert_eq!(
        report["pickerPreviewBehavior"]["selectedConfigPersisted"],
        false
    );
    assert_eq!(SAFE_WRITABLE_ROWS.len(), 341);
}
