use std::fs;

use hyprland_settings::cursor_default_monitor_oracle::{
    parse_hyprctl_monitors_fixture, validate_cursor_default_monitor_candidate,
    CursorDefaultMonitorOracleDecision, CursorDefaultMonitorSnapshot,
};
use hyprland_settings::write_classification::is_safe_writable_setting;

const ORACLE_RESEARCH_REPORT: &str =
    "data/reports/cursor-default-monitor-runtime-oracle-research.v0.55.2.json";

#[test]
fn fixture_monitor_oracle_accepts_current_monitor_names() {
    let snapshot = CursorDefaultMonitorSnapshot::from_names(["DP-1", "eDP-1", "HDMI-A-1"]);
    assert_eq!(
        validate_cursor_default_monitor_candidate("DP-1", &snapshot),
        CursorDefaultMonitorOracleDecision::Valid
    );
    assert_eq!(
        validate_cursor_default_monitor_candidate("eDP-1", &snapshot),
        CursorDefaultMonitorOracleDecision::Valid
    );
}

#[test]
fn fixture_monitor_oracle_rejects_missing_stale_and_unsafe_monitor_names() {
    let snapshot = CursorDefaultMonitorSnapshot::from_names(["DP-1"]);
    assert_eq!(
        validate_cursor_default_monitor_candidate("", &snapshot),
        CursorDefaultMonitorOracleDecision::Missing
    );
    assert_eq!(
        validate_cursor_default_monitor_candidate("HDMI-A-1", &snapshot),
        CursorDefaultMonitorOracleDecision::Stale
    );
    assert_eq!(
        validate_cursor_default_monitor_candidate("DP-1; hyprctl reload", &snapshot),
        CursorDefaultMonitorOracleDecision::UnsafeSyntax
    );
}

#[test]
fn hyprctl_monitors_fixture_parser_extracts_monitor_names_without_live_query() {
    let fixture = "\
Monitor eDP-1 (ID 0):
\t1920x1080@60.00000 at 0x0
Monitor DP-1 (ID 1):
\t2560x1440@144.00000 at 1920x0
";
    let snapshot = parse_hyprctl_monitors_fixture(fixture);
    assert_eq!(
        snapshot.monitor_names(),
        &["DP-1".to_string(), "eDP-1".to_string()]
    );
}

#[test]
fn cursor_default_monitor_remains_blocked_until_runtime_oracle_is_complete() {
    assert!(!is_safe_writable_setting("cursor.default_monitor"));
}

#[test]
fn cursor_default_monitor_research_report_records_remaining_oracle_gap() {
    let report: serde_json::Value = serde_json::from_slice(
        &fs::read(ORACLE_RESEARCH_REPORT).expect("oracle research report should exist"),
    )
    .expect("oracle research report should parse");
    assert_eq!(report["rowId"], "cursor.default_monitor");
    assert_eq!(report["fixtureOracleImplemented"], true);
    assert_eq!(
        report["runtimeOracleNeeded"],
        "runtime monitor-name allowlist/readback oracle"
    );
    assert_eq!(
        report["proofStillMissing"],
        "live/runtime monitor-name source adapter proof and stale-name refresh proof remain missing; row remains blocked"
    );
}
