use std::collections::BTreeSet;
use std::fs;
use std::path::PathBuf;
use std::time::{SystemTime, UNIX_EPOCH};

use anyhow::Result;
use hyprland_settings::cursor_input_boundary::{
    assert_cursor_input_live_execution_refused, evaluate_cursor_input_recovery_boundary,
    CursorInputBoundaryStatus, CursorInputRecoveryBoundaryPlan,
};
use hyprland_settings::write_classification::SAFE_WRITABLE_ROWS;
use serde_json::Value;

fn temp_case(name: &str) -> Result<PathBuf> {
    let nanos = SystemTime::now().duration_since(UNIX_EPOCH)?.as_nanos();
    let dir = std::env::temp_dir().join(format!(
        "hyprland-settings-cursor-input-boundary-{name}-{}-{nanos}",
        std::process::id()
    ));
    fs::create_dir_all(&dir)?;
    Ok(dir)
}

fn complete_fixture_plan() -> Result<CursorInputRecoveryBoundaryPlan> {
    let dir = temp_case("fixture")?;
    Ok(CursorInputRecoveryBoundaryPlan {
        row_id: "cursor.sync_gsettings_theme".to_owned(),
        target_config_path: dir.join("hyprland.conf"),
        backup_path: Some(dir.join("hyprland.conf.backup")),
        result_log_path: Some(dir.join("watchdog-result.json")),
        confirmation_token: Some("token".to_owned()),
        timeout_seconds: 10,
        live_execution_enabled: false,
        independent_watchdog_available: true,
        restore_command_presented_before_apply: true,
        out_of_band_recovery_instructions: true,
        confirmation_depends_on_app_ui: false,
        confirmation_depends_on_visible_cursor: false,
        confirmation_depends_on_hyprland_keybind: false,
        confirmation_depends_on_mouse_input: false,
        confirmation_depends_on_pointer_focus: false,
        confirmation_depends_on_workspace_focus: false,
        confirmation_depends_on_normal_pointer_behavior: false,
        reload_or_runtime_mutation_required: false,
    })
}

fn read_json(path: &str) -> Result<Value> {
    Ok(serde_json::from_str(&fs::read_to_string(path)?)?)
}

#[test]
fn cursor_input_boundary_represents_fixture_path_but_refuses_live_execution() -> Result<()> {
    let plan = complete_fixture_plan()?;
    let result = evaluate_cursor_input_recovery_boundary(&plan);

    assert_eq!(
        result.status,
        CursorInputBoundaryStatus::ReadyForFixtureProofOnly
    );
    assert!(result.can_represent_boundary);
    assert!(!result.live_execution_enabled);
    assert!(!result.can_execute_live_config);
    assert!(result.blocked_reasons.is_empty());
    assert_cursor_input_live_execution_refused(&result)?;

    Ok(())
}

#[test]
fn cursor_input_boundary_blocks_missing_required_fields() -> Result<()> {
    let mut plan = complete_fixture_plan()?;
    plan.backup_path = None;
    plan.confirmation_token = None;
    plan.independent_watchdog_available = false;
    plan.out_of_band_recovery_instructions = false;

    let result = evaluate_cursor_input_recovery_boundary(&plan);
    assert_eq!(result.status, CursorInputBoundaryStatus::Blocked);
    assert!(result
        .blocked_reasons
        .contains(&"missing-backup-path".to_owned()));
    assert!(result
        .blocked_reasons
        .contains(&"missing-confirmation-token".to_owned()));
    assert!(result
        .blocked_reasons
        .contains(&"missing-independent-watchdog".to_owned()));
    assert!(result
        .blocked_reasons
        .contains(&"missing-out-of-band-recovery-instructions".to_owned()));

    Ok(())
}

