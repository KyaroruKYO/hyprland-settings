use std::collections::BTreeMap;
use std::path::PathBuf;

use crate::config_graph::{inspect_config_graph, ConfigGraphSummary};
use crate::config_parser::{parse_hyprland_config_file, ParseStatus, ParsedConfigLine};
use crate::current_config::{
    CurrentConfigLoadStatus, CurrentConfigSnapshot, CurrentValue, CurrentValueStatus,
};
use crate::durable_fs::{capture_file_precondition, content_sha256, FilePrecondition};
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

    let file_preconditions = graph_file_preconditions(graph);
    let source_graph_fingerprint = graph_fingerprint(graph, &file_preconditions);

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
        file_preconditions,
        source_graph_fingerprint,
    }
}

pub fn graph_file_preconditions(graph: &ConfigGraphSummary) -> BTreeMap<PathBuf, FilePrecondition> {
    graph
        .files
        .iter()
        .filter(|file| file.readable)
        .filter_map(|file| {
            capture_file_precondition(&file.path)
                .ok()
                .map(|precondition| (file.path.clone(), precondition))
        })
        .collect()
}

pub fn graph_fingerprint(
    graph: &ConfigGraphSummary,
    preconditions: &BTreeMap<PathBuf, FilePrecondition>,
) -> Option<String> {
    if graph.files.iter().filter(|file| file.readable).count() != preconditions.len() {
        return None;
    }
    let mut evidence = Vec::new();
    evidence.extend_from_slice(graph.root_path.to_string_lossy().as_bytes());
    evidence.push(0);
    for file in &graph.files {
        evidence.extend_from_slice(file.path.to_string_lossy().as_bytes());
        evidence.push(0);
        evidence.extend_from_slice(file.source_depth.to_string().as_bytes());
        evidence.push(u8::from(file.readable));
        evidence.push(u8::from(file.is_symlink));
        if let Some(precondition) = preconditions.get(&file.path) {
            evidence.extend_from_slice(precondition.canonical_path.to_string_lossy().as_bytes());
            evidence.push(0);
            evidence.extend_from_slice(precondition.sha256.as_bytes());
            evidence.extend_from_slice(&precondition.metadata.device.to_le_bytes());
            evidence.extend_from_slice(&precondition.metadata.inode.to_le_bytes());
        }
        evidence.push(0xff);
    }
    for reference in &graph.source_references {
        evidence.extend_from_slice(reference.source_file.to_string_lossy().as_bytes());
        evidence.extend_from_slice(&reference.line_number.to_le_bytes());
        evidence.extend_from_slice(reference.raw_line.as_bytes());
        evidence.push(0xfe);
    }
    for issue in graph
        .unreadable_files
        .iter()
        .chain(graph.cycles.iter())
        .chain(graph.unsupported_sources.iter())
    {
        evidence.extend_from_slice(issue.path.to_string_lossy().as_bytes());
        evidence.extend_from_slice(issue.message.as_bytes());
        evidence.push(0xfd);
    }
    Some(content_sha256(&evidence))
}

pub fn current_source_graph_fingerprint(root: &std::path::Path) -> Option<String> {
    let graph = inspect_config_graph(root);
    let preconditions = graph_file_preconditions(&graph);
    graph_fingerprint(&graph, &preconditions)
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
