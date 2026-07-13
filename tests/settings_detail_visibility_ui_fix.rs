use std::fs;
use std::path::Path;

use anyhow::Result;
use hyprland_settings::config_discovery::{ConfigDiscovery, ConfigDiscoveryStatus};
use hyprland_settings::current_config::CurrentConfigSnapshot;
use hyprland_settings::export::ExportBundle;
use hyprland_settings::metadata::resolve_metadata_path_with_env;
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

fn read_json(path: &str) -> serde_json::Value {
    serde_json::from_slice(&fs::read(path).expect("report should exist"))
        .expect("report should parse")
}

#[test]
fn window_layout_hides_diagnostics_and_uses_centered_column_with_on_demand_detail() {
    let source = fs::read_to_string("src/ui/window.rs").expect("window source should read");

    assert!(!source.contains("fn build_status_diagnostics_expander"));
    assert!(!source.contains("fn build_summary_card"));
    assert!(!source.contains("settings_view.append(&diagnostics)"));
    assert!(!source.contains("Search export metadata"));
    assert!(source.contains("Search settings"));

    // The permanent split work area is gone: settings pages are one
    // centered clamped column, and the detail surface is an on-demand
    // popover anchored to the opened row.
    assert!(!source.contains("gtk::Paned::new(gtk::Orientation::Horizontal)"));
    assert!(source.contains("adw::Clamp::new()"));
    assert!(source.contains("settings_clamp.set_maximum_size(800)"));
    assert!(source.contains("settings_list.add_css_class(\"boxed-list\")"));
    assert!(source.contains("hyprland-settings-detail-popover"));
    assert!(source.contains("detail_popover.set_child(Some(&detail_panel))"));
    assert!(source.contains("detail_popover.popup()"));
    assert!(source.contains("settings_view.append(&settings_scroll)"));
    assert!(
        !source.contains("content.append(&detail_panel);"),
        "detail panel must not sit permanently in the page"
    );
}

#[test]
fn blur_enabled_detail_projection_is_available_for_immediate_display() -> Result<()> {
    let projection = load_projection()?;
    let detail = projection
        .detail_for_row("appearance.blur.enabled")
        .expect("Appearance Blur Enabled detail should exist");

    assert_eq!(detail.label, "Appearance Blur Enabled");
    assert_eq!(detail.row_id, "appearance.blur.enabled");
    assert_eq!(detail.official_setting, "decoration.blur.enabled");
    assert_eq!(detail.tab_label, "Appearance");
    assert!(detail.edit.editable);
    assert!(!detail.read_support.is_empty());
    assert!(!detail.write_support.is_empty());
    assert!(!detail.risk_class.is_empty());
    assert!(detail.edit.proposed_value.is_some());

    Ok(())
}

#[test]
fn high_risk_warning_metadata_remains_available_in_details() -> Result<()> {
    let projection = load_projection()?;
    let detail = projection
        .detail_for_row("cursor.default_monitor")
        .expect("cursor.default_monitor detail should exist");

    assert_eq!(detail.row_id, "cursor.default_monitor");
    assert_eq!(detail.risk_class, "cursor_input_risk");
    assert!(detail.edit.editable);
    assert!(detail
        .edit
        .pending
        .as_ref()
        .expect("pending projection expected")
        .validation_label
        .contains("runtime monitor-name oracle proof"));
    assert!(detail.write_support.contains("write"));

    Ok(())
}

#[test]
fn ui_fix_report_preserves_final_coverage_counts() {
    let report = read_json("data/reports/settings-detail-visibility-ui-fix.v0.55.2.json");

    assert_eq!(report["countsBefore"]["readableRows"], 341);
    assert_eq!(report["countsBefore"]["writableRows"], 341);
    assert_eq!(report["countsBefore"]["blockedRows"], 0);
    assert_eq!(report["countsAfter"]["readableRows"], 341);
    assert_eq!(report["countsAfter"]["writableRows"], 341);
    assert_eq!(report["countsAfter"]["blockedRows"], 0);
    assert_eq!(SAFE_WRITABLE_ROWS.len(), 341);
    assert_eq!(
        report["diagnosticsPanelBehavior"],
        "collapsed-by-default-with-bounded-scroll"
    );
    assert_eq!(
        report["detailsPaneBehavior"],
        "selected-setting-detail-pane-is-a-sibling-of-the-list-in-a-horizontal-paned-work-area"
    );
}
