use anyhow::Result;
use hyprland_settings::write_classification::SAFE_WRITABLE_ROWS;
use serde_json::Value;
use std::collections::BTreeSet;

const EXPECTED_DEFERRED_ROWS: &[&str] = &[
    "appearance.gaps_in",
    "appearance.gaps_out",
    "general.locale",
    "input.accel_profile",
    "input.scroll_points",
    "input.kb_file",
    "group.groupbar.font_family",
    "misc.font_family",
    "misc.splash_font_family",
    "misc.swallow_regex",
    "misc.swallow_exception_regex",
    "decoration.screen_shader",
];

fn read_json(path: &str) -> Result<Value> {
    Ok(serde_json::from_str(&std::fs::read_to_string(path)?)?)
}

fn row_ids(report: &Value) -> BTreeSet<String> {
    report["rows"]
        .as_array()
        .unwrap()
        .iter()
        .map(|row| row["rowId"].as_str().unwrap().to_owned())
        .collect()
}

fn expected_rows() -> BTreeSet<String> {
    EXPECTED_DEFERRED_ROWS
        .iter()
        .map(|row| row.to_string())
        .collect()
}

fn assert_research_safety(report: &Value) {
    assert_eq!(report["counts"]["validatorsChanged"], false);
    assert_eq!(report["counts"]["writeAllowlistChanged"], false);
    assert_eq!(report["counts"]["safeWritableRowsChanged"], false);
    assert_eq!(report["counts"]["rowsEnabledThisSprint"], 0);
    assert_eq!(report["counts"]["productionBehaviorChanged"], false);
    assert_eq!(report["counts"]["recoveryGatesChanged"], false);
    assert_eq!(report["counts"]["realConfigModified"], false);
    assert_eq!(report["counts"]["activeRuntimeModified"], false);
    assert_eq!(report["counts"]["reloadEvalLuaUsed"], false);

    if let Some(safety) = report.get("safety") {
        assert_eq!(safety["validatorsChanged"], false);
        assert_eq!(safety["safeWritableRowsChanged"], false);
        assert_eq!(safety["writeBehaviorChanged"], false);
        assert_eq!(safety["realConfigModified"], false);
        assert_eq!(safety["activeRuntimeModified"], false);
        assert_eq!(safety["recoveryGatesChanged"], false);
    }
}

#[test]
fn deferred_consumer_research_reports_exist_and_preserve_state() -> Result<()> {
    let coverage = read_json("data/reports/scalar-read-write-coverage.v0.55.2.json")?;
    let main = read_json("data/reports/deferred-consumer-source-validator-research.v0.55.2.json")?;
    let cssgap = read_json("data/reports/deferred-cssgap-consumer-research.v0.55.2.json")?;
    let string_path_font =
        read_json("data/reports/deferred-string-path-font-consumer-research.v0.55.2.json")?;
    let accel = read_json("data/reports/deferred-accel-profile-consumer-research.v0.55.2.json")?;
    let scroll = read_json("data/reports/deferred-scroll-points-consumer-research.v0.55.2.json")?;
    let regex = read_json("data/reports/deferred-regex-consumer-research.v0.55.2.json")?;
    let shader = read_json("data/reports/deferred-screen-shader-consumer-research.v0.55.2.json")?;
    let plan = read_json("data/reports/deferred-validator-repair-plan.v0.55.2.json")?;

    assert_eq!(SAFE_WRITABLE_ROWS.len(), 278);
    assert_eq!(coverage["counts"]["writableRows"], 278);
    assert_eq!(coverage["counts"]["blockedWriteRows"], 63);

    for report in [
        &main,
        &cssgap,
        &string_path_font,
        &accel,
        &scroll,
        &regex,
        &shader,
        &plan,
    ] {
        assert_research_safety(report);
    }

    assert_eq!(main["counts"]["deferredRows"], 12);
    assert_eq!(main["counts"]["sourceMissingRows"], 0);
    assert_eq!(main["counts"]["readyForValidatorRepairRows"], 7);
    assert_eq!(main["counts"]["shouldBecomeHighRiskRows"], 1);
    assert_eq!(plan["counts"]["officialSourceStillMissingRows"], 0);

    Ok(())
}

