use std::fs;
use std::path::Path;

use anyhow::Result;
use hyprland_settings::config_discovery::{ConfigDiscovery, ConfigDiscoveryStatus};
use hyprland_settings::current_config::CurrentConfigSnapshot;
use hyprland_settings::export::ExportBundle;
use hyprland_settings::metadata::resolve_metadata_path_with_env;
use hyprland_settings::search::search_projection;
use hyprland_settings::ui::model::UiProjection;
use hyprland_settings::validation::validate_bundle;
use hyprland_settings::write_classification::SAFE_WRITABLE_ROWS;

fn load_projection() -> Result<UiProjection> {
    let resolution = resolve_metadata_path_with_env(None, None)?;
    let bundle = ExportBundle::load(Path::new(&resolution.export_dir))?;
    let summary = validate_bundle(&bundle)?;
    Ok(UiProjection::from_bundle(
        &bundle,
        &summary,
        ConfigDiscovery {
            status: ConfigDiscoveryStatus::Missing,
            attempted_paths: Vec::new(),
        },
        CurrentConfigSnapshot::read_unavailable("test fixture has no live config"),
    ))
}

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
fn dashboard_and_sidebar_include_config_entry_point() {
    let source = fs::read_to_string("src/ui/window.rs").expect("window source should read");
    let sidebar_source = source_slice(&source, "fn sidebar_items", "fn build_sidebar");
    let dashboard_source = source_slice(&source, "fn dashboard_cards", "fn build_dashboard_card");

    assert!(source.contains("const DASHBOARD_ID: &str = \"dashboard\""));
    assert!(source.contains("const CONFIG_ID: &str = \"config\""));
    assert!(sidebar_source.contains("label: \"Dashboard\".to_string()"));
    assert!(sidebar_source.contains("SIDEBAR_PAGE_LAYOUT"));

    assert!(dashboard_source.contains("title: \"Settings\""));
    assert!(dashboard_source.contains("target_tab_id: CONFIG_ID"));
    assert!(dashboard_source.contains("Choose which Hyprland config the app reviews"));
}

#[test]
fn config_page_is_read_only_scaffold_with_future_controls_disabled() {
    let source = fs::read_to_string("src/ui/window.rs").expect("window source should read");
    let config_source = source_slice(&source, "fn build_config_view", "fn config_path_summary");
    let config_file_source = source_slice(
        &source,
        "fn config_file_selection_section",
        "fn update_config_selection_preview",
    );
    let selection_source = source_slice(
        &source,
        "fn config_selection_scaffold_lines",
        "fn config_graph_summary_lines",
    );
    let graph_source = source_slice(
        &source,
        "fn config_graph_summary_lines",
        "fn config_section",
    );
    let render_source = source_slice(&source, "fn render_main_view", "fn render_settings_view");

    for text in [
        "Config file",
        "Choose Config File...",
        "Profiles",
        "Profile switching is not active yet.",
        "When a setting is controlled in more than one place",
    ] {
        assert!(
            config_source.contains(text)
                || config_file_source.contains(text)
                || source.contains(text),
            "missing Config page copy: {text}"
        );
    }

    for text in [
        "Using:",
        "Auto-detection is a starting point.",
        "Choose another config file to review.",
        "This has not changed what the app will write.",
        "The selected file is preview-only until a future review step.",
    ] {
        assert!(
            selection_source.contains(text),
            "missing Config selection copy: {text}"
        );
    }

    for text in [
        "Connected files",
        "hyprland-settings-connected-files-section",
        "config_graph_summary_lines",
        "This setup uses",
        "Some files are connected through source/include lines.",
        "No connected config files were detected.",
        "Connected-file review is read-only right now.",
        "Some files may be changed by scripts.",
        "Some connected files may not be shown yet.",
    ] {
        assert!(
            source.contains(text) || graph_source.contains(text),
            "missing Config graph copy or helper: {text}"
        );
    }

    assert!(source.contains("gtk::Button::with_label(\"Choose Config File...\")"));
    assert!(source.contains("gtk::Button::with_label(\"Choose review mode (planned)\")"));
    assert!(config_source.contains("profile_mode_detail_section()"));
    assert!(source.contains("Some((\"Profile switching planned\", false))"));
    assert!(source.contains("hyprland-settings-profile-mode-detail"));
    assert!(source.contains("action.set_sensitive(active)"));
    // The Config page routes through the shared standalone-page path.
    assert!(render_source.contains("standalone.show_only(selected_page_id)"));
    assert!(source.contains("(CONFIG_ID, config_view.clone())"));
    assert!(render_source.contains("settings_view.set_visible(false)"));
    assert!(render_source.contains("render_empty_detail(detail_content)"));
}

#[test]
fn normal_category_pages_search_and_details_still_work() -> Result<()> {
    let projection = load_projection()?;
    let appearance = search_projection(&projection, "appearance", "appearance.blur.enabled");
    assert!(appearance
        .results
        .iter()
        .any(|result| result.setting.row_id == "appearance.blur.enabled"));

    let blur = projection
        .detail_for_row("appearance.blur.enabled")
        .expect("appearance blur detail should exist");
    assert_eq!(blur.official_setting, "decoration.blur.enabled");

    let default_monitor = projection
        .detail_for_row("cursor.default_monitor")
        .expect("cursor.default_monitor detail should exist");
    assert_eq!(default_monitor.official_setting, "cursor.default_monitor");

    Ok(())
}

#[test]
fn config_page_scaffold_report_preserves_final_counts() {
    let report: serde_json::Value = serde_json::from_slice(
        &fs::read("data/reports/dashboard-config-page-scaffold.v0.55.2.json")
            .expect("report should exist"),
    )
    .expect("report should parse");

    assert_eq!(report["countsBefore"]["readableRows"], 341);
    assert_eq!(report["countsBefore"]["writableRows"], 341);
    assert_eq!(report["countsBefore"]["blockedRows"], 0);
    assert_eq!(report["countsAfter"]["readableRows"], 341);
    assert_eq!(report["countsAfter"]["writableRows"], 341);
    assert_eq!(report["countsAfter"]["blockedRows"], 0);
    assert_eq!(SAFE_WRITABLE_ROWS.len(), 341);
    assert_eq!(
        report["disabledOrFutureControls"][0],
        "Choose Config File..."
    );
}
