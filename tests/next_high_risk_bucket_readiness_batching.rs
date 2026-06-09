use anyhow::Result;
use hyprland_settings::write_classification::SAFE_WRITABLE_ROWS;
use serde_json::Value;

const BATCHING_REPORT: &str = "data/reports/next-high-risk-bucket-readiness-batching.v0.55.2.json";

fn read_json(path: &str) -> Result<Value> {
    Ok(serde_json::from_str(&std::fs::read_to_string(path)?)?)
}

#[test]
fn batching_report_exists_and_preserves_counts() -> Result<()> {
    let report = read_json(BATCHING_REPORT)?;
    let coverage = read_json("data/reports/scalar-read-write-coverage.v0.55.2.json")?;

    assert_eq!(SAFE_WRITABLE_ROWS.len(), 341);
    assert_eq!(coverage["counts"]["readableRows"], 341);
    assert_eq!(coverage["counts"]["writableRows"], 341);
    assert_eq!(coverage["counts"]["blockedWriteRows"], 0);

    assert_eq!(
        report["artifactKind"],
        "next-high-risk-bucket-readiness-batching"
    );
    assert_eq!(report["startingCommit"], "8cdf69c");
    assert_eq!(report["currentCounts"]["readableRows"], 341);
    assert_eq!(report["currentCounts"]["writableRows"], 278);
    assert_eq!(report["currentCounts"]["blockedRows"], 63);
    assert_eq!(report["safeWritableRowsChanged"], false);
    assert_eq!(report["writeAllowlistChanged"], false);
    assert_eq!(report["rowsEnabledThisSprint"], 0);
    assert_eq!(report["screenShaderTrackStatus"], "closed-for-now");

    Ok(())
}

#[test]
fn batching_report_records_remaining_buckets_and_selected_target() -> Result<()> {
    let report = read_json(BATCHING_REPORT)?;

    assert_eq!(
        report["remainingBlockedBuckets"]["displayRender"]["blockedRows"],
        23
    );
    assert_eq!(
        report["remainingBlockedBuckets"]["cursorInput"]["blockedRows"],
        18
    );
    assert_eq!(
        report["remainingBlockedBuckets"]["debugCrash"]["blockedRows"],
        22
    );

    assert_eq!(
        report["selectedNextBucket"],
        "display-render-bucket-readiness"
    );
    assert_eq!(
        report["selectedNextBatch"]["name"],
        "non-enabling display/render readiness inventory for non-display-critical boolean/finite-choice candidates"
    );
    assert_eq!(report["selectedNextBatch"]["enableRows"], false);
    assert!(report["selectionReason"]
        .as_str()
        .unwrap()
        .contains("Existing project evidence identifies display/render"));
    assert_eq!(
        report["nextRecommendedSprint"],
        "Display/render blocked rows readiness batching sprint"
    );
    assert!(!report["nextRecommendedSprint"]
        .as_str()
        .unwrap()
        .contains("screen_shader"));
    assert!(!report["nextRecommendedSprint"]
        .as_str()
        .unwrap()
        .contains("decoration.screen_shader"));

    Ok(())
}

#[test]
fn batching_policy_requires_specific_proof_and_blocks_missing_proof() -> Result<()> {
    let report = read_json(BATCHING_REPORT)?;

    assert_eq!(report["rowSpecificProofRequired"], true);
    assert_eq!(report["bucketSpecificProofRequired"], true);
    assert_eq!(report["blockedIfProofMissing"], true);
    assert!(report["batchingPolicy"]
        .as_str()
        .unwrap()
        .contains("grouped high-risk bucket planning"));
    assert!(report["avoidRabbitHolePolicy"]
        .as_str()
        .unwrap()
        .contains("Do not continue decoration.screen_shader work"));
    assert!(report["sourceEvidencePolicy"]
        .as_str()
        .unwrap()
        .contains("official Hyprland documentation/source"));
    assert!(report["writeEnablementPolicy"]
        .as_str()
        .unwrap()
        .contains("This sprint enables no rows"));

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

    assert!(report["proofStillMissing"]
        .as_array()
        .unwrap()
        .iter()
        .any(|item| item
            .as_str()
            .unwrap()
            .contains("exact display/render row list")));

    Ok(())
}

#[test]
fn aggregate_reports_link_to_batching_plan() -> Result<()> {
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
            follow_up["nextHighRiskBucketReadinessBatchingReport"], BATCHING_REPORT,
            "{path}"
        );
        assert_eq!(
            follow_up["selectedNextBucket"], "display-render-bucket-readiness",
            "{path}"
        );
        assert_eq!(
            follow_up["selectedNextBatch"], "unresolved-display-render-inventory",
            "{path}"
        );
        assert_eq!(follow_up["displayRenderCandidateRowsCount"], 23, "{path}");
        assert_eq!(
            follow_up["nextWorkMode"], "grouped-high-risk-bucket-level-readiness",
            "{path}"
        );
        assert_eq!(
            follow_up["screenShaderSpecificWorkDefaultNext"], false,
            "{path}"
        );
        assert_eq!(
            follow_up["recommendedNextSprint"],
            "Display/render blocked rows source evidence inventory sprint",
            "{path}"
        );
    }

    Ok(())
}
