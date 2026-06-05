use std::collections::BTreeSet;

use anyhow::Result;
use hyprland_settings::write_classification::SAFE_WRITABLE_ROWS;
use serde_json::Value;

fn read_json(path: &str) -> Result<Value> {
    Ok(serde_json::from_str(&std::fs::read_to_string(path)?)?)
}

#[test]
fn next_display_render_selection_reports_exist_and_keep_counts_unchanged() -> Result<()> {
    let coverage = read_json("data/reports/scalar-read-write-coverage.v0.55.2.json")?;
    let smoke = read_json("data/reports/display-render-smoke-subset-review.v0.55.2.json")?;
    let selection = read_json("data/reports/next-display-render-subset-selection.v0.55.2.json")?;
    let proof_plan =
        read_json("data/reports/next-display-render-subset-readiness-proof-plan.v0.55.2.json")?;

    assert_eq!(coverage["counts"]["writableRows"], 274);
    assert_eq!(coverage["counts"]["blockedWriteRows"], 67);
    assert_eq!(SAFE_WRITABLE_ROWS.len(), 274);

    assert_eq!(smoke["counts"]["reviewedRows"], 2);
    assert_eq!(smoke["counts"]["enabledRows"], 2);
    assert_eq!(smoke["counts"]["issuesFound"], 0);
    assert_eq!(smoke["counts"]["regressionsFound"], 0);
    assert_eq!(smoke["counts"]["warningsFound"], 0);

    assert_eq!(
        selection["counts"]["remainingDisplayRenderRowsReviewed"],
        23
    );
    assert_eq!(selection["counts"]["selectedRows"], 0);
    assert_eq!(selection["counts"]["rowsEnabled"], 0);
    assert_eq!(selection["counts"]["finalWritableRows"], 274);
    assert_eq!(selection["counts"]["finalBlockedRows"], 67);
    assert_eq!(selection["counts"]["displayRenderBlockedRows"], 23);
    assert_eq!(selection["counts"]["cursorInputBlockedRows"], 22);
    assert_eq!(selection["counts"]["debugCrashBlockedRows"], 22);
    assert_eq!(selection["counts"]["writeAllowlistChanged"], false);
    assert_eq!(selection["counts"]["productionBehaviorChanged"], false);
    assert_eq!(selection["selectionDecision"].as_str(), Some("select-none"));

    assert_eq!(proof_plan["counts"]["selectedRows"], 0);
    assert_eq!(proof_plan["counts"]["plannedRows"], 0);
    assert_eq!(proof_plan["counts"]["rowsEnabled"], 0);
    assert_eq!(proof_plan["counts"]["writeAllowlistChanged"], false);
    assert_eq!(proof_plan["counts"]["productionBehaviorChanged"], false);

    Ok(())
}

#[test]
fn selected_subset_is_empty_and_hard_excluded_rows_are_not_selected() -> Result<()> {
    let selection = read_json("data/reports/next-display-render-subset-selection.v0.55.2.json")?;

    let selected = selection["selectedRows"]
        .as_array()
        .expect("selected rows should be an array");
    assert!(selected.len() <= 3);
    assert!(selected.is_empty());

    let excluded_ids = [
        "xwayland.enabled",
        "xwayland.create_abstract_socket",
        "render.new_render_scheduling",
        "render.commit_timing_enabled",
    ]
    .into_iter()
    .collect::<BTreeSet<_>>();

    let rows = selection["rows"]
        .as_array()
        .expect("selection rows should be an array");
    assert_eq!(rows.len(), 23);
    for row in rows {
        let row_id = row["rowId"].as_str().unwrap();
        assert_eq!(
            row["selectedForNextSubset"].as_bool(),
            Some(false),
            "{row_id} should not be selected in this sprint"
        );
        if excluded_ids.contains(row_id) {
            assert_eq!(
                row["excludedByHardRule"].as_bool(),
                Some(true),
                "{row_id} should be hard-excluded"
            );
        }
        assert_ne!(
            row["excludedReason"].as_str(),
            Some(""),
            "{row_id} should have an explicit exclusion reason"
        );
    }

    Ok(())
}

#[test]
fn no_new_display_render_rows_became_writable() -> Result<()> {
    let coverage = read_json("data/reports/scalar-read-write-coverage.v0.55.2.json")?;
    let writable_ids = coverage["rows"]
        .as_array()
        .unwrap()
        .iter()
        .filter(|row| row["writeStatus"].as_str() == Some("writable"))
        .map(|row| row["rowId"].as_str().unwrap())
        .collect::<BTreeSet<_>>();

    assert!(writable_ids.contains("xwayland.use_nearest_neighbor"));
    assert!(writable_ids.contains("xwayland.force_zero_scaling"));
    assert!(!writable_ids.contains("xwayland.enabled"));
    assert!(!writable_ids.contains("xwayland.create_abstract_socket"));
    assert!(!writable_ids.contains("render.new_render_scheduling"));
    assert!(!writable_ids.contains("render.cm_enabled"));
    assert!(!writable_ids.contains("experimental.wp_cm_1_2"));

    Ok(())
}
