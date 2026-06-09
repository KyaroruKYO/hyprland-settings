use std::collections::BTreeSet;
use std::fs;

use hyprland_settings::write_classification::{
    high_risk_write_policy, is_high_risk_gated_writable_setting, is_safe_writable_setting,
    SAFE_WRITABLE_ROWS,
};

fn read_json(path: &str) -> serde_json::Value {
    serde_json::from_slice(&fs::read(path).expect("report should exist"))
        .expect("report should parse")
}

#[test]
fn final_341_writable_coverage_report_records_no_blocked_rows() {
    let report = read_json("data/reports/final-341-writable-coverage.v0.55.2.json");
    assert_eq!(report["readableRows"], 341);
    assert_eq!(report["writableRows"], 341);
    assert_eq!(report["blockedRows"], 0);
    assert_eq!(
        report["remainingBlockedRows"]
            .as_array()
            .expect("remainingBlockedRows should be an array")
            .len(),
        0
    );
    assert_eq!(
        report["cursorDefaultMonitorStatus"],
        "writable-through-gated-high-risk-path-with-runtime-monitor-name-oracle-proof"
    );
}

#[test]
fn current_aggregate_reports_match_final_341_state() {
    let coverage = read_json("data/reports/scalar-read-write-coverage.v0.55.2.json");
    assert_eq!(coverage["counts"]["readableRows"], 341);
    assert_eq!(coverage["counts"]["writableRows"], 341);
    assert_eq!(coverage["counts"]["blockedWriteRows"], 0);

    let pipeline = read_json("data/reports/all-341-unified-pipeline.v0.55.2.json");
    assert_eq!(pipeline["counts"]["totalRows"], 341);
    assert_eq!(pipeline["counts"]["readableRows"], 341);
    assert_eq!(pipeline["counts"]["writableRows"], 341);
    assert_eq!(pipeline["counts"]["blockedRows"], 0);
    assert_eq!(pipeline["counts"]["safeWritableRowsFromRustTable"], 341);
    assert!(pipeline["sourceReports"]
        .as_array()
        .expect("sourceReports should be an array")
        .iter()
        .any(|value| value == "data/reports/final-341-writable-coverage.v0.55.2.json"));
}

#[test]
fn rust_safe_writable_table_contains_all_341_unique_rows() {
    assert_eq!(SAFE_WRITABLE_ROWS.len(), 341);
    let unique: BTreeSet<_> = SAFE_WRITABLE_ROWS.iter().map(|row| row.row_id).collect();
    assert_eq!(unique.len(), 341);
    assert!(is_safe_writable_setting("cursor.default_monitor"));
    assert!(is_high_risk_gated_writable_setting(
        "cursor.default_monitor"
    ));
    assert!(high_risk_write_policy("cursor.default_monitor")
        .expect("cursor.default_monitor should have a high-risk policy")
        .review_warning
        .contains("runtime monitor-name oracle proof"));
}

#[test]
fn all_pipeline_rows_are_writable_and_screen_shader_remains_gated() {
    let pipeline = read_json("data/reports/all-341-unified-pipeline.v0.55.2.json");
    let rows = pipeline["rows"]
        .as_array()
        .expect("rows should be an array");
    assert_eq!(rows.len(), 341);
    assert!(rows.iter().all(|row| row["writeStatus"] == "writable"));

    let screen_shader = rows
        .iter()
        .find(|row| row["rowId"] == "decoration.screen_shader")
        .expect("screen shader row should be present");
    assert_eq!(screen_shader["writeStatus"], "writable");
    assert!(screen_shader["approvalGate"]
        .as_str()
        .expect("approval gate should be text")
        .contains("screen-shader"));
}
