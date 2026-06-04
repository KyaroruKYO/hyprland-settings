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

fn results_report() -> Result<Value> {
    Ok(serde_json::from_str(include_str!(
        "../data/reports/live-validation-results.v0.55.2.json"
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
        assert_eq!(row["batch"].as_str(), Some("batch-a-likely-safe-booleans"));
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

#[test]
fn live_results_record_batch_a_probe_without_enablement() -> Result<()> {
    let results = results_report()?;
    let rows = results["rows"]
        .as_array()
        .expect("live validation result rows should be an array");

    assert_eq!(results["mode"].as_str(), Some("live"));
    assert_eq!(results["counts"]["rows"], 39);
    assert_eq!(results["counts"]["level1_passed"], 39);
    assert_eq!(results["counts"]["level2_passed"], 39);
    assert_eq!(results["counts"]["level3_passed"], 0);
    assert_eq!(results["counts"]["level4_passed"], 39);
    assert_eq!(results["counts"]["level5_manual_observation"], 39);
    assert_eq!(results["counts"]["enabled_rows"], 0);

    for row in rows {
        assert_eq!(row["level1_parse_read_status"].as_str(), Some("passed"));
        assert_eq!(
            row["level2_fixture_write_reread_status"].as_str(),
            Some("passed")
        );
        assert_eq!(
            row["level3_hyprland_accepts_value_status"].as_str(),
            Some("rejected")
        );
        assert_eq!(row["level4_revert_status"].as_str(), Some("passed"));
        assert_eq!(
            row["level5_behavior_status"].as_str(),
            Some("requires-manual-observation")
        );
        assert_eq!(row["safe_to_enable"].as_bool(), Some(false));
        assert_eq!(row["rollback_watchdog_armed"].as_bool(), Some(true));
        assert_eq!(row["revert_verified"].as_bool(), Some(true));
    }

    Ok(())
}
