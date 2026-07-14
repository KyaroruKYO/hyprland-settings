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
fn config_page_shows_manual_selection_scaffold_copy() {
    let source = fs::read_to_string("src/ui/window.rs").expect("window source should read");
    let selection_source = source_slice(
        &source,
        "fn config_selection_scaffold_lines",
        "fn config_graph_summary_lines",
    );

    for copy in [
        "Auto-detection is a starting point.",
        "Choose another config file to review.",
        "This has not changed what the app will write.",
        "The selected file is preview-only until a future review step.",
        "Choose Config File...",
    ] {
        assert!(
            source.contains(copy) || selection_source.contains(copy),
            "missing config selection UI copy: {copy}"
        );
    }
}

#[test]
fn config_selection_ui_keeps_future_controls_disabled() {
    let source = fs::read_to_string("src/ui/window.rs").expect("window source should read");
    let config_source = source_slice(&source, "fn build_config_view", "fn config_path_summary");

    assert!(source.contains("gtk::Button::with_label(\"Choose Config File...\")"));
    assert!(source.contains("gtk::Button::with_label(\"Choose review mode (planned)\")"));
    assert!(config_source.contains("profile_mode_detail_section()"));
    assert!(source.contains("Some((\"Profile switching planned\", false))"));
    assert!(source.contains("hyprland-settings-connected-files-section"));
    assert!(source.contains("hyprland-settings-profile-mode-detail"));
    assert!(source.contains("Some files are connected through source/include lines."));
    assert!(
        source.contains("action.set_sensitive(false)")
            || source.contains("action.set_sensitive(active)")
    );
}

#[test]
fn config_selection_ui_preserves_existing_config_routes_and_details() {
    let source = fs::read_to_string("src/ui/window.rs").expect("window source should read");
    let render_source = source_slice(&source, "fn render_main_view", "fn render_settings_view");
    let dashboard_source = source_slice(&source, "fn dashboard_cards", "fn build_dashboard_card");

    assert!(source.contains("const CONFIG_ID: &str = \"config\""));
    // The config page id keeps its internal name; the sidebar shows "Settings".
    assert!(fs::read_to_string("src/ux_presentation.rs")
        .expect("presentation reads")
        .contains("label: \"Settings\""));
    assert!(dashboard_source.contains("title: \"Settings\""));
    assert!(dashboard_source.contains("target_tab_id: CONFIG_ID"));
    // The Config page routes through the shared standalone-page path.
    assert!(render_source.contains("standalone.show_only(selected_page_id)"));
    assert!(source.contains("(CONFIG_ID, config_view.clone())"));
    assert!(source.contains("append_connected_file_details"));
    assert!(source.contains("connected_file_card(file, graph)"));
    assert!(source.contains("search_projection(model, source_tab, query)"));
    // Scalar saves route through the Safe Live Save Mode gate; UI code no
    // longer calls apply_setting_change directly.
    assert!(source.contains("gated_scalar_save_live("));
    assert!(!source.contains("apply_setting_change("));
    assert_eq!(SAFE_WRITABLE_ROWS.len(), 341);
}

#[test]
fn config_selection_report_preserves_final_counts_and_boundaries() {
    let report: serde_json::Value = serde_json::from_slice(
        &fs::read("data/reports/config-selection-state-picker-scaffold.v0.55.2.json")
            .expect("report should exist"),
    )
    .expect("report should parse");

    assert_eq!(report["countsBefore"]["readableRows"], 341);
    assert_eq!(report["countsBefore"]["writableRows"], 341);
    assert_eq!(report["countsBefore"]["blockedRows"], 0);
    assert_eq!(report["countsAfter"]["readableRows"], 341);
    assert_eq!(report["countsAfter"]["writableRows"], 341);
    assert_eq!(report["countsAfter"]["blockedRows"], 0);
    assert_eq!(report["manualPickerScaffold"]["active"], false);
    assert_eq!(
        report["writeFlowPreservation"]["selectedConfigAffectsWrites"],
        false
    );
    assert_eq!(
        report["writeFlowPreservation"]["selectedConfigPersisted"],
        false
    );
}
