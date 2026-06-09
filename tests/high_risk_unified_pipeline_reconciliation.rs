use anyhow::Result;
use hyprland_settings::write_classification::{is_safe_writable_setting, SAFE_WRITABLE_ROWS};
use serde_json::Value;
use std::collections::{BTreeMap, BTreeSet};

fn read_json(path: &str) -> Result<Value> {
    Ok(serde_json::from_str(&std::fs::read_to_string(path)?)?)
}

fn row_map<'a>(report: &'a Value, key: &str) -> BTreeMap<&'a str, &'a Value> {
    report[key]
        .as_array()
        .unwrap()
        .iter()
        .map(|row| (row["rowId"].as_str().unwrap(), row))
        .collect()
}

fn non_empty(row: &Value, key: &str) -> bool {
    match &row[key] {
        Value::String(value) => !value.is_empty(),
        Value::Bool(_) | Value::Number(_) => true,
        Value::Array(values) => !values.is_empty(),
        Value::Object(values) => !values.is_empty(),
        Value::Null => false,
    }
}

#[test]
fn enabled_high_risk_rows_conform_to_unified_pipeline() -> Result<()> {
    let coverage = read_json("data/reports/scalar-read-write-coverage.v0.55.2.json")?;
    let pipeline = read_json("data/reports/all-341-unified-pipeline.v0.55.2.json")?;
    let candidates = read_json("data/reports/high-risk-write-candidates.v0.55.2.json")?;
    let audit = read_json("data/reports/high-risk-enabled-row-pipeline-audit.v0.55.2.json")?;

    let expected = BTreeSet::from([
        "ecosystem.enforce_permissions",
        "ecosystem.no_donation_nag",
        "ecosystem.no_update_news",
        "xwayland.force_zero_scaling",
        "xwayland.use_nearest_neighbor",
        "cursor.sync_gsettings_theme",
        "cursor.hide_on_touch",
        "cursor.hide_on_tablet",
        "cursor.hide_on_key_press",
    ]);
    let required_fields = [
        "rowId",
        "officialSetting",
        "valueType",
        "validatorRef",
        "validatorAuthority",
        "scope",
        "riskClass",
        "recoveryBucket",
        "applyPath",
        "rereadOracle",
        "approvalGate",
        "watchdogRequirement",
        "recoveryStrategy",
        "proofSource",
        "watchdogProofSource",
        "gateStatus",
        "preflightProofStatus",
        "commitProofStatus",
        "uiReviewWarning",
        "productionBehaviorChanged",
        "writeAllowlistChanged",
        "safeWriteSupported",
        "currentWriteStatus",
        "pipelineTemplate",
    ];

    assert_eq!(coverage["counts"]["writableRows"], 340);
    assert_eq!(coverage["counts"]["blockedWriteRows"], 1);
    assert_eq!(pipeline["counts"]["metadataGapRows"], 0);
    assert_eq!(SAFE_WRITABLE_ROWS.len(), 340);
    assert_eq!(audit["counts"]["enabledHighRiskRowsAudited"], 9);
    assert_eq!(audit["counts"]["rowsWithMissingUnifiedFields"], 0);
    assert_eq!(audit["counts"]["rowsFailingUnifiedPipelineConformance"], 0);

    let coverage_by_id = row_map(&coverage, "rows");
    let pipeline_by_id = row_map(&pipeline, "rows");
    let candidate_ids = candidates["items"]
        .as_array()
        .unwrap()
        .iter()
        .map(|row| row["rowId"].as_str().unwrap())
        .collect::<BTreeSet<_>>();

    for row_id in &expected {
        assert!(is_safe_writable_setting(row_id), "{row_id}");
        assert!(!candidate_ids.contains(row_id), "{row_id}");

        let coverage_row = coverage_by_id[row_id];
        assert_eq!(
            coverage_row["writeStatus"].as_str(),
            Some("writable"),
            "{row_id}"
        );
        assert_eq!(
            coverage_row["safeWriteSupported"].as_bool(),
            Some(true),
            "{row_id}"
        );

        let pipeline_row = pipeline_by_id[row_id];
        assert_eq!(
            pipeline_row["currentWriteStatus"].as_str(),
            Some("writable"),
            "{row_id}"
        );
        assert_eq!(
            pipeline_row["safeWriteSupported"].as_bool(),
            Some(true),
            "{row_id}"
        );
        for field in required_fields {
            assert!(non_empty(pipeline_row, field), "{row_id} missing {field}");
        }
        assert!(
            pipeline_row["approvalGate"]
                .as_str()
                .unwrap()
                .contains("dead-man"),
            "{row_id}"
        );
        assert!(
            pipeline_row["watchdogRequirement"]
                .as_str()
                .unwrap()
                .contains("backup"),
            "{row_id}"
        );
        assert!(
            pipeline_row["uiReviewWarning"]
                .as_str()
                .unwrap()
                .contains("High-risk"),
            "{row_id}"
        );
    }

    for row_id in candidate_ids {
        let coverage_row = coverage_by_id[row_id];
        if row_id == "cursor.default_monitor" {
            assert_ne!(
                coverage_row["writeStatus"].as_str(),
                Some("writable"),
                "{row_id}"
            );
            assert_eq!(
                coverage_row["safeWriteSupported"].as_bool(),
                Some(false),
                "{row_id}"
            );
            assert!(!is_safe_writable_setting(row_id), "{row_id}");
        } else {
            assert_eq!(
                coverage_row["writeStatus"].as_str(),
                Some("writable"),
                "{row_id}"
            );
            assert_eq!(
                coverage_row["safeWriteSupported"].as_bool(),
                Some(true),
                "{row_id}"
            );
            assert!(is_safe_writable_setting(row_id), "{row_id}");
        }
    }

    Ok(())
}

