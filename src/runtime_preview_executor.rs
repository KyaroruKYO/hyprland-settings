//! Runtime preview executor: builds and (through a pluggable runner) applies
//! reversible runtime preview commands for capability-approved rows only.
//!
//! Hard boundaries: no config file is ever read for mutation or written here,
//! no compositor reload is ever issued, unsupported and blocked rows are
//! rejected before any command is built, dead-man rows are rejected without a
//! confirmed dead-man session, and every applied preview captures the
//! original value first so it can be reverted. Persistence is a separate,
//! explicit Save through the existing config write path — never from here.

use std::process::Command;

use serde::Serialize;

use crate::runtime_preview::{
    classify_runtime_preview_row, RuntimePreviewCapability, RuntimePreviewRowCapability,
};
use crate::write_classification::{SafeWritableRow, ScalarWriteValueKind, SAFE_WRITABLE_ROWS};

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum RuntimePreviewError {
    UnknownRow(String),
    RowNotLivePreviewable {
        row_id: String,
        capability: &'static str,
    },
    DeadManConfirmationRequired(String),
    InvalidValue {
        row_id: String,
        reason: &'static str,
    },
    RunnerFailed(String),
    OriginalValueUnavailable(String),
    NoActiveSession,
}

/// A fully built runtime preview command. Arguments are passed as argv (no
/// shell), and construction guarantees the expression never contains a
/// reload form.
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct RuntimePreviewCommand {
    pub program: &'static str,
    pub args: Vec<String>,
    pub row_id: String,
    pub official_setting: String,
    pub rendered_value: String,
}

fn rendered_lua_value(
    row: &SafeWritableRow,
    raw_value: &str,
) -> Result<String, RuntimePreviewError> {
    let value = raw_value.trim();
    if value.is_empty() {
        return Err(RuntimePreviewError::InvalidValue {
            row_id: row.row_id.to_string(),
            reason: "empty value",
        });
    }
    // No quotes, backslashes, braces, or control characters ever reach the
    // expression; values are matched against conservative per-kind grammars.
    if value.chars().any(|ch| {
        ch.is_control()
            || matches!(
                ch,
                '"' | '\'' | '\\' | '{' | '}' | '[' | ']' | '=' | ';' | '$' | '`'
            )
    }) {
        return Err(RuntimePreviewError::InvalidValue {
            row_id: row.row_id.to_string(),
            reason: "value contains characters outside the runtime-safe grammar",
        });
    }
    match row.value_kind {
        ScalarWriteValueKind::Boolean => match value {
            "true" | "false" => Ok(value.to_string()),
            "1" => Ok("true".to_string()),
            "0" => Ok("false".to_string()),
            _ => Err(RuntimePreviewError::InvalidValue {
                row_id: row.row_id.to_string(),
                reason: "boolean value must be true/false/1/0",
            }),
        },
        ScalarWriteValueKind::Number | ScalarWriteValueKind::Percent => {
            if value.parse::<f64>().map(f64::is_finite).unwrap_or(false) {
                Ok(value.to_string())
            } else {
                Err(RuntimePreviewError::InvalidValue {
                    row_id: row.row_id.to_string(),
                    reason: "numeric value did not parse as a finite number",
                })
            }
        }
        ScalarWriteValueKind::CssGap => {
            // Proven grammar: css_gap accepts an integer or a table with
            // top/right/bottom/left fields. CSS shorthand order applies.
            let parts: Vec<&str> = value.split_whitespace().collect();
            if parts.is_empty()
                || !parts
                    .iter()
                    .all(|part| part.parse::<f64>().map(f64::is_finite).unwrap_or(false))
            {
                return Err(RuntimePreviewError::InvalidValue {
                    row_id: row.row_id.to_string(),
                    reason: "css gap value must be 1-4 finite numbers",
                });
            }
            match parts.as_slice() {
                [all] => Ok((*all).to_string()),
                [vertical, horizontal] => Ok(format!(
                    "{{ top = {vertical}, right = {horizontal}, bottom = {vertical}, left = {horizontal} }}"
                )),
                [top, horizontal, bottom] => Ok(format!(
                    "{{ top = {top}, right = {horizontal}, bottom = {bottom}, left = {horizontal} }}"
                )),
                [top, right, bottom, left] => Ok(format!(
                    "{{ top = {top}, right = {right}, bottom = {bottom}, left = {left} }}"
                )),
                _ => Err(RuntimePreviewError::InvalidValue {
                    row_id: row.row_id.to_string(),
                    reason: "css gap value must be 1-4 finite numbers",
                }),
            }
        }
        ScalarWriteValueKind::Vector2
        | ScalarWriteValueKind::NumericList
        | ScalarWriteValueKind::CommaSeparatedFloatList => {
            if value.parse::<f64>().map(f64::is_finite).unwrap_or(false) {
                Ok(value.to_string())
            } else {
                Err(RuntimePreviewError::InvalidValue {
                    row_id: row.row_id.to_string(),
                    reason: "multi-component numeric-list runtime grammar is not proven; only single numeric values preview",
                })
            }
        }
        _ => {
            if value.chars().all(|ch| {
                ch.is_ascii_alphanumeric()
                    || matches!(
                        ch,
                        ' ' | ',' | '.' | '(' | ')' | '#' | '%' | '-' | '_' | '*'
                    )
            }) {
                Ok(format!("\"{value}\""))
            } else {
                Err(RuntimePreviewError::InvalidValue {
                    row_id: row.row_id.to_string(),
                    reason:
                        "string-like value contains characters outside the runtime-safe grammar",
                })
            }
        }
    }
}

