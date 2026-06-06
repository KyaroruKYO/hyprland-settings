use std::collections::BTreeSet;
use std::fs;

use anyhow::Result;
use hyprland_settings::write_classification::SAFE_WRITABLE_ROWS;
use serde_json::Value;

fn read_json(path: &str) -> Result<Value> {
    Ok(serde_json::from_str(&fs::read_to_string(path)?)?)
}

#[test]
fn hyprmod_cursor_input_comparison_covers_remaining_blocked_rows() -> Result<()> {
    let coverage = read_json("data/reports/scalar-read-write-coverage.v0.55.2.json")?;
    let comparison = read_json("data/reports/hyprmod-cursor-input-risk-comparison.v0.55.2.json")?;

    assert_eq!(coverage["counts"]["writableRows"], 277);
    assert_eq!(coverage["counts"]["blockedWriteRows"], 64);
    assert_eq!(SAFE_WRITABLE_ROWS.len(), 277);

    assert_eq!(comparison["counts"]["cursorInputRowsCompared"], 21);
    assert_eq!(comparison["counts"]["hyprmodOverlayMatches"], 10);
    assert_eq!(comparison["counts"]["hyprmodSchemaOnlyMatches"], 11);
    assert_eq!(comparison["counts"]["hyprmodNoMatches"], 0);
    assert_eq!(comparison["counts"]["hyprmodRowsWithUsefulMetadata"], 21);
    assert_eq!(
        comparison["counts"]["hyprmodRowsWithRecoveryRollbackConfirmation"],
        0
    );
    assert_eq!(comparison["counts"]["rowsEnabled"], 0);
    assert_eq!(comparison["counts"]["finalWritableRows"], 275);
    assert_eq!(comparison["counts"]["finalBlockedRows"], 66);
    assert_eq!(comparison["counts"]["cursorInputBlockedRows"], 21);
    assert_eq!(comparison["counts"]["displayRenderBlockedRows"], 23);
    assert_eq!(comparison["counts"]["debugCrashBlockedRows"], 22);
    assert_eq!(comparison["counts"]["writeAllowlistChanged"], false);
    assert_eq!(comparison["counts"]["productionBehaviorChanged"], false);

    let rows = comparison["rows"].as_array().expect("comparison rows");
    assert_eq!(rows.len(), 21);
    let row_ids = rows
        .iter()
        .map(|row| row["rowId"].as_str().unwrap())
        .collect::<BTreeSet<_>>();
    assert!(!row_ids.contains("cursor.sync_gsettings_theme"));
    assert!(row_ids.contains("cursor.no_warps"));
    assert!(row_ids.contains("cursor.zoom_factor"));
    assert!(row_ids.contains("cursor.default_monitor"));

    for row in rows {
        assert!(row["hyprmodMatchStatus"].as_str().is_some());
        assert!(row["hyprmodValueType"].as_str().is_some());
        assert_eq!(
            row["hyprmodHasRecoveryRollbackConfirmation"].as_bool(),
            Some(false)
        );
        assert_eq!(row["ourCurrentWriteStatus"].as_str(), Some("high-risk"));
    }

    Ok(())
}

