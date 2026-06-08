use anyhow::Result;
use hyprland_settings::write_classification::SAFE_WRITABLE_ROWS;
use serde_json::Value;
use std::collections::BTreeSet;

const COMPLETION: &str =
    "data/reports/all-blocked-rows-autonomous-writability-completion.v0.55.2.json";
const ERROR_LOG: &str =
    "data/reports/all-blocked-rows-autonomous-error-and-future-research-log.v0.55.2.json";
const SUMMARY: &str = "data/reports/all-blocked-rows-autonomous-writability-summary.v0.55.2.json";
const REVIEW_LOG: &str = "docs/ALL-BLOCKED-ROWS-AUTONOMOUS-WRITABILITY-REVIEW-LOG.md";

fn read_json(path: &str) -> Result<Value> {
    Ok(serde_json::from_str(&std::fs::read_to_string(path)?)?)
}

fn completion_row_ids(report: &Value) -> BTreeSet<String> {
    report["rows"]
        .as_array()
        .unwrap()
        .iter()
        .map(|row| row["rowId"].as_str().unwrap().to_string())
        .collect()
}

#[test]
fn autonomous_completion_report_covers_all_63_rows_and_preserves_counts() -> Result<()> {
    let completion = read_json(COMPLETION)?;
    let summary = read_json(SUMMARY)?;
    let coverage = read_json("data/reports/scalar-read-write-coverage.v0.55.2.json")?;

    assert_eq!(
        completion["artifactKind"],
        "all-blocked-rows-autonomous-writability-completion"
    );
    assert_eq!(completion["startingCommit"], "f304481");
    assert_eq!(completion["inputBlockedRows"], 63);
    assert_eq!(completion["rowsProcessed"], 63);
    assert_eq!(completion["rowsEnabledThisSprint"], 0);
    assert_eq!(completion["rowsStillBlocked"], 63);
    assert_eq!(completion["safeWritableRowsBefore"], 278);
    assert_eq!(completion["safeWritableRowsAfter"], 278);
    assert_eq!(completion["writeAllowlistChanged"], false);
    assert_eq!(completion["countsBefore"]["readableRows"], 341);
    assert_eq!(completion["countsBefore"]["writableRows"], 278);
    assert_eq!(completion["countsBefore"]["blockedRows"], 63);
    assert_eq!(completion["countsAfter"]["readableRows"], 341);
    assert_eq!(completion["countsAfter"]["writableRows"], 278);
    assert_eq!(completion["countsAfter"]["blockedRows"], 63);

    assert_eq!(completion["rows"].as_array().unwrap().len(), 63);
    assert_eq!(summary["rowsEnabledThisSprint"], 0);
    assert_eq!(summary["blockedRowsAfter"], 63);
    assert_eq!(summary["writableRowsAfter"], 278);
    assert_eq!(summary["safeWritableRowsChanged"], false);
    assert_eq!(summary["writeAllowlistChanged"], false);
    assert_eq!(SAFE_WRITABLE_ROWS.len(), 278);
    assert_eq!(coverage["counts"]["readableRows"], 341);
    assert_eq!(coverage["counts"]["writableRows"], 278);
    assert_eq!(coverage["counts"]["blockedWriteRows"], 63);

    Ok(())
}

#[test]
fn review_log_has_step_1_through_11_for_every_blocked_row() -> Result<()> {
    let completion = read_json(COMPLETION)?;
    let review = std::fs::read_to_string(REVIEW_LOG)?;

    for row_id in completion_row_ids(&completion) {
        let heading = format!("## {row_id}");
        let start = review
            .find(&heading)
            .unwrap_or_else(|| panic!("missing review section for {row_id}"));
        let rest = &review[start + heading.len()..];
        let end = rest.find("\n## ").unwrap_or(rest.len());
        let section = &rest[..end];
        for step in [
            "### Starting status",
            "### Step 1 — Official evidence",
            "### Step 2 — Allowed values",
            "### Step 3 — Invalid-value behavior",
            "### Step 4 — Validators",
            "### Step 5 — Fixture write/reread",
            "### Step 6 — Safety gate",
            "### Step 7 — UI warning",
            "### Step 8 — Tests",
            "### Step 9 — Writability decision",
            "### Step 10 — Deeper official-source pass",
            "### Step 11 — Final blocker or completion",
        ] {
            assert!(
                section.contains(step),
                "{row_id} review section missing {step}"
            );
        }
    }

    assert_eq!(review.matches("\n## ").count(), 63);
    Ok(())
}

