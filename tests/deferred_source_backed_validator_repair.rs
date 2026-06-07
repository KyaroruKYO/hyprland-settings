use anyhow::Result;
use hyprland_settings::current_config::CurrentValueProjection;
use hyprland_settings::pending_change::{stage_pending_change, PendingChangeValidation};
use hyprland_settings::write_classification::{
    safe_writable_value_kind, ScalarWriteValueKind, SAFE_WRITABLE_ROWS,
};
use serde_json::Value;

fn read_json(path: &str) -> Result<Value> {
    Ok(serde_json::from_str(&std::fs::read_to_string(path)?)?)
}

fn assert_valid(row_id: &str, value: &str) {
    let change = stage_pending_change(row_id, &CurrentValueProjection::not_configured(), value);
    assert_eq!(
        change.validation,
        PendingChangeValidation::Valid,
        "{row_id} should accept {value:?}"
    );
    assert!(change.can_be_applied(), "{row_id} should be applicable");
}

fn assert_invalid(row_id: &str, value: &str) {
    let change = stage_pending_change(row_id, &CurrentValueProjection::not_configured(), value);
    assert!(
        matches!(change.validation, PendingChangeValidation::Invalid { .. }),
        "{row_id} should reject {value:?}, got {:?}",
        change.validation
    );
    assert!(!change.can_be_applied(), "{row_id} should not apply");
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

#[test]
fn deferred_source_backed_repair_reports_exist_and_preserve_counts() -> Result<()> {
    let coverage = read_json("data/reports/scalar-read-write-coverage.v0.55.2.json")?;
    let main = read_json("data/reports/deferred-source-backed-validator-repair.v0.55.2.json")?;
    let cssgap = read_json("data/reports/deferred-cssgap-validator-repair.v0.55.2.json")?;
    let accel = read_json("data/reports/deferred-accel-profile-validator-repair.v0.55.2.json")?;
    let scroll = read_json("data/reports/deferred-scroll-points-validator-repair.v0.55.2.json")?;
    let metadata =
        read_json("data/reports/deferred-string-path-font-metadata-repair.v0.55.2.json")?;
    let regex = read_json("data/reports/deferred-regex-validator-repair-decision.v0.55.2.json")?;
    let shader =
        read_json("data/reports/deferred-screen-shader-high-risk-review-candidate.v0.55.2.json")?;
    let remaining = read_json("data/reports/deferred-validator-remaining-items.v0.55.2.json")?;

    assert_eq!(SAFE_WRITABLE_ROWS.len(), 278);
    assert_eq!(coverage["counts"]["writableRows"], 278);
    assert_eq!(coverage["counts"]["blockedWriteRows"], 63);

    for report in [
        &main, &cssgap, &accel, &scroll, &metadata, &regex, &shader, &remaining,
    ] {
        assert_repair_safety(report);
    }

    assert_eq!(main["counts"]["validatorChangedRows"], 5);
    assert_eq!(main["counts"]["metadataOnlyRows"], 6);
    assert_eq!(main["counts"]["remainingDeferredRows"], 4);
    assert_eq!(regex["counts"]["re2ImplementationAvailable"], false);
    assert_eq!(shader["counts"]["highRiskReviewCandidateRows"], 1);
    assert_eq!(remaining["counts"]["remainingDeferredRows"], 4);

    Ok(())
}

#[test]
fn cssgap_validator_accepts_source_backed_component_forms() {
    for row_id in ["appearance.gaps_in", "appearance.gaps_out"] {
        assert_eq!(
            safe_writable_value_kind(row_id),
            Some(ScalarWriteValueKind::CssGap)
        );
        for value in ["0", "8", "8 12", "8 12 16", "8 12 16 20"] {
            assert_valid(row_id, value);
        }
        for value in [
            "",
            "-1",
            "8.5",
            "8 12 16 20 24",
            "8,12",
            "8 # comment",
            "8\n12",
            "`cmd`",
            "$(cmd)",
        ] {
            assert_invalid(row_id, value);
        }
    }
}

#[test]
fn accel_profile_validator_accepts_only_source_backed_profiles() {
    assert_eq!(
        safe_writable_value_kind("input.accel_profile"),
        Some(ScalarWriteValueKind::AccelProfile)
    );

    for value in ["", "adaptive", "flat", "custom 0.2 0.0 0.5 1"] {
        assert_valid("input.accel_profile", value);
    }
    for value in [
        "linear",
        "custom",
        "custom 0.2",
        "custom 0.2 nope",
        "custom NaN 0.5",
        "custom inf 0.5",
        "custom 0.2,0.5",
        "flat # comment",
        "flat\nadaptive",
    ] {
        assert_invalid("input.accel_profile", value);
    }
}

#[test]
fn scroll_points_validator_rejects_non_source_backed_shapes() {
    assert_eq!(
        safe_writable_value_kind("input.scroll_points"),
        Some(ScalarWriteValueKind::NumericList)
    );

    for value in ["0.2 0.0", "0.2 0.0 0.5 1 1.2"] {
        assert_valid("input.scroll_points", value);
    }
    for value in [
        "", "0.2", "0 1", "-0.2 1", "0.2, 0.5", "0.2 nope", "NaN 1", "inf 1", "0.2\n1", "0.2 # 1",
    ] {
        assert_invalid("input.scroll_points", value);
    }
}

#[test]
fn metadata_only_and_deferred_decisions_are_explicit() -> Result<()> {
    let metadata =
        read_json("data/reports/deferred-string-path-font-metadata-repair.v0.55.2.json")?;
    let regex = read_json("data/reports/deferred-regex-validator-repair-decision.v0.55.2.json")?;
    let shader =
        read_json("data/reports/deferred-screen-shader-high-risk-review-candidate.v0.55.2.json")?;

    let metadata_rows = metadata["rows"]
        .as_array()
        .unwrap()
        .iter()
        .map(|row| row["rowId"].as_str().unwrap())
        .collect::<std::collections::BTreeSet<_>>();
    for row_id in [
        "general.locale",
        "input.kb_file",
        "group.groupbar.font_family",
        "misc.font_family",
        "misc.splash_font_family",
    ] {
        assert!(metadata_rows.contains(row_id), "{row_id}");
    }

    for row in regex["rows"].as_array().unwrap() {
        assert_eq!(row["validatorChanged"], false);
        assert_eq!(row["remainingDeferred"], true);
        assert!(row["metadata"].as_str().unwrap().contains("RE2"));
    }

    assert_eq!(shader["rows"][0]["rowId"], "decoration.screen_shader");
    assert_eq!(shader["rows"][0]["validatorChanged"], false);
    assert_eq!(shader["rows"][0]["highRiskReviewCandidate"], true);
    assert!(shader["decision"]
        .as_str()
        .unwrap()
        .contains("display/render high-risk"));

    Ok(())
}
