use std::collections::BTreeSet;

use anyhow::Result;
use serde_json::Value;

fn design_report() -> Result<Value> {
    Ok(serde_json::from_str(include_str!(
        "../data/reports/config-persistence-validation-design.v0.55.2.json"
    ))?)
}

fn batch_a_candidates_report() -> Result<Value> {
    Ok(serde_json::from_str(include_str!(
        "../data/reports/batch-a-config-persistence-candidates.v0.55.2.json"
    ))?)
}

fn batch_a_semantics_report() -> Result<Value> {
    Ok(serde_json::from_str(include_str!(
        "../data/reports/live-validation-batch-a-semantics-classification.v0.55.2.json"
    ))?)
}

fn scalar_coverage_report() -> Result<Value> {
    Ok(serde_json::from_str(include_str!(
        "../data/reports/scalar-read-write-coverage.v0.55.2.json"
    ))?)
}

#[test]
fn config_persistence_design_selects_temp_config_verify_policy() -> Result<()> {
    let report = design_report()?;

    assert_eq!(
        report["artifactKind"].as_str(),
        Some("config-persistence-validation-design")
    );
    assert_eq!(
        report["recommendedApproach"]["id"].as_str(),
        Some("hyprland-verify-config-temp-file")
    );
    assert_eq!(
        report["recommendedApproach"]["policy"].as_str(),
        Some("strict-config-persistence-validation")
    );
    assert_eq!(report["invariants"]["rowsEnabledThisSprint"], 0);
    assert_eq!(report["invariants"]["writableRowsRemain"], 55);
    assert_eq!(report["invariants"]["doesNotChangeWriteBehavior"], true);
    assert_eq!(report["invariants"]["doesNotRunReload"], true);
    assert_eq!(report["invariants"]["doesNotRunEval"], true);

    let unsafe_approaches = report["unsafeApproaches"]
        .as_array()
        .expect("unsafeApproaches should be an array");
    assert!(unsafe_approaches
        .iter()
        .any(|item| item.as_str().unwrap_or("").contains("hyprctl eval")));

    Ok(())
}

#[test]
fn batch_a_config_persistence_candidates_match_semantics_rows() -> Result<()> {
    let candidates = batch_a_candidates_report()?;
    let semantics = batch_a_semantics_report()?;

    assert_eq!(
        candidates["artifactKind"].as_str(),
        Some("batch-a-config-persistence-candidates")
    );
    assert_eq!(candidates["counts"]["rows"], 39);
    assert_eq!(candidates["counts"]["acceptedUnobservable"], 3);
    assert_eq!(candidates["counts"]["unproven"], 36);
    assert_eq!(candidates["counts"]["safeToImplementNextSprint"], 39);
    assert_eq!(candidates["counts"]["safeToEnableNow"], 0);
    assert_eq!(candidates["counts"]["highRiskRows"], 0);

    let candidate_ids = candidates["rows"]
        .as_array()
        .expect("candidate rows should be an array")
        .iter()
        .map(|row| row["rowId"].as_str().expect("candidate rowId should exist"))
        .collect::<BTreeSet<_>>();
    let semantics_ids = semantics["items"]
        .as_array()
        .expect("semantics items should be an array")
        .iter()
        .map(|row| row["rowId"].as_str().expect("semantics rowId should exist"))
        .collect::<BTreeSet<_>>();

    assert_eq!(candidate_ids, semantics_ids);

    for row in candidates["rows"]
        .as_array()
        .expect("candidate rows should be an array")
    {
        assert_eq!(row["safeToImplementNextSprint"].as_bool(), Some(true));
        assert_eq!(row["safeToEnableNow"].as_bool(), Some(false));
        assert!(row["requiredProof"]
            .as_array()
            .expect("requiredProof should be an array")
            .iter()
            .any(|proof| proof.as_str() == Some("hyprland-verify-config-temp-file-passed")));
    }

    Ok(())
}

#[test]
fn config_persistence_design_does_not_change_scalar_write_counts() -> Result<()> {
    let coverage = scalar_coverage_report()?;
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

    assert_eq!(coverage["counts"]["writableRows"], 55);
    assert_eq!(coverage["counts"]["blockedWriteRows"], 286);
    assert_eq!(writable, 55);
    assert_eq!(manual, 214);
    assert_eq!(high_risk, 72);

    Ok(())
}
