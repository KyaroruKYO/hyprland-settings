use crate::monitor_name_oracle::{
    parse_hyprctl_monitors_snapshot, validate_monitor_name_candidate, MonitorNameSnapshot,
    MonitorNameSnapshotSource, MonitorNameValidationStatus,
};

pub type CursorDefaultMonitorSnapshot = MonitorNameSnapshot;

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum CursorDefaultMonitorOracleDecision {
    Valid,
    Missing,
    Stale,
    UnsafeSyntax,
}

pub fn parse_hyprctl_monitors_fixture(output: &str) -> CursorDefaultMonitorSnapshot {
    parse_hyprctl_monitors_snapshot(output, MonitorNameSnapshotSource::Fixture)
        .expect("fixture monitor output should contain valid monitor names")
}

pub fn validate_cursor_default_monitor_candidate(
    candidate: &str,
    snapshot: &CursorDefaultMonitorSnapshot,
) -> CursorDefaultMonitorOracleDecision {
    match validate_monitor_name_candidate(candidate, snapshot).status {
        MonitorNameValidationStatus::Accepted => CursorDefaultMonitorOracleDecision::Valid,
        MonitorNameValidationStatus::Missing => CursorDefaultMonitorOracleDecision::Missing,
        MonitorNameValidationStatus::Stale => CursorDefaultMonitorOracleDecision::Stale,
        MonitorNameValidationStatus::UnsafeSyntax => {
            CursorDefaultMonitorOracleDecision::UnsafeSyntax
        }
    }
}
