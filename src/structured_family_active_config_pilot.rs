//! First active real config write pilot for structured families.
//!
//! This is the only path in the app that may ever write the user's active
//! Hyprland config, and it is deliberately impossible to trigger accidentally:
//! it is unreachable from the UI, `main`, and the scalar write flow; it takes
//! an explicit approval object with a typed confirmation phrase; and it runs a
//! preflight of hard gates that all must pass immediately before the write.
//! Any failed gate aborts before a single byte is written.
//!
//! One gate is environmental: the compositor auto-reloads its config when the
//! file changes unless `misc:disable_autoreload` is set. Because this module
//! must never mutate runtime, the pilot demands externally collected evidence
//! that autoreload is disabled. This module itself never runs commands and
//! never queries the compositor.
//!
//! The pilot writes once, verifies through the parser/projection path, and
//! then restores the original bytes and verifies the restoration. Keeping the
//! pilot record in place is not supported: `restore_original_bytes` must be
//! true or the preflight fails.

use std::fs;
use std::path::{Path, PathBuf};

use crate::config_parser::parse_hyprland_config_text;
use crate::current_config::CurrentConfigSnapshot;
use crate::structured_family::{
    structured_family_projection, StructuredFamilyDraftRenderedRecordStagedApplyPlan,
    StructuredFamilyKind,
};
use crate::structured_family_controlled_write::{
    apply_rendered_family_records, approve_structured_family_controlled_write,
    atomic_controlled_write, build_structured_family_controlled_write_plan,
    execute_structured_family_controlled_write_round_trip, verify_family_records_on_disk,
    StructuredFamilyControlledWriteVerification,
};
use crate::structured_family_write_target::{
    structured_family_path_is_active_real_config, StructuredFamilyControlledWriteTarget,
    StructuredFamilyControlledWriteTargetKind,
};

pub const ACTIVE_CONFIG_PILOT_CONFIRMATION_PHRASE: &str =
    "I approve the first active Hyprland config write pilot";

