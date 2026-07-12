//! Supervised dead-man live preview: classification, UI state, and the
//! session controller for the 78 dead-man-gated rows.
//!
//! A dead-man preview is an explicitly armed, countdown-supervised runtime
//! mutation: the app captures the original value, applies the preview, and
//! automatically reverts unless the user confirms "Keep changes" before the
//! timeout. Nothing persists, nothing writes config, nothing reloads.
//!
//! Not all 78 rows are enabled. Each row is reclassified honestly:
//! only rows whose runtime mechanism, value grammar, and revert path are
//! proven get an armed button; everything else shows its specific reason.
//! Monitor/display rows are not in this set at all — they remain in the
//! blocked-high-risk bucket and are untouched by this module.

use serde::Serialize;

use crate::runtime_preview::{classify_runtime_preview_row, RuntimePreviewCapability};
use crate::runtime_preview_executor::{
    apply_runtime_preview_value_supervised, revert_runtime_preview_session_supervised,
    start_runtime_preview_session, HyprctlRuntimePreviewRunner, RuntimePreviewDeadManVerdict,
    RuntimePreviewError, RuntimePreviewRunner, RuntimePreviewSession, RuntimePreviewSessionState,
};
use crate::write_classification::{ScalarWriteValueKind, SAFE_WRITABLE_ROWS};

pub const DEAD_MAN_COUNTDOWN_SECONDS: u64 = 10;

pub const DEAD_MAN_RECOVERY_INSTRUCTION: &str =
    "If this app exits during an unconfirmed supervised preview, the change was runtime-only: restart Hyprland or re-apply the recorded original value; your config file was never modified.";

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
pub enum RuntimePreviewDeadManClassification {
    DeadManPreviewCandidate,
    DeadManPreviewCandidateNeedsLiveProof,
    DeadManPreviewModelOnly,
    DeadManPreviewBlockedNoSafeRuntimeMechanism,
    DeadManPreviewBlockedRequiresRelog,
    DeadManPreviewBlockedRequiresRestart,
    DeadManPreviewBlockedNoVisibleEffect,
    DeadManPreviewBlockedTooDangerous,
}

impl RuntimePreviewDeadManClassification {
    pub fn as_str(self) -> &'static str {
        match self {
            Self::DeadManPreviewCandidate => "DeadManPreviewCandidate",
            Self::DeadManPreviewCandidateNeedsLiveProof => "DeadManPreviewCandidateNeedsLiveProof",
            Self::DeadManPreviewModelOnly => "DeadManPreviewModelOnly",
            Self::DeadManPreviewBlockedNoSafeRuntimeMechanism => {
                "DeadManPreviewBlockedNoSafeRuntimeMechanism"
            }
            Self::DeadManPreviewBlockedRequiresRelog => "DeadManPreviewBlockedRequiresRelog",
            Self::DeadManPreviewBlockedRequiresRestart => "DeadManPreviewBlockedRequiresRestart",
            Self::DeadManPreviewBlockedNoVisibleEffect => "DeadManPreviewBlockedNoVisibleEffect",
            Self::DeadManPreviewBlockedTooDangerous => "DeadManPreviewBlockedTooDangerous",
        }
    }

    /// Only proven candidates get an armed supervised preview button.
    pub fn supervised_preview_enabled(self) -> bool {
        self == Self::DeadManPreviewCandidate
    }

    /// Candidates and model-only rows render the supervised panel (armed or
    /// disarmed); blocked rows render only their reason.
    pub fn shows_supervised_panel(self) -> bool {
        matches!(
            self,
            Self::DeadManPreviewCandidate
                | Self::DeadManPreviewCandidateNeedsLiveProof
                | Self::DeadManPreviewModelOnly
        )
    }
}

#[derive(Debug, Clone, Serialize)]
pub struct RuntimePreviewDeadManRowClassification {
    pub row_id: &'static str,
    pub official_setting: &'static str,
    pub value_kind: String,
    pub classification: RuntimePreviewDeadManClassification,
    pub reason: &'static str,
}

fn grammar_is_proven_scalar(kind: ScalarWriteValueKind) -> bool {
    matches!(
        kind,
        ScalarWriteValueKind::Boolean
            | ScalarWriteValueKind::FiniteChoice
            | ScalarWriteValueKind::Number
            | ScalarWriteValueKind::Percent
    )
}

