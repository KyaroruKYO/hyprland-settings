use std::collections::{BTreeSet, HashSet};
use std::fs;

use hyprland_settings::blocked_row_pre_enablement::blocked_pre_enablement_rows;
use hyprland_settings::high_risk_production_gate::{
    high_risk_production_gate_rows, HighRiskProductionGateDecisionKind, HighRiskProductionGateError,
};
use hyprland_settings::write_classification::{
    is_high_risk_gated_writable_setting, is_safe_writable_setting, SAFE_WRITABLE_ROWS,
};
use serde_json::Value;

const FREEZE_REPORT: &str =
    "data/reports/high-risk-dry-run-accepted-candidate-approval-freeze.v0.55.2.json";
const FREEZE_TESTS_REPORT: &str =
    "data/reports/high-risk-candidate-approval-freeze-tests.v0.55.2.json";
const FREEZE_BLOCKERS_REPORT: &str =
    "data/reports/high-risk-candidate-approval-freeze-blockers.v0.55.2.json";
const SCREEN_SHADER_CLOSURE_REPORT: &str = "data/reports/screen-shader-track-closure.v0.55.2.json";

fn read_json(path: &str) -> Value {
    let bytes = fs::read(path).unwrap_or_else(|error| panic!("failed to read {path}: {error}"));
    serde_json::from_slice(&bytes).unwrap_or_else(|error| panic!("failed to parse {path}: {error}"))
}

fn selected_rows(report: &Value) -> Vec<String> {
    report["candidateRows"]
        .as_array()
        .expect("candidate rows should be an array")
        .iter()
        .map(|row| {
            row["rowId"]
                .as_str()
                .expect("candidate row id should be a string")
                .to_string()
        })
        .collect()
}

#[test]
fn candidate_freeze_report_covers_all_rows_and_preserves_counts() {
    let report = read_json(FREEZE_REPORT);

    assert_eq!(report["rowsAnalyzed"], 63);
    assert_eq!(report["dryRunAcceptedRows"], 62);
    assert_eq!(report["dryRunRejectedRows"], 1);
    assert_eq!(report["rowsEnabledThisSprint"], 0);
    assert_eq!(report["safeWritableRowsChanged"], false);
    assert_eq!(report["writeAllowlistChanged"], false);
    assert_eq!(report["countsBefore"]["readable"], 341);
    assert_eq!(report["countsBefore"]["writable"], 278);
    assert_eq!(report["countsBefore"]["blocked"], 63);
    assert_eq!(report["countsAfter"]["readable"], 341);
    assert_eq!(report["countsAfter"]["writable"], 278);
    assert_eq!(report["countsAfter"]["blocked"], 63);

    assert_eq!(
        report["candidateRows"]
            .as_array()
            .expect("candidate rows should be an array")
            .len(),
        5
    );
    assert_eq!(
        report["excludedRowClassifications"]
            .as_array()
            .expect("excluded rows should be an array")
            .len(),
        58
    );
    assert_eq!(
        report["selectedCandidateRows"]
            .as_array()
            .expect("selected candidate ids should be an array")
            .len()
            + report["excludedRows"]
                .as_array()
                .expect("excluded row ids should be an array")
                .len(),
        63
    );
}

#[test]
fn selected_candidate_batch_is_small_and_source_justified() {
    let report = read_json(FREEZE_REPORT);
    let selected = selected_rows(&report);
    let selected_set = selected.iter().cloned().collect::<BTreeSet<_>>();

    assert!((3..=8).contains(&selected.len()));
    assert_eq!(
        selected_set,
        BTreeSet::from([
            "debug.colored_stdout_logs".to_string(),
            "debug.disable_logs".to_string(),
            "debug.disable_time".to_string(),
            "debug.enable_stdout_logs".to_string(),
            "debug.error_position".to_string(),
        ])
    );
    assert!(!selected_set.contains("cursor.default_monitor"));
    assert!(!selected_set.contains("debug.manual_crash"));

    for row in report["candidateRows"]
        .as_array()
        .expect("candidate rows should be an array")
    {
        assert_eq!(
            row["dryRunGateStatus"],
            "accepted-complete-temp-only-scaffold-proof"
        );
        assert_eq!(
            row["productionWriteStatus"],
            "refused-production-write-disabled"
        );
        assert!(row["currentAllowlistStatus"]
            .as_str()
            .expect("allowlist status should be a string")
            .contains("is_safe_writable_setting=false"));
        assert_eq!(row["approvalStatus"], "candidateForReview");
        assert_ne!(row["approvalStatus"], "approvedForFutureEnablementSprint");
        assert_eq!(
            row["liveRuntimeProofStatus"],
            "missing-or-explicit-future-waiver-required"
        );
        assert_eq!(
            row["requiredFutureApproval"],
            "explicit high-risk approval required; current status is candidateForReview only"
        );
        assert!(row["selectionReason"]
            .as_str()
            .expect("selection reason should be a string")
            .contains("debug logging/error presentation"));
    }
}

