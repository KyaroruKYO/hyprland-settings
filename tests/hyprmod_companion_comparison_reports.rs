use anyhow::Result;
use hyprland_settings::write_classification::SAFE_WRITABLE_ROWS;
use serde_json::Value;

fn merged_evidence_report() -> Result<Value> {
    Ok(serde_json::from_str(include_str!(
        "../data/reports/hyprmod-merged-schema-evidence.v0.55.2.json"
    ))?)
}

fn companion_comparison_report() -> Result<Value> {
    Ok(serde_json::from_str(include_str!(
        "../data/reports/hyprmod-companion-full-scalar-comparison.v0.55.2.json"
    ))?)
}

fn unmatched_companion_report() -> Result<Value> {
    Ok(serde_json::from_str(include_str!(
        "../data/reports/hyprmod-unmatched-companion-scalar-candidates.v0.55.2.json"
    ))?)
}

fn scalar_coverage_report() -> Result<Value> {
    Ok(serde_json::from_str(include_str!(
        "../data/reports/scalar-read-write-coverage.v0.55.2.json"
    ))?)
}

#[test]
fn hyprmod_companion_reports_cover_all_scalar_rows() -> Result<()> {
    let merged = merged_evidence_report()?;
    let comparison = companion_comparison_report()?;
    let rows = comparison["rows"]
        .as_array()
        .expect("comparison rows should be an array");

    assert_eq!(merged["counts"]["hyprlandSchemaRowsFound"], 341);
    assert_eq!(merged["counts"]["mergedEvidenceRowsFound"], 341);
    assert_eq!(comparison["counts"]["ourScalarRowsCompared"], 341);
    assert_eq!(comparison["counts"]["hyprmodMergedEvidenceRowsFound"], 341);
    assert_eq!(comparison["counts"]["normalizedKeyMatches"], 341);
    assert_eq!(comparison["counts"]["noMatches"], 0);
    assert_eq!(rows.len(), 341);

    for row in rows {
        assert!(row["evidenceStrength"].as_str().is_some());
        assert!(row["comparisonResult"].as_str().is_some());
        assert!(row["recommendedAction"].as_str().is_some());
        assert!(
            row["evidenceSourcePath"].as_str().is_some(),
            "{} lacks a companion evidence source path",
            row["rowId"]
        );
    }

    Ok(())
}

#[test]
fn hyprmod_companion_comparison_is_advisory_only() -> Result<()> {
    let comparison = companion_comparison_report()?;
    let coverage = scalar_coverage_report()?;

    assert_eq!(comparison["counts"]["rowsEnabledThisSprint"], 0);
    assert_eq!(comparison["counts"]["writeBehaviorChanged"], false);
    assert_eq!(comparison["counts"]["currentWritableRowsRemain"], 236);
    assert_eq!(coverage["counts"]["writableRows"], 340);
    assert_eq!(SAFE_WRITABLE_ROWS.len(), 340);

    let rows = comparison["rows"]
        .as_array()
        .expect("comparison rows should be an array");
    assert_eq!(
        rows.iter()
            .filter(|row| row["currentWriteStatus"].as_str() == Some("high-risk"))
            .count(),
        72
    );
    assert_eq!(
        rows.iter()
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
fn hyprmod_companion_unmatched_report_exists() -> Result<()> {
    let unmatched = unmatched_companion_report()?;
    assert_eq!(
        unmatched["artifactKind"].as_str(),
        Some("hyprmod-unmatched-companion-scalar-candidates")
    );
    assert!(unmatched["rows"].as_array().is_some());
    assert_eq!(
        unmatched["counts"]["rows"],
        unmatched["rows"].as_array().unwrap().len()
    );
    Ok(())
}
