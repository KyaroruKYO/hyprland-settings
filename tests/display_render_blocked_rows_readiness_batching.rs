use anyhow::Result;
use hyprland_settings::write_classification::SAFE_WRITABLE_ROWS;
use serde_json::Value;

const REPORT: &str = "data/reports/display-render-blocked-rows-readiness-batching.v0.55.2.json";

fn read_json(path: &str) -> Result<Value> {
    Ok(serde_json::from_str(&std::fs::read_to_string(path)?)?)
}

#[test]
fn display_render_readiness_report_exists_and_preserves_counts() -> Result<()> {
    let report = read_json(REPORT)?;
    let coverage = read_json("data/reports/scalar-read-write-coverage.v0.55.2.json")?;

    assert_eq!(SAFE_WRITABLE_ROWS.len(), 341);
    assert_eq!(coverage["counts"]["readableRows"], 341);
    assert_eq!(coverage["counts"]["writableRows"], 341);
    assert_eq!(coverage["counts"]["blockedWriteRows"], 0);

    assert_eq!(
        report["artifactKind"],
        "display-render-blocked-rows-readiness-batching"
    );
    assert_eq!(report["startingCommit"], "761abf2");
    assert_eq!(report["currentCounts"]["readableRows"], 341);
    assert_eq!(report["currentCounts"]["writableRows"], 278);
    assert_eq!(report["currentCounts"]["blockedRows"], 63);
    assert_eq!(report["safeWritableRowsChanged"], false);
    assert_eq!(report["writeAllowlistChanged"], false);
    assert_eq!(report["rowsEnabledThisSprint"], 0);
    assert_eq!(report["screenShaderTrackStatus"], "closed-for-now");
    assert_eq!(report["selectedBucket"], "display-render-bucket-readiness");
    assert_eq!(report["bucketBlockedRows"], 23);
    assert_eq!(report["blockedIfProofMissing"], true);

    Ok(())
}

#[test]
fn exact_display_render_blocked_rows_are_recorded_with_required_fields() -> Result<()> {
    let report = read_json(REPORT)?;
    let rows = report["candidateRows"].as_array().unwrap();
    assert_eq!(rows.len(), 23);

    for row in rows {
        for field in [
            "rowId",
            "blockedReason",
            "valueShape",
            "riskReason",
            "readinessTier",
            "proofStillMissing",
        ] {
            assert!(
                row.get(field).is_some(),
                "{} missing {field}",
                row["rowId"].as_str().unwrap_or("<unknown>")
            );
        }
        assert_ne!(row["rowId"], "");
        assert_ne!(row["blockedReason"], "");
        assert_ne!(row["valueShape"], "");
        assert_ne!(row["riskReason"], "");
        assert_eq!(row["sourceEvidenceStatus"], "notProven");
        assert!(row["proofStillMissing"]
            .as_array()
            .unwrap()
            .iter()
            .any(|item| item.as_str().unwrap().contains("official source proof")));
    }

    assert!(rows.iter().any(|row| row["rowId"] == "xwayland.enabled"));
    assert!(rows
        .iter()
        .any(|row| row["rowId"] == "render.direct_scanout"));
    assert!(rows
        .iter()
        .any(|row| row["rowId"] == "experimental.wp_cm_1_2"));
    assert!(!rows
        .iter()
        .any(|row| row["rowId"] == "decoration.screen_shader"));

    Ok(())
}

#[test]
fn candidate_batch_remains_unresolved_and_non_enabling() -> Result<()> {
    let report = read_json(REPORT)?;

    assert_eq!(
        report["candidateBatch"],
        "unresolved-display-render-inventory"
    );
    assert!(report["candidateBatchSelectionReason"]
        .as_str()
        .unwrap()
        .contains("hard-excludes every remaining blocked display/render row"));
    assert_eq!(
        report["rowsExcludedFromBatch"].as_array().unwrap().len(),
        23
    );
    assert_eq!(
        report["nextRecommendedSprint"],
        "Display/render blocked rows source evidence inventory sprint"
    );
    assert!(!report["nextRecommendedSprint"]
        .as_str()
        .unwrap()
        .to_ascii_lowercase()
        .contains("enablement"));
    assert!(!report["nextRecommendedSprint"]
        .as_str()
        .unwrap()
        .contains("decoration.screen_shader"));

    Ok(())
}

#[test]
fn display_render_report_forbids_unsafe_inference_and_blocks_missing_proof() -> Result<()> {
    let report = read_json(REPORT)?;

    let do_not_infer = report["doNotInferSafetyFrom"].as_array().unwrap();
    for forbidden in [
        "parser acceptance",
        "HyprMod exposure",
        "UI metadata alone",
        "advisory helper existence alone",
        "advisory UI existence alone",
        "policy name without source-backed evidence and tests",
    ] {
        assert!(
            do_not_infer
                .iter()
                .any(|item| item.as_str().unwrap() == forbidden),
            "missing forbidden inference: {forbidden}"
        );
    }

    assert_eq!(report["blockedIfProofMissing"], true);
    assert!(report["proofStillMissing"]
        .as_array()
        .unwrap()
        .iter()
        .any(|item| item
            .as_str()
            .unwrap()
            .contains("small display/render candidate batch")));

    Ok(())
}

#[test]
fn aggregate_reports_link_display_render_readiness_inventory() -> Result<()> {
    let aggregate_paths = [
        "data/reports/all-341-unified-pipeline.v0.55.2.json",
        "data/reports/deferred-validator-remaining-items.v0.55.2.json",
        "data/reports/next-high-risk-bucket-readiness.v0.55.2.json",
        "data/reports/writable-value-type-evidence-matrix.v0.55.2.json",
        "data/reports/writable-value-type-gap-summary.v0.55.2.json",
    ];

    for path in aggregate_paths {
        let report = read_json(path)?;
        let follow_up = &report["screenShaderDisplayRenderReviewFollowUp"];
        assert_eq!(follow_up["screenShaderTrackClosedForNow"], true, "{path}");
        assert_eq!(
            follow_up["selectedNextBucket"], "display-render-bucket-readiness",
            "{path}"
        );
        assert_eq!(
            follow_up["displayRenderBlockedRowsReadinessBatchingReport"], REPORT,
            "{path}"
        );
        assert_eq!(
            follow_up["selectedNextBatch"], "unresolved-display-render-inventory",
            "{path}"
        );
        assert_eq!(follow_up["displayRenderCandidateRowsCount"], 23, "{path}");
        assert_eq!(
            follow_up["recommendedNextSprint"],
            "Display/render blocked rows source evidence inventory sprint",
            "{path}"
        );
        assert!(!follow_up["recommendedNextSprint"]
            .as_str()
            .unwrap()
            .to_ascii_lowercase()
            .contains("enablement"));
    }

    Ok(())
}
