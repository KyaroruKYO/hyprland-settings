use anyhow::Result;
use hyprland_settings::write_classification::{
    high_risk_write_policy, is_safe_writable_setting, SAFE_WRITABLE_ROWS,
};
use serde_json::Value;
use std::collections::BTreeSet;

fn read_json(path: &str) -> Result<Value> {
    Ok(serde_json::from_str(&std::fs::read_to_string(path)?)?)
}

#[test]
fn cursor_visibility_conditional_enablement_report_enables_only_touch_and_tablet() -> Result<()> {
    let enablements =
        read_json("data/reports/cursor-visibility-conditional-policy-enablements.v0.55.2.json")?;
    let proof =
        read_json("data/reports/cursor-visibility-conditional-policy-enable-proof.v0.55.2.json")?;
    let coverage = read_json("data/reports/scalar-read-write-coverage.v0.55.2.json")?;
    let pipeline = read_json("data/reports/all-341-unified-pipeline.v0.55.2.json")?;

    assert_eq!(enablements["counts"]["attemptedRows"], 2);
    assert_eq!(enablements["counts"]["enabledRows"], 2);
    assert_eq!(enablements["counts"]["notEnabledRows"], 0);
    assert_eq!(proof["counts"]["rows"], 2);
    assert_eq!(proof["counts"]["rowsEnabled"], 2);
    assert_eq!(proof["counts"]["readyForEnablementRows"], 2);
    assert_eq!(coverage["counts"]["writableRows"], 341);
    assert_eq!(coverage["counts"]["blockedWriteRows"], 0);
    assert_eq!(pipeline["counts"]["totalRows"], 341);
    assert_eq!(pipeline["counts"]["metadataGapRows"], 0);

    let enabled_ids = enablements["rows"]
        .as_array()
        .unwrap()
        .iter()
        .map(|row| row["rowId"].as_str().unwrap())
        .collect::<BTreeSet<_>>();
    assert_eq!(
        enabled_ids,
        BTreeSet::from(["cursor.hide_on_touch", "cursor.hide_on_tablet"])
    );

    let safe_ids = SAFE_WRITABLE_ROWS
        .iter()
        .map(|row| row.row_id)
        .collect::<BTreeSet<_>>();
    assert!(safe_ids.contains("cursor.hide_on_touch"));
    assert!(safe_ids.contains("cursor.hide_on_tablet"));

    let coverage_by_id = coverage["rows"]
        .as_array()
        .unwrap()
        .iter()
        .map(|row| (row["rowId"].as_str().unwrap(), row))
        .collect::<std::collections::BTreeMap<_, _>>();
    let enablement_by_id = enablements["rows"]
        .as_array()
        .unwrap()
        .iter()
        .map(|row| (row["rowId"].as_str().unwrap(), row))
        .collect::<std::collections::BTreeMap<_, _>>();

    for row_id in ["cursor.hide_on_touch", "cursor.hide_on_tablet"] {
        let row = coverage_by_id[row_id];
        let enablement = enablement_by_id[row_id];
        assert_eq!(row["writeStatus"].as_str(), Some("writable"));
        assert_eq!(row["safeWriteSupported"].as_bool(), Some(true));
        assert_eq!(row["validatorSupported"].as_bool(), Some(true));
        assert_eq!(enablement["valueType"].as_str(), Some("bool"));
        assert_eq!(
            enablement["acceptedValues"],
            serde_json::json!(["true", "false"])
        );
        assert!(is_safe_writable_setting(row_id));

        let policy = high_risk_write_policy(row_id).expect("policy should exist");
        assert_eq!(
            policy.recovery_bucket,
            "cursor-input-recovery:cursor-visibility-conditional-touch-tablet-subset"
        );
        assert!(policy.approval_gate.contains("dead-man"));
        assert!(policy
            .watchdog_requirement
            .contains("plan must be persisted"));
        assert!(policy
            .watchdog_requirement
            .contains("backup must exist before mutation"));
        assert!(policy
            .watchdog_requirement
            .contains("timeout restores the backup"));
        assert!(policy.watchdog_requirement.contains("visible cursor"));
        assert!(policy.watchdog_requirement.contains("mouse input"));
        assert!(policy.watchdog_requirement.contains("Hyprland keybinds"));
        assert!(policy.watchdog_requirement.contains("pointer focus"));
        assert!(policy.watchdog_requirement.contains("workspace focus"));
        assert!(policy.review_warning.contains("Cursor may disappear"));
    }

    Ok(())
}