/// Reclassify one dead-man row. Returns None for rows that are not dead-man
/// gated in the capability matrix.
pub fn classify_dead_man_row(row_id: &str) -> Option<RuntimePreviewDeadManRowClassification> {
    let row = SAFE_WRITABLE_ROWS.iter().find(|row| row.row_id == row_id)?;
    let capability = classify_runtime_preview_row(row);
    if capability.capability != RuntimePreviewCapability::LivePreviewSupportedWithDeadMan {
        return None;
    }

    let is_animation = row.official_setting.starts_with("animations.");
    let (classification, reason) = if is_animation && grammar_is_proven_scalar(row.value_kind) {
        (
            RuntimePreviewDeadManClassification::DeadManPreviewCandidate,
            "animation toggles are visual, use the proven scalar runtime mechanism, and auto-revert reliably; supervision covers the performance risk",
        )
    } else if matches!(
        row.value_kind,
        ScalarWriteValueKind::Vector2
            | ScalarWriteValueKind::NumericList
            | ScalarWriteValueKind::MonitorName
            | ScalarWriteValueKind::Path
    ) {
        (
            RuntimePreviewDeadManClassification::DeadManPreviewBlockedNoSafeRuntimeMechanism,
            "this value grammar (multi-component vector/list, monitor name, or path) has no proven runtime-set representation; previewing it live could apply a malformed value to input hardware",
        )
    } else if matches!(row.value_kind, ScalarWriteValueKind::SourceBacked) {
        (
            RuntimePreviewDeadManClassification::DeadManPreviewModelOnly,
            "source-backed string values would be applied as quoted runtime strings, which is unproven for input devices; the supervised model applies but live use stays disabled",
        )
    } else if grammar_is_proven_scalar(row.value_kind) {
        (
            RuntimePreviewDeadManClassification::DeadManPreviewCandidateNeedsLiveProof,
            "the scalar runtime mechanism should apply, but changing input/cursor behavior mid-session can impair the devices used to confirm or revert; live use requires a per-row live proof first",
        )
    } else {
        (
            RuntimePreviewDeadManClassification::DeadManPreviewBlockedNoSafeRuntimeMechanism,
            "no runtime-safe value grammar for this row",
        )
    };

    Some(RuntimePreviewDeadManRowClassification {
        row_id: row.row_id,
        official_setting: row.official_setting,
        value_kind: format!("{:?}", row.value_kind),
        classification,
        reason,
    })
}

pub fn dead_man_row_classifications() -> Vec<RuntimePreviewDeadManRowClassification> {
    SAFE_WRITABLE_ROWS
        .iter()
        .filter_map(|row| classify_dead_man_row(row.row_id))
        .collect()
}

#[derive(Debug, Clone, Serialize)]
pub struct RuntimePreviewDeadManUiState {
    pub row_id: &'static str,
    pub official_setting: &'static str,
    pub classification: RuntimePreviewDeadManClassification,
    pub badge: &'static str,
    pub why_supervised: &'static str,
    pub warning_text: String,
    pub countdown_seconds: u64,
    pub arm_enabled: bool,
    pub shows_panel: bool,
    pub disabled_reason: Option<&'static str>,
    pub recovery_instruction: &'static str,
}

pub fn dead_man_ui_state(row_id: &str) -> Option<RuntimePreviewDeadManUiState> {
    let classification = classify_dead_man_row(row_id)?;
    let arm_enabled = classification.classification.supervised_preview_enabled();
    Some(RuntimePreviewDeadManUiState {
        row_id: classification.row_id,
        official_setting: classification.official_setting,
        classification: classification.classification,
        badge: "Dead-man preview required",
        why_supervised: "This setting can affect how you control your desktop, so preview runs under a recovery countdown.",
        warning_text: format!(
            "The change applies live and reverts automatically after {DEAD_MAN_COUNTDOWN_SECONDS} seconds unless you confirm Keep changes."
        ),
        countdown_seconds: DEAD_MAN_COUNTDOWN_SECONDS,
        arm_enabled,
        shows_panel: classification.classification.shows_supervised_panel(),
        disabled_reason: if arm_enabled {
            None
        } else {
            Some(classification.reason)
        },
        recovery_instruction: DEAD_MAN_RECOVERY_INSTRUCTION,
    })
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
pub enum RuntimePreviewDeadManUiPhase {
    Disarmed,
    Armed,
    CountingDown,
    Kept,
    Reverted,
    Cancelled,
    TimedOutReverted,
}

impl RuntimePreviewDeadManUiPhase {
    pub fn as_str(self) -> &'static str {
        match self {
            Self::Disarmed => "Disarmed",
            Self::Armed => "Armed",
            Self::CountingDown => "Counting down",
            Self::Kept => "Kept",
            Self::Reverted => "Reverted",
            Self::Cancelled => "Cancelled",
            Self::TimedOutReverted => "Timed out and reverted",
        }
    }
}