#[test]
fn cursor_default_monitor_and_manual_crash_are_excluded_with_exact_categories() {
    let report = read_json(FREEZE_REPORT);
    let excluded = report["excludedRowClassifications"]
        .as_array()
        .expect("excluded rows should be an array");

    let default_monitor = excluded
        .iter()
        .find(|row| row["rowId"] == "cursor.default_monitor")
        .expect("cursor.default_monitor should be excluded");
    assert_eq!(default_monitor["exclusionCategory"], "runtimeDynamic");
    assert_eq!(
        default_monitor["dryRunGateStatus"],
        "rejected-runtime-dynamic-oracle-missing"
    );
    assert!(default_monitor["reason"]
        .as_str()
        .expect("reason should be a string")
        .contains("runtime monitor-name allowlist/readback oracle proof"));

    let manual_crash = excluded
        .iter()
        .find(|row| row["rowId"] == "debug.manual_crash")
        .expect("debug.manual_crash should be excluded");
    assert_eq!(
        manual_crash["exclusionCategory"],
        "intentionalCrashOrCrashAdjacent"
    );
    assert!(manual_crash["reason"]
        .as_str()
        .expect("reason should be a string")
        .contains("manual crash"));
}

#[test]
fn excluded_rows_have_exact_categories_and_reasons() {
    let report = read_json(FREEZE_REPORT);
    let selected: HashSet<_> = selected_rows(&report).into_iter().collect();
    let excluded = report["excludedRowClassifications"]
        .as_array()
        .expect("excluded rows should be an array");
    let allowed_categories = HashSet::from([
        "runtimeDynamic",
        "intentionalCrashOrCrashAdjacent",
        "displayRenderLiveProofFirst",
        "cursorInputRecoveryConcern",
        "debugCrashDisruptionConcern",
        "requiresExplicitFutureLiveProof",
        "notBestFirstBatch",
        "dryRunRejected",
        "other",
    ]);

    for row in excluded {
        let row_id = row["rowId"].as_str().expect("row id should be a string");
        assert!(!selected.contains(row_id));
        let category = row["exclusionCategory"]
            .as_str()
            .expect("exclusion category should be a string");
        assert!(
            allowed_categories.contains(category),
            "unexpected category for {row_id}: {category}"
        );
        assert!(!row["reason"]
            .as_str()
            .expect("reason should be a string")
            .trim()
            .is_empty());
        assert!(!row["nextConcreteAction"]
            .as_str()
            .expect("next action should be a string")
            .trim()
            .is_empty());
    }
}

#[test]
fn frozen_gate_requirements_are_present_for_each_candidate() {
    let report = read_json(FREEZE_REPORT);
    let frozen = report["frozenProductionGateRequirements"]
        .as_array()
        .expect("frozen requirements should be an array");
    let selected: BTreeSet<_> = selected_rows(&report).into_iter().collect();

    assert_eq!(frozen.len(), selected.len());
    for row in frozen {
        let row_id = row["rowId"].as_str().expect("row id should be a string");
        assert!(selected.contains(row_id));
        assert_eq!(
            row["requiredProductionWriteRefusalUntilEnablementSprint"],
            true
        );
        assert_eq!(row["waiverStatus"], "notGranted");
        assert_eq!(
            row["liveRuntimeProofStatus"],
            "missing-or-explicit-future-waiver-required"
        );
        for key in [
            "requiredApproval",
            "requiredConfirmation",
            "requiredRollback",
            "requiredTimeoutBehavior",
            "requiredPersistedRecoveryPlan",
            "requiredDryRunGateProof",
            "futureEnablementRule",
        ] {
            assert!(
                !row[key].as_str().unwrap_or_default().trim().is_empty(),
                "{key} should be populated for {row_id}"
            );
        }
        assert!(row["futureEnablementRule"]
            .as_str()
            .expect("future enablement rule should be a string")
            .contains("explicit high-risk approval"));
    }
}

