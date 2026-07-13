use std::fs;
use std::path::Path;

use anyhow::Result;
use hyprland_settings::config_discovery::{ConfigDiscovery, ConfigDiscoveryStatus};
use hyprland_settings::config_parser::parse_hyprland_config_text;
use hyprland_settings::current_config::CurrentConfigSnapshot;
use hyprland_settings::export::ExportBundle;
use hyprland_settings::metadata::resolve_metadata_path_with_env;
use hyprland_settings::ui::model::UiProjection;
use hyprland_settings::validation::validate_bundle;
use hyprland_settings::write_classification::SAFE_WRITABLE_ROWS;

fn load_projection() -> Result<UiProjection> {
    load_projection_with_current_config(CurrentConfigSnapshot::read_unavailable(
        "test fixture has no live config",
    ))
}

fn load_projection_with_current_config(
    current_config: CurrentConfigSnapshot,
) -> Result<UiProjection> {
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
        current_config,
    ))
}

fn read_json(path: &str) -> serde_json::Value {
    serde_json::from_slice(&fs::read(path).expect("report should exist"))
        .expect("report should parse")
}

#[test]
fn dashboard_sidebar_source_records_expected_navigation() {
    let source = fs::read_to_string("src/ui/window.rs").expect("window source should read");

    assert!(source.contains("const DASHBOARD_ID: &str = \"dashboard\""));
    assert!(source.contains("label: \"Dashboard\".to_string()"));
    // The tab ordering now lives in the grouped category model.
    let categories =
        fs::read_to_string("src/ux_presentation.rs").expect("presentation source should read");
    assert!(source.contains("SIDEBAR_CATEGORIES"));
    assert!(categories.contains("\"appearance\""));
    assert!(categories.contains("\"windows-layout\""));
    assert!(categories.contains("\"display\""));
    assert!(categories.contains("\"input\""));
    assert!(categories.contains("\"keybinds\""));
    assert!(categories.contains("\"cursor\""));
    assert!(categories.contains("\"permissions\""));
    assert!(categories.contains("\"system\""));
    assert!(categories.contains("\"animations\""));
    assert!(source.contains("\"keybinds\" => \"Keyboard\".to_string()"));

    assert!(
        !source.contains("format!(\"{} rows\", tab.row_count)"),
        "sidebar must not render row-count labels"
    );
    assert!(
        !source.contains("row_box.append(&count)"),
        "sidebar must not append a row-count widget"
    );
    assert!(source.contains("find(|tab| tab.id == tab_id && tab.row_count > 0)"));
}

#[test]
fn dashboard_source_hides_settings_work_area_and_omits_backend_counts() {
    let source = fs::read_to_string("src/ui/window.rs").expect("window source should read");

    assert!(source.contains("fn build_dashboard_view"));
    assert!(source.contains("fn render_main_view"));
    // Standalone pages share one routed show/hide path.
    assert!(source.contains("dashboard_view.set_visible(page_id == DASHBOARD_ID)"));
    assert!(source.contains("settings_view.set_visible(false)"));
    assert!(source.contains("dashboard_view.set_visible(false)"));
    assert!(source.contains("settings_view.set_visible(true)"));
    assert!(source.contains("render_empty_detail(detail_content)"));

    let dashboard_layout_start = source
        .find("fn build_dashboard_view")
        .expect("dashboard function should exist");
    let dashboard_layout_end = source
        .find("struct DashboardCard")
        .expect("dashboard card struct should exist");
    let dashboard_source = &source[dashboard_layout_start..dashboard_layout_end];
    let dashboard_cards_start = source
        .find("fn dashboard_cards")
        .expect("dashboard cards function should exist");
    let dashboard_cards_end = source
        .find("fn build_dashboard_card")
        .expect("dashboard card builder should exist");
    let dashboard_cards_source = &source[dashboard_cards_start..dashboard_cards_end];

    for card in [
        "Config",
        "Appearance",
        "Windows & Layout",
        "Input",
        "Displays",
        "Shortcuts",
        "Advanced",
    ] {
        assert!(
            dashboard_cards_source.contains(card),
            "missing dashboard card {card}"
        );
    }
    for forbidden in [
        "readable",
        "writable",
        "blocked",
        "official scalar coverage",
    ] {
        assert!(
            !dashboard_source.contains(forbidden),
            "dashboard should not show backend count/proof wording: {forbidden}"
        );
    }
}

#[test]
fn needs_attention_is_backed_by_duplicate_or_warning_projection() -> Result<()> {
    let clean = load_projection()?;
    assert_eq!(clean.current_value_summary.duplicate_conflict_rows, 0);

    let parsed = parse_hyprland_config_text(
        "/tmp/dashboard-sidebar.conf",
        "decoration:blur:enabled = true\ndecoration:blur:enabled = false\n",
    );
    let duplicate =
        load_projection_with_current_config(CurrentConfigSnapshot::from_parsed(parsed))?;
    assert!(duplicate.current_value_summary.duplicate_conflict_rows > 0);

    let source = fs::read_to_string("src/ui/window.rs").expect("window source should read");
    assert!(source.contains("fn dashboard_needs_attention"));
    assert!(source.contains("CurrentConfigLoadStatus::Loaded"));
    assert!(source.contains("duplicate_conflict_rows > 0"));
    assert!(source.contains("parser_warning_rows > 0"));
    assert!(source.contains("Some settings appear more than once in your config."));

    Ok(())
}

#[test]
fn normal_settings_details_remain_available_after_dashboard_change() -> Result<()> {
    let projection = load_projection()?;

    let blur = projection
        .detail_for_row("appearance.blur.enabled")
        .expect("appearance blur detail should exist");
    assert_eq!(blur.label, "Appearance Blur Enabled");
    assert_eq!(blur.official_setting, "decoration.blur.enabled");
    assert!(blur.edit.editable);

    let default_monitor = projection
        .detail_for_row("cursor.default_monitor")
        .expect("cursor default monitor detail should exist");
    assert_eq!(default_monitor.row_id, "cursor.default_monitor");
    assert!(default_monitor
        .edit
        .pending
        .expect("pending projection expected")
        .validation_label
        .contains("runtime monitor-name oracle proof"));

    Ok(())
}

#[test]
fn dashboard_report_preserves_final_counts() {
    let report = read_json("data/reports/dashboard-sidebar-simplification.v0.55.2.json");

    assert_eq!(report["countsBefore"]["readableRows"], 341);
    assert_eq!(report["countsBefore"]["writableRows"], 341);
    assert_eq!(report["countsBefore"]["blockedRows"], 0);
    assert_eq!(report["countsAfter"]["readableRows"], 341);
    assert_eq!(report["countsAfter"]["writableRows"], 341);
    assert_eq!(report["countsAfter"]["blockedRows"], 0);
    assert_eq!(SAFE_WRITABLE_ROWS.len(), 341);
    assert_eq!(report["sidebarAfter"][0], "Dashboard");
    assert!(report["removedTabs"]
        .as_array()
        .expect("removedTabs should be an array")
        .iter()
        .any(|tab| tab == "Overview"));
}
