//! Supervised preview controllers for the two live-proven structured-family
//! records: the `global` animation node (hl.animation) and the `default`
//! bezier curve (hl.curve).
//!
//! Scope is modify-existing only, exactly as proven: the controller refuses
//! to arm unless the record already exists in the runtime readback, captures
//! the original values first, applies validated minimal changes through
//! fixed-shape commands (no free-form command construction), verifies every
//! step through the read-only animations listing, runs under the dead-man
//! countdown, and restores the exact original values on revert, cancel,
//! timeout, or session drop. There is no record creation and no deletion —
//! those operations do not exist in this module.

use serde::Serialize;

use crate::runtime_preview_executor::{
    HyprctlRuntimePreviewRunner, RuntimePreviewDeadMan, RuntimePreviewDeadManVerdict,
    RuntimePreviewRunner,
};
use crate::structured_family_runtime_preview::{
    parse_animation_leaf, parse_bezier_points, proven_family_record_proof,
};

pub const FAMILY_PREVIEW_COUNTDOWN_MS: u64 = 10_000;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
pub enum FamilyPreviewPhase {
    Disarmed,
    CountingDown,
    Kept,
    Reverted,
    TimedOutReverted,
    Cancelled,
}

impl FamilyPreviewPhase {
    pub fn as_str(self) -> &'static str {
        match self {
            Self::Disarmed => "Disarmed",
            Self::CountingDown => "Counting down",
            Self::Kept => "Kept",
            Self::Reverted => "Reverted",
            Self::TimedOutReverted => "Timed out and reverted",
            Self::Cancelled => "Cancelled",
        }
    }
}

#[derive(Debug, Clone, Serialize)]
pub struct FamilyPreviewReceipt {
    pub family_id: &'static str,
    pub record: &'static str,
    pub action: &'static str,
    pub phase: FamilyPreviewPhase,
    pub original: Option<String>,
    pub applied: Option<String>,
    pub config_written: bool,
    pub reload_run: bool,
    pub status_text: String,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum FamilyPreviewError {
    FamilyNotProven(&'static str),
    RecordMissing(&'static str),
    InvalidValue(String),
    NotArmed,
    Runner(String),
    VerificationFailed(String),
}

impl FamilyPreviewError {
    pub fn user_text(&self) -> String {
        match self {
            Self::FamilyNotProven(reason) => format!("Family preview unavailable: {reason}"),
            Self::RecordMissing(record) => format!(
                "Record {record} was not found in the runtime readback; modify-existing preview refuses to run"
            ),
            Self::InvalidValue(detail) => format!("Value rejected: {detail}"),
            Self::NotArmed => "Start a supervised family preview first".to_string(),
            Self::Runner(detail) => format!("Preview failed: {detail}"),
            Self::VerificationFailed(detail) => format!("Verification failed: {detail}"),
        }
    }
}

fn read_animations(runner: &mut dyn RuntimePreviewRunner) -> Result<String, FamilyPreviewError> {
    runner
        .run("hyprctl", &["animations".to_string()])
        .map_err(FamilyPreviewError::Runner)
}

/// The proven target records. Only these two exist; there is no general
/// record targeting in this controller.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
pub enum FamilyPreviewTarget {
    AnimationGlobalSpeed,
    CurveDefaultY0,
}

impl FamilyPreviewTarget {
    pub fn family_id(self) -> &'static str {
        match self {
            Self::AnimationGlobalSpeed => "hl.animation",
            Self::CurveDefaultY0 => "hl.curve",
        }
    }

    pub fn record(self) -> &'static str {
        match self {
            Self::AnimationGlobalSpeed => "global",
            Self::CurveDefaultY0 => "default",
        }
    }

    pub fn control_label(self) -> &'static str {
        match self {
            Self::AnimationGlobalSpeed => "Global animation speed",
            Self::CurveDefaultY0 => "Default curve first control point (Y0)",
        }
    }
}

