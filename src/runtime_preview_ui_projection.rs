//! UI projection and controller for per-setting live runtime preview.
//!
//! The GTK layer never talks to the runtime preview executor directly: it
//! renders `RuntimePreviewUiRowState` projections and calls controller
//! actions, which route through the executor's session/apply/revert
//! functions. Command construction, capability gating, throttling, and value
//! grammar validation all live below this module — UI code cannot build
//! `hyprctl` invocations, cannot mutate unsupported rows, and cannot persist
//! anything (Save defers to the app's existing safe scalar write flow).

use serde::Serialize;

use crate::runtime_preview::{
    classify_runtime_preview_row, RuntimePreviewCapability, RuntimePreviewRiskClass,
};
use crate::runtime_preview_executor::{
    apply_runtime_preview_value, mark_runtime_preview_session_saved,
    revert_runtime_preview_session, start_runtime_preview_session, HyprctlRuntimePreviewRunner,
    RuntimePreviewError, RuntimePreviewRunner, RuntimePreviewSession, RuntimePreviewSessionState,
    RuntimePreviewThrottle,
};
use crate::write_classification::{
    finite_choice_options, source_backed_numeric_bounds, ScalarWriteValueKind, SAFE_WRITABLE_ROWS,
};
use crate::write_flow::write_flow_config_setting;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
pub enum RuntimePreviewUiControlKind {
    Switch,
    Slider,
    SpinRow,
    ColorEntry,
    ValueEntry,
    Dropdown,
    NoControl,
}