#[test]
fn hyprmod_cursor_input_deltas_keep_our_safety_model_stronger() -> Result<()> {
    let deltas = read_json("data/reports/hyprmod-cursor-input-risk-deltas.v0.55.2.json")?;

    assert_eq!(deltas["counts"]["rowsCompared"], 21);
    assert_eq!(deltas["counts"]["betterThanHyprmodRows"], 21);
    assert_eq!(deltas["counts"]["equalToHyprmodRows"], 21);
    assert_eq!(deltas["counts"]["worseThanHyprmodRows"], 0);
    assert_eq!(deltas["counts"]["hyprmodUsefulMetadataRows"], 21);
    assert_eq!(deltas["counts"]["hyprmodConflictingMetadataRows"], 0);
    assert_eq!(deltas["counts"]["hyprmodNoMatchRows"], 0);
    assert_eq!(deltas["counts"]["rowsEnabled"], 0);
    assert_eq!(deltas["counts"]["writeAllowlistChanged"], false);
    assert_eq!(deltas["counts"]["productionBehaviorChanged"], false);

    for row in deltas["rows"].as_array().expect("delta rows") {
        let statuses = row["comparisonStatuses"]
            .as_array()
            .expect("comparison statuses");
        assert!(statuses
            .iter()
            .any(|status| status.as_str() == Some("better-than-hyprmod")));
        assert!(statuses
            .iter()
            .any(|status| status.as_str() == Some("hyprmod-has-useful-metadata")));
        assert_eq!(
            row["recoveryPolicyComparison"].as_str(),
            Some("our-model-better: independent watchdog/dead-man recovery required; HyprMod cursor config recovery not found")
        );
        assert_eq!(
            row["recommendedAction"].as_str(),
            Some("keep-blocked; optionally adopt metadata labels/descriptions/bounds/choices where useful")
        );
    }

    Ok(())
}

#[test]
fn hyprmod_cursor_input_recommendation_selects_no_subset_or_enablement() -> Result<()> {
    let recommendation =
        read_json("data/reports/hyprmod-cursor-input-next-subset-recommendation.v0.55.2.json")?;

    assert_eq!(
        recommendation["recommendation"].as_str(),
        Some("no-next-subset-selected")
    );
    assert_eq!(recommendation["counts"]["rowsReviewed"], 21);
    assert_eq!(recommendation["counts"]["selectedRows"], 0);
    assert_eq!(recommendation["counts"]["rowsEnabled"], 0);
    assert_eq!(recommendation["counts"]["finalWritableRows"], 275);
    assert_eq!(recommendation["counts"]["finalBlockedRows"], 66);
    assert_eq!(recommendation["counts"]["cursorInputBlockedRows"], 21);
    assert_eq!(recommendation["counts"]["displayRenderBlockedRows"], 23);
    assert_eq!(recommendation["counts"]["debugCrashBlockedRows"], 22);
    assert_eq!(recommendation["counts"]["writeAllowlistChanged"], false);
    assert_eq!(recommendation["counts"]["productionBehaviorChanged"], false);
    assert!(recommendation["selectedRows"]
        .as_array()
        .expect("selected rows")
        .is_empty());
    assert_eq!(recommendation["safety"]["realConfigModified"], false);
    assert_eq!(recommendation["safety"]["activeRuntimeModified"], false);
    assert_eq!(recommendation["safety"]["reloadRun"], false);
    assert_eq!(recommendation["safety"]["evalRun"], false);
    assert_eq!(recommendation["safety"]["luaExecuted"], false);

    Ok(())
}