fn capability_for_preview(
    row_id: &str,
    dead_man_confirmed: bool,
) -> Result<(&'static SafeWritableRow, RuntimePreviewRowCapability), RuntimePreviewError> {
    let row = SAFE_WRITABLE_ROWS
        .iter()
        .find(|row| row.row_id == row_id)
        .ok_or_else(|| RuntimePreviewError::UnknownRow(row_id.to_string()))?;
    let capability = classify_runtime_preview_row(row);
    if capability.capability.live_previewable_by_default() {
        return Ok((row, capability));
    }
    if capability.capability == RuntimePreviewCapability::LivePreviewSupportedWithDeadMan {
        if dead_man_confirmed {
            return Ok((row, capability));
        }
        return Err(RuntimePreviewError::DeadManConfirmationRequired(
            row_id.to_string(),
        ));
    }
    Err(RuntimePreviewError::RowNotLivePreviewable {
        row_id: row_id.to_string(),
        capability: capability.capability.as_str(),
    })
}

/// Build the runtime preview command for a row/value. Rejects unknown rows,
/// rows whose capability forbids live preview, dead-man rows without a
/// confirmed session, and values outside the runtime-safe grammar.
pub fn build_runtime_preview_command(
    row_id: &str,
    raw_value: &str,
    dead_man_confirmed: bool,
) -> Result<RuntimePreviewCommand, RuntimePreviewError> {
    let (row, _capability) = capability_for_preview(row_id, dead_man_confirmed)?;
    let rendered = rendered_lua_value(row, raw_value)?;
    let mut segments: Vec<&str> = row.official_setting.split('.').collect();
    let option = segments.pop().unwrap_or_default();
    let mut expression = format!("{option} = {rendered}");
    for segment in segments.iter().rev() {
        expression = format!("{segment} = {{ {expression} }}");
    }
    Ok(RuntimePreviewCommand {
        program: "hyprctl",
        args: vec!["eval".to_string(), format!("hl.config({{ {expression} }})")],
        row_id: row.row_id.to_string(),
        official_setting: row.official_setting.to_string(),
        rendered_value: rendered,
    })
}

/// The read-only option query used to capture the original value.
pub fn runtime_option_query(official_setting: &str) -> Vec<String> {
    vec!["getoption".to_string(), official_setting.replace('.', ":")]
}

/// Pluggable runner so tests never touch the live compositor. The real
/// hyprctl runner is used only by explicitly gated live paths.
pub trait RuntimePreviewRunner {
    fn run(&mut self, program: &str, args: &[String]) -> Result<String, String>;
}

/// Real runner. Only ever invoked with `eval hl.config(...)` set expressions
/// and `getoption` reads built by this module; never with reload forms.
pub struct HyprctlRuntimePreviewRunner;

impl RuntimePreviewRunner for HyprctlRuntimePreviewRunner {
    fn run(&mut self, program: &str, args: &[String]) -> Result<String, String> {
        let output = Command::new(program)
            .args(args)
            .output()
            .map_err(|error| error.to_string())?;
        if !output.status.success() {
            return Err(String::from_utf8_lossy(&output.stderr).into_owned());
        }
        let stdout = String::from_utf8_lossy(&output.stdout).into_owned();
        // hyprctl eval reports evaluation errors on stdout with a zero exit
        // status; treat them as failures so reverts are never assumed.
        if stdout.trim_start().starts_with("error") {
            return Err(stdout);
        }
        Ok(stdout)
    }
}

