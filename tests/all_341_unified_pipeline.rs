use std::collections::{BTreeMap, BTreeSet};

use anyhow::Result;
use serde_json::Value;

use hyprland_settings::write_classification::SAFE_WRITABLE_ROWS;

fn coverage() -> Result<Value> {
    Ok(serde_json::from_str(include_str!(
        "../data/reports/scalar-read-write-coverage.v0.55.2.json"
    ))?)
}

fn all_pipeline() -> Result<Value> {
    Ok(serde_json::from_str(include_str!(
        "../data/reports/all-341-unified-pipeline.v0.55.2.json"
    ))?)
}

fn writable_proof() -> Result<Value> {
    Ok(serde_json::from_str(include_str!(
        "../data/reports/writable-253-unified-pipeline-proof.v0.55.2.json"
    ))?)
}

fn backfill_audit() -> Result<Value> {
    Ok(serde_json::from_str(include_str!(
        "../data/reports/all-341-unified-pipeline-backfill-audit.v0.55.2.json"
    ))?)
}

#[test]
fn all_341_pipeline_report_matches_current_scalar_counts() -> Result<()> {
    let coverage = coverage()?;
    let pipeline = all_pipeline()?;
    let writable_proof = writable_proof()?;
    let audit = backfill_audit()?;

    assert_eq!(coverage["counts"]["totalScalarRows"], 341);
    assert_eq!(coverage["counts"]["readableRows"], 341);
    assert_eq!(coverage["counts"]["writableRows"], 341);
    assert_eq!(coverage["counts"]["blockedWriteRows"], 0);

    assert_eq!(pipeline["counts"]["totalRows"], 341);
    assert_eq!(pipeline["counts"]["readableRows"], 341);
    assert_eq!(pipeline["counts"]["writableRows"], 341);
    assert_eq!(pipeline["counts"]["blockedRows"], 0);
    assert_eq!(pipeline["counts"]["safeWritableRowsFromRustTable"], 341);
    assert_eq!(pipeline["counts"]["highRiskRows"], 72);
    assert_eq!(pipeline["counts"]["sessionRuntimeSensitiveRows"], 0);
    assert_eq!(pipeline["counts"]["metadataGapRows"], 0);
    assert_eq!(pipeline["counts"]["behaviorMismatchRows"], 0);
    assert_eq!(pipeline["counts"]["writeAllowlistChanged"], true);
    assert_eq!(pipeline["counts"]["productionBehaviorChanged"], true);

    assert_eq!(writable_proof["counts"]["writableRows"], 341);
    assert_eq!(
        writable_proof["counts"]["safeWritableRowsFromRustTable"],
        341
    );
    assert_eq!(writable_proof["counts"]["metadataGapRows"], 0);
    assert_eq!(writable_proof["counts"]["behaviorMismatchRows"], 0);

    assert_eq!(audit["counts"]["totalRows"], 341);
    assert_eq!(audit["counts"]["writableRows"], 341);
    assert_eq!(audit["counts"]["blockedRows"], 0);
    assert_eq!(audit["counts"]["metadataGapRows"], 0);
    assert_eq!(audit["counts"]["behaviorMismatchRows"], 0);
    assert_eq!(audit["counts"]["rowsNeedingFutureCleanup"], 0);
    assert_eq!(audit["counts"]["writeAllowlistChanged"], true);
    assert_eq!(audit["counts"]["productionBehaviorChanged"], true);

    assert_eq!(SAFE_WRITABLE_ROWS.len(), 341);

    Ok(())
}

#[test]
fn every_scalar_row_has_complete_unified_pipeline_metadata() -> Result<()> {
    let pipeline = all_pipeline()?;
    let rows = pipeline["rows"]
        .as_array()
        .expect("rows should be an array");
    assert_eq!(rows.len(), 341);

    let required = [
        "rowId",
        "officialSetting",
        "key",
        "type",
        "valueType",
        "validatorRef",
        "validatorAuthority",
        "scope",
        "riskClass",
        "applyPath",
        "rereadOracle",
        "recoveryStrategy",
        "approvalGate",
        "preflightProofStatus",
        "commitProofStatus",
        "gateStatus",
    ];

    let ids = rows
        .iter()
        .map(|row| row["rowId"].as_str().unwrap().to_owned())
        .collect::<BTreeSet<_>>();
    assert_eq!(ids.len(), 341);

    for row in rows {
        let row_id = row["rowId"].as_str().unwrap();
        for field in required {
            let value = row[field]
                .as_str()
                .unwrap_or_else(|| panic!("{row_id} should have string field {field}"));
            assert!(!value.is_empty(), "{row_id} has empty {field}");
        }
        assert_eq!(
            row["pipelineMetadataComplete"].as_bool(),
            Some(true),
            "{row_id} should have complete metadata"
        );
        assert!(
            row["pipelineMetadataGaps"].as_array().unwrap().is_empty(),
            "{row_id} should not have metadata gaps"
        );
    }

    Ok(())
}