#[derive(Debug, Clone, Serialize)]
pub struct RuntimePreviewDeadManReceipt {
    pub row_id: String,
    pub action: &'static str,
    pub phase: RuntimePreviewDeadManUiPhase,
    pub value: Option<String>,
    pub original_value: Option<String>,
    pub remaining_seconds: Option<u64>,
    pub config_written: bool,
    pub reload_run: bool,
    pub status_text: String,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum RuntimePreviewDeadManUiError {
    RowNotSupervisable(&'static str),
    NotArmed,
    Executor(String),
}

impl RuntimePreviewDeadManUiError {
    pub fn user_text(&self) -> String {
        match self {
            Self::RowNotSupervisable(reason) => {
                format!("Supervised preview unavailable: {reason}")
            }
            Self::NotArmed => "Start a supervised preview first".to_string(),
            Self::Executor(detail) => format!("Supervised preview failed: {detail}"),
        }
    }
}

/// The supervised session controller. One per detail pane; the GTK layer
/// calls actions and a periodic tick, and never touches the executor.
pub struct RuntimePreviewDeadManController {
    runner: Box<dyn RuntimePreviewRunner>,
    ui_state: RuntimePreviewDeadManUiState,
    session: Option<RuntimePreviewSession>,
    phase: RuntimePreviewDeadManUiPhase,
    elapsed_ms: u64,
}

impl RuntimePreviewDeadManController {
    pub fn new(
        row_id: &str,
        runner: Box<dyn RuntimePreviewRunner>,
    ) -> Result<Self, RuntimePreviewDeadManUiError> {
        let ui_state = dead_man_ui_state(row_id).ok_or(
            RuntimePreviewDeadManUiError::RowNotSupervisable("row is not dead-man gated"),
        )?;
        Ok(Self {
            runner,
            ui_state,
            session: None,
            phase: RuntimePreviewDeadManUiPhase::Disarmed,
            elapsed_ms: 0,
        })
    }

    pub fn new_live(row_id: &str) -> Result<Self, RuntimePreviewDeadManUiError> {
        Self::new(row_id, Box::new(HyprctlRuntimePreviewRunner))
    }

    pub fn ui_state(&self) -> &RuntimePreviewDeadManUiState {
        &self.ui_state
    }

    pub fn phase(&self) -> RuntimePreviewDeadManUiPhase {
        self.phase
    }

    pub fn remaining_seconds(&self) -> u64 {
        (self.ui_state.countdown_seconds * 1000).saturating_sub(self.elapsed_ms) / 1000
    }

    pub fn original_value(&self) -> Option<String> {
        self.session
            .as_ref()
            .map(|session| session.original_value.clone())
    }

    fn map_error(error: RuntimePreviewError) -> RuntimePreviewDeadManUiError {
        match error {
            RuntimePreviewError::InvalidValue { reason, .. } => {
                RuntimePreviewDeadManUiError::Executor(format!("value rejected: {reason}"))
            }
            other => RuntimePreviewDeadManUiError::Executor(format!("{other:?}")),
        }
    }

    /// Arm supervision: capture the original runtime value (read-only) and
    /// enter the armed phase. Rejects rows without an enabled candidate
    /// classification — model-only and needs-live-proof rows cannot arm.
    pub fn arm(&mut self) -> Result<RuntimePreviewDeadManReceipt, RuntimePreviewDeadManUiError> {
        if !self.ui_state.arm_enabled {
            return Err(RuntimePreviewDeadManUiError::RowNotSupervisable(
                self.ui_state
                    .disabled_reason
                    .unwrap_or("supervised preview is not enabled for this row"),
            ));
        }
        let session =
            start_runtime_preview_session(self.runner.as_mut(), self.ui_state.row_id, true)
                .map_err(Self::map_error)?;
        let original_value = session.original_value.clone();
        self.session = Some(session);
        self.phase = RuntimePreviewDeadManUiPhase::Armed;
        self.elapsed_ms = 0;
        Ok(self.receipt(
            "arm",
            None,
            Some(original_value.clone()),
            format!(
                "Supervised preview armed for {}: original value {original_value} captured. Apply a value to start the {DEAD_MAN_COUNTDOWN_SECONDS}-second countdown.",
                self.ui_state.official_setting
            ),
        ))
    }

    /// Apply a preview value inside the armed session and start (or restart)
    /// the confirmation countdown.
    pub fn apply(
        &mut self,
        value: &str,
    ) -> Result<RuntimePreviewDeadManReceipt, RuntimePreviewDeadManUiError> {
        if !matches!(
            self.phase,
            RuntimePreviewDeadManUiPhase::Armed | RuntimePreviewDeadManUiPhase::CountingDown
        ) {
            return Err(RuntimePreviewDeadManUiError::NotArmed);
        }
        let session = self
            .session
            .as_mut()
            .ok_or(RuntimePreviewDeadManUiError::NotArmed)?;
        let receipt = apply_runtime_preview_value_supervised(self.runner.as_mut(), session, value)
            .map_err(Self::map_error)?;
        self.phase = RuntimePreviewDeadManUiPhase::CountingDown;
        self.elapsed_ms = 0;
        let status = format!(
            "Previewing with recovery: {} = {} (original {}). Auto-revert in {} seconds unless you Keep changes.",
            self.ui_state.official_setting,
            receipt.applied_value,
            receipt.original_value,
            self.remaining_seconds()
        );
        Ok(self.receipt(
            "apply",
            Some(receipt.applied_value),
            Some(receipt.original_value),
            status,
        ))
    }

    /// Advance the countdown; auto-reverts on timeout. Returns a receipt only
    /// when the phase changed (timeout fired).
    pub fn tick(
        &mut self,
        delta_ms: u64,
    ) -> Result<Option<RuntimePreviewDeadManReceipt>, RuntimePreviewDeadManUiError> {
        if self.phase != RuntimePreviewDeadManUiPhase::CountingDown {
            return Ok(None);
        }
        self.elapsed_ms = self.elapsed_ms.saturating_add(delta_ms);
        let verdict = {
            let session = self
                .session
                .as_mut()
                .ok_or(RuntimePreviewDeadManUiError::NotArmed)?;
            if let Some(dead_man) = session.dead_man.as_mut() {
                dead_man.tick(delta_ms);
                dead_man.evaluate()
            } else {
                RuntimePreviewDeadManVerdict::KeepActive
            }
        };
        if verdict == RuntimePreviewDeadManVerdict::RevertRequired {
            let receipt = self.revert_internal("timeout")?;
            self.phase = RuntimePreviewDeadManUiPhase::TimedOutReverted;
            return Ok(Some(RuntimePreviewDeadManReceipt {
                phase: RuntimePreviewDeadManUiPhase::TimedOutReverted,
                status_text: format!(
                    "Countdown expired: {} automatically restored to {}.",
                    self.ui_state.official_setting,
                    receipt.value.as_deref().unwrap_or("its original value")
                ),
                ..receipt
            }));
        }
        Ok(None)
    }

    /// Confirm "Keep changes": the preview value stays applied to the runtime
    /// and the countdown stops. Persistence remains a separate explicit step.
    pub fn confirm_keep(
        &mut self,
    ) -> Result<RuntimePreviewDeadManReceipt, RuntimePreviewDeadManUiError> {
        if self.phase != RuntimePreviewDeadManUiPhase::CountingDown {
            return Err(RuntimePreviewDeadManUiError::NotArmed);
        }
        let session = self
            .session
            .as_mut()
            .ok_or(RuntimePreviewDeadManUiError::NotArmed)?;
        if let Some(dead_man) = session.dead_man.as_mut() {
            dead_man.confirm();
        }
        self.phase = RuntimePreviewDeadManUiPhase::Kept;
        let value = session.last_applied_value.clone();
        let status = format!(
            "Kept: {} stays at {} for this session. Nothing was saved to your config.",
            self.ui_state.official_setting,
            value.as_deref().unwrap_or("the previewed value")
        );
        Ok(self.receipt("confirm-keep", value, self.original_value(), status))
    }

    fn revert_internal(
        &mut self,
        action: &'static str,
    ) -> Result<RuntimePreviewDeadManReceipt, RuntimePreviewDeadManUiError> {
        let session = self
            .session
            .as_mut()
            .ok_or(RuntimePreviewDeadManUiError::NotArmed)?;
        let receipt = revert_runtime_preview_session_supervised(self.runner.as_mut(), session)
            .map_err(Self::map_error)?;
        Ok(RuntimePreviewDeadManReceipt {
            row_id: self.ui_state.row_id.to_string(),
            action,
            phase: RuntimePreviewDeadManUiPhase::Reverted,
            value: Some(receipt.restored_value.clone()),
            original_value: Some(receipt.restored_value),
            remaining_seconds: None,
            config_written: false,
            reload_run: false,
            status_text: String::new(),
        })
    }

    /// Manual "Revert now".
    pub fn revert_now(
        &mut self,
    ) -> Result<RuntimePreviewDeadManReceipt, RuntimePreviewDeadManUiError> {
        if !matches!(
            self.phase,
            RuntimePreviewDeadManUiPhase::Armed
                | RuntimePreviewDeadManUiPhase::CountingDown
                | RuntimePreviewDeadManUiPhase::Kept
        ) {
            return Err(RuntimePreviewDeadManUiError::NotArmed);
        }
        let mut receipt = self.revert_internal("revert-now")?;
        self.phase = RuntimePreviewDeadManUiPhase::Reverted;
        receipt.status_text = format!(
            "Reverted: {} restored to {}.",
            self.ui_state.official_setting,
            receipt.value.as_deref().unwrap_or("its original value")
        );
        Ok(receipt)
    }

    /// Cancel: revert and disarm.
    pub fn cancel(&mut self) -> Result<RuntimePreviewDeadManReceipt, RuntimePreviewDeadManUiError> {
        let mut receipt = self.revert_internal("cancel")?;
        self.session = None;
        self.phase = RuntimePreviewDeadManUiPhase::Cancelled;
        receipt.phase = RuntimePreviewDeadManUiPhase::Cancelled;
        receipt.status_text = format!(
            "Supervised preview cancelled: {} restored to {}.",
            self.ui_state.official_setting,
            receipt.value.as_deref().unwrap_or("its original value")
        );
        Ok(receipt)
    }

    /// Session-drop / app-close recovery: revert any armed or unconfirmed
    /// counting-down preview. Kept previews are left in place (the user
    /// explicitly confirmed them for this session).
    pub fn revert_if_unconfirmed(&mut self) -> Option<RuntimePreviewDeadManReceipt> {
        if matches!(
            self.phase,
            RuntimePreviewDeadManUiPhase::Armed | RuntimePreviewDeadManUiPhase::CountingDown
        ) && self
            .session
            .as_ref()
            .map(|session| {
                session.state == RuntimePreviewSessionState::Active
                    && session.last_applied_value.is_some()
            })
            .unwrap_or(false)
        {
            let receipt = self.revert_internal("session-drop").ok()?;
            self.phase = RuntimePreviewDeadManUiPhase::Reverted;
            return Some(receipt);
        }
        None
    }

    fn receipt(
        &self,
        action: &'static str,
        value: Option<String>,
        original_value: Option<String>,
        status_text: String,
    ) -> RuntimePreviewDeadManReceipt {
        RuntimePreviewDeadManReceipt {
            row_id: self.ui_state.row_id.to_string(),
            action,
            phase: self.phase,
            value,
            original_value,
            remaining_seconds: if self.phase == RuntimePreviewDeadManUiPhase::CountingDown {
                Some(self.remaining_seconds())
            } else {
                None
            },
            config_written: false,
            reload_run: false,
            status_text,
        }
    }
}

#[derive(Debug, Clone, Serialize)]
pub struct RuntimePreviewDeadManClassificationSummary {
    pub dead_man_rows_total: usize,
    pub candidates: usize,
    pub candidates_needing_live_proof: usize,
    pub model_only: usize,
    pub blocked_no_safe_runtime_mechanism: usize,
    pub blocked_requires_relog: usize,
    pub blocked_requires_restart: usize,
    pub blocked_no_visible_effect: usize,
    pub blocked_too_dangerous: usize,
}

pub fn dead_man_classification_summary() -> RuntimePreviewDeadManClassificationSummary {
    let rows = dead_man_row_classifications();
    let count = |classification: RuntimePreviewDeadManClassification| {
        rows.iter()
            .filter(|row| row.classification == classification)
            .count()
    };
    RuntimePreviewDeadManClassificationSummary {
        dead_man_rows_total: rows.len(),
        candidates: count(RuntimePreviewDeadManClassification::DeadManPreviewCandidate),
        candidates_needing_live_proof: count(
            RuntimePreviewDeadManClassification::DeadManPreviewCandidateNeedsLiveProof,
        ),
        model_only: count(RuntimePreviewDeadManClassification::DeadManPreviewModelOnly),
        blocked_no_safe_runtime_mechanism: count(
            RuntimePreviewDeadManClassification::DeadManPreviewBlockedNoSafeRuntimeMechanism,
        ),
        blocked_requires_relog: count(
            RuntimePreviewDeadManClassification::DeadManPreviewBlockedRequiresRelog,
        ),
        blocked_requires_restart: count(
            RuntimePreviewDeadManClassification::DeadManPreviewBlockedRequiresRestart,
        ),
        blocked_no_visible_effect: count(
            RuntimePreviewDeadManClassification::DeadManPreviewBlockedNoVisibleEffect,
        ),
        blocked_too_dangerous: count(
            RuntimePreviewDeadManClassification::DeadManPreviewBlockedTooDangerous,
        ),
    }
}
