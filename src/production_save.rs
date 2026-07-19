//! Production scalar Save: the only path UI code may use to persist a
//! scalar change to the active config.
//!
//! Every active-config Save is gated on Safe Live Save Mode: the save
//! proceeds only after a live, read-only verification that autoreload is
//! disabled at runtime, so the write cannot trigger a compositor reload.
//! The gate fails closed and has no bypass parameter. Structured-family
//! saves have their own gated path in `structured_family_gated_persistence`.

use std::collections::BTreeSet;

use crate::config_discovery::ConfigDiscovery;
use crate::current_config::CurrentConfigSnapshot;
use crate::safe_batch_write::{SafeBatchChangeRequest, SafeBatchWriteReport};
use crate::safe_live_save_mode::require_safe_live_save_mode;
use crate::write_flow::{apply_setting_change, ApplyOutcome};

/// Gated scalar save: verifies Safe Live Save Mode live, then delegates to
/// the existing backup/write/reread apply flow exactly once. The wrapper
/// owns the runner, so UI code never constructs one.
pub fn gated_scalar_save_live(
    known_setting_ids: BTreeSet<String>,
    discovery: &ConfigDiscovery,
    current_config: &CurrentConfigSnapshot,
    setting_id: &str,
    proposed_value: &str,
) -> Result<ApplyOutcome, String> {
    let mut runner = crate::runtime_preview_executor::HyprctlRuntimePreviewRunner;
    require_safe_live_save_mode(&mut runner)?;
    apply_setting_change(
        known_setting_ids,
        discovery,
        current_config,
        setting_id,
        proposed_value,
    )
    .map_err(|failure| format!("Save failed: {}", failure.reason))
}

/// Gated all-or-nothing save for pending scalar changes. The write layer
/// accepts only one target file, stages every row before commit, and returns
/// one durable receipt. Any preflight or commit failure leaves all callers'
/// pending state untouched.
pub fn gated_safe_batch_save_live(
    known_setting_ids: BTreeSet<String>,
    discovery: &ConfigDiscovery,
    current_config: &CurrentConfigSnapshot,
    pending_changes: Vec<SafeBatchChangeRequest>,
) -> Result<SafeBatchWriteReport, String> {
    let mut runner = crate::runtime_preview_executor::HyprctlRuntimePreviewRunner;
    require_safe_live_save_mode(&mut runner)?;
    crate::write_flow::apply_safe_batch_setting_changes(
        known_setting_ids,
        discovery,
        current_config,
        pending_changes,
    )
    .map_err(|failure| {
        let details = failure.failures.join("; ");
        if details.is_empty() {
            format!("Batch save failed: {}", failure.reason)
        } else {
            format!("Batch save failed: {} ({details})", failure.reason)
        }
    })
}
