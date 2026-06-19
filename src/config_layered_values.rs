use std::path::PathBuf;

use crate::config_graph::{ConfigGraphFile, ConfigGraphSummary, ConfigManagementHintKind};
use crate::config_parser::{parse_hyprland_config_file, ParseStatus};

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct LayeredSettingValues {
    pub setting_id: String,
    pub occurrences: Vec<LayeredValueOccurrence>,
    pub controlled_in_more_than_one_place: bool,
    pub currently_active_value: Option<String>,
}

impl LayeredSettingValues {
    pub fn display_lines(&self) -> Vec<String> {
        let mut lines = Vec::new();
        if self.controlled_in_more_than_one_place {
            lines.push("This setting is controlled in more than one place.".to_string());
        }
        for occurrence in &self.occurrences {
            lines.push(format!(
                "{}: {}",
                occurrence.role_label, occurrence.raw_value
            ));
            for note in occurrence.friendly_notes() {
                lines.push(format!("{} note: {}", occurrence.role_label, note));
            }
        }
        if let Some(value) = &self.currently_active_value {
            lines.push(format!("Currently active: {value}"));
        }
        if self.controlled_in_more_than_one_place {
            lines.push("Choose where to save changes in a future version.".to_string());
        }
        lines
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct LayeredValueOccurrence {
    pub setting_id: String,
    pub raw_value: String,
    pub raw_line: String,
    pub file_path: PathBuf,
    pub resolved_path: Option<PathBuf>,
    pub line_number: usize,
    pub source_depth: usize,
    pub role_label: String,
    pub read_only: bool,
    pub generated_or_script_managed: bool,
    pub symlink_managed: bool,
}

impl LayeredValueOccurrence {
    pub fn friendly_notes(&self) -> Vec<String> {
        let mut notes = Vec::new();
        if self.generated_or_script_managed {
            notes.push("This file may be changed by scripts or generated tooling.".to_string());
        }
        if self.symlink_managed {
            notes.push("This file is symlinked.".to_string());
        }
        notes
    }
}

pub fn layered_values_for_setting(
    graph: &ConfigGraphSummary,
    setting_id: &str,
) -> LayeredSettingValues {
    let mut occurrences = Vec::new();
    for file in &graph.files {
        if !file.readable {
            continue;
        }
        let Ok(parsed) = parse_hyprland_config_file(&file.path) else {
            continue;
        };
        for record in parsed.scalar_records() {
            if record.normalized_setting_id.as_deref() != Some(setting_id) {
                continue;
            }
            if record.status != ParseStatus::Scalar {
                continue;
            }
            occurrences.push(LayeredValueOccurrence {
                setting_id: setting_id.to_string(),
                raw_value: record.raw_value.clone().unwrap_or_default(),
                raw_line: record.raw_line.clone(),
                file_path: record.path.clone(),
                resolved_path: file.resolved_path.clone(),
                line_number: record.line_number,
                source_depth: file.source_depth,
                role_label: layered_file_role_label(file),
                read_only: true,
                generated_or_script_managed: file_generated_or_script_managed(file),
                symlink_managed: file_symlink_managed(file),
            });
        }
    }

    let currently_active_value = occurrences
        .last()
        .map(|occurrence| occurrence.raw_value.clone());
    LayeredSettingValues {
        setting_id: setting_id.to_string(),
        controlled_in_more_than_one_place: occurrences.len() > 1,
        occurrences,
        currently_active_value,
    }
}

fn file_generated_or_script_managed(file: &ConfigGraphFile) -> bool {
    file.hints.iter().any(|hint| {
        matches!(
            hint.kind,
            ConfigManagementHintKind::GeneratedFile
                | ConfigManagementHintKind::ScriptManaged
                | ConfigManagementHintKind::ScriptReferenced
        )
    })
}

fn file_symlink_managed(file: &ConfigGraphFile) -> bool {
    file.is_symlink
        || file
            .hints
            .iter()
            .any(|hint| hint.kind == ConfigManagementHintKind::SymlinkManaged)
}

pub fn layered_file_role_label(file: &ConfigGraphFile) -> String {
    if file.source_depth == 0 {
        "Main config".to_string()
    } else {
        for (kind, label) in [
            (
                crate::config_graph::ConfigManagementHintKind::CurrentProfile,
                "Current profile",
            ),
            (
                crate::config_graph::ConfigManagementHintKind::DesktopProfile,
                "Desktop profile",
            ),
            (
                crate::config_graph::ConfigManagementHintKind::GamingProfile,
                "Gaming profile",
            ),
            (
                crate::config_graph::ConfigManagementHintKind::ThemeProfile,
                "Theme profile",
            ),
            (
                crate::config_graph::ConfigManagementHintKind::HostProfile,
                "Host profile",
            ),
            (
                crate::config_graph::ConfigManagementHintKind::ModeProfile,
                "Profile file",
            ),
        ] {
            if file.hints.iter().any(|hint| hint.kind == kind) {
                return label.to_string();
            }
        }
        "Connected config".to_string()
    }
}
