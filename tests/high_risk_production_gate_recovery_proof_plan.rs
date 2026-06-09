use std::collections::BTreeSet;
use std::fs;

use anyhow::Result;
use hyprland_settings::blocked_row_pre_enablement::blocked_pre_enablement_rows;
use hyprland_settings::write_classification::SAFE_WRITABLE_ROWS;
use serde_json::Value;

const PLAN: &str = "data/reports/high-risk-production-gate-recovery-proof-plan.v0.55.2.json";
const MODELS: &str = "data/reports/high-risk-recovery-models-by-bucket.v0.55.2.json";
const CANDIDATES: &str =
    "data/reports/high-risk-production-gate-enablement-candidates.v0.55.2.json";
const BLOCKERS: &str = "data/reports/high-risk-production-gate-recovery-blockers.v0.55.2.json";
const REVIEW_LOG: &str = "docs/HIGH-RISK-PRODUCTION-GATE-RECOVERY-PROOF-REVIEW-LOG.md";

fn read_json(path: &str) -> Result<Value> {
    Ok(serde_json::from_str(&fs::read_to_string(path)?)?)
}

fn row_ids(rows: &[Value]) -> BTreeSet<String> {
    rows.iter()
        .map(|row| row["rowId"].as_str().unwrap().to_string())
        .collect()
}

fn spec_row_ids() -> BTreeSet<String> {
    blocked_pre_enablement_rows()
        .iter()
        .map(|row| row.row_id.to_string())
        .collect()
}

fn contains_string(array: &Value, needle: &str) -> bool {
    array
        .as_array()
        .unwrap()
        .iter()
        .any(|item| item.as_str().is_some_and(|value| value.contains(needle)))
}

#[test]
fn gate_recovery_plan_covers_all_rows_and_preserves_counts() -> Result<()> {
    let plan = read_json(PLAN)?;
    let coverage = read_json("data/reports/scalar-read-write-coverage.v0.55.2.json")?;

    assert_eq!(
        plan["artifactKind"],
        "high-risk-production-gate-recovery-proof-plan"
    );
    assert_eq!(plan["startingCommit"], "7a21f07");
    assert_eq!(plan["rowsAnalyzed"], 63);
    assert_eq!(plan["bucketsAnalyzed"].as_array().unwrap().len(), 3);
    assert_eq!(plan["rowsEnabledThisSprint"], 0);
    assert_eq!(plan["safeWritableRowsChanged"], false);
    assert_eq!(plan["writeAllowlistChanged"], false);
    assert_eq!(plan["countsBefore"]["readableRows"], 341);
    assert_eq!(plan["countsBefore"]["writableRows"], 278);
    assert_eq!(plan["countsBefore"]["blockedRows"], 63);
    assert_eq!(plan["countsAfter"]["readableRows"], 341);
    assert_eq!(plan["countsAfter"]["writableRows"], 278);
    assert_eq!(plan["countsAfter"]["blockedRows"], 63);
    assert_eq!(SAFE_WRITABLE_ROWS.len(), 278);
    assert_eq!(coverage["counts"]["readableRows"], 341);
    assert_eq!(coverage["counts"]["writableRows"], 278);
    assert_eq!(coverage["counts"]["blockedWriteRows"], 63);

    let rows = plan["rowClassifications"].as_array().unwrap();
    assert_eq!(rows.len(), 63);
    assert_eq!(row_ids(rows), spec_row_ids());

    Ok(())
}

#[test]
fn every_row_has_gate_recovery_and_blocker_decisions() -> Result<()> {
    let plan = read_json(PLAN)?;

    for row in plan["rowClassifications"].as_array().unwrap() {
        assert_eq!(
            row["preEnablementProofStatus"],
            "complete-pre-enablement-proof-only"
        );
        assert_eq!(row["productionGateRequired"], true);
        assert!(!row["recoveryModelRequired"].as_str().unwrap().is_empty());
        assert_eq!(row["nonLiveProofPossible"], true);
        assert_eq!(row["liveRuntimeProofRequired"], true);
        assert_eq!(row["explicitApprovalRequired"], true);
        assert_eq!(row["canBeEnabledWithoutLiveProof"], false);
        assert!(!row["recommendedNextAction"].as_str().unwrap().is_empty());
        assert!(row["exactRemainingBlocker"]
            .as_str()
            .unwrap()
            .contains("missing production gate implementation"));
        assert!(row["exactRemainingBlocker"]
            .as_str()
            .unwrap()
            .contains("explicit high-risk enablement approval is missing"));
    }

    assert_eq!(
        plan["nonLiveProofPossibleRows"].as_array().unwrap().len(),
        63
    );
    assert_eq!(
        plan["liveRuntimeProofRequiredRows"]
            .as_array()
            .unwrap()
            .len(),
        63
    );
    assert_eq!(
        plan["explicitApprovalRequiredRows"]
            .as_array()
            .unwrap()
            .len(),
        63
    );

    Ok(())
}