#[test]
fn hyprmod_cursor_input_metadata_adoption_is_report_only() -> Result<()> {
    let adoption = read_json("data/reports/hyprmod-cursor-input-metadata-adoption.v0.55.2.json")?;
    let provenance =
        read_json("data/reports/hyprmod-cursor-input-metadata-provenance.v0.55.2.json")?;
    let after =
        read_json("data/reports/cursor-input-metadata-after-hyprmod-adoption.v0.55.2.json")?;
    let coverage = read_json("data/reports/scalar-read-write-coverage.v0.55.2.json")?;

    assert_eq!(coverage["counts"]["writableRows"], 277);
    assert_eq!(coverage["counts"]["blockedWriteRows"], 64);
    assert_eq!(SAFE_WRITABLE_ROWS.len(), 277);

    assert_eq!(adoption["counts"]["rowsConsidered"], 21);
    assert_eq!(adoption["counts"]["rowsWithAdoptedMetadata"], 21);
    assert_eq!(adoption["counts"]["rowsWithMetadataProvenance"], 21);
    assert_eq!(
        adoption["counts"]["hyprmodRecoveryRollbackDeadManEvidenceRows"],
        0
    );
    assert_eq!(adoption["counts"]["labelsAdoptedRows"], 21);
    assert_eq!(adoption["counts"]["descriptionsAdoptedRows"], 21);
    assert_eq!(adoption["counts"]["defaultsAdoptedRows"], 21);
    assert_eq!(adoption["counts"]["choicesAdoptedRows"], 5);
    assert_eq!(adoption["counts"]["boundsAdoptedRows"], 4);
    assert_eq!(adoption["counts"]["rowsEnabled"], 0);
    assert_eq!(adoption["counts"]["finalWritableRows"], 275);
    assert_eq!(adoption["counts"]["finalBlockedRows"], 66);
    assert_eq!(adoption["counts"]["cursorInputBlockedRows"], 21);
    assert_eq!(adoption["counts"]["displayRenderBlockedRows"], 23);
    assert_eq!(adoption["counts"]["debugCrashBlockedRows"], 22);
    assert_eq!(adoption["counts"]["writeAllowlistChanged"], false);
    assert_eq!(adoption["counts"]["productionBehaviorChanged"], false);
    assert_eq!(adoption["counts"]["recoveryGateWeakenedRows"], 0);
    assert_eq!(adoption["subsetRecommendationChanged"], false);

    assert_eq!(provenance["counts"]["rows"], 21);
    assert_eq!(provenance["counts"]["rowsWithProvenance"], 21);
    assert_eq!(provenance["counts"]["copiedImplementationCodeRows"], 0);

    assert_eq!(after["counts"]["rows"], 21);
    assert_eq!(after["counts"]["writeStatusHighRiskRows"], 21);
    assert_eq!(after["counts"]["safeWriteSupportedFalseRows"], 21);
    assert_eq!(after["counts"]["recoveryGatePreservedRows"], 21);
    assert_eq!(after["counts"]["independentWatchdogRequiredRows"], 21);
    assert_eq!(after["counts"]["productionProjectionChangedRows"], 0);
    assert_eq!(after["counts"]["writeAllowlistChanged"], false);
    assert_eq!(after["counts"]["productionBehaviorChanged"], false);

    for row in after["rows"].as_array().expect("after rows") {
        assert_eq!(row["writeStatus"].as_str(), Some("high-risk"));
        assert_eq!(row["safeWriteSupported"].as_bool(), Some(false));
        assert_eq!(row["highRiskWarningPreserved"].as_bool(), Some(true));
        assert_eq!(row["recoveryGatePreserved"].as_bool(), Some(true));
        assert_eq!(
            row["independentWatchdogStillRequired"].as_bool(),
            Some(true)
        );
        assert_eq!(row["productionProjectionChanged"].as_bool(), Some(false));
        assert!(row["metadataAfterHyprModAdoption"]["label"]
            .as_str()
            .is_some());
        assert!(row["metadataAfterHyprModAdoption"]["description"]
            .as_str()
            .is_some());
        assert!(
            row["metadataAfterHyprModAdoption"]["sourceProvenance"]
                .as_array()
                .expect("source provenance")
                .len()
                >= 2
        );
    }

    for row in adoption["rows"].as_array().expect("adoption rows") {
        assert_eq!(row["safetyPolicyChanged"].as_bool(), Some(false));
        assert_eq!(row["writeStatusChanged"].as_bool(), Some(false));
        assert_eq!(row["safeWriteSupportedChanged"].as_bool(), Some(false));
        assert_eq!(row["recoveryGateWeakened"].as_bool(), Some(false));
        assert!(!row["adoptedFields"]
            .as_array()
            .expect("adopted fields")
            .is_empty());
        assert!(row["rejectedFields"]
            .as_array()
            .expect("rejected fields")
            .iter()
            .any(|field| field["field"].as_str() == Some("hyprmodExposureTreatment")));
        assert!(row["rejectedFields"]
            .as_array()
            .expect("rejected fields")
            .iter()
            .any(|field| field["field"].as_str() == Some("hyprmodRecoveryRollbackConfirmation")));
    }

    Ok(())
}
