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

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum HighRiskWatchdogMode {
    DryRunTempOnly,
    ProductionPlannedDisabled,
    LiveConfigPlannedDisabled,
    LiveConfigArmed,
    LiveConfigConfirmed,
    LiveConfigReverted,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct HighRiskWatchdogPlan {
    pub mode: HighRiskWatchdogMode,
    pub plan_path: PathBuf,
    pub result_log_path: PathBuf,
    pub recovery: HighRiskRecoveryPlan,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct HighRiskWatchdogResult {
    pub mode: HighRiskWatchdogMode,
    pub plan_path: PathBuf,
    pub result_log_path: PathBuf,
    pub recovery: HighRiskRecoveryResult,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct InertLiveConfigWatchdogPlan {
    pub mode: HighRiskWatchdogMode,
    pub recovery_session_id: String,
    pub live_config_path: PathBuf,
    pub backup_path: PathBuf,
    pub proposed_mutation_summary: String,
    pub previous_known_good_hash: String,
    pub timeout_seconds: u64,
    pub confirmation_token: String,
    pub confirmation_deadline_unix_seconds: u64,
    pub restore_command_description: String,
    pub result_log_path: PathBuf,
    pub live_execution_enabled: bool,
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

    pub fn arm_watchdog_for_temp_config(
        &self,
        target_config_path: impl AsRef<Path>,
        proposed_contents: &str,
        timeout_seconds: u64,
        plan_path: impl AsRef<Path>,
        result_log_path: impl AsRef<Path>,
    ) -> Result<HighRiskWatchdogPlan> {
        let target_config_path = target_config_path.as_ref();
        ensure_dry_run_target_path(target_config_path)?;
        let plan_path = plan_path.as_ref();
        let result_log_path = result_log_path.as_ref();
        ensure_dry_run_target_path(plan_path)?;
        ensure_dry_run_target_path(result_log_path)?;
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
        let recovery_session_id = unique_id("high-risk-watchdog")?;
        let confirmation_token = unique_id("confirm")?;
        let recovery = HighRiskRecoveryPlan {
            recovery_session_id,
            target_config_path: target_config_path.to_path_buf(),
            backup_path: backup.backup_path,
            proposed_mutation: proposed_contents.to_owned(),
            previous_known_good_value,
            timeout_seconds,
            confirmation_token,
            confirmation_deadline_unix_seconds: self.now_unix_seconds + timeout_seconds,
            recovery_action: "restore-backup-file-and-reread".to_owned(),
            cleanup_action: "retain-backup-and-result-log-for-audit".to_owned(),
            status: RecoveryStatus::Armed,
        };
        let watchdog = HighRiskWatchdogPlan {
            mode: HighRiskWatchdogMode::DryRunTempOnly,
            plan_path: plan_path.to_path_buf(),
            result_log_path: result_log_path.to_path_buf(),
            recovery,
        };

        persist_watchdog_plan(plan_path, &watchdog)?;
        atomic_write(target_config_path, proposed_contents.as_bytes())?;
        Ok(watchdog)
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

    pub fn confirm_watchdog(
        &self,
        plan: &HighRiskWatchdogPlan,
        confirmation_token: &str,
    ) -> Result<HighRiskWatchdogResult> {
        ensure_supported_watchdog_mode(plan.mode)?;
        let recovery = self.confirm(plan.recovery.clone(), confirmation_token);
        if recovery.status == RecoveryStatus::Failed {
            return Err(anyhow!(
                "{}",
                recovery
                    .error
                    .as_deref()
                    .unwrap_or("watchdog confirmation failed")
            ));
        }
        let result = HighRiskWatchdogResult {
            mode: plan.mode,
            plan_path: plan.plan_path.clone(),
            result_log_path: plan.result_log_path.clone(),
            recovery,
        };
        write_watchdog_result(&plan.result_log_path, &result)?;
        Ok(result)
    }

    pub fn expire_watchdog_and_restore(
        &self,
        plan: &HighRiskWatchdogPlan,
    ) -> Result<HighRiskWatchdogResult> {
        ensure_supported_watchdog_mode(plan.mode)?;
        if self.now_unix_seconds < plan.recovery.confirmation_deadline_unix_seconds {
            return Err(anyhow!(
                "watchdog deadline has not expired; now={} deadline={}",
                self.now_unix_seconds,
                plan.recovery.confirmation_deadline_unix_seconds
            ));
        }
        let recovery = self.expire_and_recover(&plan.recovery);
        if recovery.status == RecoveryStatus::Failed {
            let result = HighRiskWatchdogResult {
                mode: plan.mode,
                plan_path: plan.plan_path.clone(),
                result_log_path: plan.result_log_path.clone(),
                recovery,
            };
            let _ = write_watchdog_result(&plan.result_log_path, &result);
            return Err(anyhow!(
                "{}",
                result
                    .recovery
                    .error
                    .as_deref()
                    .unwrap_or("watchdog restore failed")
            ));
        }
        let result = HighRiskWatchdogResult {
            mode: plan.mode,
            plan_path: plan.plan_path.clone(),
            result_log_path: plan.result_log_path.clone(),
            recovery,
        };
        write_watchdog_result(&plan.result_log_path, &result)?;
        Ok(result)
    }

    pub fn build_inert_live_config_plan(
        &self,
        live_config_path: impl Into<PathBuf>,
        backup_path: impl Into<PathBuf>,
        result_log_path: impl Into<PathBuf>,
        proposed_mutation_summary: impl Into<String>,
        previous_known_good_hash: impl Into<String>,
        timeout_seconds: u64,
    ) -> Result<InertLiveConfigWatchdogPlan> {
        if timeout_seconds == 0 {
            return Err(anyhow!("timeout_seconds must be greater than zero"));
        }

        let plan = InertLiveConfigWatchdogPlan {
            mode: HighRiskWatchdogMode::LiveConfigPlannedDisabled,
            recovery_session_id: unique_id("live-config-watchdog")?,
            live_config_path: live_config_path.into(),
            backup_path: backup_path.into(),
            proposed_mutation_summary: proposed_mutation_summary.into(),
            previous_known_good_hash: previous_known_good_hash.into(),
            timeout_seconds,
            confirmation_token: unique_id("confirm")?,
            confirmation_deadline_unix_seconds: self.now_unix_seconds + timeout_seconds,
            restore_command_description:
                "planned restore of backup file followed by file reread verification".to_owned(),
            result_log_path: result_log_path.into(),
            live_execution_enabled: false,
        };
        validate_inert_live_config_plan(&plan)?;
        Ok(plan)
    }
}

pub fn validate_inert_live_config_plan(plan: &InertLiveConfigWatchdogPlan) -> Result<()> {
    if plan.mode != HighRiskWatchdogMode::LiveConfigPlannedDisabled {
        return Err(anyhow!(
            "inert live config plan must use live-config-planned-disabled mode"
        ));
    }
    if plan.live_execution_enabled {
        return Err(anyhow!(
            "inert live config plan must keep live execution disabled"
        ));
    }
    if plan.live_config_path.as_os_str().is_empty() {
        return Err(anyhow!("live config path must be present"));
    }
    if plan.backup_path.as_os_str().is_empty() {
        return Err(anyhow!("backup path must be present"));
    }
    if plan.result_log_path.as_os_str().is_empty() {
        return Err(anyhow!("result log path must be present"));
    }
    if plan.timeout_seconds == 0 {
        return Err(anyhow!("timeout_seconds must be greater than zero"));
    }
    if plan.confirmation_token.is_empty() {
        return Err(anyhow!("confirmation token must be present"));
    }
    if plan.restore_command_description.is_empty() {
        return Err(anyhow!("restore command description must be present"));
    }
    Ok(())
}

pub fn refuse_inert_live_config_execution(plan: &InertLiveConfigWatchdogPlan) -> Result<()> {
    validate_inert_live_config_plan(plan)?;
    Err(anyhow!(
        "live config watchdog execution is disabled; this plan is representation-only"
    ))
}

pub fn persist_watchdog_plan(path: impl AsRef<Path>, plan: &HighRiskWatchdogPlan) -> Result<()> {
    let path = path.as_ref();
    ensure_dry_run_target_path(path)?;
    validate_watchdog_plan(plan)?;
    let bytes = serde_json::to_vec_pretty(plan)?;
    atomic_write(path, &bytes)
}

pub fn load_watchdog_plan(path: impl AsRef<Path>) -> Result<HighRiskWatchdogPlan> {
    let path = path.as_ref();
    ensure_dry_run_target_path(path)?;
    let bytes =
        fs::read(path).with_context(|| format!("failed to read plan {}", path.display()))?;
    let plan: HighRiskWatchdogPlan = serde_json::from_slice(&bytes)
        .with_context(|| format!("failed to parse plan {}", path.display()))?;
    ensure_supported_watchdog_mode(plan.mode)?;
    validate_watchdog_plan(&plan)?;
    Ok(plan)
}

pub fn validate_watchdog_plan(plan: &HighRiskWatchdogPlan) -> Result<()> {
    ensure_supported_watchdog_mode(plan.mode)?;
    match plan.mode {
        HighRiskWatchdogMode::DryRunTempOnly => {
            ensure_dry_run_target_path(&plan.plan_path)?;
            ensure_dry_run_target_path(&plan.result_log_path)?;
            ensure_dry_run_target_path(&plan.recovery.target_config_path)?;
            ensure_dry_run_target_path(&plan.recovery.backup_path)?;
        }
        HighRiskWatchdogMode::ProductionPlannedDisabled
        | HighRiskWatchdogMode::LiveConfigPlannedDisabled
        | HighRiskWatchdogMode::LiveConfigArmed
        | HighRiskWatchdogMode::LiveConfigConfirmed
        | HighRiskWatchdogMode::LiveConfigReverted => {}
    }
    Ok(())
}

pub fn write_watchdog_result(
    path: impl AsRef<Path>,
    result: &HighRiskWatchdogResult,
) -> Result<()> {
    let path = path.as_ref();
    ensure_dry_run_target_path(path)?;
    let bytes = serde_json::to_vec_pretty(result)?;
    atomic_write(path, &bytes)
}

pub fn load_watchdog_result(path: impl AsRef<Path>) -> Result<HighRiskWatchdogResult> {
    let path = path.as_ref();
    ensure_dry_run_target_path(path)?;
    let bytes =
        fs::read(path).with_context(|| format!("failed to read result {}", path.display()))?;
    Ok(serde_json::from_slice(&bytes)
        .with_context(|| format!("failed to parse result {}", path.display()))?)
}

fn ensure_supported_watchdog_mode(mode: HighRiskWatchdogMode) -> Result<()> {
    match mode {
        HighRiskWatchdogMode::DryRunTempOnly => Ok(()),
        HighRiskWatchdogMode::ProductionPlannedDisabled
        | HighRiskWatchdogMode::LiveConfigPlannedDisabled
        | HighRiskWatchdogMode::LiveConfigArmed
        | HighRiskWatchdogMode::LiveConfigConfirmed
        | HighRiskWatchdogMode::LiveConfigReverted => {
            Err(anyhow!("live config watchdog mode is planned but disabled"))
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
