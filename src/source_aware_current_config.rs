use std::collections::BTreeMap;

use crate::config_graph::ConfigGraphSummary;
use crate::config_parser::{parse_hyprland_config_file, ParseStatus, ParsedConfigLine};
use crate::current_config::{
    CurrentConfigLoadStatus, CurrentConfigSnapshot, CurrentValue, CurrentValueStatus,
};
use crate::source_values::{monitor_source_values_from_records, MonitorSourceValue};

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SourceAwareCurrentConfigReport {
    pub root_file_mapped: bool,
    pub readable_files_mapped: usize,
    pub unreadable_files: usize,
    pub scalar_count: usize,
    pub structured_count: usize,
    pub unsupported_count: usize,
    pub duplicate_setting_count: usize,
}

pub fn current_config_from_graph(graph: &ConfigGraphSummary) -> CurrentConfigSnapshot {
    let mut parsed_records = Vec::<ParsedConfigLine>::new();
    let mut read_errors = Vec::<ParsedConfigLine>::new();

    for file in &graph.files {
        if !file.readable {
            continue;
        }
        match parse_hyprland_config_file(&file.path) {
            Ok(parsed) => parsed_records.extend(parsed.records),
            Err(error) => read_errors.push(ParsedConfigLine {
                path: file.path.clone(),
                line_number: 0,
                raw_line: String::new(),
                parsed_key: None,
                raw_value: None,
                normalized_setting_id: None,
                status: ParseStatus::Unsupported,
                warning: Some(format!(
                    "connected config file could not be parsed: {error}"
                )),
            }),
        }
    }

    let mut duplicate_lines = BTreeMap::<String, Vec<usize>>::new();
    let mut occurrence_counts = BTreeMap::<String, usize>::new();
    for record in parsed_records
        .iter()
        .filter(|record| record.status == ParseStatus::Scalar)
    {
        let Some(setting_id) = &record.normalized_setting_id else {
            continue;
        };
        *occurrence_counts.entry(setting_id.clone()).or_default() += 1;
        duplicate_lines
            .entry(setting_id.clone())
            .or_default()
            .push(record.line_number);
    }
    duplicate_lines.retain(|setting_id, _| {
        occurrence_counts
            .get(setting_id)
            .copied()
            .unwrap_or_default()
            > 1
    });

    let structured_records = parsed_records
        .iter()
        .filter(|record| record.status == ParseStatus::StructuredRaw)
        .cloned()
        .collect::<Vec<_>>();
    let mut unsupported_records = parsed_records
        .iter()
        .filter(|record| record.status == ParseStatus::Unsupported || record.warning.is_some())
        .cloned()
        .collect::<Vec<_>>();
    unsupported_records.extend(read_errors);

    let mut values = BTreeMap::new();
    for record in parsed_records
        .iter()
        .filter(|record| record.status == ParseStatus::Scalar)
    {
        let Some(setting_id) = &record.normalized_setting_id else {
            continue;
        };
        let duplicate_lines = duplicate_lines.get(setting_id).cloned().unwrap_or_default();
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

    CurrentConfigSnapshot {
        status: CurrentConfigLoadStatus::Loaded {
            path: graph.root_path.clone(),
            scalar_count: values.len(),
            structured_count: structured_records.len(),
            unsupported_count: unsupported_records.len() + graph.unreadable_files.len(),
        },
        values,
        structured_records,
        unsupported_records,
    }
}

pub fn source_aware_mapping_report(
    graph: &ConfigGraphSummary,
    snapshot: &CurrentConfigSnapshot,
) -> SourceAwareCurrentConfigReport {
    let (scalar_count, structured_count, unsupported_count) = match &snapshot.status {
        CurrentConfigLoadStatus::Loaded {
            scalar_count,
            structured_count,
            unsupported_count,
            ..
        } => (*scalar_count, *structured_count, *unsupported_count),
        CurrentConfigLoadStatus::ReadUnavailable { .. }
        | CurrentConfigLoadStatus::LoadError { .. } => (0, 0, 0),
    };
    SourceAwareCurrentConfigReport {
        root_file_mapped: graph.files.iter().any(|file| file.path == graph.root_path),
        readable_files_mapped: graph.files.iter().filter(|file| file.readable).count(),
        unreadable_files: graph.unreadable_files.len(),
        scalar_count,
        structured_count,
        unsupported_count,
        duplicate_setting_count: snapshot
            .values
            .values()
            .filter(|value| value.status == CurrentValueStatus::DuplicateConflict)
            .count(),
    }
}

pub fn source_aware_monitor_source_values(
    snapshot: &CurrentConfigSnapshot,
) -> Vec<MonitorSourceValue> {
    monitor_source_values_from_records(&snapshot.structured_records)
}
