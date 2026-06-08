use std::collections::BTreeSet;
use std::fs;
use std::path::PathBuf;
use std::time::{SystemTime, UNIX_EPOCH};

use anyhow::Result;
use hyprland_settings::blocked_row_pre_enablement::{
    blocked_pre_enablement_rows, invalid_pre_enablement_example, pre_enablement_gate_projection,
    prove_fixture_write_reread, ui_warning_projection, valid_pre_enablement_example,
    validate_pre_enablement_value,
};
use hyprland_settings::write_classification::{is_safe_writable_setting, SAFE_WRITABLE_ROWS};
use serde_json::Value;

const PROOF: &str = "data/reports/all-blocked-rows-pre-enablement-proof.v0.55.2.json";
const BLOCKERS: &str = "data/reports/all-blocked-rows-pre-enablement-blockers.v0.55.2.json";
const SUMMARY: &str = "data/reports/all-blocked-rows-pre-enablement-summary.v0.55.2.json";
const REVIEW_LOG: &str = "docs/ALL-BLOCKED-ROWS-PRE-ENABLEMENT-PROOF-REVIEW-LOG.md";

fn read_json(path: &str) -> Result<Value> {
    Ok(serde_json::from_str(&fs::read_to_string(path)?)?)
}

fn temp_root(name: &str) -> Result<PathBuf> {
    let stamp = SystemTime::now().duration_since(UNIX_EPOCH)?.as_nanos();
    let root = std::env::temp_dir().join(format!(
        "hyprland-settings-pre-enablement-{name}-{}-{stamp}",
        std::process::id()
    ));
    fs::create_dir_all(&root)?;
    Ok(root)
}

fn proof_row_ids(report: &Value) -> BTreeSet<String> {
    report["rows"]
        .as_array()
        .unwrap()
        .iter()
        .map(|row| row["rowId"].as_str().unwrap().to_string())
        .collect()
}

#[test]
fn pre_enablement_report_covers_all_63_rows_and_preserves_counts() -> Result<()> {
    let proof = read_json(PROOF)?;
    let summary = read_json(SUMMARY)?;
    let coverage = read_json("data/reports/scalar-read-write-coverage.v0.55.2.json")?;

    assert_eq!(
        proof["artifactKind"],
        "all-blocked-rows-pre-enablement-proof"
    );
    assert_eq!(proof["startingCommit"], "5fde674");
    assert_eq!(proof["inputBlockedRows"], 63);
    assert_eq!(proof["rowsProcessed"], 63);
    assert_eq!(proof["rowsEnabledThisSprint"], 0);
    assert_eq!(proof["rowsStillBlocked"], 63);
    assert_eq!(proof["safeWritableRowsBefore"], 278);
    assert_eq!(proof["safeWritableRowsAfter"], 278);
    assert_eq!(proof["writeAllowlistChanged"], false);
    assert_eq!(proof["countsBefore"]["readableRows"], 341);
    assert_eq!(proof["countsBefore"]["writableRows"], 278);
    assert_eq!(proof["countsBefore"]["blockedRows"], 63);
    assert_eq!(proof["countsAfter"]["readableRows"], 341);
    assert_eq!(proof["countsAfter"]["writableRows"], 278);
    assert_eq!(proof["countsAfter"]["blockedRows"], 63);
    assert_eq!(proof["rows"].as_array().unwrap().len(), 63);

    assert_eq!(summary["rowsProcessed"], 63);
    assert_eq!(summary["rowsEnabledThisSprint"], 0);
    assert_eq!(summary["blockedRowsAfter"], 63);
    assert_eq!(summary["writableRowsAfter"], 278);
    assert_eq!(summary["safeWritableRowsChanged"], false);
    assert_eq!(summary["writeAllowlistChanged"], false);
    assert_eq!(SAFE_WRITABLE_ROWS.len(), 278);
    assert_eq!(coverage["counts"]["readableRows"], 341);
    assert_eq!(coverage["counts"]["writableRows"], 278);
    assert_eq!(coverage["counts"]["blockedWriteRows"], 63);

    let spec_ids: BTreeSet<_> = blocked_pre_enablement_rows()
        .iter()
        .map(|row| row.row_id.to_string())
        .collect();
    assert_eq!(proof_row_ids(&proof), spec_ids);
    assert_eq!(spec_ids.len(), 63);

    Ok(())
}

