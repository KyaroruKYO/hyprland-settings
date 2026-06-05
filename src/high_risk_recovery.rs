use std::fs::{self, OpenOptions};
use std::io::Write;
use std::path::{Component, Path, PathBuf};
use std::time::{Duration, SystemTime, UNIX_EPOCH};

use anyhow::{anyhow, Context, Result};
use serde::{Deserialize, Serialize};

use crate::config_backup::{BackupManager, ConfigBackup};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum RecoveryStatus {
    Armed,
    Confirmed,
    Reverted,
    Expired,
    Failed,
    Cancelled,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct HighRiskRecoveryPlan {
    pub recovery_session_id: String,
    pub target_config_path: PathBuf,
    pub backup_path: PathBuf,
    pub proposed_mutation: String,
    pub previous_known_good_value: String,
    pub timeout_seconds: u64,
    pub confirmation_token: String,
    pub confirmation_deadline_unix_seconds: u64,
    pub recovery_action: String,
    pub cleanup_action: String,
    pub status: RecoveryStatus,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct HighRiskRecoveryResult {
    pub recovery_session_id: String,
    pub status: RecoveryStatus,
    pub target_config_path: PathBuf,
    pub backup_path: PathBuf,
    pub backup_created_before_mutation: bool,
    pub simulated_mutation_applied: bool,
    pub restore_attempted: bool,
    pub restore_verified: bool,
    pub requires_hyprland_keybind: bool,
    pub requires_app_ui: bool,
    pub requires_visible_display: bool,
    pub requires_mouse_input: bool,
    pub reload_run: bool,
    pub eval_run: bool,
    pub lua_executed: bool,
    pub active_runtime_modified: bool,
    pub active_config_modified: bool,
    pub error: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct HighRiskRecoveryPlanner {
    pub backup_root: PathBuf,
    pub now_unix_seconds: u64,
}

impl HighRiskRecoveryPlanner {
    pub fn new(backup_root: impl Into<PathBuf>, now_unix_seconds: u64) -> Self {
        Self {
            backup_root: backup_root.into(),
            now_unix_seconds,
        }
    }

    pub fn arm_for_temp_config(
        &self,
        target_config_path: impl AsRef<Path>,
        proposed_contents: &str,
        timeout_seconds: u64,
    ) -> Result<(HighRiskRecoveryPlan, HighRiskRecoveryResult)> {
        let target_config_path = target_config_path.as_ref();
        ensure_dry_run_target_path(target_config_path)?;
        if timeout_seconds == 0 {
            return Err(anyhow!("timeout_seconds must be greater than zero"));
        }

        let previous_known_good_value =
            fs::read_to_string(target_config_path).with_context(|| {
                format!(
                    "failed to read dry-run target {}",
                    target_config_path.display()
                )
            })?;
        let backup_manager = BackupManager::new(&self.backup_root);
        let backup = backup_manager.create_backup(target_config_path)?;
        atomic_write(target_config_path, proposed_contents.as_bytes())?;

        let recovery_session_id = unique_id("high-risk-recovery")?;
        let confirmation_token = unique_id("confirm")?;
        let plan = HighRiskRecoveryPlan {
            recovery_session_id: recovery_session_id.clone(),
            target_config_path: target_config_path.to_path_buf(),
            backup_path: backup.backup_path.clone(),
            proposed_mutation: proposed_contents.to_owned(),
            previous_known_good_value,
            timeout_seconds,
            confirmation_token,
            confirmation_deadline_unix_seconds: self.now_unix_seconds + timeout_seconds,
            recovery_action: "restore-backup-file-and-reread".to_owned(),
            cleanup_action: "retain-backup-for-audit".to_owned(),
            status: RecoveryStatus::Armed,
        };
        let result = base_result(&plan, RecoveryStatus::Armed, true, true);
        Ok((plan, result))
    }

    pub fn confirm(
        &self,
        mut plan: HighRiskRecoveryPlan,
        confirmation_token: &str,
    ) -> HighRiskRecoveryResult {
        if plan.confirmation_token != confirmation_token {
            return failed_result(&plan, "confirmation token did not match");
        }
        plan.status = RecoveryStatus::Confirmed;
        base_result(&plan, RecoveryStatus::Confirmed, true, true)
    }

    pub fn expire_and_recover(&self, plan: &HighRiskRecoveryPlan) -> HighRiskRecoveryResult {
        let backup = ConfigBackup {
            source_path: plan.target_config_path.clone(),
            backup_path: plan.backup_path.clone(),
            byte_len: plan.previous_known_good_value.len(),
        };
        let backup_manager = BackupManager::new(&self.backup_root);
        if let Err(error) = backup_manager.rollback(&backup) {
            return failed_result(plan, &format!("restore failed: {error:#}"));
        }

        match fs::read_to_string(&plan.target_config_path) {
            Ok(restored) if restored == plan.previous_known_good_value => {
                let mut result = base_result(plan, RecoveryStatus::Reverted, true, true);
                result.restore_attempted = true;
                result.restore_verified = true;
                result
            }
            Ok(restored) => failed_result(
                plan,
                &format!("restore verification failed; got {restored:?}"),
            ),
            Err(error) => failed_result(plan, &format!("restore reread failed: {error:#}")),
        }
    }
}

pub fn ensure_dry_run_target_path(path: &Path) -> Result<()> {
    if !path.is_absolute() {
        return Err(anyhow!("dry-run recovery target must be an absolute path"));
    }
    if !is_under_temp_dir(path) {
        return Err(anyhow!(
            "dry-run recovery target must be under the system temp directory"
        ));
    }
    if looks_like_live_hyprland_config(path) {
        return Err(anyhow!(
            "dry-run recovery refuses live Hyprland config paths"
        ));
    }
    Ok(())
}

fn base_result(
    plan: &HighRiskRecoveryPlan,
    status: RecoveryStatus,
    backup_created_before_mutation: bool,
    simulated_mutation_applied: bool,
) -> HighRiskRecoveryResult {
    HighRiskRecoveryResult {
        recovery_session_id: plan.recovery_session_id.clone(),
        status,
        target_config_path: plan.target_config_path.clone(),
        backup_path: plan.backup_path.clone(),
        backup_created_before_mutation,
        simulated_mutation_applied,
        restore_attempted: false,
        restore_verified: false,
        requires_hyprland_keybind: false,
        requires_app_ui: false,
        requires_visible_display: false,
        requires_mouse_input: false,
        reload_run: false,
        eval_run: false,
        lua_executed: false,
        active_runtime_modified: false,
        active_config_modified: false,
        error: None,
    }
}

fn failed_result(plan: &HighRiskRecoveryPlan, error: &str) -> HighRiskRecoveryResult {
    let mut result = base_result(plan, RecoveryStatus::Failed, true, true);
    result.restore_attempted = true;
    result.error = Some(error.to_owned());
    result
}

fn is_under_temp_dir(path: &Path) -> bool {
    path.starts_with(std::env::temp_dir())
}

fn looks_like_live_hyprland_config(path: &Path) -> bool {
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

fn atomic_write(target: &Path, bytes: &[u8]) -> Result<()> {
    let parent = target
        .parent()
        .ok_or_else(|| anyhow!("target path has no parent"))?;
    let temp_path = parent.join(format!(
        ".{}.high-risk-recovery-{}.tmp",
        target
            .file_name()
            .and_then(|name| name.to_str())
            .unwrap_or("hyprland.conf"),
        unique_id("write")?
    ));
    {
        let mut file = OpenOptions::new()
            .write(true)
            .create_new(true)
            .open(&temp_path)
            .with_context(|| format!("failed to create temp {}", temp_path.display()))?;
        file.write_all(bytes)
            .with_context(|| format!("failed to write temp {}", temp_path.display()))?;
        file.sync_all()
            .with_context(|| format!("failed to sync temp {}", temp_path.display()))?;
    }
    fs::rename(&temp_path, target).with_context(|| {
        format!(
            "failed to replace {} from temp {}",
            target.display(),
            temp_path.display()
        )
    })?;
    Ok(())
}

fn unique_id(prefix: &str) -> Result<String> {
    let duration = SystemTime::now().duration_since(UNIX_EPOCH)?;
    Ok(format!(
        "{prefix}-{}-{}",
        std::process::id(),
        duration.as_nanos()
    ))
}

pub fn deadline_from_now(timeout_seconds: u64) -> Result<SystemTime> {
    Ok(SystemTime::now()
        .checked_add(Duration::from_secs(timeout_seconds))
        .ok_or_else(|| anyhow!("deadline overflow"))?)
}
