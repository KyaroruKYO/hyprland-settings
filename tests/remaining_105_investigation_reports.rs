use std::collections::{BTreeMap, BTreeSet};

use anyhow::Result;
use serde_json::Value;

fn coverage() -> Result<Value> {
    Ok(serde_json::from_str(include_str!(
        "../data/reports/scalar-read-write-coverage.v0.55.2.json"
    ))?)
}

fn investigation() -> Result<Value> {
    Ok(serde_json::from_str(include_str!(
        "../data/reports/remaining-105-scalar-investigation.v0.55.2.json"
    ))?)
}

fn proof_plan() -> Result<Value> {
    Ok(serde_json::from_str(include_str!(
        "../data/reports/remaining-105-proof-plan.v0.55.2.json"
    ))?)
}

fn proof_results() -> Result<Value> {
    Ok(serde_json::from_str(include_str!(
        "../data/reports/remaining-105-proof-results.v0.55.2.json"
    ))?)
}

fn classifications() -> Result<Value> {
    Ok(serde_json::from_str(include_str!(
        "../data/reports/remaining-105-missing-proof-classification.v0.55.2.json"
    ))?)
}

#[test]
fn remaining_105_reports_cover_every_blocked_scalar_row() -> Result<()> {
    let coverage = coverage()?;
    let investigation = investigation()?;
    let plan = proof_plan()?;
    let results = proof_results()?;
    let classifications = classifications()?;

    assert_eq!(coverage["counts"]["writableRows"], 243);
    assert_eq!(coverage["counts"]["blockedWriteRows"], 98);
    assert_eq!(investigation["counts"]["rows"], 105);
    assert_eq!(plan["counts"]["rows"], 105);
    assert_eq!(results["counts"]["rows"], 105);
    assert_eq!(classifications["counts"]["rows"], 105);

    let blocked_ids = coverage["rows"]
        .as_array()
        .expect("coverage rows should be an array")
        .iter()
        .filter(|row| row["writeStatus"].as_str() != Some("writable"))
        .map(|row| row["rowId"].as_str().unwrap().to_string())
        .collect::<BTreeSet<_>>();
    assert_eq!(blocked_ids.len(), 98);

    for report in [&investigation, &plan, &results, &classifications] {
        let ids = report["rows"]
            .as_array()
            .expect("report rows should be an array")
            .iter()
            .map(|row| row["rowId"].as_str().unwrap().to_string())
            .collect::<BTreeSet<_>>();
        if report["artifactKind"].as_str() == Some("remaining-105-missing-proof-classification") {
            assert!(ids.is_superset(&blocked_ids));
        } else {
            assert!(ids.len() >= blocked_ids.len());
        }
    }

    Ok(())
}

#[test]
fn remaining_105_classifications_have_explicit_missing_proof() -> Result<()> {
    let classifications = classifications()?;

    assert_eq!(classifications["counts"]["unknownClassifications"], 0);
    assert_eq!(
        classifications["counts"]["rowsWithAtLeastOneHyprlandAcceptedCandidate"],
        105
    );

    let allowed = classifications["allowedBlockerCategories"]
        .as_array()
        .expect("allowed categories should be present")
        .iter()
        .map(|value| value.as_str().unwrap().to_string())
        .collect::<BTreeSet<_>>();

    for row in classifications["rows"].as_array().unwrap() {
        let categories = row["missingProofCategories"]
            .as_array()
            .expect("missingProofCategories should be an array");
        if row["currentWriteStatus"].as_str() == Some("writable") {
            assert!(
                categories.is_empty(),
                "{} should not keep blockers after enablement",
                row["rowId"]
            );
            continue;
        }
        assert!(!categories.is_empty(), "{} has no blockers", row["rowId"]);
        assert_ne!(
            row["exactNextStepToMakeWritable"].as_str(),
            Some(""),
            "{} lacks a next step",
            row["rowId"]
        );
        for category in categories {
            let category = category.as_str().unwrap();
            assert!(allowed.contains(category), "unexpected blocker {category}");
            assert_ne!(category, "unknown");
        }
    }

    Ok(())
}

#[test]
fn remaining_105_high_risk_rows_remain_blocked() -> Result<()> {
    let coverage = coverage()?;
    let classifications = classifications()?;

    let high_risk_ids = classifications["rows"]
        .as_array()
        .unwrap()
        .iter()
        .filter(|row| row["currentWriteStatus"].as_str() == Some("high-risk"))
        .map(|row| row["rowId"].as_str().unwrap().to_string())
        .collect::<BTreeSet<_>>();
    assert_eq!(high_risk_ids.len(), 72);

    let coverage_by_id = coverage["rows"]
        .as_array()
        .unwrap()
        .iter()
        .map(|row| (row["rowId"].as_str().unwrap(), row))
        .collect::<BTreeMap<_, _>>();

    for row_id in high_risk_ids {
        let coverage_row = coverage_by_id.get(row_id.as_str()).unwrap();
        assert_eq!(coverage_row["writeStatus"].as_str(), Some("high-risk"));
        assert_eq!(coverage_row["safeWriteSupported"].as_bool(), Some(false));
    }

    Ok(())
}

#[test]
fn remaining_105_proof_results_never_reference_active_config_or_runtime_mutation() -> Result<()> {
    let results = proof_results()?;
    let serialized = serde_json::to_string(&results)?;

    assert!(!serialized.contains("/home/kyo/.config/hypr/hyprland.conf"));
    assert_eq!(
        results["counts"]["activeConfigModified"].as_bool(),
        Some(false)
    );
    assert_eq!(
        results["counts"]["activeRuntimeModified"].as_bool(),
        Some(false)
    );
    assert_eq!(results["safety"]["reloadUsed"].as_bool(), Some(false));
    assert_eq!(results["safety"]["evalUsed"].as_bool(), Some(false));
    assert_eq!(results["safety"]["luaUsed"].as_bool(), Some(false));

    for row in results["rows"].as_array().unwrap() {
        assert_eq!(row["activeConfigModified"].as_bool(), Some(false));
        assert_eq!(row["activeRuntimeModified"].as_bool(), Some(false));
        assert_eq!(row["hyprlandVerifyConfigAttempted"].as_bool(), Some(true));
    }

    Ok(())
}