#[test]
fn writable_pipeline_rows_match_the_production_safe_write_table() -> Result<()> {
    let pipeline = all_pipeline()?;
    let proof = writable_proof()?;

    let production_ids = SAFE_WRITABLE_ROWS
        .iter()
        .map(|row| row.row_id.to_owned())
        .collect::<BTreeSet<_>>();

    let pipeline_writable_ids = pipeline["rows"]
        .as_array()
        .unwrap()
        .iter()
        .filter(|row| row["writeStatus"].as_str() == Some("writable"))
        .map(|row| row["rowId"].as_str().unwrap().to_owned())
        .collect::<BTreeSet<_>>();

    let proof_ids = proof["rows"]
        .as_array()
        .unwrap()
        .iter()
        .map(|row| row["rowId"].as_str().unwrap().to_owned())
        .collect::<BTreeSet<_>>();

    assert_eq!(pipeline_writable_ids, production_ids);
    assert_eq!(proof_ids, production_ids);

    for row in proof["rows"].as_array().unwrap() {
        let row_id = row["rowId"].as_str().unwrap();
        assert_eq!(row["safeWritableTablePresent"].as_bool(), Some(true));
        assert_eq!(row["coverageWritable"].as_bool(), Some(true));
        if row["proofSource"].as_str() == Some("high-risk-ecosystem-bucket-proof.v0.55.2.json") {
            assert_eq!(
                row["gateStatus"].as_str(),
                Some("passed-ecosystem-high-risk-watchdog-gate"),
                "{row_id} should use the ecosystem high-risk watchdog gate"
            );
            assert_eq!(
                row["applyPath"].as_str(),
                Some("persistent-config-write-with-backup-reread-and-high-risk-watchdog"),
                "{row_id} should record the high-risk watchdog apply path"
            );
            assert_eq!(
                row["recoveryStrategy"].as_str(),
                Some("backup-rollback-plus-independent-dead-man-watchdog"),
                "{row_id} should record the watchdog recovery strategy"
            );
            assert_eq!(row["productionBehaviorChanged"].as_bool(), Some(true));
            assert_eq!(row["writeAllowlistChanged"].as_bool(), Some(true));
        } else if row["proofSource"].as_str()
            == Some("xwayland-scaling-policy-smoke-subset-proof.v0.55.2.json")
        {
            assert_eq!(
                row["gateStatus"].as_str(),
                Some("passed-xwayland-scaling-display-render-watchdog-gate"),
                "{row_id} should use the XWayland display/render watchdog gate"
            );
            assert_eq!(
                row["applyPath"].as_str(),
                Some("persistent-config-write-with-backup-reread-and-display-render-watchdog"),
                "{row_id} should record the display/render watchdog apply path"
            );
            assert!(
                row["recoveryStrategy"]
                    .as_str()
                    .unwrap()
                    .contains("dead-man-watchdog"),
                "{row_id} should record the watchdog recovery strategy"
            );
            assert_eq!(row["productionBehaviorChanged"].as_bool(), Some(true));
            assert_eq!(row["writeAllowlistChanged"].as_bool(), Some(true));
        } else if row["proofSource"].as_str()
            == Some("cursor-theme-sync-policy-smoke-subset-proof.v0.55.2.json")
        {
            assert_eq!(
                row["gateStatus"].as_str(),
                Some("passed-cursor-theme-sync-cursor-input-watchdog-gate"),
                "{row_id} should use the cursor/input watchdog gate"
            );
            assert_eq!(
                row["applyPath"].as_str(),
                Some("persistent-config-write-with-backup-reread-and-cursor-input-watchdog"),
                "{row_id} should record the cursor/input watchdog apply path"
            );
            assert!(
                row["recoveryStrategy"]
                    .as_str()
                    .unwrap()
                    .contains("cursor-input-independent-confirmation"),
                "{row_id} should record input-independent watchdog recovery"
            );
            assert_eq!(row["productionBehaviorChanged"].as_bool(), Some(true));
            assert_eq!(row["writeAllowlistChanged"].as_bool(), Some(true));
        } else if row["proofSource"].as_str()
            == Some("cursor-visibility-conditional-policy-proof.v0.55.2.json")
        {
            assert_eq!(
                row["gateStatus"].as_str(),
                Some("passed-cursor-visibility-conditional-watchdog-gate"),
                "{row_id} should use the cursor visibility watchdog gate"
            );
            assert_eq!(
                row["applyPath"].as_str(),
                Some("persistent-config-write-with-backup-reread-and-cursor-visibility-watchdog"),
                "{row_id} should record the cursor visibility watchdog apply path"
            );
            assert!(
                row["recoveryStrategy"]
                    .as_str()
                    .unwrap()
                    .contains("cursor-visibility-independent-confirmation"),
                "{row_id} should record visibility-independent watchdog recovery"
            );
            assert_eq!(row["productionBehaviorChanged"].as_bool(), Some(true));
            assert_eq!(row["writeAllowlistChanged"].as_bool(), Some(true));
        } else if row["proofSource"].as_str()
            == Some("cursor-hide-on-key-press-usability-proof.v0.55.2.json")
        {
            assert_eq!(
                row["gateStatus"].as_str(),
                Some("passed-cursor-hide-on-key-press-keyboard-token-watchdog-gate"),
                "{row_id} should use the cursor visibility keyboard-token watchdog gate"
            );
            assert_eq!(
                row["applyPath"].as_str(),
                Some("persistent-config-write-with-backup-reread-and-cursor-visibility-keyboard-token-watchdog"),
                "{row_id} should record the keyboard-token watchdog apply path"
            );
            assert!(
                row["recoveryStrategy"]
                    .as_str()
                    .unwrap()
                    .contains("keyboard-token-confirmation"),
                "{row_id} should record keyboard-token watchdog recovery"
            );
            assert_eq!(row["productionBehaviorChanged"].as_bool(), Some(true));
            assert_eq!(row["writeAllowlistChanged"].as_bool(), Some(true));
        } else if row["scope"].as_str().is_some_and(|scope| {
            matches!(
                scope,
                "persistent-config-only" | "persistent-needs-reload" | "startup-only"
            ) && row["proofSource"].as_str() == Some("session-runtime-write-proof.v0.55.2.json")
        }) {
            assert_eq!(
                row["gateStatus"].as_str(),
                Some("passed-session-runtime-write-gate"),
                "{row_id} should use the session/runtime gate"
            );
        } else if row["proofSource"].as_str()
            == Some("high-risk-accepted-rows-enablement.v0.55.2.json")
        {
            assert_eq!(
                row["gateStatus"].as_str(),
                Some("passed-high-risk-production-gate-with-explicit-approval"),
                "{row_id} should use the high-risk production gate"
            );
            assert_eq!(
                row["applyPath"].as_str(),
                Some("gated-high-risk-persistent-config-write-with-persisted-recovery"),
                "{row_id} should use the persisted recovery high-risk write path"
            );
            assert!(
                row["recoveryStrategy"]
                    .as_str()
                    .unwrap()
                    .contains("persisted"),
                "{row_id} should record persisted high-risk recovery"
            );
            assert_eq!(row["productionBehaviorChanged"].as_bool(), Some(true));
            assert_eq!(row["writeAllowlistChanged"].as_bool(), Some(true));
        } else if row["proofSource"].as_str()
            == Some("cursor-default-monitor-runtime-oracle-proof.v0.55.2.json")
        {
            assert_eq!(
                row["gateStatus"].as_str(),
                Some("passed-high-risk-production-gate-with-explicit-approval-and-monitor-oracle"),
                "{row_id} should use the high-risk production gate plus monitor oracle"
            );
            assert_eq!(
                row["applyPath"].as_str(),
                Some("gated-high-risk-persistent-config-write-with-persisted-recovery-and-monitor-name-oracle"),
                "{row_id} should use the monitor-oracle high-risk write path"
            );
            assert!(
                row["recoveryStrategy"]
                    .as_str()
                    .unwrap()
                    .contains("persisted"),
                "{row_id} should record persisted recovery"
            );
            assert_eq!(row["productionBehaviorChanged"].as_bool(), Some(true));
            assert_eq!(row["writeAllowlistChanged"].as_bool(), Some(true));
        } else {
            assert_eq!(
                row["gateStatus"].as_str(),
                Some("passed-normal-write-gate"),
                "{row_id} should preserve the existing writable gate"
            );
            assert_eq!(
                row["applyPath"].as_str(),
                Some("persistent-config-write-with-backup-reread"),
                "{row_id} should preserve the existing safe write path"
            );
            assert_eq!(row["recoveryStrategy"].as_str(), Some("backup-rollback"));
        }
        if row["proofSource"].as_str() == Some("high-risk-accepted-rows-enablement.v0.55.2.json") {
            assert_eq!(
                row["rereadOracle"].as_str(),
                Some("file-reread-plus-recovery-rollback-parser-reread")
            );
        } else if row["proofSource"].as_str()
            == Some("cursor-default-monitor-runtime-oracle-proof.v0.55.2.json")
        {
            assert_eq!(
                row["rereadOracle"].as_str(),
                Some("runtime-monitor-name-oracle-plus-file-reread-plus-recovery-rollback-parser-reread")
            );
        } else {
            assert_eq!(row["rereadOracle"].as_str(), Some("file-reread"));
        }
        assert!(
            row["metadataGaps"].as_array().unwrap().is_empty(),
            "{row_id} should not have writable proof metadata gaps"
        );
        assert!(
            row["behaviorMismatch"].as_array().unwrap().is_empty(),
            "{row_id} should not have behavior mismatches"
        );
        if matches!(
            row["proofSource"].as_str(),
            Some("session-runtime-write-proof.v0.55.2.json")
                | Some("high-risk-ecosystem-bucket-proof.v0.55.2.json")
                | Some("xwayland-scaling-policy-smoke-subset-proof.v0.55.2.json")
                | Some("cursor-theme-sync-policy-smoke-subset-proof.v0.55.2.json")
                | Some("cursor-visibility-conditional-policy-proof.v0.55.2.json")
                | Some("cursor-hide-on-key-press-usability-proof.v0.55.2.json")
                | Some("high-risk-accepted-rows-enablement.v0.55.2.json")
                | Some("cursor-default-monitor-runtime-oracle-proof.v0.55.2.json")
        ) {
            assert_eq!(row["productionBehaviorChanged"].as_bool(), Some(true));
            assert_eq!(row["writeAllowlistChanged"].as_bool(), Some(true));
        } else {
            assert_eq!(row["productionBehaviorChanged"].as_bool(), Some(false));
            assert_eq!(row["writeAllowlistChanged"].as_bool(), Some(false));
        }
    }

    Ok(())
}

