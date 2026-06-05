use std::path::PathBuf;

use crate::current_config::CurrentConfigSnapshot;
use crate::current_config::CurrentValueProjection;
use crate::source_values::read_system_xkb_rules;
use crate::value::{
    color::ColorValue, gradient::GradientValue, numeric_list::NumericListValue,
    path_value::PathValue, regex_value::RegexValue, sanitized_string::SanitizedStringValue,
    vector::Vec2Value,
};
use crate::write_classification::{
    is_safe_writable_setting, is_verified_finite_choice_value, safe_writable_value_kind,
    ScalarWriteValueKind,
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

#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct PendingChangeValueSources {
    pub monitor_names: Vec<String>,
}

impl PendingChangeValueSources {
    pub fn from_current_config(current_config: &CurrentConfigSnapshot) -> Self {
        Self {
            monitor_names: current_config
                .monitor_source_values()
                .into_iter()
                .map(|value| value.raw_value)
                .collect(),
        }
    }
}

pub fn stage_pending_change(
    setting_id: &str,
    current_value: &CurrentValueProjection,
    proposed_value: impl Into<String>,
) -> PendingChange {
    stage_pending_change_with_sources(
        setting_id,
        current_value,
        proposed_value,
        &PendingChangeValueSources::default(),
    )
}

pub fn stage_pending_change_with_sources(
    setting_id: &str,
    current_value: &CurrentValueProjection,
    proposed_value: impl Into<String>,
    sources: &PendingChangeValueSources,
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

    let validation = validate_safe_writable_value(setting_id, &proposed_value, sources);
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

fn validate_safe_writable_value(
    setting_id: &str,
    value: &str,
    sources: &PendingChangeValueSources,
) -> PendingChangeValidation {
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
        Some(ScalarWriteValueKind::FiniteChoice) => {
            validate_finite_choice_setting(setting_id, value)
        }
        Some(ScalarWriteValueKind::SourceBacked) => {
            validate_source_backed_setting(setting_id, value)
        }
        Some(ScalarWriteValueKind::MonitorName) => validate_monitor_name_setting(value, sources),
        Some(ScalarWriteValueKind::Number) => validate_number_setting(setting_id, value),
        Some(ScalarWriteValueKind::Percent) => validate_percent_setting(setting_id, value),
        Some(ScalarWriteValueKind::Color) => validate_color_literal(value),
        Some(ScalarWriteValueKind::Gradient) => validate_gradient_value(value),
        Some(ScalarWriteValueKind::Vector2) => validate_vec2_value(value),
        Some(ScalarWriteValueKind::NumericList) => validate_numeric_list_value(value),
        Some(ScalarWriteValueKind::CommaSeparatedFloatList) => {
            validate_comma_separated_float_list(value)
        }
        Some(ScalarWriteValueKind::LineSafeString) => validate_line_safe_string(value),
        Some(ScalarWriteValueKind::Path) => validate_path_value(value),
        Some(ScalarWriteValueKind::RegexString) => validate_regex_value(value),
        Some(ScalarWriteValueKind::StringLike)
        | Some(ScalarWriteValueKind::ComplexRaw)
        | Some(ScalarWriteValueKind::Unknown)
        | None => PendingChangeValidation::Invalid {
            reason: "no safe validator is available for this value family".to_string(),
        },
    }
}

fn validate_finite_choice_setting(setting_id: &str, value: &str) -> PendingChangeValidation {
    if is_verified_finite_choice_value(setting_id, value) {
        PendingChangeValidation::Valid
    } else {
        invalid("finite-choice writes require a Hyprland-verified stored raw value")
    }
}

fn validate_source_backed_setting(setting_id: &str, value: &str) -> PendingChangeValidation {
    match read_system_xkb_rules() {
        Ok(rules) if rules.validates_setting_value(setting_id, value) => {
            PendingChangeValidation::Valid
        }
        Ok(_) => invalid("source-backed writes require a known XKB rules value"),
        Err(error) => invalid(&format!(
            "source-backed XKB values are unavailable: {error}"
        )),
    }
}

fn validate_monitor_name_setting(
    value: &str,
    sources: &PendingChangeValueSources,
) -> PendingChangeValidation {
    let trimmed = value.trim();
    if trimmed.contains('\n')
        || trimmed.contains('\r')
        || trimmed.contains('$')
        || trimmed.contains('`')
        || trimmed.contains(';')
    {
        return invalid("monitor output writes require a line-safe monitor name");
    }
    if trimmed.is_empty() {
        return PendingChangeValidation::Valid;
    }
    if sources.monitor_names.iter().any(|name| name == trimmed) {
        PendingChangeValidation::Valid
    } else {
        invalid("monitor output writes require a monitor declared in the current config")
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
    match ColorValue::parse(value) {
        Ok(_) => PendingChangeValidation::Valid,
        Err(error) => invalid(&error.to_string()),
    }
}

fn validate_gradient_value(value: &str) -> PendingChangeValidation {
    match GradientValue::parse(value) {
        Ok(_) => PendingChangeValidation::Valid,
        Err(error) => invalid(&error.to_string()),
    }
}

fn validate_vec2_value(value: &str) -> PendingChangeValidation {
    match Vec2Value::parse(value) {
        Ok(_) => PendingChangeValidation::Valid,
        Err(error) => invalid(&error.to_string()),
    }
}

fn validate_numeric_list_value(value: &str) -> PendingChangeValidation {
    match NumericListValue::parse(value) {
        Ok(_) => PendingChangeValidation::Valid,
        Err(error) => invalid(&error.to_string()),
    }
}

fn validate_comma_separated_float_list(value: &str) -> PendingChangeValidation {
    let trimmed = value.trim();
    if trimmed.is_empty() {
        return invalid("comma-separated float list cannot be empty");
    }
    if trimmed.contains('\n') || trimmed.contains('\r') {
        return invalid("comma-separated float list must stay on one config line");
    }
    if trimmed.starts_with(',') || trimmed.ends_with(',') {
        return invalid("comma-separated float list cannot have empty entries");
    }

    let mut count = 0usize;
    for entry in trimmed.split(',') {
        let entry = entry.trim();
        if entry.is_empty() {
            return invalid("comma-separated float list cannot have empty entries");
        }
        if entry.contains(char::is_whitespace) {
            return invalid("float entries cannot contain internal whitespace");
        }
        match entry.parse::<f64>() {
            Ok(number) if number.is_finite() => {
                count += 1;
            }
            Ok(_) => return invalid("float entries must be finite"),
            Err(_) => return invalid("float entries must be numeric"),
        }
    }

    if count == 0 {
        invalid("comma-separated float list requires at least one value")
    } else {
        PendingChangeValidation::Valid
    }
}

fn validate_line_safe_string(value: &str) -> PendingChangeValidation {
    match SanitizedStringValue::parse(value) {
        Ok(_) => PendingChangeValidation::Valid,
        Err(error) => invalid(&error.to_string()),
    }
}

fn validate_path_value(value: &str) -> PendingChangeValidation {
    match PathValue::parse(value) {
        Ok(_) => PendingChangeValidation::Valid,
        Err(error) => invalid(&error.to_string()),
    }
}

fn validate_regex_value(value: &str) -> PendingChangeValidation {
    match RegexValue::parse(value) {
        Ok(_) => PendingChangeValidation::Valid,
        Err(error) => invalid(&error.to_string()),
    }
}

fn invalid(reason: &str) -> PendingChangeValidation {
    PendingChangeValidation::Invalid {
        reason: reason.to_string(),
    }
}
