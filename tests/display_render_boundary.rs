use std::fs;
use std::path::PathBuf;
use std::time::{SystemTime, UNIX_EPOCH};

use anyhow::Result;
use hyprland_settings::display_render_boundary::{
    assert_live_execution_refused, evaluate_display_render_live_boundary,
    DisplayRenderBoundaryStatus, DisplayRenderLiveBoundaryPlan,
};
use serde_json::Value;

fn temp_case(name: &str) -> Result<PathBuf> {
    let nanos = SystemTime::now().duration_since(UNIX_EPOCH)?.as_nanos();
    let dir = std::env::temp_dir().join(format!(
        "hyprland-settings-display-render-boundary-{name}-{}-{nanos}",
        std::process::id()
    ));
    fs::create_dir_all(&dir)?;
    Ok(dir)
}

fn complete_fixture_plan() -> Result<DisplayRenderLiveBoundaryPlan> {
    let dir = temp_case("fixture")?;
    Ok(DisplayRenderLiveBoundaryPlan {
        row_id: "xwayland.use_nearest_neighbor".to_owned(),
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
        confirmation_depends_on_visible_display: false,
        confirmation_depends_on_hyprland_keybind: false,
        confirmation_depends_on_mouse_input: false,
        reload_or_runtime_mutation_required: false,
    })
}

fn read_json(path: &str) -> Result<Value> {
    Ok(serde_json::from_str(&fs::read_to_string(path)?)?)
}

#[test]
fn display_render_live_boundary_represents_fixture_path_but_refuses_execution() -> Result<()> {
    let plan = complete_fixture_plan()?;
    let result = evaluate_display_render_live_boundary(&plan);

    assert_eq!(
        result.status,
        DisplayRenderBoundaryStatus::ReadyForFixtureProofOnly
    );
    assert!(result.can_represent_boundary);
    assert!(!result.live_execution_enabled);
    assert!(!result.can_execute_live_config);
    assert!(result.blocked_reasons.is_empty());
    assert_live_execution_refused(&result)?;

    Ok(())
}

#[test]
fn display_render_live_boundary_blocks_missing_required_fields() -> Result<()> {
    let mut plan = complete_fixture_plan()?;
    plan.backup_path = None;
    plan.confirmation_token = None;
    plan.independent_watchdog_available = false;
    plan.out_of_band_recovery_instructions = false;

    let result = evaluate_display_render_live_boundary(&plan);
    assert_eq!(result.status, DisplayRenderBoundaryStatus::Blocked);
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
fn display_render_live_boundary_rejects_dependent_confirmation_paths() -> Result<()> {
    let mut plan = complete_fixture_plan()?;
    plan.confirmation_depends_on_app_ui = true;
    plan.confirmation_depends_on_visible_display = true;
    plan.confirmation_depends_on_hyprland_keybind = true;
    plan.confirmation_depends_on_mouse_input = true;

    let result = evaluate_display_render_live_boundary(&plan);
    assert_eq!(result.status, DisplayRenderBoundaryStatus::Blocked);
    assert!(result
        .blocked_reasons
        .contains(&"visible-ui-only-confirmation-rejected".to_owned()));
    assert!(result
        .blocked_reasons
        .contains(&"visible-display-confirmation-rejected".to_owned()));
    assert!(result
        .blocked_reasons
        .contains(&"hyprland-keybind-confirmation-rejected".to_owned()));
    assert!(result
        .blocked_reasons
        .contains(&"mouse-only-confirmation-rejected".to_owned()));

    Ok(())
}

#[test]
fn display_render_live_boundary_refuses_real_config_while_live_disabled() -> Result<()> {
    let mut plan = complete_fixture_plan()?;
    plan.target_config_path = PathBuf::from("/home/kyo/.config/hypr/hyprland.conf");

    let result = evaluate_display_render_live_boundary(&plan);
    assert_eq!(result.status, DisplayRenderBoundaryStatus::Blocked);
    assert!(result
        .blocked_reasons
        .contains(&"real-config-path-refused-while-live-disabled".to_owned()));
    assert!(!result.can_execute_live_config);

    Ok(())
}

#[test]
fn display_render_boundary_reports_keep_all_high_risk_rows_blocked() -> Result<()> {
    let boundary = read_json("data/reports/display-render-live-boundary-design.v0.55.2.json")?;
    let gate = read_json("data/reports/display-render-live-mode-gate-proof.v0.55.2.json")?;
    let subset = read_json("data/reports/display-render-small-subset-readiness.v0.55.2.json")?;
    let coverage = read_json("data/reports/scalar-read-write-coverage.v0.55.2.json")?;

    assert_eq!(boundary["counts"]["displayRenderRows"], 25);
    assert_eq!(boundary["counts"]["rowsEnabled"], 2);
    assert_eq!(boundary["counts"]["displayRenderRowsStillBlocked"], 23);
    assert_eq!(gate["counts"]["liveExecutionEnabled"], false);
    assert_eq!(gate["counts"]["liveExecutionRefused"], true);
    assert_eq!(gate["counts"]["realConfigPathRefused"], true);
    assert_eq!(subset["counts"]["selectedRows"], 2);
    assert_eq!(subset["counts"]["rowsEnabled"], 2);
    assert_eq!(coverage["counts"]["writableRows"], 340);
    assert_eq!(coverage["counts"]["blockedWriteRows"], 1);

    for row in boundary["rows"].as_array().unwrap() {
        if row["rowId"].as_str().is_some_and(|row_id| {
            matches!(
                row_id,
                "xwayland.use_nearest_neighbor" | "xwayland.force_zero_scaling"
            )
        }) {
            assert_eq!(row["writeStatus"].as_str(), Some("writable"));
            assert_eq!(row["safeToEnableNow"].as_bool(), Some(true));
        } else {
            assert_eq!(row["writeStatus"].as_str(), Some("high-risk"));
            assert_eq!(row["safeToEnableNow"].as_bool(), Some(false));
        }
    }

    Ok(())
}
