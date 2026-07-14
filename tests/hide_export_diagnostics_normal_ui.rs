use std::fs;

use hyprland_settings::write_classification::SAFE_WRITABLE_ROWS;

fn read_json(path: &str) -> serde_json::Value {
    serde_json::from_slice(&fs::read(path).expect("report should exist"))
        .expect("report should parse")
}

#[test]
fn normal_ui_does_not_append_export_diagnostics() {
    let source = fs::read_to_string("src/ui/window.rs").expect("window source should read");

    assert!(!source.contains("fn build_status_diagnostics_expander"));
    assert!(!source.contains("fn build_summary_card"));
    assert!(
        !source.contains("let diagnostics = build_status_diagnostics_expander(&model);"),
        "normal UI construction must not instantiate the export diagnostics expander"
    );
    assert!(
        !source.contains("settings_view.append(&diagnostics)"),
        "normal UI construction must not append export diagnostics above search"
    );
    assert!(
        !source.contains("Search export metadata"),
        "search placeholder should be user-facing"
    );
    assert!(source.contains("Search settings"));
}

#[test]
fn dashboard_sidebar_row_copy_and_details_remain_intact() {
    let source = fs::read_to_string("src/ui/window.rs").expect("window source should read");

    assert!(source.contains("const DASHBOARD_ID: &str = \"dashboard\""));
    assert!(source.contains("fn build_dashboard_view"));
    assert!(source.contains("fn dashboard_cards"));
    // Page labels now come from the sidebar page layout ("Keybinds").
    assert!(fs::read_to_string("src/ux_presentation.rs")
        .expect("presentation reads")
        .contains("label: \"Keybinds\""));
    assert!(source.contains("friendly_row_current_status"));
    assert!(source.contains("friendly_row_attention_status"));
    assert!(source.contains("Source / advanced metadata"));
    assert!(source.contains("append_detail_section(detail_content, \"Setting\""));
    assert!(source.contains("append_detail_section(detail_content, \"Current value\""));
    assert!(source.contains("append_detail_section(detail_content, \"Edit\""));
    assert!(source.contains("append_detail_section(detail_content, \"Safety\""));
}

#[test]
fn hide_export_diagnostics_report_preserves_final_counts() {
    let report = read_json("data/reports/hide-export-diagnostics-normal-ui.v0.55.2.json");

    assert_eq!(report["countsBefore"]["readableRows"], 341);
    assert_eq!(report["countsBefore"]["writableRows"], 341);
    assert_eq!(report["countsBefore"]["blockedRows"], 0);
    assert_eq!(report["countsAfter"]["readableRows"], 341);
    assert_eq!(report["countsAfter"]["writableRows"], 341);
    assert_eq!(report["countsAfter"]["blockedRows"], 0);
    assert_eq!(SAFE_WRITABLE_ROWS.len(), 341);
    assert_eq!(
        report["searchPlaceholderChange"]["after"],
        "Search settings"
    );
}
