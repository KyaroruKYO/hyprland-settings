//! Controlled structured-family write executor.
//!
//! This is real write code, wired internally for controlled targets only:
//! test-owned fixtures, copied config trees, and temporary config files. It
//! takes an accepted, unblocked staged apply plan plus a classified controlled
//! target, creates a byte-exact backup, writes the rendered structured-family
//! records, rereads the target through the parser/projection path, verifies
//! the intended records are present, and can restore the backup and verify
//! the restoration. Every step fails closed: missing approval, a rejected
//! target, a missing backup/restore/verification plan, or an unsafe staged
//! apply plan aborts before any byte is written.
//!
//! The active real Hyprland config is not a valid target. The target policy
//! rejects it, the approval model cannot approve it, and no code path here
//! runs commands, reloads the compositor, or mutates runtime.

use std::fs;
use std::path::{Path, PathBuf};

use crate::config_parser::parse_hyprland_config_text;
use crate::current_config::CurrentConfigSnapshot;
use crate::structured_family::{
    structured_family_projection, StructuredFamilyDraftRenderedRecordStagedApplyPlan,
    StructuredFamilyDraftRenderedRecordStagedApplyStatus, StructuredFamilyKind,
};
use crate::structured_family_write_target::{
    classify_structured_family_write_target, StructuredFamilyControlledWriteTarget,
    StructuredFamilyControlledWriteTargetKind, StructuredFamilyControlledWriteTargetRejection,
};

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum StructuredFamilyControlledWriteError {
    TargetRejected(StructuredFamilyControlledWriteTargetRejection),
    MissingApproval,
    ActiveRealConfigApprovalForbidden,
    MissingBackupPlan,
    MissingRestorePlan,
    MissingVerificationPlan,
    UnsafeStagedApplyPlan(&'static str),
    EmptyRenderedRecords,
    BackupPathOutsideControlledRoot,
    TargetReadFailed(String),
    BackupFailed(String),
    WriteFailed(String),
    RereadFailed(String),
    PostWriteVerificationFailed,
    RestoreFailed(String),
    PostRestoreVerificationFailed,
}

impl StructuredFamilyControlledWriteError {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::TargetRejected(_) => "TargetRejected",
            Self::MissingApproval => "MissingApproval",
            Self::ActiveRealConfigApprovalForbidden => "ActiveRealConfigApprovalForbidden",
            Self::MissingBackupPlan => "MissingBackupPlan",
            Self::MissingRestorePlan => "MissingRestorePlan",
            Self::MissingVerificationPlan => "MissingVerificationPlan",
            Self::UnsafeStagedApplyPlan(_) => "UnsafeStagedApplyPlan",
            Self::EmptyRenderedRecords => "EmptyRenderedRecords",
            Self::BackupPathOutsideControlledRoot => "BackupPathOutsideControlledRoot",
            Self::TargetReadFailed(_) => "TargetReadFailed",
            Self::BackupFailed(_) => "BackupFailed",
            Self::WriteFailed(_) => "WriteFailed",
            Self::RereadFailed(_) => "RereadFailed",
            Self::PostWriteVerificationFailed => "PostWriteVerificationFailed",
            Self::RestoreFailed(_) => "RestoreFailed",
            Self::PostRestoreVerificationFailed => "PostRestoreVerificationFailed",
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct StructuredFamilyControlledWriteApproval {
    pub controlled_write_approved: bool,
    /// Must always be false. The controlled executor refuses to run when this
    /// is true: active real config writes require a future, separate approval
    /// flow that does not exist yet.
    pub active_real_config_write_approved: bool,
    pub backup_acknowledged: bool,
    pub restore_acknowledged: bool,
    pub verification_acknowledged: bool,
}

pub fn approve_structured_family_controlled_write() -> StructuredFamilyControlledWriteApproval {
    StructuredFamilyControlledWriteApproval {
        controlled_write_approved: true,
        active_real_config_write_approved: false,
        backup_acknowledged: true,
        restore_acknowledged: true,
        verification_acknowledged: true,
    }
}

pub fn verify_structured_family_controlled_write_approval(
    approval: &StructuredFamilyControlledWriteApproval,
) -> Result<(), StructuredFamilyControlledWriteError> {
    if approval.active_real_config_write_approved {
        return Err(StructuredFamilyControlledWriteError::ActiveRealConfigApprovalForbidden);
    }
    if !approval.controlled_write_approved
        || !approval.backup_acknowledged
        || !approval.restore_acknowledged
        || !approval.verification_acknowledged
    {
        return Err(StructuredFamilyControlledWriteError::MissingApproval);
    }
    Ok(())
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct StructuredFamilyControlledWriteBackupPlan {
    pub backup_path: PathBuf,
    pub required: bool,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct StructuredFamilyControlledWriteRestorePlan {
    pub restore_required: bool,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct StructuredFamilyControlledWriteVerificationPlan {
    pub verify_post_write: bool,
    pub verify_post_restore: bool,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct StructuredFamilyControlledWritePlan {
    pub family: StructuredFamilyKind,
    pub target: StructuredFamilyControlledWriteTarget,
    pub rendered_records: Vec<String>,
    pub staged_apply_linked: bool,
    pub staged_apply_blocker_count: usize,
    pub backup_plan: Option<StructuredFamilyControlledWriteBackupPlan>,
    pub restore_plan: Option<StructuredFamilyControlledWriteRestorePlan>,
    pub verification_plan: Option<StructuredFamilyControlledWriteVerificationPlan>,
}

/// Build a controlled write plan from an accepted staged apply plan. Rejects
/// any staged apply plan that is blocked, unconfirmed, already executed, or
/// inconsistent with the requested family.
pub fn build_structured_family_controlled_write_plan(
    staged_apply: &StructuredFamilyDraftRenderedRecordStagedApplyPlan,
    target: StructuredFamilyControlledWriteTarget,
    rendered_records: Vec<String>,
) -> Result<StructuredFamilyControlledWritePlan, StructuredFamilyControlledWriteError> {
    if !staged_apply.accepted_confirmation_linked {
        return Err(StructuredFamilyControlledWriteError::UnsafeStagedApplyPlan(
            "staged apply plan is not linked to an accepted confirmation",
        ));
    }
    if !staged_apply.blockers.is_empty() {
        return Err(StructuredFamilyControlledWriteError::UnsafeStagedApplyPlan(
            "staged apply plan carries blockers",
        ));
    }
    if staged_apply.staged_apply_status
        != StructuredFamilyDraftRenderedRecordStagedApplyStatus::PlanReady
    {
        return Err(StructuredFamilyControlledWriteError::UnsafeStagedApplyPlan(
            "staged apply plan is not in the plan-ready state",
        ));
    }
    if staged_apply.staged_apply_executed
        || staged_apply.real_config_touched
        || staged_apply.runtime_mutated
        || staged_apply.hyprctl_reload_run
    {
        return Err(StructuredFamilyControlledWriteError::UnsafeStagedApplyPlan(
            "staged apply plan reports prior execution or mutation",
        ));
    }
    if rendered_records.is_empty() {
        return Err(StructuredFamilyControlledWriteError::EmptyRenderedRecords);
    }

    let backup_path = controlled_backup_path(&target.path);
    Ok(StructuredFamilyControlledWritePlan {
        family: staged_apply.family,
        target,
        rendered_records,
        staged_apply_linked: true,
        staged_apply_blocker_count: staged_apply.blockers.len(),
        backup_plan: Some(StructuredFamilyControlledWriteBackupPlan {
            backup_path,
            required: true,
        }),
        restore_plan: Some(StructuredFamilyControlledWriteRestorePlan {
            restore_required: true,
        }),
        verification_plan: Some(StructuredFamilyControlledWriteVerificationPlan {
            verify_post_write: true,
            verify_post_restore: true,
        }),
    })
}

/// Write bytes to a sibling temp file, then rename over the target. A crash
/// mid-write leaves the original file intact instead of a truncated target.
pub fn atomic_controlled_write(path: &Path, bytes: &[u8]) -> std::io::Result<()> {
    let mut temp_name = path
        .file_name()
        .map(|name| name.to_string_lossy().into_owned())
        .unwrap_or_else(|| "controlled-target".to_string());
    temp_name.push_str(".controlled-write-tmp");
    let temp_path = path.with_file_name(temp_name);
    fs::write(&temp_path, bytes)?;
    match fs::rename(&temp_path, path) {
        Ok(()) => Ok(()),
        Err(error) => {
            let _ = fs::remove_file(&temp_path);
            Err(error)
        }
    }
}

fn controlled_backup_path(target_path: &Path) -> PathBuf {
    let mut file_name = target_path
        .file_name()
        .map(|name| name.to_string_lossy().into_owned())
        .unwrap_or_else(|| "controlled-target".to_string());
    file_name.push_str(".controlled-write-backup");
    target_path.with_file_name(file_name)
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct StructuredFamilyControlledWriteBackup {
    pub backup_path: PathBuf,
    pub created: bool,
    pub byte_exact: bool,
    pub original_byte_count: usize,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct StructuredFamilyControlledWriteVerification {
    pub reread_completed: bool,
    pub intended_records_present: bool,
    pub reread_record_count: usize,
    pub intended_record_count: usize,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct StructuredFamilyControlledWriteRollback {
    pub restore_executed: bool,
    pub restored_byte_exact: bool,
    pub post_restore_verification: StructuredFamilyControlledWriteVerification,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct StructuredFamilyControlledWriteReceipt {
    pub family: StructuredFamilyKind,
    pub target_kind: StructuredFamilyControlledWriteTargetKind,
    pub target_path: PathBuf,
    pub controlled_root: PathBuf,
    pub executed: bool,
    pub backup: StructuredFamilyControlledWriteBackup,
    pub written: bool,
    pub post_write_verification: StructuredFamilyControlledWriteVerification,
    pub rollback: Option<StructuredFamilyControlledWriteRollback>,
    pub active_real_config_touched: bool,
    pub hyprctl_reload_run: bool,
    pub runtime_mutated: bool,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct StructuredFamilyControlledWriteAuditRecord {
    pub family: StructuredFamilyKind,
    pub target_kind: StructuredFamilyControlledWriteTargetKind,
    pub target_path: PathBuf,
    pub executed: bool,
    pub backup_created: bool,
    pub post_write_verified: bool,
    pub restore_executed: bool,
    pub post_restore_verified: bool,
    pub active_real_config_touched: bool,
    pub hyprctl_reload_run: bool,
    pub runtime_mutated: bool,
    pub summary: String,
}

pub fn family_records_in_text(
    label: &Path,
    text: &str,
    family: StructuredFamilyKind,
) -> Vec<(usize, String)> {
    let parsed = parse_hyprland_config_text(label, text);
    let snapshot = CurrentConfigSnapshot::from_parsed(parsed);
    let projection = structured_family_projection(&snapshot, family);
    projection
        .records
        .iter()
        .map(|record| (record.line_number, record.raw_line.clone()))
        .collect()
}

/// Replace the family's records in `original` with `rendered_records`,
/// preserving every non-family line (comments, unknown syntax, other
/// families, scalars). If the family has no records yet, the rendered records
/// are appended at the end.
pub fn apply_rendered_family_records(
    label: &Path,
    original: &str,
    family: StructuredFamilyKind,
    rendered_records: &[String],
) -> String {
    let family_lines: Vec<usize> = family_records_in_text(label, original, family)
        .into_iter()
        .map(|(line_number, _)| line_number)
        .collect();
    let insert_at = family_lines.first().copied();

    let mut output: Vec<String> = Vec::new();
    for (index, line) in original.lines().enumerate() {
        let line_number = index + 1;
        if Some(line_number) == insert_at {
            output.extend(rendered_records.iter().cloned());
        }
        if family_lines.contains(&line_number) {
            continue;
        }
        output.push(line.to_string());
    }
    if insert_at.is_none() {
        output.extend(rendered_records.iter().cloned());
    }

    let mut text = output.join("\n");
    text.push('\n');
    text
}

pub fn verify_family_records_on_disk(
    path: &Path,
    family: StructuredFamilyKind,
    intended_records: &[String],
) -> Result<StructuredFamilyControlledWriteVerification, StructuredFamilyControlledWriteError> {
    let contents = fs::read_to_string(path)
        .map_err(|error| StructuredFamilyControlledWriteError::RereadFailed(error.to_string()))?;
    let reread: Vec<String> = family_records_in_text(path, &contents, family)
        .into_iter()
        .map(|(_, raw_line)| raw_line.trim().to_string())
        .collect();
    let intended: Vec<String> = intended_records
        .iter()
        .map(|record| record.trim().to_string())
        .collect();
    Ok(StructuredFamilyControlledWriteVerification {
        reread_completed: true,
        intended_records_present: reread == intended,
        reread_record_count: reread.len(),
        intended_record_count: intended.len(),
    })
}

fn required_plans(
    plan: &StructuredFamilyControlledWritePlan,
) -> Result<
    (
        &StructuredFamilyControlledWriteBackupPlan,
        &StructuredFamilyControlledWriteRestorePlan,
        &StructuredFamilyControlledWriteVerificationPlan,
    ),
    StructuredFamilyControlledWriteError,
> {
    let backup_plan = plan
        .backup_plan
        .as_ref()
        .filter(|backup_plan| backup_plan.required)
        .ok_or(StructuredFamilyControlledWriteError::MissingBackupPlan)?;
    let restore_plan = plan
        .restore_plan
        .as_ref()
        .filter(|restore_plan| restore_plan.restore_required)
        .ok_or(StructuredFamilyControlledWriteError::MissingRestorePlan)?;
    let verification_plan = plan
        .verification_plan
        .as_ref()
        .filter(|verification_plan| {
            verification_plan.verify_post_write && verification_plan.verify_post_restore
        })
        .ok_or(StructuredFamilyControlledWriteError::MissingVerificationPlan)?;
    Ok((backup_plan, restore_plan, verification_plan))
}

/// Execute a controlled structured-family write: approval check, target
/// policy check, backup, write, reread, verification. On post-write
/// verification failure the original bytes are restored before the error is
/// returned. Never touches the active real config, never runs commands,
/// never reloads the compositor, never mutates runtime.
pub fn execute_structured_family_controlled_write(
    plan: &StructuredFamilyControlledWritePlan,
    approval: &StructuredFamilyControlledWriteApproval,
) -> Result<StructuredFamilyControlledWriteReceipt, StructuredFamilyControlledWriteError> {
    verify_structured_family_controlled_write_approval(approval)?;

    let policy = classify_structured_family_write_target(&plan.target);
    if !policy.writable {
        let reason = policy
            .rejection_reasons
            .first()
            .copied()
            .unwrap_or(StructuredFamilyControlledWriteTargetRejection::UnknownTargetRejected);
        return Err(StructuredFamilyControlledWriteError::TargetRejected(reason));
    }

    if !plan.staged_apply_linked || plan.staged_apply_blocker_count > 0 {
        return Err(StructuredFamilyControlledWriteError::UnsafeStagedApplyPlan(
            "controlled write plan lost its staged apply linkage",
        ));
    }
    if plan.rendered_records.is_empty() {
        return Err(StructuredFamilyControlledWriteError::EmptyRenderedRecords);
    }

    let (backup_plan, _restore_plan, _verification_plan) = required_plans(plan)?;
    if !backup_plan
        .backup_path
        .starts_with(&plan.target.controlled_root)
    {
        return Err(StructuredFamilyControlledWriteError::BackupPathOutsideControlledRoot);
    }

    let original = fs::read(&plan.target.path).map_err(|error| {
        StructuredFamilyControlledWriteError::TargetReadFailed(error.to_string())
    })?;

    fs::write(&backup_plan.backup_path, &original)
        .map_err(|error| StructuredFamilyControlledWriteError::BackupFailed(error.to_string()))?;
    let backup_bytes = fs::read(&backup_plan.backup_path)
        .map_err(|error| StructuredFamilyControlledWriteError::BackupFailed(error.to_string()))?;
    if backup_bytes != original {
        return Err(StructuredFamilyControlledWriteError::BackupFailed(
            "backup bytes are not byte-exact".to_string(),
        ));
    }
    let backup = StructuredFamilyControlledWriteBackup {
        backup_path: backup_plan.backup_path.clone(),
        created: true,
        byte_exact: true,
        original_byte_count: original.len(),
    };

    let original_text = String::from_utf8_lossy(&original).into_owned();
    let new_text = apply_rendered_family_records(
        &plan.target.path,
        &original_text,
        plan.family,
        &plan.rendered_records,
    );
    atomic_controlled_write(&plan.target.path, new_text.as_bytes())
        .map_err(|error| StructuredFamilyControlledWriteError::WriteFailed(error.to_string()))?;

    let post_write_verification =
        verify_family_records_on_disk(&plan.target.path, plan.family, &plan.rendered_records)?;
    if !post_write_verification.intended_records_present {
        // Fail closed: put the original bytes back before reporting failure.
        atomic_controlled_write(&plan.target.path, &original).map_err(|error| {
            StructuredFamilyControlledWriteError::RestoreFailed(error.to_string())
        })?;
        return Err(StructuredFamilyControlledWriteError::PostWriteVerificationFailed);
    }

    Ok(StructuredFamilyControlledWriteReceipt {
        family: plan.family,
        target_kind: policy.resolved_kind,
        target_path: plan.target.path.clone(),
        controlled_root: plan.target.controlled_root.clone(),
        executed: true,
        backup,
        written: true,
        post_write_verification,
        rollback: None,
        active_real_config_touched: false,
        hyprctl_reload_run: false,
        runtime_mutated: false,
    })
}

/// Restore the controlled target from its backup and verify the restoration
/// byte-exactly and through the parser/projection reread path.
pub fn restore_structured_family_controlled_write(
    plan: &StructuredFamilyControlledWritePlan,
    receipt: &StructuredFamilyControlledWriteReceipt,
) -> Result<StructuredFamilyControlledWriteRollback, StructuredFamilyControlledWriteError> {
    let (_backup_plan, restore_plan, _verification_plan) = required_plans(plan)?;
    if !restore_plan.restore_required {
        return Err(StructuredFamilyControlledWriteError::MissingRestorePlan);
    }

    let backup_bytes = fs::read(&receipt.backup.backup_path)
        .map_err(|error| StructuredFamilyControlledWriteError::RestoreFailed(error.to_string()))?;
    atomic_controlled_write(&receipt.target_path, &backup_bytes)
        .map_err(|error| StructuredFamilyControlledWriteError::RestoreFailed(error.to_string()))?;
    let restored_bytes = fs::read(&receipt.target_path)
        .map_err(|error| StructuredFamilyControlledWriteError::RestoreFailed(error.to_string()))?;
    let restored_byte_exact = restored_bytes == backup_bytes;

    let backup_text = String::from_utf8_lossy(&backup_bytes).into_owned();
    let original_records: Vec<String> =
        family_records_in_text(&receipt.target_path, &backup_text, plan.family)
            .into_iter()
            .map(|(_, raw_line)| raw_line)
            .collect();
    let post_restore_verification =
        verify_family_records_on_disk(&receipt.target_path, plan.family, &original_records)?;

    if !restored_byte_exact || !post_restore_verification.intended_records_present {
        return Err(StructuredFamilyControlledWriteError::PostRestoreVerificationFailed);
    }

    Ok(StructuredFamilyControlledWriteRollback {
        restore_executed: true,
        restored_byte_exact,
        post_restore_verification,
    })
}

/// Full controlled round trip: write, verify, restore, verify restoration.
/// The returned receipt carries the rollback proof.
pub fn execute_structured_family_controlled_write_round_trip(
    plan: &StructuredFamilyControlledWritePlan,
    approval: &StructuredFamilyControlledWriteApproval,
) -> Result<StructuredFamilyControlledWriteReceipt, StructuredFamilyControlledWriteError> {
    let mut receipt = execute_structured_family_controlled_write(plan, approval)?;
    let rollback = restore_structured_family_controlled_write(plan, &receipt)?;
    receipt.rollback = Some(rollback);
    Ok(receipt)
}

pub fn structured_family_controlled_write_audit_record(
    receipt: &StructuredFamilyControlledWriteReceipt,
) -> StructuredFamilyControlledWriteAuditRecord {
    let restore_executed = receipt
        .rollback
        .as_ref()
        .map(|rollback| rollback.restore_executed)
        .unwrap_or(false);
    let post_restore_verified = receipt
        .rollback
        .as_ref()
        .map(|rollback| {
            rollback.restored_byte_exact
                && rollback.post_restore_verification.intended_records_present
        })
        .unwrap_or(false);
    StructuredFamilyControlledWriteAuditRecord {
        family: receipt.family,
        target_kind: receipt.target_kind,
        target_path: receipt.target_path.clone(),
        executed: receipt.executed,
        backup_created: receipt.backup.created,
        post_write_verified: receipt.post_write_verification.intended_records_present,
        restore_executed,
        post_restore_verified,
        active_real_config_touched: false,
        hyprctl_reload_run: false,
        runtime_mutated: false,
        summary: format!(
            "controlled structured-family write for {} against {} target {}: executed {}, backup {}, post-write verified {}, restore executed {}, post-restore verified {}; active real config untouched, no reload, no runtime mutation",
            receipt.family.family_id(),
            receipt.target_kind.as_str(),
            receipt.target_path.display(),
            receipt.executed,
            receipt.backup.created,
            receipt.post_write_verification.intended_records_present,
            restore_executed,
            post_restore_verified,
        ),
    }
}