#[test]
fn all_deferred_rows_are_represented_with_source_and_policy() -> Result<()> {
    let main = read_json("data/reports/deferred-consumer-source-validator-research.v0.55.2.json")?;
    let plan = read_json("data/reports/deferred-validator-repair-plan.v0.55.2.json")?;

    assert_eq!(row_ids(&main), expected_rows());
    assert_eq!(row_ids(&plan), expected_rows());

    for row in main["rows"].as_array().unwrap() {
        assert!(row["rowId"].as_str().unwrap().contains('.'));
        assert!(row["officialSetting"].as_str().unwrap().contains('.'));
        let source_files = row["sourceFilesInspected"].as_array().unwrap();
        let missing_reason = row["missingSourceReason"].as_str();
        assert!(
            !source_files.is_empty() || missing_reason.is_some(),
            "row must have inspected source or explicit missing-source reason: {}",
            row["rowId"]
        );
        assert!(!row["consumerBehaviorFound"].as_str().unwrap().is_empty());
        assert!(!row["recommendedValidatorPolicy"]
            .as_str()
            .unwrap()
            .is_empty());
        assert!(!row["recommendedUiEditorPolicy"]
            .as_str()
            .unwrap()
            .is_empty());
        assert!(row.get("validatorWouldChange").is_some());
        assert!(row.get("reportsTestsWouldChange").is_some());
        assert_eq!(row["highRiskRecoveryGatesAffected"], false);
    }

    Ok(())
}

#[test]
fn family_reports_cover_their_exact_deferred_rows() -> Result<()> {
    let cssgap = read_json("data/reports/deferred-cssgap-consumer-research.v0.55.2.json")?;
    let string_path_font =
        read_json("data/reports/deferred-string-path-font-consumer-research.v0.55.2.json")?;
    let accel = read_json("data/reports/deferred-accel-profile-consumer-research.v0.55.2.json")?;
    let scroll = read_json("data/reports/deferred-scroll-points-consumer-research.v0.55.2.json")?;
    let regex = read_json("data/reports/deferred-regex-consumer-research.v0.55.2.json")?;
    let shader = read_json("data/reports/deferred-screen-shader-consumer-research.v0.55.2.json")?;

    assert_eq!(
        row_ids(&cssgap),
        BTreeSet::from(["appearance.gaps_in".into(), "appearance.gaps_out".into()])
    );
    assert_eq!(
        row_ids(&string_path_font),
        BTreeSet::from([
            "general.locale".into(),
            "input.kb_file".into(),
            "group.groupbar.font_family".into(),
            "misc.font_family".into(),
            "misc.splash_font_family".into(),
        ])
    );
    assert_eq!(
        row_ids(&accel),
        BTreeSet::from(["input.accel_profile".into()])
    );
    assert_eq!(
        row_ids(&scroll),
        BTreeSet::from(["input.scroll_points".into()])
    );
    assert_eq!(
        row_ids(&regex),
        BTreeSet::from([
            "misc.swallow_regex".into(),
            "misc.swallow_exception_regex".into(),
        ])
    );
    assert_eq!(
        row_ids(&shader),
        BTreeSet::from(["decoration.screen_shader".into()])
    );

    Ok(())
}

#[test]
fn repair_plan_groups_rows_without_changing_behavior() -> Result<()> {
    let plan = read_json("data/reports/deferred-validator-repair-plan.v0.55.2.json")?;
    let groups = plan["groups"].as_array().unwrap();

    for group in groups {
        assert!(!group["group"].as_str().unwrap().is_empty());
        assert!(group["rows"].is_array());
        assert!(!group["evidence"].as_str().unwrap().is_empty());
        assert!(!group["futureAction"].as_str().unwrap().is_empty());
        assert!(group.get("validatorWouldChange").is_some());
        assert!(group.get("uiEditorWouldChange").is_some());
        assert!(group.get("reportsTestsWouldChange").is_some());
        assert_eq!(group["highRiskRecoveryGatesAffected"], false);
    }

    let high_risk_group = groups
        .iter()
        .find(|group| group["group"].as_str() == Some("should-become-high-risk-review-candidate"))
        .expect("screen shader high-risk review group should exist");
    assert_eq!(
        high_risk_group["rows"],
        serde_json::json!(["decoration.screen_shader"])
    );

    let missing_source_group = groups
        .iter()
        .find(|group| group["group"].as_str() == Some("official-source-still-missing"))
        .expect("missing source group should exist");
    assert!(missing_source_group["rows"].as_array().unwrap().is_empty());

    Ok(())
}
