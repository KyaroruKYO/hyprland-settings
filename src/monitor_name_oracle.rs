use std::path::PathBuf;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum MonitorNameSnapshotSource {
    Fixture,
    Mock,
    ReadOnlyHyprctl,
}

impl MonitorNameSnapshotSource {
    pub fn as_str(self) -> &'static str {
        match self {
            Self::Fixture => "fixture",
            Self::Mock => "mock",
            Self::ReadOnlyHyprctl => "read-only-hyprctl",
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct MonitorNameSnapshot {
    source: MonitorNameSnapshotSource,
    monitor_names: Vec<String>,
}

impl MonitorNameSnapshot {
    pub fn from_names(
        source: MonitorNameSnapshotSource,
        names: impl IntoIterator<Item = impl Into<String>>,
    ) -> Result<Self, MonitorNameOracleError> {
        let mut monitor_names = Vec::new();
        for name in names {
            let name = name.into();
            let trimmed = name.trim();
            if trimmed.is_empty() {
                continue;
            }
            if contains_unsafe_monitor_syntax(trimmed) {
                return Err(MonitorNameOracleError::UnsafeMonitorName(
                    trimmed.to_string(),
                ));
            }
            monitor_names.push(trimmed.to_string());
        }
        monitor_names.sort();
        monitor_names.dedup();
        if monitor_names.is_empty() {
            return Err(MonitorNameOracleError::EmptySnapshot);
        }
        Ok(Self {
            source,
            monitor_names,
        })
    }

    pub fn source(&self) -> MonitorNameSnapshotSource {
        self.source
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
pub struct MonitorNameCandidate {
    pub value: String,
}

impl MonitorNameCandidate {
    pub fn new(value: impl Into<String>) -> Self {
        Self {
            value: value.into(),
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct MonitorNameValidation {
    pub candidate: String,
    pub status: MonitorNameValidationStatus,
    pub snapshot_source: MonitorNameSnapshotSource,
    pub valid_names: Vec<String>,
    pub non_mutating: bool,
}

impl MonitorNameValidation {
    pub fn accepted(&self) -> bool {
        self.status == MonitorNameValidationStatus::Accepted && self.non_mutating
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum MonitorNameValidationStatus {
    Accepted,
    Missing,
    Stale,
    UnsafeSyntax,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum MonitorNameOracleError {
    EmptySnapshot,
    MalformedMonitorLine(String),
    UnsafeMonitorName(String),
    ReadOnlyAdapterUnavailable(String),
}

impl std::fmt::Display for MonitorNameOracleError {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::EmptySnapshot => write!(formatter, "monitor snapshot contains no monitor names"),
            Self::MalformedMonitorLine(line) => {
                write!(formatter, "malformed monitor line: {line}")
            }
            Self::UnsafeMonitorName(name) => write!(formatter, "unsafe monitor name: {name}"),
            Self::ReadOnlyAdapterUnavailable(reason) => {
                write!(formatter, "read-only monitor adapter unavailable: {reason}")
            }
        }
    }
}

impl std::error::Error for MonitorNameOracleError {}

pub trait MonitorNameOracle {
    fn snapshot(&self) -> Result<MonitorNameSnapshot, MonitorNameOracleError>;

    fn validate_candidate(
        &self,
        candidate: MonitorNameCandidate,
    ) -> Result<MonitorNameValidation, MonitorNameOracleError> {
        let snapshot = self.snapshot()?;
        Ok(validate_monitor_name_candidate(&candidate.value, &snapshot))
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct FixtureMonitorNameOracle {
    output: String,
}

impl FixtureMonitorNameOracle {
    pub fn new(output: impl Into<String>) -> Self {
        Self {
            output: output.into(),
        }
    }
}

impl MonitorNameOracle for FixtureMonitorNameOracle {
    fn snapshot(&self) -> Result<MonitorNameSnapshot, MonitorNameOracleError> {
        parse_hyprctl_monitors_snapshot(&self.output, MonitorNameSnapshotSource::Fixture)
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct MockMonitorNameOracle {
    snapshot: MonitorNameSnapshot,
}

impl MockMonitorNameOracle {
    pub fn from_names(
        names: impl IntoIterator<Item = impl Into<String>>,
    ) -> Result<Self, MonitorNameOracleError> {
        Ok(Self {
            snapshot: MonitorNameSnapshot::from_names(MonitorNameSnapshotSource::Mock, names)?,
        })
    }
}

impl MonitorNameOracle for MockMonitorNameOracle {
    fn snapshot(&self) -> Result<MonitorNameSnapshot, MonitorNameOracleError> {
        Ok(self.snapshot.clone())
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ReadOnlyHyprctlMonitorNameOracle {
    hyprctl_path: PathBuf,
}

impl ReadOnlyHyprctlMonitorNameOracle {
    pub fn new(hyprctl_path: impl Into<PathBuf>) -> Self {
        Self {
            hyprctl_path: hyprctl_path.into(),
        }
    }

    pub fn command_path(&self) -> &PathBuf {
        &self.hyprctl_path
    }

    pub fn command_args(&self) -> &'static [&'static str] {
        &["monitors"]
    }

    pub fn parse_read_only_output(
        &self,
        output: &str,
    ) -> Result<MonitorNameSnapshot, MonitorNameOracleError> {
        parse_hyprctl_monitors_snapshot(output, MonitorNameSnapshotSource::ReadOnlyHyprctl)
    }
}

pub fn parse_hyprctl_monitors_snapshot(
    output: &str,
    source: MonitorNameSnapshotSource,
) -> Result<MonitorNameSnapshot, MonitorNameOracleError> {
    let mut names = Vec::new();
    for line in output.lines() {
        let trimmed = line.trim_start();
        if !trimmed.starts_with("Monitor ") {
            continue;
        }
        let rest = trimmed
            .strip_prefix("Monitor ")
            .expect("starts_with checked prefix");
        let Some(raw_name) = rest.split_whitespace().next() else {
            return Err(MonitorNameOracleError::MalformedMonitorLine(
                trimmed.to_string(),
            ));
        };
        let name = raw_name.trim_end_matches(':').trim();
        if name.is_empty() {
            return Err(MonitorNameOracleError::MalformedMonitorLine(
                trimmed.to_string(),
            ));
        }
        if contains_unsafe_monitor_syntax(name) {
            return Err(MonitorNameOracleError::UnsafeMonitorName(name.to_string()));
        }
        names.push(name.to_string());
    }
    MonitorNameSnapshot::from_names(source, names)
}

pub fn validate_monitor_name_candidate(
    candidate: &str,
    snapshot: &MonitorNameSnapshot,
) -> MonitorNameValidation {
    let trimmed = candidate.trim();
    let status = if trimmed.is_empty() {
        MonitorNameValidationStatus::Missing
    } else if contains_unsafe_monitor_syntax(trimmed) {
        MonitorNameValidationStatus::UnsafeSyntax
    } else if snapshot.contains(trimmed) {
        MonitorNameValidationStatus::Accepted
    } else {
        MonitorNameValidationStatus::Stale
    };
    MonitorNameValidation {
        candidate: trimmed.to_string(),
        status,
        snapshot_source: snapshot.source(),
        valid_names: snapshot.monitor_names().to_vec(),
        non_mutating: true,
    }
}

pub fn contains_unsafe_monitor_syntax(value: &str) -> bool {
    value.contains('\n')
        || value.contains('\r')
        || value.contains(';')
        || value.contains('`')
        || value.contains('$')
        || value.contains('/')
        || value.contains('\\')
        || value.contains("..")
        || value.starts_with('~')
}
