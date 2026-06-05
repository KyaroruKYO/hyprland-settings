use std::collections::BTreeSet;
use std::path::Path;

use anyhow::Result;
use hyprland_settings::config_discovery::{ConfigDiscovery, ConfigDiscoveryStatus};
use hyprland_settings::config_parser::parse_hyprland_config_text;
use hyprland_settings::current_config::{CurrentConfigSnapshot, CurrentValueSourceStatus};
use hyprland_settings::export::ExportBundle;
use hyprland_settings::metadata::resolve_metadata_path_with_env;
use hyprland_settings::ui::model::UiProjection;
use hyprland_settings::validation::validate_bundle;
use hyprland_settings::write_classification::SAFE_WRITABLE_ROWS;
use serde_json::Value;

fn load_bundle() -> Result<ExportBundle> {
    let resolution = resolve_metadata_path_with_env(None, None)?;
    ExportBundle::load(Path::new(&resolution.export_dir))
}

fn load_projection_with_current_config(
    bundle: &ExportBundle,
    current_config: CurrentConfigSnapshot,
) -> Result<UiProjection> {
    let summary = validate_bundle(bundle)?;
    Ok(UiProjection::from_bundle(
        bundle,
        &summary,
        ConfigDiscovery {
            status: ConfigDiscoveryStatus::Missing,
            attempted_paths: Vec::new(),
        },
        current_config,
    ))
}

fn official_setting_to_config_key(setting: &str) -> String {
    setting.replace('.', ":")
}

#[test]
fn raw_scalar_parser_projects_all_inventory_rows_as_readable() -> Result<()> {
    let bundle = load_bundle()?;
    let mut fixture = String::new();
    for setting in &bundle.inventory.settings {
        fixture.push_str(&format!(
            "{} = raw-value-{}\n",
            official_setting_to_config_key(&setting.official_setting),
            setting.row_order
        ));
    }
    let parsed = parse_hyprland_config_text("/tmp/all-scalar-current-values.conf", &fixture);
    let projection =
        load_projection_with_current_config(&bundle, CurrentConfigSnapshot::from_parsed(parsed))?;

    assert_eq!(projection.settings.len(), 341);
    let unreadable: Vec<_> = projection
        .settings
        .iter()
        .filter(|setting| setting.current_value.status != CurrentValueSourceStatus::Configured)
        .map(|setting| setting.row_id.as_str())
        .collect();

    assert!(
        unreadable.is_empty(),
        "all inventory rows should be raw-readable, unreadable: {unreadable:?}"
    );
    assert_eq!(projection.current_value_summary.total_rows, 341);
    assert_eq!(projection.current_value_summary.readable_rows, 341);
    assert_eq!(projection.current_value_summary.unreadable_rows, 0);
    assert_eq!(projection.current_value_summary.configured_rows, 341);
    assert_eq!(projection.current_value_summary.duplicate_conflict_rows, 0);

    Ok(())
}

#[test]
fn scalar_coverage_report_contains_all_rows_with_explicit_statuses() -> Result<()> {
    let bundle = load_bundle()?;
    let report: Value = serde_json::from_str(include_str!(
        "../data/reports/scalar-read-write-coverage.v0.55.2.json"
    ))?;
    let rows = report["rows"]
        .as_array()
        .expect("coverage report rows should be an array");
    let report_ids = rows
        .iter()
        .map(|row| row["rowId"].as_str().expect("rowId should be a string"))
        .collect::<BTreeSet<_>>();
    let inventory_ids = bundle
        .inventory
        .settings
        .iter()
        .map(|setting| setting.row_id.as_str())
        .collect::<BTreeSet<_>>();

    assert_eq!(rows.len(), 341);
    assert_eq!(report_ids, inventory_ids);
    assert_eq!(report["counts"]["readableRows"], 341);
    assert_eq!(report["counts"]["blockedReadRows"], 0);
    assert_eq!(report["counts"]["writableRows"], 272);
    assert_eq!(report["counts"]["blockedWriteRows"], 69);
    for row in rows {
        assert_eq!(row["readStatus"].as_str(), Some("readable"));
        assert_eq!(row["parserSupported"].as_bool(), Some(true));
        assert!(row["writeStatus"].as_str().is_some());
        if row["writeStatus"].as_str() == Some("writable") {
            assert_eq!(row["validatorSupported"].as_bool(), Some(true));
            assert_eq!(row["safeWriteSupported"].as_bool(), Some(true));
            assert_eq!(row["testsPresent"].as_bool(), Some(true));
        } else {
            assert!(
                row["writeBlocker"].as_str().is_some(),
                "{} needs a write blocker",
                row["rowId"]
            );
        }
    }

    Ok(())
}

#[test]
fn coverage_report_enforces_safe_writable_rows() -> Result<()> {
    let report: Value = serde_json::from_str(include_str!(
        "../data/reports/scalar-read-write-coverage.v0.55.2.json"
    ))?;
    let rows = report["rows"]
        .as_array()
        .expect("coverage report rows should be an array");
    let writable = rows
        .iter()
        .filter(|row| row["writeStatus"].as_str() == Some("writable"))
        .map(|row| row["rowId"].as_str().expect("rowId should be a string"))
        .collect::<BTreeSet<_>>();
    let expected = SAFE_WRITABLE_ROWS
        .iter()
        .map(|row| row.row_id)
        .collect::<BTreeSet<_>>();
    let allowed_statuses = [
        "writable",
        "blocked",
        "deferred",
        "unsupported",
        "high-risk",
        "structured",
        "parser-needed",
        "validator-needed",
        "manual-review-needed",
    ]
    .into_iter()
    .collect::<BTreeSet<_>>();

    assert_eq!(writable, expected);
    for row in rows {
        let status = row["writeStatus"]
            .as_str()
            .expect("writeStatus should be a string");
        assert!(
            allowed_statuses.contains(status),
            "unknown write status {status}"
        );
        if status != "writable" {
            assert!(
                row["writeBlocker"].as_str().is_some(),
                "{} must have a blocker",
                row["rowId"]
            );
        }
    }

    Ok(())
}

#[test]
fn coverage_report_has_no_remaining_parser_or_validator_needed_write_rows() -> Result<()> {
    let report: Value = serde_json::from_str(include_str!(
        "../data/reports/scalar-read-write-coverage.v0.55.2.json"
    ))?;
    let rows = report["rows"]
        .as_array()
        .expect("coverage report rows should be an array");

    for row in rows {
        let row_id = row["rowId"].as_str().expect("rowId should be a string");
        let status = row["writeStatus"]
            .as_str()
            .expect("writeStatus should be a string");
        assert_ne!(status, "parser-needed", "{row_id} still needs a parser");
        assert_ne!(
            status, "validator-needed",
            "{row_id} still needs a validator"
        );
        if status == "writable" {
            assert_eq!(row["parserSupported"].as_bool(), Some(true), "{row_id}");
            assert_eq!(row["validatorSupported"].as_bool(), Some(true), "{row_id}");
            assert_eq!(row["safeWriteSupported"].as_bool(), Some(true), "{row_id}");
            assert_eq!(row["testsPresent"].as_bool(), Some(true), "{row_id}");
        }
    }

    Ok(())
}
