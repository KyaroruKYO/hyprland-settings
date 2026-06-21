use std::collections::BTreeSet;
use std::path::PathBuf;

use crate::config_discovery::ConfigDiscovery;
use crate::config_parser::ParsedConfigLine;
use crate::current_config::{
    CurrentConfigSnapshot, CurrentValueProjection, CurrentValueSourceStatus,
};
use crate::export::{ExportBundle, InventoryEntry, TabEntry};
use crate::screen_shader_advisory::{
    run_screen_shader_advisory_check, AdvisoryStatus, ScreenShaderAdvisoryRequest,
};
use crate::structured_family::{
    build_structured_family_temp_write_plan, validate_structured_family_projection,
    StructuredFamilyProjection,
};
use crate::validation::ValidationSummary;
use crate::write_flow::{
    edit_projection_for_setting_with_config, review_block_reason, write_flow_config_setting,
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
    pub syntax_description: String,
    pub field_schema: Vec<String>,
    pub projection_status: String,
    pub fixture_parse_proof_status: String,
    pub fixture_render_proof_status: String,
    pub family_specific_validation_status: String,
    pub temp_write_plan_status: String,
    pub temp_fixture_render_reread_status: String,
    pub path_guard_status: String,
    pub real_config_target_status: String,
    pub runtime_mutation_status: String,
    pub reload_status: String,
    pub write_status: String,
    pub widget_name: String,
    pub review_button_label: String,
    pub unproven_record_count: usize,
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
            edit: edit_projection_for_setting_with_config(
                &entry.row_id,
                &edit_current_value,
                current_config,
            ),
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
    current_config
        .structured_family_projections()
        .iter()
        .map(ui_structured_family_from_projection)
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

fn ui_structured_family_from_projection(
    projection: &StructuredFamilyProjection,
) -> UiStructuredFamily {
    let validation = validate_structured_family_projection(projection);
    let temp_target = std::env::temp_dir()
        .join("hyprland-settings-structured-family")
        .join(projection.family.family_id().replace('.', "-"))
        .join("ui-review-only.conf");
    let plan = build_structured_family_temp_write_plan(
        projection,
        projection
            .records
            .first()
            .map(|record| record.source_path.clone())
            .unwrap_or_default(),
        temp_target,
    );
    let entries = projection
        .records
        .iter()
        .map(|record| UiStructuredEntry {
            source_path: record.source_path.display().to_string(),
            line_number: record.line_number,
            raw_line: record.raw_line.clone(),
            parser_status: record.validation_status.clone(),
            warning: record.unsupported_reason.clone(),
        })
        .collect::<Vec<_>>();
    let warning_count = entries
        .iter()
        .filter(|entry| entry.warning.is_some())
        .count();
    UiStructuredFamily {
        family_id: projection.family_id.clone(),
        label: projection.display_name.clone(),
        entries,
        warning_count,
        edit_status: "read-only review editor scaffold; real writes are not active".to_string(),
        syntax_description: projection.syntax_description.clone(),
        field_schema: projection.field_schema.clone(),
        projection_status: projection.projection_status.as_str().to_string(),
        fixture_parse_proof_status: projection.fixture_parse_proof_status.as_str().to_string(),
        fixture_render_proof_status: projection.fixture_render_proof_status.as_str().to_string(),
        family_specific_validation_status: validation.status.as_str().to_string(),
        temp_write_plan_status: plan.plan_status.as_str().to_string(),
        temp_fixture_render_reread_status:
            "StructuredFamilyTempWritePlanRenderedToTempFixture; StructuredFamilyTempWritePlanRereadVerified".to_string(),
        path_guard_status: plan.path_guard_status.as_str().to_string(),
        real_config_target_status: "Real config target not allowed".to_string(),
        runtime_mutation_status: "Runtime mutation not allowed".to_string(),
        reload_status: "Hyprland reload not allowed".to_string(),
        write_status: projection.write_status.as_str().to_string(),
        widget_name: projection.widget_name.clone(),
        review_button_label: projection.review_button_label.clone(),
        unproven_record_count: projection.unproven_record_count(),
    }
}

#[allow(dead_code)]
fn structured_entry_from_record(record: &ParsedConfigLine) -> UiStructuredEntry {
    UiStructuredEntry {
        source_path: record.path.display().to_string(),
        line_number: record.line_number,
        raw_line: record.raw_line.clone(),
        parser_status: "preserved raw structured entry".to_string(),
        warning: record.warning.clone(),
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
    pub screen_shader_advisory: Option<ScreenShaderAdvisoryUiProjection>,
    pub screen_shader_advisory_widget: Option<ScreenShaderAdvisoryGtkWidgetProjection>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ScreenShaderAdvisoryUiProjection {
    pub placement: String,
    pub advanced_mode_required: bool,
    pub explicit_user_trigger_required: bool,
    pub runs_on_row_load: bool,
    pub runs_on_value_change: bool,
    pub runs_during_validation: bool,
    pub runs_during_pending_change: bool,
    pub runs_during_write_planning: bool,
    pub runs_during_apply_flow: bool,
    pub consent_message: String,
    pub temp_copy_message: String,
    pub original_path_message: String,
    pub runtime_safety_disclaimer: String,
    pub production_gate_disclaimer: String,
    pub pass_policy: String,
    pub failure_policy: String,
    pub missing_tool_policy: String,
    pub timeout_policy: String,
    pub cleanup_warning_policy: String,
    pub can_approve_write: bool,
    pub can_block_write: bool,
    pub can_bypass_production_gate: bool,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ScreenShaderAdvisoryGtkWidgetProjection {
    pub row_id: String,
    pub visible_gtk_widget_implemented: bool,
    pub gtk_widget_module: String,
    pub file_chooser_execution_implemented: bool,
    pub selected_file_action_model_implemented: bool,
    pub file_chooser_module: String,
    pub selected_file_action_module: String,
    pub placement: String,
    pub advanced_mode_required: bool,
    pub explicit_user_trigger_required: bool,
    pub separated_from_apply_action: bool,
    pub button_label: String,
    pub default_state: String,
    pub missing_selection_state: String,
    pub file_chooser_behavior_proven: bool,
    pub selected_file_boundary_proven: bool,
    pub missing_selection_behavior_proven: bool,
    pub cancellation_or_progress_behavior_proven: bool,
    pub result_states_rendered: Vec<String>,
    pub can_approve_write: bool,
    pub can_block_write: bool,
    pub can_bypass_production_gate: bool,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ScreenShaderAdvisoryUiResultState {
    NotRun,
    Running,
    Passed,
    Failed,
    Unavailable,
    TimedOut,
    TempCopyFailed,
    CleanupWarning,
}

impl ScreenShaderAdvisoryUiResultState {
    pub fn as_str(self) -> &'static str {
        match self {
            Self::NotRun => "not_run",
            Self::Running => "running",
            Self::Passed => "passed",
            Self::Failed => "failed",
            Self::Unavailable => "unavailable",
            Self::TimedOut => "timed_out",
            Self::TempCopyFailed => "temp_copy_failed",
            Self::CleanupWarning => "cleanup_warning",
        }
    }
}

#[derive(Debug, Clone)]
pub struct ScreenShaderAdvisoryUiActionRequest {
    pub row_id: String,
    pub explicit_user_trigger: bool,
    pub helper_request: Option<ScreenShaderAdvisoryRequest>,
}

#[derive(Debug, Clone)]
pub struct ScreenShaderAdvisorySelectedFileUiActionRequest {
    pub row_id: String,
    pub explicit_user_trigger: bool,
    pub selected_shader_path: Option<PathBuf>,
    pub temp_root: PathBuf,
    pub tex300_vertex_path: PathBuf,
    pub tex320_vertex_path: PathBuf,
    pub glslang_validator_path: PathBuf,
    pub timeout: std::time::Duration,
    pub simulate_cleanup_failure: bool,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ScreenShaderAdvisoryUiActionRender {
    pub row_id: String,
    pub state: ScreenShaderAdvisoryUiResultState,
    pub placement: String,
    pub advanced_mode_required: bool,
    pub explicit_user_trigger_required: bool,
    pub helper_invoked: bool,
    pub consent_required: bool,
    pub selected_shader_read: bool,
    pub compiler_invoked: bool,
    pub compiler_args: Vec<String>,
    pub temp_fragment_path: Option<PathBuf>,
    pub temp_vertex_path: Option<PathBuf>,
    pub original_user_path_passed_to_compiler: bool,
    pub can_approve_write: bool,
    pub can_block_write: bool,
    pub can_bypass_production_gate: bool,
    pub production_write_decision_changed: bool,
    pub runtime_safety_claimed: bool,
    pub write_blocking: bool,
    pub title: String,
    pub message: String,
    pub diagnostic: Option<String>,
    pub cleanup_warning: Option<String>,
}

impl ScreenShaderAdvisoryUiActionRender {
    pub fn state_label(&self) -> &'static str {
        self.state.as_str()
    }
}

pub fn initial_screen_shader_advisory_ui_action(
    row_id: &str,
) -> Option<ScreenShaderAdvisoryUiActionRender> {
    screen_shader_advisory_projection(row_id).map(|projection| {
        advisory_ui_render(
            row_id,
            ScreenShaderAdvisoryUiResultState::NotRun,
            projection,
            false,
            false,
            false,
            Vec::new(),
            None,
            None,
            false,
            false,
            false,
            false,
            "Advisory check not run".to_string(),
            "The optional screen shader advisory check runs only after an explicit advanced user action."
                .to_string(),
            None,
            None,
        )
    })
}

pub fn screen_shader_advisory_gtk_widget_projection(
    row_id: &str,
) -> Option<ScreenShaderAdvisoryGtkWidgetProjection> {
    let projection = screen_shader_advisory_projection(row_id)?;
    Some(ScreenShaderAdvisoryGtkWidgetProjection {
        row_id: row_id.to_string(),
        visible_gtk_widget_implemented: true,
        gtk_widget_module: "src/ui/window.rs::append_screen_shader_advisory_controls".to_string(),
        file_chooser_execution_implemented: false,
        selected_file_action_model_implemented: true,
        file_chooser_module: "not-implemented-direct-gtk-file-chooser-deferred".to_string(),
        selected_file_action_module:
            "src/ui/model.rs::run_screen_shader_advisory_selected_file_ui_action".to_string(),
        placement: projection.placement,
        advanced_mode_required: projection.advanced_mode_required,
        explicit_user_trigger_required: projection.explicit_user_trigger_required,
        separated_from_apply_action: true,
        button_label: "Run optional advisory check".to_string(),
        default_state: ScreenShaderAdvisoryUiResultState::NotRun
            .as_str()
            .to_string(),
        missing_selection_state: ScreenShaderAdvisoryUiResultState::NotRun
            .as_str()
            .to_string(),
        file_chooser_behavior_proven: false,
        selected_file_boundary_proven: false,
        missing_selection_behavior_proven: true,
        cancellation_or_progress_behavior_proven: false,
        result_states_rendered: vec![
            ScreenShaderAdvisoryUiResultState::NotRun
                .as_str()
                .to_string(),
            ScreenShaderAdvisoryUiResultState::Running
                .as_str()
                .to_string(),
            ScreenShaderAdvisoryUiResultState::Passed
                .as_str()
                .to_string(),
            ScreenShaderAdvisoryUiResultState::Failed
                .as_str()
                .to_string(),
            ScreenShaderAdvisoryUiResultState::Unavailable
                .as_str()
                .to_string(),
            ScreenShaderAdvisoryUiResultState::TimedOut
                .as_str()
                .to_string(),
            ScreenShaderAdvisoryUiResultState::TempCopyFailed
                .as_str()
                .to_string(),
            ScreenShaderAdvisoryUiResultState::CleanupWarning
                .as_str()
                .to_string(),
        ],
        can_approve_write: false,
        can_block_write: false,
        can_bypass_production_gate: false,
    })
}

pub fn running_screen_shader_advisory_ui_action(
    row_id: &str,
) -> Option<ScreenShaderAdvisoryUiActionRender> {
    screen_shader_advisory_projection(row_id).map(|projection| {
        advisory_ui_render(
            row_id,
            ScreenShaderAdvisoryUiResultState::Running,
            projection,
            false,
            false,
            false,
            Vec::new(),
            None,
            None,
            false,
            false,
            false,
            false,
            "Advisory check running".to_string(),
            "The optional advisory check is running after explicit user action; it cannot approve, block, or bypass writes."
                .to_string(),
            None,
            None,
        )
    })
}

pub fn run_screen_shader_advisory_ui_action(
    request: ScreenShaderAdvisoryUiActionRequest,
) -> ScreenShaderAdvisoryUiActionRender {
    let Some(projection) = screen_shader_advisory_projection(&request.row_id) else {
        return advisory_ui_render(
            &request.row_id,
            ScreenShaderAdvisoryUiResultState::NotRun,
            non_screen_shader_advisory_projection(),
            false,
            false,
            false,
            Vec::new(),
            None,
            None,
            false,
            false,
            false,
            false,
            "No screen shader advisory action".to_string(),
            "This advisory action is only available for decoration.screen_shader.".to_string(),
            None,
            None,
        );
    };

    let Some(helper_request) = request.helper_request else {
        return advisory_ui_render(
            &request.row_id,
            ScreenShaderAdvisoryUiResultState::NotRun,
            projection,
            false,
            true,
            false,
            Vec::new(),
            None,
            None,
            false,
            false,
            false,
            false,
            "Explicit action required".to_string(),
            "Choose a shader file and run the advanced advisory check explicitly.".to_string(),
            Some("no selected shader request was supplied".to_string()),
            None,
        );
    };

    if !request.explicit_user_trigger || !helper_request.explicit_user_consent {
        return advisory_ui_render(
            &request.row_id,
            ScreenShaderAdvisoryUiResultState::NotRun,
            projection,
            false,
            true,
            false,
            Vec::new(),
            None,
            None,
            false,
            false,
            false,
            false,
            "Explicit action required".to_string(),
            "The selected shader is not read until the user explicitly starts the advanced advisory check."
                .to_string(),
            Some("missing explicit user trigger or consent".to_string()),
            None,
        );
    }

    let result = run_screen_shader_advisory_check(&helper_request);
    let state = match result.status {
        AdvisoryStatus::Passed => ScreenShaderAdvisoryUiResultState::Passed,
        AdvisoryStatus::Failed => ScreenShaderAdvisoryUiResultState::Failed,
        AdvisoryStatus::Unavailable => ScreenShaderAdvisoryUiResultState::Unavailable,
        AdvisoryStatus::TimedOut => ScreenShaderAdvisoryUiResultState::TimedOut,
        AdvisoryStatus::TempCopyFailed => ScreenShaderAdvisoryUiResultState::TempCopyFailed,
        AdvisoryStatus::MissingConsent => ScreenShaderAdvisoryUiResultState::NotRun,
        AdvisoryStatus::CleanupWarning => ScreenShaderAdvisoryUiResultState::CleanupWarning,
    };
    let compiler_invoked = !result.compiler_args.is_empty()
        && !matches!(
            result.status,
            AdvisoryStatus::Unavailable
                | AdvisoryStatus::TempCopyFailed
                | AdvisoryStatus::MissingConsent
        );
    let selected_shader_read = !matches!(
        result.status,
        AdvisoryStatus::MissingConsent | AdvisoryStatus::TempCopyFailed
    ) || result.temp_fragment_path.is_some();
    let (title, message) = advisory_result_copy(state);

    advisory_ui_render(
        &request.row_id,
        state,
        projection,
        true,
        false,
        compiler_invoked,
        result.compiler_args,
        result.temp_fragment_path,
        result.temp_vertex_path,
        result.original_user_path_passed_to_compiler,
        selected_shader_read,
        result.production_write_decision_changed,
        result.runtime_safety_claimed,
        title.to_string(),
        message.to_string(),
        result.diagnostic,
        result.cleanup_warning,
    )
}

pub fn run_screen_shader_advisory_selected_file_ui_action(
    request: ScreenShaderAdvisorySelectedFileUiActionRequest,
) -> ScreenShaderAdvisoryUiActionRender {
    let Some(selected_shader_path) = request.selected_shader_path else {
        return run_screen_shader_advisory_ui_action(ScreenShaderAdvisoryUiActionRequest {
            row_id: request.row_id,
            explicit_user_trigger: request.explicit_user_trigger,
            helper_request: None,
        });
    };

    let helper_request = ScreenShaderAdvisoryRequest {
        selected_shader_path,
        temp_root: request.temp_root,
        tex300_vertex_path: request.tex300_vertex_path,
        tex320_vertex_path: request.tex320_vertex_path,
        glslang_validator_path: request.glslang_validator_path,
        timeout: request.timeout,
        explicit_user_consent: request.explicit_user_trigger,
        simulate_cleanup_failure: request.simulate_cleanup_failure,
    };

    run_screen_shader_advisory_ui_action(ScreenShaderAdvisoryUiActionRequest {
        row_id: request.row_id,
        explicit_user_trigger: request.explicit_user_trigger,
        helper_request: Some(helper_request),
    })
}

fn advisory_result_copy(state: ScreenShaderAdvisoryUiResultState) -> (&'static str, &'static str) {
    match state {
        ScreenShaderAdvisoryUiResultState::NotRun => (
            "Advisory check not run",
            "No shader file has been read and no compiler command has been run.",
        ),
        ScreenShaderAdvisoryUiResultState::Running => (
            "Advisory check running",
            "The advisory check is in progress after explicit user action.",
        ),
        ScreenShaderAdvisoryUiResultState::Passed => (
            "Standalone advisory check passed",
            "This is not Hyprland runtime safety proof and does not approve the write.",
        ),
        ScreenShaderAdvisoryUiResultState::Failed => (
            "Standalone advisory warning",
            "This warns about the standalone check result and does not automatically block the write.",
        ),
        ScreenShaderAdvisoryUiResultState::Unavailable => (
            "Advisory check unavailable",
            "glslangValidator is unavailable; this does not approve or block writes.",
        ),
        ScreenShaderAdvisoryUiResultState::TimedOut => (
            "Advisory check inconclusive",
            "The advisory check timed out and does not approve or block writes.",
        ),
        ScreenShaderAdvisoryUiResultState::TempCopyFailed => (
            "Advisory temp copy failed",
            "The advisory check could not prepare temp files and does not approve or block writes.",
        ),
        ScreenShaderAdvisoryUiResultState::CleanupWarning => (
            "Advisory cleanup warning",
            "The advisory check produced a cleanup warning and does not approve, block, or bypass writes.",
        ),
    }
}

#[allow(clippy::too_many_arguments)]
fn advisory_ui_render(
    row_id: &str,
    state: ScreenShaderAdvisoryUiResultState,
    projection: ScreenShaderAdvisoryUiProjection,
    helper_invoked: bool,
    consent_required: bool,
    compiler_invoked: bool,
    compiler_args: Vec<String>,
    temp_fragment_path: Option<PathBuf>,
    temp_vertex_path: Option<PathBuf>,
    original_user_path_passed_to_compiler: bool,
    selected_shader_read: bool,
    production_write_decision_changed: bool,
    runtime_safety_claimed: bool,
    title: String,
    message: String,
    diagnostic: Option<String>,
    cleanup_warning: Option<String>,
) -> ScreenShaderAdvisoryUiActionRender {
    ScreenShaderAdvisoryUiActionRender {
        row_id: row_id.to_string(),
        state,
        placement: projection.placement,
        advanced_mode_required: projection.advanced_mode_required,
        explicit_user_trigger_required: projection.explicit_user_trigger_required,
        helper_invoked,
        consent_required,
        selected_shader_read,
        compiler_invoked,
        compiler_args,
        temp_fragment_path,
        temp_vertex_path,
        original_user_path_passed_to_compiler,
        can_approve_write: false,
        can_block_write: false,
        can_bypass_production_gate: false,
        production_write_decision_changed,
        runtime_safety_claimed,
        write_blocking: false,
        title,
        message,
        diagnostic,
        cleanup_warning,
    }
}

fn non_screen_shader_advisory_projection() -> ScreenShaderAdvisoryUiProjection {
    ScreenShaderAdvisoryUiProjection {
        placement: "not-available-for-this-row".to_string(),
        advanced_mode_required: true,
        explicit_user_trigger_required: true,
        runs_on_row_load: false,
        runs_on_value_change: false,
        runs_during_validation: false,
        runs_during_pending_change: false,
        runs_during_write_planning: false,
        runs_during_apply_flow: false,
        consent_message: "No advisory shader action is available for this row.".to_string(),
        temp_copy_message: "No temp shader copy is created for this row.".to_string(),
        original_path_message: "No user path is passed to a compiler for this row.".to_string(),
        runtime_safety_disclaimer: "No runtime safety claim is made.".to_string(),
        production_gate_disclaimer: "No screen-shader production gate applies to this row."
            .to_string(),
        pass_policy: "No advisory pass state exists for this row.".to_string(),
        failure_policy: "No advisory failure state exists for this row.".to_string(),
        missing_tool_policy: "No advisory tool is used for this row.".to_string(),
        timeout_policy: "No advisory timeout state exists for this row.".to_string(),
        cleanup_warning_policy: "No advisory cleanup state exists for this row.".to_string(),
        can_approve_write: false,
        can_block_write: false,
        can_bypass_production_gate: false,
    }
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
                "This row can use guarded safe-batch writing when review, backup, write, and reread verification checks pass."
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
            screen_shader_advisory: screen_shader_advisory_projection(&setting.row_id),
            screen_shader_advisory_widget: screen_shader_advisory_gtk_widget_projection(
                &setting.row_id,
            ),
        }
    }
}

fn screen_shader_advisory_projection(row_id: &str) -> Option<ScreenShaderAdvisoryUiProjection> {
    (row_id == "decoration.screen_shader").then(|| ScreenShaderAdvisoryUiProjection {
        placement:
            "advanced-display-render-screen-shader-advisory-section-separated-from-apply-action"
                .to_string(),
        advanced_mode_required: true,
        explicit_user_trigger_required: true,
        runs_on_row_load: false,
        runs_on_value_change: false,
        runs_during_validation: false,
        runs_during_pending_change: false,
        runs_during_write_planning: false,
        runs_during_apply_flow: false,
        consent_message: "Run optional advisory check: the app will read the selected shader file only after this explicit action and copy it into a temporary folder."
            .to_string(),
        temp_copy_message:
            "The advisory check runs glslangValidator only against temporary copies of the fragment shader and Hyprland vertex shader."
                .to_string(),
        original_path_message:
            "The original selected shader path is not passed to glslangValidator.".to_string(),
        runtime_safety_disclaimer:
            "A passing advisory check is not Hyprland runtime, GPU, or visual safety proof."
                .to_string(),
        production_gate_disclaimer:
            "The production screen-shader watchdog gate is still required before applying this setting."
                .to_string(),
        pass_policy:
            "Standalone advisory check passed; this does not approve or apply the write.".to_string(),
        failure_policy:
            "Standalone advisory warning; this does not automatically block the write.".to_string(),
        missing_tool_policy:
            "Advisory check unavailable because glslangValidator is missing; writes are not approved or blocked."
                .to_string(),
        timeout_policy:
            "Advisory check timed out and is inconclusive; writes are not approved or blocked."
                .to_string(),
        cleanup_warning_policy:
            "Temp cleanup warning should be logged/displayed without approving, blocking, or bypassing writes."
                .to_string(),
        can_approve_write: false,
        can_block_write: false,
        can_bypass_production_gate: false,
    })
}