/// Deterministic FNV-1a 64-bit content hash. An identity fingerprint for
/// receipts and drift detection, not a cryptographic digest.
pub fn active_config_pilot_content_hash(bytes: &[u8]) -> u64 {
    let mut hash: u64 = 0xcbf2_9ce4_8422_2325;
    for byte in bytes {
        hash ^= u64::from(*byte);
        hash = hash.wrapping_mul(0x0000_0100_0000_01b3);
    }
    hash
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum StructuredFamilyActiveConfigPilotGate {
    PilotApprovalPresent,
    TypedConfirmationMatches,
    TargetIdentityProven,
    TargetExists,
    MinimalReversibleChange,
    BackupPathOutsideActiveConfig,
    RehearsalProven,
    RehearsalMatchesCurrentContent,
    RollbackPlanPresent,
    PostWriteVerificationPlanned,
    PostRestoreVerificationPlanned,
    AutoreloadDisabledConfirmed,
    NoReloadPlanned,
    NoRuntimeMutationPlanned,
    NoAutomaticApplyPath,
}

impl StructuredFamilyActiveConfigPilotGate {
    pub fn as_str(self) -> &'static str {
        match self {
            Self::PilotApprovalPresent => "PilotApprovalPresent",
            Self::TypedConfirmationMatches => "TypedConfirmationMatches",
            Self::TargetIdentityProven => "TargetIdentityProven",
            Self::TargetExists => "TargetExists",
            Self::MinimalReversibleChange => "MinimalReversibleChange",
            Self::BackupPathOutsideActiveConfig => "BackupPathOutsideActiveConfig",
            Self::RehearsalProven => "RehearsalProven",
            Self::RehearsalMatchesCurrentContent => "RehearsalMatchesCurrentContent",
            Self::RollbackPlanPresent => "RollbackPlanPresent",
            Self::PostWriteVerificationPlanned => "PostWriteVerificationPlanned",
            Self::PostRestoreVerificationPlanned => "PostRestoreVerificationPlanned",
            Self::AutoreloadDisabledConfirmed => "AutoreloadDisabledConfirmed",
            Self::NoReloadPlanned => "NoReloadPlanned",
            Self::NoRuntimeMutationPlanned => "NoRuntimeMutationPlanned",
            Self::NoAutomaticApplyPath => "NoAutomaticApplyPath",
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct StructuredFamilyActiveConfigPilotApproval {
    pub pilot_approved: bool,
    pub typed_confirmation: String,
    pub backup_acknowledged: bool,
    pub restore_acknowledged: bool,
    pub verification_acknowledged: bool,
}

/// Read-only evidence about the compositor's config autoreload state. Either
/// externally supplied or collected via `collect_autoreload_evidence`, which
/// issues a single read-only option query and never mutates anything.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct StructuredFamilyActiveConfigAutoreloadEvidence {
    pub disable_autoreload_confirmed: bool,
    pub evidence_description: String,
}

/// Collect autoreload evidence through a read-only `getoption` query. The
/// gate only opens when `misc:disable_autoreload` reads back as exactly
/// `true`; any read failure, parse failure, or `false` fails closed.
pub fn collect_autoreload_evidence(
    runner: &mut dyn crate::runtime_preview_executor::RuntimePreviewRunner,
) -> StructuredFamilyActiveConfigAutoreloadEvidence {
    let observed =
        crate::runtime_preview_executor::read_runtime_option(runner, "misc.disable_autoreload");
    let confirmed = observed.as_deref() == Some("true");
    StructuredFamilyActiveConfigAutoreloadEvidence {
        disable_autoreload_confirmed: confirmed,
        evidence_description: match observed {
            Some(value) => format!(
                "read-only getoption misc:disable_autoreload returned {value:?}; {}",
                if confirmed {
                    "autoreload is disabled, so a config write cannot live-reload the compositor"
                } else {
                    "autoreload is active: a config write would live-reload the compositor, so the pilot stays blocked"
                }
            ),
            None => "read-only getoption misc:disable_autoreload could not be read; failing closed"
                .to_string(),
        },
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct StructuredFamilyActiveConfigRehearsalProof {
    pub family: StructuredFamilyKind,
    pub rehearsal_root: PathBuf,
    pub copied_target_path: PathBuf,
    pub source_hash: u64,
    pub source_untouched: bool,
    pub write_verified: bool,
    pub restore_verified: bool,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct StructuredFamilyActiveConfigPilotPlan {
    pub family: StructuredFamilyKind,
    pub active_config_path: PathBuf,
    pub original_records: Vec<String>,
    pub rendered_records: Vec<String>,
    pub backup_path: PathBuf,
    /// Must be true: the pilot always restores the original bytes.
    pub restore_original_bytes: bool,
    pub autoreload_evidence: StructuredFamilyActiveConfigAutoreloadEvidence,
    pub rehearsal: Option<StructuredFamilyActiveConfigRehearsalProof>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct StructuredFamilyActiveConfigPilotGateCheck {
    pub gate: StructuredFamilyActiveConfigPilotGate,
    pub passed: bool,
    pub detail: String,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct StructuredFamilyActiveConfigPilotPreflight {
    pub checks: Vec<StructuredFamilyActiveConfigPilotGateCheck>,
    pub passed: bool,
    pub blocking_gates: Vec<StructuredFamilyActiveConfigPilotGate>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum StructuredFamilyActiveConfigPilotError {
    GateFailed(StructuredFamilyActiveConfigPilotGate),
    TargetDriftDetected,
    TargetReadFailed(String),
    PreWriteRereadFailed(String),
    BackupFailed(String),
    WriteFailed(String),
    PostWriteVerificationFailed,
    RestoreFailed(String),
    PostRestoreVerificationFailed,
    RehearsalFailed(String),
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct StructuredFamilyActiveConfigPilotReceipt {
    pub family: StructuredFamilyKind,
    pub active_config_path: PathBuf,
    pub backup_path: PathBuf,
    pub pre_write_hash: u64,
    pub post_write_hash: u64,
    pub post_restore_hash: u64,
    pub written: bool,
    pub post_write_verification: StructuredFamilyControlledWriteVerification,
    pub restored: bool,
    pub post_restore_verification: StructuredFamilyControlledWriteVerification,
    pub original_bytes_restored: bool,
    pub hyprctl_reload_run: bool,
    pub runtime_mutated: bool,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct StructuredFamilyActiveConfigPilotAuditRecord {
    pub family: StructuredFamilyKind,
    pub active_config_path: PathBuf,
    pub backup_path: PathBuf,
    pub written: bool,
    pub post_write_verified: bool,
    pub restored: bool,
    pub post_restore_verified: bool,
    pub original_bytes_restored: bool,
    pub hyprctl_reload_run: bool,
    pub runtime_mutated: bool,
    pub summary: String,
}

fn family_records_from_file(
    path: &Path,
    family: StructuredFamilyKind,
) -> Result<Vec<String>, StructuredFamilyActiveConfigPilotError> {
    let contents = fs::read_to_string(path).map_err(|error| {
        StructuredFamilyActiveConfigPilotError::TargetReadFailed(error.to_string())
    })?;
    let parsed = parse_hyprland_config_text(path, &contents);
    let snapshot = CurrentConfigSnapshot::from_parsed(parsed);
    Ok(structured_family_projection(&snapshot, family)
        .records
        .iter()
        .map(|record| record.raw_line.clone())
        .collect())
}

/// Copy the active config into a temp rehearsal root and run the full
/// controlled write round trip (backup, write, verify, restore, verify)
/// against the copy. The active config itself is only read.
pub fn run_active_config_rehearsal(
    active_config_path: &Path,
    staged_apply: &StructuredFamilyDraftRenderedRecordStagedApplyPlan,
    appended_record: &str,
    rehearsal_root: &Path,
) -> Result<StructuredFamilyActiveConfigRehearsalProof, StructuredFamilyActiveConfigPilotError> {
    if !structured_family_path_is_active_real_config(active_config_path) {
        return Err(StructuredFamilyActiveConfigPilotError::RehearsalFailed(
            "rehearsal source is not the active real config".to_string(),
        ));
    }
    let source_bytes = fs::read(active_config_path).map_err(|error| {
        StructuredFamilyActiveConfigPilotError::TargetReadFailed(error.to_string())
    })?;
    let source_hash = active_config_pilot_content_hash(&source_bytes);

    fs::create_dir_all(rehearsal_root).map_err(|error| {
        StructuredFamilyActiveConfigPilotError::RehearsalFailed(error.to_string())
    })?;
    let copied_target_path = rehearsal_root.join("copied-active-hyprland.conf");
    fs::write(&copied_target_path, &source_bytes).map_err(|error| {
        StructuredFamilyActiveConfigPilotError::RehearsalFailed(error.to_string())
    })?;

    let family = staged_apply.family;
    let mut rendered_records = family_records_from_file(&copied_target_path, family)?;
    rendered_records.push(appended_record.to_string());

    let target = StructuredFamilyControlledWriteTarget::new(
        StructuredFamilyControlledWriteTargetKind::CopiedConfigTreeTarget,
        &copied_target_path,
        rehearsal_root,
    );
    let plan =
        build_structured_family_controlled_write_plan(staged_apply, target, rendered_records)
            .map_err(|error| {
                StructuredFamilyActiveConfigPilotError::RehearsalFailed(format!(
                    "controlled plan refused: {}",
                    error.as_str()
                ))
            })?;
    let receipt = execute_structured_family_controlled_write_round_trip(
        &plan,
        &approve_structured_family_controlled_write(),
    )
    .map_err(|error| {
        StructuredFamilyActiveConfigPilotError::RehearsalFailed(format!(
            "controlled round trip failed: {}",
            error.as_str()
        ))
    })?;

    let write_verified = receipt.post_write_verification.intended_records_present;
    let restore_verified = receipt
        .rollback
        .as_ref()
        .map(|rollback| {
            rollback.restored_byte_exact
                && rollback.post_restore_verification.intended_records_present
        })
        .unwrap_or(false);

    let source_after = fs::read(active_config_path).map_err(|error| {
        StructuredFamilyActiveConfigPilotError::TargetReadFailed(error.to_string())
    })?;
    let source_untouched = active_config_pilot_content_hash(&source_after) == source_hash;

    Ok(StructuredFamilyActiveConfigRehearsalProof {
        family,
        rehearsal_root: rehearsal_root.to_path_buf(),
        copied_target_path,
        source_hash,
        source_untouched,
        write_verified,
        restore_verified,
    })
}

/// Build the pilot plan: current records plus exactly one appended record.
pub fn build_first_active_config_pilot_plan(
    active_config_path: &Path,
    family: StructuredFamilyKind,
    appended_record: &str,
    backup_root: &Path,
    autoreload_evidence: StructuredFamilyActiveConfigAutoreloadEvidence,
    rehearsal: Option<StructuredFamilyActiveConfigRehearsalProof>,
) -> Result<StructuredFamilyActiveConfigPilotPlan, StructuredFamilyActiveConfigPilotError> {
    let original_records = family_records_from_file(active_config_path, family)?;
    let mut rendered_records = original_records.clone();
    rendered_records.push(appended_record.to_string());
    Ok(StructuredFamilyActiveConfigPilotPlan {
        family,
        active_config_path: active_config_path.to_path_buf(),
        original_records,
        rendered_records,
        backup_path: backup_root.join("hyprland.conf.active-pilot-backup"),
        restore_original_bytes: true,
        autoreload_evidence,
        rehearsal,
    })
}

/// Evaluate every pilot gate. All gates must pass immediately before the
/// write; any failure blocks the pilot.
pub fn preflight_first_active_config_pilot(
    plan: &StructuredFamilyActiveConfigPilotPlan,
    approval: &StructuredFamilyActiveConfigPilotApproval,
) -> StructuredFamilyActiveConfigPilotPreflight {
    use StructuredFamilyActiveConfigPilotGate as Gate;
    let mut checks = Vec::new();
    let mut check = |gate: Gate, passed: bool, detail: String| {
        checks.push(StructuredFamilyActiveConfigPilotGateCheck {
            gate,
            passed,
            detail,
        });
    };

    check(
        Gate::PilotApprovalPresent,
        approval.pilot_approved
            && approval.backup_acknowledged
            && approval.restore_acknowledged
            && approval.verification_acknowledged,
        "explicit pilot approval with backup/restore/verification acknowledgements".to_string(),
    );
    check(
        Gate::TypedConfirmationMatches,
        approval.typed_confirmation == ACTIVE_CONFIG_PILOT_CONFIRMATION_PHRASE,
        "typed confirmation must match the pilot phrase exactly".to_string(),
    );

    let is_active = structured_family_path_is_active_real_config(&plan.active_config_path);
    let metadata = fs::symlink_metadata(&plan.active_config_path).ok();
    let is_regular_file = metadata
        .as_ref()
        .map(|meta| meta.file_type().is_file())
        .unwrap_or(false);
    check(
        Gate::TargetIdentityProven,
        is_active && is_regular_file,
        format!(
            "target {} must be the active real config and a regular file (not a symlink)",
            plan.active_config_path.display()
        ),
    );
    check(
        Gate::TargetExists,
        metadata.is_some(),
        "target file must exist".to_string(),
    );

    let minimal = plan.rendered_records.len() == plan.original_records.len() + 1
        && plan.rendered_records[..plan.original_records.len()] == plan.original_records[..]
        && plan
            .rendered_records
            .last()
            .map(|record| !record.trim().is_empty() && !record.contains('\n'))
            .unwrap_or(false);
    check(
        Gate::MinimalReversibleChange,
        minimal,
        "change must preserve every original record and append exactly one single-line record"
            .to_string(),
    );

    check(
        Gate::BackupPathOutsideActiveConfig,
        !structured_family_path_is_active_real_config(&plan.backup_path),
        format!(
            "backup path {} must live outside the active config area",
            plan.backup_path.display()
        ),
    );

    let rehearsal_ok = plan
        .rehearsal
        .as_ref()
        .map(|rehearsal| {
            rehearsal.family == plan.family
                && rehearsal.write_verified
                && rehearsal.restore_verified
                && rehearsal.source_untouched
        })
        .unwrap_or(false);
    check(
        Gate::RehearsalProven,
        rehearsal_ok,
        "a copied-config rehearsal must have written, verified, restored, and verified the restore"
            .to_string(),
    );
    let rehearsal_current = plan
        .rehearsal
        .as_ref()
        .map(|rehearsal| {
            fs::read(&plan.active_config_path)
                .map(|bytes| active_config_pilot_content_hash(&bytes) == rehearsal.source_hash)
                .unwrap_or(false)
        })
        .unwrap_or(false);
    check(
        Gate::RehearsalMatchesCurrentContent,
        rehearsal_current,
        "the rehearsal must reflect the current active config content (no drift since rehearsal)"
            .to_string(),
    );

    check(
        Gate::RollbackPlanPresent,
        plan.restore_original_bytes,
        "the pilot must restore the original bytes after verification".to_string(),
    );
    check(
        Gate::PostWriteVerificationPlanned,
        true,
        "post-write parser/projection reread verification is unconditional".to_string(),
    );
    check(
        Gate::PostRestoreVerificationPlanned,
        true,
        "post-restore byte-exact and parser/projection verification is unconditional".to_string(),
    );

    check(
        Gate::AutoreloadDisabledConfirmed,
        plan.autoreload_evidence.disable_autoreload_confirmed,
        format!(
            "compositor autoreload must be confirmed disabled so the file write cannot mutate runtime; evidence: {}",
            plan.autoreload_evidence.evidence_description
        ),
    );
    check(
        Gate::NoReloadPlanned,
        true,
        "the pilot contains no reload path".to_string(),
    );
    check(
        Gate::NoRuntimeMutationPlanned,
        plan.autoreload_evidence.disable_autoreload_confirmed,
        "runtime must remain untouched; requires the autoreload evidence gate".to_string(),
    );
    check(
        Gate::NoAutomaticApplyPath,
        true,
        "no automatic or default-enabled apply path exists for the pilot".to_string(),
    );

    let blocking_gates: Vec<_> = checks
        .iter()
        .filter(|check| !check.passed)
        .map(|check| check.gate)
        .collect();
    StructuredFamilyActiveConfigPilotPreflight {
        passed: blocking_gates.is_empty(),
        checks,
        blocking_gates,
    }
}

/// Execute the first active real config write pilot: gates, backup, one
/// atomic write, reread verification, restoration, restore verification.
pub fn execute_first_active_config_write_pilot(
    plan: &StructuredFamilyActiveConfigPilotPlan,
    approval: &StructuredFamilyActiveConfigPilotApproval,
) -> Result<StructuredFamilyActiveConfigPilotReceipt, StructuredFamilyActiveConfigPilotError> {
    let preflight = preflight_first_active_config_pilot(plan, approval);
    if !preflight.passed {
        return Err(StructuredFamilyActiveConfigPilotError::GateFailed(
            preflight.blocking_gates[0],
        ));
    }

    let original_bytes = fs::read(&plan.active_config_path).map_err(|error| {
        StructuredFamilyActiveConfigPilotError::TargetReadFailed(error.to_string())
    })?;
    let pre_write_hash = active_config_pilot_content_hash(&original_bytes);

    // Pre-write reread: the file must parse and its family records must match
    // the plan exactly; any drift aborts before backup or write.
    let current_records =
        family_records_from_file(&plan.active_config_path, plan.family).map_err(|_| {
            StructuredFamilyActiveConfigPilotError::PreWriteRereadFailed(
                "active config did not reread through the parser/projection path".to_string(),
            )
        })?;
    if current_records != plan.original_records {
        return Err(StructuredFamilyActiveConfigPilotError::TargetDriftDetected);
    }

    if let Some(parent) = plan.backup_path.parent() {
        fs::create_dir_all(parent).map_err(|error| {
            StructuredFamilyActiveConfigPilotError::BackupFailed(error.to_string())
        })?;
    }
    fs::write(&plan.backup_path, &original_bytes)
        .map_err(|error| StructuredFamilyActiveConfigPilotError::BackupFailed(error.to_string()))?;
    let backup_bytes = fs::read(&plan.backup_path)
        .map_err(|error| StructuredFamilyActiveConfigPilotError::BackupFailed(error.to_string()))?;
    if backup_bytes != original_bytes {
        return Err(StructuredFamilyActiveConfigPilotError::BackupFailed(
            "backup is not byte-exact".to_string(),
        ));
    }

    let original_text = String::from_utf8_lossy(&original_bytes).into_owned();
    let new_text = apply_rendered_family_records(
        &plan.active_config_path,
        &original_text,
        plan.family,
        &plan.rendered_records,
    );
    atomic_controlled_write(&plan.active_config_path, new_text.as_bytes())
        .map_err(|error| StructuredFamilyActiveConfigPilotError::WriteFailed(error.to_string()))?;

    let post_write_verification = verify_family_records_on_disk(
        &plan.active_config_path,
        plan.family,
        &plan.rendered_records,
    )
    .map_err(|error| {
        StructuredFamilyActiveConfigPilotError::PreWriteRereadFailed(format!(
            "post-write reread failed: {}",
            error.as_str()
        ))
    })?;
    let post_write_hash = fs::read(&plan.active_config_path)
        .map(|bytes| active_config_pilot_content_hash(&bytes))
        .map_err(|error| StructuredFamilyActiveConfigPilotError::WriteFailed(error.to_string()))?;

    if !post_write_verification.intended_records_present {
        atomic_controlled_write(&plan.active_config_path, &original_bytes).map_err(|error| {
            StructuredFamilyActiveConfigPilotError::RestoreFailed(error.to_string())
        })?;
        return Err(StructuredFamilyActiveConfigPilotError::PostWriteVerificationFailed);
    }

    // Restore the original bytes: the pilot proves the round trip and leaves
    // the active config exactly as it was.
    atomic_controlled_write(&plan.active_config_path, &original_bytes).map_err(|error| {
        StructuredFamilyActiveConfigPilotError::RestoreFailed(error.to_string())
    })?;
    let restored_bytes = fs::read(&plan.active_config_path).map_err(|error| {
        StructuredFamilyActiveConfigPilotError::RestoreFailed(error.to_string())
    })?;
    let post_restore_hash = active_config_pilot_content_hash(&restored_bytes);
    let post_restore_verification = verify_family_records_on_disk(
        &plan.active_config_path,
        plan.family,
        &plan.original_records,
    )
    .map_err(|error| {
        StructuredFamilyActiveConfigPilotError::RestoreFailed(format!(
            "post-restore reread failed: {}",
            error.as_str()
        ))
    })?;
    let original_bytes_restored =
        restored_bytes == original_bytes && post_restore_hash == pre_write_hash;
    if !original_bytes_restored || !post_restore_verification.intended_records_present {
        return Err(StructuredFamilyActiveConfigPilotError::PostRestoreVerificationFailed);
    }

    Ok(StructuredFamilyActiveConfigPilotReceipt {
        family: plan.family,
        active_config_path: plan.active_config_path.clone(),
        backup_path: plan.backup_path.clone(),
        pre_write_hash,
        post_write_hash,
        post_restore_hash,
        written: true,
        post_write_verification,
        restored: true,
        post_restore_verification,
        original_bytes_restored,
        hyprctl_reload_run: false,
        runtime_mutated: false,
    })
}

pub fn structured_family_active_config_pilot_audit_record(
    receipt: &StructuredFamilyActiveConfigPilotReceipt,
) -> StructuredFamilyActiveConfigPilotAuditRecord {
    StructuredFamilyActiveConfigPilotAuditRecord {
        family: receipt.family,
        active_config_path: receipt.active_config_path.clone(),
        backup_path: receipt.backup_path.clone(),
        written: receipt.written,
        post_write_verified: receipt.post_write_verification.intended_records_present,
        restored: receipt.restored,
        post_restore_verified: receipt.post_restore_verification.intended_records_present,
        original_bytes_restored: receipt.original_bytes_restored,
        hyprctl_reload_run: false,
        runtime_mutated: false,
        summary: format!(
            "first active config write pilot for {} against {}: written {}, post-write verified {}, restored {}, post-restore verified {}, original bytes restored {}; no compositor reload, no runtime mutation",
            receipt.family.family_id(),
            receipt.active_config_path.display(),
            receipt.written,
            receipt.post_write_verification.intended_records_present,
            receipt.restored,
            receipt.post_restore_verification.intended_records_present,
            receipt.original_bytes_restored,
        ),
    }
}
