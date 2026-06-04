use std::collections::BTreeMap;

use anyhow::Result;
use serde_json::Value;

fn parser_family_report() -> Result<Value> {
    Ok(serde_json::from_str(include_str!(
        "../data/reports/parser-family-write-targets.v0.55.2.json"
    ))?)
}

#[test]
fn parser_family_report_has_expected_remaining_targets() -> Result<()> {
    let report = parser_family_report()?;
    let targets = report["targets"]
        .as_array()
        .expect("parser-family targets should be an array");

    assert_eq!(report["counts"]["totalRemainingParserNeededRows"], 27);
    assert_eq!(report["counts"]["enabledRows"], 27);
    assert_eq!(report["counts"]["blockedRows"], 0);
    assert_eq!(targets.len(), 27);

    Ok(())
}

#[test]
fn parser_family_report_groups_all_remaining_parser_families() -> Result<()> {
    let report = parser_family_report()?;
    let targets = report["targets"]
        .as_array()
        .expect("parser-family targets should be an array");
    let mut families = BTreeMap::new();

    for target in targets {
        let family = target["parserFamily"]
            .as_str()
            .expect("parserFamily should be a string");
        *families.entry(family.to_string()).or_insert(0usize) += 1;
        if target["writeStatus"].as_str() == Some("writable") {
            assert!(
                target["blocker"].is_null(),
                "{} should clear blocker when enabled",
                target["rowId"]
            );
            assert!(
                !target["testNames"]
                    .as_array()
                    .expect("testNames should be an array")
                    .is_empty(),
                "{} should list tests when enabled",
                target["rowId"]
            );
        } else {
            assert!(
                target["blocker"].as_str().is_some(),
                "{} should keep a blocker until enabled",
                target["rowId"]
            );
        }
    }

    assert_eq!(families.get("gradient/color-list"), Some(&12));
    assert_eq!(families.get("vector/tuple"), Some(&6));
    assert_eq!(families.get("enum/custom string"), Some(&4));
    assert_eq!(families.get("path/string requiring sanitization"), Some(&2));
    assert_eq!(
        families.get("regex/string requiring sanitization"),
        Some(&2)
    );
    assert_eq!(families.get("custom numeric list parser"), Some(&1));

    Ok(())
}
