use anyhow::Result;
use hyprland_settings::write_classification::SAFE_WRITABLE_ROWS;
use serde_json::Value;

fn comparison_report() -> Result<Value> {
    Ok(serde_json::from_str(include_str!(
        "../data/reports/hyprmod-full-scalar-model-comparison.v0.55.2.json"
    ))?)
}

fn unmatched_report() -> Result<Value> {
    Ok(serde_json::from_str(include_str!(
        "../data/reports/hyprmod-unmatched-scalar-candidates.v0.55.2.json"
    ))?)
}

fn coverage_report() -> Result<Value> {
    Ok(serde_json::from_str(include_str!(
        "../data/reports/scalar-read-write-coverage.v0.55.2.json"
    ))?)
}

#[test]
fn hyprmod_full_comparison_covers_all_scalar_rows() -> Result<()> {
    let report = comparison_report()?;
    let rows = report["rows"]
        .as_array()
        .expect("comparison rows should be an array");

    assert_eq!(report["counts"]["ourScalarRowsCompared"], 341);
    assert_eq!(rows.len(), 341);
    assert_eq!(report["counts"]["currentWritableRowsRemain"], 236);
    assert_eq!(report["counts"]["rowsEnabledThisSprint"], 0);
    assert_eq!(report["counts"]["writeBehaviorChanged"], false);
    assert_eq!(SAFE_WRITABLE_ROWS.len(), 272);

    for row in rows {
        assert!(row["evidenceStrength"].as_str().is_some());
        assert!(row["comparisonResult"].as_str().is_some());
        assert!(row["recommendedAction"].as_str().is_some());
        if row["evidenceStrength"].as_str() != Some("no-match") {
            assert!(
                row["evidenceSourcePath"].as_str().is_some(),
                "{} matched HyprMod but lacks a source path",
                row["rowId"]
            );
        }
    }

    Ok(())
}

#[test]
fn hyprmod_comparison_does_not_enable_rows_or_high_risk_writes() -> Result<()> {
    let comparison = comparison_report()?;
    let coverage = coverage_report()?;
    let coverage_rows = coverage["rows"]
        .as_array()
        .expect("coverage rows should be an array");
    let comparison_rows = comparison["rows"]
        .as_array()
        .expect("comparison rows should be an array");

    assert_eq!(coverage["counts"]["writableRows"], 272);
    assert_eq!(
        coverage_rows
            .iter()
            .filter(|row| row["writeStatus"].as_str() == Some("high-risk"))
            .count(),
        69
    );
    assert_eq!(
        comparison_rows
            .iter()
            .filter(|row| row["currentWriteStatus"].as_str() == Some("high-risk"))
            .count(),
        72
    );
    assert_eq!(
        comparison_rows
            .iter()
            .filter(|row| {
                row["currentWriteStatus"].as_str() == Some("high-risk")
                    && row["recommendedAction"].as_str() != Some("keep-blocked")
            })
            .count(),
        0
    );

    Ok(())
}

#[test]
fn hyprmod_unmatched_report_exists_and_is_advisory() -> Result<()> {
    let report = unmatched_report()?;
    let rows = report["rows"]
        .as_array()
        .expect("unmatched rows should be an array");

    assert_eq!(
        report["artifactKind"].as_str(),
        Some("hyprmod-unmatched-scalar-candidates")
    );
    assert_eq!(report["counts"]["rows"], rows.len());
    for row in rows {
        assert!(row["hyprmodKey"].as_str().is_some());
        assert!(row["reasonNoMatchFound"].as_str().is_some());
        assert!(
            row["mayBeMissingFromOurScalarInventory"]
                .as_bool()
                .is_some()
                || row["mayCorrespondToStructuredNonScalarSetting"]
                    .as_bool()
                    .is_some()
        );
    }

    Ok(())
}
