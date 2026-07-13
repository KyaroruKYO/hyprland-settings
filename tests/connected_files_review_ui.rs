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
fn config_page_renders_connected_files_review_helpers() {
    let source = fs::read_to_string("src/ui/window.rs").expect("window source should read");
    let connected_source = source_slice(
        &source,
        "fn connected_files_review_section",
        "fn friendly_config_graph_summary",
    );

    for helper in [
        "append_connected_files_review",
        "connected_file_card",
        "connected_file_title",
        "connected_file_hint_lines",
        "friendly_config_hint_label",
        "append_connected_file_issue_warnings",
        "friendly_path",
    ] {
        assert!(
            source.contains(helper),
            "missing connected-files UI helper: {helper}"
        );
    }

    assert!(connected_source.contains("Connected files"));
    assert!(connected_source.contains("Choose review mode (planned)"));
    assert!(connected_source.contains("action.set_sensitive(false)"));
}

#[test]
fn connected_file_user_facing_labels_are_present() {
    let source = fs::read_to_string("src/ui/window.rs").expect("window source should read");
    let title_source = source_slice(
        &source,
        "fn connected_file_title",
        "fn connected_file_hint_lines",
    );
    let hint_source = source_slice(
        &source,
        "fn friendly_config_hint_label",
        "fn append_connected_file_issue_warnings",
    );

    for label in [
        "Main config",
        "Current profile",
        "Desktop profile",
        "Gaming profile",
        "Theme profile",
        "Host profile",
        "Generated file",
        "May be changed by scripts",
        "Symlinked file",
        "Profile file",
    ] {
        assert!(
            title_source.contains(label) || hint_source.contains(label),
            "missing friendly label: {label}"
        );
    }

    for avoided in [
        "source graph",
        "symlink provenance",
        "duplicate scalar conflict",
        "ambiguous write target",
        "parser normalization",
    ] {
        assert!(
            !hint_source.contains(avoided),
            "main connected-files labels should not expose technical wording: {avoided}"
        );
    }
}

#[test]
fn connected_file_issue_warnings_are_user_facing() {
    let source = fs::read_to_string("src/ui/window.rs").expect("window source should read");
    let warning_source = source_slice(
        &source,
        "fn append_connected_file_issue_warnings",
        "fn friendly_path",
    );

    for copy in [
        "Some connected files could not be read.",
        "Missing or unreadable:",
        "Some connected files refer back to each other.",
        "The app stopped following them to avoid looping.",
        "Some connected file patterns are not shown yet.",
        "Review carefully before editing these files in a future version.",
    ] {
        assert!(
            warning_source.contains(copy),
            "missing connected-files warning copy: {copy}"
        );
    }
}

#[test]
fn dashboard_sidebar_normal_pages_and_counts_remain_preserved() {
    let source = fs::read_to_string("src/ui/window.rs").expect("window source should read");
    let render_source = source_slice(&source, "fn render_main_view", "fn render_settings_view");
    let dashboard_source = source_slice(&source, "fn dashboard_cards", "fn build_dashboard_card");

    assert!(source.contains("const CONFIG_ID: &str = \"config\""));
    assert!(source.contains("label: \"Config\".to_string()"));
    assert!(dashboard_source.contains("title: \"Config\""));
    assert!(dashboard_source.contains("target_tab_id: CONFIG_ID"));
    // The Config page routes through the shared standalone-page path.
    assert!(render_source.contains("[DASHBOARD_ID, CONFIG_ID, SAFETY_ID]"));
    assert!(render_source.contains("config_view.set_visible(page_id == CONFIG_ID)"));
    assert!(render_source.contains("settings_view.set_visible(false)"));
    assert!(render_source.contains("render_settings_view("));
    assert!(source.contains("search_projection(model, selected_tab_id, query)"));
    assert_eq!(SAFE_WRITABLE_ROWS.len(), 341);
}

#[test]
fn connected_files_review_report_preserves_final_counts() {
    let report: serde_json::Value = serde_json::from_slice(
        &fs::read("data/reports/connected-files-review-ui.v0.55.2.json")
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
        report["disabledOrFutureControls"][0],
        "Choose Config File..."
    );
    assert_eq!(
        report["disabledOrFutureControls"][1],
        "Choose review mode (planned)"
    );
    assert_eq!(
        report["disabledOrFutureControls"][2],
        "Profile switching planned"
    );
}
