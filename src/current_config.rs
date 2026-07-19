use std::collections::BTreeMap;
use std::path::PathBuf;

use crate::config_discovery::{ConfigDiscovery, ConfigDiscoveryStatus};
use crate::config_parser::{
    parse_hyprland_config_file, ParseStatus, ParsedConfig, ParsedConfigLine,
};
use crate::durable_fs::{capture_file_precondition, FilePrecondition};
use crate::source_values::{monitor_source_values_from_records, MonitorSourceValue};
use crate::structured_family::{structured_family_projections, StructuredFamilyProjection};

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CurrentConfigSnapshot {
    pub status: CurrentConfigLoadStatus,
    pub values: BTreeMap<String, CurrentValue>,
    pub structured_records: Vec<ParsedConfigLine>,
    pub unsupported_records: Vec<ParsedConfigLine>,
    /// Exact startup/reload evidence used by every active-config write.
    pub file_preconditions: BTreeMap<PathBuf, FilePrecondition>,
    /// Present for source-aware snapshots. A write must reproduce this graph
    /// immediately before touching any target.
    pub source_graph_fingerprint: Option<String>,
}

impl CurrentConfigSnapshot {
    pub fn read_unavailable(reason: impl Into<String>) -> Self {
        Self {
            status: CurrentConfigLoadStatus::ReadUnavailable {
                reason: reason.into(),
            },
            values: BTreeMap::new(),
            structured_records: Vec::new(),
            unsupported_records: Vec::new(),
            file_preconditions: BTreeMap::new(),
            source_graph_fingerprint: None,
        }
    }

    pub fn from_discovery(discovery: &ConfigDiscovery) -> Self {
        match &discovery.status {
            ConfigDiscoveryStatus::Found { path, .. } => match parse_hyprland_config_file(path) {
                Ok(parsed) => Self::from_parsed(parsed),
                Err(error) => Self {
                    status: CurrentConfigLoadStatus::LoadError {
                        path: path.clone(),
                        error: error.to_string(),
                    },
                    values: BTreeMap::new(),
                    structured_records: Vec::new(),
                    unsupported_records: Vec::new(),
                    file_preconditions: BTreeMap::new(),
                    source_graph_fingerprint: None,
                },
            },
            ConfigDiscoveryStatus::Missing => Self::read_unavailable("No Hyprland config found."),
            ConfigDiscoveryStatus::Unreadable { path, error, .. } => Self {
                status: CurrentConfigLoadStatus::LoadError {
                    path: path.clone(),
                    error: error.clone(),
                },
                values: BTreeMap::new(),
                structured_records: Vec::new(),
                unsupported_records: Vec::new(),
                file_preconditions: BTreeMap::new(),
                source_graph_fingerprint: None,
            },
            ConfigDiscoveryStatus::NotAFile { path, .. } => Self {
                status: CurrentConfigLoadStatus::LoadError {
                    path: path.clone(),
                    error: "config path is not a regular file".to_string(),
                },
                values: BTreeMap::new(),
                structured_records: Vec::new(),
                unsupported_records: Vec::new(),
                file_preconditions: BTreeMap::new(),
                source_graph_fingerprint: None,
            },
        }
    }

    pub fn from_parsed(parsed: ParsedConfig) -> Self {
        let structured_records: Vec<_> = parsed
            .records
            .iter()
            .filter(|record| record.status == ParseStatus::StructuredRaw)
            .cloned()
            .collect();
        let unsupported_records: Vec<_> = parsed
            .records
            .iter()
            .filter(|record| record.status == ParseStatus::Unsupported || record.warning.is_some())
            .cloned()
            .collect();

        let mut values = BTreeMap::new();
        for record in parsed.scalar_records() {
            let Some(setting_id) = &record.normalized_setting_id else {
                continue;
            };
            let duplicate_lines = parsed
                .duplicate_scalar_keys
                .get(setting_id)
                .cloned()
                .unwrap_or_default();
            let status = if duplicate_lines.is_empty() {
                CurrentValueStatus::Configured
            } else {
                CurrentValueStatus::DuplicateConflict
            };
            values.insert(
                setting_id.clone(),
                CurrentValue {
                    setting_id: setting_id.clone(),
                    raw_value: record.raw_value.clone().unwrap_or_default(),
                    source_path: record.path.clone(),
                    line_number: record.line_number,
                    raw_line: record.raw_line.clone(),
                    duplicate_lines,
                    status,
                    warning: record.warning.clone(),
                },
            );
        }

        let path = parsed.path;
        let mut file_preconditions = BTreeMap::new();
        if let Ok(precondition) = capture_file_precondition(&path) {
            file_preconditions.insert(path.clone(), precondition);
        }

        Self {
            status: CurrentConfigLoadStatus::Loaded {
                path,
                scalar_count: values.len(),
                structured_count: structured_records.len(),
                unsupported_count: unsupported_records.len(),
            },
            values,
            structured_records,
            unsupported_records,
            file_preconditions,
            source_graph_fingerprint: None,
        }
    }

