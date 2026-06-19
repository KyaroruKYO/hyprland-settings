use std::collections::{BTreeMap, BTreeSet};
use std::fs::{self, OpenOptions};
use std::io::Write;
use std::path::{Path, PathBuf};
use std::time::{SystemTime, UNIX_EPOCH};

use anyhow::{anyhow, Context, Result};

use crate::config_graph::{ConfigGraphFile, ConfigGraphSummary, ConfigManagementHintKind};
use crate::config_parser::{parse_hyprland_config_file, parse_hyprland_config_text, ParseStatus};
use crate::current_config::{CurrentConfigSnapshot, CurrentValueSourceStatus};
use crate::pending_change::{stage_pending_change, PendingChangeValidation};
use crate::production_backup_contract::{
    backup_path_policy_for_target, choose_unique_backup_path, BackupPathPolicy,
};
use crate::production_recovery_contract::PRODUCTION_RECOVERY_CONTRACT_ENABLED;
use crate::production_verification_contract::PRODUCTION_VERIFICATION_CONTRACT_ENABLED;
use crate::write_classification::{
    config_key_from_official_setting, high_risk_write_policy, is_safe_writable_setting,
    safe_writable_official_setting, SAFE_WRITABLE_ROWS,
};
use crate::write_review_walkthrough::PRODUCTION_WRITE_REVIEW_WALKTHROUGH_CAN_WRITE;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SafeBatchChangeRequest {
    pub setting_id: String,
    pub proposed_value: String,
}

