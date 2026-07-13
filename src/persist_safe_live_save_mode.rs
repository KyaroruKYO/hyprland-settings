//! Persist Safe Live Save Mode: save `misc.disable_autoreload = true` to the
//! config through the already-gated production scalar Save, so the mode
//! survives restarts and is naturally active from config.
//!
//! Nothing here is automatic: the user chooses "Save as default". The
//! persistence path is deliberately narrow:
//!
//! - The setting id and value are module constants — this module can write
//!   only `misc.disable_autoreload = true`, nothing else.
//! - Runtime Safe Live Save Mode must already be active (live-verified)
//!   before the write is attempted, so the write itself cannot trigger a
//!   compositor reload; unreadable state fails closed. The gated scalar
//!   Save re-verifies the same gate internally.
//! - The write goes through `production_save::gated_scalar_save_live`
//!   exactly once: backup first, one write, reread verification, no reload.

use serde::Serialize;

use crate::config_discovery::ConfigDiscovery;
use crate::current_config::{CurrentConfigSnapshot, CurrentValueSourceStatus};
use crate::runtime_preview_executor::RuntimePreviewRunner;
use crate::safe_live_save_mode::require_safe_live_save_mode;
use std::collections::BTreeSet;

/// The only setting this module can persist, with the only value.
pub const SAFE_LIVE_SAVE_MODE_SETTING_ID: &str = "misc.disable_autoreload";
pub const SAFE_LIVE_SAVE_MODE_PERSIST_VALUE: &str = "true";

/// Whether the mode is already persisted in the user's config.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
pub enum PersistedSafeLiveSaveModeState {
    /// `misc.disable_autoreload = true` is configured: the mode survives
    /// restarts.
    PersistedTrue,
    /// The setting is configured with a non-true value.
    PersistedOther,
    /// The setting is not in the config: Hyprland's default (autoreload
    /// active) returns after a restart.
    NotPersisted,
    /// The config could not be read or the value conflicts; unknown.
    Unknown,
}

impl PersistedSafeLiveSaveModeState {
    pub fn as_str(self) -> &'static str {
        match self {
            Self::PersistedTrue => "PersistedTrue",
            Self::PersistedOther => "PersistedOther",
            Self::NotPersisted => "NotPersisted",
            Self::Unknown => "Unknown",
        }
    }

    pub fn user_text(self) -> &'static str {
        match self {
            Self::PersistedTrue => "yes - Safe Live Save Mode is active from config after restarts",
            Self::PersistedOther => {
                "no - the config sets misc:disable_autoreload to a non-true value"
            }
            Self::NotPersisted => {
                "no - after a restart, Hyprland's default autoreload returns until you re-enable the mode"
            }
            Self::Unknown => "unknown - the configured value could not be read",
        }
    }
}

/// Read the persisted state from the current-config projection (the same
/// source-aware snapshot the rest of the app reads; no file access here).
pub fn read_persisted_safe_live_save_mode(
    current_config: &CurrentConfigSnapshot,
) -> PersistedSafeLiveSaveModeState {
    let projection = current_config.value_for(SAFE_LIVE_SAVE_MODE_SETTING_ID);
    match projection.status {
        CurrentValueSourceStatus::NotConfigured => PersistedSafeLiveSaveModeState::NotPersisted,
        CurrentValueSourceStatus::Configured => match projection.raw_value.as_deref() {
            Some("true") | Some("1") | Some("yes") | Some("on") => {
                PersistedSafeLiveSaveModeState::PersistedTrue
            }
            Some(_) => PersistedSafeLiveSaveModeState::PersistedOther,
            None => PersistedSafeLiveSaveModeState::Unknown,
        },
        CurrentValueSourceStatus::DuplicateConflict | CurrentValueSourceStatus::ReadUnavailable => {
            PersistedSafeLiveSaveModeState::Unknown
        }
    }
}

#[derive(Debug, Clone, Serialize)]
pub struct PersistSafeLiveSaveModeReceipt {
    pub setting_id: &'static str,
    pub persisted_value: &'static str,
    pub target_path: std::path::PathBuf,
    pub backup_path: std::path::PathBuf,
    pub verified_value: Option<String>,
    pub reload_run: bool,
    pub status_text: String,
}

/// The explicit pre-gate: the runtime mode must already be active before
/// this module even reaches the gated Save (which re-verifies the same gate
/// itself). Fails closed on inactive or unreadable state.
pub fn persist_safe_live_save_mode_gate(
    runner: &mut dyn RuntimePreviewRunner,
) -> Result<(), String> {
    require_safe_live_save_mode(runner)
        .map(|_| ())
        .map_err(|reason| {
            format!(
                "Persisting Safe Live Save Mode is blocked: {reason} The mode must be active at runtime before it can be saved as the default, so the save itself cannot reload Hyprland."
            )
        })
}

/// Persist `misc.disable_autoreload = true` once through the gated scalar
/// Save (backup first, one write, reread verification, no reload). The
/// runtime gate is verified live before the write; the gated Save verifies
/// it again internally.
pub fn persist_safe_live_save_mode_live(
    known_setting_ids: BTreeSet<String>,
    discovery: &ConfigDiscovery,
    current_config: &CurrentConfigSnapshot,
) -> Result<PersistSafeLiveSaveModeReceipt, String> {
    let mut runner = crate::runtime_preview_executor::HyprctlRuntimePreviewRunner;
    persist_safe_live_save_mode_gate(&mut runner)?;
    let outcome = crate::production_save::gated_scalar_save_live(
        known_setting_ids,
        discovery,
        current_config,
        SAFE_LIVE_SAVE_MODE_SETTING_ID,
        SAFE_LIVE_SAVE_MODE_PERSIST_VALUE,
    )?;
    Ok(PersistSafeLiveSaveModeReceipt {
        setting_id: SAFE_LIVE_SAVE_MODE_SETTING_ID,
        persisted_value: SAFE_LIVE_SAVE_MODE_PERSIST_VALUE,
        target_path: outcome.target_path.clone(),
        backup_path: outcome.backup_path.clone(),
        verified_value: outcome.verified_value.clone(),
        reload_run: false,
        status_text: format!(
            "Saved as default: misc:disable_autoreload = true persisted to {} (backup: {}; reread-verified; no reload). Safe Live Save Mode is now active from config after restarts.",
            outcome.target_path.display(),
            outcome.backup_path.display()
        ),
    })
}
