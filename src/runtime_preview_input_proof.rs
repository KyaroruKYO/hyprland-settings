//! Per-row supervised live-proof plans for the input/cursor dead-man rows.
//!
//! Input and cursor settings have a different failure mode from visual
//! settings: a bad live preview can impair the user's ability to click, type,
//! focus, or confirm. Every row therefore gets an explicit proof plan that
//! names the input subsystem it touches, the fallback input path that must
//! remain usable, the minimal preview value, the verification strategy, and
//! the recovery instruction — and rows are promoted from needs-live-proof to
//! armable only after an env-gated live proof actually ran and verified both
//! apply and revert. Nothing here mutates anything: this module produces
//! plans and gates; execution happens only through the supervised executor
//! paths driven by the env-gated harness.

use serde::Serialize;

use crate::runtime_preview_dead_man::{classify_dead_man_row, RuntimePreviewDeadManClassification};
use crate::write_classification::{ScalarWriteValueKind, SAFE_WRITABLE_ROWS};

/// Rows whose per-row live proof has actually run and passed, with the
/// receipt recorded. Only entries here may be promoted to armable.
/// Evidence: the env-gated harness in tests/runtime_preview_input_live_proof.rs,
/// executed against the running compositor.
pub const PROVEN_INPUT_ROWS: &[ProvenInputRow] = &[ProvenInputRow {
    official_setting: "cursor.inactive_timeout",
    original_value: "0.000000",
    preview_value: "1",
    fallback_used: "keyboard and pointer both remained fully usable; only idle cursor-hide timing changed; the revert needed no user input",
    proof_date: "2026-07-12",
    proof_env: "HYPRLAND_SETTINGS_RUN_INPUT_LIVE_PROOF=1 HYPRLAND_SETTINGS_INPUT_PROOF_ROW=cursor.inactive_timeout",
}];

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
pub struct ProvenInputRow {
    pub official_setting: &'static str,
    pub original_value: &'static str,
    pub preview_value: &'static str,
    pub fallback_used: &'static str,
    pub proof_date: &'static str,
    pub proof_env: &'static str,
}

pub fn proven_input_row(official_setting: &str) -> Option<&'static ProvenInputRow> {
    PROVEN_INPUT_ROWS
        .iter()
        .find(|row| row.official_setting == official_setting)
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
pub enum RuntimePreviewInputProofClassification {
    NeedsLiveProofInputKeyboard,
    NeedsLiveProofInputPointer,
    NeedsLiveProofInputTouchpad,
    NeedsLiveProofCursor,
    NeedsLiveProofFocus,
    NeedsLiveProofGesture,
    ProofModelOnly,
    ProofBlockedNoSafeFallback,
    ProofBlockedNoRuntimeVerification,
    ProofBlockedTooDangerous,
    ProofReadyForEnvGatedLiveTest,
    ProofPassedArmableCandidate,
}

impl RuntimePreviewInputProofClassification {
    pub fn as_str(self) -> &'static str {
        match self {
            Self::NeedsLiveProofInputKeyboard => "NeedsLiveProofInputKeyboard",
            Self::NeedsLiveProofInputPointer => "NeedsLiveProofInputPointer",
            Self::NeedsLiveProofInputTouchpad => "NeedsLiveProofInputTouchpad",
            Self::NeedsLiveProofCursor => "NeedsLiveProofCursor",
            Self::NeedsLiveProofFocus => "NeedsLiveProofFocus",
            Self::NeedsLiveProofGesture => "NeedsLiveProofGesture",
            Self::ProofModelOnly => "ProofModelOnly",
            Self::ProofBlockedNoSafeFallback => "ProofBlockedNoSafeFallback",
            Self::ProofBlockedNoRuntimeVerification => "ProofBlockedNoRuntimeVerification",
            Self::ProofBlockedTooDangerous => "ProofBlockedTooDangerous",
            Self::ProofReadyForEnvGatedLiveTest => "ProofReadyForEnvGatedLiveTest",
            Self::ProofPassedArmableCandidate => "ProofPassedArmableCandidate",
        }
    }

    pub fn live_proof_may_run(self) -> bool {
        self == Self::ProofReadyForEnvGatedLiveTest
    }

    pub fn armable(self) -> bool {
        self == Self::ProofPassedArmableCandidate
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
pub enum RuntimePreviewInputSubsystem {
    Keyboard,
    Pointer,
    Touchpad,
    TabletTouch,
    Cursor,
    Focus,
}

impl RuntimePreviewInputSubsystem {
    pub fn as_str(self) -> &'static str {
        match self {
            Self::Keyboard => "Keyboard",
            Self::Pointer => "Pointer",
            Self::Touchpad => "Touchpad",
            Self::TabletTouch => "TabletTouch",
            Self::Cursor => "Cursor",
            Self::Focus => "Focus",
        }
    }
}