#[test]
fn proof_only_validators_accept_valid_values_and_reject_invalid_examples() {
    for row in blocked_pre_enablement_rows() {
        let valid = valid_pre_enablement_example(row);
        let invalid = invalid_pre_enablement_example(row);

        assert!(
            validate_pre_enablement_value(row, &valid).is_accepted(),
            "{} should accept valid fixture value {valid:?}",
            row.row_id
        );
        assert!(
            !validate_pre_enablement_value(row, &invalid).is_accepted(),
            "{} should reject invalid fixture value {invalid:?}",
            row.row_id
        );
        assert!(
            !is_safe_writable_setting(row.row_id),
            "{} must remain outside the production write allowlist",
            row.row_id
        );
    }
}

#[test]
fn temp_fixture_write_reread_and_rollback_proof_covers_every_blocked_row() -> Result<()> {
    for row in blocked_pre_enablement_rows() {
        let root = temp_root(row.row_id)?;
        let valid = valid_pre_enablement_example(row);
        let proof = prove_fixture_write_reread(row, &valid, &root)
            .unwrap_or_else(|error| panic!("{} fixture proof failed: {error}", row.row_id));

        assert!(proof.config_path.starts_with(std::env::temp_dir()));
        assert_eq!(proof.official_setting, row.official_setting);
        assert_eq!(proof.written_value, valid);
        assert_eq!(proof.reread_value.as_deref(), Some(valid.as_str()));
        assert!(proof.rollback_restored);
        assert!(!proof.production_allowlist_used);
        fs::remove_dir_all(root)?;
    }

    Ok(())
}

#[test]
fn safety_gate_and_ui_warning_projections_are_present_but_non_production() {
    for row in blocked_pre_enablement_rows() {
        let gate = pre_enablement_gate_projection(row);
        assert_eq!(gate.row_id, row.row_id);
        assert!(gate.gate_family.contains("pre-enablement-gate-model"));
        assert!(gate.ungated_write_rejected_by_current_allowlist);
        assert!(!gate.production_gate_added);
        assert!(gate
            .remaining_gate_blocker
            .contains("production-capable high-risk gate"));

        let warning = ui_warning_projection(row);
        assert_eq!(warning.row_id, row.row_id);
        assert!(warning.placement.contains("advanced/high-risk"));
        assert!(warning.warning.contains("High-risk") || warning.warning.contains("high-risk"));
        assert!(!warning.production_ui_wiring_added);
    }
}

#[test]
fn every_row_has_required_proof_statuses_and_still_blocked_exactly() -> Result<()> {
    let proof = read_json(PROOF)?;

    assert_eq!(proof["rowsWithValidatorProofAdded"], 63);
    assert_eq!(proof["rowsWithInvalidValueProofAdded"], 63);
    assert_eq!(proof["rowsWithFixtureWriteRereadProofAdded"], 63);
    assert_eq!(proof["rowsWithSafetyGateProofAdded"], 63);
    assert_eq!(proof["rowsWithUiWarningProofAdded"], 63);
    assert_eq!(proof["hyprmodUsed"], true);
    assert_eq!(proof["hyprmodPolicyFollowed"], true);
    assert_eq!(proof["realConfigTouched"], false);
    assert_eq!(proof["runtimeTouched"], false);
    assert_eq!(proof["reloadEvalLuaUsed"], false);

    for row in proof["rows"].as_array().unwrap() {
        assert!(!row["validatorProofStatus"].as_str().unwrap().is_empty());
        assert!(!row["invalidValueProofStatus"].as_str().unwrap().is_empty());
        assert!(!row["fixtureWriteRereadProofStatus"]
            .as_str()
            .unwrap()
            .is_empty());
        assert!(!row["safetyGateProofStatus"].as_str().unwrap().is_empty());
        assert!(!row["uiWarningProofStatus"].as_str().unwrap().is_empty());
        assert_ne!(row["hyprmodCompanionStatus"], "officialProof");
        assert_eq!(row["enabledThisSprint"], false);
        assert!(row["exactBlockerIfStillBlocked"]
            .as_str()
            .unwrap()
            .contains("production-capable high-risk safety gate"));
    }

    Ok(())
}