#[test]
fn cursor_input_boundary_rejects_input_dependent_confirmation_paths() -> Result<()> {
    let mut plan = complete_fixture_plan()?;
    plan.confirmation_depends_on_app_ui = true;
    plan.confirmation_depends_on_visible_cursor = true;
    plan.confirmation_depends_on_hyprland_keybind = true;
    plan.confirmation_depends_on_mouse_input = true;
    plan.confirmation_depends_on_pointer_focus = true;
    plan.confirmation_depends_on_workspace_focus = true;
    plan.confirmation_depends_on_normal_pointer_behavior = true;

    let result = evaluate_cursor_input_recovery_boundary(&plan);
    assert_eq!(result.status, CursorInputBoundaryStatus::Blocked);
    for reason in [
        "app-ui-only-confirmation-rejected",
        "visible-cursor-confirmation-rejected",
        "hyprland-keybind-confirmation-rejected",
        "mouse-only-confirmation-rejected",
        "pointer-focus-confirmation-rejected",
        "workspace-focus-confirmation-rejected",
        "normal-pointer-behavior-confirmation-rejected",
    ] {
        assert!(result.blocked_reasons.contains(&reason.to_owned()));
    }

    Ok(())
}

#[test]
fn cursor_input_boundary_refuses_real_config_while_live_disabled() -> Result<()> {
    let mut plan = complete_fixture_plan()?;
    plan.target_config_path = PathBuf::from("/home/kyo/.config/hypr/hyprland.conf");

    let result = evaluate_cursor_input_recovery_boundary(&plan);
    assert_eq!(result.status, CursorInputBoundaryStatus::Blocked);
    assert!(result
        .blocked_reasons
        .contains(&"real-config-path-refused-while-live-disabled".to_owned()));
    assert!(!result.can_execute_live_config);

    Ok(())
}

#[test]
fn cursor_input_reports_select_only_theme_sync_future_subset() -> Result<()> {
    let coverage = read_json("data/reports/scalar-read-write-coverage.v0.55.2.json")?;
    let boundary = read_json("data/reports/cursor-input-recovery-boundary-design.v0.55.2.json")?;
    let selection = read_json("data/reports/cursor-input-small-subset-selection.v0.55.2.json")?;
    let proof_plan =
        read_json("data/reports/cursor-input-subset-readiness-proof-plan.v0.55.2.json")?;

    assert_eq!(coverage["counts"]["writableRows"], 340);
    assert_eq!(coverage["counts"]["blockedWriteRows"], 1);
    assert_eq!(SAFE_WRITABLE_ROWS.len(), 340);
    assert_eq!(boundary["counts"]["cursorInputRowsAudited"], 22);
    assert_eq!(boundary["counts"]["rowsEnabled"], 4);
    assert_eq!(selection["counts"]["selectedRows"], 1);
    assert_eq!(selection["counts"]["rowsEnabled"], 1);
    assert_eq!(selection["counts"]["cursorInputBlockedRows"], 21);
    assert_eq!(selection["counts"]["displayRenderBlockedRows"], 23);
    assert_eq!(selection["counts"]["debugCrashBlockedRows"], 22);
    assert_eq!(selection["counts"]["writeAllowlistChanged"], true);
    assert_eq!(selection["counts"]["productionBehaviorChanged"], true);
    assert_eq!(proof_plan["counts"]["selectedRows"], 1);
    assert_eq!(proof_plan["counts"]["plannedRows"], 1);
    assert_eq!(proof_plan["counts"]["rowsEnabled"], 1);

    let selected = selection["selectedRows"].as_array().unwrap();
    assert_eq!(selected.len(), 1);
    assert_eq!(
        selected[0]["rowId"].as_str(),
        Some("cursor.sync_gsettings_theme")
    );
    assert_eq!(
        selected[0]["cursorInputRiskClass"].as_str(),
        Some("cursor-theme-sync-policy")
    );

    let disallowed = [
        "cursor-visibility-critical",
        "hardware-cursor-critical",
        "cursor-warping-critical",
        "cursor-zoom-critical",
        "cursor-monitor-targeting-critical",
    ]
    .into_iter()
    .collect::<BTreeSet<_>>();
    assert!(!disallowed.contains(
        selected[0]["cursorInputRiskClass"]
            .as_str()
            .expect("selected row should have a risk class")
    ));

    Ok(())
}

