use anyhow::Result;
use hyprland_settings::write_classification::SAFE_WRITABLE_ROWS;
use serde_json::Value;
use std::collections::{BTreeMap, BTreeSet};

fn read_json(path: &str) -> Result<Value> {
    Ok(serde_json::from_str(&std::fs::read_to_string(path)?)?)
}

fn row_map<'a>(report: &'a Value) -> BTreeMap<&'a str, &'a Value> {
    report["rows"]
        .as_array()
        .unwrap()
        .iter()
        .map(|row| (row["rowId"].as_str().unwrap(), row))
        .collect()
}

#[test]
fn writable_value_type_audit_reports_cover_every_safe_writable_row() -> Result<()> {
    let audit = read_json("data/reports/writable-value-type-exhaustiveness-audit.v0.55.2.json")?;
    let matrix = read_json("data/reports/writable-value-type-evidence-matrix.v0.55.2.json")?;
    let gap = read_json("data/reports/writable-value-type-gap-summary.v0.55.2.json")?;
    let risk = read_json("data/reports/writable-validator-risk-audit.v0.55.2.json")?;
    let coverage = read_json("data/reports/scalar-read-write-coverage.v0.55.2.json")?;

    assert_eq!(SAFE_WRITABLE_ROWS.len(), 340);
    assert_eq!(coverage["counts"]["writableRows"], 340);
    assert_eq!(coverage["counts"]["blockedWriteRows"], 1);

    for report in [&audit, &matrix] {
        assert_eq!(report["counts"]["totalWritableRowsAudited"], 340);
        assert_eq!(report["counts"]["rowsEnabledThisSprint"], 62);
        assert_eq!(report["counts"]["writeAllowlistChanged"], true);
        assert_eq!(report["counts"]["productionBehaviorChanged"], true);
        assert_eq!(report["counts"]["validatorsChanged"], false);
        assert_eq!(report["counts"]["recoveryGatesChanged"], true);
        assert_eq!(report["counts"]["realConfigModified"], false);
        assert_eq!(report["counts"]["activeRuntimeModified"], false);
        assert_eq!(report["counts"]["reloadEvalLuaUsed"], false);
    }

    assert_eq!(gap["counts"]["totalWritableRowsAudited"], 340);
    assert_eq!(risk["counts"]["rowsEnabledThisSprint"], 62);
    assert_eq!(risk["counts"]["writeAllowlistChanged"], true);
    assert_eq!(risk["counts"]["productionBehaviorChanged"], true);

    let audit_rows = row_map(&audit);
    let matrix_rows = row_map(&matrix);
    let expected = SAFE_WRITABLE_ROWS
        .iter()
        .map(|row| row.row_id)
        .collect::<BTreeSet<_>>();
    let actual = audit_rows.keys().copied().collect::<BTreeSet<_>>();
    assert_eq!(actual, expected);
    assert_eq!(
        matrix_rows.keys().copied().collect::<BTreeSet<_>>(),
        expected
    );

    Ok(())
}

#[test]
fn every_writable_value_type_row_has_classification_and_evidence_status() -> Result<()> {
    let audit = read_json("data/reports/writable-value-type-exhaustiveness-audit.v0.55.2.json")?;
    let allowed = BTreeSet::from([
        "complete",
        "partial",
        "parser-only",
        "conflicting",
        "unknown",
        "app-validator-too-narrow",
        "app-validator-too-broad",
        "source-research-needed",
    ]);
    let required_evidence = [
        "officialSourceEvidence",
        "companionEvidence",
        "hyprmodEvidence",
        "appValidatorEvidence",
        "tempConfigEvidence",
        "invalidRejectionEvidence",
        "fixtureWriteRereadEvidence",
        "highRiskRecoveryEvidence",
    ];

    for row in audit["rows"].as_array().unwrap() {
        let row_id = row["rowId"].as_str().unwrap();
        assert!(!row_id.is_empty());
        assert!(
            !row["officialSetting"]
                .as_str()
                .unwrap_or_default()
                .is_empty(),
            "{row_id}"
        );
        assert!(
            !row["appValueType"].as_str().unwrap_or_default().is_empty(),
            "{row_id}"
        );
        assert!(
            allowed.contains(row["classification"].as_str().unwrap()),
            "{row_id}"
        );
        assert!(
            !row["recommendedNextAction"]
                .as_str()
                .unwrap_or_default()
                .is_empty(),
            "{row_id}"
        );
        for evidence_key in required_evidence {
            assert!(
                !row["evidenceStatus"][evidence_key]
                    .as_str()
                    .unwrap_or_default()
                    .is_empty(),
                "{row_id} missing {evidence_key}"
            );
        }
    }

    Ok(())
}

