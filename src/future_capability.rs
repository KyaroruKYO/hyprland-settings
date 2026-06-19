use std::fs;
use std::path::{Path, PathBuf};

use anyhow::{anyhow, Result};

use crate::config_parser::{parse_hyprland_config_file, ParseStatus};
use crate::missing_default_insertion::MissingDefaultInsertionPlan;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct DisabledInsertionReview {
    pub setting_id: String,
    pub target_path: PathBuf,
    pub proposed_line: String,
    pub production_apply_enabled: bool,
    pub user_copy: String,
    pub required_gates: Vec<String>,
}

pub fn disabled_missing_default_insertion_review(
    plan: &MissingDefaultInsertionPlan,
) -> DisabledInsertionReview {
    DisabledInsertionReview {
        setting_id: plan.setting_id.clone(),
        target_path: plan.target_path.clone(),
        proposed_line: plan.insertion_line.clone(),
        production_apply_enabled: false,
        user_copy: "This setting uses Hyprland's default value. Insertion review is available as fixture proof only; production Apply does not add new config lines yet.".to_string(),
        required_gates: vec![
            "explicit insertion target".to_string(),
            "backup byte-equality proof".to_string(),
            "reread verification".to_string(),
            "restore-on-failure proof".to_string(),
            "production UI approval gate".to_string(),
        ],
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct DuplicateOccurrence {
    pub setting_id: String,
    pub path: PathBuf,
    pub line_number: usize,
    pub raw_line: String,
    pub raw_value: String,
    pub source_depth: usize,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct DuplicateOccurrenceModel {
    pub setting_id: String,
    pub occurrences: Vec<DuplicateOccurrence>,
    pub selector_enabled: bool,
    pub production_write_enabled: bool,
    pub user_copy: String,
}

pub fn duplicate_occurrence_model(
    setting_id: &str,
    files: &[(PathBuf, usize)],
) -> Result<DuplicateOccurrenceModel> {
    let mut occurrences = Vec::new();
    for (path, source_depth) in files {
        let parsed = parse_hyprland_config_file(path)?;
        for record in parsed.scalar_records() {
            if record.normalized_setting_id.as_deref() == Some(setting_id) {
                occurrences.push(DuplicateOccurrence {
                    setting_id: setting_id.to_string(),
                    path: record.path.clone(),
                    line_number: record.line_number,
                    raw_line: record.raw_line.clone(),
                    raw_value: record.raw_value.clone().unwrap_or_default(),
                    source_depth: *source_depth,
                });
            }
        }
    }

    Ok(DuplicateOccurrenceModel {
        setting_id: setting_id.to_string(),
        selector_enabled: false,
        production_write_enabled: false,
        user_copy: "This setting appears more than once. The app can show each occurrence, but production Apply will not choose one automatically.".to_string(),
        occurrences,
    })
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct DuplicateReplacementRequest {
    pub occurrence: DuplicateOccurrence,
    pub expected_old_value: String,
    pub proposed_value: String,
    pub backup_stamp: String,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum DuplicateReplacementStatus {
    Succeeded,
    Blocked,
    RecoveredFailure,
    UnrecoveredFailure,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct DuplicateReplacementOptions {
    pub force_verification_failure: bool,
    pub force_restore_failure: bool,
}

impl Default for DuplicateReplacementOptions {
    fn default() -> Self {
        Self {
            force_verification_failure: false,
            force_restore_failure: false,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct DuplicateReplacementReport {
    pub status: DuplicateReplacementStatus,
    pub backup_path: Option<PathBuf>,
    pub backup_bytes_equal: bool,
    pub exact_line_replaced: bool,
    pub reread_verified: bool,
    pub restore_attempted: bool,
    pub restore_succeeded: bool,
    pub production_write_enabled: bool,
    pub real_config_touched: bool,
    pub runtime_touched: bool,
    pub errors: Vec<String>,
}

pub fn replace_duplicate_occurrence_safe_env(
    request: &DuplicateReplacementRequest,
    options: &DuplicateReplacementOptions,
) -> DuplicateReplacementReport {
    let path = &request.occurrence.path;
    if !path.starts_with(std::env::temp_dir()) {
        return DuplicateReplacementReport {
            status: DuplicateReplacementStatus::Blocked,
            backup_path: None,
            backup_bytes_equal: false,
            exact_line_replaced: false,
            reread_verified: false,
            restore_attempted: false,
            restore_succeeded: false,
            production_write_enabled: false,
            real_config_touched: false,
            runtime_touched: false,
            errors: vec!["duplicate replacement proof only accepts temp fixture paths".to_string()],
        };
    }

    let original = match fs::read_to_string(path) {
        Ok(contents) => contents,
        Err(error) => return duplicate_failed(format!("read target failed: {error}")),
    };
    let mut lines: Vec<String> = original.lines().map(str::to_string).collect();
    let Some(line) = lines.get_mut(request.occurrence.line_number.saturating_sub(1)) else {
        return duplicate_failed("line number is outside target file".to_string());
    };
    if line.trim() != request.occurrence.raw_line.trim() {
        return duplicate_failed("raw line no longer matches selected occurrence".to_string());
    }
    if request.occurrence.raw_value.trim() != request.expected_old_value.trim() {
        return duplicate_failed("expected old value does not match occurrence value".to_string());
    }

    let backup_path = path.with_extension(format!(
        "duplicate-replacement-backup-{}",
        request.backup_stamp
    ));
    if let Err(error) = fs::write(&backup_path, original.as_bytes()) {
        return duplicate_failed(format!("backup write failed: {error}"));
    }
    let backup = match fs::read_to_string(&backup_path) {
        Ok(contents) => contents,
        Err(error) => return duplicate_failed(format!("backup reread failed: {error}")),
    };
    if backup != original {
        return duplicate_failed("backup byte equality failed".to_string());
    }

    let key = request
        .occurrence
        .raw_line
        .split_once('=')
        .map(|(key, _)| key.trim())
        .unwrap_or("");
    if key.is_empty() {
        return duplicate_failed("selected occurrence is not an assignment line".to_string());
    }
    *line = format!("{key} = {}", request.proposed_value.trim());
    let mut updated = lines.join("\n");
    if original.ends_with('\n') {
        updated.push('\n');
    }
    if let Err(error) = fs::write(path, updated.as_bytes()) {
        return duplicate_failed(format!("replacement write failed: {error}"));
    }

    if options.force_verification_failure
        || !duplicate_line_verifies(
            path,
            request.occurrence.line_number,
            &request.proposed_value,
        )
        .unwrap_or(false)
    {
        return restore_duplicate(
            path,
            &backup_path,
            &original,
            "replacement verification failed",
            options,
        );
    }

    DuplicateReplacementReport {
        status: DuplicateReplacementStatus::Succeeded,
        backup_path: Some(backup_path),
        backup_bytes_equal: true,
        exact_line_replaced: true,
        reread_verified: true,
        restore_attempted: false,
        restore_succeeded: false,
        production_write_enabled: false,
        real_config_touched: false,
        runtime_touched: false,
        errors: Vec::new(),
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum MockWatchdogState {
    Pending,
    Confirmed,
    TimedOut,
    Reverted,
    RecoveryFailed,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct MockWatchdog {
    pub session_id: String,
    pub confirmation_token: String,
    pub deadline_tick: u64,
    pub state: MockWatchdogState,
    pub real_runtime_enabled: bool,
}

impl MockWatchdog {
    pub fn arm(session_id: &str, confirmation_token: &str, deadline_tick: u64) -> Self {
        Self {
            session_id: session_id.to_string(),
            confirmation_token: confirmation_token.to_string(),
            deadline_tick,
            state: MockWatchdogState::Pending,
            real_runtime_enabled: false,
        }
    }

    pub fn confirm(&mut self, token: &str) -> MockWatchdogState {
        if self.state == MockWatchdogState::Pending && token == self.confirmation_token {
            self.state = MockWatchdogState::Confirmed;
        }
        self.state
    }

    pub fn tick(&mut self, now_tick: u64, restore_succeeds: bool) -> MockWatchdogState {
        if self.state != MockWatchdogState::Pending || now_tick < self.deadline_tick {
            return self.state;
        }
        self.state = MockWatchdogState::TimedOut;
        self.state = if restore_succeeds {
            MockWatchdogState::Reverted
        } else {
            MockWatchdogState::RecoveryFailed
        };
        self.state
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct StructuredFamilyModel {
    pub family_id: String,
    pub entries: Vec<StructuredFamilyEntry>,
    pub editor_enabled: bool,
    pub production_write_enabled: bool,
    pub lossless_render_proven: bool,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct StructuredFamilyEntry {
    pub path: PathBuf,
    pub line_number: usize,
    pub raw_line: String,
    pub parsed_key: String,
    pub raw_value: String,
}

pub fn structured_family_model(
    path: impl AsRef<Path>,
    family_id: &str,
) -> Result<StructuredFamilyModel> {
    let parsed = parse_hyprland_config_file(path)?;
    let entries = parsed
        .records
        .into_iter()
        .filter(|record| {
            record.status == ParseStatus::StructuredRaw
                && record.normalized_setting_id.as_deref() == Some(family_id)
        })
        .map(|record| StructuredFamilyEntry {
            path: record.path,
            line_number: record.line_number,
            raw_line: record.raw_line,
            parsed_key: record.parsed_key.unwrap_or_default(),
            raw_value: record.raw_value.unwrap_or_default(),
        })
        .collect();
    Ok(StructuredFamilyModel {
        family_id: family_id.to_string(),
        entries,
        editor_enabled: false,
        production_write_enabled: false,
        lossless_render_proven: false,
    })
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ProfileSwitchStatus {
    Succeeded,
    Blocked,
    RestoredAfterFailure,
    RestoreFailed,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ProfileSwitchReport {
    pub status: ProfileSwitchStatus,
    pub original_target: Option<PathBuf>,
    pub target_after_switch: Option<PathBuf>,
    pub restored_target: Option<PathBuf>,
    pub production_switch_enabled: bool,
    pub real_config_touched: bool,
    pub runtime_touched: bool,
    pub errors: Vec<String>,
}

#[cfg(unix)]
pub fn switch_profile_symlink_safe_env(
    root: impl AsRef<Path>,
    current_symlink: impl AsRef<Path>,
    target_profile: impl AsRef<Path>,
    force_restore_failure: bool,
) -> ProfileSwitchReport {
    use std::os::unix::fs::symlink;

    let root = root.as_ref();
    let current_symlink = current_symlink.as_ref();
    let target_profile = target_profile.as_ref();
    if !root.starts_with(std::env::temp_dir())
        || !current_symlink.starts_with(root)
        || !target_profile.starts_with(root)
    {
        return profile_switch_blocked("profile switch proof only accepts temp fixture paths");
    }
    let original_target = match fs::read_link(current_symlink) {
        Ok(target) => target,
        Err(error) => {
            return profile_switch_blocked(&format!("read current symlink failed: {error}"))
        }
    };
    if let Err(error) = fs::remove_file(current_symlink) {
        return profile_switch_blocked(&format!("remove current symlink failed: {error}"));
    }
    if let Err(error) = symlink(target_profile, current_symlink) {
        let _ = symlink(&original_target, current_symlink);
        return profile_switch_blocked(&format!("switch symlink failed: {error}"));
    }
    let target_after_switch = fs::read_link(current_symlink).ok();
    let restore_result = if force_restore_failure {
        Err(anyhow!("forced restore failure"))
    } else {
        fs::remove_file(current_symlink)
            .map_err(anyhow::Error::from)
            .and_then(|_| symlink(&original_target, current_symlink).map_err(anyhow::Error::from))
    };

    match restore_result {
        Ok(()) => ProfileSwitchReport {
            status: ProfileSwitchStatus::Succeeded,
            original_target: Some(original_target),
            target_after_switch,
            restored_target: fs::read_link(current_symlink).ok(),
            production_switch_enabled: false,
            real_config_touched: false,
            runtime_touched: false,
            errors: Vec::new(),
        },
        Err(error) => ProfileSwitchReport {
            status: ProfileSwitchStatus::RestoreFailed,
            original_target: Some(original_target),
            target_after_switch,
            restored_target: fs::read_link(current_symlink).ok(),
            production_switch_enabled: false,
            real_config_touched: false,
            runtime_touched: false,
            errors: vec![error.to_string()],
        },
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum RuntimeAction {
    Reload,
    Keyword { key: String, value: String },
    Dispatch { command: String },
    Status { query: String },
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RuntimeDryRunResult {
    pub action: RuntimeAction,
    pub accepted_by_allowlist: bool,
    pub would_mutate_runtime: bool,
    pub real_command_executed: bool,
    pub production_runtime_enabled: bool,
    pub explanation: String,
}

#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct RuntimeDryRunExecutor {
    pub recorded_actions: Vec<RuntimeDryRunResult>,
}

impl RuntimeDryRunExecutor {
    pub fn evaluate(&mut self, action: RuntimeAction) -> RuntimeDryRunResult {
        let would_mutate_runtime = !matches!(action, RuntimeAction::Status { .. });
        let accepted_by_allowlist = matches!(action, RuntimeAction::Status { .. });
        let explanation = if would_mutate_runtime {
            "runtime mutation is dry-run only; no hyprctl command was executed"
        } else {
            "read-only status query represented without command execution"
        }
        .to_string();
        let result = RuntimeDryRunResult {
            action,
            accepted_by_allowlist,
            would_mutate_runtime,
            real_command_executed: false,
            production_runtime_enabled: false,
            explanation,
        };
        self.recorded_actions.push(result.clone());
        result
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct VersionMigrationAssessment {
    pub current_default_version: String,
    pub requested_version: String,
    pub trusted_export_available: bool,
    pub migration_activated: bool,
    pub production_default_changed: bool,
    pub status: String,
    pub blockers: Vec<String>,
}

pub fn assess_hyprland_version_migration(
    requested_version: &str,
    trusted_export_available: bool,
) -> VersionMigrationAssessment {
    let mut blockers = Vec::new();
    if requested_version != "0.55.2" && !trusted_export_available {
        blockers.push("trusted official export data is required before migration".to_string());
    }
    VersionMigrationAssessment {
        current_default_version: "0.55.2".to_string(),
        requested_version: requested_version.to_string(),
        trusted_export_available,
        migration_activated: false,
        production_default_changed: false,
        status: if blockers.is_empty() && requested_version == "0.55.2" {
            "already-current"
        } else {
            "assessment-only"
        }
        .to_string(),
        blockers,
    }
}

fn duplicate_line_verifies(path: &Path, line_number: usize, proposed_value: &str) -> Result<bool> {
    let parsed = parse_hyprland_config_file(path)?;
    Ok(parsed.records.iter().any(|record| {
        record.line_number == line_number && record.raw_value.as_deref() == Some(proposed_value)
    }))
}

fn duplicate_failed(error: String) -> DuplicateReplacementReport {
    DuplicateReplacementReport {
        status: DuplicateReplacementStatus::Blocked,
        backup_path: None,
        backup_bytes_equal: false,
        exact_line_replaced: false,
        reread_verified: false,
        restore_attempted: false,
        restore_succeeded: false,
        production_write_enabled: false,
        real_config_touched: false,
        runtime_touched: false,
        errors: vec![error],
    }
}

fn restore_duplicate(
    path: &Path,
    backup_path: &Path,
    original: &str,
    error: &str,
    options: &DuplicateReplacementOptions,
) -> DuplicateReplacementReport {
    if options.force_restore_failure {
        return DuplicateReplacementReport {
            status: DuplicateReplacementStatus::UnrecoveredFailure,
            backup_path: Some(backup_path.to_path_buf()),
            backup_bytes_equal: true,
            exact_line_replaced: true,
            reread_verified: false,
            restore_attempted: true,
            restore_succeeded: false,
            production_write_enabled: false,
            real_config_touched: false,
            runtime_touched: false,
            errors: vec![error.to_string(), "forced restore failure".to_string()],
        };
    }
    let restore_succeeded = fs::write(path, original.as_bytes()).is_ok()
        && fs::read_to_string(path)
            .map(|contents| contents == original)
            .unwrap_or(false);
    DuplicateReplacementReport {
        status: if restore_succeeded {
            DuplicateReplacementStatus::RecoveredFailure
        } else {
            DuplicateReplacementStatus::UnrecoveredFailure
        },
        backup_path: Some(backup_path.to_path_buf()),
        backup_bytes_equal: true,
        exact_line_replaced: true,
        reread_verified: false,
        restore_attempted: true,
        restore_succeeded,
        production_write_enabled: false,
        real_config_touched: false,
        runtime_touched: false,
        errors: vec![error.to_string()],
    }
}

fn profile_switch_blocked(error: &str) -> ProfileSwitchReport {
    ProfileSwitchReport {
        status: ProfileSwitchStatus::Blocked,
        original_target: None,
        target_after_switch: None,
        restored_target: None,
        production_switch_enabled: false,
        real_config_touched: false,
        runtime_touched: false,
        errors: vec![error.to_string()],
    }
}
