use std::collections::BTreeSet;
use std::path::PathBuf;

use anyhow::Result;

use crate::config_backup::BackupManager;
use crate::config_discovery::{ConfigDiscovery, ConfigDiscoveryStatus};
use crate::current_config::{
    CurrentConfigSnapshot, CurrentValueProjection, CurrentValueSourceStatus,
};
use crate::pending_change::{
    stage_pending_change, PendingChange, PendingChangeValidation, ACTIVE_PENDING_CHANGE_SETTING,
};
use crate::write_pilot::apply_windows_snap_enabled_plan;
use crate::write_safety::{review_write_plan, WriteGateFailure, WritePlanRequest, WriteResult};

const WINDOWS_SNAP_CONFIG_SETTING: &str = "general.snap.enabled";

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SettingEditProjection {
    pub setting_id: String,
    pub editable: bool,
    pub disabled_reason: Option<String>,
    pub proposed_value: Option<String>,
    pub pending: Option<PendingChangeProjection>,
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
    if setting_id != ACTIVE_PENDING_CHANGE_SETTING {
        return SettingEditProjection {
            setting_id: setting_id.to_string(),
            editable: false,
            disabled_reason: Some("not write-allowlisted".to_string()),
            proposed_value: None,
            pending: None,
        };
    }

    let proposed_value = next_bool_value(current_value);
    let pending = stage_pending_change(setting_id, current_value, proposed_value.clone());
    SettingEditProjection {
        setting_id: setting_id.to_string(),
        editable: true,
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
    if setting_id != ACTIVE_PENDING_CHANGE_SETTING {
        return Err(ApplyFailure {
            reason: "setting is not write-allowlisted".to_string(),
            failures: vec!["NotAllowlisted".to_string()],
        });
    }

    let target_path = detected_config_path(discovery).map_err(|reason| ApplyFailure {
        reason,
        failures: vec!["MissingCurrentSource".to_string()],
    })?;
    let current_value = current_config.value_for(WINDOWS_SNAP_CONFIG_SETTING);
    let pending_change = stage_pending_change(setting_id, &current_value, proposed_value);

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

    let result = apply_windows_snap_enabled_plan(
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
    (setting_id == ACTIVE_PENDING_CHANGE_SETTING).then_some(WINDOWS_SNAP_CONFIG_SETTING)
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
