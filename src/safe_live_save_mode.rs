//! Safe Live Save Mode: the app's answer to the autoreload problem.
//!
//! The product principle: live changes come from runtime preview (instant,
//! reversible, no config writes); persistence is one config write after
//! preview; and that write must not trigger an accidental compositor reload.
//!
//! Live-proven strategy (2026-07-12, against the running compositor):
//! `misc:disable_autoreload` can be set at runtime through the same eval
//! mechanism the preview executor uses, with getoption-verified apply and
//! exact revert. With the runtime value true, a config write genuinely does
//! not auto-reload — proven by the first active-config write pilot, which
//! wrote and byte-exactly restored the real hyprland.conf while the runtime
//! flag stayed true throughout (a reload would have reset it to the config's
//! false). No compositor reload command is ever issued, and enabling the
//! mode at runtime touches no file.
//!
//! The commands here are fixed constants (no user input reaches them), and
//! every transition is verified through read-only readback.

use serde::Serialize;

use crate::runtime_preview_executor::{read_runtime_option, RuntimePreviewRunner};

pub const SAFE_LIVE_SAVE_ENABLE_EXPRESSION: &str =
    "hl.config({ misc = { disable_autoreload = true } })";
pub const SAFE_LIVE_SAVE_DISABLE_EXPRESSION: &str =
    "hl.config({ misc = { disable_autoreload = false } })";

/// Evidence from the live proof runs that back this feature.
pub const SAFE_LIVE_SAVE_MODE_PROOF: &[&str] = &[
    "runtime control proven: misc:disable_autoreload set true and reverted false with getoption verification (2026-07-12)",
    "reload avoidance proven: with the runtime flag true, the active-config pilot wrote and byte-exactly restored hyprland.conf while the flag stayed true throughout - no reload fired (config SHA-256 identical before and after)",
    "the first active-config write pilot passed through its designed fifteen-gate path with live-collected autoreload evidence",
];

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
pub enum SafeLiveSaveModeState {
    /// Autoreload is disabled at runtime: saves cannot trigger a reload.
    ActiveViaRuntime,
    /// Autoreload is active: a config write would reload the compositor.
    Inactive,
    /// The runtime value could not be read; fail closed.
    Unknown,
}

impl SafeLiveSaveModeState {
    pub fn as_str(self) -> &'static str {
        match self {
            Self::ActiveViaRuntime => "ActiveViaRuntime",
            Self::Inactive => "Inactive",
            Self::Unknown => "Unknown",
        }
    }
}

#[derive(Debug, Clone, Serialize)]
pub struct SafeLiveSaveModeStatus {
    pub state: SafeLiveSaveModeState,
    pub runtime_disable_autoreload: Option<String>,
    pub save_gate_open: bool,
    pub blocked_reason: Option<&'static str>,
    pub explanation: &'static str,
    pub one_time_reload_warning_needed: bool,
}

/// Read the current mode from the runtime (one read-only option query).
pub fn read_safe_live_save_mode_status(
    runner: &mut dyn RuntimePreviewRunner,
) -> SafeLiveSaveModeStatus {
    let observed = read_runtime_option(runner, "misc.disable_autoreload");
    let state = match observed.as_deref() {
        Some("true") => SafeLiveSaveModeState::ActiveViaRuntime,
        Some(_) => SafeLiveSaveModeState::Inactive,
        None => SafeLiveSaveModeState::Unknown,
    };
    SafeLiveSaveModeStatus {
        state,
        runtime_disable_autoreload: observed,
        save_gate_open: state == SafeLiveSaveModeState::ActiveViaRuntime,
        blocked_reason: match state {
            SafeLiveSaveModeState::ActiveViaRuntime => None,
            SafeLiveSaveModeState::Inactive => Some(
                "autoreload is active: writing hyprland.conf now would reload Hyprland immediately",
            ),
            SafeLiveSaveModeState::Unknown => {
                Some("the runtime autoreload state could not be read; failing closed")
            }
        },
        explanation: "Safe Live Save Mode disables Hyprland's config autoreload at runtime so the app can preview changes live and save once without triggering a compositor reload. Enabling it at runtime touches no file and is instantly reversible.",
        // Enabling via runtime never needs a reload; only persisting the
        // setting into the config would (and that is a separate, gated step).
        one_time_reload_warning_needed: false,
    }
}

#[derive(Debug, Clone, Serialize)]
pub struct SafeLiveSaveModeTransitionReceipt {
    pub action: &'static str,
    pub before: Option<String>,
    pub after: Option<String>,
    pub verified: bool,
    pub config_written: bool,
    pub reload_run: bool,
    pub status_text: String,
}

fn transition(
    runner: &mut dyn RuntimePreviewRunner,
    action: &'static str,
    expression: &'static str,
    expected: &str,
) -> Result<SafeLiveSaveModeTransitionReceipt, String> {
    let before = read_runtime_option(runner, "misc.disable_autoreload");
    runner
        .run("hyprctl", &["eval".to_string(), expression.to_string()])
        .map_err(|error| format!("runtime transition failed: {error}"))?;
    let after = read_runtime_option(runner, "misc.disable_autoreload");
    let verified = after.as_deref() == Some(expected);
    if !verified {
        return Err(format!(
            "transition verification failed: expected {expected:?}, observed {after:?}"
        ));
    }
    Ok(SafeLiveSaveModeTransitionReceipt {
        action,
        before,
        after: after.clone(),
        verified,
        config_written: false,
        reload_run: false,
        status_text: format!(
            "Safe Live Save Mode {action}: misc:disable_autoreload is now {} (runtime only; no file was written, no reload ran).",
            after.as_deref().unwrap_or("unknown")
        ),
    })
}

/// Enable the mode at runtime: no config write, no reload, verified readback.
pub fn enable_safe_live_save_mode(
    runner: &mut dyn RuntimePreviewRunner,
) -> Result<SafeLiveSaveModeTransitionReceipt, String> {
    transition(runner, "enabled", SAFE_LIVE_SAVE_ENABLE_EXPRESSION, "true")
}

/// Disable the mode at runtime (restore Hyprland's default behavior).
pub fn disable_safe_live_save_mode(
    runner: &mut dyn RuntimePreviewRunner,
) -> Result<SafeLiveSaveModeTransitionReceipt, String> {
    transition(
        runner,
        "disabled",
        SAFE_LIVE_SAVE_DISABLE_EXPRESSION,
        "false",
    )
}

/// Live wrappers owning the runner, so UI code never constructs one.
pub fn read_safe_live_save_mode_status_live() -> SafeLiveSaveModeStatus {
    let mut runner = crate::runtime_preview_executor::HyprctlRuntimePreviewRunner;
    read_safe_live_save_mode_status(&mut runner)
}

pub fn enable_safe_live_save_mode_live() -> Result<SafeLiveSaveModeTransitionReceipt, String> {
    let mut runner = crate::runtime_preview_executor::HyprctlRuntimePreviewRunner;
    enable_safe_live_save_mode(&mut runner)
}

pub fn disable_safe_live_save_mode_live() -> Result<SafeLiveSaveModeTransitionReceipt, String> {
    let mut runner = crate::runtime_preview_executor::HyprctlRuntimePreviewRunner;
    disable_safe_live_save_mode(&mut runner)
}
