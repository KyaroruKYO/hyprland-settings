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

/// One color token normalized to the `rgba(RRGGBBAA)` string the runtime
/// color-table grammar accepts. Accepts `rgba(hex8)` (RRGGBBAA),
/// `rgb(hex6)`, the `0xAARRGGBB` config form, and the readback's bare
/// `AARRGGBB`/`RRGGBB` hex. Anything else is rejected (fail closed).
fn color_token_to_rgba(token: &str) -> Option<String> {
    fn is_hex(text: &str) -> bool {
        !text.is_empty() && text.chars().all(|character| character.is_ascii_hexdigit())
    }
    let token = token.trim();
    if let Some(inner) = token
        .strip_prefix("rgba(")
        .and_then(|rest| rest.strip_suffix(')'))
    {
        let inner = inner.trim();
        if inner.len() == 8 && is_hex(inner) {
            return Some(format!("rgba({})", inner.to_ascii_lowercase()));
        }
        return None;
    }
    if let Some(inner) = token
        .strip_prefix("rgb(")
        .and_then(|rest| rest.strip_suffix(')'))
    {
        let inner = inner.trim();
        if inner.len() == 6 && is_hex(inner) {
            return Some(format!("rgba({}ff)", inner.to_ascii_lowercase()));
        }
        return None;
    }
    // Pure-decimal u32: the readback form for int-typed single-color
    // options ("int: 1426063360"); the bits are AARRGGBB.
    if token.chars().all(|character| character.is_ascii_digit()) && token.len() >= 8 {
        if let Ok(bits) = token.parse::<u32>() {
            let hex = format!("{bits:08x}");
            return Some(format!("rgba({}{})", &hex[2..8], &hex[0..2]));
        }
    }
    let bare = token.strip_prefix("0x").unwrap_or(token);
    // Six bare digits keep the config convention (RRGGBB, opaque).
    if bare.len() == 6 && is_hex(bare) && !token.starts_with("0x") {
        return Some(format!("rgba({}ff)", bare.to_ascii_lowercase()));
    }
    // AARRGGBB (config 0x form and getoption readback) -> RRGGBBAA. The
    // readback prints %x without zero padding, so seven-digit (or shorter
    // non-six) tokens left-pad to eight. Known ambiguity: a fully
    // transparent color whose readback collapses to exactly six digits
    // reads as opaque RGB instead; there is no type information to
    // disambiguate.
    if (1..=8).contains(&bare.len()) && bare.len() != 6 && is_hex(bare) {
        let padded = format!("{:0>8}", bare.to_ascii_lowercase());
        return Some(format!("rgba({}{})", &padded[2..8], &padded[0..2]));
    }
    if bare.len() == 6 && is_hex(bare) {
        // 0x-prefixed six digits: zero-padded AARRGGBB.
        let padded = format!("{:0>8}", bare.to_ascii_lowercase());
        return Some(format!("rgba({}{})", &padded[2..8], &padded[0..2]));
    }
    None
}

/// Render 2-4 finite numbers as the proven css-gap lua table (CSS
/// shorthand order); None when the text is not gap-shaped.
fn rendered_css_gap_table(value: &str) -> Option<String> {
    let parts: Vec<&str> = value.split_whitespace().collect();
    if !(2..=4).contains(&parts.len())
        || !parts
            .iter()
            .all(|part| part.parse::<f64>().map(f64::is_finite).unwrap_or(false))
    {
        return None;
    }
    Some(match parts.as_slice() {
        [vertical, horizontal] => format!(
            "{{ top = {vertical}, right = {horizontal}, bottom = {vertical}, left = {horizontal} }}"
        ),
        [top, horizontal, bottom] => format!(
            "{{ top = {top}, right = {horizontal}, bottom = {bottom}, left = {horizontal} }}"
        ),
        [top, right, bottom, left] => {
            format!("{{ top = {top}, right = {right}, bottom = {bottom}, left = {left} }}")
        }
        _ => unreachable!(),
    })
}

