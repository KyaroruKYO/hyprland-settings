use std::str;

use anyhow::{anyhow, Result};
use thiserror::Error;

use crate::config_backup::BackupManager;
use crate::config_parser::{parse_hyprland_config_text, ParsedConfigLine};
use crate::durable_fs::{hardened_atomic_replace, DurableFsError};
use crate::write_classification::{
    config_key_from_official_setting, safe_writable_official_setting,
};
use crate::write_safety::{WritePlan, WritePlanAction, WriteResult};

#[derive(Debug, Error)]
pub enum ScalarWriteError {
    #[error("{0}")]
    Drift(#[from] DurableFsError),
    #[error("ExpectedValueChanged: {0}")]
    ExpectedValueChanged(String),
    #[error("ExpectedSettingNowPresent: {0}")]
    ExpectedSettingNowPresent(String),
    #[error("ExpectedRecordChanged: {0}")]
    ExpectedRecordChanged(String),
    #[error("SourceGraphChanged: {0}")]
    SourceGraphChanged(String),
    #[error("InvalidWritePlan: {0}")]
    InvalidWritePlan(String),
    #[error("PostWriteVerificationFailed: {message}; restored={restored}")]
    PostWriteVerificationFailed { message: String, restored: bool },
    #[error("RestoreFailed: {0}")]
    RestoreFailed(String),
}

impl ScalarWriteError {
    pub fn code(&self) -> &'static str {
        match self {
            Self::Drift(error) => error.code(),
            Self::ExpectedValueChanged(_) => "ExpectedValueChanged",
            Self::ExpectedSettingNowPresent(_) => "ExpectedSettingNowPresent",
            Self::ExpectedRecordChanged(_) => "ExpectedRecordChanged",
            Self::SourceGraphChanged(_) => "SourceGraphChanged",
            Self::InvalidWritePlan(_) => "InvalidWritePlan",
            Self::PostWriteVerificationFailed { .. } => "PostWriteVerificationFailed",
            Self::RestoreFailed(_) => "RestoreFailed",
        }
    }

    pub fn user_message(&self) -> &'static str {
        match self {
            Self::Drift(error) => error.user_message(),
            Self::ExpectedValueChanged(_)
            | Self::ExpectedSettingNowPresent(_)
            | Self::ExpectedRecordChanged(_)
            | Self::SourceGraphChanged(_) => {
                "The config changed on disk after this edit was prepared. Nothing was written. Reload or reread the setting before saving again."
            }
            _ => "The config write failed. The setting remains pending and recovery remains available.",
        }
    }
}

/// Read-only: what the config text would become if this change were written.
/// This helper never touches a file.
pub fn preview_scalar_change_text(
    contents: &str,
    setting_id: &str,
    line_number: Option<usize>,
    proposed_value: &str,
) -> Result<String> {
    let official_setting = safe_writable_official_setting(setting_id)
        .ok_or_else(|| anyhow!("setting is not safe-writable: {setting_id}"))?;
    let config_key = config_key_from_official_setting(official_setting);
    match line_number {
        Some(line) => replace_line_value(contents, line, &config_key, proposed_value),
        None => Ok(append_scalar_setting(contents, &config_key, proposed_value)),
    }
}

fn validate_source_graph_precondition(plan: &WritePlan) -> Result<(), ScalarWriteError> {
    match (
        plan.source_graph_root.as_deref(),
        plan.source_graph_fingerprint.as_deref(),
    ) {
        (None, None) => Ok(()),
        (Some(root), Some(expected)) => {
            let actual = crate::source_aware_current_config::current_source_graph_fingerprint(root)
                .ok_or_else(|| {
                    ScalarWriteError::SourceGraphChanged(
                        "source graph could not be reproduced".to_string(),
                    )
                })?;
            if actual == expected {
                Ok(())
            } else {
                Err(ScalarWriteError::SourceGraphChanged(
                    "source/include mapping or connected file bytes changed".to_string(),
                ))
            }
        }
        _ => Err(ScalarWriteError::SourceGraphChanged(
            "source graph evidence is incomplete".to_string(),
        )),
    }
}

pub fn apply_scalar_write_plan(plan: &WritePlan) -> Result<WriteResult, ScalarWriteError> {
    let official_setting = safe_writable_official_setting(&plan.setting_id).ok_or_else(|| {
        ScalarWriteError::InvalidWritePlan(format!(
            "setting is not safe-writable: {}",
            plan.setting_id
        ))
    })?;
    let config_key = config_key_from_official_setting(official_setting);
    validate_source_graph_precondition(plan)?;
    let original = str::from_utf8(&plan.target_precondition.bytes).map_err(|error| {
        ScalarWriteError::InvalidWritePlan(format!("target is not valid UTF-8: {error}"))
    })?;
    validate_expected_record_shape(plan, original, official_setting)?;

    let updated = match plan.action {
        WritePlanAction::ReplaceLine { line_number } => {
            replace_line_value(original, line_number, &config_key, &plan.proposed_value)
                .map_err(|error| ScalarWriteError::ExpectedRecordChanged(error.to_string()))?
        }
        WritePlanAction::AppendSetting => {
            append_scalar_setting(original, &config_key, &plan.proposed_value)
        }
    };

    // Recheck the entire source graph after backup creation and immediately
    // before entering the target replacement primitive.
    validate_source_graph_precondition(plan)?;
    let durable_receipt = hardened_atomic_replace(&plan.target_precondition, updated.as_bytes())?;
    let verified = std::fs::read_to_string(&plan.target_path).map_err(|error| {
        restore_after_semantic_failure(plan, updated.as_bytes(), format!("reread failed: {error}"))
    })?;
    let parsed = parse_hyprland_config_text(&plan.target_path, &verified);
    let matches = parsed
        .scalar_records()
        .filter(|record| record.normalized_setting_id.as_deref() == Some(official_setting))
        .collect::<Vec<_>>();
    let verified_value = matches.last().and_then(|record| record.raw_value.clone());

    if matches.len() != 1 || verified_value.as_deref() != Some(plan.proposed_value.as_str()) {
        return Err(restore_after_semantic_failure(
            plan,
            updated.as_bytes(),
            format!(
                "expected one {} value {}, got count {} and value {:?}",
                plan.setting_id,
                plan.proposed_value,
                matches.len(),
                verified_value
            ),
        ));
    }

    Ok(WriteResult {
        plan: plan.clone(),
        verified_value,
        durable_receipt,
    })
}

