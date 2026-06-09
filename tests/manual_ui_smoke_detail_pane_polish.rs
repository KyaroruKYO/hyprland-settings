use std::fs;
use std::path::Path;

use anyhow::Result;
use hyprland_settings::config_discovery::{ConfigDiscovery, ConfigDiscoveryStatus};
use hyprland_settings::config_parser::parse_hyprland_config_text;
use hyprland_settings::current_config::{CurrentConfigSnapshot, CurrentValueSourceStatus};
use hyprland_settings::export::ExportBundle;
use hyprland_settings::metadata::resolve_metadata_path_with_env;
use hyprland_settings::ui::model::{RowDetailProjection, UiProjection};
use hyprland_settings::validation::validate_bundle;
use hyprland_settings::write_classification::{high_risk_write_policy, SAFE_WRITABLE_ROWS};

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

fn detail_for(projection: &UiProjection, row_id: &str) -> RowDetailProjection {
    projection
        .detail_for_row(row_id)
        .unwrap_or_else(|| panic!("missing detail for {row_id}"))
}

fn read_json(path: &str) -> serde_json::Value {
    serde_json::from_slice(&fs::read(path).expect("report should exist"))
        .expect("report should parse")
}

#[test]
fn detail_pane_source_uses_user_facing_sections_and_advanced_expander() {
    let source = fs::read_to_string("src/ui/window.rs").expect("window source should read");

    for section in ["Setting", "Current value", "Edit", "Safety"] {
        assert!(
            source.contains(&format!(
                "append_detail_section(detail_content, \"{section}\""
            )),
            "missing {section} detail section"
        );
    }
    assert!(source.contains("append_advanced_detail_expander"));
    assert!(source.contains("Source / advanced metadata"));
    assert!(source.contains("append_user_facing_write_reason"));
    assert!(source.contains("append_safety_summary"));
    assert!(source.contains("This setting appears more than once in your config"));
    assert!(source.contains("not a freeform string write"));
    assert!(source.contains("crash/debug sensitive"));
}

#[test]
fn smoke_review_rows_have_detail_projection_and_expected_risk_metadata() -> Result<()> {
    let projection = load_projection()?;
    let rows = [
        "appearance.blur.enabled",
        "cursor.default_monitor",
        "debug.manual_crash",
        "decoration.screen_shader",
        "render.direct_scanout",
        "windows.snap.enabled",
    ];

    for row_id in rows {
        let detail = detail_for(&projection, row_id);
        assert_eq!(detail.row_id, row_id);
        assert!(!detail.label.is_empty());
        assert!(!detail.official_setting.is_empty());
        assert!(!detail.tab_label.is_empty());
        assert!(!detail.write_support.is_empty());
        assert!(detail.edit.editable, "{row_id} should remain editable");
    }

    assert!(detail_for(&projection, "cursor.default_monitor")
        .edit
        .pending
        .expect("cursor.default_monitor pending projection expected")
        .validation_label
        .contains("runtime monitor-name oracle proof"));
    assert!(high_risk_write_policy("debug.manual_crash")
        .expect("manual crash should have high-risk policy")
        .review_warning
        .contains("debug.manual_crash is gated"));
    assert!(detail_for(&projection, "decoration.screen_shader")
        .screen_shader_advisory
        .expect("screen shader advisory should exist")
        .production_gate_disclaimer
        .contains("production screen-shader watchdog gate"));
    assert!(high_risk_write_policy("render.direct_scanout")
        .expect("direct scanout should have display/render policy")
        .review_warning
        .contains("display safety proof"));
    assert!(
        high_risk_write_policy("windows.snap.enabled").is_none(),
        "windows.snap.enabled is the normal writable-row comparison"
    );

    Ok(())
}

#[test]
fn duplicate_blur_enabled_conflict_has_user_facing_data_to_explain_blocked_apply() -> Result<()> {
    let parsed = parse_hyprland_config_text(
        "/tmp/manual-ui-smoke.conf",
        "decoration:blur:enabled = true\ndecoration:blur:enabled = false\n",
    );
    let projection =
        load_projection_with_current_config(CurrentConfigSnapshot::from_parsed(parsed))?;
    let detail = detail_for(&projection, "appearance.blur.enabled");

    assert_eq!(
        detail.current_value.status,
        CurrentValueSourceStatus::DuplicateConflict
    );
    assert_eq!(detail.current_value.duplicate_lines, vec![1, 2]);
    let pending = detail.edit.pending.expect("pending projection expected");
    assert!(!pending.can_review);
    assert!(pending
        .review_summary
        .iter()
        .any(|line| line.contains("duplicate config entries must be resolved manually")));

    Ok(())
}

#[test]
fn polish_report_preserves_final_counts_and_records_smoke_rows() {
    let report = read_json("data/reports/manual-ui-smoke-review-detail-pane-polish.v0.55.2.json");

    assert_eq!(report["countsBefore"]["readableRows"], 341);
    assert_eq!(report["countsBefore"]["writableRows"], 341);
    assert_eq!(report["countsBefore"]["blockedRows"], 0);
    assert_eq!(report["countsAfter"]["readableRows"], 341);
    assert_eq!(report["countsAfter"]["writableRows"], 341);
    assert_eq!(report["countsAfter"]["blockedRows"], 0);
    assert_eq!(SAFE_WRITABLE_ROWS.len(), 341);

    let reviewed = report["rowsManuallyReviewed"]
        .as_array()
        .expect("rowsManuallyReviewed should be an array");
    for row_id in [
        "appearance.blur.enabled",
        "cursor.default_monitor",
        "debug.manual_crash",
        "decoration.screen_shader",
        "render.direct_scanout",
        "windows.snap.enabled",
    ] {
        assert!(
            reviewed.iter().any(|row| row["rowId"] == row_id),
            "{row_id} should be recorded in smoke review"
        );
    }

    assert_eq!(
        report["advancedMetadataBehavior"],
        "raw/internal metadata moved into collapsed Source / advanced metadata expander"
    );
}
