//! Save-only pending ledger: the config-draft side of the unified pending
//! model.
//!
//! Rows the app classifies [`StatusChip::SaveOnly`] have a proven, gated
//! config-write path but no live-preview stage: their value only takes
//! effect after a reload/relog/restart, or the compositor exposes no live
//! setter. Before this module those rows had no unsaved intermediate state
//! — the detail-pane Apply wrote straight through the gate. Now editing such
//! a row STAGES a config draft here, and the shared pending surfaces (row
//! accent, sidebar count, header chip, bottom bar, Pending Changes page and
//! its config diff) present it exactly like a live-preview change. The
//! shared "Save now" persists it through the same gated scalar save; Discard
//! drops the draft.
//!
//! Nothing in this module touches the runtime. Staging, clearing, and
//! querying drafts are pure `Vec` operations with no `hyprctl` runner in
//! sight — that is the defining guarantee of "save-only": no runtime
//! mutation happens before Save. The module deliberately does not import the
//! runtime-preview executor, so a save-only edit cannot preview or apply
//! anything live.

use crate::runtime_preview_dead_man::dead_man_ui_state;
use crate::runtime_preview_ui_projection::runtime_preview_ui_row_state;
use crate::write_classification::{
    is_high_risk_gated_writable_setting, ScalarWriteValueKind, SAFE_WRITABLE_ROWS,
};

/// The GTK control a save-only row edits with. Derived purely from the
/// row's value grammar — the same value-kind → control mapping the
/// live-preview controls use, minus the live-preview gate.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SaveOnlyControlKind {
    /// Booleans.
    Switch,
    /// Finite enumerations (source-backed choice lists).
    Dropdown,
    /// Numbers and percentages.
    Spin,
    /// Line-safe strings, regexes, and source-backed string values
    /// (keyboard layout, fonts, swallow regexes).
    Entry,
}

impl SaveOnlyControlKind {
    pub fn as_str(self) -> &'static str {
        match self {
            Self::Switch => "Switch",
            Self::Dropdown => "Dropdown",
            Self::Spin => "Spin",
            Self::Entry => "Entry",
        }
    }
}

/// The control kind for a save-only row's value grammar, or `None` when the
/// grammar has no safe scalar control (exotic/complex values keep their
/// read-only presentation and never stage a plain-text draft). Fail-closed:
/// only value kinds with a proven, unambiguous scalar control are mapped.
pub fn save_only_control_kind(row_id: &str) -> Option<SaveOnlyControlKind> {
    let row = SAFE_WRITABLE_ROWS.iter().find(|row| row.row_id == row_id)?;
    Some(match row.value_kind {
        ScalarWriteValueKind::Boolean => SaveOnlyControlKind::Switch,
        ScalarWriteValueKind::FiniteChoice => SaveOnlyControlKind::Dropdown,
        ScalarWriteValueKind::Number | ScalarWriteValueKind::Percent => SaveOnlyControlKind::Spin,
        // Text-representable grammars: a plain entry stages the raw token and
        // the gated save validates + reread-verifies it (invalid input is
        // rejected at Save, never silently written). This covers keyboard
        // layout, fonts, regexes, vectors, paths, monitor names, accel
        // profiles, css gaps, and numeric lists.
        ScalarWriteValueKind::LineSafeString
        | ScalarWriteValueKind::RegexString
        | ScalarWriteValueKind::SourceBacked
        | ScalarWriteValueKind::StringLike
        | ScalarWriteValueKind::Path
        | ScalarWriteValueKind::MonitorName
        | ScalarWriteValueKind::Vector2
        | ScalarWriteValueKind::NumericList
        | ScalarWriteValueKind::CommaSeparatedFloatList
        | ScalarWriteValueKind::AccelProfile
        | ScalarWriteValueKind::CssGap => SaveOnlyControlKind::Entry,
        // Colors/gradients live-preview; ComplexRaw/Unknown have no proven
        // safe scalar control — fail closed.
        ScalarWriteValueKind::Color
        | ScalarWriteValueKind::Gradient
        | ScalarWriteValueKind::ComplexRaw
        | ScalarWriteValueKind::Unknown => return None,
    })
}

