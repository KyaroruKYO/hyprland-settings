use std::collections::BTreeMap;

use anyhow::Result;
use hyprland_settings::write_classification::SAFE_WRITABLE_ROWS;
use serde_json::Value;

fn integration_report() -> Result<Value> {
    Ok(serde_json::from_str(include_str!(
        "../data/reports/companion-schema-metadata-integration.v0.55.2.json"
    ))?)
}

fn conflict_report() -> Result<Value> {
    Ok(serde_json::from_str(include_str!(
        "../data/reports/companion-schema-conflict-review.v0.55.2.json"
    ))?)
}

fn coverage_report() -> Result<Value> {
    Ok(serde_json::from_str(include_str!(
        "../data/reports/scalar-read-write-coverage.v0.55.2.json"
    ))?)
}

#[test]
fn companion_schema_conflicts_are_isolated_for_manual_review() -> Result<()> {
    let conflicts = conflict_report()?;
    let rows = conflicts["rows"]
        .as_array()
        .expect("conflict rows should be an array");
    let coverage = coverage_report()?;
    let coverage_by_row: BTreeMap<_, _> = coverage["rows"]
        .as_array()
        .expect("coverage rows should be an array")
        .iter()
        .map(|row| (row["rowId"].as_str().unwrap(), row))
        .collect();

    assert_eq!(conflicts["counts"]["conflictRows"], 12);
    assert_eq!(rows.len(), 12);
    assert_eq!(conflicts["counts"]["rowsChanged"], 12);
    assert_eq!(conflicts["counts"]["writeBehaviorChanged"], true);

    for row in rows {
        let row_id = row["rowId"].as_str().unwrap();
        let coverage_row = coverage_by_row
            .get(row_id)
            .unwrap_or_else(|| panic!("{row_id} missing from scalar coverage"));
        assert_eq!(row["currentWriteStatus"], coverage_row["writeStatus"]);
        assert_eq!(row["currentControlKind"], coverage_row["controlKind"]);
        assert_eq!(row["currentControlKind"].as_str(), Some("dropdown"));
        assert_eq!(row["currentValueFamily"].as_str(), Some("finite-choice"));
        assert_eq!(
            row["recommendedProvisionalAction"].as_str(),
            Some("approved-companion-model-applied")
        );
        assert_eq!(
            row["userDecision"].as_str(),
            Some("approved-companion-model")
        );
        assert_eq!(
            row["chosenStorageMode"].as_str(),
            Some("numeric-raw-values-with-semantic-labels")
        );
    }

    Ok(())
}

#[test]
fn companion_schema_metadata_integration_is_advisory_only() -> Result<()> {
    let integration = integration_report()?;
    let coverage = coverage_report()?;

    assert_eq!(integration["counts"]["schemaBetterRowsIntegrated"], 86);
    assert_eq!(integration["counts"]["complementaryRowsIntegrated"], 130);
    assert_eq!(integration["counts"]["conflictRowsExcluded"], 12);
    assert_eq!(integration["counts"]["rowsEnabled"], 0);
    assert_eq!(integration["counts"]["startingWritableRows"], 236);
    assert_eq!(integration["counts"]["finalWritableRows"], 236);
    assert_eq!(coverage["counts"]["writableRows"], 269);
    assert_eq!(SAFE_WRITABLE_ROWS.len(), 269);
    assert_eq!(integration["invariants"]["writeBehaviorChanged"], false);
    assert_eq!(integration["invariants"]["writeAllowlistChanged"], false);
    assert_eq!(integration["invariants"]["rowsEnabledByMetadataAlone"], 0);

    Ok(())
}

#[test]
fn integrated_metadata_has_future_proof_test_values() -> Result<()> {
    let integration = integration_report()?;
    let mut all_rows = Vec::new();
    all_rows.extend(
        integration["schemaBetterRows"]
            .as_array()
            .expect("schema better rows should be an array"),
    );
    all_rows.extend(
        integration["complementaryRows"]
            .as_array()
            .expect("complementary rows should be an array"),
    );

    assert_eq!(all_rows.len(), 216);
    for row in all_rows {
        let fields = row["adoptedMetadataFields"]
            .as_array()
            .expect("adopted fields should be an array");
        assert!(
            fields
                .iter()
                .any(|field| field.as_str() == Some("test-values")),
            "{} should expose companion test values",
            row["rowId"]
        );
        if fields.iter().any(|field| {
            matches!(
                field.as_str(),
                Some("bounds" | "enum-choice-values" | "default")
            )
        }) {
            assert!(
                !row["suggestedValidTestValues"]
                    .as_array()
                    .unwrap()
                    .is_empty(),
                "{} should have valid test values",
                row["rowId"]
            );
            assert!(
                !row["suggestedInvalidTestValues"]
                    .as_array()
                    .unwrap()
                    .is_empty(),
                "{} should have invalid test values",
                row["rowId"]
            );
        }
    }

    Ok(())
}

#[test]
fn high_risk_rows_remain_blocked_after_metadata_integration() -> Result<()> {
    let coverage = coverage_report()?;
    let high_risk_rows = coverage["rows"]
        .as_array()
        .expect("coverage rows should be an array")
        .iter()
        .filter(|row| row["writeStatus"].as_str() == Some("high-risk"))
        .count();
    assert_eq!(high_risk_rows, 72);

    let integration = integration_report()?;
    assert_eq!(integration["counts"]["highRiskRowsStillBlocked"], 72);
    Ok(())
}
