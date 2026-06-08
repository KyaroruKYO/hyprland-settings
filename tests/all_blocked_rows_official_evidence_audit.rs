use anyhow::Result;
use hyprland_settings::write_classification::SAFE_WRITABLE_ROWS;
use serde_json::Value;
use std::collections::BTreeSet;

const EVIDENCE_MATRIX: &str = "data/reports/all-blocked-rows-official-evidence-matrix.v0.55.2.json";
const ALLOWED_VALUES_MATRIX: &str =
    "data/reports/all-blocked-rows-allowed-values-matrix.v0.55.2.json";
const WRITABILITY_READINESS: &str =
    "data/reports/all-blocked-rows-writability-readiness.v0.55.2.json";
const ERROR_LOG: &str = "data/reports/all-blocked-rows-errors-and-further-research.v0.55.2.json";
const SUMMARY: &str = "data/reports/all-blocked-rows-official-evidence-summary.v0.55.2.json";

fn read_json(path: &str) -> Result<Value> {
    Ok(serde_json::from_str(&std::fs::read_to_string(path)?)?)
}

fn row_ids(report: &Value) -> BTreeSet<String> {
    report["rows"]
        .as_array()
        .unwrap()
        .iter()
        .map(|row| row["rowId"].as_str().unwrap().to_string())
        .collect()
}

#[test]
fn all_blocked_row_audit_reports_exist_and_preserve_counts() -> Result<()> {
    let evidence = read_json(EVIDENCE_MATRIX)?;
    let allowed = read_json(ALLOWED_VALUES_MATRIX)?;
    let readiness = read_json(WRITABILITY_READINESS)?;
    let errors = read_json(ERROR_LOG)?;
    let summary = read_json(SUMMARY)?;
    let coverage = read_json("data/reports/scalar-read-write-coverage.v0.55.2.json")?;

    assert_eq!(SAFE_WRITABLE_ROWS.len(), 278);
    assert_eq!(coverage["counts"]["readableRows"], 341);
    assert_eq!(coverage["counts"]["writableRows"], 278);
    assert_eq!(coverage["counts"]["blockedWriteRows"], 63);

    assert_eq!(evidence["blockedRowsTotal"], 63);
    assert_eq!(allowed["blockedRowsTotal"], 63);
    assert_eq!(readiness["blockedRowsTotal"], 63);
    assert_eq!(summary["totalBlockedRows"], 63);
    assert_eq!(
        errors["rowsNeedingFurtherResearch"]
            .as_array()
            .unwrap()
            .len(),
        63
    );

    for report in [&evidence, &allowed, &readiness] {
        assert_eq!(report["safeWritableRowsChanged"], false);
        assert_eq!(report["writeAllowlistChanged"], false);
        assert_eq!(report["rowsEnabledThisSprint"], 0);
        assert_eq!(report["currentCounts"]["readableRows"], 341);
        assert_eq!(report["currentCounts"]["writableRows"], 278);
        assert_eq!(report["currentCounts"]["blockedRows"], 63);
    }
    assert_eq!(summary["safeWritableRowsChanged"], false);
    assert_eq!(summary["writeAllowlistChanged"], false);
    assert_eq!(summary["rowsEnabledThisSprint"], 0);

    Ok(())
}

#[test]
fn all_three_row_matrices_cover_the_same_63_blocked_rows() -> Result<()> {
    let evidence = read_json(EVIDENCE_MATRIX)?;
    let allowed = read_json(ALLOWED_VALUES_MATRIX)?;
    let readiness = read_json(WRITABILITY_READINESS)?;

    let evidence_ids = row_ids(&evidence);
    let allowed_ids = row_ids(&allowed);
    let readiness_ids = row_ids(&readiness);

    assert_eq!(evidence_ids.len(), 63);
    assert_eq!(allowed_ids.len(), 63);
    assert_eq!(readiness_ids.len(), 63);
    assert_eq!(evidence_ids, allowed_ids);
    assert_eq!(evidence_ids, readiness_ids);

    Ok(())
}

#[test]
fn evidence_statuses_and_allowed_values_do_not_guess() -> Result<()> {
    let evidence = read_json(EVIDENCE_MATRIX)?;
    let allowed = read_json(ALLOWED_VALUES_MATRIX)?;

    for row in evidence["rows"].as_array().unwrap() {
        let official_status = row["officialSupportStatus"].as_str().unwrap();
        assert!(
            matches!(official_status, "official" | "notOfficial" | "notProven"),
            "unexpected official status for {}: {official_status}",
            row["rowId"].as_str().unwrap()
        );
        assert_ne!(row["currentStatus"], "writable");
        assert_eq!(row["readyForFutureEnablement"], false);
        assert_eq!(row["hyprmodCrossReferenceStatus"], "notUsed");
        assert_ne!(row["hyprmodCrossReferenceStatus"], "official");
    }

    for row in allowed["rows"].as_array().unwrap() {
        if row["allowedValuesStatus"] == "notProven" {
            assert!(
                row["allowedValues"].as_array().unwrap().is_empty(),
                "{} has unproven values but non-empty allowedValues",
                row["rowId"].as_str().unwrap()
            );
        }
        assert!(row.get("hyprmodHintOnly").is_some());
    }

    Ok(())
}