#[derive(Debug, Clone)]
enum CapturedOriginal {
    Animation {
        enabled: String,
        speed: String,
        bezier: String,
    },
    Curve {
        x0: f64,
        y0: f64,
        x1: f64,
        y1: f64,
    },
}

/// Supervised controller for one proven family record.
pub struct FamilyPreviewController {
    runner: Box<dyn RuntimePreviewRunner>,
    target: FamilyPreviewTarget,
    original: Option<CapturedOriginal>,
    phase: FamilyPreviewPhase,
    dead_man: Option<RuntimePreviewDeadMan>,
}

impl FamilyPreviewController {
    pub fn new(
        target: FamilyPreviewTarget,
        runner: Box<dyn RuntimePreviewRunner>,
    ) -> Result<Self, FamilyPreviewError> {
        if proven_family_record_proof(target.family_id()).is_none() {
            return Err(FamilyPreviewError::FamilyNotProven(
                "no passed live proof receipt exists for this family",
            ));
        }
        Ok(Self {
            runner,
            target,
            original: None,
            phase: FamilyPreviewPhase::Disarmed,
            dead_man: None,
        })
    }

    pub fn new_live(target: FamilyPreviewTarget) -> Result<Self, FamilyPreviewError> {
        Self::new(target, Box::new(HyprctlRuntimePreviewRunner))
    }

    pub fn phase(&self) -> FamilyPreviewPhase {
        self.phase
    }

    pub fn remaining_seconds(&self) -> u64 {
        self.dead_man
            .as_ref()
            .map(|dead_man| {
                dead_man
                    .timeout_ms
                    .saturating_sub(dead_man.elapsed_ms)
                    .div_ceil(1000)
            })
            .unwrap_or(0)
    }

    /// Read the current value of the proven field (read-only).
    pub fn current_value(&mut self) -> Result<String, FamilyPreviewError> {
        let listing = read_animations(self.runner.as_mut())?;
        match self.target {
            FamilyPreviewTarget::AnimationGlobalSpeed => {
                let (_, speed, _) = parse_animation_leaf(&listing, "global")
                    .ok_or(FamilyPreviewError::RecordMissing("global"))?;
                Ok(speed)
            }
            FamilyPreviewTarget::CurveDefaultY0 => {
                let (_, y0, _, _) = parse_bezier_points(&listing, "default")
                    .ok_or(FamilyPreviewError::RecordMissing("default"))?;
                Ok(format!("{y0}"))
            }
        }
    }