#[test]
fn enabled_high_risk_rows_map_to_reusable_templates() -> Result<()> {
    let template_report =
        read_json("data/reports/high-risk-pipeline-template-normalization.v0.55.2.json")?;
    let reconciliation =
        read_json("data/reports/high-risk-unified-pipeline-reconciliation.v0.55.2.json")?;

    assert_eq!(template_report["counts"]["templates"], 4);
    assert_eq!(template_report["counts"]["enabledHighRiskRowsMapped"], 9);
    assert_eq!(template_report["counts"]["rowsWithoutTemplate"], 0);
    assert_eq!(template_report["counts"]["missingTemplateFields"], 0);

    let template_rows = template_report["templates"]
        .as_array()
        .unwrap()
        .iter()
        .map(|template| {
            (
                template["templateId"].as_str().unwrap(),
                template["rowCount"].as_u64().unwrap(),
            )
        })
        .collect::<BTreeMap<_, _>>();
    assert_eq!(template_rows["high-risk-policy-watchdog-template"], 3);
    assert_eq!(template_rows["display-render-watchdog-template"], 2);
    assert_eq!(
        template_rows["cursor-input-theme-sync-watchdog-template"],
        1
    );
    assert_eq!(
        template_rows["cursor-visibility-conditional-watchdog-template"],
        3
    );

    assert_eq!(
        reconciliation["counts"]["rowsFailingUnifiedPipelineConformance"],
        0
    );
    assert_eq!(reconciliation["counts"]["rowsMissingProofMetadata"], 0);
    assert_eq!(reconciliation["counts"]["rowsMissingWatchdogMetadata"], 0);
    assert_eq!(
        reconciliation["counts"]["rowsStillIncorrectlyListedAsHighRiskCandidates"],
        0
    );
    assert_eq!(reconciliation["counts"]["rowsEnabledThisSprint"], 0);
    assert_eq!(
        reconciliation["counts"]["writeAllowlistChangedThisSprint"],
        false
    );
    assert_eq!(
        reconciliation["counts"]["productionBehaviorChangedThisSprint"],
        false
    );
    assert_eq!(reconciliation["counts"]["recoveryGateWeakenedRows"], 0);
    assert_eq!(reconciliation["counts"]["realConfigModified"], false);
    assert_eq!(reconciliation["counts"]["activeRuntimeModified"], false);
    assert_eq!(reconciliation["counts"]["reloadEvalLuaUsed"], false);

    Ok(())
}

#[test]
fn high_risk_candidate_counts_match_current_blocked_state() -> Result<()> {
    let coverage = read_json("data/reports/scalar-read-write-coverage.v0.55.2.json")?;
    let candidates = read_json("data/reports/high-risk-write-candidates.v0.55.2.json")?;
    let consistency = read_json("data/reports/high-risk-report-consistency-audit.v0.55.2.json")?;

    assert_eq!(coverage["counts"]["writableRows"], SAFE_WRITABLE_ROWS.len());
    assert_eq!(coverage["counts"]["blockedWriteRows"], 1);
    let current_blocked = coverage["rows"]
        .as_array()
        .expect("coverage rows should be an array")
        .iter()
        .filter(|row| row["writeStatus"].as_str() != Some("writable"))
        .map(|row| row["rowId"].as_str().expect("row id should be a string"))
        .collect::<BTreeSet<_>>();
    assert_eq!(current_blocked, BTreeSet::from(["cursor.default_monitor"]));
    assert_eq!(candidates["counts"]["rows"], 63);
    assert_eq!(candidates["counts"]["byRiskClass"]["cursor_input_risk"], 18);
    assert_eq!(
        candidates["counts"]["byRiskClass"]["display_render_session_risk"],
        23
    );
    assert_eq!(candidates["counts"]["byRiskClass"]["debug_crash_risk"], 22);

    assert_eq!(consistency["counts"]["sourceOfTruthMismatches"], 0);
    assert_eq!(consistency["counts"]["driftFoundRows"], 9);
    assert_eq!(consistency["counts"]["driftFixedRows"], 9);
    assert_eq!(consistency["counts"]["rowsEnabledThisSprint"], 0);
    assert_eq!(consistency["safety"]["writeAllowlistChanged"], false);
    assert_eq!(consistency["safety"]["productionBehaviorChanged"], false);
    assert_eq!(consistency["safety"]["recoveryGateWeakened"], false);

    Ok(())
}