#[test]
fn no_row_is_ready_without_all_seven_writability_questions_proven() -> Result<()> {
    let readiness = read_json(WRITABILITY_READINESS)?;
    let summary = read_json(SUMMARY)?;

    assert_eq!(summary["readyForFutureEnablementCount"], 0);
    assert_eq!(summary["officialButBlockedCount"], 63);
    assert_eq!(summary["needsFurtherResearchCount"], 63);

    for row in readiness["rows"].as_array().unwrap() {
        assert_ne!(row["finalReadiness"], "readyForFutureEnablement");
        assert_eq!(row["testsProveAllOfThat"], false);
        assert!(row["missingProof"].as_array().unwrap().len() >= 4);
        assert_ne!(
            row["enablementRecommendation"].as_str().unwrap(),
            "enable-now"
        );
        assert!(row["enablementRecommendation"]
            .as_str()
            .unwrap()
            .contains("do-not-enable"));
    }

    Ok(())
}

#[test]
fn error_log_is_non_empty_and_records_main_missing_proof_categories() -> Result<()> {
    let errors = read_json(ERROR_LOG)?;
    let summary = read_json(SUMMARY)?;

    assert_eq!(
        errors["rowsNeedingFurtherResearch"]
            .as_array()
            .unwrap()
            .len(),
        63
    );
    assert_eq!(
        errors["rowsWithMissingInvalidValueBehavior"]
            .as_array()
            .unwrap()
            .len(),
        63
    );
    assert_eq!(
        errors["rowsWithMissingWriteReadbackProof"]
            .as_array()
            .unwrap()
            .len(),
        63
    );
    assert_eq!(
        errors["rowsWithMissingSafetyGate"]
            .as_array()
            .unwrap()
            .len(),
        63
    );
    assert_eq!(
        errors["rowsWithMissingUiWarning"].as_array().unwrap().len(),
        63
    );
    assert_eq!(errors["rowsWithMissingTests"].as_array().unwrap().len(), 63);
    assert_eq!(
        errors["hyprmodOnlyHintsNotAcceptedAsOfficial"]
            .as_array()
            .unwrap()
            .len(),
        0
    );
    assert_eq!(summary["byMissingProofType"]["invalidValueBehavior"], 63);
    assert_eq!(summary["byMissingProofType"]["writeReadback"], 63);
    assert_eq!(summary["byMissingProofType"]["safetyGate"], 63);
    assert_eq!(summary["byMissingProofType"]["uiWarning"], 63);
    assert_eq!(summary["byMissingProofType"]["tests"], 63);

    Ok(())
}

#[test]
fn screen_shader_remains_closed_and_next_sprint_is_not_direct_enablement() -> Result<()> {
    let closure = read_json("data/reports/screen-shader-track-closure.v0.55.2.json")?;
    let summary = read_json(SUMMARY)?;

    assert_eq!(closure["screenShaderTrackClosedForNow"], true);
    assert_eq!(closure["nextScreenShaderWorkPolicy"], "deferred-by-default; only resume decoration.screen_shader work with explicit user approval or a proven current safety failure");
    assert!(!summary["recommendedNextSprint"]
        .as_str()
        .unwrap()
        .to_ascii_lowercase()
        .contains("enablement"));
    assert!(!summary["recommendedNextSprint"]
        .as_str()
        .unwrap()
        .contains("decoration.screen_shader"));

    Ok(())
}

#[test]
fn aggregate_reports_link_to_all_blocked_rows_evidence_summary() -> Result<()> {
    let aggregate_paths = [
        "data/reports/all-341-unified-pipeline.v0.55.2.json",
        "data/reports/deferred-validator-remaining-items.v0.55.2.json",
        "data/reports/next-high-risk-bucket-readiness.v0.55.2.json",
        "data/reports/writable-value-type-evidence-matrix.v0.55.2.json",
        "data/reports/writable-value-type-gap-summary.v0.55.2.json",
        "data/reports/next-high-risk-bucket-readiness-batching.v0.55.2.json",
        "data/reports/display-render-blocked-rows-readiness-batching.v0.55.2.json",
    ];

    for path in aggregate_paths {
        let report = read_json(path)?;
        let follow_up = &report["screenShaderDisplayRenderReviewFollowUp"];
        assert_eq!(
            follow_up["allBlockedRowsOfficialEvidenceSummaryReport"], SUMMARY,
            "{path}"
        );
        assert_eq!(follow_up["allBlockedRowsAudited"], 63, "{path}");
        assert_eq!(
            follow_up["allBlockedRowsReadyForFutureEnablementCount"], 0,
            "{path}"
        );
        assert_eq!(
            follow_up["allBlockedRowsAuditRowsEnabledThisSprint"], 0,
            "{path}"
        );
        assert_eq!(
            follow_up["allBlockedRowsAuditSafeWritableRowsChanged"], false,
            "{path}"
        );
        assert_eq!(
            follow_up["allBlockedRowsAuditWriteAllowlistChanged"], false,
            "{path}"
        );
    }

    Ok(())
}