    /// Apply a validated preview value. Arms the session (capturing the full
    /// original record) on first use and starts/restarts the countdown.
    /// Modify-existing is enforced: a missing record refuses before any
    /// command is issued.
    pub fn preview(&mut self, value: f64) -> Result<FamilyPreviewReceipt, FamilyPreviewError> {
        if !value.is_finite() {
            return Err(FamilyPreviewError::InvalidValue(
                "value must be finite".to_string(),
            ));
        }
        let listing = read_animations(self.runner.as_mut())?;
        let expression = match self.target {
            FamilyPreviewTarget::AnimationGlobalSpeed => {
                if !(0.1..=20.0).contains(&value) {
                    return Err(FamilyPreviewError::InvalidValue(
                        "global animation speed previews are limited to 0.1..=20".to_string(),
                    ));
                }
                let (enabled, speed, bezier) = parse_animation_leaf(&listing, "global")
                    .ok_or(FamilyPreviewError::RecordMissing("global"))?;
                if self.original.is_none() {
                    self.original = Some(CapturedOriginal::Animation {
                        enabled: enabled.clone(),
                        speed,
                        bezier: bezier.clone(),
                    });
                }
                let enabled_lua = if enabled == "1" { "true" } else { "false" };
                let bezier_name = if bezier.is_empty() {
                    "default"
                } else {
                    &bezier
                };
                format!(
                    "hl.animation({{ leaf = \"global\", enabled = {enabled_lua}, speed = {value}, bezier = \"{bezier_name}\" }})"
                )
            }
            FamilyPreviewTarget::CurveDefaultY0 => {
                if !(-1.0..=2.0).contains(&value) {
                    return Err(FamilyPreviewError::InvalidValue(
                        "curve control point previews are limited to -1..=2".to_string(),
                    ));
                }
                let (x0, y0, x1, y1) = parse_bezier_points(&listing, "default")
                    .ok_or(FamilyPreviewError::RecordMissing("default"))?;
                if self.original.is_none() {
                    self.original = Some(CapturedOriginal::Curve { x0, y0, x1, y1 });
                }
                format!(
                    "hl.curve(\"default\", {{ type = \"bezier\", points = {{ {{{x0}, {value}}}, {{{x1}, {y1}}} }} }})"
                )
            }
        };
        self.runner
            .run("hyprctl", &["eval".to_string(), expression])
            .map_err(FamilyPreviewError::Runner)?;

        // Verify the apply through readback.
        let observed = self.current_value()?;
        let observed_value: f64 = observed
            .parse()
            .map_err(|_| FamilyPreviewError::VerificationFailed("readback did not parse".into()))?;
        if (observed_value - value).abs() > 1e-3 {
            return Err(FamilyPreviewError::VerificationFailed(format!(
                "expected {value}, observed {observed}"
            )));
        }

        self.phase = FamilyPreviewPhase::CountingDown;
        let mut dead_man = RuntimePreviewDeadMan::new(FAMILY_PREVIEW_COUNTDOWN_MS);
        dead_man.recovery_instruction =
            "if the app exits during an unconfirmed family preview, the change was runtime-only; reapply the recorded original values or restart Hyprland";
        self.dead_man = Some(dead_man);
        Ok(self.receipt(
            "preview",
            Some(observed),
            format!(
                "Previewing live: {} = {value} (auto-revert in {} seconds unless you Keep changes)",
                self.target.control_label(),
                FAMILY_PREVIEW_COUNTDOWN_MS / 1000
            ),
        ))
    }

    /// Advance the countdown; auto-reverts on timeout.
    pub fn tick(
        &mut self,
        delta_ms: u64,
    ) -> Result<Option<FamilyPreviewReceipt>, FamilyPreviewError> {
        if self.phase != FamilyPreviewPhase::CountingDown {
            return Ok(None);
        }
        let verdict = match self.dead_man.as_mut() {
            Some(dead_man) => {
                dead_man.tick(delta_ms);
                dead_man.evaluate()
            }
            None => RuntimePreviewDeadManVerdict::KeepActive,
        };
        if verdict == RuntimePreviewDeadManVerdict::RevertRequired {
            let receipt = self.revert_internal("timeout")?;
            self.phase = FamilyPreviewPhase::TimedOutReverted;
            return Ok(Some(FamilyPreviewReceipt {
                phase: FamilyPreviewPhase::TimedOutReverted,
                ..receipt
            }));
        }
        Ok(None)
    }

    /// Keep the previewed value for this session (runtime-only; persistence
    /// is a separate gated step through Safe Live Save Mode).
    pub fn keep(&mut self) -> Result<FamilyPreviewReceipt, FamilyPreviewError> {
        if self.phase != FamilyPreviewPhase::CountingDown {
            return Err(FamilyPreviewError::NotArmed);
        }
        if let Some(dead_man) = self.dead_man.as_mut() {
            dead_man.confirm();
        }
        self.phase = FamilyPreviewPhase::Kept;
        Ok(self.receipt(
            "keep",
            None,
            format!(
                "Kept: {} stays at the previewed value for this session. Nothing was saved to your config.",
                self.target.control_label()
            ),
        ))
    }

