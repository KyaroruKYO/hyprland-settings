use std::path::PathBuf;

use crate::current_config::CurrentValueProjection;
use crate::value::vector::Vec2Value;
use crate::write_classification::{
    is_safe_writable_setting, safe_writable_value_kind, ScalarWriteValueKind,
};

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

    if !is_safe_writable_setting(setting_id) {
        return PendingChange {
            setting_id: setting_id.to_string(),
            old_parsed_value: current_value.raw_value.clone(),
            proposed_value,
            validation: PendingChangeValidation::NotAllowed {
                reason: "setting is not pending-change allowlisted".to_string(),
            },
            source,
            non_editable_reason: Some(
                "setting is not in the safe scalar write allowlist".to_string(),
            ),
        };
    }

    let validation = validate_safe_writable_value(setting_id, &proposed_value);
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

fn validate_safe_writable_value(setting_id: &str, value: &str) -> PendingChangeValidation {
    match safe_writable_value_kind(setting_id) {
        Some(ScalarWriteValueKind::Boolean) => {
            if is_bool_literal(value) {
                PendingChangeValidation::Valid
            } else {
                PendingChangeValidation::Invalid {
                    reason: "safe scalar toggle writes require a boolean value".to_string(),
                }
            }
        }
        Some(ScalarWriteValueKind::Number) => validate_number_setting(setting_id, value),
        Some(ScalarWriteValueKind::Percent) => validate_percent_setting(setting_id, value),
        Some(ScalarWriteValueKind::Color) => validate_color_literal(value),
        Some(ScalarWriteValueKind::Vector2) => validate_vec2_value(value),
        Some(ScalarWriteValueKind::StringLike)
        | Some(ScalarWriteValueKind::ComplexRaw)
        | Some(ScalarWriteValueKind::Unknown)
        | None => PendingChangeValidation::Invalid {
            reason: "no safe validator is available for this value family".to_string(),
        },
    }
}

fn validate_number_setting(setting_id: &str, value: &str) -> PendingChangeValidation {
    let trimmed = value.trim();
    if trimmed.is_empty() {
        return invalid("numeric value cannot be empty");
    }
    if trimmed.contains(char::is_whitespace) {
        return invalid("numeric value cannot contain whitespace");
    }
    if setting_id == "input.pointer_sensitivity" {
        return validate_unit_float(trimmed, -1.0, 1.0, "pointer sensitivity");
    }
    match trimmed.parse::<i64>() {
        Ok(number) if number >= 0 => PendingChangeValidation::Valid,
        Ok(_) => invalid("numeric value must be non-negative"),
        Err(_) => invalid("numeric value must be an integer"),
    }
}

fn validate_percent_setting(setting_id: &str, value: &str) -> PendingChangeValidation {
    let trimmed = value.trim();
    if setting_id == "input.pointer_sensitivity" {
        return validate_unit_float(trimmed, -1.0, 1.0, "pointer sensitivity");
    }
    validate_unit_float(trimmed, 0.0, 1.0, "percent-like value")
}

fn validate_unit_float(value: &str, min: f64, max: f64, label: &str) -> PendingChangeValidation {
    if value.is_empty() {
        return invalid("floating-point value cannot be empty");
    }
    if value.contains(char::is_whitespace) {
        return invalid("floating-point value cannot contain whitespace");
    }
    match value.parse::<f64>() {
        Ok(number) if number.is_finite() && number >= min && number <= max => {
            PendingChangeValidation::Valid
        }
        Ok(number) if !number.is_finite() => invalid("floating-point value must be finite"),
        Ok(_) => invalid(&format!("{label} must be between {min} and {max}")),
        Err(_) => invalid("floating-point value must be numeric"),
    }
}

fn validate_color_literal(value: &str) -> PendingChangeValidation {
    let trimmed = value.trim();
    if trimmed.is_empty() {
        return invalid("color value cannot be empty");
    }
    if trimmed.contains(char::is_whitespace) {
        return invalid("color value cannot contain whitespace");
    }
    if let Some(hex) = trimmed
        .strip_prefix("rgb(")
        .and_then(|value| value.strip_suffix(')'))
    {
        return validate_hex_digits(hex, 6, "rgb color");
    }
    if let Some(hex) = trimmed
        .strip_prefix("rgba(")
        .and_then(|value| value.strip_suffix(')'))
    {
        return validate_hex_digits(hex, 8, "rgba color");
    }
    if let Some(hex) = trimmed.strip_prefix("0x") {
        return validate_hex_digits(hex, 8, "0xAARRGGBB color");
    }
    invalid("color value must be rgb(RRGGBB), rgba(RRGGBBAA), or 0xAARRGGBB")
}

fn validate_hex_digits(value: &str, expected_len: usize, label: &str) -> PendingChangeValidation {
    if value.len() != expected_len {
        return invalid(&format!("{label} must contain {expected_len} hex digits"));
    }
    if value.chars().all(|char| char.is_ascii_hexdigit()) {
        PendingChangeValidation::Valid
    } else {
        invalid(&format!("{label} contains non-hex characters"))
    }
}

fn validate_vec2_value(value: &str) -> PendingChangeValidation {
    match Vec2Value::parse(value) {
        Ok(_) => PendingChangeValidation::Valid,
        Err(error) => invalid(&error.to_string()),
    }
}

fn invalid(reason: &str) -> PendingChangeValidation {
    PendingChangeValidation::Invalid {
        reason: reason.to_string(),
    }
}
