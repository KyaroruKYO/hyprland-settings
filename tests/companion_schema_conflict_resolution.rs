use std::collections::BTreeMap;

use anyhow::Result;
use hyprland_settings::write_classification::{
    finite_choice_options, safe_writable_value_kind, ScalarWriteValueKind,
    CONFLICT_FINITE_CHOICE_ROWS, SAFE_WRITABLE_ROWS,
};
use serde_json::Value;

fn conflict_resolution_report() -> Result<Value> {
    Ok(serde_json::from_str(include_str!(
        "../data/reports/companion-schema-conflict-resolution.v0.55.2.json"
    ))?)
}

fn coverage_report() -> Result<Value> {
    Ok(serde_json::from_str(include_str!(
        "../data/reports/scalar-read-write-coverage.v0.55.2.json"
    ))?)
}

#[test]
fn conflict_resolution_report_tracks_verified_numeric_dropdown_migration() -> Result<()> {
    let report = conflict_resolution_report()?;
    let rows = report["rows"].as_array().expect("rows should be an array");

    assert_eq!(report["counts"]["rows"], 12);
    assert_eq!(report["counts"]["migratableRows"], 12);
    assert_eq!(report["counts"]["notMigratedRows"], 0);
    assert_eq!(report["counts"]["numericBackedRows"], 12);
    assert_eq!(report["counts"]["semanticBackedRows"], 0);
    assert_eq!(report["counts"]["activeConfigModified"], false);
    assert_eq!(report["counts"]["activeRuntimeModified"], false);

    for row in rows {
        let row_id = row["rowId"].as_str().expect("rowId should be string");
        let options = finite_choice_options(row_id)
            .unwrap_or_else(|| panic!("{row_id} should have finite choices"));
        let chosen_raw: Vec<_> = row["chosenRawValues"]
            .as_array()
            .expect("chosen raw values should be an array")
            .iter()
            .map(|value| value.as_str().unwrap())
            .collect();
        let chosen_labels: Vec<_> = row["chosenDisplayLabels"]
            .as_array()
            .expect("chosen labels should be an array")
            .iter()
            .map(|value| value.as_str().unwrap())
            .collect();

        assert_eq!(
            chosen_raw,
            options
                .iter()
                .map(|option| option.raw_value)
                .collect::<Vec<_>>(),
            "{row_id} raw values should match production choices"
        );
        assert_eq!(
            chosen_labels,
            options
                .iter()
                .map(|option| option.label)
                .collect::<Vec<_>>(),
            "{row_id} labels should match production choices"
        );
        assert_eq!(
            row["chosenStorageMode"].as_str(),
            Some("numeric-raw-values-with-semantic-labels"),
            "{row_id} should store verified numeric raw values"
        );
        assert_eq!(row["migratable"], true, "{row_id} should be migratable");
    }

    Ok(())
}

#[test]
fn conflict_rows_are_finite_choice_without_changing_write_allowlist_size() {
    assert_eq!(SAFE_WRITABLE_ROWS.len(), 269);
    for row_id in CONFLICT_FINITE_CHOICE_ROWS {
        assert_eq!(
            safe_writable_value_kind(row_id),
            Some(ScalarWriteValueKind::FiniteChoice),
            "{row_id} should use finite-choice validation"
        );
    }
}

#[test]
fn scalar_coverage_marks_conflict_rows_as_dropdown_finite_choice() -> Result<()> {
    let coverage = coverage_report()?;
    assert_eq!(coverage["counts"]["writableRows"], 269);

    let coverage_by_row: BTreeMap<_, _> = coverage["rows"]
        .as_array()
        .expect("coverage rows should be an array")
        .iter()
        .map(|row| (row["rowId"].as_str().unwrap(), row))
        .collect();

    for row_id in CONFLICT_FINITE_CHOICE_ROWS {
        let row = coverage_by_row
            .get(row_id)
            .unwrap_or_else(|| panic!("{row_id} missing from coverage report"));
        assert_eq!(row["writeStatus"].as_str(), Some("writable"), "{row_id}");
        assert_eq!(row["controlKind"].as_str(), Some("dropdown"), "{row_id}");
        assert_eq!(
            row["valueFamily"].as_str(),
            Some("finite-choice"),
            "{row_id}"
        );
        assert_eq!(row["safeWriteSupported"], true, "{row_id}");
    }

    Ok(())
}