#[test]
fn bucket_recovery_models_state_forbidden_dependencies() -> Result<()> {
    let models = read_json(MODELS)?;
    assert_eq!(
        models["artifactKind"],
        "high-risk-recovery-models-by-bucket"
    );
    assert_eq!(models["models"].as_array().unwrap().len(), 3);

    let model_for = |bucket: &str| {
        models["models"]
            .as_array()
            .unwrap()
            .iter()
            .find(|model| model["bucket"] == bucket)
            .unwrap_or_else(|| panic!("missing model for {bucket}"))
    };

    let display = model_for("display/render");
    assert!(contains_string(
        &display["mustNotDependOn"],
        "visible compositor output"
    ));
    assert!(contains_string(
        &display["safeNonLiveProofPieces"],
        "persisted plan"
    ));
    assert_eq!(display["rowsCovered"].as_array().unwrap().len(), 23);

    let cursor = model_for("cursor/input");
    for forbidden in [
        "pointer visibility",
        "mouse input",
        "app UI",
        "Hyprland keybinds",
    ] {
        assert!(
            contains_string(&cursor["mustNotDependOn"], forbidden),
            "cursor/input model missing forbidden dependency: {forbidden}"
        );
    }
    assert!(cursor["rowsExcludedOrSpecialCase"]
        .as_array()
        .unwrap()
        .iter()
        .any(|row| row["rowId"] == "cursor.default_monitor"));
    assert_eq!(cursor["rowsCovered"].as_array().unwrap().len(), 18);

    let debug = model_for("debug/crash");
    assert!(contains_string(
        &debug["mustNotDependOn"],
        "process that may be disrupted"
    ));
    assert!(contains_string(
        &debug["safeNonLiveProofPieces"],
        "external-process ownership model"
    ));
    assert_eq!(debug["rowsCovered"].as_array().unwrap().len(), 22);

    Ok(())
}

#[test]
fn candidate_report_keeps_all_rows_out_of_enablement() -> Result<()> {
    let candidates = read_json(CANDIDATES)?;
    let all_ids = spec_row_ids();

    assert_eq!(
        candidates["artifactKind"],
        "high-risk-production-gate-enablement-candidates"
    );
    assert_eq!(
        candidates["nonLiveGateCandidateRows"]
            .as_array()
            .unwrap()
            .len(),
        63
    );
    assert_eq!(
        candidates["liveProofRequiredRows"]
            .as_array()
            .unwrap()
            .len(),
        63
    );
    assert_eq!(
        candidates["explicitApprovalOnlyRows"]
            .as_array()
            .unwrap()
            .len(),
        0
    );
    assert_eq!(
        candidates["runtimeDynamicRows"].as_array().unwrap().len(),
        1
    );
    assert_eq!(
        candidates["runtimeDynamicRows"][0]["rowId"],
        "cursor.default_monitor"
    );
    assert_eq!(
        candidates["doNotEnableYetRows"].as_array().unwrap().len(),
        63
    );
    assert_eq!(
        row_ids(candidates["doNotEnableYetRows"].as_array().unwrap()),
        all_ids
    );

    for row in candidates["doNotEnableYetRows"].as_array().unwrap() {
        assert!(!row["reason"].as_str().unwrap().is_empty());
        assert!(!row["nextConcreteAction"].as_str().unwrap().is_empty());
    }

    Ok(())
}

#[test]
fn blockers_report_groups_all_remaining_gate_and_recovery_blockers() -> Result<()> {
    let blockers = read_json(BLOCKERS)?;

    assert_eq!(
        blockers["artifactKind"],
        "high-risk-production-gate-recovery-blockers"
    );
    assert_eq!(
        blockers["blockerCategories"]["missingProductionGate"]
            .as_array()
            .unwrap()
            .len(),
        63
    );
    assert_eq!(
        blockers["blockerCategories"]["missingRecoveryModel"]
            .as_array()
            .unwrap()
            .len(),
        0
    );
    assert_eq!(
        blockers["blockerCategories"]["missingPersistentPlan"]
            .as_array()
            .unwrap()
            .len(),
        63
    );
    assert_eq!(
        blockers["blockerCategories"]["missingOutOfBandConfirmation"]
            .as_array()
            .unwrap()
            .len(),
        63
    );
    assert_eq!(
        blockers["blockerCategories"]["missingRollbackProof"]
            .as_array()
            .unwrap()
            .len(),
        63
    );
    assert_eq!(
        blockers["blockerCategories"]["requiresLiveRuntimeProof"]
            .as_array()
            .unwrap()
            .len(),
        63
    );
    assert_eq!(
        blockers["blockerCategories"]["requiresExplicitApproval"]
            .as_array()
            .unwrap()
            .len(),
        63
    );
    assert_eq!(
        blockers["blockerCategories"]["dynamicRuntimeState"]
            .as_array()
            .unwrap()
            .len(),
        1
    );

    for category in [
        "missingProductionGate",
        "missingPersistentPlan",
        "missingOutOfBandConfirmation",
        "missingRollbackProof",
        "requiresLiveRuntimeProof",
        "requiresExplicitApproval",
    ] {
        let ids = row_ids(blockers["blockerCategories"][category].as_array().unwrap());
        assert_eq!(ids, spec_row_ids(), "{category}");
    }

    Ok(())
}

