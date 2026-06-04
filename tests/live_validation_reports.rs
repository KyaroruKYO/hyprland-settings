use std::collections::BTreeSet;

use anyhow::Result;
use serde_json::Value;

fn plan_report() -> Result<Value> {
    Ok(serde_json::from_str(include_str!(
        "../data/reports/live-validation-plan.v0.55.2.json"
    ))?)
}

fn manual_report() -> Result<Value> {
    Ok(serde_json::from_str(include_str!(
        "../data/reports/manual-review-write-candidates.v0.55.2.json"
    ))?)
}

#[test]
fn live_validation_plan_contains_only_batch_a_rows() -> Result<()> {
    let plan = plan_report()?;
    let manual = manual_report()?;
    let rows = plan["rows"]
        .as_array()
        .expect("live validation plan rows should be an array");
    let batch_a = manual["items"]
        .as_array()
        .expect("manual report items should be an array")
        .iter()
        .filter(|item| item["recommendedBatch"].as_str() == Some("batch-a-likely-safe-booleans"))
        .map(|item| item["rowId"].as_str().expect("rowId should be a string"))
        .collect::<BTreeSet<_>>();
    let plan_ids = rows
        .iter()
        .map(|row| row["row_id"].as_str().expect("row_id should be a string"))
        .collect::<BTreeSet<_>>();

    assert_eq!(plan["counts"]["rows"], 39);
    assert_eq!(rows.len(), 39);
    assert_eq!(plan_ids, batch_a);
    for row in rows {
        assert_eq!(
            row["batch"].as_str(),
            Some("batch-a-likely-safe-booleans")
        );
        assert_eq!(row["value_kind"].as_str(), Some("boolean"));
        assert_eq!(row["high_risk"].as_bool(), Some(false));
        assert!(
            row["rollback_deadline_seconds"]
                .as_u64()
                .expect("rollback deadline should be a number")
                <= 10
        );
        let candidates = row["candidate_values"]
            .as_array()
            .expect("candidate_values should be an array")
            .iter()
            .map(|value| value.as_str().expect("candidate should be a string"))
            .collect::<BTreeSet<_>>();
        assert_eq!(candidates, ["false", "true"].into_iter().collect());
    }

    Ok(())
}
