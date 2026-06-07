use anyhow::Result;
use hyprland_settings::write_classification::SAFE_WRITABLE_ROWS;
use serde_json::Value;
use std::collections::BTreeSet;

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

fn assert_research_safety(report: &Value) {
    assert_eq!(report["counts"]["validatorsChanged"], false);
    assert_eq!(report["counts"]["writeAllowlistChanged"], false);
    assert_eq!(report["counts"]["productionBehaviorChanged"], false);
    assert_eq!(report["counts"]["recoveryGatesChanged"], false);
    assert_eq!(report["counts"]["rowsEnabledThisSprint"], 0);

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
fn official_source_research_reports_exist_and_preserve_state() -> Result<()> {
    let summary =
        read_json("data/reports/official-writable-validator-source-research-summary.v0.55.2.json")?;
    let boolean = read_json("data/reports/official-bool-parser-policy-research.v0.55.2.json")?;
    let numeric = read_json("data/reports/official-numeric-bounds-source-research.v0.55.2.json")?;
    let parser = read_json("data/reports/official-parser-only-semantic-research.v0.55.2.json")?;
    let complex = read_json("data/reports/official-complex-grammar-source-research.v0.55.2.json")?;
    let coverage = read_json("data/reports/scalar-read-write-coverage.v0.55.2.json")?;

    assert_eq!(SAFE_WRITABLE_ROWS.len(), 278);
    assert_eq!(coverage["counts"]["writableRows"], 278);
    assert_eq!(coverage["counts"]["blockedWriteRows"], 63);

    assert_eq!(summary["counts"]["writableRows"], 278);
    assert_eq!(summary["counts"]["blockedRows"], 63);
    assert_eq!(summary["counts"]["booleanAliasRows"], 130);
    assert_eq!(summary["counts"]["numericBoundsRows"], 74);
    assert_eq!(summary["counts"]["parserOnlyRows"], 2);
    assert_eq!(summary["counts"]["complexGrammarRows"], 38);
    assert_eq!(summary["counts"]["officialSourceMissingAreas"], 1);

    for report in [&summary, &boolean, &numeric, &parser, &complex] {
        assert_research_safety(report);
    }

    assert_eq!(summary["counts"]["realConfigModified"], false);
    assert_eq!(summary["counts"]["activeRuntimeModified"], false);
    assert_eq!(summary["counts"]["reloadEvalLuaUsed"], false);

    Ok(())
}

#[test]
fn official_reports_cover_all_prior_research_rows() -> Result<()> {
    let prior_boolean = read_json("data/reports/writable-boolean-alias-research.v0.55.2.json")?;
    let prior_numeric = read_json("data/reports/writable-numeric-bounds-research.v0.55.2.json")?;
    let prior_parser = read_json("data/reports/writable-parser-only-row-research.v0.55.2.json")?;
    let prior_complex = read_json("data/reports/writable-complex-grammar-research.v0.55.2.json")?;

    let official_boolean =
        read_json("data/reports/official-bool-parser-policy-research.v0.55.2.json")?;
    let official_numeric =
        read_json("data/reports/official-numeric-bounds-source-research.v0.55.2.json")?;
    let official_parser =
        read_json("data/reports/official-parser-only-semantic-research.v0.55.2.json")?;
    let official_complex =
        read_json("data/reports/official-complex-grammar-source-research.v0.55.2.json")?;

    assert_eq!(row_ids(&prior_boolean), row_ids(&official_boolean));
    assert_eq!(row_ids(&prior_numeric), row_ids(&official_numeric));
    assert_eq!(row_ids(&prior_parser), row_ids(&official_parser));
    assert_eq!(row_ids(&prior_complex), row_ids(&official_complex));

    assert_eq!(row_ids(&official_boolean).len(), 130);
    assert_eq!(row_ids(&official_numeric).len(), 74);
    assert_eq!(row_ids(&official_parser).len(), 2);
    assert_eq!(row_ids(&official_complex).len(), 38);

    Ok(())
}

#[test]
fn official_source_research_classifies_key_policy_findings() -> Result<()> {
    let boolean = read_json("data/reports/official-bool-parser-policy-research.v0.55.2.json")?;
    let numeric = read_json("data/reports/official-numeric-bounds-source-research.v0.55.2.json")?;
    let parser = read_json("data/reports/official-parser-only-semantic-research.v0.55.2.json")?;
    let complex = read_json("data/reports/official-complex-grammar-source-research.v0.55.2.json")?;

    assert_eq!(boolean["counts"]["sourceBackedAliasRows"], 130);
    assert_eq!(boolean["counts"]["parserOnlyAliasRows"], 0);
    assert_eq!(
        boolean["counts"]["appAcceptedAliasesWithInsufficientOfficialEvidenceRows"],
        0
    );
    assert_eq!(boolean["counts"]["rowsRecommendedForUiTrueFalseOnly"], 130);

    assert_eq!(numeric["counts"]["sourceBackedBoundsRows"], 72);
    assert_eq!(numeric["counts"]["cssGapCustomRows"], 2);
    assert_eq!(numeric["counts"]["validatorNarrowingCandidates"], 72);

    assert_eq!(parser["counts"]["parserOnlyRowsResearched"], 2);
    assert_eq!(parser["counts"]["sourceSemanticsFoundRows"], 2);

    assert_eq!(complex["counts"]["complexGrammarRowsResearched"], 38);
    assert_eq!(complex["counts"]["tooBroadRows"], 18);
    assert_eq!(complex["counts"]["tooNarrowRows"], 22);
    assert_eq!(complex["counts"]["sourceResearchStillIncompleteRows"], 10);

    let parser_ids = row_ids(&parser);
    assert!(parser_ids.contains("master.center_master_fallback"));
    assert!(parser_ids.contains("scrolling.explicit_column_widths"));

    Ok(())
}

#[test]
fn prompt_four_repair_groups_are_actionable_without_changing_behavior() -> Result<()> {
    let summary =
        read_json("data/reports/official-writable-validator-source-research-summary.v0.55.2.json")?;
    let groups = summary["prompt4RepairGroups"].as_array().unwrap();

    assert_eq!(groups.len(), 9);

    let group_ids = groups
        .iter()
        .map(|group| group["groupId"].as_str().unwrap())
        .collect::<BTreeSet<_>>();

    for expected in [
        "safe-no-change-source-backed-bool-aliases",
        "validator-narrowing-source-bounds-numeric",
        "validator-broadening-cssgap",
        "parser-only-safe-no-change-center-master-fallback",
        "parser-only-repair-explicit-column-widths",
        "validator-broadening-color-source-grammar",
        "validator-mixed-gradient-source-grammar",
        "validator-narrowing-vector-source-grammar",
        "official-source-missing-consumer-string-path-regex-list",
    ] {
        assert!(group_ids.contains(expected), "{expected}");
    }

    for group in groups {
        let group_id = group["groupId"].as_str().unwrap();
        let rows = group["rows"].as_array().unwrap();
        assert!(!rows.is_empty(), "{group_id}");
        assert_eq!(group["rowCount"].as_u64().unwrap() as usize, rows.len());
        assert!(
            !group["evidence"].as_str().unwrap_or_default().is_empty(),
            "{group_id}"
        );
        assert!(
            !group["futureAction"]
                .as_str()
                .unwrap_or_default()
                .is_empty(),
            "{group_id}"
        );
        assert!(
            !group["validatorWouldChange"]
                .as_str()
                .unwrap_or_default()
                .is_empty(),
            "{group_id}"
        );
        assert!(
            !group["uiEditorWouldChange"]
                .as_str()
                .unwrap_or_default()
                .is_empty(),
            "{group_id}"
        );
        assert!(
            !group["reportsTestsWouldChange"]
                .as_str()
                .unwrap_or_default()
                .is_empty(),
            "{group_id}"
        );
        assert_eq!(group["highRiskRecoveryGatesAffected"], "no", "{group_id}");
    }

    Ok(())
}