    pub fn value_for(&self, setting_id: &str) -> CurrentValueProjection {
        match &self.status {
            CurrentConfigLoadStatus::ReadUnavailable { reason } => {
                CurrentValueProjection::read_unavailable(reason.clone())
            }
            CurrentConfigLoadStatus::LoadError { error, .. } => {
                CurrentValueProjection::read_unavailable(error.clone())
            }
            CurrentConfigLoadStatus::Loaded { .. } => self
                .values
                .get(setting_id)
                .map(CurrentValueProjection::from)
                .unwrap_or_else(CurrentValueProjection::not_configured),
        }
    }

    pub fn summary(&self) -> String {
        match &self.status {
            CurrentConfigLoadStatus::Loaded {
                path,
                scalar_count,
                structured_count,
                unsupported_count,
            } => format!(
                "Current config loaded: {} · scalar values: {} · structured records: {} · parser warnings: {}",
                path.display(),
                scalar_count,
                structured_count,
                unsupported_count
            ),
            CurrentConfigLoadStatus::ReadUnavailable { reason } => {
                format!("Current config values unavailable: {reason}")
            }
            CurrentConfigLoadStatus::LoadError { path, error } => {
                format!("Current config could not be parsed: {}: {error}", path.display())
            }
        }
    }

    pub fn structured_family_counts(&self) -> BTreeMap<String, usize> {
        let mut counts = BTreeMap::new();
        for record in &self.structured_records {
            if let Some(family) = &record.normalized_setting_id {
                *counts.entry(family.clone()).or_insert(0) += 1;
            }
        }
        counts
    }

    pub fn monitor_source_values(&self) -> Vec<MonitorSourceValue> {
        monitor_source_values_from_records(&self.structured_records)
    }

    pub fn structured_summary(&self) -> String {
        let counts = self.structured_family_counts();
        if counts.is_empty() {
            return "Structured config entries: none parsed.".to_string();
        }
        let entries = counts
            .iter()
            .map(|(family, count)| format!("{family}: {count}"))
            .collect::<Vec<_>>()
            .join(" · ");
        format!("Structured config entries preserved read-only: {entries}")
    }

    pub fn structured_family_projections(&self) -> Vec<StructuredFamilyProjection> {
        structured_family_projections(self)
    }

    pub fn file_precondition(&self, path: &PathBuf) -> Option<&FilePrecondition> {
        self.file_preconditions.get(path).or_else(|| {
            self.file_preconditions
                .values()
                .find(|precondition| &precondition.canonical_path == path)
        })
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum CurrentConfigLoadStatus {
    Loaded {
        path: PathBuf,
        scalar_count: usize,
        structured_count: usize,
        unsupported_count: usize,
    },
    ReadUnavailable {
        reason: String,
    },
    LoadError {
        path: PathBuf,
        error: String,
    },
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CurrentValue {
    pub setting_id: String,
    pub raw_value: String,
    pub source_path: PathBuf,
    pub line_number: usize,
    pub raw_line: String,
    pub duplicate_lines: Vec<usize>,
    pub status: CurrentValueStatus,
    pub warning: Option<String>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CurrentValueStatus {
    Configured,
    DuplicateConflict,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CurrentValueProjection {
    pub status: CurrentValueSourceStatus,
    pub raw_value: Option<String>,
    pub source_path: Option<PathBuf>,
    pub line_number: Option<usize>,
    pub raw_line: Option<String>,
    pub duplicate_lines: Vec<usize>,
    pub warning: Option<String>,
}

impl CurrentValueProjection {
    pub fn not_configured() -> Self {
        Self {
            status: CurrentValueSourceStatus::NotConfigured,
            raw_value: None,
            source_path: None,
            line_number: None,
            raw_line: None,
            duplicate_lines: Vec::new(),
            warning: None,
        }
    }

    pub fn read_unavailable(reason: String) -> Self {
        Self {
            status: CurrentValueSourceStatus::ReadUnavailable,
            raw_value: None,
            source_path: None,
            line_number: None,
            raw_line: None,
            duplicate_lines: Vec::new(),
            warning: Some(reason),
        }
    }

    pub fn status_label(&self) -> &'static str {
        match self.status {
            CurrentValueSourceStatus::NotConfigured => "not configured",
            CurrentValueSourceStatus::Configured => "configured in user config",
            CurrentValueSourceStatus::DuplicateConflict => "duplicate/conflicting",
            CurrentValueSourceStatus::ReadUnavailable => "read unavailable",
        }
    }
}

impl From<&CurrentValue> for CurrentValueProjection {
    fn from(value: &CurrentValue) -> Self {
        Self {
            status: match value.status {
                CurrentValueStatus::Configured => CurrentValueSourceStatus::Configured,
                CurrentValueStatus::DuplicateConflict => {
                    CurrentValueSourceStatus::DuplicateConflict
                }
            },
            raw_value: Some(value.raw_value.clone()),
            source_path: Some(value.source_path.clone()),
            line_number: Some(value.line_number),
            raw_line: Some(value.raw_line.clone()),
            duplicate_lines: value.duplicate_lines.clone(),
            warning: value.warning.clone(),
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CurrentValueSourceStatus {
    NotConfigured,
    Configured,
    DuplicateConflict,
    ReadUnavailable,
}
