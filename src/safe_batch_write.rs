use std::collections::{BTreeMap, BTreeSet};
use std::fs;
use std::path::{Path, PathBuf};
use std::time::{SystemTime, UNIX_EPOCH};

use anyhow::{anyhow, Context, Result};

use crate::config_backup::{BackupManager, ConfigBackup};
use crate::config_graph::{ConfigGraphFile, ConfigGraphSummary, ConfigManagementHintKind};
use crate::config_parser::{parse_hyprland_config_file, parse_hyprland_config_text, ParseStatus};
use crate::current_config::{CurrentConfigSnapshot, CurrentValueSourceStatus};
use crate::durable_fs::{hardened_atomic_replace, DurableWriteReceipt, FilePrecondition};
use crate::pending_change::{stage_pending_change, PendingChangeValidation};
use crate::production_backup_contract::{backup_path_policy_for_target, BackupPathPolicy};
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
                "Blocked: this setting uses Hyprland's default value, and this config layout is not safe for automatic insertion."
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
pub struct SafeBatchInsertionChange {
    pub setting_id: String,
    pub official_setting: String,
    pub config_key: String,
    pub target_path: PathBuf,
    pub proposed_value: String,
    pub insertion_line: String,
    pub review_copy: String,
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
    pub precondition: Option<FilePrecondition>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SafeBatchWritePlan {
    pub batch_id: String,
    pub pending_changes: Vec<SafeBatchChangeRequest>,
    pub eligible_changes: Vec<SafeBatchEligibleChange>,
    pub insertion_changes: Vec<SafeBatchInsertionChange>,
    pub blocked_changes: Vec<SafeBatchBlockedChange>,
    pub target_files: Vec<SafeBatchTargetFilePlan>,
    pub can_execute: bool,
    pub cannot_execute_reasons: Vec<String>,
    pub safe_writable_rows_len: usize,
    pub source_graph_root: PathBuf,
    pub source_graph_fingerprint: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SafeBatchExecutionOptions {
    pub backup_timestamp: String,
    pub fail_after_writing_target: Option<PathBuf>,
    pub force_backup_verification_failure_for: Option<PathBuf>,
    pub force_verification_failure_for: Option<String>,
    pub force_restore_failure: bool,
    /// `Some` for production callers. Fixture callers leave this `None`, in
    /// which case a test-owned sibling backup directory is used.
    pub backup_root: Option<PathBuf>,
}

impl Default for SafeBatchExecutionOptions {
    fn default() -> Self {
        Self {
            backup_timestamp: unique_stamp().unwrap_or_else(|_| "safe-batch".to_string()),
            fail_after_writing_target: None,
            force_backup_verification_failure_for: None,
            force_verification_failure_for: None,
            force_restore_failure: false,
            backup_root: None,
        }
    }
}

impl SafeBatchExecutionOptions {
    pub fn production() -> Result<Self> {
        Ok(Self {
            backup_root: Some(BackupManager::default_user_backup_root()?),
            ..Self::default()
        })
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SafeBatchBackupRecord {
    pub target_path: PathBuf,
    pub backup_path: PathBuf,
    pub bytes_equal: bool,
    pub byte_len: usize,
    pub mode: u32,
    pub sha256: String,
    pub verified_backup: ConfigBackup,
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
    pub durable_receipts: Vec<DurableWriteReceipt>,
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
    let mut insertion_changes = Vec::new();
    let mut blocked_changes = Vec::new();

    for request in &pending_changes {
        match classify_safe_batch_change(known_setting_ids, current_config, config_graph, request) {
            Ok(SafeBatchPlannedChange::ExistingScalar(change)) => eligible_changes.push(change),
            Ok(SafeBatchPlannedChange::MissingDefaultInsertion(change)) => {
                insertion_changes.push(change)
            }
            Err(blocked) => blocked_changes.push(blocked),
        }
    }

    let mut file_counts: BTreeMap<PathBuf, usize> = BTreeMap::new();
    for change in &eligible_changes {
        *file_counts.entry(change.target_path.clone()).or_default() += 1;
    }
    for change in &insertion_changes {
        *file_counts.entry(change.target_path.clone()).or_default() += 1;
    }
    let target_files = file_counts
        .into_iter()
        .map(|(target_path, change_count)| SafeBatchTargetFilePlan {
            backup_policy: backup_path_policy_for_target(&target_path, backup_timestamp.as_ref()),
            precondition: current_config.file_precondition(&target_path).cloned(),
            target_path,
            change_count,
        })
        .collect::<Vec<_>>();

    let mut cannot_execute_reasons = Vec::new();
    if pending_changes.is_empty() {
        cannot_execute_reasons.push("no pending changes were selected".to_string());
    }
    if eligible_changes.is_empty() && insertion_changes.is_empty() {
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
    if target_files.len() > 1 {
        cannot_execute_reasons.push(
            "multi-file batch rejected before writing: cross-file renames are not crash-atomic; save each file separately after explicit review".to_string(),
        );
    }
    if target_files
        .iter()
        .any(|target| target.precondition.is_none())
    {
        cannot_execute_reasons
            .push("one or more targets lack an immutable startup file precondition".to_string());
    }
    let mut selected = BTreeSet::new();
    if pending_changes
        .iter()
        .any(|change| !selected.insert(change.setting_id.as_str()))
    {
        cannot_execute_reasons.push(
            "the batch contains more than one pending value for the same setting".to_string(),
        );
    }

    let observed_graph_fingerprint = crate::source_aware_current_config::graph_fingerprint(
        config_graph,
        &current_config.file_preconditions,
    );
    if let Some(expected) = current_config.source_graph_fingerprint.as_deref() {
        if observed_graph_fingerprint.as_deref() != Some(expected) {
            cannot_execute_reasons.push(
                "source graph differs from the snapshot used to prepare pending changes"
                    .to_string(),
            );
        }
    }
    let source_graph_fingerprint = current_config
        .source_graph_fingerprint
        .clone()
        .or(observed_graph_fingerprint);
    if source_graph_fingerprint.is_none() {
        cannot_execute_reasons.push("source graph precondition is unavailable".to_string());
    }

    SafeBatchWritePlan {
        batch_id,
        pending_changes,
        eligible_changes,
        insertion_changes,
        blocked_changes,
        target_files,
        can_execute: cannot_execute_reasons.is_empty(),
        cannot_execute_reasons,
        safe_writable_rows_len: SAFE_WRITABLE_ROWS.len(),
        source_graph_root: config_graph.root_path.clone(),
        source_graph_fingerprint,
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
            durable_receipts: Vec::new(),
        };
    }

    if plan.target_files.len() != 1 {
        return failure_without_recovery(
            &plan.batch_id,
            Vec::new(),
            "multi-file batch rejected before writing".to_string(),
        );
    }
    let Some(expected_graph) = plan.source_graph_fingerprint.as_deref() else {
        return failure_without_recovery(
            &plan.batch_id,
            Vec::new(),
            "source graph precondition is unavailable".to_string(),
        );
    };
    if crate::source_aware_current_config::current_source_graph_fingerprint(&plan.source_graph_root)
        .as_deref()
        != Some(expected_graph)
    {
        return failure_without_recovery(
            &plan.batch_id,
            Vec::new(),
            "SourceGraphChanged: config sources or bytes changed after review".to_string(),
        );
    }

    let target = &plan.target_files[0];
    let Some(precondition) = target.precondition.as_ref() else {
        return failure_without_recovery(
            &plan.batch_id,
            Vec::new(),
            "target drift precondition is unavailable".to_string(),
        );
    };
    let changes = changes_for_target(&plan.eligible_changes, &target.target_path);
    let insertions = insertions_for_target(&plan.insertion_changes, &target.target_path);
    let staged_bytes = match stage_target_file(precondition, &changes, &insertions) {
        Ok(bytes) => bytes,
        Err(error) => {
            return failure_without_recovery(
                &plan.batch_id,
                Vec::new(),
                format!("batch staging failed: {error}"),
            )
        }
    };
    if let Err(error) = verify_values_in_text(
        &target.target_path,
        &staged_bytes,
        &changes,
        &insertions,
        None,
    ) {
        return failure_without_recovery(
            &plan.batch_id,
            Vec::new(),
            format!("staged parse/reread verification failed: {error}"),
        );
    }

    let backup = match create_verified_backup(target, options) {
        Ok(record) => record,
        Err(error) => {
            return failure_without_recovery(
                &plan.batch_id,
                Vec::new(),
                format!(
                    "backup failed for {}: {error}",
                    target.target_path.display()
                ),
            )
        }
    };
    let backups = vec![backup];
    if crate::source_aware_current_config::current_source_graph_fingerprint(&plan.source_graph_root)
        .as_deref()
        != Some(expected_graph)
    {
        return failure_without_recovery(
            &plan.batch_id,
            backups,
            "SourceGraphChanged: config sources or bytes changed immediately before commit"
                .to_string(),
        );
    }
    let durable_receipt = match hardened_atomic_replace(precondition, &staged_bytes) {
        Ok(receipt) => receipt,
        Err(error) => {
            return failure_without_recovery(
                &plan.batch_id,
                backups,
                format!("write failed before durable completion: {error}"),
            )
        }
    };

    if options.fail_after_writing_target.as_ref() == Some(&target.target_path) {
        return recover_failure(
            plan,
            backups,
            vec![durable_receipt],
            &staged_bytes,
            "injected failure after target replacement".to_string(),
            options.force_restore_failure,
        );
    }

    match verify_written_values(
        &plan.eligible_changes,
        &plan.insertion_changes,
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
            durable_receipts: vec![durable_receipt],
        },
        Err(error) => recover_failure(
            plan,
            backups,
            vec![durable_receipt],
            &staged_bytes,
            format!("verification failed: {error}"),
            options.force_restore_failure,
        ),
    }
}

enum SafeBatchPlannedChange {
    ExistingScalar(SafeBatchEligibleChange),
    MissingDefaultInsertion(SafeBatchInsertionChange),
}

fn classify_safe_batch_change(
    known_setting_ids: &BTreeSet<String>,
    current_config: &CurrentConfigSnapshot,
    config_graph: &ConfigGraphSummary,
    request: &SafeBatchChangeRequest,
) -> Result<SafeBatchPlannedChange, SafeBatchBlockedChange> {
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
        if current_value.status == CurrentValueSourceStatus::NotConfigured {
            return classify_missing_default_insertion(config_graph, request, official_setting);
        }
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

    Ok(SafeBatchPlannedChange::ExistingScalar(
        SafeBatchEligibleChange {
            setting_id: setting_id.to_string(),
            official_setting: official_setting.to_string(),
            config_key: config_key_from_official_setting(official_setting),
            target_path,
            line_number,
            old_value,
            proposed_value: pending.proposed_value,
            original_raw_line,
        },
    ))
}

fn classify_missing_default_insertion(
    config_graph: &ConfigGraphSummary,
    request: &SafeBatchChangeRequest,
    official_setting: &str,
) -> Result<SafeBatchPlannedChange, SafeBatchBlockedChange> {
    let target_path = config_graph.root_path.clone();
    let Some(file) = graph_file_for(config_graph, &target_path) else {
        return blocked(
            request,
            SafeBatchEligibility::BlockedMissingLine,
            "missing/default insertion requires an explicit reviewed root config file",
        );
    };
    if config_graph.multi_file || file.source_depth != 0 {
        return blocked(
            request,
            SafeBatchEligibility::BlockedMissingLine,
            "missing/default insertion is limited to a single explicit root config file",
        );
    }
    if file.is_symlink
        || file.symlink_target.is_some()
        || has_hint(file, ConfigManagementHintKind::SymlinkManaged)
    {
        return blocked(
            request,
            SafeBatchEligibility::BlockedSymlinkManaged,
            "missing/default insertion target is symlink-managed",
        );
    }
    if has_hint(file, ConfigManagementHintKind::GeneratedFile) {
        return blocked(
            request,
            SafeBatchEligibility::BlockedGeneratedFile,
            "missing/default insertion target is generated",
        );
    }
    if has_hint(file, ConfigManagementHintKind::ScriptManaged)
        || has_hint(file, ConfigManagementHintKind::ScriptReferenced)
    {
        return blocked(
            request,
            SafeBatchEligibility::BlockedScriptManaged,
            "missing/default insertion target is script-managed or script-referenced",
        );
    }
    if !file.readable {
        return blocked(
            request,
            SafeBatchEligibility::BlockedUnknownTarget,
            "missing/default insertion target is not readable",
        );
    }
    let metadata = fs::symlink_metadata(&target_path).map_err(|error| {
        blocked_value(
            request,
            SafeBatchEligibility::BlockedMissingLine,
            format!("missing/default insertion target cannot be inspected: {error}"),
        )
    })?;
    if !metadata.file_type().is_file() || metadata.file_type().is_symlink() {
        return blocked(
            request,
            SafeBatchEligibility::BlockedMissingLine,
            "missing/default insertion target must be an existing normal file",
        );
    }
    if metadata.permissions().readonly() {
        return blocked(
            request,
            SafeBatchEligibility::BlockedUnknownTarget,
            "missing/default insertion target is not writable",
        );
    }
    ensure_setting_absent_from_file(&target_path, official_setting).map_err(|error| {
        blocked_value(
            request,
            SafeBatchEligibility::BlockedDuplicateConflict,
            error.to_string(),
        )
    })?;
    let config_key = config_key_from_official_setting(official_setting);
    let insertion_line = format!("{config_key} = {}", request.proposed_value.trim());

    Ok(SafeBatchPlannedChange::MissingDefaultInsertion(
        SafeBatchInsertionChange {
            setting_id: request.setting_id.clone(),
            official_setting: official_setting.to_string(),
            config_key,
            target_path,
            proposed_value: request.proposed_value.clone(),
            insertion_line,
            review_copy: "Apply will insert this missing/default setting into the reviewed root config file after creating and verifying a backup.".to_string(),
        },
    ))
}

fn blocked<T>(
    request: &SafeBatchChangeRequest,
    reason: SafeBatchEligibility,
    evidence: impl Into<String>,
) -> Result<T, SafeBatchBlockedChange> {
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
    target: &SafeBatchTargetFilePlan,
    options: &SafeBatchExecutionOptions,
) -> Result<SafeBatchBackupRecord> {
    let precondition = target
        .precondition
        .as_ref()
        .ok_or_else(|| anyhow!("missing target precondition"))?;
    let backup_root = options.backup_root.clone().unwrap_or_else(|| {
        target
            .target_path
            .parent()
            .unwrap_or_else(|| Path::new("."))
            .join(".hyprland-settings-test-backups")
    });
    let backup = BackupManager::new(backup_root).create_backup_from_precondition(precondition)?;
    let forced = options
        .force_backup_verification_failure_for
        .as_ref()
        .map(|path| path == &target.target_path)
        .unwrap_or(false);
    if !backup.backup_precondition.bytes.eq(&precondition.bytes) || forced {
        return Err(anyhow!("backup byte equality verification failed"));
    }
    Ok(SafeBatchBackupRecord {
        target_path: target.target_path.clone(),
        backup_path: backup.backup_path.clone(),
        bytes_equal: true,
        byte_len: precondition.bytes.len(),
        mode: backup.backup_precondition.metadata.mode,
        sha256: backup.backup_precondition.sha256.clone(),
        verified_backup: backup,
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

fn insertions_for_target<'a>(
    insertions: &'a [SafeBatchInsertionChange],
    target_path: &Path,
) -> Vec<&'a SafeBatchInsertionChange> {
    insertions
        .iter()
        .filter(|change| change.target_path == target_path)
        .collect::<Vec<_>>()
}

fn stage_target_file(
    precondition: &FilePrecondition,
    changes: &[&SafeBatchEligibleChange],
    insertions: &[&SafeBatchInsertionChange],
) -> Result<Vec<u8>> {
    let target_path = &precondition.requested_path;
    let original = std::str::from_utf8(&precondition.bytes)
        .with_context(|| format!("{} is not valid UTF-8", target_path.display()))?;
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
        if line != &change.original_raw_line {
            return Err(anyhow!(
                "source line bytes changed for {}",
                change.setting_id
            ));
        }
        lines[index] = replace_value_preserving_key(line, &change.proposed_value)?;
    }
    let original_parsed = parse_hyprland_config_text(target_path, original);
    for insertion in insertions {
        let count = original_parsed
            .scalar_records()
            .filter(|record| {
                record.normalized_setting_id.as_deref() == Some(insertion.official_setting.as_str())
            })
            .count();
        if count != 0 {
            return Err(anyhow!(
                "ExpectedSettingNowPresent: {} appeared after review",
                insertion.setting_id
            ));
        }
    }
    let mut updated = lines.join("\n");
    if had_trailing_newline || !updated.is_empty() {
        updated.push('\n');
    }
    if !insertions.is_empty() {
        updated.push('\n');
        updated.push_str("# Added by Hyprland Settings safe-batch missing/default insertion\n");
        for insertion in insertions {
            updated.push_str(&insertion.insertion_line);
            updated.push('\n');
        }
    }
    Ok(updated.into_bytes())
}

fn verify_values_in_text(
    target_path: &Path,
    bytes: &[u8],
    changes: &[&SafeBatchEligibleChange],
    insertions: &[&SafeBatchInsertionChange],
    force_verification_failure_for: Option<&str>,
) -> Result<Vec<String>> {
    let text = std::str::from_utf8(bytes).context("staged config is not valid UTF-8")?;
    let parsed = parse_hyprland_config_text(target_path, text);
    let mut verified = Vec::new();
    for change in changes {
        if force_verification_failure_for == Some(change.setting_id.as_str()) {
            return Err(anyhow!(
                "forced verification failure for {}",
                change.setting_id
            ));
        }
        let record = parsed
            .scalar_records()
            .find(|record| {
                record.normalized_setting_id.as_deref() == Some(change.official_setting.as_str())
                    && record.line_number == change.line_number
            })
            .ok_or_else(|| anyhow!("verified setting missing for {}", change.setting_id))?;
        if record.raw_value.as_deref() != Some(change.proposed_value.as_str()) {
            return Err(anyhow!("staged value mismatch for {}", change.setting_id));
        }
        verified.push(change.setting_id.clone());
    }
    for insertion in insertions {
        if force_verification_failure_for == Some(insertion.setting_id.as_str()) {
            return Err(anyhow!(
                "forced verification failure for {}",
                insertion.setting_id
            ));
        }
        let matches = parsed
            .scalar_records()
            .filter(|record| {
                record.normalized_setting_id.as_deref() == Some(insertion.official_setting.as_str())
            })
            .collect::<Vec<_>>();
        if matches.len() != 1
            || matches[0].raw_value.as_deref() != Some(insertion.proposed_value.as_str())
        {
            return Err(anyhow!(
                "staged insertion mismatch for {}",
                insertion.setting_id
            ));
        }
        verified.push(insertion.setting_id.clone());
    }
    Ok(verified)
}

fn verify_written_values(
    changes: &[SafeBatchEligibleChange],
    insertions: &[SafeBatchInsertionChange],
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
    for insertion in insertions {
        if force_verification_failure_for == Some(insertion.setting_id.as_str()) {
            return Err(anyhow!(
                "forced verification failure for {}",
                insertion.setting_id
            ));
        }
        let parsed = parse_hyprland_config_file(&insertion.target_path)
            .with_context(|| format!("failed to reread {}", insertion.target_path.display()))?;
        let matches = parsed
            .scalar_records()
            .filter(|record| {
                record.status == ParseStatus::Scalar
                    && record.normalized_setting_id.as_deref()
                        == Some(insertion.official_setting.as_str())
            })
            .collect::<Vec<_>>();
        if matches.len() != 1 {
            return Err(anyhow!(
                "expected one inserted setting for {}, got {}",
                insertion.setting_id,
                matches.len()
            ));
        }
        let record = matches[0];
        if record.raw_value.as_deref() != Some(insertion.proposed_value.as_str()) {
            return Err(anyhow!(
                "expected inserted {}={}, got {:?}",
                insertion.setting_id,
                insertion.proposed_value,
                record.raw_value
            ));
        }
        if record.raw_line.trim() != insertion.insertion_line {
            return Err(anyhow!(
                "inserted line did not match review line for {}",
                insertion.setting_id
            ));
        }
        verified.push(insertion.setting_id.clone());
    }
    Ok(verified)
}

fn recover_failure(
    plan: &SafeBatchWritePlan,
    backups: Vec<SafeBatchBackupRecord>,
    durable_receipts: Vec<DurableWriteReceipt>,
    expected_committed_bytes: &[u8],
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
        let manager = BackupManager::new(
            backup
                .backup_path
                .parent()
                .unwrap_or_else(|| Path::new(".")),
        );
        match manager.rollback(&backup.verified_backup, expected_committed_bytes) {
            Ok(_) => {}
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
        durable_receipts,
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
    for insertion in plan
        .insertion_changes
        .iter()
        .filter(|change| change.target_path == backup.target_path)
    {
        ensure_setting_absent_from_file(&backup.target_path, &insertion.official_setting)?;
    }
    Ok(())
}

fn ensure_setting_absent_from_file(target_path: &Path, official_setting: &str) -> Result<()> {
    let parsed = parse_hyprland_config_file(target_path)
        .with_context(|| format!("failed to read {}", target_path.display()))?;
    let count = parsed
        .scalar_records()
        .filter(|record| record.normalized_setting_id.as_deref() == Some(official_setting))
        .count();
    if count > 0 {
        return Err(anyhow!(
            "setting is already present in target file; duplicate resolution is required"
        ));
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
        durable_receipts: Vec::new(),
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

fn unique_stamp() -> Result<String> {
    let nanos = SystemTime::now().duration_since(UNIX_EPOCH)?.as_nanos();
    Ok(format!("{}-{nanos}", std::process::id()))
}

impl crate::runtime_preview_ui_projection::DurableSaveReceipt for SafeBatchWriteReport {
    fn durable_save_succeeded(&self) -> bool {
        self.status == SafeBatchWriteStatus::Succeeded
            && self.failures.is_empty()
            && self.durable_receipts.len() == 1
            && !self.verified_changes.is_empty()
    }
}