/// Whether a row is safely editable through the save-only staged path.
///
/// Save-only is a deferred, gated config write: staging changes nothing at
/// runtime, and "Save now" writes the value through the existing Safe Live
/// Save Mode gate (backup + write + reread, no reload), so it only takes
/// effect on the user's next reload. That makes it safe for far more than
/// the rows that live-preview.
///
/// A row is save-only-editable when ALL hold:
/// - it is NOT live-previewable (live rows use the preview ledger), and
/// - this machine cannot arm it under the dead-man countdown (armable rows
///   use the modal), and
/// - it has a concrete scalar control for its value grammar, and
/// - it is backed by the app's gated scalar write flow, and
/// - it is NOT a production-gated high-risk setting.
///
/// The production-gate exclusion is the one hard stop: settings in the
/// accepted high-risk display/render, cursor/input, and debug/crash lists
/// require a recovery-plan proof to save (they can brick display or session
/// on the next reload), and the plain save-only save cannot supply that
/// proof. Exposing a control for them would be editable-but-unsavable, so
/// they stay out until the production-gate recovery flow exists. Fail-closed
/// on everything else.
pub fn is_save_only_editable(row_id: &str) -> bool {
    let Some(state) = runtime_preview_ui_row_state(row_id) else {
        return false;
    };
    if state.preview_enabled {
        return false;
    }
    if dead_man_ui_state(row_id)
        .map(|dm| dm.arm_enabled)
        .unwrap_or(false)
    {
        return false;
    }
    if save_only_control_kind(row_id).is_none() {
        return false;
    }
    if !state.save_state.available() {
        return false;
    }
    if is_high_risk_gated_writable_setting(row_id) {
        return false;
    }
    true
}

/// One staged, not-yet-saved config change for a save-only row. Pure data:
/// constructing one performs no runtime mutation.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SaveOnlyDraft {
    pub row_id: String,
    pub official_setting: String,
    /// The sidebar page the row lives on, for per-page pending counts and
    /// review-page grouping. Stored as an owned string; the UI resolves it
    /// back to the interned page id.
    pub page_id: Option<String>,
    /// The row's saved/effective value before staging — the baseline the
    /// draft is compared against and the value Discard restores the control
    /// to.
    pub original_value: String,
    /// What "Save now" will write through the gated scalar save.
    pub staged_value: String,
    /// Whether the target config already has a line for this setting
    /// (Modified) or the save would append one (Added).
    pub config_has_line: bool,
}

impl SaveOnlyDraft {
    /// A draft is "pending" only while the staged value semantically differs
    /// from the original — staging a value back to the original (e.g.
    /// toggling a switch and toggling it back) is not an unsaved change.
    /// Uses the same semantic comparison as the live-preview ledger so
    /// spelling differences ("true"/"1", "0.5"/"0.500000") do not register.
    pub fn is_pending(&self) -> bool {
        !crate::pending_changes_ui::values_semantically_equal(
            &self.staged_value,
            &self.original_value,
        )
    }
}

/// The set of staged save-only drafts. A plain value (not tied to GTK) so
/// the staging/clearing/pending logic is unit-testable in isolation and is
/// provably free of runtime mutation — there is no runner to call. The UI
/// holds exactly one of these in a thread-local.
#[derive(Debug, Default)]
pub struct SaveOnlyLedger {
    drafts: Vec<SaveOnlyDraft>,
}

impl SaveOnlyLedger {
    pub fn new() -> Self {
        Self::default()
    }

    /// Stage (or restage) a row's draft, replacing any existing draft for
    /// the same row so a row never has two competing staged values.
    pub fn stage(&mut self, draft: SaveOnlyDraft) {
        self.drafts
            .retain(|existing| existing.row_id != draft.row_id);
        self.drafts.push(draft);
    }

    /// Remove a row's staged draft. Returns whether a draft was removed.
    pub fn clear(&mut self, row_id: &str) -> bool {
        let before = self.drafts.len();
        self.drafts.retain(|draft| draft.row_id != row_id);
        before != self.drafts.len()
    }

    /// Drop every staged draft (Discard-all / window close).
    pub fn clear_all(&mut self) {
        self.drafts.clear();
    }

    pub fn get(&self, row_id: &str) -> Option<&SaveOnlyDraft> {
        self.drafts.iter().find(|draft| draft.row_id == row_id)
    }

    /// The drafts whose staged value still semantically differs from the
    /// original — the ones the pending surfaces should count and the gated
    /// save should write.
    pub fn pending(&self) -> Vec<&SaveOnlyDraft> {
        self.drafts
            .iter()
            .filter(|draft| draft.is_pending())
            .collect()
    }

    /// Total number of staged drafts (including any that have been restaged
    /// back to their original and are not pending).
    pub fn len(&self) -> usize {
        self.drafts.len()
    }

    pub fn is_empty(&self) -> bool {
        self.drafts.is_empty()
    }
}
