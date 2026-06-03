use std::collections::{BTreeMap, BTreeSet};

use crate::config_discovery::ConfigDiscovery;
use crate::config_parser::ParsedConfigLine;
use crate::current_config::{
    CurrentConfigSnapshot, CurrentValueProjection, CurrentValueSourceStatus,
};
use crate::export::{ExportBundle, InventoryEntry, TabEntry};
use crate::validation::ValidationSummary;
use crate::write_flow::{
    edit_projection_for_setting, review_block_reason, write_flow_config_setting,
    SettingEditProjection,
};

#[derive(Debug)]
pub struct UiProjection {
    pub export_dir: String,
    pub hyprland_version: String,
    pub schema_version: u32,
    pub summary: ValidationSummary,
    pub config_discovery: ConfigDiscovery,
    pub current_config: CurrentConfigSnapshot,
    pub known_setting_ids: BTreeSet<String>,
    pub tabs: Vec<UiTab>,
    pub settings: Vec<UiSetting>,
    pub active_write_candidates: Vec<UiWriteCandidate>,
    pub structured_families: Vec<UiStructuredFamily>,
    pub current_value_summary: UiCurrentValueSummary,
}

impl UiProjection {
    pub fn from_bundle(
        bundle: &ExportBundle,
        summary: &ValidationSummary,
        config_discovery: ConfigDiscovery,
        current_config: CurrentConfigSnapshot,
    ) -> Self {
        let mut tabs: Vec<_> = bundle.inventory.tabs.iter().map(UiTab::from).collect();
        tabs.sort_by_key(|tab| tab.order);

        let active_write_candidates: Vec<_> = bundle
            .write_safety
            .active_candidates
            .iter()
            .map(|entry| UiWriteCandidate {
                row_id: entry.row_id.clone(),
                target_mode: entry.target_mode.clone(),
                executable: entry.executable,
                command_generation_allowed: entry.command_generation_allowed,
            })
            .collect();

        let active_write_ids: Vec<_> = active_write_candidates
            .iter()
            .map(|entry| entry.row_id.as_str())
            .collect();
        let known_setting_ids = bundle
            .inventory
            .settings
            .iter()
            .map(|entry| entry.row_id.clone())
            .collect::<BTreeSet<_>>();
        let mut settings: Vec<_> = bundle
            .inventory
            .settings
            .iter()
            .map(|entry| UiSetting::from_entry(entry, &active_write_ids, &current_config))
            .collect();
        settings.sort_by_key(|setting| (setting.tab_order(&tabs), setting.row_order));

        let structured_families = structured_families_from_config(&current_config);
        let current_value_summary = current_value_summary_from_settings(&settings);

        Self {
            export_dir: bundle.export_dir.display().to_string(),
            hyprland_version: bundle.manifest.hyprland_version.clone(),
            schema_version: bundle.manifest.schema_version,
            summary: ValidationSummary {
                inventory_rows: summary.inventory_rows,
                official_scalar_covered: summary.official_scalar_covered,
                official_scalar_total: summary.official_scalar_total,
                read_allowlist_rows: summary.read_allowlist_rows,
                non_read_rows: summary.non_read_rows,
                preview_parser_needed_rows: summary.preview_parser_needed_rows,
                report_only_high_risk_rows: summary.report_only_high_risk_rows,
                safe_parsed_preview_candidates: summary.safe_parsed_preview_candidates,
                warning_preview_candidates: summary.warning_preview_candidates,
                deferred_parser_rows: summary.deferred_parser_rows,
                active_write_candidate_ids: summary.active_write_candidate_ids.clone(),
                structured_family_count: summary.structured_family_count,
            },
            config_discovery,
            current_config,
            known_setting_ids,
            tabs,
            settings,
            active_write_candidates,
            structured_families,
            current_value_summary,
        }
    }

