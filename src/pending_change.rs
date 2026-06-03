use std::path::PathBuf;

use crate::current_config::CurrentValueProjection;

pub const ACTIVE_PENDING_CHANGE_SETTING: &str = "windows.snap.enabled";

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PendingChange {
    pub setting_id: String,
    pub old_parsed_value: Option<String>,
    pub proposed_value: String,
    pub validation: PendingChangeValidation,
    pub source: Option<SourceLineRef>,
    pub non_editable_reason: Option<String>,
}

impl PendingChange {
    pub fn can_be_applied(&self) -> bool {
        self.validation == PendingChangeValidation::Valid && self.non_editable_reason.is_none()
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum PendingChangeValidation {
    Valid,
    Invalid { reason: String },
    NotAllowed { reason: String },
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SourceLineRef {
    pub path: PathBuf,
    pub line_number: usize,
    pub raw_line: String,
}

pub fn stage_pending_change(
    setting_id: &str,
    current_value: &CurrentValueProjection,
    proposed_value: impl Into<String>,
) -> PendingChange {
    let proposed_value = proposed_value.into();
    let source = current_value
        .source_path
        .as_ref()
        .zip(current_value.line_number)
        .map(|(path, line_number)| SourceLineRef {
            path: path.clone(),
            line_number,
            raw_line: current_value.raw_line.clone().unwrap_or_default(),
        });

    if setting_id != ACTIVE_PENDING_CHANGE_SETTING {
        return PendingChange {
            setting_id: setting_id.to_string(),
            old_parsed_value: current_value.raw_value.clone(),
            proposed_value,
            validation: PendingChangeValidation::NotAllowed {
                reason: "setting is not pending-change allowlisted".to_string(),
            },
            source,
            non_editable_reason: Some("only windows.snap.enabled can be staged".to_string()),
        };
    }

    let validation = if is_bool_literal(&proposed_value) {
        PendingChangeValidation::Valid
    } else {
        PendingChangeValidation::Invalid {
            reason: "windows.snap.enabled requires a boolean value".to_string(),
        }
    };
    let non_editable_reason = (validation != PendingChangeValidation::Valid)
        .then(|| "proposed value failed validation".to_string());

    PendingChange {
        setting_id: setting_id.to_string(),
        old_parsed_value: current_value.raw_value.clone(),
        proposed_value,
        validation,
        source,
        non_editable_reason,
    }
}

fn is_bool_literal(value: &str) -> bool {
    matches!(
        value.trim().to_ascii_lowercase().as_str(),
        "true" | "false" | "1" | "0" | "yes" | "no" | "on" | "off"
    )
}
