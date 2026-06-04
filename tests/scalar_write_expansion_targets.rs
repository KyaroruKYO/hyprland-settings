use anyhow::Result;
use std::collections::BTreeMap;

use serde_json::Value;

fn target_report() -> Result<Value> {
    Ok(serde_json::from_str(include_str!(
        "../data/reports/scalar-write-expansion-targets.v0.55.2.json"
    ))?)
}

#[test]
fn parser_needed_target_research_decisions_are_recorded() -> Result<()> {
    let report = target_report()?;
    let targets = report["targets"]
        .as_array()
        .expect("target report targets should be an array");
    let mut decisions = BTreeMap::new();

    for target in targets
        .iter()
        .filter(|target| target["targetGroup"].as_str() == Some("parser-needed"))
    {
        let decision = target["decision"]
            .as_str()
            .expect("parser target should have a decision");
        *decisions.entry(decision.to_string()).or_insert(0usize) += 1;
    }

    assert_eq!(decisions.get("enabled-parser-backed-color"), Some(&10));
    assert_eq!(
        decisions.get("enabled-parser-backed-line-safe-string"),
        Some(&4)
    );
    assert_eq!(decisions.get("enabled-parser-backed-vector"), Some(&6));
    assert_eq!(decisions.get("manual-review-needed"), Some(&17));

    Ok(())
}

#[test]
fn scalar_write_expansion_target_report_has_expected_counts() -> Result<()> {
    let report = target_report()?;
    let targets = report["targets"]
        .as_array()
        .expect("target report targets should be an array");

    assert_eq!(report["counts"]["validatorNeededTargets"], 14);
    assert_eq!(report["counts"]["parserNeededTargets"], 37);
    assert_eq!(report["counts"]["totalTargets"], 51);
    assert_eq!(report["counts"]["validatorNeededTargetsEnabled"], 14);
    assert_eq!(report["counts"]["validatorNeededTargetsBlocked"], 0);
    assert_eq!(report["counts"]["parserNeededTargetsEnabled"], 20);
    assert_eq!(report["counts"]["parserNeededTargetsBlocked"], 17);
    assert_eq!(targets.len(), 51);

    let validator_needed = targets
        .iter()
        .filter(|target| target["targetGroup"].as_str() == Some("validator-needed"))
        .count();
    let parser_needed = targets
        .iter()
        .filter(|target| target["targetGroup"].as_str() == Some("parser-needed"))
        .count();

    assert_eq!(validator_needed, 14);
    assert_eq!(parser_needed, 37);

    Ok(())
}

#[test]
fn scalar_write_expansion_targets_have_decisions_tests_and_blockers() -> Result<()> {
    let report = target_report()?;
    let targets = report["targets"]
        .as_array()
        .expect("target report targets should be an array");

    for target in targets {
        let row_id = target["rowId"]
            .as_str()
            .expect("target rowId should be a string");
        assert!(
            target["officialSetting"].as_str().is_some(),
            "{row_id} needs officialSetting"
        );
        assert!(
            target["decision"].as_str().is_some(),
            "{row_id} needs a decision"
        );
        let enabled = target["enabled"].as_bool().expect("enabled should be bool");
        let tests = target["tests"]
            .as_array()
            .expect("target tests should be an array");
        if enabled {
            assert!(
                !tests.is_empty(),
                "{row_id} needs tests when enabled for writing"
            );
            assert!(
                target["blocker"].is_null(),
                "{row_id} should not retain a blocker when enabled"
            );
        } else {
            assert!(
                target["blocker"].as_str().is_some(),
                "{row_id} needs a blocker while not enabled"
            );
        }
    }

    Ok(())
}