impl RuntimePreviewUiControlKind {
    pub fn as_str(self) -> &'static str {
        match self {
            Self::Switch => "Switch",
            Self::Slider => "Slider",
            Self::SpinRow => "SpinRow",
            Self::ColorEntry => "ColorEntry",
            Self::ValueEntry => "ValueEntry",
            Self::Dropdown => "Dropdown",
            Self::NoControl => "NoControl",
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
pub enum RuntimePreviewUiSessionState {
    Idle,
    PreviewingLive,
    Reverted,
    Saved,
    Cancelled,
}

impl RuntimePreviewUiSessionState {
    pub fn as_str(self) -> &'static str {
        match self {
            Self::Idle => "Idle",
            Self::PreviewingLive => "Previewing Live",
            Self::Reverted => "Reverted",
            Self::Saved => "Saved",
            Self::Cancelled => "Cancelled",
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
pub enum RuntimePreviewUiSaveState {
    /// The row is on the app's existing safe scalar write allowlist; Save
    /// persists the final previewed value once through that flow.
    AvailableThroughExistingWriteFlow,
    /// The row cannot persist; preview stays temporary.
    DisabledWithReason(&'static str),
}

impl RuntimePreviewUiSaveState {
    pub fn available(&self) -> bool {
        matches!(self, Self::AvailableThroughExistingWriteFlow)
    }

    pub fn reason(&self) -> &'static str {
        match self {
            Self::AvailableThroughExistingWriteFlow => {
                "Save persists once through the app's backup/write/reread flow"
            }
            Self::DisabledWithReason(reason) => reason,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum RuntimePreviewUiAction {
    PreviewValue(String),
    Save,
    Revert,
    Cancel,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum RuntimePreviewUiError {
    RowNotPreviewable(&'static str),
    Executor(String),
    InvalidValue(String),
    NoActiveSession,
}

impl RuntimePreviewUiError {
    pub fn user_text(&self) -> String {
        match self {
            Self::RowNotPreviewable(reason) => format!("Live preview unavailable: {reason}"),
            Self::Executor(detail) => format!("Preview failed: {detail}"),
            Self::InvalidValue(detail) => format!("Value rejected: {detail}"),
            Self::NoActiveSession => "No active preview session".to_string(),
        }
    }
}

#[derive(Debug, Clone, Serialize)]
pub struct RuntimePreviewUiReceipt {
    pub row_id: String,
    pub action: &'static str,
    pub value: Option<String>,
    pub original_runtime_value: Option<String>,
    pub session_state: RuntimePreviewUiSessionState,
    pub config_written: bool,
    pub reload_run: bool,
    pub status_text: String,
}

/// Everything the detail pane needs to render live preview for one row.
#[derive(Debug, Clone, Serialize)]
pub struct RuntimePreviewUiRowState {
    pub row_id: &'static str,
    pub section: String,
    pub option: String,
    pub display_label: String,
    pub capability: RuntimePreviewCapability,
    pub capability_badge: String,
    pub risk: RuntimePreviewRiskClass,
    pub control_kind: RuntimePreviewUiControlKind,
    pub preview_enabled: bool,
    pub dead_man_required: bool,
    pub throttle_ms: Option<u64>,
    pub slider_bounds: Option<(f64, f64)>,
    pub dropdown_choices: Vec<(String, String)>,
    pub save_state: RuntimePreviewUiSaveState,
    pub revert_available: bool,
    pub cancel_available: bool,
    pub unavailable_reason: Option<String>,
    pub status_text: String,
}

fn control_kind_for_row(
    row_id: &str,
    value_kind: ScalarWriteValueKind,
    preview_enabled: bool,
) -> RuntimePreviewUiControlKind {
    if !preview_enabled {
        return RuntimePreviewUiControlKind::NoControl;
    }
    match value_kind {
        ScalarWriteValueKind::Boolean => RuntimePreviewUiControlKind::Switch,
        ScalarWriteValueKind::FiniteChoice => RuntimePreviewUiControlKind::Dropdown,
        ScalarWriteValueKind::Number => {
            if source_backed_numeric_bounds(row_id).is_some() {
                RuntimePreviewUiControlKind::Slider
            } else {
                RuntimePreviewUiControlKind::SpinRow
            }
        }
        ScalarWriteValueKind::Percent => RuntimePreviewUiControlKind::Slider,
        ScalarWriteValueKind::Color | ScalarWriteValueKind::Gradient => {
            RuntimePreviewUiControlKind::ColorEntry
        }
        ScalarWriteValueKind::CssGap
        | ScalarWriteValueKind::Vector2
        | ScalarWriteValueKind::NumericList
        | ScalarWriteValueKind::CommaSeparatedFloatList
        | ScalarWriteValueKind::SourceBacked => RuntimePreviewUiControlKind::ValueEntry,
        _ => RuntimePreviewUiControlKind::NoControl,
    }
}

pub fn runtime_preview_ui_row_state(row_id: &str) -> Option<RuntimePreviewUiRowState> {
    let row = SAFE_WRITABLE_ROWS.iter().find(|row| row.row_id == row_id)?;
    let capability = classify_runtime_preview_row(row);
    let preview_enabled = capability.capability.live_previewable_by_default();
    let control_kind = control_kind_for_row(row.row_id, row.value_kind, preview_enabled);
    let preview_enabled = preview_enabled && control_kind != RuntimePreviewUiControlKind::NoControl;

    let mut segments = row.official_setting.rsplitn(2, '.');
    let option = segments.next().unwrap_or_default().to_string();
    let section = segments.next().unwrap_or_default().to_string();

    let capability_badge = match capability.capability {
        RuntimePreviewCapability::LivePreviewSupported => "Live Preview: Supported".to_string(),
        RuntimePreviewCapability::LivePreviewSupportedWithThrottle => {
            "Live Preview: Supported with throttle".to_string()
        }
        RuntimePreviewCapability::LivePreviewSupportedWithDeadMan => {
            "Dead-man preview required".to_string()
        }
        RuntimePreviewCapability::RequiresConfigWrite => {
            "Preview unavailable: persists through config write".to_string()
        }
        RuntimePreviewCapability::RequiresReload => "Requires reload".to_string(),
        RuntimePreviewCapability::RequiresRelog => "Requires log out/in".to_string(),
        RuntimePreviewCapability::RequiresRestart => "Requires restart".to_string(),
        RuntimePreviewCapability::BlockedHighRisk => {
            "Preview blocked: high-risk setting".to_string()
        }
        RuntimePreviewCapability::BlockedUnsupportedGrammar
        | RuntimePreviewCapability::BlockedStructuredFamilySemantics => {
            "Preview blocked: unsupported value grammar".to_string()
        }
        RuntimePreviewCapability::LivePreviewReadOnlyOnly
        | RuntimePreviewCapability::NotProvenYet => {
            "Preview unavailable: not proven yet".to_string()
        }
    };

    let save_state = if write_flow_config_setting(row.row_id).is_some() {
        RuntimePreviewUiSaveState::AvailableThroughExistingWriteFlow
    } else {
        RuntimePreviewUiSaveState::DisabledWithReason(
            "this row is not on the safe scalar write allowlist; preview stays temporary",
        )
    };

    let slider_bounds = match row.value_kind {
        ScalarWriteValueKind::Percent => Some((0.0, 1.0)),
        ScalarWriteValueKind::Number => {
            source_backed_numeric_bounds(row.row_id).map(|bounds| (bounds.min, bounds.max))
        }
        _ => None,
    };
    let dropdown_choices = finite_choice_options(row.row_id)
        .map(|choices| {
            choices
                .iter()
                .map(|choice| (choice.raw_value.to_string(), choice.label.to_string()))
                .collect()
        })
        .unwrap_or_default();

    Some(RuntimePreviewUiRowState {
        row_id: row.row_id,
        section,
        option,
        display_label: row.official_setting.to_string(),
        capability: capability.capability,
        capability_badge,
        risk: capability.risk,
        control_kind,
        preview_enabled,
        dead_man_required: capability.dead_man_required,
        throttle_ms: capability.throttle_ms,
        slider_bounds,
        dropdown_choices,
        save_state,
        revert_available: preview_enabled,
        cancel_available: preview_enabled,
        unavailable_reason: if preview_enabled {
            None
        } else {
            Some(capability.reason.to_string())
        },
        status_text: if preview_enabled {
            "Adjust the control to preview this setting live. Nothing is saved until you choose Save.".to_string()
        } else {
            capability.reason.to_string()
        },
    })
}

pub fn runtime_preview_ui_projections() -> Vec<RuntimePreviewUiRowState> {
    SAFE_WRITABLE_ROWS
        .iter()
        .filter_map(|row| runtime_preview_ui_row_state(row.row_id))
        .collect()
}

/// Session controller: owns the runner, the active session, and the throttle.
/// The GTK layer holds one controller per detail pane and calls actions only.
pub struct RuntimePreviewUiController {
    runner: Box<dyn RuntimePreviewRunner>,
    session: Option<RuntimePreviewSession>,
    throttle: Option<RuntimePreviewThrottle>,
    row_state: RuntimePreviewUiRowState,
}

impl RuntimePreviewUiController {
    pub fn new(
        row_id: &str,
        runner: Box<dyn RuntimePreviewRunner>,
    ) -> Result<Self, RuntimePreviewUiError> {
        let row_state = runtime_preview_ui_row_state(row_id)
            .ok_or(RuntimePreviewUiError::RowNotPreviewable("unknown row"))?;
        Ok(Self {
            runner,
            session: None,
            throttle: row_state.throttle_ms.map(RuntimePreviewThrottle::new),
            row_state,
        })
    }

    /// Live controller for the running compositor.
    pub fn new_live(row_id: &str) -> Result<Self, RuntimePreviewUiError> {
        Self::new(row_id, Box::new(HyprctlRuntimePreviewRunner))
    }

    pub fn row_state(&self) -> &RuntimePreviewUiRowState {
        &self.row_state
    }

    pub fn session_state(&self) -> RuntimePreviewUiSessionState {
        match &self.session {
            None => RuntimePreviewUiSessionState::Idle,
            Some(session) => match session.state {
                RuntimePreviewSessionState::Active => RuntimePreviewUiSessionState::PreviewingLive,
                RuntimePreviewSessionState::Saved => RuntimePreviewUiSessionState::Saved,
                RuntimePreviewSessionState::Reverted => RuntimePreviewUiSessionState::Reverted,
                RuntimePreviewSessionState::Cancelled => RuntimePreviewUiSessionState::Cancelled,
            },
        }
    }

    pub fn original_runtime_value(&self) -> Option<String> {
        self.session
            .as_ref()
            .map(|session| session.original_value.clone())
    }

    pub fn last_applied_value(&self) -> Option<String> {
        self.session
            .as_ref()
            .and_then(|session| session.last_applied_value.clone())
    }

    fn map_error(error: RuntimePreviewError) -> RuntimePreviewUiError {
        match error {
            RuntimePreviewError::InvalidValue { reason, .. } => {
                RuntimePreviewUiError::InvalidValue(reason.to_string())
            }
            RuntimePreviewError::RowNotLivePreviewable { capability, .. } => {
                RuntimePreviewUiError::RowNotPreviewable(capability)
            }
            RuntimePreviewError::DeadManConfirmationRequired(_) => {
                RuntimePreviewUiError::RowNotPreviewable("dead-man confirmation required")
            }
            RuntimePreviewError::NoActiveSession => RuntimePreviewUiError::NoActiveSession,
            other => RuntimePreviewUiError::Executor(format!("{other:?}")),
        }
    }

    fn ensure_session(&mut self) -> Result<(), RuntimePreviewUiError> {
        if !self.row_state.preview_enabled {
            return Err(RuntimePreviewUiError::RowNotPreviewable(
                "row is not default-previewable",
            ));
        }
        let stale = matches!(
            self.session.as_ref().map(|session| &session.state),
            Some(RuntimePreviewSessionState::Reverted)
                | Some(RuntimePreviewSessionState::Saved)
                | Some(RuntimePreviewSessionState::Cancelled)
        );
        if self.session.is_none() || stale {
            let session =
                start_runtime_preview_session(self.runner.as_mut(), self.row_state.row_id, false)
                    .map_err(Self::map_error)?;
            self.session = Some(session);
        }
        Ok(())
    }

    /// Offer a value from a continuous control at `now_ms`. Values are
    /// throttled (trailing edge, latest wins); returns a receipt when a value
    /// was actually applied to the running session.
    pub fn offer_value(
        &mut self,
        value: &str,
        now_ms: u64,
    ) -> Result<Option<RuntimePreviewUiReceipt>, RuntimePreviewUiError> {
        self.ensure_session()?;
        let due_value = match self.throttle.as_mut() {
            Some(throttle) => throttle.offer(value, now_ms),
            None => Some(value.to_string()),
        };
        match due_value {
            Some(value) => self.apply_now(&value).map(Some),
            None => Ok(None),
        }
    }

    /// Drain the throttle's pending value if its interval has elapsed.
    pub fn drain_pending(
        &mut self,
        now_ms: u64,
    ) -> Result<Option<RuntimePreviewUiReceipt>, RuntimePreviewUiError> {
        let Some(value) = self
            .throttle
            .as_mut()
            .and_then(|throttle| throttle.take_due(now_ms))
        else {
            return Ok(None);
        };
        self.apply_now(&value).map(Some)
    }

    fn apply_now(&mut self, value: &str) -> Result<RuntimePreviewUiReceipt, RuntimePreviewUiError> {
        let session = self
            .session
            .as_mut()
            .ok_or(RuntimePreviewUiError::NoActiveSession)?;
        let receipt = apply_runtime_preview_value(self.runner.as_mut(), session, value)
            .map_err(Self::map_error)?;
        Ok(RuntimePreviewUiReceipt {
            row_id: receipt.row_id,
            action: "preview",
            value: Some(receipt.applied_value.clone()),
            original_runtime_value: Some(receipt.original_value),
            session_state: RuntimePreviewUiSessionState::PreviewingLive,
            config_written: false,
            reload_run: false,
            status_text: format!(
                "Previewing Live: {} = {} (original {})",
                self.row_state.display_label,
                receipt.applied_value,
                self.session
                    .as_ref()
                    .map(|session| session.original_value.as_str())
                    .unwrap_or("unknown")
            ),
        })
    }

    /// Revert to the original runtime value; the session stays cleared.
    pub fn revert(&mut self) -> Result<RuntimePreviewUiReceipt, RuntimePreviewUiError> {
        let session = self
            .session
            .as_mut()
            .ok_or(RuntimePreviewUiError::NoActiveSession)?;
        let receipt = revert_runtime_preview_session(self.runner.as_mut(), session)
            .map_err(Self::map_error)?;
        Ok(RuntimePreviewUiReceipt {
            row_id: receipt.row_id,
            action: "revert",
            value: Some(receipt.restored_value.clone()),
            original_runtime_value: Some(receipt.restored_value.clone()),
            session_state: RuntimePreviewUiSessionState::Reverted,
            config_written: false,
            reload_run: false,
            status_text: format!(
                "Reverted: {} restored to {}",
                self.row_state.display_label, receipt.restored_value
            ),
        })
    }

    /// Cancel: revert (if anything was applied) and clear the session.
    pub fn cancel(&mut self) -> Result<RuntimePreviewUiReceipt, RuntimePreviewUiError> {
        let mut receipt = self.revert()?;
        self.session = None;
        if let Some(throttle) = self.throttle.as_mut() {
            throttle.pending_value = None;
        }
        receipt.action = "cancel";
        receipt.session_state = RuntimePreviewUiSessionState::Cancelled;
        receipt.status_text = format!(
            "Preview cancelled: {} restored to {}",
            self.row_state.display_label,
            receipt.value.as_deref().unwrap_or("original")
        );
        Ok(receipt)
    }

    /// Mark the session saved. The actual persistence must be performed by
    /// the caller exactly once through the app's existing safe scalar write
    /// flow; this controller cannot write config files.
    pub fn mark_saved(&mut self) -> Result<RuntimePreviewUiReceipt, RuntimePreviewUiError> {
        if !self.row_state.save_state.available() {
            return Err(RuntimePreviewUiError::RowNotPreviewable(
                self.row_state.save_state.reason(),
            ));
        }
        let last_value = self.last_applied_value();
        let session = self
            .session
            .as_mut()
            .ok_or(RuntimePreviewUiError::NoActiveSession)?;
        mark_runtime_preview_session_saved(session).map_err(Self::map_error)?;
        Ok(RuntimePreviewUiReceipt {
            row_id: self.row_state.row_id.to_string(),
            action: "save",
            value: last_value,
            original_runtime_value: self.original_runtime_value(),
            session_state: RuntimePreviewUiSessionState::Saved,
            config_written: false,
            reload_run: false,
            status_text: format!(
                "Saved: persist {} once through the existing backup/write/reread flow",
                self.row_state.display_label
            ),
        })
    }

    /// App-close / detail-pane-drop recovery: if a preview is still active,
    /// restore the original runtime value.
    pub fn revert_if_active(&mut self) -> Option<RuntimePreviewUiReceipt> {
        if matches!(
            self.session.as_ref().map(|session| &session.state),
            Some(RuntimePreviewSessionState::Active)
        ) && self
            .session
            .as_ref()
            .map(|session| session.last_applied_value.is_some())
            .unwrap_or(false)
        {
            return self.revert().ok();
        }
        None
    }
}
