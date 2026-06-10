use std::path::PathBuf;

use crate::config_graph::{ConfigGraphFile, ConfigManagementHintKind, ConfigSourceReference};
use crate::config_layered_values::{LayeredSettingValues, LayeredValueOccurrence};

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct WriteTargetCandidate {
    pub label: String,
    pub file_path: PathBuf,
    pub resolved_path: Option<PathBuf>,
    pub line_number: Option<usize>,
    pub safe: bool,
    pub generated_or_script_managed: bool,
    pub symlink_managed: bool,
    pub requires_advanced_confirmation: bool,
    pub backup_required: bool,
    pub fixture_only: bool,
}

pub fn write_target_candidates_for_layered_setting(
    layered: &LayeredSettingValues,
    files: &[ConfigGraphFile],
) -> Vec<WriteTargetCandidate> {
    layered
        .occurrences
        .iter()
        .map(|occurrence| candidate_for_occurrence(occurrence, files))
        .collect()
}

pub fn write_target_candidates_from_source_references(
    references: &[ConfigSourceReference],
) -> Vec<WriteTargetCandidate> {
    references
        .iter()
        .filter_map(|reference| {
            reference
                .resolved_target
                .as_ref()
                .map(|path| WriteTargetCandidate {
                    label: "Selected connected file".to_string(),
                    file_path: path.clone(),
                    resolved_path: None,
                    line_number: Some(reference.line_number),
                    safe: true,
                    generated_or_script_managed: false,
                    symlink_managed: false,
                    requires_advanced_confirmation: false,
                    backup_required: true,
                    fixture_only: true,
                })
        })
        .collect()
}

fn candidate_for_occurrence(
    occurrence: &LayeredValueOccurrence,
    files: &[ConfigGraphFile],
) -> WriteTargetCandidate {
    let file = files.iter().find(|file| {
        file.path == occurrence.file_path
            || file.resolved_path.as_ref() == Some(&occurrence.file_path)
    });
    let generated_or_script_managed = file.is_some_and(file_has_advanced_hints);
    let symlink_managed = occurrence.symlink_managed;
    WriteTargetCandidate {
        label: occurrence.role_label.clone(),
        file_path: occurrence.file_path.clone(),
        resolved_path: occurrence.resolved_path.clone(),
        line_number: Some(occurrence.line_number),
        safe: !generated_or_script_managed,
        generated_or_script_managed,
        symlink_managed,
        requires_advanced_confirmation: generated_or_script_managed || symlink_managed,
        backup_required: true,
        fixture_only: true,
    }
}

fn file_has_advanced_hints(file: &ConfigGraphFile) -> bool {
    file.hints.iter().any(|hint| {
        matches!(
            hint.kind,
            ConfigManagementHintKind::GeneratedFile
                | ConfigManagementHintKind::ScriptManaged
                | ConfigManagementHintKind::ScriptReferenced
                | ConfigManagementHintKind::SymlinkManaged
        )
    })
}