#[test]
fn classification_counts_and_gap_buckets_are_consistent() -> Result<()> {
    let audit = read_json("data/reports/writable-value-type-exhaustiveness-audit.v0.55.2.json")?;
    let gap = read_json("data/reports/writable-value-type-gap-summary.v0.55.2.json")?;
    let risk = read_json("data/reports/writable-validator-risk-audit.v0.55.2.json")?;

    let rows = audit["rows"].as_array().unwrap();
    let count = |classification: &str| {
        rows.iter()
            .filter(|row| row["classification"].as_str() == Some(classification))
            .count()
    };

    assert_eq!(
        count("complete"),
        audit["counts"]["completeRows"].as_u64().unwrap() as usize
    );
    assert_eq!(
        count("partial"),
        audit["counts"]["partialRows"].as_u64().unwrap() as usize
    );
    assert_eq!(
        count("parser-only"),
        audit["counts"]["parserOnlyRows"].as_u64().unwrap() as usize
    );
    assert_eq!(
        count("conflicting"),
        audit["counts"]["conflictingRows"].as_u64().unwrap() as usize
    );
    assert_eq!(
        count("unknown"),
        audit["counts"]["unknownRows"].as_u64().unwrap() as usize
    );
    assert_eq!(
        count("app-validator-too-narrow"),
        audit["counts"]["appValidatorTooNarrowRows"]
            .as_u64()
            .unwrap() as usize
    );
    assert_eq!(
        count("app-validator-too-broad"),
        audit["counts"]["appValidatorTooBroadRows"]
            .as_u64()
            .unwrap() as usize
    );
    assert_eq!(
        count("source-research-needed"),
        audit["counts"]["sourceResearchNeededRows"]
            .as_u64()
            .unwrap() as usize
    );

    for (classification, bucket_name) in [
        ("complete", "completeRows"),
        ("partial", "partialRows"),
        ("parser-only", "parserOnlyRows"),
        ("conflicting", "conflictingRows"),
        ("unknown", "unknownRows"),
        ("app-validator-too-narrow", "appValidatorTooNarrowRows"),
        ("app-validator-too-broad", "appValidatorTooBroadRows"),
        ("source-research-needed", "sourceResearchNeededRows"),
    ] {
        let expected = count(classification);
        assert_eq!(
            gap["buckets"][bucket_name].as_array().unwrap().len(),
            expected,
            "{bucket_name}"
        );
    }

    assert_eq!(
        risk["counts"]["appValidatorTooNarrowRows"],
        audit["counts"]["appValidatorTooNarrowRows"]
    );
    assert_eq!(
        risk["counts"]["appValidatorTooBroadRows"],
        audit["counts"]["appValidatorTooBroadRows"]
    );
    assert_eq!(
        risk["counts"]["conflictingRows"],
        audit["counts"]["conflictingRows"]
    );

    Ok(())
}

#[test]
fn complete_rows_have_required_critical_evidence() -> Result<()> {
    let audit = read_json("data/reports/writable-value-type-exhaustiveness-audit.v0.55.2.json")?;

    for row in audit["rows"]
        .as_array()
        .unwrap()
        .iter()
        .filter(|row| row["classification"].as_str() == Some("complete"))
    {
        let row_id = row["rowId"].as_str().unwrap();
        assert_eq!(row["acceptedValueSetKnownExhaustively"], true, "{row_id}");
        assert_eq!(
            row["evidenceStatus"]["tempConfigEvidence"].as_str(),
            Some("complete"),
            "{row_id}"
        );
        assert_eq!(
            row["evidenceStatus"]["invalidRejectionEvidence"].as_str(),
            Some("complete"),
            "{row_id}"
        );
        assert_eq!(
            row["evidenceStatus"]["fixtureWriteRereadEvidence"].as_str(),
            Some("complete"),
            "{row_id}"
        );
        assert_ne!(
            row["evidenceStatus"]["appValidatorEvidence"].as_str(),
            Some("broad"),
            "{row_id}"
        );
        assert_ne!(
            row["evidenceStatus"]["appValidatorEvidence"].as_str(),
            Some("narrow"),
            "{row_id}"
        );
    }

    Ok(())
}
