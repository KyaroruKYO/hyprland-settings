use anyhow::Result;
use hyprland_settings::current_config::CurrentValueProjection;
use hyprland_settings::pending_change::{stage_pending_change, PendingChangeValidation};
use hyprland_settings::write_classification::{
    source_backed_numeric_bounds, SourceBackedNumericType, SAFE_WRITABLE_ROWS,
};
use serde_json::Value;

fn read_json(path: &str) -> Result<Value> {
    Ok(serde_json::from_str(&std::fs::read_to_string(path)?)?)
}

fn assert_repair_safety(report: &Value) {
    assert_eq!(report["counts"]["writeAllowlistChanged"], false);
    assert_eq!(report["counts"]["safeWritableRowsChanged"], false);
    assert_eq!(report["counts"]["rowsEnabledThisSprint"], 0);
    assert_eq!(report["counts"]["recoveryGatesChanged"], false);
    assert_eq!(report["counts"]["realConfigModified"], false);
    assert_eq!(report["counts"]["activeRuntimeModified"], false);
    assert_eq!(report["counts"]["reloadEvalLuaUsed"], false);
}

fn assert_valid(row_id: &str, value: &str) {
    let change = stage_pending_change(row_id, &CurrentValueProjection::not_configured(), value);
    assert_eq!(
        change.validation,
        PendingChangeValidation::Valid,
        "{row_id} should accept {value:?}"
    );
}

fn assert_invalid(row_id: &str, value: &str) {
    let change = stage_pending_change(row_id, &CurrentValueProjection::not_configured(), value);
    assert!(
        matches!(change.validation, PendingChangeValidation::Invalid { .. }),
        "{row_id} should reject {value:?}, got {:?}",
        change.validation
    );
}

fn valid_value_for_bounds(value_type: SourceBackedNumericType, min: f64, max: f64) -> String {
    match value_type {
        SourceBackedNumericType::Integer => {
            let min = min as i64;
            let max = max as i64;
            let preferred = if min <= 10 && 10 <= max { 10 } else { min };
            preferred.to_string()
        }
        SourceBackedNumericType::Float => {
            let preferred = if min <= 0.5 && 0.5 <= max { 0.5 } else { min };
            preferred.to_string()
        }
    }
}

fn below_value(value_type: SourceBackedNumericType, min: f64) -> String {
    match value_type {
        SourceBackedNumericType::Integer => ((min as i64) - 1).to_string(),
        SourceBackedNumericType::Float => (min - 0.1).to_string(),
    }
}

fn above_value(value_type: SourceBackedNumericType, max: f64) -> String {
    match value_type {
        SourceBackedNumericType::Integer => ((max as i64) + 1).to_string(),
        SourceBackedNumericType::Float => (max + 0.1).to_string(),
    }
}

#[test]
fn source_backed_repair_reports_exist_and_preserve_counts() -> Result<()> {
    let coverage = read_json("data/reports/scalar-read-write-coverage.v0.55.2.json")?;
    let summary = read_json("data/reports/source-backed-writable-validator-repair.v0.55.2.json")?;
    let boolean = read_json("data/reports/source-backed-bool-policy-repair.v0.55.2.json")?;
    let numeric = read_json("data/reports/source-backed-numeric-bounds-repair.v0.55.2.json")?;
    let parser = read_json("data/reports/source-backed-parser-only-repair.v0.55.2.json")?;
    let complex = read_json("data/reports/source-backed-complex-grammar-repair.v0.55.2.json")?;
    let deferred = read_json("data/reports/source-backed-validator-deferred-items.v0.55.2.json")?;

    assert_eq!(SAFE_WRITABLE_ROWS.len(), 278);
    assert_eq!(coverage["counts"]["writableRows"], 278);
    assert_eq!(coverage["counts"]["blockedWriteRows"], 63);

    for report in [&summary, &boolean, &numeric, &parser, &complex, &deferred] {
        assert_repair_safety(report);
    }

    assert_eq!(boolean["counts"]["booleanRowsReviewed"], 130);
    assert_eq!(boolean["counts"]["validatorChangedRows"], 0);
    assert_eq!(numeric["counts"]["sourceBackedBoundsRows"], 72);
    assert_eq!(numeric["counts"]["cssGapDeferredRows"], 2);
    assert_eq!(parser["counts"]["rowsReclassifiedSourceConsumerBacked"], 2);
    assert_eq!(complex["counts"]["colorRowsRepaired"], 10);
    assert_eq!(complex["counts"]["gradientRowsRepaired"], 12);
    assert_eq!(complex["counts"]["vectorRowsRepaired"], 6);
    assert_eq!(deferred["counts"]["deferredRows"], 12);

    assert_eq!(summary["beforeGapCounts"]["appValidatorTooBroadRows"], 204);
    assert_eq!(summary["beforeGapCounts"]["parserOnlyRows"], 2);
    assert_eq!(summary["beforeGapCounts"]["sourceResearchNeededRows"], 38);
    assert_eq!(summary["afterGapCounts"]["appValidatorTooBroadRows"], 0);
    assert_eq!(summary["afterGapCounts"]["parserOnlyRows"], 0);
    assert_eq!(summary["afterGapCounts"]["sourceResearchNeededRows"], 10);
    assert_eq!(summary["counts"]["validatorChangedRows"], 101);
    assert_eq!(summary["counts"]["validatorUnchangedRows"], 177);

    Ok(())
}

