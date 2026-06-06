use std::collections::BTreeSet;

use anyhow::Result;
use serde_json::Value;

fn coverage() -> Result<Value> {
    Ok(serde_json::from_str(include_str!(
        "../data/reports/scalar-read-write-coverage.v0.55.2.json"
    ))?)
}

fn pipeline() -> Result<Value> {
    Ok(serde_json::from_str(include_str!(
        "../data/reports/unified-remaining-90-pipeline-design.v0.55.2.json"
    ))?)
}

fn official_evidence() -> Result<Value> {
    Ok(serde_json::from_str(include_str!(
        "../data/reports/remaining-90-official-source-evidence.v0.55.2.json"
    ))?)
}

fn scope_policy() -> Result<Value> {
    Ok(serde_json::from_str(include_str!(
        "../data/reports/remaining-90-scope-risk-apply-policy.v0.55.2.json"
    ))?)
}

#[test]
fn unified_remaining_90_pipeline_covers_current_blocked_rows() -> Result<()> {
    let coverage = coverage()?;
    let pipeline = pipeline()?;
    let official = official_evidence()?;
    let scope_policy = scope_policy()?;

    assert_eq!(coverage["counts"]["writableRows"], 275);
    assert_eq!(coverage["counts"]["blockedWriteRows"], 66);
    assert_eq!(pipeline["counts"]["rows"], 90);
    assert_eq!(official["counts"]["rows"], 90);
    assert_eq!(scope_policy["counts"]["rows"], 90);

    let blocked_ids = coverage["rows"]
        .as_array()
        .expect("coverage rows should be an array")
        .iter()
        .filter(|row| row["writeStatus"].as_str() != Some("writable"))
        .map(|row| row["rowId"].as_str().unwrap().to_owned())
        .collect::<BTreeSet<_>>();
    assert_eq!(blocked_ids.len(), 66);

    for report in [&pipeline, &official, &scope_policy] {
        let ids = report["rows"]
            .as_array()
            .expect("report rows should be an array")
            .iter()
            .map(|row| row["rowId"].as_str().unwrap().to_owned())
            .collect::<BTreeSet<_>>();
        assert!(
            ids.is_superset(&blocked_ids),
            "unified remaining-90 reports should retain the original sprint set and include all currently blocked rows"
        );
    }

    Ok(())
}

#[test]
fn every_remaining_row_has_pipeline_fields_and_no_unknown_classification() -> Result<()> {
    let pipeline = pipeline()?;

    let disallowed_unknown_fields = [
        "valueType",
        "validatorAuthority",
        "scope",
        "applyPath",
        "rereadOracle",
        "recoveryStrategy",
        "approvalGate",
        "nextRequiredWork",
    ];

    for row in pipeline["rows"].as_array().unwrap() {
        for field in disallowed_unknown_fields {
            let value = row[field]
                .as_str()
                .unwrap_or_else(|| panic!("{field} should be a string"));
            assert!(!value.is_empty(), "{} has empty {field}", row["rowId"]);
            assert_ne!(value, "unknown", "{} has unknown {field}", row["rowId"]);
        }
        assert_ne!(
            row["rereadOracle"].as_str(),
            Some("none-yet"),
            "{} has no reread oracle",
            row["rowId"]
        );
    }

    Ok(())
}

#[test]
fn custom_semantic_rows_are_parser_only_and_have_specific_validator_paths() -> Result<()> {
    let pipeline = pipeline()?;
    let official = official_evidence()?;

    assert_eq!(pipeline["counts"]["customStringSemanticRows"], 0);
    assert_eq!(pipeline["counts"]["resolvedCustomStringSemanticRows"], 2);
    assert_eq!(pipeline["counts"]["verifyConfigParserOnlyRows"], 2);

    let rows = pipeline["rows"].as_array().unwrap();
    let master = rows
        .iter()
        .find(|row| row["rowId"].as_str() == Some("master.center_master_fallback"))
        .expect("master.center_master_fallback should be classified");
    assert_eq!(master["currentWriteStatus"].as_str(), Some("writable"));
    assert_eq!(
        master["validatorRef"].as_str(),
        Some("enum:left|right|top|bottom")
    );
    assert_eq!(
        master["validatorAuthority"].as_str(),
        Some("official-source-authoritative")
    );
    assert_eq!(master["verifyConfigRole"].as_str(), Some("parser-only"));

    let widths = rows
        .iter()
        .find(|row| row["rowId"].as_str() == Some("scrolling.explicit_column_widths"))
        .expect("scrolling.explicit_column_widths should be classified");
    assert_eq!(widths["currentWriteStatus"].as_str(), Some("writable"));
    assert_eq!(
        widths["validatorAuthority"].as_str(),
        Some("app-validator-authoritative")
    );
    assert_eq!(widths["verifyConfigRole"].as_str(), Some("parser-only"));
    assert_eq!(widths["type"].as_str(), Some("comma-separated-float-list"));

    for row_id in [
        "master.center_master_fallback",
        "scrolling.explicit_column_widths",
    ] {
        let evidence_row = official["rows"]
            .as_array()
            .unwrap()
            .iter()
            .find(|row| row["rowId"].as_str() == Some(row_id))
            .unwrap_or_else(|| panic!("missing official evidence for {row_id}"));
        assert_eq!(
            evidence_row["officialSourceStatus"].as_str(),
            Some("complete")
        );
        assert_eq!(
            evidence_row["officialDocsStatus"].as_str(),
            Some("complete")
        );
        assert_eq!(
            evidence_row["verifyConfigRole"].as_str(),
            Some("parser-only")
        );
    }

    Ok(())
}

#[test]
fn session_rows_are_resolved_and_high_risk_rows_remain_blocked_with_recovery_gates() -> Result<()> {
    let coverage = coverage()?;
    let pipeline = pipeline()?;

    assert_eq!(pipeline["counts"]["sessionRuntimeSensitiveRows"], 0);
    assert_eq!(pipeline["counts"]["highRiskRows"], 69);

    let coverage_by_id = coverage["rows"]
        .as_array()
        .unwrap()
        .iter()
        .map(|row| (row["rowId"].as_str().unwrap(), row))
        .collect::<std::collections::BTreeMap<_, _>>();

    for row in pipeline["rows"].as_array().unwrap() {
        let row_id = row["rowId"].as_str().unwrap();
        let coverage_row = coverage_by_id.get(row_id).unwrap();
        if row["currentWriteStatus"].as_str() == Some("writable") {
            assert_eq!(coverage_row["writeStatus"].as_str(), Some("writable"));
            assert_eq!(coverage_row["safeWriteSupported"].as_bool(), Some(true));
            continue;
        }

        assert_ne!(coverage_row["writeStatus"].as_str(), Some("writable"));
        assert_eq!(coverage_row["safeWriteSupported"].as_bool(), Some(false));

        if row["currentWriteStatus"].as_str() == Some("high-risk") {
            assert!(
                row["approvalGate"].as_str().unwrap().contains("dead-man"),
                "{row_id} should require dead-man recovery"
            );
            assert!(
                row["applyPath"].as_str().unwrap().contains("blocked"),
                "{row_id} should stay blocked"
            );
        }
    }

    Ok(())
}