#[test]
fn current_high_risk_rows_are_allowlisted_but_naked_production_write_is_refused() {
    assert_eq!(SAFE_WRITABLE_ROWS.len(), 341);
    for row in blocked_pre_enablement_rows() {
        assert!(
            is_safe_writable_setting(row.row_id),
            "{} should now be writable through the high-risk gated path",
            row.row_id
        );
        assert!(
            is_high_risk_gated_writable_setting(row.row_id),
            "{} should remain high-risk gated",
            row.row_id
        );
    }

    let evaluations = high_risk_production_gate_rows();
    assert_eq!(evaluations.len(), 63);
    for evaluation in evaluations {
        assert_eq!(
            evaluation.decision.kind,
            HighRiskProductionGateDecisionKind::ProductionWriteRefused
        );
        assert!(evaluation
            .decision
            .errors
            .contains(&HighRiskProductionGateError::MissingRecoveryPlan));
        assert!(evaluation.is_safe_writable_setting);
    }
}

#[test]
fn screen_shader_remains_closed_by_default() {
    let report = read_json(SCREEN_SHADER_CLOSURE_REPORT);
    assert_eq!(report["screenShaderTrackClosedForNow"], true);
}

#[test]
fn blockers_report_groups_candidate_and_excluded_rows() {
    let blockers = read_json(FREEZE_BLOCKERS_REPORT);
    assert_eq!(blockers["rowsEnabledThisSprint"], 0);
    assert_eq!(blockers["rowsStillBlocked"], 63);
    assert_eq!(blockers["excludedRowsCount"], 58);
    assert_eq!(
        blockers["blockerCategories"]["runtimeDynamicState"]
            .as_array()
            .expect("runtime dynamic category should be an array")
            .len(),
        1
    );
    assert_eq!(
        blockers["blockerCategories"]["excludedFromFirstCandidateBatch"]
            .as_array()
            .expect("excluded category should be an array")
            .len(),
        58
    );
    assert_eq!(
        blockers["blockerCategories"]["notYetEnablementSprint"]
            .as_array()
            .expect("not enablement category should be an array")
            .len(),
        63
    );
}

#[test]
fn aggregate_reports_link_to_candidate_approval_freeze() {
    for path in [FREEZE_REPORT, FREEZE_TESTS_REPORT, FREEZE_BLOCKERS_REPORT] {
        assert!(std::path::Path::new(path).exists(), "{path} should exist");
    }

    let aggregate_paths = [
        "data/reports/all-341-unified-pipeline.v0.55.2.json",
        "data/reports/scalar-read-write-coverage.v0.55.2.json",
        "data/reports/deferred-validator-remaining-items.v0.55.2.json",
        "data/reports/next-high-risk-bucket-readiness.v0.55.2.json",
        "data/reports/writable-value-type-evidence-matrix.v0.55.2.json",
        "data/reports/writable-value-type-gap-summary.v0.55.2.json",
    ];

    for path in aggregate_paths {
        let report = read_json(path);
        let follow_up = &report["screenShaderDisplayRenderReviewFollowUp"];
        assert_eq!(
            follow_up["highRiskCandidateApprovalFreezeReport"],
            FREEZE_REPORT
        );
        assert_eq!(
            follow_up["highRiskCandidateApprovalFreezeTestsReport"],
            FREEZE_TESTS_REPORT
        );
        assert_eq!(
            follow_up["highRiskCandidateApprovalFreezeBlockersReport"],
            FREEZE_BLOCKERS_REPORT
        );
        assert_eq!(follow_up["highRiskCandidateApprovalFreezeRowsAnalyzed"], 63);
        assert_eq!(
            follow_up["highRiskCandidateApprovalFreezeDryRunAcceptedRows"],
            62
        );
        assert_eq!(
            follow_up["highRiskCandidateApprovalFreezeDryRunRejectedRows"],
            1
        );
        assert_eq!(
            follow_up["highRiskCandidateApprovalFreezeSelectedRowsCount"],
            5
        );
        assert_eq!(
            follow_up["highRiskCandidateApprovalFreezeExcludedRowsCount"],
            58
        );
        assert_eq!(
            follow_up["highRiskCandidateApprovalFreezeRowsEnabledThisSprint"],
            0
        );
    }
}