    pub fn settings_for_tab(&self, tab_id: &str) -> Vec<UiSetting> {
        let mut rows: Vec<_> = self
            .settings
            .iter()
            .filter(|setting| setting.tab_id == tab_id)
            .cloned()
            .collect();
        rows.sort_by_key(|setting| setting.row_order);
        rows
    }

    pub fn tab_order_for(&self, tab_id: &str) -> usize {
        self.tabs
            .iter()
            .find(|tab| tab.id == tab_id)
            .map(|tab| tab.order)
            .unwrap_or(usize::MAX)
    }

    pub fn detail_for_row(&self, row_id: &str) -> Option<RowDetailProjection> {
        self.settings
            .iter()
            .find(|setting| setting.row_id == row_id)
            .map(|setting| {
                RowDetailProjection::from_setting(setting, &self.active_write_candidates)
            })
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct UiCurrentValueSummary {
    pub total_rows: usize,
    pub readable_rows: usize,
    pub unreadable_rows: usize,
    pub configured_rows: usize,
    pub unconfigured_rows: usize,
    pub duplicate_conflict_rows: usize,
    pub parser_warning_rows: usize,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct UiStructuredFamily {
    pub family_id: String,
    pub label: String,
    pub entries: Vec<UiStructuredEntry>,
    pub warning_count: usize,
    pub edit_status: String,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct UiStructuredEntry {
    pub source_path: String,
    pub line_number: usize,
    pub raw_line: String,
    pub parser_status: String,
    pub warning: Option<String>,
}

#[derive(Debug, Clone)]
pub struct UiTab {
    pub id: String,
    pub label: String,
    pub order: usize,
    pub row_count: usize,
}

impl From<&TabEntry> for UiTab {
    fn from(tab: &TabEntry) -> Self {
        Self {
            id: tab.tab_id.clone(),
            label: tab.tab_label.clone(),
            order: tab.tab_order,
            row_count: tab.row_count,
        }
    }
}

#[derive(Debug, Clone)]
pub struct UiSetting {
    pub row_id: String,
    pub official_setting: String,
    pub tab_id: String,
    pub tab_label: String,
    pub subsection: String,
    pub row_order: usize,
    pub label: String,
    pub description: String,
    pub default_config_presence: String,
    pub read_support: String,
    pub write_support: String,
    pub risk_class: String,
    pub preview_status: String,
    pub report_only: bool,
    pub is_write_candidate: bool,
    pub current_value: CurrentValueProjection,
    pub comparison: ComparisonProjection,
    pub edit: SettingEditProjection,
}

impl UiSetting {
    fn from_entry(
        entry: &InventoryEntry,
        active_write_ids: &[&str],
        current_config: &CurrentConfigSnapshot,
    ) -> Self {
        let current_value = current_value_for_entry(entry, current_config);
        let edit_current_value = write_flow_config_setting(&entry.row_id)
            .map(|setting_id| current_config.value_for(setting_id))
            .unwrap_or_else(|| current_value.clone());

        Self {
            row_id: entry.row_id.clone(),
            official_setting: entry.official_setting.clone(),
            tab_id: entry.tab_id.clone(),
            tab_label: entry.tab_label.clone(),
            subsection: entry.subsection.clone(),
            row_order: entry.row_order,
            label: entry.label.clone(),
            description: entry.description.clone(),
            default_config_presence: entry.default_config_presence.clone(),
            read_support: entry.read_support.clone(),
            write_support: entry.write_support.clone(),
            risk_class: entry.risk_class.clone(),
            preview_status: entry.preview_status.clone(),
            report_only: entry.report_only,
            is_write_candidate: active_write_ids.contains(&entry.row_id.as_str()),
            current_value: current_value.clone(),
            comparison: ComparisonProjection::from_current_value(
                &entry.default_config_presence,
                &current_value,
            ),
            edit: edit_projection_for_setting(&entry.row_id, &edit_current_value),
        }
    }

    fn tab_order(&self, tabs: &[UiTab]) -> usize {
        tabs.iter()
            .find(|tab| tab.id == self.tab_id)
            .map(|tab| tab.order)
            .unwrap_or(usize::MAX)
    }
}

fn current_value_for_entry(
    entry: &InventoryEntry,
    current_config: &CurrentConfigSnapshot,
) -> CurrentValueProjection {
    let row_value = current_config.value_for(&entry.row_id);
    if row_value.status != CurrentValueSourceStatus::NotConfigured {
        return row_value;
    }

    current_config.value_for(&entry.official_setting)
}

fn structured_families_from_config(
    current_config: &CurrentConfigSnapshot,
) -> Vec<UiStructuredFamily> {
    let mut groups: BTreeMap<String, Vec<UiStructuredEntry>> = BTreeMap::new();
    for record in &current_config.structured_records {
        let Some(family_id) = &record.normalized_setting_id else {
            continue;
        };
        groups
            .entry(family_id.clone())
            .or_default()
            .push(structured_entry_from_record(record));
    }

    groups
        .into_iter()
        .map(|(family_id, entries)| {
            let warning_count = entries
                .iter()
                .filter(|entry| entry.warning.is_some())
                .count();
            UiStructuredFamily {
                label: structured_family_label(&family_id).to_string(),
                family_id,
                entries,
                warning_count,
                edit_status: "read-only; structured editing is not implemented yet".to_string(),
            }
        })
        .collect()
}

fn current_value_summary_from_settings(settings: &[UiSetting]) -> UiCurrentValueSummary {
    let total_rows = settings.len();
    let configured_rows = settings
        .iter()
        .filter(|setting| setting.current_value.status == CurrentValueSourceStatus::Configured)
        .count();
    let unconfigured_rows = settings
        .iter()
        .filter(|setting| setting.current_value.status == CurrentValueSourceStatus::NotConfigured)
        .count();
    let duplicate_conflict_rows = settings
        .iter()
        .filter(|setting| {
            setting.current_value.status == CurrentValueSourceStatus::DuplicateConflict
        })
        .count();
    let unreadable_rows = settings
        .iter()
        .filter(|setting| setting.current_value.status == CurrentValueSourceStatus::ReadUnavailable)
        .count();
    let parser_warning_rows = settings
        .iter()
        .filter(|setting| setting.current_value.warning.is_some())
        .count();
    UiCurrentValueSummary {
        total_rows,
        readable_rows: total_rows - unreadable_rows,
        unreadable_rows,
        configured_rows,
        unconfigured_rows,
        duplicate_conflict_rows,
        parser_warning_rows,
    }
}

fn structured_entry_from_record(record: &ParsedConfigLine) -> UiStructuredEntry {
    UiStructuredEntry {
        source_path: record.path.display().to_string(),
        line_number: record.line_number,
        raw_line: record.raw_line.clone(),
        parser_status: "preserved raw structured entry".to_string(),
        warning: record.warning.clone(),
    }
}

fn structured_family_label(family_id: &str) -> &str {
    match family_id {
        "hl.monitor" => "Monitors",
        "hl.bind" => "Key bindings",
        "hl.animation" => "Animations",
        "hl.curve" => "Curves",
        "hl.gesture" => "Gestures",
        "hl.device" => "Devices",
        "hl.permission" => "Permissions",
        "hl.windowrule" => "Window rules",
        _ => "Structured entries",
    }
}

#[derive(Debug, Clone)]
pub struct UiWriteCandidate {
    pub row_id: String,
    pub target_mode: String,
    pub executable: bool,
    pub command_generation_allowed: bool,
}

#[derive(Debug, Clone)]
pub struct ComparisonProjection {
    pub badge: String,
    pub detail: String,
}

impl ComparisonProjection {
    fn from_current_value(
        default_config_presence: &str,
        current_value: &CurrentValueProjection,
    ) -> Self {
        let default_detail = if default_config_presence == "not-exported" {
            "official default value is not exported"
        } else {
            default_config_presence
        };

        match current_value.status {
            CurrentValueSourceStatus::Configured => Self {
                badge: "User configured".to_string(),
                detail: format!("user override present; {default_detail}"),
            },
            CurrentValueSourceStatus::DuplicateConflict => Self {
                badge: "Conflict".to_string(),
                detail: format!("duplicate user config entries; {default_detail}"),
            },
            CurrentValueSourceStatus::NotConfigured => Self {
                badge: "Default".to_string(),
                detail: format!("no user override found; {default_detail}"),
            },
            CurrentValueSourceStatus::ReadUnavailable => Self {
                badge: "Read unavailable".to_string(),
                detail: format!("user config was not parsed; {default_detail}"),
            },
        }
    }
}

#[derive(Debug, Clone)]
pub struct RowDetailProjection {
    pub label: String,
    pub row_id: String,
    pub official_setting: String,
    pub tab_label: String,
    pub subsection: String,
    pub description: String,
    pub default_config_presence: String,
    pub read_support: String,
    pub non_read_status: Option<String>,
    pub preview_status: String,
    pub risk_class: String,
    pub report_only_status: String,
    pub write_support: String,
    pub write_candidate_status: String,
    pub write_candidate_target_mode: Option<String>,
    pub write_candidate_executable: Option<bool>,
    pub write_candidate_command_generation_allowed: Option<bool>,
    pub current_value: CurrentValueProjection,
    pub comparison: ComparisonProjection,
    pub edit: SettingEditProjection,
    pub safety_notes: Vec<String>,
}

impl RowDetailProjection {
    fn from_setting(setting: &UiSetting, active_write_candidates: &[UiWriteCandidate]) -> Self {
        let write_candidate = active_write_candidates
            .iter()
            .find(|candidate| candidate.row_id == setting.row_id);
        let is_read_supported = setting
            .read_support
            .contains("current-value-read-allowlisted");

        let mut safety_notes = vec![
            "Current values are parsed from hyprland.conf as read-only text when available."
                .to_string(),
        ];
        if setting.edit.editable {
            safety_notes.push(
                "This row is the only active write pilot and requires review, backup, write, and reread verification."
                    .to_string(),
            );
            if let Some(reason) = review_block_reason(setting.current_value.status) {
                safety_notes.push(format!("Apply currently blocked: {reason}."));
            }
        } else {
            safety_notes.push(
                setting
                    .edit
                    .disabled_reason
                    .clone()
                    .unwrap_or_else(|| "not write-allowlisted".to_string()),
            );
        }
        safety_notes.push("No Hyprland reload command is run.".to_string());

        Self {
            label: setting.label.clone(),
            row_id: setting.row_id.clone(),
            official_setting: setting.official_setting.clone(),
            tab_label: setting.tab_label.clone(),
            subsection: setting.subsection.clone(),
            description: setting.description.clone(),
            default_config_presence: setting.default_config_presence.clone(),
            read_support: setting.read_support.clone(),
            non_read_status: (!is_read_supported)
                .then(|| "Current-value reads blocked".to_string()),
            preview_status: setting.preview_status.clone(),
            risk_class: setting.risk_class.clone(),
            report_only_status: if setting.report_only {
                "report-only".to_string()
            } else {
                "not report-only".to_string()
            },
            write_support: setting.write_support.clone(),
            write_candidate_status: if write_candidate.is_some() {
                "active write candidate gated by backup and review".to_string()
            } else {
                "not an active write candidate".to_string()
            },
            write_candidate_target_mode: write_candidate
                .map(|candidate| candidate.target_mode.clone()),
            write_candidate_executable: write_candidate.map(|candidate| candidate.executable),
            write_candidate_command_generation_allowed: write_candidate
                .map(|candidate| candidate.command_generation_allowed),
            current_value: setting.current_value.clone(),
            comparison: setting.comparison.clone(),
            edit: setting.edit.clone(),
            safety_notes,
        }
    }
}