#[test]
fn blocked_pipeline_rows_remain_blocked_with_policy_metadata() -> Result<()> {
    let coverage = coverage()?;
    let pipeline = all_pipeline()?;

    let coverage_by_id = coverage["rows"]
        .as_array()
        .unwrap()
        .iter()
        .map(|row| (row["rowId"].as_str().unwrap(), row))
        .collect::<BTreeMap<_, _>>();

    let mut blocked = 0;
    let mut high_risk = 0;

    for row in pipeline["rows"].as_array().unwrap() {
        if row["writeStatus"].as_str() == Some("writable") {
            continue;
        }

        blocked += 1;
        let row_id = row["rowId"].as_str().unwrap();
        let coverage_row = coverage_by_id.get(row_id).unwrap();

        assert_ne!(coverage_row["writeStatus"].as_str(), Some("writable"));
        assert_eq!(coverage_row["safeWriteSupported"].as_bool(), Some(false));
        assert_ne!(row["gateStatus"].as_str(), Some("passed-normal-write-gate"));
        assert!(
            row["nextRequiredWork"].as_str().unwrap_or_default().len() > 10,
            "{row_id} should explain next required work"
        );

        match row["riskBucket"].as_str() {
            Some("display-render-recovery")
            | Some("cursor-input-recovery")
            | Some("debug-crash-recovery")
            | Some("ecosystem-permission-policy") => {
                high_risk += 1;
                assert!(
                    row["approvalGate"].as_str().unwrap().contains("dead-man"),
                    "{row_id} should retain a dead-man/recovery approval gate"
                );
                assert!(
                    row["applyPath"].as_str().unwrap().contains("blocked"),
                    "{row_id} should not gain an active apply path"
                );
            }
            other => panic!("{row_id} has unexpected blocked risk bucket {other:?}"),
        }
    }

    assert_eq!(blocked, 0);
    assert_eq!(high_risk, 0);

    Ok(())
}

#[test]
fn backfill_audit_records_no_behavior_or_allowlist_changes() -> Result<()> {
    let audit = backfill_audit()?;

    for row in audit["rows"].as_array().unwrap() {
        let row_id = row["rowId"].as_str().unwrap();
        assert_eq!(
            row["pipelineStatus"].as_str(),
            Some("complete"),
            "{row_id} should have complete pipeline status"
        );
        assert!(
            row["missingMetadataFields"].as_array().unwrap().is_empty(),
            "{row_id} should not have missing metadata fields"
        );
        assert!(
            row["behaviorPipelineMismatch"]
                .as_array()
                .unwrap()
                .is_empty(),
            "{row_id} should not have behavior/pipeline mismatches"
        );
        if row["productionBehaviorChanged"].as_bool() == Some(true) {
            assert_eq!(row["writeAllowlistChanged"].as_bool(), Some(true));
        } else {
            assert_eq!(row["productionBehaviorChanged"].as_bool(), Some(false));
            assert_eq!(row["writeAllowlistChanged"].as_bool(), Some(false));
        }
        assert_eq!(row["needsFutureCleanup"].as_bool(), Some(false));
    }

    Ok(())
}