impl SafeBatchChangeRequest {
    pub fn new(setting_id: impl Into<String>, proposed_value: impl Into<String>) -> Self {
        Self {
            setting_id: setting_id.into(),
            proposed_value: proposed_value.into(),
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SafeBatchEligibility {
    EligibleSafeBatchScalar,
    BlockedHighRisk,
    BlockedDisplayRenderRisk,
    BlockedGeneratedFile,
    BlockedScriptManaged,
    BlockedSymlinkManaged,
    BlockedAmbiguousFile,
    BlockedDuplicateConflict,
    BlockedMissingLine,
    BlockedStructuredFamily,
    BlockedUnknownTarget,
    BlockedRuntimeOnly,
    BlockedProfileModeSwitch,
}

impl SafeBatchEligibility {
    pub fn label(self) -> &'static str {
        match self {
            Self::EligibleSafeBatchScalar => "eligible_safe_batch_scalar",
            Self::BlockedHighRisk => "blocked_high_risk",
            Self::BlockedDisplayRenderRisk => "blocked_display_render_risk",
            Self::BlockedGeneratedFile => "blocked_generated_file",
            Self::BlockedScriptManaged => "blocked_script_managed",
            Self::BlockedSymlinkManaged => "blocked_symlink_managed",
            Self::BlockedAmbiguousFile => "blocked_ambiguous_file",
            Self::BlockedDuplicateConflict => "blocked_duplicate_conflict",
            Self::BlockedMissingLine => "blocked_missing_line",
            Self::BlockedStructuredFamily => "blocked_structured_family",
            Self::BlockedUnknownTarget => "blocked_unknown_target",
            Self::BlockedRuntimeOnly => "blocked_runtime_only",
            Self::BlockedProfileModeSwitch => "blocked_profile_mode_switch",
        }
    }

    pub fn user_facing_blocked_copy(self) -> &'static str {
        match self {
            Self::EligibleSafeBatchScalar => "Safe batch write is available for this setting.",
            Self::BlockedHighRisk => {
                "Blocked: this setting needs a family-specific recovery path before the app can write it."
            }
            Self::BlockedDisplayRenderRisk => {
                "Blocked: display/render settings need separate safety approval."
            }
            Self::BlockedGeneratedFile => {
                "Blocked: this file appears to be generated, so the app will not write it yet."
            }
            Self::BlockedScriptManaged => {
                "Blocked: this file may be changed by a script, so safe-batch writing is disabled."
            }
            Self::BlockedSymlinkManaged => {
                "Blocked: this file may be a symlink or current-profile file, so the app will not write it yet."
            }
            Self::BlockedAmbiguousFile => "Blocked: this file target is ambiguous.",
            Self::BlockedDuplicateConflict => {
                "Blocked: this setting appears in more than one place. Resolve the duplicate entries manually before applying."
            }
            Self::BlockedMissingLine => {
                "Blocked: this setting is using Hyprland's default value. The app does not add new config lines yet."
            }
            Self::BlockedStructuredFamily => {
                "Blocked: structured settings are not part of safe-batch writing yet."
            }
            Self::BlockedUnknownTarget => "Blocked: this setting does not have a safe target yet.",
            Self::BlockedRuntimeOnly => {
                "Blocked: runtime-only settings are not part of this batch."
            }
            Self::BlockedProfileModeSwitch => {
                "Blocked: profile and mode switching are not part of this batch."
            }
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SafeBatchEligibleChange {
    pub setting_id: String,
    pub official_setting: String,
    pub config_key: String,
    pub target_path: PathBuf,
    pub line_number: usize,
    pub old_value: String,
    pub proposed_value: String,
    pub original_raw_line: String,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SafeBatchBlockedChange {
    pub setting_id: String,
    pub proposed_value: String,
    pub reason: SafeBatchEligibility,
    pub evidence: String,
    pub user_facing_copy: String,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SafeBatchTargetFilePlan {
    pub target_path: PathBuf,
    pub backup_policy: BackupPathPolicy,
    pub change_count: usize,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SafeBatchWritePlan {
    pub batch_id: String,
    pub pending_changes: Vec<SafeBatchChangeRequest>,
    pub eligible_changes: Vec<SafeBatchEligibleChange>,
    pub blocked_changes: Vec<SafeBatchBlockedChange>,
    pub target_files: Vec<SafeBatchTargetFilePlan>,
    pub can_execute: bool,
    pub cannot_execute_reasons: Vec<String>,
    pub safe_writable_rows_len: usize,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SafeBatchExecutionOptions {
    pub backup_timestamp: String,
    pub fail_after_writing_target: Option<PathBuf>,
    pub force_backup_verification_failure_for: Option<PathBuf>,
    pub force_verification_failure_for: Option<String>,
    pub force_restore_failure: bool,
}

impl Default for SafeBatchExecutionOptions {
    fn default() -> Self {
        Self {
            backup_timestamp: unique_stamp().unwrap_or_else(|_| "safe-batch".to_string()),
            fail_after_writing_target: None,
            force_backup_verification_failure_for: None,
            force_verification_failure_for: None,
            force_restore_failure: false,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SafeBatchBackupRecord {
    pub target_path: PathBuf,
    pub backup_path: PathBuf,
    pub bytes_equal: bool,
    pub byte_len: usize,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SafeBatchWriteStatus {
    Blocked,
    Succeeded,
    RecoveredFailure,
    UnrecoveredFailure,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SafeBatchWriteReport {
    pub batch_id: String,
    pub status: SafeBatchWriteStatus,
    pub backups: Vec<SafeBatchBackupRecord>,
    pub verified_changes: Vec<String>,
    pub failures: Vec<String>,
    pub recovery_attempted: bool,
    pub recovery_succeeded: bool,
    pub restore_verification_succeeded: bool,
    pub hyprland_reload_attempted: bool,
    pub mutating_hyprctl_used: bool,
    pub runtime_mutated: bool,
}

pub fn safe_batch_write_execution_gate_enabled() -> bool {
    PRODUCTION_WRITE_REVIEW_WALKTHROUGH_CAN_WRITE
        && PRODUCTION_VERIFICATION_CONTRACT_ENABLED
        && PRODUCTION_RECOVERY_CONTRACT_ENABLED
}

pub fn safe_batch_write_user_facing_lines() -> Vec<String> {
    vec![
        "Safe batch write is available for normal settings.".to_string(),
        "Some settings are blocked because they need extra safety review.".to_string(),
        "The app will back up files before writing.".to_string(),
        "The app will check the result after writing.".to_string(),
        "If something fails, the app will restore the backup.".to_string(),
    ]
}

pub fn build_safe_batch_write_plan(
    batch_id: impl Into<String>,
    known_setting_ids: &BTreeSet<String>,
    current_config: &CurrentConfigSnapshot,
    config_graph: &ConfigGraphSummary,
    pending_changes: Vec<SafeBatchChangeRequest>,
    backup_timestamp: impl AsRef<str>,
) -> SafeBatchWritePlan {
    let batch_id = batch_id.into();
    let mut eligible_changes = Vec::new();
    let mut blocked_changes = Vec::new();

    for request in &pending_changes {
        match classify_safe_batch_change(known_setting_ids, current_config, config_graph, request) {
            Ok(change) => eligible_changes.push(change),
            Err(blocked) => blocked_changes.push(blocked),
        }
    }

    let mut file_counts: BTreeMap<PathBuf, usize> = BTreeMap::new();
    for change in &eligible_changes {
        *file_counts.entry(change.target_path.clone()).or_default() += 1;
    }
    let target_files = file_counts
        .into_iter()
        .map(|(target_path, change_count)| SafeBatchTargetFilePlan {
            backup_policy: backup_path_policy_for_target(&target_path, backup_timestamp.as_ref()),
            target_path,
            change_count,
        })
        .collect::<Vec<_>>();

    let mut cannot_execute_reasons = Vec::new();
    if pending_changes.is_empty() {
        cannot_execute_reasons.push("no pending changes were selected".to_string());
    }
    if eligible_changes.is_empty() {
        cannot_execute_reasons
            .push("no eligible safe-batch scalar changes were selected".to_string());
    }
    if !blocked_changes.is_empty() {
        cannot_execute_reasons.push(
            "one or more selected changes are blocked; partial apply is not enabled".to_string(),
        );
    }
    if !safe_batch_write_execution_gate_enabled() {
        cannot_execute_reasons.push("safe-batch write execution gate is not enabled".to_string());
    }

    SafeBatchWritePlan {
        batch_id,
        pending_changes,
        eligible_changes,
        blocked_changes,
        target_files,
        can_execute: cannot_execute_reasons.is_empty(),
        cannot_execute_reasons,
        safe_writable_rows_len: SAFE_WRITABLE_ROWS.len(),
    }
}

pub fn execute_safe_batch_write_plan(
    plan: &SafeBatchWritePlan,
    options: &SafeBatchExecutionOptions,
) -> SafeBatchWriteReport {
    if !plan.can_execute {
        return SafeBatchWriteReport {
            batch_id: plan.batch_id.clone(),
            status: SafeBatchWriteStatus::Blocked,
            backups: Vec::new(),
            verified_changes: Vec::new(),
            failures: plan.cannot_execute_reasons.clone(),
            recovery_attempted: false,
            recovery_succeeded: false,
            restore_verification_succeeded: false,
            hyprland_reload_attempted: false,
            mutating_hyprctl_used: false,
            runtime_mutated: false,
        };
    }

    let mut backups = Vec::new();
    for target in &plan.target_files {
        match create_verified_backup(
            &target.target_path,
            &options.backup_timestamp,
            options.force_backup_verification_failure_for.as_ref(),
        ) {
            Ok(record) => backups.push(record),
            Err(error) => {
                return failure_without_recovery(
                    &plan.batch_id,
                    backups,
                    format!(
                        "backup failed for {}: {error}",
                        target.target_path.display()
                    ),
                );
            }
        }
    }

    let backup_map = backups
        .iter()
        .map(|backup| (backup.target_path.clone(), backup.backup_path.clone()))
        .collect::<BTreeMap<_, _>>();

    let mut written_targets = Vec::new();
    for target in &plan.target_files {
        let changes = changes_for_target(&plan.eligible_changes, &target.target_path);
        match write_target_file(&target.target_path, &changes) {
            Ok(()) => {
                written_targets.push(target.target_path.clone());
                if options.fail_after_writing_target.as_ref() == Some(&target.target_path) {
                    return recover_failure(
                        plan,
                        backups,
                        "write failed after target replacement".to_string(),
                        options.force_restore_failure,
                    );
                }
            }
            Err(error) => {
                return recover_failure(
                    plan,
                    backups,
                    format!("write failed for {}: {error}", target.target_path.display()),
                    options.force_restore_failure,
                );
            }
        }
    }

    match verify_written_values(
        &plan.eligible_changes,
        options.force_verification_failure_for.as_deref(),
    ) {
        Ok(verified_changes) => SafeBatchWriteReport {
            batch_id: plan.batch_id.clone(),
            status: SafeBatchWriteStatus::Succeeded,
            backups,
            verified_changes,
            failures: Vec::new(),
            recovery_attempted: false,
            recovery_succeeded: false,
            restore_verification_succeeded: false,
            hyprland_reload_attempted: false,
            mutating_hyprctl_used: false,
            runtime_mutated: false,
        },
        Err(error) => {
            let mut report = recover_failure(
                plan,
                backups,
                format!("verification failed: {error}"),
                options.force_restore_failure,
            );
            if written_targets.is_empty() && !backup_map.is_empty() {
                report
                    .failures
                    .push("no target writes completed before verification failure".to_string());
            }
            report
        }
    }
}

fn classify_safe_batch_change(
    known_setting_ids: &BTreeSet<String>,
    current_config: &CurrentConfigSnapshot,
    config_graph: &ConfigGraphSummary,
    request: &SafeBatchChangeRequest,
) -> Result<SafeBatchEligibleChange, SafeBatchBlockedChange> {
    let setting_id = request.setting_id.as_str();
    if setting_id.starts_with("hl.") {
        return blocked(
            request,
            SafeBatchEligibility::BlockedStructuredFamily,
            "structured config family",
        );
    }
    if setting_id.contains("profile") || setting_id.contains("mode_switch") {
        return blocked(
            request,
            SafeBatchEligibility::BlockedProfileModeSwitch,
            "profile/mode switching is excluded",
        );
    }
    if setting_id.starts_with("runtime.") {
        return blocked(
            request,
            SafeBatchEligibility::BlockedRuntimeOnly,
            "runtime-only setting is excluded",
        );
    }
    if display_render_risky(setting_id) {
        return blocked(
            request,
            SafeBatchEligibility::BlockedDisplayRenderRisk,
            "display/render risk policy is present",
        );
    }
    if high_risk_write_policy(setting_id).is_some() {
        return blocked(
            request,
            SafeBatchEligibility::BlockedHighRisk,
            "high-risk write policy is present",
        );
    }
    if !known_setting_ids.contains(setting_id) || !is_safe_writable_setting(setting_id) {
        return blocked(
            request,
            SafeBatchEligibility::BlockedUnknownTarget,
            "setting is not a known safe writable row",
        );
    }
    let official_setting = safe_writable_official_setting(setting_id).ok_or_else(|| {
        blocked_value(
            request,
            SafeBatchEligibility::BlockedUnknownTarget,
            "missing official safe writable setting",
        )
    })?;
    let current_value = current_config.value_for(official_setting);
    let pending = stage_pending_change(setting_id, &current_value, request.proposed_value.clone());
    match &pending.validation {
        PendingChangeValidation::Valid => {}
        PendingChangeValidation::Invalid { reason }
        | PendingChangeValidation::NotAllowed { reason } => {
            return blocked(request, SafeBatchEligibility::BlockedUnknownTarget, reason);
        }
    }
    let CurrentValueSourceStatus::Configured = current_value.status else {
        let reason = match current_value.status {
            CurrentValueSourceStatus::DuplicateConflict => {
                SafeBatchEligibility::BlockedDuplicateConflict
            }
            CurrentValueSourceStatus::NotConfigured => SafeBatchEligibility::BlockedMissingLine,
            CurrentValueSourceStatus::ReadUnavailable => SafeBatchEligibility::BlockedUnknownTarget,
            CurrentValueSourceStatus::Configured => unreachable!(),
        };
        return blocked(request, reason, current_value.status_label());
    };
    let Some(target_path) = current_value.source_path.clone() else {
        return blocked(
            request,
            SafeBatchEligibility::BlockedMissingLine,
            "missing source path",
        );
    };
    let Some(line_number) = current_value.line_number else {
        return blocked(
            request,
            SafeBatchEligibility::BlockedMissingLine,
            "missing line number",
        );
    };
    let Some(old_value) = current_value.raw_value.clone() else {
        return blocked(
            request,
            SafeBatchEligibility::BlockedUnknownTarget,
            "missing old value",
        );
    };
    let Some(original_raw_line) = current_value.raw_line.clone() else {
        return blocked(
            request,
            SafeBatchEligibility::BlockedMissingLine,
            "missing raw source line",
        );
    };
    let Some(file) = graph_file_for(config_graph, &target_path) else {
        return blocked(
            request,
            SafeBatchEligibility::BlockedAmbiguousFile,
            "target file is not in the reviewed config graph",
        );
    };
    if file.is_symlink
        || file.symlink_target.is_some()
        || has_hint(file, ConfigManagementHintKind::SymlinkManaged)
    {
        return blocked(
            request,
            SafeBatchEligibility::BlockedSymlinkManaged,
            "target file is symlink-managed",
        );
    }
    if has_hint(file, ConfigManagementHintKind::GeneratedFile) {
        return blocked(
            request,
            SafeBatchEligibility::BlockedGeneratedFile,
            "target file is generated",
        );
    }
    if has_hint(file, ConfigManagementHintKind::ScriptManaged)
        || has_hint(file, ConfigManagementHintKind::ScriptReferenced)
    {
        return blocked(
            request,
            SafeBatchEligibility::BlockedScriptManaged,
            "target file is script-managed or script-referenced",
        );
    }
    if !file.readable {
        return blocked(
            request,
            SafeBatchEligibility::BlockedUnknownTarget,
            "target file is not readable",
        );
    }
    if fs::metadata(&target_path)
        .map(|metadata| metadata.permissions().readonly())
        .unwrap_or(true)
    {
        return blocked(
            request,
            SafeBatchEligibility::BlockedUnknownTarget,
            "target file is not writable",
        );
    }
    ensure_source_line_matches(&target_path, line_number, official_setting, &old_value).map_err(
        |error| {
            blocked_value(
                request,
                SafeBatchEligibility::BlockedUnknownTarget,
                error.to_string(),
            )
        },
    )?;

    Ok(SafeBatchEligibleChange {
        setting_id: setting_id.to_string(),
        official_setting: official_setting.to_string(),
        config_key: config_key_from_official_setting(official_setting),
        target_path,
        line_number,
        old_value,
        proposed_value: pending.proposed_value,
        original_raw_line,
    })
}

fn blocked(
    request: &SafeBatchChangeRequest,
    reason: SafeBatchEligibility,
    evidence: impl Into<String>,
) -> Result<SafeBatchEligibleChange, SafeBatchBlockedChange> {
    Err(blocked_value(request, reason, evidence))
}

fn blocked_value(
    request: &SafeBatchChangeRequest,
    reason: SafeBatchEligibility,
    evidence: impl Into<String>,
) -> SafeBatchBlockedChange {
    SafeBatchBlockedChange {
        setting_id: request.setting_id.clone(),
        proposed_value: request.proposed_value.clone(),
        reason,
        evidence: evidence.into(),
        user_facing_copy: reason.user_facing_blocked_copy().to_string(),
    }
}

fn display_render_risky(setting_id: &str) -> bool {
    high_risk_write_policy(setting_id)
        .map(|policy| policy.recovery_bucket.contains("display-render"))
        .unwrap_or(false)
        || matches!(
            setting_id.split('.').next(),
            Some("render" | "xwayland" | "opengl" | "experimental" | "quirks")
        )
        || setting_id == "decoration.screen_shader"
}

fn graph_file_for<'a>(
    graph: &'a ConfigGraphSummary,
    target_path: &Path,
) -> Option<&'a ConfigGraphFile> {
    graph.files.iter().find(|file| {
        file.path == target_path
            || file.resolved_path.as_deref() == Some(target_path)
            || fs::canonicalize(target_path)
                .ok()
                .as_ref()
                .is_some_and(|canonical| file.resolved_path.as_ref() == Some(canonical))
    })
}

fn has_hint(file: &ConfigGraphFile, hint: ConfigManagementHintKind) -> bool {
    file.hints.iter().any(|candidate| candidate.kind == hint)
}

fn ensure_source_line_matches(
    target_path: &Path,
    line_number: usize,
    official_setting: &str,
    old_value: &str,
) -> Result<()> {
    let contents = fs::read_to_string(target_path)
        .with_context(|| format!("failed to read {}", target_path.display()))?;
    let line = contents
        .lines()
        .nth(line_number.saturating_sub(1))
        .ok_or_else(|| anyhow!("target line {line_number} is missing"))?;
    let parsed = parse_hyprland_config_text(target_path, line);
    let record = parsed
        .scalar_records()
        .find(|record| record.normalized_setting_id.as_deref() == Some(official_setting))
        .ok_or_else(|| anyhow!("target line no longer matches {official_setting}"))?;
    if record.raw_value.as_deref() != Some(old_value) {
        return Err(anyhow!(
            "target line old value changed; expected {old_value}, got {:?}",
            record.raw_value
        ));
    }
    Ok(())
}

fn create_verified_backup(
    target_path: &Path,
    timestamp: &str,
    force_backup_verification_failure_for: Option<&PathBuf>,
) -> Result<SafeBatchBackupRecord> {
    let original = fs::read(target_path)
        .with_context(|| format!("failed to read target {}", target_path.display()))?;
    let backup_path = choose_unique_backup_path(target_path, timestamp);
    fs::write(&backup_path, &original)
        .with_context(|| format!("failed to write backup {}", backup_path.display()))?;
    let backup = fs::read(&backup_path)
        .with_context(|| format!("failed to reread backup {}", backup_path.display()))?;
    let forced = force_backup_verification_failure_for
        .map(|path| path == target_path)
        .unwrap_or(false);
    if backup != original || forced {
        return Err(anyhow!("backup byte equality verification failed"));
    }
    Ok(SafeBatchBackupRecord {
        target_path: target_path.to_path_buf(),
        backup_path,
        bytes_equal: true,
        byte_len: original.len(),
    })
}

fn changes_for_target<'a>(
    changes: &'a [SafeBatchEligibleChange],
    target_path: &Path,
) -> Vec<&'a SafeBatchEligibleChange> {
    let mut selected = changes
        .iter()
        .filter(|change| change.target_path == target_path)
        .collect::<Vec<_>>();
    selected.sort_by_key(|change| change.line_number);
    selected
}

fn write_target_file(target_path: &Path, changes: &[&SafeBatchEligibleChange]) -> Result<()> {
    let original = fs::read_to_string(target_path)
        .with_context(|| format!("failed to read {}", target_path.display()))?;
    let had_trailing_newline = original.ends_with('\n');
    let mut lines = original.lines().map(ToOwned::to_owned).collect::<Vec<_>>();
    for change in changes {
        let index = change
            .line_number
            .checked_sub(1)
            .ok_or_else(|| anyhow!("line numbers are 1-based"))?;
        let line = lines
            .get(index)
            .ok_or_else(|| anyhow!("source line {} does not exist", change.line_number))?;
        let parsed = parse_hyprland_config_text(target_path, line);
        let record = parsed
            .scalar_records()
            .find(|record| {
                record.normalized_setting_id.as_deref() == Some(change.official_setting.as_str())
            })
            .ok_or_else(|| anyhow!("source line does not parse as {}", change.official_setting))?;
        if record.raw_value.as_deref() != Some(change.old_value.as_str()) {
            return Err(anyhow!(
                "source line old value changed for {}",
                change.setting_id
            ));
        }
        lines[index] = replace_value_preserving_key(line, &change.proposed_value)?;
    }
    let mut updated = lines.join("\n");
    if had_trailing_newline {
        updated.push('\n');
    }
    atomic_write(target_path, updated.as_bytes())
}

fn verify_written_values(
    changes: &[SafeBatchEligibleChange],
    force_verification_failure_for: Option<&str>,
) -> Result<Vec<String>> {
    let mut verified = Vec::new();
    for change in changes {
        if force_verification_failure_for == Some(change.setting_id.as_str()) {
            return Err(anyhow!(
                "forced verification failure for {}",
                change.setting_id
            ));
        }
        let parsed = parse_hyprland_config_file(&change.target_path)
            .with_context(|| format!("failed to reread {}", change.target_path.display()))?;
        let record = parsed
            .scalar_records()
            .filter(|record| {
                record.status == ParseStatus::Scalar
                    && record.normalized_setting_id.as_deref()
                        == Some(change.official_setting.as_str())
            })
            .find(|record| record.line_number == change.line_number)
            .ok_or_else(|| anyhow!("verified setting missing for {}", change.setting_id))?;
        if record.raw_value.as_deref() != Some(change.proposed_value.as_str()) {
            return Err(anyhow!(
                "expected {}={}, got {:?}",
                change.setting_id,
                change.proposed_value,
                record.raw_value
            ));
        }
        verified.push(change.setting_id.clone());
    }
    Ok(verified)
}

fn recover_failure(
    plan: &SafeBatchWritePlan,
    backups: Vec<SafeBatchBackupRecord>,
    failure: String,
    force_restore_failure: bool,
) -> SafeBatchWriteReport {
    let mut failures = vec![failure];
    let mut recovery_succeeded = true;
    let mut restore_verification_succeeded = true;
    for backup in &backups {
        if force_restore_failure {
            recovery_succeeded = false;
            restore_verification_succeeded = false;
            failures.push(format!(
                "forced restore failure for {}",
                backup.target_path.display()
            ));
            continue;
        }
        match fs::read(&backup.backup_path)
            .and_then(|bytes| fs::write(&backup.target_path, bytes).map(|_| ()))
        {
            Ok(()) => {}
            Err(error) => {
                recovery_succeeded = false;
                failures.push(format!(
                    "restore failed for {}: {error}",
                    backup.target_path.display()
                ));
                continue;
            }
        }
        match verify_restored_backup(plan, backup) {
            Ok(()) => {}
            Err(error) => {
                restore_verification_succeeded = false;
                failures.push(format!(
                    "restore verification failed for {}: {error}",
                    backup.target_path.display()
                ));
            }
        }
    }

    SafeBatchWriteReport {
        batch_id: plan.batch_id.clone(),
        status: if recovery_succeeded && restore_verification_succeeded {
            SafeBatchWriteStatus::RecoveredFailure
        } else {
            SafeBatchWriteStatus::UnrecoveredFailure
        },
        backups,
        verified_changes: Vec::new(),
        failures,
        recovery_attempted: true,
        recovery_succeeded,
        restore_verification_succeeded,
        hyprland_reload_attempted: false,
        mutating_hyprctl_used: false,
        runtime_mutated: false,
    }
}

fn verify_restored_backup(plan: &SafeBatchWritePlan, backup: &SafeBatchBackupRecord) -> Result<()> {
    let target = fs::read(&backup.target_path)?;
    let backup_bytes = fs::read(&backup.backup_path)?;
    if target != backup_bytes {
        return Err(anyhow!("restored bytes do not match backup"));
    }
    for change in plan
        .eligible_changes
        .iter()
        .filter(|change| change.target_path == backup.target_path)
    {
        ensure_source_line_matches(
            &backup.target_path,
            change.line_number,
            &change.official_setting,
            &change.old_value,
        )?;
    }
    Ok(())
}

fn failure_without_recovery(
    batch_id: &str,
    backups: Vec<SafeBatchBackupRecord>,
    failure: String,
) -> SafeBatchWriteReport {
    SafeBatchWriteReport {
        batch_id: batch_id.to_string(),
        status: SafeBatchWriteStatus::Blocked,
        backups,
        verified_changes: Vec::new(),
        failures: vec![failure],
        recovery_attempted: false,
        recovery_succeeded: false,
        restore_verification_succeeded: false,
        hyprland_reload_attempted: false,
        mutating_hyprctl_used: false,
        runtime_mutated: false,
    }
}

fn replace_value_preserving_key(line: &str, proposed_value: &str) -> Result<String> {
    let (before_comment, comment) = line
        .split_once('#')
        .map(|(before, comment)| (before, Some(comment)))
        .unwrap_or((line, None));
    let (key, _) = before_comment
        .split_once('=')
        .ok_or_else(|| anyhow!("source line has no scalar assignment"))?;
    let mut replaced = format!("{key}= {proposed_value}");
    if let Some(comment) = comment {
        replaced.push_str(" #");
        replaced.push_str(comment);
    }
    Ok(replaced)
}

fn atomic_write(target: &Path, bytes: &[u8]) -> Result<()> {
    let parent = target
        .parent()
        .ok_or_else(|| anyhow!("target path has no parent"))?;
    let temp_path = parent.join(format!(
        ".{}.safe-batch-{}.tmp",
        target
            .file_name()
            .and_then(|name| name.to_str())
            .unwrap_or("hyprland.conf"),
        unique_stamp()?
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

fn unique_stamp() -> Result<String> {
    let nanos = SystemTime::now().duration_since(UNIX_EPOCH)?.as_nanos();
    Ok(format!("{}-{nanos}", std::process::id()))
}
