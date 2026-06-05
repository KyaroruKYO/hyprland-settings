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

fn batch_c_string_semantics() -> Result<Value> {
    Ok(serde_json::from_str(include_str!(
        "../data/reports/batch-c-string-semantic-investigation.v0.55.2.json"
    ))?)
}

fn source_backed_input_values() -> Result<Value> {
    Ok(serde_json::from_str(include_str!(
        "../data/reports/source-backed-input-value-evidence.v0.55.2.json"
    ))?)
}

fn source_backed_input_write_proof() -> Result<Value> {
    Ok(serde_json::from_str(include_str!(
        "../data/reports/source-backed-input-write-proof.v0.55.2.json"
    ))?)
}

fn batch_d_remaining_input_proof() -> Result<Value> {
    Ok(serde_json::from_str(include_str!(
        "../data/reports/batch-d-remaining-input-proof.v0.55.2.json"
    ))?)
}

fn semantic_validator_implementation() -> Result<Value> {
    Ok(serde_json::from_str(include_str!(
        "../data/reports/semantic-validator-implementation.v0.55.2.json"
    ))?)
}

#[test]
fn remaining_105_reports_cover_every_blocked_scalar_row() -> Result<()> {
    let coverage = coverage()?;
    let investigation = investigation()?;
    let plan = proof_plan()?;
    let results = proof_results()?;
    let classifications = classifications()?;

    assert_eq!(coverage["counts"]["writableRows"], 274);
    assert_eq!(coverage["counts"]["blockedWriteRows"], 67);
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
    assert_eq!(blocked_ids.len(), 67);

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
    assert_eq!(high_risk_ids.len(), 67);

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

#[test]
fn batch_c_string_rows_have_semantic_validators_and_are_writable() -> Result<()> {
    let semantics = batch_c_string_semantics()?;
    let implementation = semantic_validator_implementation()?;
    let classifications = classifications()?;
    let coverage = coverage()?;

    assert_eq!(semantics["counts"]["rows"], 2);
    assert_eq!(semantics["counts"]["rowsWithArbitraryStringAcceptance"], 2);
    assert_eq!(semantics["counts"]["rowsEnabled"], 2);
    assert_eq!(implementation["counts"]["rows"], 2);
    assert_eq!(implementation["counts"]["validatorsImplemented"], 2);
    assert_eq!(implementation["counts"]["rowsEnabled"], 2);
    assert_eq!(implementation["counts"]["finalWritableRows"], 253);
    assert_eq!(implementation["counts"]["finalBlockedRows"], 88);
    assert!(
        implementation["counts"]["invalidValuesAcceptedByVerifyConfig"]
            .as_u64()
            .expect("invalid parser-only count should be numeric")
            >= 2,
        "report should preserve evidence that verify-config is parser-only"
    );
    assert_eq!(
        semantics["safety"]["activeConfigModified"].as_bool(),
        Some(false)
    );
    assert_eq!(
        semantics["safety"]["activeRuntimeModified"].as_bool(),
        Some(false)
    );

    let target_ids = [
        "master.center_master_fallback",
        "scrolling.explicit_column_widths",
    ];
    let classification_rows = classifications["rows"].as_array().unwrap();
    let coverage_rows = coverage["rows"].as_array().unwrap();

    for row_id in target_ids {
        let semantic_row = semantics["rows"]
            .as_array()
            .unwrap()
            .iter()
            .find(|row| row["rowId"].as_str() == Some(row_id))
            .unwrap_or_else(|| panic!("missing semantic investigation for {row_id}"));
        assert_eq!(
            semantic_row["verifyConfigSemanticStrength"].as_str(),
            Some("parser-only-app-validator-authoritative")
        );
        assert_eq!(
            semantic_row["sufficientToEnableWrites"].as_bool(),
            Some(true)
        );
        assert!(
            !semantic_row["acceptedRawValues"]
                .as_array()
                .expect("acceptedRawValues should be an array")
                .is_empty(),
            "{row_id} should record accepted weak-parser probes"
        );

        let classification_row = classification_rows
            .iter()
            .find(|row| row["rowId"].as_str() == Some(row_id))
            .unwrap_or_else(|| panic!("missing classification for {row_id}"));
        let blockers = classification_row["missingProofCategories"]
            .as_array()
            .unwrap()
            .iter()
            .map(|value| value.as_str().unwrap())
            .collect::<BTreeSet<_>>();
        assert!(
            blockers.is_empty(),
            "{row_id} should not keep missing-proof blockers"
        );

        let coverage_row = coverage_rows
            .iter()
            .find(|row| row["rowId"].as_str() == Some(row_id))
            .unwrap_or_else(|| panic!("missing coverage row {row_id}"));
        assert_eq!(coverage_row["writeStatus"].as_str(), Some("writable"));
        assert_eq!(coverage_row["safeWriteSupported"].as_bool(), Some(true));
    }

    Ok(())
}

#[test]
fn source_backed_input_rows_have_xkb_proof_and_are_writable() -> Result<()> {
    let source_values = source_backed_input_values()?;
    let write_proof = source_backed_input_write_proof()?;
    let classifications = classifications()?;
    let coverage = coverage()?;

    assert_eq!(source_values["counts"]["rows"], 5);
    assert_eq!(source_values["counts"]["rowsEnabled"], 5);
    assert_eq!(write_proof["counts"]["rows"], 5);
    assert_eq!(write_proof["counts"]["rowsEnabled"], 5);
    assert!(
        source_values["counts"]["xkbModelValues"]
            .as_u64()
            .expect("xkbModelValues should be numeric")
            > 10
    );
    assert!(
        source_values["counts"]["xkbLayoutValues"]
            .as_u64()
            .expect("xkbLayoutValues should be numeric")
            > 10
    );
    assert!(
        source_values["counts"]["xkbOptionValues"]
            .as_u64()
            .expect("xkbOptionValues should be numeric")
            > 10
    );

    let target_ids = [
        "input.kb_model",
        "input.kb_layout",
        "input.kb_variant",
        "input.kb_options",
        "input.kb_rules",
    ];
    let source_rows = source_values["rows"].as_array().unwrap();
    let classification_rows = classifications["rows"].as_array().unwrap();
    let coverage_rows = coverage["rows"].as_array().unwrap();

    for row_id in target_ids {
        let source_row = source_rows
            .iter()
            .find(|row| row["rowId"].as_str() == Some(row_id))
            .unwrap_or_else(|| panic!("missing source-backed evidence for {row_id}"));
        assert_eq!(
            source_row["enumeratorImplementedInRust"].as_bool(),
            Some(true)
        );
        assert_eq!(source_row["safeToEnableNow"].as_bool(), Some(true));
        assert!(
            source_row["valueCount"]
                .as_u64()
                .expect("valueCount should be numeric")
                >= 2
        );

        let classification_row = classification_rows
            .iter()
            .find(|row| row["rowId"].as_str() == Some(row_id))
            .unwrap_or_else(|| panic!("missing classification for {row_id}"));
        let blockers = classification_row["missingProofCategories"]
            .as_array()
            .unwrap()
            .iter()
            .map(|value| value.as_str().unwrap())
            .collect::<BTreeSet<_>>();
        assert!(blockers.is_empty(), "{row_id} should not keep blockers");
        assert!(!blockers.contains("needs-source-backed-enumerator"));
        assert_eq!(
            classification_row["currentWriteStatus"].as_str(),
            Some("writable")
        );

        let coverage_row = coverage_rows
            .iter()
            .find(|row| row["rowId"].as_str() == Some(row_id))
            .unwrap_or_else(|| panic!("missing coverage row {row_id}"));
        assert_eq!(coverage_row["writeStatus"].as_str(), Some("writable"));
        assert_eq!(coverage_row["safeWriteSupported"].as_bool(), Some(true));

        let proof_row = write_proof["rows"]
            .as_array()
            .unwrap()
            .iter()
            .find(|row| row["rowId"].as_str() == Some(row_id))
            .unwrap_or_else(|| panic!("missing source-backed write proof for {row_id}"));
        assert_eq!(proof_row["safeToEnable"].as_bool(), Some(true));
        assert_eq!(
            proof_row["invalidValuesRejectedByApp"].as_bool(),
            Some(true)
        );
    }

    Ok(())
}

#[test]
fn batch_d_scroll_and_monitor_outputs_are_source_validated_writes() -> Result<()> {
    let proof = batch_d_remaining_input_proof()?;
    let classifications = classifications()?;
    let coverage = coverage()?;

    assert_eq!(proof["counts"]["rows"], 3);
    assert_eq!(proof["counts"]["rowsEnabled"], 3);
    assert_eq!(proof["counts"]["rowsStillBlocked"], 0);
    assert_eq!(
        proof["counts"]["activeConfigModified"].as_bool(),
        Some(false)
    );
    assert_eq!(
        proof["counts"]["activeRuntimeModified"].as_bool(),
        Some(false)
    );

    let proof_rows = proof["rows"].as_array().unwrap();
    let coverage_rows = coverage["rows"].as_array().unwrap();
    let classification_rows = classifications["rows"].as_array().unwrap();

    let scroll_proof = proof_rows
        .iter()
        .find(|row| row["rowId"].as_str() == Some("input.scroll_method"))
        .expect("scroll method proof should exist");
    assert_eq!(scroll_proof["status"].as_str(), Some("enabled"));
    assert_eq!(scroll_proof["valueKind"].as_str(), Some("finite-choice"));
    assert_eq!(scroll_proof["safeToEnable"].as_bool(), Some(true));
    assert_eq!(
        scroll_proof["appValidatorRejectsInvalidProbe"].as_bool(),
        Some(true)
    );
    assert_eq!(scroll_proof["candidateValues"].as_array().unwrap().len(), 4);

    let scroll_coverage = coverage_rows
        .iter()
        .find(|row| row["rowId"].as_str() == Some("input.scroll_method"))
        .expect("scroll method coverage should exist");
    assert_eq!(scroll_coverage["writeStatus"].as_str(), Some("writable"));
    assert_eq!(scroll_coverage["safeWriteSupported"].as_bool(), Some(true));

    for row_id in ["input.touchdevice.output", "input.tablet.output"] {
        let proof_row = proof_rows
            .iter()
            .find(|row| row["rowId"].as_str() == Some(row_id))
            .unwrap_or_else(|| panic!("missing proof row {row_id}"));
        assert_eq!(proof_row["status"].as_str(), Some("enabled"));
        assert_eq!(proof_row["safeToEnable"].as_bool(), Some(true));
        assert_eq!(
            proof_row["hyprlandVerifyConfigAlsoAcceptedInvalidProbe"].as_bool(),
            Some(true)
        );
        assert_eq!(
            proof_row["appValidatorRejectsInvalidProbe"].as_bool(),
            Some(true)
        );

        let classification_row = classification_rows
            .iter()
            .find(|row| row["rowId"].as_str() == Some(row_id))
            .unwrap_or_else(|| panic!("missing classification row {row_id}"));
        let blockers = classification_row["missingProofCategories"]
            .as_array()
            .unwrap()
            .iter()
            .map(|value| value.as_str().unwrap())
            .collect::<BTreeSet<_>>();
        assert!(blockers.is_empty(), "{row_id} should not keep blockers");

        let coverage_row = coverage_rows
            .iter()
            .find(|row| row["rowId"].as_str() == Some(row_id))
            .unwrap_or_else(|| panic!("missing coverage row {row_id}"));
        assert_eq!(coverage_row["writeStatus"].as_str(), Some("writable"));
        assert_eq!(coverage_row["safeWriteSupported"].as_bool(), Some(true));
    }

    Ok(())
}