/// Parse the value out of a `getoption` response (`int: 5`, `float: 0.5`,
/// `bool: false`, `css gap data: 5 5 5 5`, `str: ...`).
pub fn parse_getoption_value(output: &str) -> Option<String> {
    for line in output.lines() {
        let line = line.trim();
        for prefix in ["int:", "float:", "bool:", "css gap data:", "str:", "color:"] {
            if let Some(rest) = line.strip_prefix(prefix) {
                let value = rest.trim();
                if !value.is_empty() {
                    return Some(value.to_string());
                }
            }
        }
    }
    None
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub enum RuntimePreviewSessionState {
    Active,
    Saved,
    Reverted,
    Cancelled,
}

#[derive(Debug, Clone, Serialize)]
pub struct RuntimePreviewApplyReceipt {
    pub row_id: String,
    pub official_setting: String,
    pub applied_value: String,
    pub original_value: String,
    pub config_written: bool,
    pub reload_run: bool,
}

#[derive(Debug, Clone, Serialize)]
pub struct RuntimePreviewRevertReceipt {
    pub row_id: String,
    pub official_setting: String,
    pub restored_value: String,
    pub config_written: bool,
    pub reload_run: bool,
}

/// One live preview session for one row: captures the original value on
/// start, applies throttled preview values, and reverts on cancel/timeout.
#[derive(Debug, Clone, Serialize)]
pub struct RuntimePreviewSession {
    pub row_id: String,
    pub official_setting: String,
    pub original_value: String,
    pub last_applied_value: Option<String>,
    pub state: RuntimePreviewSessionState,
    pub dead_man: Option<RuntimePreviewDeadMan>,
    pub config_writes_during_preview: u32,
    pub reload_runs_during_preview: u32,
}

/// Start a session: verifies capability, captures the original runtime value
/// read-only. No mutation happens here.
pub fn start_runtime_preview_session(
    runner: &mut dyn RuntimePreviewRunner,
    row_id: &str,
    dead_man_confirmed: bool,
) -> Result<RuntimePreviewSession, RuntimePreviewError> {
    let (row, capability) = capability_for_preview(row_id, dead_man_confirmed)?;
    let query = runtime_option_query(row.official_setting);
    let output = runner
        .run("hyprctl", &query)
        .map_err(RuntimePreviewError::RunnerFailed)?;
    let original_value = parse_getoption_value(&output)
        .ok_or_else(|| RuntimePreviewError::OriginalValueUnavailable(row_id.to_string()))?;
    Ok(RuntimePreviewSession {
        row_id: row.row_id.to_string(),
        official_setting: row.official_setting.to_string(),
        original_value,
        last_applied_value: None,
        state: RuntimePreviewSessionState::Active,
        dead_man: if capability.dead_man_required {
            Some(RuntimePreviewDeadMan::new(
                RUNTIME_PREVIEW_DEAD_MAN_TIMEOUT_MS,
            ))
        } else {
            None
        },
        config_writes_during_preview: 0,
        reload_runs_during_preview: 0,
    })
}

/// Apply one preview value inside an active session.
pub fn apply_runtime_preview_value(
    runner: &mut dyn RuntimePreviewRunner,
    session: &mut RuntimePreviewSession,
    raw_value: &str,
) -> Result<RuntimePreviewApplyReceipt, RuntimePreviewError> {
    if session.state != RuntimePreviewSessionState::Active {
        return Err(RuntimePreviewError::NoActiveSession);
    }
    let dead_man_confirmed = session
        .dead_man
        .as_ref()
        .map(|dead_man| dead_man.confirmed)
        .unwrap_or(false);
    let command = build_runtime_preview_command(&session.row_id, raw_value, dead_man_confirmed)?;
    runner
        .run(command.program, &command.args)
        .map_err(RuntimePreviewError::RunnerFailed)?;
    session.last_applied_value = Some(raw_value.trim().to_string());
    Ok(RuntimePreviewApplyReceipt {
        row_id: session.row_id.clone(),
        official_setting: session.official_setting.clone(),
        applied_value: raw_value.trim().to_string(),
        original_value: session.original_value.clone(),
        config_written: false,
        reload_run: false,
    })
}

/// Revert the session to its captured original value (Cancel / timeout /
/// app-close recovery path).
pub fn revert_runtime_preview_session(
    runner: &mut dyn RuntimePreviewRunner,
    session: &mut RuntimePreviewSession,
) -> Result<RuntimePreviewRevertReceipt, RuntimePreviewError> {
    if session.state != RuntimePreviewSessionState::Active {
        return Err(RuntimePreviewError::NoActiveSession);
    }
    if session.last_applied_value.is_some() {
        let dead_man_confirmed = session
            .dead_man
            .as_ref()
            .map(|dead_man| dead_man.confirmed)
            .unwrap_or(false);
        let command = build_runtime_preview_command(
            &session.row_id,
            &session.original_value,
            dead_man_confirmed,
        )?;
        runner
            .run(command.program, &command.args)
            .map_err(RuntimePreviewError::RunnerFailed)?;
    }
    session.state = RuntimePreviewSessionState::Reverted;
    Ok(RuntimePreviewRevertReceipt {
        row_id: session.row_id.clone(),
        official_setting: session.official_setting.clone(),
        restored_value: session.original_value.clone(),
        config_written: false,
        reload_run: false,
    })
}

/// Mark the session saved. Persistence itself must go through the existing
/// config write path exactly once; this function never writes anything.
pub fn mark_runtime_preview_session_saved(
    session: &mut RuntimePreviewSession,
) -> Result<&'static str, RuntimePreviewError> {
    if session.state != RuntimePreviewSessionState::Active {
        return Err(RuntimePreviewError::NoActiveSession);
    }
    session.state = RuntimePreviewSessionState::Saved;
    Ok("persist the final value once through the existing backup/write/reread config path")
}