#[test]
fn no_row_is_enabled_without_complete_proof() -> Result<()> {
    let completion = read_json(COMPLETION)?;

    for row in completion["rows"].as_array().unwrap() {
        assert_eq!(row["enabledThisSprint"], false);
        assert_eq!(row["writabilityDecision"], "keepBlocked");
        assert_eq!(row["officialEvidenceResult"], "official-source-proven");
        assert_ne!(row["allowedValuesResult"], "notChecked");
        assert_ne!(row["invalidValueBehaviorResult"], "complete");
        assert!(row["validatorResult"]
            .as_str()
            .unwrap()
            .contains("not-added"));
        assert!(row["fixtureWriteRereadResult"]
            .as_str()
            .unwrap()
            .contains("notProven"));
        assert!(row["safetyGateResult"]
            .as_str()
            .unwrap()
            .contains("notProven"));
        assert!(row["uiWarningResult"]
            .as_str()
            .unwrap()
            .contains("notProven"));
        assert!(!row["exactBlockerIfStillBlocked"]
            .as_str()
            .unwrap()
            .is_empty());
        assert!(!row["futureResearchNeeded"].as_array().unwrap().is_empty());
    }

    Ok(())
}

#[test]
fn all_still_blocked_rows_are_in_error_log_with_exact_blockers() -> Result<()> {
    let completion = read_json(COMPLETION)?;
    let error_log = read_json(ERROR_LOG)?;

    let completion_ids = completion_row_ids(&completion);
    let error_ids: BTreeSet<String> = error_log["rows"]
        .as_array()
        .unwrap()
        .iter()
        .map(|row| row["rowId"].as_str().unwrap().to_string())
        .collect();
    assert_eq!(completion_ids, error_ids);
    assert_eq!(error_ids.len(), 63);

    for row in error_log["rows"].as_array().unwrap() {
        assert!(!row["exactBlocker"].as_str().unwrap().is_empty());
        assert!(!row["whatCodexTried"].as_array().unwrap().is_empty());
        assert!(!row["whyItStillCannotBeEnabled"]
            .as_str()
            .unwrap()
            .is_empty());
        assert!(!row["nextConcreteAction"].as_str().unwrap().is_empty());
    }

    assert_eq!(
        error_log["blockerCategories"]["missingSafetyGate"]
            .as_array()
            .unwrap()
            .len(),
        63
    );
    assert_eq!(
        error_log["blockerCategories"]["missingTests"]
            .as_array()
            .unwrap()
            .len(),
        63
    );
    assert_eq!(
        error_log["blockerCategories"]["requiresLiveRuntimeProof"]
            .as_array()
            .unwrap()
            .len(),
        63
    );

    Ok(())
}

#[test]
fn screen_shader_stays_closed_and_hyprmod_is_not_official_proof() -> Result<()> {
    let completion = read_json(COMPLETION)?;
    let closure = read_json("data/reports/screen-shader-track-closure.v0.55.2.json")?;

    assert_eq!(closure["screenShaderTrackClosedForNow"], true);
    assert_eq!(completion["hyprmodUsed"], false);
    assert_eq!(completion["hyprmodPolicyFollowed"], true);
    for row in completion["rows"].as_array().unwrap() {
        assert_ne!(row["officialEvidenceResult"], "hyprmod");
    }

    Ok(())
}

#[test]
fn aggregate_reports_link_to_autonomous_completion_without_count_changes() -> Result<()> {
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
            follow_up["allBlockedRowsAutonomousCompletionReport"], COMPLETION,
            "{path}"
        );
        assert_eq!(
            follow_up["allBlockedRowsAutonomousErrorFutureResearchLog"], ERROR_LOG,
            "{path}"
        );
        assert_eq!(
            follow_up["allBlockedRowsAutonomousSummaryReport"], SUMMARY,
            "{path}"
        );
        assert_eq!(
            follow_up["allBlockedRowsAutonomousRowsProcessed"], 63,
            "{path}"
        );
        assert_eq!(
            follow_up["allBlockedRowsAutonomousRowsEnabledThisSprint"], 0,
            "{path}"
        );
        assert_eq!(
            follow_up["allBlockedRowsAutonomousRowsStillBlocked"], 63,
            "{path}"
        );
        assert_eq!(
            follow_up["allBlockedRowsAutonomousSafeWritableRowsAfter"], 278,
            "{path}"
        );
        assert_eq!(
            follow_up["allBlockedRowsAutonomousWriteAllowlistChanged"], false,
            "{path}"
        );
    }

    Ok(())
}

#[test]
fn next_recommended_sprint_follows_remaining_blockers() -> Result<()> {
    let summary = read_json(SUMMARY)?;
    let next = summary["nextRecommendedSprint"].as_str().unwrap();
    assert_eq!(
        next,
        "Blocked high-risk rows safety-gate and invalid-value proof design sprint"
    );
    assert!(!next.to_ascii_lowercase().contains("enablement"));
    assert!(!next.contains("decoration.screen_shader"));
    assert_eq!(
        summary["topBlockerCategories"]["missingInvalidValueBehavior"],
        63
    );
    assert_eq!(summary["topBlockerCategories"]["missingSafetyGate"], 63);
    assert_eq!(summary["topBlockerCategories"]["missingUiWarning"], 63);
    assert_eq!(
        summary["topBlockerCategories"]["requiresLiveRuntimeProof"],
        63
    );

    Ok(())
}