/// Render a color/gradient value as the runtime's proven lua table form.
fn rendered_color_lua_table(
    row: &SafeWritableRow,
    value: &str,
) -> Result<String, RuntimePreviewError> {
    let mut colors: Vec<String> = Vec::new();
    let mut angle: Option<u16> = None;
    for part in value.split_whitespace() {
        if let Some(degrees) = part.strip_suffix("deg") {
            if angle.is_some() {
                return Err(RuntimePreviewError::InvalidValue {
                    row_id: row.row_id.to_string(),
                    reason: "color value has more than one angle",
                });
            }
            angle =
                Some(
                    degrees
                        .parse::<u16>()
                        .map_err(|_| RuntimePreviewError::InvalidValue {
                            row_id: row.row_id.to_string(),
                            reason: "color angle must be a whole number of degrees",
                        })?,
                );
            continue;
        }
        colors.push(
            color_token_to_rgba(part).ok_or(RuntimePreviewError::InvalidValue {
                row_id: row.row_id.to_string(),
                reason: "color token is not in a recognized color form",
            })?,
        );
    }
    if colors.is_empty() || colors.len() > 10 {
        return Err(RuntimePreviewError::InvalidValue {
            row_id: row.row_id.to_string(),
            reason: "color value must have between one and ten color stops",
        });
    }
    // Shape picks the grammar: int-typed single-color options reject the
    // gradient table ("color type requires a color string") and accept a
    // plain color string, which single-stop gradient options accept too;
    // multi-stop or angled values require the table (strings are rejected
    // with "invalid color" there).
    if colors.len() == 1 && angle.is_none() {
        return Ok(format!("\"{}\"", colors[0]));
    }
    let rendered_colors = colors
        .iter()
        .map(|color| format!("\"{color}\""))
        .collect::<Vec<_>>()
        .join(", ");
    Ok(match angle {
        Some(angle) => format!("{{ colors = {{ {rendered_colors} }}, angle = {angle} }}"),
        None => format!("{{ colors = {{ {rendered_colors} }} }}"),
    })
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
            } else if let Some(rendered) = rendered_css_gap_table(value) {
                // Some Number-classified rows are css-gap typed at runtime
                // (float_gaps reads back "1 1 1 1"): a revert re-applies
                // that readback original, which must render as the proven
                // gap table instead of failing the numeric parse and
                // leaving the session stuck.
                Ok(rendered)
            } else {
                Err(RuntimePreviewError::InvalidValue {
                    row_id: row.row_id.to_string(),
                    reason: "numeric value did not parse as a finite number",
                })
            }
        }
        ScalarWriteValueKind::FiniteChoice => {
            // Live-proven: int-typed finite-choice options reject quoted
            // strings ("integer type requires a bool or an integer"), so
            // numeric choice values must render bare. Values are validated
            // against the row's own allowed choices.
            let allowed = crate::write_classification::finite_choice_options(row.row_id)
                .map(|choices| choices.iter().any(|choice| choice.raw_value == value))
                .unwrap_or(false);
            if !allowed {
                return Err(RuntimePreviewError::InvalidValue {
                    row_id: row.row_id.to_string(),
                    reason: "value is not one of this row's allowed choices",
                });
            }
            if value.parse::<i64>().is_ok() || value.parse::<f64>().is_ok() {
                Ok(value.to_string())
            } else {
                Ok(format!("\"{value}\""))
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
        ScalarWriteValueKind::Color | ScalarWriteValueKind::Gradient => {
            // Live-proven grammar on the lua runtime: color/gradient
            // options accept a table `{ colors = { "rgba(RRGGBBAA)", … },
            // angle = N }`; multi-stop or angle-suffixed plain strings are
            // rejected with "invalid color". Tokens normalize from every
            // accepted config form — including the bare AARRGGBB hex the
            // getoption readback reports — so applies AND reverts (which
            // re-apply the readback original) round-trip.
            rendered_color_lua_table(row, value)
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

/// Run one read-only option query through a runner and parse the value.
/// The only command this issues is `getoption`; it can never mutate.
pub fn read_runtime_option(
    runner: &mut dyn RuntimePreviewRunner,
    official_setting: &str,
) -> Option<String> {
    let query = runtime_option_query(official_setting);
    runner
        .run("hyprctl", &query)
        .ok()
        .and_then(|output| parse_getoption_value(&output))
}

/// Parse the value out of a `getoption` response (`int: 5`, `float: 0.5`,
/// `bool: false`, `css gap data: 5 5 5 5`, `str: ...`, and gradient/color
/// rows' `gradient data: AARRGGBB [AARRGGBB…] [Ndeg]` — without this
/// prefix, color-row preview sessions could never capture an original and
/// every color preview failed at session start).
pub fn parse_getoption_value(output: &str) -> Option<String> {
    for line in output.lines() {
        let line = line.trim();
        for prefix in [
            "int:",
            "float:",
            "bool:",
            "css gap data:",
            "str:",
            "color:",
            "gradient data:",
        ] {
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

/// Apply a preview value inside an armed supervised (dead-man) session. The
/// caller must be the dead-man controller, which only exists after the user
/// explicitly armed supervision; the session's own `confirmed` flag keeps
/// tracking the later "Keep changes" decision.
pub fn apply_runtime_preview_value_supervised(
    runner: &mut dyn RuntimePreviewRunner,
    session: &mut RuntimePreviewSession,
    raw_value: &str,
) -> Result<RuntimePreviewApplyReceipt, RuntimePreviewError> {
    if session.state != RuntimePreviewSessionState::Active {
        return Err(RuntimePreviewError::NoActiveSession);
    }
    if session.dead_man.is_none() {
        return Err(RuntimePreviewError::RowNotLivePreviewable {
            row_id: session.row_id.clone(),
            capability: "supervised apply is only valid for dead-man sessions",
        });
    }
    let command = build_runtime_preview_command(&session.row_id, raw_value, true)?;
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

/// Revert an armed supervised session to its original value. Used by manual
/// revert, cancel, timeout auto-revert, and app-close recovery — it must work
/// precisely when the user has NOT confirmed, so it authorizes the revert
/// command itself.
pub fn revert_runtime_preview_session_supervised(
    runner: &mut dyn RuntimePreviewRunner,
    session: &mut RuntimePreviewSession,
) -> Result<RuntimePreviewRevertReceipt, RuntimePreviewError> {
    if session.state != RuntimePreviewSessionState::Active {
        return Err(RuntimePreviewError::NoActiveSession);
    }
    if session.dead_man.is_none() {
        return Err(RuntimePreviewError::RowNotLivePreviewable {
            row_id: session.row_id.clone(),
            capability: "supervised revert is only valid for dead-man sessions",
        });
    }
    if session.last_applied_value.is_some() {
        let command =
            build_runtime_preview_command(&session.row_id, &session.original_value, true)?;
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
