use anyhow::Result;
use serde_json::Value;

fn target_report() -> Result<Value> {
    Ok(serde_json::from_str(include_str!(
        "../data/reports/scalar-write-expansion-targets.v0.55.2.json"
    ))?)
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
fn scalar_write_expansion_targets_have_decisions_and_blockers() -> Result<()> {
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
        assert!(
            target["blocker"].as_str().is_some(),
            "{row_id} needs a blocker while not enabled"
        );
        assert_eq!(
            target["enabled"].as_bool(),
            Some(false),
            "{row_id} should not be enabled during target audit"
        );
        assert!(
            target["tests"].as_array().is_some(),
            "{row_id} needs a tests array"
        );
    }

    Ok(())
}
