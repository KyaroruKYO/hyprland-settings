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

fn source_slice<'a>(source: &'a str, start: &str, end: &str) -> &'a str {
    let start = source
        .find(start)
        .expect("source start marker should exist");
    let end = source.find(end).expect("source end marker should exist");
    &source[start..end]
}

#[test]
fn setting_row_source_uses_friendly_copy_and_hides_raw_metadata_labels() {
    let source = fs::read_to_string("src/ui/window.rs").expect("window source should read");
    let row_source = source_slice(&source, "fn build_setting_row", "fn build_detail_panel");

    assert!(row_source.contains("friendly_row_current_status"));
    assert!(row_source.contains("friendly_row_attention_status"));
    assert!(row_source.contains("Uses Hyprland default"));
    assert!(row_source.contains("Extra care needed"));
    assert!(row_source.contains("Needs attention"));
    assert!(row_source.contains("Current: {value}"));

    for hidden in [
        "read-allowlisted metadata",
        "non-read classified metadata",
        "not report-only",
        "editable pilot",
        "write metadata present",
        "preview:",
        "risk:",
        "write support:",
        "current-value-read-allowlisted",
        "not-write-allowlisted",
        "setting.official_setting",
    ] {
        assert!(
            !row_source.contains(hidden),
            "row list should not render raw metadata label: {hidden}"
        );
    }

    assert!(source.contains("Source / advanced metadata"));
    assert!(source.contains("Write support raw label"));
    assert!(source.contains("Report-only status"));
}

#[test]
fn smoke_review_rows_still_have_detail_projection_after_copy_polish() -> Result<()> {
    let projection = load_projection()?;

    for row_id in [
        "appearance.blur.enabled",
        "cursor.default_monitor",
        "debug.manual_crash",
        "decoration.screen_shader",
        "render.direct_scanout",
        "windows.snap.enabled",
    ] {
        let detail = projection
            .detail_for_row(row_id)
            .unwrap_or_else(|| panic!("detail projection should exist for {row_id}"));
        assert_eq!(detail.row_id, row_id);
        assert!(!detail.label.is_empty());
        assert!(!detail.description.is_empty());
    }

    let default_monitor = projection
        .detail_for_row("cursor.default_monitor")
        .expect("cursor.default_monitor detail should exist");
    assert!(default_monitor
        .edit
        .pending
        .expect("pending projection expected")
        .validation_label
        .contains("runtime monitor-name oracle proof"));

    Ok(())
}

#[test]
fn dashboard_cards_keep_open_button_navigation_targets() {
    let source = fs::read_to_string("src/ui/window.rs").expect("window source should read");
    let dashboard_source = source_slice(
        &source,
        "fn dashboard_cards",
        "fn dashboard_needs_attention",
    );

    for (title, target) in [
        ("General", "general"),
        ("Decoration", "decoration"),
        ("Devices", "devices"),
        ("Monitors", "monitors"),
        ("Keybinds", "keybinds"),
        ("System", "system"),
    ] {
        assert!(dashboard_source.contains(title), "missing card {title}");
        assert!(
            dashboard_source.contains(target),
            "missing card target {target}"
        );
    }
    assert!(dashboard_source.contains("gtk::Button::with_label(\"Open\")"));
    assert!(dashboard_source.contains("connect_clicked"));
    assert!(dashboard_source.contains("sidebar.select_row(Some(&row))"));
}

#[test]
fn dashboard_report_records_copy_polish_without_count_changes() {
    let report: serde_json::Value = serde_json::from_slice(
        &fs::read("data/reports/dashboard-card-navigation-row-list-copy-polish.v0.55.2.json")
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
    assert!(report["removedFromVisibleRows"]
        .as_array()
        .expect("removedFromVisibleRows should be an array")
        .iter()
        .any(|item| item == "read support labels"));
}
