#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CursorDefaultMonitorSnapshot {
    monitor_names: Vec<String>,
}

impl CursorDefaultMonitorSnapshot {
    pub fn from_names(names: impl IntoIterator<Item = impl Into<String>>) -> Self {
        let mut monitor_names = names
            .into_iter()
            .map(Into::into)
            .map(|name| name.trim().to_string())
            .filter(|name| !name.is_empty())
            .collect::<Vec<_>>();
        monitor_names.sort();
        monitor_names.dedup();
        Self { monitor_names }
    }

    pub fn monitor_names(&self) -> &[String] {
        &self.monitor_names
    }

    pub fn contains(&self, name: &str) -> bool {
        let trimmed = name.trim();
        self.monitor_names
            .iter()
            .any(|candidate| candidate == trimmed)
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum CursorDefaultMonitorOracleDecision {
    Valid,
    Missing,
    Stale,
    UnsafeSyntax,
}

pub fn parse_hyprctl_monitors_fixture(output: &str) -> CursorDefaultMonitorSnapshot {
    CursorDefaultMonitorSnapshot::from_names(output.lines().filter_map(parse_monitor_line))
}

pub fn validate_cursor_default_monitor_candidate(
    candidate: &str,
    snapshot: &CursorDefaultMonitorSnapshot,
) -> CursorDefaultMonitorOracleDecision {
    let trimmed = candidate.trim();
    if trimmed.is_empty() {
        return CursorDefaultMonitorOracleDecision::Missing;
    }
    if contains_unsafe_monitor_syntax(trimmed) {
        return CursorDefaultMonitorOracleDecision::UnsafeSyntax;
    }
    if snapshot.contains(trimmed) {
        CursorDefaultMonitorOracleDecision::Valid
    } else {
        CursorDefaultMonitorOracleDecision::Stale
    }
}

fn parse_monitor_line(line: &str) -> Option<String> {
    let trimmed = line.trim_start();
    let rest = trimmed.strip_prefix("Monitor ")?;
    let name = rest.split_whitespace().next()?;
    let cleaned = name.trim_end_matches(':').trim();
    (!cleaned.is_empty()).then(|| cleaned.to_string())
}

fn contains_unsafe_monitor_syntax(value: &str) -> bool {
    value.contains('\n')
        || value.contains('\r')
        || value.contains(';')
        || value.contains('`')
        || value.contains('$')
}