#[test]
fn cursor_input_selection_does_not_enable_any_blocked_bucket() -> Result<()> {
    let coverage = read_json("data/reports/scalar-read-write-coverage.v0.55.2.json")?;
    let writable_ids = coverage["rows"]
        .as_array()
        .unwrap()
        .iter()
        .filter(|row| row["writeStatus"].as_str() == Some("writable"))
        .map(|row| row["rowId"].as_str().unwrap())
        .collect::<BTreeSet<_>>();

    assert!(writable_ids.contains("cursor.sync_gsettings_theme"));
    assert!(writable_ids.contains("cursor.invisible"));
    assert!(writable_ids.contains("cursor.no_hardware_cursors"));
    assert!(writable_ids.contains("cursor.no_warps"));
    assert!(writable_ids.contains("cursor.zoom_factor"));
    assert!(writable_ids.contains("xwayland.enabled"));
    assert!(writable_ids.contains("debug.disable_logs"));
    assert!(!writable_ids.contains("cursor.default_monitor"));

    Ok(())
}

#[test]
fn cursor_theme_sync_smoke_subset_reports_complete_proof() -> Result<()> {
    let preflight =
        read_json("data/reports/cursor-theme-sync-policy-smoke-subset-proof.v0.55.2.json")?;
    let watchdog = read_json("data/reports/cursor-theme-sync-policy-watchdog-proof.v0.55.2.json")?;
    let enablements = read_json("data/reports/cursor-input-first-subset-enablements.v0.55.2.json")?;

    assert_eq!(preflight["counts"]["rows"], 1);
    assert_eq!(preflight["counts"]["validatorPassedRows"], 1);
    assert_eq!(preflight["counts"]["invalidRejectionPassedRows"], 1);
    assert_eq!(preflight["counts"]["hyprlandVerifyConfigPassedRows"], 1);
    assert_eq!(preflight["counts"]["safeToEnableByPreflightRows"], 1);
    assert_eq!(watchdog["counts"]["planPersistedBeforeMutationRows"], 1);
    assert_eq!(watchdog["counts"]["backupExistsBeforeMutationRows"], 1);
    assert_eq!(watchdog["counts"]["separateProcessConfirmPassedRows"], 1);
    assert_eq!(
        watchdog["counts"]["separateProcessTimeoutRestorePassedRows"],
        1
    );
    assert_eq!(watchdog["counts"]["wrongTokenFailedRows"], 1);
    assert_eq!(watchdog["counts"]["realConfigTargetRefusedInDryRunRows"], 1);
    assert_eq!(watchdog["counts"]["recoveryIndependencePassedRows"], 1);
    assert_eq!(watchdog["counts"]["safeToEnableByWatchdogGateRows"], 1);
    assert_eq!(enablements["counts"]["attemptedRows"], 1);
    assert_eq!(enablements["counts"]["enabledRows"], 1);
    assert_eq!(enablements["counts"]["finalWritableRows"], 275);
    assert_eq!(enablements["counts"]["finalBlockedRows"], 66);
    assert_eq!(enablements["counts"]["cursorInputRowsStillBlocked"], 21);
    assert_eq!(enablements["counts"]["displayRenderRowsStillBlocked"], 23);
    assert_eq!(enablements["counts"]["debugCrashRowsStillBlocked"], 22);

    let rows = enablements["rows"].as_array().expect("enablement rows");
    assert_eq!(rows.len(), 1);
    assert_eq!(
        rows[0]["rowId"].as_str(),
        Some("cursor.sync_gsettings_theme")
    );
    assert_eq!(rows[0]["enabled"].as_bool(), Some(true));
    assert_eq!(
        watchdog["rows"][0]["recoveryIndependenceProof"]["requiresMouseInput"].as_bool(),
        Some(false)
    );
    assert_eq!(
        watchdog["rows"][0]["recoveryIndependenceProof"]["requiresVisibleCursor"].as_bool(),
        Some(false)
    );
    assert_eq!(
        watchdog["rows"][0]["recoveryIndependenceProof"]["requiresHyprlandKeybind"].as_bool(),
        Some(false)
    );
    assert_eq!(watchdog["safety"]["realConfigModified"], false);
    assert_eq!(watchdog["safety"]["activeRuntimeModified"], false);
    assert_eq!(watchdog["safety"]["reloadRun"], false);
    assert_eq!(watchdog["safety"]["evalRun"], false);
    assert_eq!(watchdog["safety"]["luaExecuted"], false);

    Ok(())
}