#[derive(Debug, Clone, Serialize)]
pub struct RuntimePreviewInputFallbackPath {
    pub keyboard_remains_usable: bool,
    pub pointer_remains_usable: bool,
    pub external_devices_help: bool,
    pub timeout_auto_revert_needs_no_input: bool,
    pub tty_rollback_instruction: &'static str,
    pub summary: String,
}

#[derive(Debug, Clone, Serialize)]
pub struct RuntimePreviewInputProofPlan {
    pub row_id: &'static str,
    pub official_setting: &'static str,
    pub value_kind: String,
    pub category: RuntimePreviewInputSubsystem,
    pub current_dead_man_classification: &'static str,
    pub proof_classification: RuntimePreviewInputProofClassification,
    pub what_it_controls: String,
    pub what_could_go_wrong: String,
    pub affects_keyboard: bool,
    pub affects_pointer: bool,
    pub affects_focus: bool,
    pub fallback: RuntimePreviewInputFallbackPath,
    pub original_value_capture: &'static str,
    pub minimal_preview_value: String,
    pub apply_strategy: &'static str,
    pub revert_strategy: &'static str,
    pub verification_strategy: &'static str,
    pub manual_warning: String,
    pub live_proof_env: String,
    pub live_proof_exists: bool,
    pub promotion_decision: &'static str,
    pub blocked_reason: Option<&'static str>,
    pub recovery_instruction: String,
}

fn subsystem_for(official_setting: &str) -> RuntimePreviewInputSubsystem {
    if official_setting.starts_with("cursor.") {
        RuntimePreviewInputSubsystem::Cursor
    } else if official_setting.starts_with("input.touchpad.") {
        RuntimePreviewInputSubsystem::Touchpad
    } else if official_setting.starts_with("input.tablet.")
        || official_setting.starts_with("input.touchdevice.")
    {
        RuntimePreviewInputSubsystem::TabletTouch
    } else if official_setting.starts_with("input.virtualkeyboard.")
        || matches!(
            official_setting,
            "input.numlock_by_default"
                | "input.repeat_delay"
                | "input.repeat_rate"
                | "input.resolve_binds_by_sym"
        )
    {
        RuntimePreviewInputSubsystem::Keyboard
    } else if matches!(
        official_setting,
        "input.follow_mouse"
            | "input.follow_mouse_shrink"
            | "input.follow_mouse_threshold"
            | "input.float_switch_override_focus"
            | "input.focus_on_close"
            | "input.mouse_refocus"
            | "input.special_fallthrough"
    ) {
        RuntimePreviewInputSubsystem::Focus
    } else {
        RuntimePreviewInputSubsystem::Pointer
    }
}

fn minimal_preview_value(official_setting: &str, value_kind: ScalarWriteValueKind) -> String {
    match official_setting {
        "cursor.inactive_timeout" => "1".to_string(),
        "cursor.hotspot_padding" => "2".to_string(),
        "cursor.zoom_factor" => "1.1".to_string(),
        "input.repeat_rate" => "25".to_string(),
        "input.repeat_delay" => "600".to_string(),
        "input.sensitivity" => "0".to_string(),
        "input.scroll_factor" => "1".to_string(),
        "input.touchpad.scroll_factor" => "1".to_string(),
        "input.rotation" => "0".to_string(),
        "input.scroll_button" => "0".to_string(),
        "input.follow_mouse_shrink" => "0".to_string(),
        "input.follow_mouse_threshold" => "0".to_string(),
        "input.tablet.transform" => "0".to_string(),
        "input.touchdevice.transform" => "0".to_string(),
        _ => match value_kind {
            ScalarWriteValueKind::Boolean => {
                "toggle of the captured original (single boolean flip)".to_string()
            }
            ScalarWriteValueKind::FiniteChoice => {
                "the nearest allowed non-default choice from the row's finite list".to_string()
            }
            _ => "the captured original value (no-op probe) before any different value".to_string(),
        },
    }
}