#[test]
fn bool_policy_keeps_exact_aliases_and_true_false_ui() -> Result<()> {
    let report = read_json("data/reports/source-backed-bool-policy-repair.v0.55.2.json")?;
    assert_eq!(
        report["policy"]["uiOfferedValues"],
        serde_json::json!(["true", "false"])
    );
    assert_eq!(report["counts"]["arbitraryIntegerAliasesAcceptedRows"], 0);
    assert_eq!(report["counts"]["prefixAliasesAcceptedRows"], 0);

    for value in ["true", "false", "1", "0", "yes", "no", "on", "off"] {
        assert_valid("windows.snap.enabled", value);
    }
    for value in ["2", "-1", "truthy", "yesplease", "offsite", "maybe"] {
        assert_invalid("windows.snap.enabled", value);
    }

    Ok(())
}

#[test]
fn numeric_bounds_are_source_backed_and_enforced() -> Result<()> {
    let numeric = read_json("data/reports/source-backed-numeric-bounds-repair.v0.55.2.json")?;
    let repaired = numeric["rows"]
        .as_array()
        .unwrap()
        .iter()
        .filter(|row| row["repairDecision"].as_str() == Some("source-bounds-enforced"))
        .collect::<Vec<_>>();
    assert_eq!(repaired.len(), 72);

    for row in repaired {
        let row_id = row["rowId"].as_str().unwrap();
        let bounds = source_backed_numeric_bounds(row_id).expect("bounds should be registered");
        assert_valid(
            row_id,
            &valid_value_for_bounds(bounds.value_type, bounds.min, bounds.max),
        );
        assert_invalid(row_id, &below_value(bounds.value_type, bounds.min));
        assert_invalid(row_id, &above_value(bounds.value_type, bounds.max));
        if bounds.value_type == SourceBackedNumericType::Integer {
            assert_invalid(row_id, "1.5");
        }
    }

    assert!(source_backed_numeric_bounds("appearance.gaps_in").is_none());
    assert!(source_backed_numeric_bounds("appearance.gaps_out").is_none());
    assert_valid("appearance.gaps_in", "8");
    assert_valid("appearance.gaps_out", "12");
    assert_valid("appearance.gaps_in", "5 10");
    assert_valid("appearance.gaps_out", "5 10 15 20");
    assert_invalid("appearance.gaps_in", "5 10 15 20 25");
    assert_invalid("appearance.gaps_out", "-5");

    Ok(())
}

#[test]
fn parser_only_semantics_are_repaired_without_enablement_changes() {
    for value in ["left", "right", "top", "bottom"] {
        assert_valid("master.center_master_fallback", value);
    }
    for value in ["", "center", "left right", "auto"] {
        assert_invalid("master.center_master_fallback", value);
    }

    for value in ["1.0", "0.333, 0.5, 0.667, 1.0", "0.05,1.0"] {
        assert_valid("scrolling.explicit_column_widths", value);
    }
    for value in ["", "0.01", "1.01", "0.5, bad", "0.5 0.75", "0.5,"] {
        assert_invalid("scrolling.explicit_column_widths", value);
    }
}

#[test]
fn complex_grammar_repairs_match_official_source_policy() {
    for value in [
        "#f0a",
        "#ff00aa",
        "#ff00aaff",
        "rgb(ff00aa)",
        "rgb(255,0,170)",
        "rgba(ff00aaff)",
        "rgba(255,0,170,0.5)",
        "0xff00aaff",
        "16711935",
    ] {
        assert_valid("misc.background_color", value);
    }

    assert_valid("general.col.active_border", "#fff #000 45deg");
    assert_valid(
        "general.col.active_border",
        "rgba(255,0,170,0.5) rgb(0,0,0)",
    );
    assert_invalid(
        "general.col.active_border",
        "rgba(ffffffff) rgba(000000ff) rgba(111111ff) rgba(222222ff) rgba(333333ff) rgba(444444ff) rgba(555555ff) rgba(666666ff) rgba(777777ff) rgba(888888ff) rgba(999999ff)",
    );
    assert_invalid("general.col.active_border", "rgba(ffffffff) 45.5deg");
    assert_invalid(
        "general.col.active_border",
        "rgba(ffffffff) 45deg rgba(000000ff)",
    );

    assert_valid("decoration.shadow.offset", "10 20");
    assert_valid("layout.single_window_aspect_ratio", "-1.5 2.25");
    assert_invalid("decoration.shadow.offset", "10,20");
}

#[test]
fn string_path_regex_and_numeric_list_rows_remain_deferred() -> Result<()> {
    let deferred = read_json("data/reports/source-backed-validator-deferred-items.v0.55.2.json")?;
    let rows = deferred["rows"]
        .as_array()
        .unwrap()
        .iter()
        .map(|row| row["rowId"].as_str().unwrap())
        .collect::<std::collections::BTreeSet<_>>();

    for row_id in [
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
    ] {
        assert!(rows.contains(row_id), "{row_id}");
    }

    Ok(())
}
