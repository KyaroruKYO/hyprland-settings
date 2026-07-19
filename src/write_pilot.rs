use anyhow::{anyhow, Result};

use crate::pending_change::ACTIVE_PENDING_CHANGE_SETTING;
use crate::scalar_write::apply_scalar_write_plan;
use crate::write_safety::{WritePlan, WriteResult};

/// Legacy fixture-facing pilot entry point. It delegates to the same hardened,
/// drift-checked writer used by the production scalar flow.
pub fn apply_windows_snap_enabled_plan(plan: &WritePlan) -> Result<WriteResult> {
    if plan.setting_id != ACTIVE_PENDING_CHANGE_SETTING {
        return Err(anyhow!(
            "write pilot only supports {}",
            ACTIVE_PENDING_CHANGE_SETTING
        ));
    }
    apply_scalar_write_plan(plan).map_err(|error| anyhow!(error))
}