fn proof_classification_for(
    official_setting: &str,
    subsystem: RuntimePreviewInputSubsystem,
) -> (
    RuntimePreviewInputProofClassification,
    &'static str,
    Option<&'static str>,
) {
    if proven_input_row(official_setting).is_some() {
        return (
            RuntimePreviewInputProofClassification::ProofPassedArmableCandidate,
            "promoted: the env-gated live proof ran and verified apply and revert with all input paths usable",
            None,
        );
    }
    match official_setting {
        "cursor.inactive_timeout" => (
            RuntimePreviewInputProofClassification::ProofReadyForEnvGatedLiveTest,
            "run the env-gated live proof; promote only if it passes",
            None,
        ),
        "cursor.invisible" => (
            RuntimePreviewInputProofClassification::ProofBlockedTooDangerous,
            "stay blocked",
            Some(
                "the preview value itself removes cursor visibility entirely; even with timeout auto-revert, a user-facing armed preview of an invisible cursor is not worth the risk",
            ),
        ),
        "cursor.no_hardware_cursors"
        | "cursor.use_cpu_buffer"
        | "cursor.min_refresh_rate"
        | "cursor.no_break_fs_vrr" => (
            RuntimePreviewInputProofClassification::ProofBlockedNoRuntimeVerification,
            "stay blocked",
            Some(
                "these switch the cursor rendering pipeline; getoption can verify the option value but not that the cursor still renders correctly, and a glitched cursor cannot be detected by the app",
            ),
        ),
        _ => {
            let classification = match subsystem {
                RuntimePreviewInputSubsystem::Cursor => {
                    RuntimePreviewInputProofClassification::NeedsLiveProofCursor
                }
                RuntimePreviewInputSubsystem::Keyboard => {
                    RuntimePreviewInputProofClassification::NeedsLiveProofInputKeyboard
                }
                RuntimePreviewInputSubsystem::Pointer => {
                    RuntimePreviewInputProofClassification::NeedsLiveProofInputPointer
                }
                RuntimePreviewInputSubsystem::Touchpad
                | RuntimePreviewInputSubsystem::TabletTouch => {
                    RuntimePreviewInputProofClassification::NeedsLiveProofInputTouchpad
                }
                RuntimePreviewInputSubsystem::Focus => {
                    RuntimePreviewInputProofClassification::NeedsLiveProofFocus
                }
            };
            (
                classification,
                "disarmed until a per-row env-gated live proof passes",
                None,
            )
        }
    }
}

fn fallback_for(
    official_setting: &str,
    subsystem: RuntimePreviewInputSubsystem,
) -> RuntimePreviewInputFallbackPath {
    let (keyboard, pointer, summary): (bool, bool, String) = match subsystem {
        RuntimePreviewInputSubsystem::Cursor => (
            true,
            true,
            "cursor rows change cursor appearance/warp behavior only; keyboard and pointer events keep flowing".to_string(),
        ),
        RuntimePreviewInputSubsystem::Keyboard => (
            false,
            true,
            "keyboard behavior may change; the pointer remains usable to click Revert/Cancel, and timeout auto-revert needs no input at all".to_string(),
        ),
        RuntimePreviewInputSubsystem::Pointer => (
            true,
            false,
            "pointer behavior may change (speed, handedness, scrolling); the keyboard remains usable and timeout auto-revert needs no input".to_string(),
        ),
        RuntimePreviewInputSubsystem::Touchpad | RuntimePreviewInputSubsystem::TabletTouch => (
            true,
            true,
            "touchpad/tablet/touch rows do not affect external mice or keyboards; on this class of machine a non-touch input path stays available".to_string(),
        ),
        RuntimePreviewInputSubsystem::Focus => (
            true,
            true,
            "focus-follow behavior changes which window receives events, but both devices keep working; keyboard navigation and timeout auto-revert remain available".to_string(),
        ),
    };
    let _ = official_setting;
    RuntimePreviewInputFallbackPath {
        keyboard_remains_usable: keyboard,
        pointer_remains_usable: pointer,
        external_devices_help: true,
        timeout_auto_revert_needs_no_input: true,
        tty_rollback_instruction:
            "worst case: switch to a TTY (Ctrl+Alt+F3) and run `hyprctl eval` with the recorded original value, or restart Hyprland; the config file was never modified",
        summary,
    }
}

