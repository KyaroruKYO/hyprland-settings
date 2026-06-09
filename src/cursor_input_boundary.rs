use std::path::{Component, PathBuf};

use anyhow::{anyhow, Result};
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum CursorInputBoundaryStatus {
    ReadyForFixtureProofOnly,
    Blocked,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct CursorInputRecoveryBoundaryPlan {
    pub row_id: String,
    pub target_config_path: PathBuf,
    pub backup_path: Option<PathBuf>,
    pub result_log_path: Option<PathBuf>,
    pub confirmation_token: Option<String>,
    pub timeout_seconds: u64,
    pub live_execution_enabled: bool,
    pub independent_watchdog_available: bool,
    pub restore_command_presented_before_apply: bool,
    pub out_of_band_recovery_instructions: bool,
    pub confirmation_depends_on_app_ui: bool,
    pub confirmation_depends_on_visible_cursor: bool,
    pub confirmation_depends_on_hyprland_keybind: bool,
    pub confirmation_depends_on_mouse_input: bool,
    pub confirmation_depends_on_pointer_focus: bool,
    pub confirmation_depends_on_workspace_focus: bool,
    pub confirmation_depends_on_normal_pointer_behavior: bool,
    pub reload_or_runtime_mutation_required: bool,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct CursorInputRecoveryBoundaryResult {
    pub row_id: String,
    pub status: CursorInputBoundaryStatus,
    pub live_execution_enabled: bool,
    pub can_represent_boundary: bool,
    pub can_execute_live_config: bool,
    pub blocked_reasons: Vec<String>,
}

pub fn evaluate_cursor_input_recovery_boundary(
    plan: &CursorInputRecoveryBoundaryPlan,
) -> CursorInputRecoveryBoundaryResult {
    let mut blocked_reasons = Vec::new();

    if plan.live_execution_enabled {
        blocked_reasons.push("live-execution-must-remain-disabled".to_owned());
    }
    if looks_like_live_hyprland_config(&plan.target_config_path) {
        blocked_reasons.push("real-config-path-refused-while-live-disabled".to_owned());
    }
    if plan.backup_path.is_none() {
        blocked_reasons.push("missing-backup-path".to_owned());
    }
    if plan.result_log_path.is_none() {
        blocked_reasons.push("missing-result-log-path".to_owned());
    }
    if plan.confirmation_token.as_deref().is_none_or(str::is_empty) {
        blocked_reasons.push("missing-confirmation-token".to_owned());
    }
    if plan.timeout_seconds == 0 {
        blocked_reasons.push("missing-timeout".to_owned());
    }
    if !plan.independent_watchdog_available {
        blocked_reasons.push("missing-independent-watchdog".to_owned());
    }
    if !plan.restore_command_presented_before_apply {
        blocked_reasons.push("missing-restore-command-before-apply".to_owned());
    }
    if !plan.out_of_band_recovery_instructions {
        blocked_reasons.push("missing-out-of-band-recovery-instructions".to_owned());
    }
    if plan.confirmation_depends_on_app_ui {
        blocked_reasons.push("app-ui-only-confirmation-rejected".to_owned());
    }
    if plan.confirmation_depends_on_visible_cursor {
        blocked_reasons.push("visible-cursor-confirmation-rejected".to_owned());
    }
    if plan.confirmation_depends_on_hyprland_keybind {
        blocked_reasons.push("hyprland-keybind-confirmation-rejected".to_owned());
    }
    if plan.confirmation_depends_on_mouse_input {
        blocked_reasons.push("mouse-only-confirmation-rejected".to_owned());
    }
    if plan.confirmation_depends_on_pointer_focus {
        blocked_reasons.push("pointer-focus-confirmation-rejected".to_owned());
    }
    if plan.confirmation_depends_on_workspace_focus {
        blocked_reasons.push("workspace-focus-confirmation-rejected".to_owned());
    }
    if plan.confirmation_depends_on_normal_pointer_behavior {
        blocked_reasons.push("normal-pointer-behavior-confirmation-rejected".to_owned());
    }
    if plan.reload_or_runtime_mutation_required {
        blocked_reasons.push("reload-or-runtime-mutation-still-disabled".to_owned());
    }

    let status = if blocked_reasons.is_empty() {
        CursorInputBoundaryStatus::ReadyForFixtureProofOnly
    } else {
        CursorInputBoundaryStatus::Blocked
    };

    CursorInputRecoveryBoundaryResult {
        row_id: plan.row_id.clone(),
        status,
        live_execution_enabled: plan.live_execution_enabled,
        can_represent_boundary: true,
        can_execute_live_config: false,
        blocked_reasons,
    }
}

pub fn assert_cursor_input_live_execution_refused(
    result: &CursorInputRecoveryBoundaryResult,
) -> Result<()> {
    if result.can_execute_live_config {
        return Err(anyhow!(
            "cursor/input recovery boundary unexpectedly allows live execution"
        ));
    }
    Ok(())
}

fn looks_like_live_hyprland_config(path: &std::path::Path) -> bool {
    let components = path
        .components()
        .filter_map(|component| match component {
            Component::Normal(value) => value.to_str(),
            _ => None,
        })
        .collect::<Vec<_>>();
    components
        .windows(3)
        .any(|window| window == [".config", "hypr", "hyprland.conf"])
}