pub const RUNTIME_PREVIEW_DEAD_MAN_TIMEOUT_MS: u64 = 10_000;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
pub enum RuntimePreviewDeadManVerdict {
    KeepActive,
    RevertRequired,
}

/// Dead-man countdown: high-risk previews revert unless confirmed in time.
#[derive(Debug, Clone, Serialize)]
pub struct RuntimePreviewDeadMan {
    pub timeout_ms: u64,
    pub elapsed_ms: u64,
    pub confirmed: bool,
    pub recovery_instruction: &'static str,
}

impl RuntimePreviewDeadMan {
    pub fn new(timeout_ms: u64) -> Self {
        Self {
            timeout_ms,
            elapsed_ms: 0,
            confirmed: false,
            recovery_instruction:
                "if the app exits during an unconfirmed high-risk preview, reapply the recorded original value via the same runtime set",
        }
    }

    pub fn tick(&mut self, delta_ms: u64) {
        self.elapsed_ms = self.elapsed_ms.saturating_add(delta_ms);
    }

    pub fn confirm(&mut self) {
        self.confirmed = true;
    }

    pub fn evaluate(&self) -> RuntimePreviewDeadManVerdict {
        if self.confirmed || self.elapsed_ms < self.timeout_ms {
            RuntimePreviewDeadManVerdict::KeepActive
        } else {
            RuntimePreviewDeadManVerdict::RevertRequired
        }
    }
}

/// Trailing-edge throttle: keeps only the latest pending value and releases
/// it when the interval has passed. Slider drags therefore issue at most one
/// runtime set per interval and never a config write.
#[derive(Debug, Clone, Serialize)]
pub struct RuntimePreviewThrottle {
    pub min_interval_ms: u64,
    pub last_applied_at_ms: Option<u64>,
    pub pending_value: Option<String>,
}

impl RuntimePreviewThrottle {
    pub fn new(min_interval_ms: u64) -> Self {
        Self {
            min_interval_ms,
            last_applied_at_ms: None,
            pending_value: None,
        }
    }

    /// Offer a new value at `now_ms`. Returns the value to apply immediately
    /// (interval elapsed) or stores it as the single pending value.
    pub fn offer(&mut self, value: &str, now_ms: u64) -> Option<String> {
        let due = self
            .last_applied_at_ms
            .map(|last| now_ms.saturating_sub(last) >= self.min_interval_ms)
            .unwrap_or(true);
        if due {
            self.last_applied_at_ms = Some(now_ms);
            self.pending_value = None;
            Some(value.to_string())
        } else {
            self.pending_value = Some(value.to_string());
            None
        }
    }

    /// Release the pending value if the interval has elapsed.
    pub fn take_due(&mut self, now_ms: u64) -> Option<String> {
        let due = self
            .last_applied_at_ms
            .map(|last| now_ms.saturating_sub(last) >= self.min_interval_ms)
            .unwrap_or(true);
        if due {
            if let Some(value) = self.pending_value.take() {
                self.last_applied_at_ms = Some(now_ms);
                return Some(value);
            }
        }
        None
    }
}