#[test]
fn next_cursor_input_subset_selection_keeps_remaining_rows_blocked() -> Result<()> {
    let coverage = read_json("data/reports/scalar-read-write-coverage.v0.55.2.json")?;
    let smoke_review = read_json("data/reports/cursor-input-smoke-subset-review.v0.55.2.json")?;
    let selection = read_json("data/reports/next-cursor-input-subset-selection.v0.55.2.json")?;
    let proof_plan =
        read_json("data/reports/next-cursor-input-subset-readiness-proof-plan.v0.55.2.json")?;

    assert_eq!(coverage["counts"]["writableRows"], 340);
    assert_eq!(coverage["counts"]["blockedWriteRows"], 1);
    assert_eq!(SAFE_WRITABLE_ROWS.len(), 340);

    assert_eq!(smoke_review["counts"]["reviewedRows"], 1);
    assert_eq!(smoke_review["counts"]["enabledRows"], 1);
    assert_eq!(smoke_review["counts"]["issuesFound"], 0);
    assert_eq!(
        smoke_review["counts"]["writeAllowlistChangedInThisSelectionSprint"],
        false
    );
    assert_eq!(
        smoke_review["rows"][0]["rowId"].as_str(),
        Some("cursor.sync_gsettings_theme")
    );
    assert_eq!(
        smoke_review["rows"][0]["recoveryIndependenceResult"].as_bool(),
        Some(true)
    );

    assert_eq!(selection["counts"]["cursorInputRowsReviewed"], 21);
    assert_eq!(selection["counts"]["selectedRows"], 0);
    assert_eq!(selection["counts"]["excludedRows"], 21);
    assert_eq!(selection["counts"]["rowsEnabled"], 0);
    assert_eq!(selection["counts"]["finalWritableRows"], 275);
    assert_eq!(selection["counts"]["finalBlockedRows"], 66);
    assert_eq!(selection["counts"]["cursorInputBlockedRows"], 21);
    assert_eq!(selection["counts"]["displayRenderBlockedRows"], 23);
    assert_eq!(selection["counts"]["debugCrashBlockedRows"], 22);
    assert_eq!(selection["counts"]["writeAllowlistChanged"], false);
    assert_eq!(selection["counts"]["productionBehaviorChanged"], false);
    assert_eq!(selection["selectedSubsetName"], Value::Null);
    assert_eq!(selection["selectionDecision"].as_str(), Some("select-none"));
    assert!(selection["selectedRows"].as_array().unwrap().is_empty());
    assert_eq!(selection["rows"].as_array().unwrap().len(), 21);

    let excluded_classes = [
        "cursor-visibility-critical",
        "hardware-cursor-critical",
        "cursor-warping-critical",
        "cursor-zoom-critical",
        "cursor-monitor-targeting-critical",
    ]
    .into_iter()
    .collect::<BTreeSet<_>>();

    for row in selection["rows"].as_array().unwrap() {
        let risk_class = row["cursorInputRiskClass"]
            .as_str()
            .expect("remaining cursor row should have a risk class");
        assert!(excluded_classes.contains(risk_class));
        assert_eq!(row["excludedByRiskClass"], true);
        assert_eq!(row["candidateForNextSubset"], false);
        assert_eq!(row["selectedForNextSubset"], false);
    }

    assert_eq!(proof_plan["counts"]["selectedRows"], 0);
    assert_eq!(proof_plan["counts"]["plannedRows"], 0);
    assert_eq!(proof_plan["counts"]["emptyPlan"], true);
    assert_eq!(proof_plan["counts"]["rowsEnabled"], 0);
    assert!(proof_plan["proofPlans"].as_array().unwrap().is_empty());
    assert_eq!(proof_plan["safety"]["realConfigModified"], false);
    assert_eq!(proof_plan["safety"]["activeRuntimeModified"], false);
    assert_eq!(proof_plan["safety"]["reloadRun"], false);
    assert_eq!(proof_plan["safety"]["evalRun"], false);
    assert_eq!(proof_plan["safety"]["luaExecuted"], false);

    Ok(())
}
