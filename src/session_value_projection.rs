use std::path::PathBuf;

use crate::config_layered_values::LayeredSettingValues;
use crate::current_config::{CurrentValueProjection, CurrentValueSourceStatus};

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SessionValueProjection {
    pub row_id: String,
    pub official_setting_id: String,
    pub active_value: Option<String>,
    pub session_preview_value: Option<String>,
    pub comparison_status: SessionValueComparisonStatus,
    pub active_source_path: Option<PathBuf>,
    pub active_source_line: Option<usize>,
    pub session_source_path: Option<PathBuf>,
    pub session_source_line: Option<usize>,
    pub read_only: bool,
    pub affects_writes: bool,
}

impl SessionValueProjection {
    pub fn user_facing_lines(&self) -> Vec<String> {
        vec![
            "Session preview comparison".to_string(),
            format!(
                "Active config value: {}",
                self.active_value.as_deref().unwrap_or("not configured")
            ),
            format!(
                "Session preview value: {}",
                self.session_preview_value
                    .as_deref()
                    .unwrap_or("not configured")
            ),
            format!("Status: {}", self.comparison_status.user_facing_label()),
            "Apply behavior has not changed.".to_string(),
        ]
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SessionValueComparisonStatus {
    Same,
    Different,
    MissingInActiveConfig,
    MissingInSessionPreview,
    Unreadable,
    Unknown,
}

impl SessionValueComparisonStatus {
    pub fn user_facing_label(self) -> &'static str {
        match self {
            Self::Same => "Same",
            Self::Different => "Different",
            Self::MissingInActiveConfig => "Missing in active config",
            Self::MissingInSessionPreview => "Missing in session preview",
            Self::Unreadable => "Unreadable",
            Self::Unknown => "Unknown",
        }
    }
}

pub fn compare_active_and_session_values(
    row_id: impl Into<String>,
    official_setting_id: impl Into<String>,
    active: &CurrentValueProjection,
    session_layered: &LayeredSettingValues,
) -> SessionValueProjection {
    let session_occurrence = session_layered.occurrences.last();
    let session_value = session_occurrence.map(|occurrence| occurrence.raw_value.clone());

    let comparison_status = match active.status {
        CurrentValueSourceStatus::ReadUnavailable => SessionValueComparisonStatus::Unreadable,
        CurrentValueSourceStatus::NotConfigured if session_value.is_some() => {
            SessionValueComparisonStatus::MissingInActiveConfig
        }
        CurrentValueSourceStatus::Configured | CurrentValueSourceStatus::DuplicateConflict
            if session_value.is_none() =>
        {
            SessionValueComparisonStatus::MissingInSessionPreview
        }
        CurrentValueSourceStatus::NotConfigured => SessionValueComparisonStatus::Unknown,
        CurrentValueSourceStatus::Configured | CurrentValueSourceStatus::DuplicateConflict => {
            if active.raw_value == session_value {
                SessionValueComparisonStatus::Same
            } else {
                SessionValueComparisonStatus::Different
            }
        }
    };

    SessionValueProjection {
        row_id: row_id.into(),
        official_setting_id: official_setting_id.into(),
        active_value: active.raw_value.clone(),
        session_preview_value: session_value,
        comparison_status,
        active_source_path: active.source_path.clone(),
        active_source_line: active.line_number,
        session_source_path: session_occurrence.map(|occurrence| occurrence.file_path.clone()),
        session_source_line: session_occurrence.map(|occurrence| occurrence.line_number),
        read_only: true,
        affects_writes: false,
    }
}
