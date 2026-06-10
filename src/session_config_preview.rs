use std::collections::BTreeMap;
use std::path::{Path, PathBuf};

use crate::config_graph::{
    inspect_config_graph_with_options, ConfigGraphOptions, ConfigGraphSummary, SourceFollowPolicy,
};
use crate::config_parser::{parse_hyprland_config_file, ParseStatus};
use crate::config_selection::SourceFollowChoice;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SessionConfigPreview {
    pub selected_config_path: PathBuf,
    pub source_follow_choice: SourceFollowChoice,
    pub graph: ConfigGraphSummary,
    pub read_status: SessionConfigPreviewReadStatus,
    pub readable_file_count: usize,
    pub scalar_value_count: usize,
    pub settings_with_multiple_locations: usize,
    pub generated_or_script_managed_hints: bool,
    pub active: bool,
    pub clearable: bool,
    pub persisted: bool,
    pub affects_writes: bool,
}

impl SessionConfigPreview {
    pub fn user_facing_lines(&self) -> Vec<String> {
        let mut lines = vec![
            "Session preview".to_string(),
            format!("Config: {}", self.selected_config_path.display()),
            "Using this config for this app session only.".to_string(),
            "This is not saved.".to_string(),
            "Apply behavior has not changed.".to_string(),
            "This config is being reread for display only.".to_string(),
            format!("Connected files found: {}", self.graph.connected_file_count),
            format!("Readable files: {}", self.readable_file_count),
            format!("Unreadable files: {}", self.graph.unreadable_file_count),
        ];

        match &self.read_status {
            SessionConfigPreviewReadStatus::Readable => {
                lines.push(format!(
                    "Values read for preview: {}",
                    self.scalar_value_count
                ));
                lines.push(format!(
                    "Settings with multiple locations: {}",
                    self.settings_with_multiple_locations
                ));
            }
            SessionConfigPreviewReadStatus::Unreadable { reason } => {
                lines.push("This file could not be read for preview.".to_string());
                lines.push(reason.clone());
                lines.push("No changes were made.".to_string());
            }
        }

        lines.push(format!(
            "Generated/script-managed hints: {}",
            if self.generated_or_script_managed_hints {
                "detected"
            } else {
                "not detected"
            }
        ));

        if self.source_follow_choice == SourceFollowChoice::OnlySelectedFile {
            lines.push("Connected files are not included in this session preview.".to_string());
        }
        lines.push("Choose where to save changes in a future version.".to_string());
        lines
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum SessionConfigPreviewReadStatus {
    Readable,
    Unreadable { reason: String },
}

pub fn build_session_config_preview(
    selected_config_path: impl AsRef<Path>,
    source_follow_choice: SourceFollowChoice,
) -> SessionConfigPreview {
    let selected_config_path = selected_config_path.as_ref().to_path_buf();
    let source_follow_policy = match source_follow_choice {
        SourceFollowChoice::ReviewAllConnectedFiles => SourceFollowPolicy::ReviewAll,
        SourceFollowChoice::OnlySelectedFile | SourceFollowChoice::Cancel => {
            SourceFollowPolicy::OnlyRoot
        }
    };
    let graph = inspect_config_graph_with_options(
        &selected_config_path,
        ConfigGraphOptions {
            source_follow_policy,
            ..ConfigGraphOptions::from_env()
        },
    );

    let readable_file_count = graph.files.iter().filter(|file| file.readable).count();
    let root_read_status = graph
        .files
        .first()
        .filter(|file| file.readable)
        .map(|_| SessionConfigPreviewReadStatus::Readable)
        .unwrap_or_else(|| SessionConfigPreviewReadStatus::Unreadable {
            reason: "The selected config could not be read.".to_string(),
        });

    let (scalar_value_count, settings_with_multiple_locations) =
        if root_read_status == SessionConfigPreviewReadStatus::Readable {
            preview_value_counts(&graph)
        } else {
            (0, 0)
        };

    SessionConfigPreview {
        selected_config_path,
        source_follow_choice,
        generated_or_script_managed_hints: graph.has_generated_hints
            || graph.has_script_managed_hints,
        graph,
        read_status: root_read_status,
        readable_file_count,
        scalar_value_count,
        settings_with_multiple_locations,
        active: true,
        clearable: true,
        persisted: false,
        affects_writes: false,
    }
}

fn preview_value_counts(graph: &ConfigGraphSummary) -> (usize, usize) {
    let mut scalar_value_count = 0usize;
    let mut occurrences: BTreeMap<String, usize> = BTreeMap::new();

    for file in &graph.files {
        if !file.readable {
            continue;
        }
        let Ok(parsed) = parse_hyprland_config_file(&file.path) else {
            continue;
        };
        for record in parsed.scalar_records() {
            if record.status != ParseStatus::Scalar {
                continue;
            }
            let Some(setting_id) = &record.normalized_setting_id else {
                continue;
            };
            scalar_value_count += 1;
            *occurrences.entry(setting_id.clone()).or_default() += 1;
        }
    }

    let settings_with_multiple_locations = occurrences.values().filter(|count| **count > 1).count();
    (scalar_value_count, settings_with_multiple_locations)
}