pub fn input_proof_plan(row_id: &str) -> Option<RuntimePreviewInputProofPlan> {
    let row = SAFE_WRITABLE_ROWS.iter().find(|row| row.row_id == row_id)?;
    let dead_man = classify_dead_man_row(row_id)?;
    // Proof plans exist for the needs-live-proof set and for promoted rows.
    if !matches!(
        dead_man.classification,
        RuntimePreviewDeadManClassification::DeadManPreviewCandidateNeedsLiveProof
    ) && proven_input_row(row.official_setting).is_none()
    {
        return None;
    }

    let subsystem = subsystem_for(row.official_setting);
    let (proof_classification, promotion_decision, blocked_reason) =
        proof_classification_for(row.official_setting, subsystem);
    let fallback = fallback_for(row.official_setting, subsystem);

    let affects_keyboard = subsystem == RuntimePreviewInputSubsystem::Keyboard;
    let affects_pointer = matches!(
        subsystem,
        RuntimePreviewInputSubsystem::Pointer | RuntimePreviewInputSubsystem::Touchpad
    );
    let affects_focus = subsystem == RuntimePreviewInputSubsystem::Focus;

    let what_it_controls = match subsystem {
        RuntimePreviewInputSubsystem::Cursor => format!(
            "{} controls cursor appearance, visibility timing, warping, or zoom behavior",
            row.official_setting
        ),
        RuntimePreviewInputSubsystem::Keyboard => format!(
            "{} controls keyboard behavior (repeat, numlock, bind resolution, or virtual keyboards)",
            row.official_setting
        ),
        RuntimePreviewInputSubsystem::Pointer => format!(
            "{} controls pointer behavior (acceleration, handedness, scrolling, or axis events)",
            row.official_setting
        ),
        RuntimePreviewInputSubsystem::Touchpad => format!(
            "{} controls touchpad behavior (tapping, scrolling, palm rejection, or gestures)",
            row.official_setting
        ),
        RuntimePreviewInputSubsystem::TabletTouch => format!(
            "{} controls tablet or touch-device mapping and transforms",
            row.official_setting
        ),
        RuntimePreviewInputSubsystem::Focus => format!(
            "{} controls which window receives input focus as the mouse moves or windows close",
            row.official_setting
        ),
    };
    let what_could_go_wrong = match subsystem {
        RuntimePreviewInputSubsystem::Cursor => {
            "the cursor could hide, warp unexpectedly, or zoom disorientingly; input events themselves keep flowing".to_string()
        }
        RuntimePreviewInputSubsystem::Keyboard => {
            "typing behavior could change mid-session (repeat storms, numlock state); confirming with the keyboard could become harder".to_string()
        }
        RuntimePreviewInputSubsystem::Pointer => {
            "pointer speed, direction, button mapping, or scrolling could change so the mouse becomes hard to use for clicking Revert".to_string()
        }
        RuntimePreviewInputSubsystem::Touchpad | RuntimePreviewInputSubsystem::TabletTouch => {
            "the touch device could behave unexpectedly; on devices where it is the only pointer, that impairs clicking".to_string()
        }
        RuntimePreviewInputSubsystem::Focus => {
            "focus could jump to the wrong window, sending confirm clicks or keystrokes somewhere unexpected".to_string()
        }
    };

    let live_proof_exists = proof_classification.live_proof_may_run()
        || proof_classification
            == RuntimePreviewInputProofClassification::ProofPassedArmableCandidate;
    let tty_rollback_instruction = fallback.tty_rollback_instruction;
    let keyboard_fallback = fallback.keyboard_remains_usable;

    Some(RuntimePreviewInputProofPlan {
        row_id: row.row_id,
        official_setting: row.official_setting,
        value_kind: format!("{:?}", row.value_kind),
        category: subsystem,
        current_dead_man_classification: dead_man.classification.as_str(),
        proof_classification,
        what_it_controls,
        what_could_go_wrong,
        affects_keyboard,
        affects_pointer,
        affects_focus,
        fallback,
        original_value_capture:
            "read the current runtime value via the read-only getoption query before any mutation; the supervised session refuses to start without it",
        minimal_preview_value: minimal_preview_value(row.official_setting, row.value_kind),
        apply_strategy:
            "one supervised runtime set through the dead-man executor path (the proven scalar runtime-set mechanism); no config write, no reload",
        revert_strategy:
            "reapply the captured original via the same supervised runtime set; timeout auto-revert fires without user input",
        verification_strategy:
            "read the option back via getoption after apply and after revert and compare against the intended and original values",
        manual_warning: format!(
            "This changes live {} behavior. Keep a second input path ready ({}).",
            subsystem.as_str().to_lowercase(),
            if keyboard_fallback {
                "keyboard stays usable"
            } else {
                "pointer stays usable"
            }
        ),
        live_proof_env: format!(
            "HYPRLAND_SETTINGS_RUN_INPUT_LIVE_PROOF=1 HYPRLAND_SETTINGS_INPUT_PROOF_ROW={}",
            row.official_setting
        ),
        live_proof_exists,
        promotion_decision,
        blocked_reason,
        recovery_instruction: format!(
            "If input feels wrong during a supervised preview of {}, do nothing: the countdown reverts automatically. {}",
            row.official_setting, tty_rollback_instruction
        ),
    })
}

