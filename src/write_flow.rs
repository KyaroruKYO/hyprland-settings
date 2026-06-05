use std::collections::BTreeSet;
use std::path::PathBuf;

use anyhow::Result;

use crate::config_backup::BackupManager;
use crate::config_discovery::{ConfigDiscovery, ConfigDiscoveryStatus};
use crate::current_config::{
    CurrentConfigSnapshot, CurrentValueProjection, CurrentValueSourceStatus,
};
use crate::pending_change::{stage_pending_change, PendingChange, PendingChangeValidation};
use crate::scalar_write::apply_scalar_write_plan;
use crate::write_classification::{
    finite_choice_options, is_safe_writable_setting, safe_writable_official_setting,
    safe_writable_value_kind, ScalarWriteValueKind,
};
use crate::write_safety::{review_write_plan, WriteGateFailure, WritePlanRequest, WriteResult};

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SettingEditProjection {
    pub setting_id: String,
    pub editable: bool,
    pub editor_kind: String,
    pub choices: Vec<FiniteChoiceEditOption>,
    pub disabled_reason: Option<String>,
    pub proposed_value: Option<String>,
    pub pending: Option<PendingChangeProjection>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct FiniteChoiceEditOption {
    pub raw_value: String,
    pub label: String,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PendingChangeProjection {
    pub setting_id: String,
    pub old_value: Option<String>,
    pub proposed_value: String,
    pub validation_label: String,
    pub can_review: bool,
    pub review_summary: Vec<String>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ApplyOutcome {
    pub setting_id: String,
    pub target_path: PathBuf,
    pub backup_path: PathBuf,
    pub rollback_source_path: PathBuf,
    pub rollback_backup_path: PathBuf,
    pub verified_value: Option<String>,
    pub reload_note: String,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ApplyFailure {
    pub reason: String,
    pub failures: Vec<String>,
}

pub fn edit_projection_for_setting(
    setting_id: &str,
    current_value: &CurrentValueProjection,
) -> SettingEditProjection {
    if !is_safe_writable_setting(setting_id) {
        return SettingEditProjection {
            setting_id: setting_id.to_string(),
            editable: false,
            editor_kind: "disabled".to_string(),
            choices: Vec::new(),
            disabled_reason: Some("not write-allowlisted".to_string()),
            proposed_value: None,
            pending: None,
        };
    }

    let value_kind = safe_writable_value_kind(setting_id);
    let proposed_value = next_proposed_value(setting_id, current_value);
    let pending = stage_pending_change(setting_id, current_value, proposed_value.clone());
    SettingEditProjection {
        setting_id: setting_id.to_string(),
        editable: true,
        editor_kind: editor_kind_for_value_kind(value_kind).to_string(),
        choices: finite_choice_edit_options(setting_id),
        disabled_reason: None,
        proposed_value: Some(proposed_value),
        pending: Some(pending_projection(&pending, current_value.status)),
    }
}

pub fn pending_projection_for_value(
    setting_id: &str,
    current_value: &CurrentValueProjection,
    proposed_value: &str,
) -> PendingChangeProjection {
    let pending = stage_pending_change(setting_id, current_value, proposed_value);
    pending_projection(&pending, current_value.status)
}

pub fn apply_setting_change(
    known_setting_ids: BTreeSet<String>,
    discovery: &ConfigDiscovery,
    current_config: &CurrentConfigSnapshot,
    setting_id: &str,
    proposed_value: &str,
) -> Result<ApplyOutcome, ApplyFailure> {
    let backup_root = BackupManager::default_user_backup_root().map_err(|error| ApplyFailure {
        reason: error.to_string(),
        failures: vec!["MissingBackup".to_string()],
    })?;
    apply_setting_change_with_backup_manager(
        known_setting_ids,
        discovery,
        current_config,
        setting_id,
        proposed_value,
        &BackupManager::new(backup_root),
    )
}

pub fn apply_setting_change_with_backup_manager(
    known_setting_ids: BTreeSet<String>,
    discovery: &ConfigDiscovery,
    current_config: &CurrentConfigSnapshot,
    setting_id: &str,
    proposed_value: &str,
    backup_manager: &BackupManager,
) -> Result<ApplyOutcome, ApplyFailure> {
    if !is_safe_writable_setting(setting_id) {
        return Err(ApplyFailure {
            reason: "setting is not write-allowlisted".to_string(),
            failures: vec!["NotAllowlisted".to_string()],
        });
    }
    if !known_setting_ids.contains(setting_id) {
        return Err(ApplyFailure {
            reason: "setting is not known to the export inventory".to_string(),
            failures: vec!["UnknownSetting".to_string()],
        });
    }

    let target_path = detected_config_path(discovery).map_err(|reason| ApplyFailure {
        reason,
        failures: vec!["MissingCurrentSource".to_string()],
    })?;
    let official_setting =
        safe_writable_official_setting(setting_id).ok_or_else(|| ApplyFailure {
            reason: "setting is not write-allowlisted".to_string(),
            failures: vec!["NotAllowlisted".to_string()],
        })?;
    let current_value = current_config.value_for(official_setting);
    let pending_change = stage_pending_change(setting_id, &current_value, proposed_value);
    match &pending_change.validation {
        PendingChangeValidation::Valid => {}
        PendingChangeValidation::Invalid { reason }
        | PendingChangeValidation::NotAllowed { reason } => {
            return Err(ApplyFailure {
                reason: reason.clone(),
                failures: vec!["InvalidProposedValue".to_string()],
            });
        }
    }
    if let Some(reason) = review_block_reason(current_value.status) {
        let failure = match current_value.status {
            CurrentValueSourceStatus::DuplicateConflict => "DuplicateConflict",
            CurrentValueSourceStatus::ReadUnavailable => "MissingCurrentSource",
            CurrentValueSourceStatus::Configured | CurrentValueSourceStatus::NotConfigured => {
                "MissingCurrentSource"
            }
        };
        return Err(ApplyFailure {
            reason: reason.to_string(),
            failures: vec![failure.to_string()],
        });
    }

    let backup = backup_manager
        .create_backup(&target_path)
        .map_err(|error| ApplyFailure {
            reason: error.to_string(),
            failures: vec!["MissingBackup".to_string()],
        })?;

    let review = review_write_plan(WritePlanRequest {
        known_setting_ids,
        detected_config_path: target_path.clone(),
        current_value,
        pending_change,
        backup: Some(backup),
    });
    if !review.is_approved() {
        return Err(ApplyFailure {
            reason: "write plan rejected by safety gates".to_string(),
            failures: review.failures.iter().map(format_gate_failure).collect(),
        });
    }

    let result = apply_scalar_write_plan(
        &review
            .plan
            .clone()
            .expect("approved review should include a plan"),
    )
    .map_err(|error| ApplyFailure {
        reason: error.to_string(),
        failures: vec!["WriteFailed".to_string()],
    })?;

    Ok(outcome_from_result(result))
}

fn detected_config_path(discovery: &ConfigDiscovery) -> Result<PathBuf, String> {
    match &discovery.status {
        ConfigDiscoveryStatus::Found { path, .. } => Ok(path.clone()),
        ConfigDiscoveryStatus::Missing => Err("no Hyprland config file was detected".to_string()),
        ConfigDiscoveryStatus::Unreadable { path, error, .. } => Err(format!(
            "Hyprland config is not readable: {}: {error}",
            path.display()
        )),
        ConfigDiscoveryStatus::NotAFile { path, .. } => Err(format!(
            "Hyprland config target is not a regular file: {}",
            path.display()
        )),
    }
}

fn pending_projection(
    pending: &PendingChange,
    current_status: CurrentValueSourceStatus,
) -> PendingChangeProjection {
    let validation_label = match &pending.validation {
        PendingChangeValidation::Valid => "valid".to_string(),
        PendingChangeValidation::Invalid { reason } => format!("invalid: {reason}"),
        PendingChangeValidation::NotAllowed { reason } => format!("not allowed: {reason}"),
    };
    let mut review_summary = vec![
        format!("setting: {}", pending.setting_id),
        format!(
            "current: {}",
            pending
                .old_parsed_value
                .clone()
                .unwrap_or_else(|| "not configured".to_string())
        ),
        format!("proposed: {}", pending.proposed_value),
        format!("validation: {validation_label}"),
    ];
    if let Some(source) = &pending.source {
        review_summary.push(format!(
            "source: {}:{}",
            source.path.display(),
            source.line_number
        ));
    }
    if let Some(reason) = &pending.non_editable_reason {
        review_summary.push(format!("blocked: {reason}"));
    }
    if let Some(reason) = review_block_reason(current_status) {
        review_summary.push(format!("blocked: {reason}"));
    }

    PendingChangeProjection {
        setting_id: pending.setting_id.clone(),
        old_value: pending.old_parsed_value.clone(),
        proposed_value: pending.proposed_value.clone(),
        validation_label,
        can_review: pending.can_be_applied() && status_allows_review(current_status),
        review_summary,
    }
}

fn next_proposed_value(setting_id: &str, current_value: &CurrentValueProjection) -> String {
    match safe_writable_value_kind(setting_id) {
        Some(ScalarWriteValueKind::Boolean) => next_bool_value(current_value),
        Some(ScalarWriteValueKind::FiniteChoice) => {
            next_finite_choice_value(setting_id, current_value)
        }
        Some(ScalarWriteValueKind::Number) => current_value
            .raw_value
            .clone()
            .filter(|value| !value.trim().is_empty())
            .unwrap_or_else(|| "0".to_string()),
        Some(ScalarWriteValueKind::Percent) => current_value
            .raw_value
            .clone()
            .filter(|value| !value.trim().is_empty())
            .unwrap_or_else(|| {
                if setting_id == "input.pointer_sensitivity" {
                    "0".to_string()
                } else {
                    "1.0".to_string()
                }
            }),
        Some(ScalarWriteValueKind::Color) => current_value
            .raw_value
            .clone()
            .filter(|value| !value.trim().is_empty())
            .unwrap_or_else(|| "rgba(ffffffff)".to_string()),
        Some(ScalarWriteValueKind::Gradient) => current_value
            .raw_value
            .clone()
            .filter(|value| !value.trim().is_empty())
            .unwrap_or_else(|| "rgba(ffffffff) rgba(000000ff) 45deg".to_string()),
        Some(ScalarWriteValueKind::Vector2) => current_value
            .raw_value
            .clone()
            .filter(|value| !value.trim().is_empty())
            .unwrap_or_else(|| "0 0".to_string()),
        Some(ScalarWriteValueKind::NumericList) => current_value
            .raw_value
            .clone()
            .filter(|value| !value.trim().is_empty())
            .unwrap_or_else(|| "0.2 0.0 0.5 1 1.2 1.5".to_string()),
        Some(ScalarWriteValueKind::LineSafeString) => current_value
            .raw_value
            .clone()
            .filter(|value| !value.trim().is_empty())
            .unwrap_or_else(|| {
                if setting_id == "input.accel_profile" {
                    "flat".to_string()
                } else {
                    "Sans".to_string()
                }
            }),
        Some(ScalarWriteValueKind::Path) => current_value
            .raw_value
            .clone()
            .filter(|value| !value.trim().is_empty())
            .unwrap_or_else(|| "~/.config/hypr/example.conf".to_string()),
        Some(ScalarWriteValueKind::RegexString) => current_value
            .raw_value
            .clone()
            .filter(|value| !value.trim().is_empty())
            .unwrap_or_else(|| "^(Alacritty|kitty)$".to_string()),
        Some(ScalarWriteValueKind::StringLike)
        | Some(ScalarWriteValueKind::ComplexRaw)
        | Some(ScalarWriteValueKind::Unknown)
        | None => current_value.raw_value.clone().unwrap_or_default(),
    }
}

fn editor_kind_for_value_kind(value_kind: Option<ScalarWriteValueKind>) -> &'static str {
    match value_kind {
        Some(ScalarWriteValueKind::Boolean) => "toggle",
        Some(ScalarWriteValueKind::FiniteChoice) => "dropdown",
        Some(ScalarWriteValueKind::Number) | Some(ScalarWriteValueKind::Percent) => "number",
        Some(ScalarWriteValueKind::Color) => "color-text",
        Some(ScalarWriteValueKind::Gradient) => "gradient-text",
        Some(ScalarWriteValueKind::Vector2) => "vector-text",
        Some(ScalarWriteValueKind::NumericList) => "numeric-list-text",
        Some(ScalarWriteValueKind::LineSafeString)
        | Some(ScalarWriteValueKind::Path)
        | Some(ScalarWriteValueKind::RegexString) => "text",
        Some(ScalarWriteValueKind::StringLike)
        | Some(ScalarWriteValueKind::ComplexRaw)
        | Some(ScalarWriteValueKind::Unknown)
        | None => "unknown",
    }
}

fn finite_choice_edit_options(setting_id: &str) -> Vec<FiniteChoiceEditOption> {
    finite_choice_options(setting_id)
        .unwrap_or(&[])
        .iter()
        .map(|option| FiniteChoiceEditOption {
            raw_value: option.raw_value.to_string(),
            label: option.label.to_string(),
        })
        .collect()
}

fn next_finite_choice_value(setting_id: &str, current_value: &CurrentValueProjection) -> String {
    let Some(options) = finite_choice_options(setting_id) else {
        return current_value.raw_value.clone().unwrap_or_default();
    };
    if options.is_empty() {
        return current_value.raw_value.clone().unwrap_or_default();
    }
    let current = current_value.raw_value.as_deref().map(str::trim);
    if let Some((index, _)) = current.and_then(|value| {
        options
            .iter()
            .enumerate()
            .find(|(_, option)| option.raw_value == value)
    }) {
        return options[(index + 1) % options.len()].raw_value.to_string();
    }
    options[0].raw_value.to_string()
}

fn next_bool_value(current_value: &CurrentValueProjection) -> String {
    match current_value
        .raw_value
        .as_deref()
        .map(normalize_bool_literal)
    {
        Some(Some(true)) => "false".to_string(),
        Some(Some(false)) => "true".to_string(),
        _ => "true".to_string(),
    }
}

fn normalize_bool_literal(value: &str) -> Option<bool> {
    match value.trim().to_ascii_lowercase().as_str() {
        "true" | "1" | "yes" | "on" => Some(true),
        "false" | "0" | "no" | "off" => Some(false),
        _ => None,
    }
}

fn outcome_from_result(result: WriteResult) -> ApplyOutcome {
    ApplyOutcome {
        setting_id: result.plan.setting_id,
        target_path: result.plan.target_path,
        backup_path: result.plan.backup_path.clone(),
        rollback_source_path: result.plan.rollback.source_path,
        rollback_backup_path: result.plan.rollback.backup_path,
        verified_value: result.verified_value,
        reload_note: "Hyprland reload is not performed by this app yet.".to_string(),
    }
}

fn format_gate_failure(failure: &WriteGateFailure) -> String {
    match failure {
        WriteGateFailure::UnknownSetting => "UnknownSetting".to_string(),
        WriteGateFailure::NotAllowlisted => "NotAllowlisted".to_string(),
        WriteGateFailure::InvalidProposedValue(reason) => {
            format!("InvalidProposedValue: {reason}")
        }
        WriteGateFailure::MissingCurrentSource => "MissingCurrentSource".to_string(),
        WriteGateFailure::DuplicateConflict => "DuplicateConflict".to_string(),
        WriteGateFailure::MissingBackup => "MissingBackup".to_string(),
        WriteGateFailure::BackupTargetMismatch => "BackupTargetMismatch".to_string(),
        WriteGateFailure::TargetMismatch => "TargetMismatch".to_string(),
        WriteGateFailure::StructuredFamilyRejected => "StructuredFamilyRejected".to_string(),
    }
}

pub fn write_flow_config_setting(setting_id: &str) -> Option<&'static str> {
    safe_writable_official_setting(setting_id)
}

pub fn write_flow_value_kind(setting_id: &str) -> Option<ScalarWriteValueKind> {
    safe_writable_value_kind(setting_id)
}

pub fn status_allows_review(status: CurrentValueSourceStatus) -> bool {
    matches!(
        status,
        CurrentValueSourceStatus::Configured | CurrentValueSourceStatus::NotConfigured
    )
}

pub fn review_block_reason(status: CurrentValueSourceStatus) -> Option<&'static str> {
    match status {
        CurrentValueSourceStatus::Configured | CurrentValueSourceStatus::NotConfigured => None,
        CurrentValueSourceStatus::DuplicateConflict => {
            Some("duplicate config entries must be resolved manually before writing")
        }
        CurrentValueSourceStatus::ReadUnavailable => Some("current config could not be read"),
    }
}