fn validate_expected_record_shape(
    plan: &WritePlan,
    original: &str,
    official_setting: &str,
) -> Result<(), ScalarWriteError> {
    let parsed = parse_hyprland_config_text(&plan.target_path, original);
    let matches = parsed
        .scalar_records()
        .filter(|record| record.normalized_setting_id.as_deref() == Some(official_setting))
        .collect::<Vec<_>>();
    if matches.len() != plan.expected_occurrence_count {
        return if plan.expected_occurrence_count == 0 && !matches.is_empty() {
            Err(ScalarWriteError::ExpectedSettingNowPresent(
                plan.setting_id.clone(),
            ))
        } else {
            Err(ScalarWriteError::ExpectedRecordChanged(format!(
                "{} occurrence count changed from {} to {}",
                plan.setting_id,
                plan.expected_occurrence_count,
                matches.len()
            )))
        };
    }
    if let WritePlanAction::ReplaceLine { line_number } = plan.action {
        let Some(record) = matches.first() else {
            return Err(ScalarWriteError::ExpectedRecordChanged(
                plan.setting_id.clone(),
            ));
        };
        if record.line_number != line_number
            || plan.expected_raw_line.as_deref() != Some(record.raw_line.as_str())
        {
            return Err(ScalarWriteError::ExpectedRecordChanged(format!(
                "{} source record moved or changed",
                plan.setting_id
            )));
        }
        if record.raw_value.as_deref() != plan.old_value.as_deref() {
            return Err(ScalarWriteError::ExpectedValueChanged(
                plan.setting_id.clone(),
            ));
        }
    }
    Ok(())
}

fn restore_after_semantic_failure(
    plan: &WritePlan,
    committed_bytes: &[u8],
    message: String,
) -> ScalarWriteError {
    let backup_root = plan
        .backup
        .backup_path
        .parent()
        .unwrap_or_else(|| std::path::Path::new("."));
    match BackupManager::new(backup_root).rollback(&plan.backup, committed_bytes) {
        Ok(_) => ScalarWriteError::PostWriteVerificationFailed {
            message,
            restored: true,
        },
        Err(error) => ScalarWriteError::RestoreFailed(format!(
            "{message}; committed hash {}; restore failed: {error}",
            crate::durable_fs::content_sha256(committed_bytes)
        )),
    }
}

fn replace_line_value(
    contents: &str,
    line_number: usize,
    config_key: &str,
    proposed_value: &str,
) -> Result<String> {
    let had_trailing_newline = contents.ends_with('\n');
    let mut lines: Vec<String> = contents.lines().map(ToOwned::to_owned).collect();
    let index = line_number
        .checked_sub(1)
        .ok_or_else(|| anyhow!("line numbers are 1-based"))?;
    let line = lines
        .get(index)
        .ok_or_else(|| anyhow!("source line {line_number} does not exist"))?;
    ensure_scalar_line(line, config_key)?;
    lines[index] = replace_value_preserving_key(line, proposed_value)?;
    let mut updated = lines.join("\n");
    if had_trailing_newline {
        updated.push('\n');
    }
    Ok(updated)
}

fn append_scalar_setting(contents: &str, config_key: &str, proposed_value: &str) -> String {
    let mut updated = contents.to_string();
    if !updated.is_empty() && !updated.ends_with('\n') {
        updated.push('\n');
    }
    updated.push_str(&format!("{config_key} = {proposed_value}\n"));
    updated
}

fn ensure_scalar_line(line: &str, config_key: &str) -> Result<()> {
    let parsed = parse_hyprland_config_text("line-check.conf", line);
    let official_setting = config_key.replace(':', ".");
    if parsed
        .records
        .iter()
        .any(|record| is_matching_scalar_record(record, &official_setting))
    {
        Ok(())
    } else {
        Err(anyhow!("source line does not parse as {official_setting}"))
    }
}

fn is_matching_scalar_record(record: &ParsedConfigLine, official_setting: &str) -> bool {
    record.normalized_setting_id.as_deref() == Some(official_setting)
}

fn replace_value_preserving_key(line: &str, proposed_value: &str) -> Result<String> {
    let (before_comment, comment) = line
        .split_once('#')
        .map(|(before, comment)| (before, Some(comment)))
        .unwrap_or((line, None));
    let (key, _) = before_comment
        .split_once('=')
        .ok_or_else(|| anyhow!("source line has no scalar assignment"))?;
    let mut replaced = format!("{key}= {proposed_value}");
    if let Some(comment) = comment {
        replaced.push_str(" #");
        replaced.push_str(comment);
    }
    Ok(replaced)
}