#[test]
fn cursor_visibility_conditional_enablement_keeps_other_high_risk_rows_blocked() -> Result<()> {
    let coverage = read_json("data/reports/scalar-read-write-coverage.v0.55.2.json")?;
    let enablements =
        read_json("data/reports/cursor-visibility-conditional-policy-enablements.v0.55.2.json")?;
    let coverage_by_id = coverage["rows"]
        .as_array()
        .unwrap()
        .iter()
        .map(|row| (row["rowId"].as_str().unwrap(), row))
        .collect::<std::collections::BTreeMap<_, _>>();

    for row_id in [
        "cursor.invisible",
        "cursor.inactive_timeout",
        "cursor.no_hardware_cursors",
        "cursor.no_break_fs_vrr",
        "cursor.min_refresh_rate",
        "cursor.hotspot_padding",
        "cursor.no_warps",
        "cursor.persistent_warps",
        "cursor.warp_on_change_workspace",
        "cursor.warp_on_toggle_special",
        "cursor.default_monitor",
        "cursor.zoom_factor",
        "cursor.zoom_rigid",
        "cursor.zoom_disable_aa",
        "cursor.zoom_detached_camera",
    ] {
        let row = coverage_by_id[row_id];
        assert_eq!(row["writeStatus"].as_str(), Some("writable"), "{row_id}");
        assert_eq!(row["safeWriteSupported"].as_bool(), Some(true), "{row_id}");
        assert!(is_safe_writable_setting(row_id), "{row_id}");
    }

    assert_eq!(enablements["counts"]["cursorInputRowsStillBlocked"], 19);
    assert_eq!(enablements["counts"]["displayRenderRowsStillBlocked"], 23);
    assert_eq!(enablements["counts"]["debugCrashRowsStillBlocked"], 22);

    Ok(())
}

#[test]
fn cursor_visibility_conditional_enablement_preserves_safety_proof_flags() -> Result<()> {
    let enablements =
        read_json("data/reports/cursor-visibility-conditional-policy-enablements.v0.55.2.json")?;
    let proof =
        read_json("data/reports/cursor-visibility-conditional-policy-enable-proof.v0.55.2.json")?;

    for report in [&enablements, &proof] {
        assert_eq!(report["counts"]["recoveryGateWeakenedRows"], 0);
        assert_eq!(report["counts"]["realConfigModified"], false);
        assert_eq!(report["counts"]["activeRuntimeModified"], false);
        assert_eq!(report["counts"]["reloadEvalLuaUsed"], false);
        assert_eq!(report["counts"]["finalWritableRows"], 277);
        assert_eq!(report["counts"]["finalBlockedRows"], 64);
    }

    for row in proof["rows"].as_array().unwrap() {
        assert_eq!(row["validatorProofExistsAndPassed"], true);
        assert_eq!(row["invalidRejectionProofExistsAndPassed"], true);
        assert_eq!(row["fixtureTempConfigProofExistsAndPassed"], true);
        assert_eq!(row["watchdogArmBeforeMutationProofExistsAndPassed"], true);
        assert_eq!(row["backupBeforeMutationProofExistsAndPassed"], true);
        assert_eq!(row["separateProcessConfirmProofExistsAndPassed"], true);
        assert_eq!(row["timeoutRestoreProofExistsAndPassed"], true);
        assert_eq!(row["wrongTokenFailureProofExistsAndPassed"], true);
        assert_eq!(row["dryRunRealConfigRefusalProofExistsAndPassed"], true);
        assert_eq!(row["recoveryIndependenceProofExistsAndPassed"], true);
        assert_eq!(row["recoveryGateWeakened"], false);
        assert_eq!(row["enabled"], true);
    }

    Ok(())
}