#[test]
fn review_log_contains_required_bucket_sections_and_all_rows() -> Result<()> {
    let review = fs::read_to_string(REVIEW_LOG)?;
    let plan = read_json(PLAN)?;

    for required in [
        "# High-risk Production Gate and Recovery Proof Review Log",
        "## Sprint summary",
        "## Bucket: display/render",
        "## Bucket: cursor/input",
        "## Bucket: debug/crash",
        "## Row-by-row classification",
        "## Projected next 3 steps",
    ] {
        assert!(review.contains(required), "review log missing {required}");
    }

    for required in [
        "### Failure modes",
        "### Production gate requirement",
        "### Recovery requirement",
        "### Non-live proof possible",
        "### Live/runtime proof required",
        "### Rows covered",
        "### Rows needing special handling",
        "### Next concrete action",
    ] {
        assert!(review.contains(required), "review log missing {required}");
    }

    for row_id in row_ids(plan["rowClassifications"].as_array().unwrap()) {
        assert!(
            review.contains(&format!("- Row: {row_id}")),
            "review log missing row classification for {row_id}"
        );
    }

    Ok(())
}

#[test]
fn aggregate_reports_link_gate_recovery_reports_without_count_changes() -> Result<()> {
    let aggregate_paths = [
        "data/reports/all-341-unified-pipeline.v0.55.2.json",
        "data/reports/scalar-read-write-coverage.v0.55.2.json",
        "data/reports/deferred-validator-remaining-items.v0.55.2.json",
        "data/reports/next-high-risk-bucket-readiness.v0.55.2.json",
        "data/reports/writable-value-type-evidence-matrix.v0.55.2.json",
        "data/reports/writable-value-type-gap-summary.v0.55.2.json",
    ];

    for path in aggregate_paths {
        let report = read_json(path)?;
        let follow_up = &report["screenShaderDisplayRenderReviewFollowUp"];
        assert_eq!(
            follow_up["highRiskProductionGateRecoveryProofPlanReport"],
            PLAN
        );
        assert_eq!(follow_up["highRiskRecoveryModelsByBucketReport"], MODELS);
        assert_eq!(
            follow_up["highRiskProductionGateEnablementCandidatesReport"],
            CANDIDATES
        );
        assert_eq!(
            follow_up["highRiskProductionGateRecoveryBlockersReport"],
            BLOCKERS
        );
        assert_eq!(follow_up["highRiskProductionGateRowsAnalyzed"], 63);
        assert_eq!(follow_up["highRiskProductionGateRowsEnabledThisSprint"], 0);
        assert_eq!(follow_up["highRiskProductionGateRowsStillBlocked"], 63);
        assert_eq!(follow_up["highRiskProductionGateBucketPlansCreated"], 3);
        assert_eq!(follow_up["highRiskProductionGateRecoveryModelsCreated"], 3);
        assert_eq!(
            follow_up["highRiskProductionGateNonLiveProofPossibleRows"],
            63
        );
        assert_eq!(
            follow_up["highRiskProductionGateLiveRuntimeProofRequiredRows"],
            63
        );
        assert_eq!(
            follow_up["highRiskProductionGateExplicitApprovalRequiredRows"],
            63
        );
        assert_eq!(
            follow_up["highRiskProductionGateSafeWritableRowsAfter"],
            278
        );
        assert_eq!(
            follow_up["highRiskProductionGateWriteAllowlistChanged"],
            false
        );
        assert_eq!(
            follow_up["highRiskProductionGateNextRecommendedSprint"],
            "Implement high-risk persisted recovery plan scaffold sprint"
        );
    }

    let closure = read_json("data/reports/screen-shader-track-closure.v0.55.2.json")?;
    assert_eq!(closure["screenShaderTrackClosedForNow"], true);

    Ok(())
}
