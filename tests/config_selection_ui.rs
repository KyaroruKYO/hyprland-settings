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
        "Review connected files: Review all connected files / Only this file / Cancel.",
        "Manual selection is preview-only and is not saved yet.",
        "Choose Config File... (planned)",
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

    assert!(config_source.contains("Some((\"Choose Config File... (planned)\", false))"));
    assert!(source.contains("gtk::Button::with_label(\"Choose review mode (planned)\")"));
    assert!(config_source.contains("Some((\"Profile switching planned\", false))"));
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
    assert!(source.contains("label: \"Config\".to_string()"));
    assert!(dashboard_source.contains("title: \"Config\""));
    assert!(dashboard_source.contains("target_tab_id: CONFIG_ID"));
    assert!(render_source.contains("selected_tab_id == CONFIG_ID"));
    assert!(source.contains("append_connected_file_details"));
    assert!(source.contains("connected_file_card(file, graph)"));
    assert!(source.contains("search_projection(model, selected_tab_id, query)"));
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