pub fn input_proof_plans() -> Vec<RuntimePreviewInputProofPlan> {
    SAFE_WRITABLE_ROWS
        .iter()
        .filter_map(|row| input_proof_plan(row.row_id))
        .collect()
}

/// Gate for the env-gated live proof harness: only proof-ready rows with a
/// usable fallback may run, and only with the explicit env vars present.
pub fn live_proof_gate(official_setting: &str) -> Result<RuntimePreviewInputProofPlan, String> {
    let plan = SAFE_WRITABLE_ROWS
        .iter()
        .find(|row| row.official_setting == official_setting)
        .and_then(|row| input_proof_plan(row.row_id))
        .ok_or_else(|| format!("no input proof plan exists for {official_setting}"))?;
    if !(plan.fallback.keyboard_remains_usable || plan.fallback.pointer_remains_usable) {
        return Err(format!(
            "{official_setting} has no usable fallback input path; live proof refused"
        ));
    }
    if !plan.proof_classification.live_proof_may_run() {
        return Err(format!(
            "{official_setting} is classified {} and is not proof-ready; live proof refused",
            plan.proof_classification.as_str()
        ));
    }
    Ok(plan)
}

#[derive(Debug, Clone, Serialize)]
pub struct RuntimePreviewInputProofSummary {
    pub needs_live_proof_rows_total: usize,
    pub keyboard_risk_rows: usize,
    pub pointer_risk_rows: usize,
    pub touchpad_risk_rows: usize,
    pub cursor_risk_rows: usize,
    pub focus_risk_rows: usize,
    pub gesture_risk_rows: usize,
    pub proof_model_only_rows: usize,
    pub proof_blocked_no_safe_fallback_rows: usize,
    pub proof_blocked_no_runtime_verification_rows: usize,
    pub proof_blocked_too_dangerous_rows: usize,
    pub proof_ready_for_env_gated_live_test_rows: usize,
    pub proof_passed_armable_candidate_rows: usize,
}

pub fn input_proof_summary() -> RuntimePreviewInputProofSummary {
    let plans = input_proof_plans();
    let count = |classification: RuntimePreviewInputProofClassification| {
        plans
            .iter()
            .filter(|plan| plan.proof_classification == classification)
            .count()
    };
    RuntimePreviewInputProofSummary {
        needs_live_proof_rows_total: plans.len(),
        keyboard_risk_rows: count(
            RuntimePreviewInputProofClassification::NeedsLiveProofInputKeyboard,
        ),
        pointer_risk_rows: count(
            RuntimePreviewInputProofClassification::NeedsLiveProofInputPointer,
        ),
        touchpad_risk_rows: count(
            RuntimePreviewInputProofClassification::NeedsLiveProofInputTouchpad,
        ),
        cursor_risk_rows: count(RuntimePreviewInputProofClassification::NeedsLiveProofCursor),
        focus_risk_rows: count(RuntimePreviewInputProofClassification::NeedsLiveProofFocus),
        gesture_risk_rows: count(RuntimePreviewInputProofClassification::NeedsLiveProofGesture),
        proof_model_only_rows: count(RuntimePreviewInputProofClassification::ProofModelOnly),
        proof_blocked_no_safe_fallback_rows: count(
            RuntimePreviewInputProofClassification::ProofBlockedNoSafeFallback,
        ),
        proof_blocked_no_runtime_verification_rows: count(
            RuntimePreviewInputProofClassification::ProofBlockedNoRuntimeVerification,
        ),
        proof_blocked_too_dangerous_rows: count(
            RuntimePreviewInputProofClassification::ProofBlockedTooDangerous,
        ),
        proof_ready_for_env_gated_live_test_rows: count(
            RuntimePreviewInputProofClassification::ProofReadyForEnvGatedLiveTest,
        ),
        proof_passed_armable_candidate_rows: count(
            RuntimePreviewInputProofClassification::ProofPassedArmableCandidate,
        ),
    }
}
