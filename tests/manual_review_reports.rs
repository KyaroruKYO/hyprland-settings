use std::collections::BTreeSet;

use anyhow::Result;
use serde_json::Value;

fn coverage_report() -> Result<Value> {
    Ok(serde_json::from_str(include_str!(
        "../data/reports/scalar-read-write-coverage.v0.55.2.json"
    ))?)
}

fn manual_review_report() -> Result<Value> {
    Ok(serde_json::from_str(include_str!(
        "../data/reports/manual-review-write-candidates.v0.55.2.json"
    ))?)
}

fn high_risk_report() -> Result<Value> {
    Ok(serde_json::from_str(include_str!(
        "../data/reports/high-risk-write-candidates.v0.55.2.json"
    ))?)
}

fn remaining_scalar_completion_report() -> Result<Value> {
    Ok(serde_json::from_str(include_str!(
        "../data/reports/remaining-scalar-completion.v0.55.2.json"
    ))?)
}

#[test]
fn manual_review_and_high_risk_reports_have_expected_counts() -> Result<()> {
    let manual = manual_review_report()?;
    let high_risk = high_risk_report()?;

    assert_eq!(manual["counts"]["rows"], 16);
    assert_eq!(high_risk["counts"]["rows"], 72);
    assert_eq!(manual["invariants"]["writableRowsRemain"], 253);
    assert_eq!(high_risk["invariants"]["writableRowsRemain"], 253);

    Ok(())
}

#[test]
fn manual_review_and_high_risk_reports_do_not_overlap() -> Result<()> {
    let manual = manual_review_report()?;
    let high_risk = high_risk_report()?;
    let manual_ids = manual["items"]
        .as_array()
        .expect("manual report items should be an array")
        .iter()
        .map(|item| item["rowId"].as_str().expect("rowId should be a string"))
        .collect::<BTreeSet<_>>();
    let high_risk_ids = high_risk["items"]
        .as_array()
        .expect("high-risk report items should be an array")
        .iter()
        .map(|item| item["rowId"].as_str().expect("rowId should be a string"))
        .collect::<BTreeSet<_>>();

    assert!(manual_ids.is_disjoint(&high_risk_ids));
    assert_eq!(manual_ids.len(), 16);
    assert_eq!(high_risk_ids.len(), 72);

    Ok(())
}

#[test]
fn candidate_reports_match_scalar_coverage_statuses() -> Result<()> {
    let coverage = coverage_report()?;
    let manual = manual_review_report()?;
    let high_risk = high_risk_report()?;
    let rows = coverage["rows"]
        .as_array()
        .expect("coverage rows should be an array");

    for item in manual["items"]
        .as_array()
        .expect("manual report items should be an array")
    {
        let row_id = item["rowId"].as_str().expect("rowId should be a string");
        let source = rows
            .iter()
            .find(|row| row["rowId"].as_str() == Some(row_id))
            .unwrap_or_else(|| panic!("missing coverage row {row_id}"));
        assert_eq!(
            source["writeStatus"].as_str(),
            Some("manual-review-needed"),
            "{row_id} should still be manual-review-needed"
        );
        assert_eq!(item["userApprovalRequired"].as_bool(), Some(true));
        assert!(item["recommendedBatch"].as_str().is_some());
        assert!(item["recommendedAction"].as_str().is_some());
        assert!(
            item["testPlan"]
                .as_array()
                .expect("testPlan should be an array")
                .len()
                >= 6,
            "{row_id} should include a concrete test plan"
        );
    }

    for item in high_risk["items"]
        .as_array()
        .expect("high-risk report items should be an array")
    {
        let row_id = item["rowId"].as_str().expect("rowId should be a string");
        let source = rows
            .iter()
            .find(|row| row["rowId"].as_str() == Some(row_id))
            .unwrap_or_else(|| panic!("missing coverage row {row_id}"));
        assert_eq!(
            source["writeStatus"].as_str(),
            Some("high-risk"),
            "{row_id} should still be high-risk"
        );
        assert_eq!(item["userApprovalRequired"].as_bool(), Some(true));
        assert!(item["whyNotWritableYet"].as_str().is_some());
        assert!(item["whatWouldBeRequired"].as_str().is_some());
    }

    Ok(())
}

#[test]
fn scalar_coverage_counts_reflect_remaining_scalar_completion() -> Result<()> {
    let coverage = coverage_report()?;
    let rows = coverage["rows"]
        .as_array()
        .expect("coverage rows should be an array");
    let writable = rows
        .iter()
        .filter(|row| row["writeStatus"].as_str() == Some("writable"))
        .count();
    let manual = rows
        .iter()
        .filter(|row| row["writeStatus"].as_str() == Some("manual-review-needed"))
        .count();
    let high_risk = rows
        .iter()
        .filter(|row| row["writeStatus"].as_str() == Some("high-risk"))
        .count();
    let parser_needed = rows
        .iter()
        .filter(|row| row["writeStatus"].as_str() == Some("parser-needed"))
        .count();
    let validator_needed = rows
        .iter()
        .filter(|row| row["writeStatus"].as_str() == Some("validator-needed"))
        .count();

    assert_eq!(coverage["counts"]["writableRows"], 253);
    assert_eq!(coverage["counts"]["blockedWriteRows"], 88);
    assert_eq!(writable, 253);
    assert_eq!(manual, 16);
    assert_eq!(high_risk, 72);
    assert_eq!(parser_needed, 0);
    assert_eq!(validator_needed, 0);

    Ok(())
}

#[test]
fn remaining_scalar_completion_report_records_enabled_and_blocked_rows() -> Result<()> {
    let report = remaining_scalar_completion_report()?;

    assert_eq!(report["counts"]["startingWritableRows"], 94);
    assert_eq!(report["counts"]["finalWritableRows"], 253);
    assert_eq!(report["counts"]["enabledRows"], 159);
    assert_eq!(
        report["counts"]["enabledByBatch"]["batch-b-likely-safe-numerics"],
        33
    );
    assert_eq!(
        report["counts"]["enabledByBatch"]["batch-c-likely-safe-enums"],
        7
    );
    assert_eq!(
        report["counts"]["enabledByBatch"]["batch-d-input-behavior"],
        74
    );
    assert_eq!(
        report["counts"]["enabledByBatch"]["batch-e-window-layout-behavior"],
        45
    );
    assert!(report["counts"]["remainingBlockedByBatch"]
        .get("batch-c-likely-safe-enums")
        .is_none());
    assert!(report["counts"]["remainingBlockedByBatch"]
        .get("batch-d-input-behavior")
        .is_none());
    assert_eq!(
        report["counts"]["remainingBlockedByBatch"]["batch-g-session-runtime-sensitive"],
        16
    );
    assert_eq!(report["counts"]["remainingBlockedByBatch"]["high-risk"], 72);
    assert_eq!(report["counts"]["hyprlandVerifyConfigPassed"], 155);
    assert_eq!(report["counts"]["hyprlandVerifyConfigFailed"], 0);

    let rows = report["rows"]
        .as_array()
        .expect("remaining completion rows should be an array");
    assert_eq!(rows.len(), 247);
    assert_eq!(
        rows.iter()
            .filter(|row| row["enabled"].as_bool() == Some(true))
            .count(),
        159
    );
    assert_eq!(
        rows.iter()
            .filter(|row| row["enabled"].as_bool() == Some(false))
            .count(),
        88
    );

    Ok(())
}