    fn revert_internal(
        &mut self,
        action: &'static str,
    ) -> Result<FamilyPreviewReceipt, FamilyPreviewError> {
        let original = self.original.clone().ok_or(FamilyPreviewError::NotArmed)?;
        let (expression, original_text) = match (&original, self.target) {
            (
                CapturedOriginal::Animation {
                    enabled,
                    speed,
                    bezier,
                },
                FamilyPreviewTarget::AnimationGlobalSpeed,
            ) => {
                let enabled_lua = if enabled == "1" { "true" } else { "false" };
                let bezier_name = if bezier.is_empty() { "default" } else { bezier };
                let speed_value: f64 = speed
                    .parse()
                    .map_err(|_| FamilyPreviewError::VerificationFailed("original speed".into()))?;
                (
                    format!(
                        "hl.animation({{ leaf = \"global\", enabled = {enabled_lua}, speed = {speed_value}, bezier = \"{bezier_name}\" }})"
                    ),
                    speed.clone(),
                )
            }
            (CapturedOriginal::Curve { x0, y0, x1, y1 }, FamilyPreviewTarget::CurveDefaultY0) => (
                format!(
                    "hl.curve(\"default\", {{ type = \"bezier\", points = {{ {{{x0}, {y0}}}, {{{x1}, {y1}}} }} }})"
                ),
                format!("{y0}"),
            ),
            _ => {
                return Err(FamilyPreviewError::VerificationFailed(
                    "captured original does not match the target".to_string(),
                ))
            }
        };
        self.runner
            .run("hyprctl", &["eval".to_string(), expression])
            .map_err(FamilyPreviewError::Runner)?;
        // Verify exact restore through readback.
        let observed = self.current_value()?;
        let observed_value: f64 = observed
            .parse()
            .map_err(|_| FamilyPreviewError::VerificationFailed("readback did not parse".into()))?;
        let original_value: f64 = original_text
            .parse()
            .map_err(|_| FamilyPreviewError::VerificationFailed("original did not parse".into()))?;
        if (observed_value - original_value).abs() > 1e-6 {
            return Err(FamilyPreviewError::VerificationFailed(format!(
                "restore expected {original_text}, observed {observed}"
            )));
        }
        Ok(self.receipt(
            action,
            Some(observed),
            format!(
                "Reverted: {} restored to {original_text} (verified via readback).",
                self.target.control_label()
            ),
        ))
    }

    /// Manual revert (works during countdown and after Keep).
    pub fn revert_now(&mut self) -> Result<FamilyPreviewReceipt, FamilyPreviewError> {
        if !matches!(
            self.phase,
            FamilyPreviewPhase::CountingDown | FamilyPreviewPhase::Kept
        ) {
            return Err(FamilyPreviewError::NotArmed);
        }
        let receipt = self.revert_internal("revert-now")?;
        self.phase = FamilyPreviewPhase::Reverted;
        Ok(receipt)
    }

    /// Cancel: revert and disarm.
    pub fn cancel(&mut self) -> Result<FamilyPreviewReceipt, FamilyPreviewError> {
        let mut receipt = self.revert_internal("cancel")?;
        self.phase = FamilyPreviewPhase::Cancelled;
        self.original = None;
        self.dead_man = None;
        receipt.phase = FamilyPreviewPhase::Cancelled;
        Ok(receipt)
    }

    /// Session-drop / app-close recovery: revert unconfirmed previews only.
    pub fn revert_if_unconfirmed(&mut self) -> Option<FamilyPreviewReceipt> {
        if self.phase == FamilyPreviewPhase::CountingDown {
            let receipt = self.revert_internal("session-drop").ok()?;
            self.phase = FamilyPreviewPhase::Reverted;
            return Some(receipt);
        }
        None
    }

    fn receipt(
        &self,
        action: &'static str,
        applied: Option<String>,
        status_text: String,
    ) -> FamilyPreviewReceipt {
        FamilyPreviewReceipt {
            family_id: self.target.family_id(),
            record: self.target.record(),
            action,
            phase: self.phase,
            original: self.original.as_ref().map(|original| match original {
                CapturedOriginal::Animation { speed, .. } => speed.clone(),
                CapturedOriginal::Curve { y0, .. } => format!("{y0}"),
            }),
            applied,
            config_written: false,
            reload_run: false,
            status_text,
        }
    }
}
