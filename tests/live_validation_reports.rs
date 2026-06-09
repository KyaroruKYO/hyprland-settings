use std::collections::BTreeSet;

use anyhow::Result;
use serde_json::Value;

fn plan_report() -> Result<Value> {
    Ok(serde_json::from_str(include_str!(
        "../data/reports/live-validation-plan.v0.55.2.json"
    ))?)
}

fn batch_a_config_persistence_candidates_report() -> Result<Value> {
    Ok(serde_json::from_str(include_str!(
        "../data/reports/batch-a-config-persistence-candidates.v0.55.2.json"
    ))?)
}

fn results_report() -> Result<Value> {
    Ok(serde_json::from_str(include_str!(
        "../data/reports/live-validation-results.v0.55.2.json"
    ))?)
}

fn future_batch_report() -> Result<Value> {
    Ok(serde_json::from_str(include_str!(
        "../data/reports/future-live-validation-batches.v0.55.2.json"
    ))?)
}

fn level3_diagnostics_report() -> Result<Value> {
    Ok(serde_json::from_str(include_str!(
        "../data/reports/live-validation-level3-diagnostics.v0.55.2.json"
    ))?)
}

fn semantics_report() -> Result<Value> {
    Ok(serde_json::from_str(include_str!(
        "../data/reports/live-validation-semantics.v0.55.2.json"
    ))?)
}

fn batch_a_semantics_classification_report() -> Result<Value> {
    Ok(serde_json::from_str(include_str!(
        "../data/reports/live-validation-batch-a-semantics-classification.v0.55.2.json"
    ))?)
}

#[test]
fn live_validation_plan_contains_only_batch_a_rows() -> Result<()> {
    let plan = plan_report()?;
    let batch_a_candidates = batch_a_config_persistence_candidates_report()?;
    let rows = plan["rows"]
        .as_array()
        .expect("live validation plan rows should be an array");
    let batch_a = batch_a_candidates["rows"]
        .as_array()
        .expect("Batch A candidate rows should be an array")
        .iter()
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

#[test]
fn future_live_validation_batches_remain_plan_only() -> Result<()> {
    let report = future_batch_report()?;

    assert_eq!(report["currentBatchAResult"]["plannedRows"], 39);
    assert_eq!(report["currentBatchAResult"]["liveProbedRows"], 39);
    assert_eq!(report["currentBatchAResult"]["level3Passed"], 0);
    assert_eq!(report["currentBatchAResult"]["level4Passed"], 39);
    assert_eq!(report["currentBatchAResult"]["enabledRows"], 39);
    assert_eq!(
        report["currentBatchAResult"]["decision"].as_str(),
        Some("enabled-by-config-persistence-proof")
    );

    let batches = report["futureBatches"]
        .as_array()
        .expect("futureBatches should be an array");
    assert_eq!(batches.len(), 6);
    assert!(batches.iter().any(|batch| batch["batch"] == "high-risk"));
    assert!(batches
        .iter()
        .any(|batch| batch["batch"] == "batch-b-likely-safe-numerics"));
    for batch in batches {
        assert_eq!(batch["liveValidationAllowedNow"].as_bool(), Some(false));
    }

    assert_eq!(
        report["invariants"]["doesNotEnableRows"].as_bool(),
        Some(true)
    );
    assert_eq!(
        report["invariants"]["doesNotChangeWriteBehavior"].as_bool(),
        Some(true)
    );
    assert_eq!(
        report["invariants"]["requiresRollbackWatchdogForFutureLiveProbe"].as_bool(),
        Some(true)
    );

    Ok(())
}

#[test]
fn level3_diagnostics_record_three_row_subset_without_enablement() -> Result<()> {
    let report = level3_diagnostics_report()?;
    let items = report["items"]
        .as_array()
        .expect("diagnostic items should be an array");

    assert_eq!(report["counts"]["rows"], 3);
    assert_eq!(report["counts"]["accepted"], 0);
    assert_eq!(report["counts"]["revertVerified"], 3);
    assert_eq!(report["counts"]["safeToRetest"], 3);

    let row_ids = items
        .iter()
        .map(|item| item["rowId"].as_str().expect("rowId should be present"))
        .collect::<BTreeSet<_>>();
    assert_eq!(
        row_ids,
        [
            "appearance.dim.modal",
            "misc.disable_hyprland_logo",
            "misc.disable_splash_rendering"
        ]
        .into_iter()
        .collect()
    );
    for item in items {
        assert_eq!(item["keywordExitSuccess"].as_bool(), Some(true));
        assert_eq!(item["valuesEquivalent"].as_bool(), Some(false));
        assert_eq!(item["revertVerified"].as_bool(), Some(true));
        assert_eq!(
            item["diagnosis"].as_str(),
            Some("keyword-succeeded-but-getoption-stayed-original")
        );
    }

    Ok(())
}

#[test]
fn live_validation_semantics_report_keeps_unobservable_rows_blocked() -> Result<()> {
    let report = semantics_report()?;
    let items = report["items"]
        .as_array()
        .expect("semantics items should be an array");

    assert_eq!(report["counts"]["rows"], 5);
    assert_eq!(report["counts"]["level3LiveObservableAccepted"], 0);
    assert_eq!(report["counts"]["level3AcceptedUnobservable"], 5);
    assert_eq!(report["counts"]["safeToEnable"], 0);
    assert_eq!(
        report["level3Policy"]["chosenPolicy"].as_str(),
        Some("strict-live-observable-for-automatic-enablement")
    );

    let expected = [
        "animations.enabled",
        "appearance.dim.modal",
        "misc.disable_hyprland_logo",
        "misc.disable_splash_rendering",
        "windows.snap.enabled",
    ]
    .into_iter()
    .collect::<BTreeSet<_>>();
    let actual = items
        .iter()
        .map(|item| item["rowId"].as_str().expect("rowId should be present"))
        .collect::<BTreeSet<_>>();
    assert_eq!(actual, expected);

    for item in items {
        assert_eq!(
            item["level3Status"].as_str(),
            Some("level3-accepted-unobservable")
        );
        assert_eq!(item["hyprctlKeywordSuccess"].as_bool(), Some(true));
        assert_eq!(item["getoptionChanged"].as_bool(), Some(false));
        assert_eq!(item["revertVerified"].as_bool(), Some(true));
        assert_eq!(item["safeToEnable"].as_bool(), Some(false));
    }

    Ok(())
}

#[test]
fn batch_a_semantics_classification_enables_no_rows_without_strict_level3() -> Result<()> {
    let report = batch_a_semantics_classification_report()?;
    let items = report["items"]
        .as_array()
        .expect("Batch A classification items should be an array");

    assert_eq!(report["counts"]["rows"], 39);
    assert_eq!(report["counts"]["level3LiveObservableAccepted"], 0);
    assert_eq!(report["counts"]["level3AcceptedUnobservable"], 3);
    assert_eq!(report["counts"]["level3Unproven"], 36);
    assert_eq!(report["counts"]["safeToEnable"], 0);
    assert_eq!(report["counts"]["stillBlocked"], 39);
    assert_eq!(report["decision"].as_str(), Some("enable-none"));
    for item in items {
        assert_eq!(item["safeToEnable"].as_bool(), Some(false));
    }

    Ok(())
}