#[test]
fn still_blocked_rows_are_grouped_by_remaining_blocker() -> Result<()> {
    let proof = read_json(PROOF)?;
    let blockers = read_json(BLOCKERS)?;

    let proof_ids = proof_row_ids(&proof);
    let blocker_ids: BTreeSet<_> = blockers["rows"]
        .as_array()
        .unwrap()
        .iter()
        .map(|row| row["rowId"].as_str().unwrap().to_string())
        .collect();
    assert_eq!(proof_ids, blocker_ids);
    assert_eq!(blocker_ids.len(), 63);
    assert_eq!(
        blockers["blockerCategories"]["missingValidatorProof"]
            .as_array()
            .unwrap()
            .len(),
        0
    );
    assert_eq!(
        blockers["blockerCategories"]["missingInvalidValueBehavior"]
            .as_array()
            .unwrap()
            .len(),
        0
    );
    assert_eq!(
        blockers["blockerCategories"]["missingFixtureWriteReread"]
            .as_array()
            .unwrap()
            .len(),
        0
    );
    assert_eq!(
        blockers["blockerCategories"]["missingUiWarning"]
            .as_array()
            .unwrap()
            .len(),
        0
    );
    assert_eq!(
        blockers["blockerCategories"]["missingSafetyGate"]
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

    for row in blockers["rows"].as_array().unwrap() {
        assert!(!row["whatWasAttemptedThisSprint"]
            .as_array()
            .unwrap()
            .is_empty());
        assert!(row["whyItStillCannotBeEnabled"]
            .as_str()
            .unwrap()
            .contains("explicit approval"));
        assert!(!row["nextConcreteAction"].as_str().unwrap().is_empty());
    }

    Ok(())
}

#[test]
fn review_log_has_required_sections_for_every_blocked_row() -> Result<()> {
    let proof = read_json(PROOF)?;
    let review = fs::read_to_string(REVIEW_LOG)?;

    for row_id in proof_row_ids(&proof) {
        let heading = format!("## {row_id}");
        let start = review
            .find(&heading)
            .unwrap_or_else(|| panic!("missing review section for {row_id}"));
        let rest = &review[start + heading.len()..];
        let end = rest.find("\n## ").unwrap_or(rest.len());
        let section = &rest[..end];
        for required in [
            "### Starting blocker",
            "### Validator proof",
            "### Invalid-value behavior proof",
            "### Fixture write/reread proof",
            "### Safety gate proof",
            "### UI warning proof",
            "### Enablement decision",
        ] {
            assert!(
                section.contains(required),
                "{row_id} review section missing {required}"
            );
        }
        assert!(section.contains("- Attempted: yes"));
        assert!(section.contains("- Enabled this sprint: no"));
        assert!(section.contains("- Exact reason:"));
    }

    assert_eq!(review.matches("\n## ").count(), 63);
    Ok(())
}

#[test]
fn aggregate_reports_link_to_pre_enablement_summary_without_count_changes() -> Result<()> {
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
        assert_eq!(follow_up["allBlockedRowsPreEnablementProofReport"], PROOF);
        assert_eq!(
            follow_up["allBlockedRowsPreEnablementBlockersReport"],
            BLOCKERS
        );
        assert_eq!(
            follow_up["allBlockedRowsPreEnablementSummaryReport"],
            SUMMARY
        );
        assert_eq!(follow_up["allBlockedRowsPreEnablementRowsProcessed"], 63);
        assert_eq!(
            follow_up["allBlockedRowsPreEnablementRowsEnabledThisSprint"],
            0
        );
        assert_eq!(follow_up["allBlockedRowsPreEnablementRowsStillBlocked"], 63);
        assert_eq!(
            follow_up["allBlockedRowsPreEnablementSafeWritableRowsAfter"],
            278
        );
        assert_eq!(
            follow_up["allBlockedRowsPreEnablementWriteAllowlistChanged"],
            false
        );
    }

    Ok(())
}
