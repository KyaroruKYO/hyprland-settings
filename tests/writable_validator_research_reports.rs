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

#[test]
fn writable_validator_research_reports_exist_and_preserve_current_counts() -> Result<()> {
    let summary = read_json("data/reports/writable-validator-research-summary.v0.55.2.json")?;
    let boolean = read_json("data/reports/writable-boolean-alias-research.v0.55.2.json")?;
    let numeric = read_json("data/reports/writable-numeric-bounds-research.v0.55.2.json")?;
    let parser = read_json("data/reports/writable-parser-only-row-research.v0.55.2.json")?;
    let complex = read_json("data/reports/writable-complex-grammar-research.v0.55.2.json")?;
    let coverage = read_json("data/reports/scalar-read-write-coverage.v0.55.2.json")?;

    assert_eq!(SAFE_WRITABLE_ROWS.len(), 278);
    assert_eq!(coverage["counts"]["writableRows"], 278);
    assert_eq!(coverage["counts"]["blockedWriteRows"], 63);

    for report in [&summary, &boolean, &numeric, &parser, &complex] {
        assert_eq!(report["counts"]["validatorsChanged"], false);
        assert_eq!(report["counts"]["writeAllowlistChanged"], false);
        assert_eq!(report["counts"]["productionBehaviorChanged"], false);
        assert_eq!(report["counts"]["rowsEnabledThisSprint"], 0);
    }

    assert_eq!(summary["counts"]["writableRows"], 278);
    assert_eq!(summary["counts"]["blockedRows"], 63);
    assert_eq!(summary["counts"]["recoveryGatesChanged"], false);
    assert_eq!(summary["counts"]["realConfigModified"], false);
    assert_eq!(summary["counts"]["activeRuntimeModified"], false);
    assert_eq!(summary["counts"]["reloadEvalLuaUsed"], false);

    Ok(())
}

#[test]
fn research_reports_cover_all_prior_gap_rows() -> Result<()> {
    let audit = read_json("data/reports/writable-value-type-exhaustiveness-audit.v0.55.2.json")?;
    let boolean = read_json("data/reports/writable-boolean-alias-research.v0.55.2.json")?;
    let numeric = read_json("data/reports/writable-numeric-bounds-research.v0.55.2.json")?;
    let parser = read_json("data/reports/writable-parser-only-row-research.v0.55.2.json")?;
    let complex = read_json("data/reports/writable-complex-grammar-research.v0.55.2.json")?;

    let boolean_ids = row_ids(&boolean);
    let numeric_ids = row_ids(&numeric);
    let parser_ids = row_ids(&parser);
    let complex_ids = row_ids(&complex);

    assert_eq!(boolean_ids.len(), 130);
    assert_eq!(numeric_ids.len(), 74);
    assert_eq!(parser_ids.len(), 2);
    assert_eq!(complex_ids.len(), 38);

    for row in audit["rows"].as_array().unwrap() {
        let row_id = row["rowId"].as_str().unwrap();
        let classification = row["classification"].as_str().unwrap();
        let kind = row["rustValueKind"].as_str().unwrap();
        match (classification, kind) {
            ("app-validator-too-broad", "Boolean") => {
                assert!(boolean_ids.contains(row_id), "{row_id}");
            }
            ("app-validator-too-broad", "Number" | "Percent") => {
                assert!(numeric_ids.contains(row_id), "{row_id}");
            }
            ("parser-only", _) => {
                assert!(parser_ids.contains(row_id), "{row_id}");
            }
            ("source-research-needed", _) => {
                assert!(complex_ids.contains(row_id), "{row_id}");
            }
            _ => {}
        }
    }

    Ok(())
}

#[test]
fn research_rows_include_future_actions_without_changing_behavior() -> Result<()> {
    let reports = [
        read_json("data/reports/writable-boolean-alias-research.v0.55.2.json")?,
        read_json("data/reports/writable-numeric-bounds-research.v0.55.2.json")?,
        read_json("data/reports/writable-parser-only-row-research.v0.55.2.json")?,
        read_json("data/reports/writable-complex-grammar-research.v0.55.2.json")?,
    ];

    for report in reports {
        for row in report["rows"].as_array().unwrap() {
            let row_id = row["rowId"].as_str().unwrap();
            assert!(
                !row["recommendedFutureAction"]
                    .as_str()
                    .unwrap_or_default()
                    .is_empty(),
                "{row_id}"
            );
            assert!(
                !row["classification"]
                    .as_str()
                    .unwrap_or_default()
                    .is_empty(),
                "{row_id}"
            );
        }
        assert_eq!(report["safety"]["validatorsChanged"], false);
        assert_eq!(report["safety"]["safeWritableRowsChanged"], false);
        assert_eq!(report["safety"]["writeBehaviorChanged"], false);
        assert_eq!(report["safety"]["realConfigModified"], false);
        assert_eq!(report["safety"]["activeRuntimeModified"], false);
        assert_eq!(report["safety"]["recoveryGatesChanged"], false);
    }

    Ok(())
}

#[test]
fn writable_validator_research_summary_has_prompt_three_scope() -> Result<()> {
    let summary = read_json("data/reports/writable-validator-research-summary.v0.55.2.json")?;
    let plan = summary["prompt3RepairPlan"].as_array().unwrap();
    let groups = plan
        .iter()
        .map(|group| group["groupId"].as_str().unwrap())
        .collect::<BTreeSet<_>>();

    for expected in [
        "boolean-alias-policy-decision",
        "numeric-bounds-repair-candidates",
        "parser-only-semantic-validator-candidates",
        "color-gradient-vector-grammar-research-candidates",
        "string-path-regex-list-grammar-research-candidates",
        "rows-safe-to-leave-unchanged-for-now",
    ] {
        assert!(groups.contains(expected), "{expected}");
    }

    assert_eq!(summary["counts"]["appValidatorTooBroadRows"], 204);
    assert_eq!(summary["counts"]["sourceResearchNeededRows"], 38);
    assert_eq!(summary["counts"]["parserOnlyRows"], 2);
    assert_eq!(summary["safety"]["safeWritableRowsChanged"], false);
    assert_eq!(summary["safety"]["writeBehaviorChanged"], false);
    assert_eq!(summary["safety"]["recoveryGatesChanged"], false);

    Ok(())
}
