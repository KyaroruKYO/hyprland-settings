use std::collections::{hash_map::DefaultHasher, BTreeMap};
use std::fs;
use std::hash::{Hash, Hasher};
use std::path::{Path, PathBuf};

use anyhow::{anyhow, Result};
use serde::Deserialize;

use crate::config_graph::{
    inspect_config_graph_with_options, ConfigGraphOptions, ConfigManagementHintKind,
    SourceFollowPolicy,
};
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

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ControlledLiveTestKind {
    SourceIncludeInsertion,
    DuplicateReplacement,
    HighRiskDisplayWrite,
    StructuredWrite,
    ProfileSwitch,
    RuntimeMutation,
    HyprlandVersionMigration,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ControlledLiveTestGuardRequest {
    pub test_id: String,
    pub target_paths: Vec<PathBuf>,
    pub backup_paths_recorded: bool,
    pub original_hashes_recorded: bool,
    pub symlink_targets_recorded: bool,
    pub read_only_runtime_snapshot_recorded: bool,
    pub restore_plan_recorded: bool,
    pub post_restore_verification_planned: bool,
    pub out_of_band_recovery_recorded: bool,
    pub trusted_data_available: bool,
    pub explicit_live_flag: bool,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ControlledLiveTestGuardReview {
    pub kind: ControlledLiveTestKind,
    pub test_id: String,
    pub live_mutation_allowed: bool,
    pub restore_required: bool,
    pub real_config_touch_allowed: bool,
    pub runtime_mutation_allowed: bool,
    pub blockers: Vec<String>,
    pub review_lines: Vec<String>,
}

pub fn controlled_live_test_guard_review(
    kind: ControlledLiveTestKind,
    request: ControlledLiveTestGuardRequest,
) -> ControlledLiveTestGuardReview {
    let mut blockers = Vec::new();
    if request.test_id.trim().is_empty() {
        blockers.push("timestamped live-test id is required".to_string());
    }
    if !request.explicit_live_flag {
        blockers.push("explicit live-test flag is required before mutation".to_string());
    }
    if request.target_paths.is_empty() {
        blockers.push("at least one target path must be recorded".to_string());
    }
    if !request.backup_paths_recorded {
        blockers.push("backup paths must be recorded before mutation".to_string());
    }
    if !request.original_hashes_recorded {
        blockers.push("original SHA256 hashes must be recorded before mutation".to_string());
    }
    if !request.restore_plan_recorded {
        blockers.push("restore plan must be prepared before mutation".to_string());
    }
    if !request.post_restore_verification_planned {
        blockers.push("post-restore verification must be planned before mutation".to_string());
    }

    match kind {
        ControlledLiveTestKind::HighRiskDisplayWrite => {
            if !request.out_of_band_recovery_recorded {
                blockers.push(
                    "out-of-band recovery path is required before high-risk/display mutation"
                        .to_string(),
                );
            }
            if !request.read_only_runtime_snapshot_recorded {
                blockers.push(
                    "read-only runtime snapshot is required before high-risk/display mutation"
                        .to_string(),
                );
            }
        }
        ControlledLiveTestKind::ProfileSwitch => {
            if !request.symlink_targets_recorded {
                blockers.push(
                    "original symlink targets must be recorded before profile switching"
                        .to_string(),
                );
            }
        }
        ControlledLiveTestKind::RuntimeMutation => {
            if !request.read_only_runtime_snapshot_recorded {
                blockers.push(
                    "read-only runtime snapshot is required before runtime mutation".to_string(),
                );
            }
        }
        ControlledLiveTestKind::HyprlandVersionMigration => {
            if !request.trusted_data_available {
                blockers.push(
                    "trusted versioned data bundle is required before migration activation"
                        .to_string(),
                );
            }
        }
        ControlledLiveTestKind::SourceIncludeInsertion
        | ControlledLiveTestKind::DuplicateReplacement
        | ControlledLiveTestKind::StructuredWrite => {}
    }

    let live_mutation_allowed = blockers.is_empty();
    ControlledLiveTestGuardReview {
        kind,
        test_id: request.test_id,
        live_mutation_allowed,
        restore_required: true,
        real_config_touch_allowed: live_mutation_allowed
            && !matches!(
                kind,
                ControlledLiveTestKind::RuntimeMutation
                    | ControlledLiveTestKind::HyprlandVersionMigration
            ),
        runtime_mutation_allowed: live_mutation_allowed
            && matches!(kind, ControlledLiveTestKind::RuntimeMutation),
        blockers,
        review_lines: vec![
            "Controlled live tests require pre-snapshot, backup, restore, and verification."
                .to_string(),
            "The guard records approval readiness only; it does not execute commands.".to_string(),
            "Every mutation must be restored before the sprint ends.".to_string(),
        ],
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ApprovalScope {
    SourceIncludeInsertion,
    DuplicateReplacement,
    StructuredHlBindWrite,
    ProfileModeSwitch,
    RuntimeKeyword,
    RuntimeReload,
    HighRiskDisplayWrite,
    Hyprland0554Migration,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ApprovalStatus {
    MissingEvidence,
    WrongScope,
    Pending,
    ApprovedButDefaultDisabled,
    Rejected,
    Expired,
    ReadyButDefaultDisabled,
    Enabled,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ApprovalToken {
    pub token: String,
    pub expires_at_tick: Option<u64>,
    pub one_shot: bool,
    pub used: bool,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ApprovalEvidence {
    pub target_path: Option<PathBuf>,
    pub runtime_command: Option<String>,
    pub copied_config_tree_proof_restored: bool,
    pub live_restore_proof_restored: bool,
    pub old_state: Option<String>,
    pub proposed_new_state: Option<String>,
    pub restore_plan: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ApprovalRequest {
    pub scope: ApprovalScope,
    pub evidence: ApprovalEvidence,
    pub token: ApprovalToken,
    pub provided_token: Option<String>,
    pub current_tick: u64,
    pub rejected: bool,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ApprovalDecision {
    pub expected_scope: ApprovalScope,
    pub status: ApprovalStatus,
    pub evidence: Option<ApprovalEvidence>,
    pub production_flag_enabled: bool,
    pub production_apply_enabled: bool,
    pub blockers: Vec<String>,
    pub review_lines: Vec<String>,
}

pub fn approval_decision_for_gate(
    expected_scope: ApprovalScope,
    gate_ready: bool,
    expected_target_path: Option<&Path>,
    expected_runtime_command: Option<&str>,
    request: Option<&ApprovalRequest>,
    production_flag_enabled: bool,
) -> ApprovalDecision {
    let mut blockers = Vec::new();
    let Some(request) = request else {
        return approval_decision_blocked(
            expected_scope,
            ApprovalStatus::MissingEvidence,
            None,
            production_flag_enabled,
            "approval request is required",
        );
    };
    if request.scope != expected_scope {
        return approval_decision_blocked(
            expected_scope,
            ApprovalStatus::WrongScope,
            Some(request.evidence.clone()),
            production_flag_enabled,
            "approval scope does not match the production gate",
        );
    }
    if request.rejected {
        return approval_decision_blocked(
            expected_scope,
            ApprovalStatus::Rejected,
            Some(request.evidence.clone()),
            production_flag_enabled,
            "approval request was rejected",
        );
    }
    if request
        .token
        .expires_at_tick
        .map(|expires_at| request.current_tick >= expires_at)
        .unwrap_or(false)
        || request.token.one_shot && request.token.used
    {
        return approval_decision_blocked(
            expected_scope,
            ApprovalStatus::Expired,
            Some(request.evidence.clone()),
            production_flag_enabled,
            "approval token is expired or already used",
        );
    }
    if request.provided_token.as_deref() != Some(request.token.token.as_str()) {
        return approval_decision_blocked(
            expected_scope,
            ApprovalStatus::Pending,
            Some(request.evidence.clone()),
            production_flag_enabled,
            "approval token has not been confirmed",
        );
    }
    if !gate_ready {
        blockers.push("production gate is not ready for approval".to_string());
    }
    if let Some(expected_target_path) = expected_target_path {
        if request.evidence.target_path.as_deref() != Some(expected_target_path) {
            blockers.push("approval target path does not match the production gate".to_string());
        }
    }
    if let Some(expected_runtime_command) = expected_runtime_command {
        if request.evidence.runtime_command.as_deref() != Some(expected_runtime_command) {
            blockers
                .push("approval runtime command does not match the production gate".to_string());
        }
    }
    if request.evidence.old_state.is_none() {
        blockers.push("approval old state is required".to_string());
    }
    if request.evidence.proposed_new_state.is_none() {
        blockers.push("approval proposed new state is required".to_string());
    }
    if request.evidence.restore_plan.is_none() {
        blockers.push("approval restore plan is required".to_string());
    }
    if !request.evidence.copied_config_tree_proof_restored
        && !request.evidence.live_restore_proof_restored
    {
        blockers.push("approval must link copied-config-tree or live-restore proof".to_string());
    }

    if !blockers.is_empty() {
        return ApprovalDecision {
            expected_scope,
            status: ApprovalStatus::MissingEvidence,
            evidence: Some(request.evidence.clone()),
            production_flag_enabled,
            production_apply_enabled: false,
            blockers,
            review_lines: vec![
                "Explicit approval is required before any gated production capability can be considered.".to_string(),
                "The approval evidence is incomplete or does not match the gate.".to_string(),
            ],
        };
    }

    let mut status = if request.evidence.live_restore_proof_restored {
        ApprovalStatus::ReadyButDefaultDisabled
    } else {
        ApprovalStatus::ApprovedButDefaultDisabled
    };
    if production_flag_enabled {
        status = ApprovalStatus::Enabled;
    }
    let production_apply_enabled = status == ApprovalStatus::Enabled;
    let mut blockers = Vec::new();
    if !production_flag_enabled {
        blockers.push("production flag remains default-disabled".to_string());
    }
    ApprovalDecision {
        expected_scope,
        status,
        evidence: Some(request.evidence.clone()),
        production_flag_enabled,
        production_apply_enabled,
        blockers: blockers.clone(),
        review_lines: vec![
            "Approval scope, target/command, state change, restore plan, and proof are linked."
                .to_string(),
            "Approval remains one-shot/expiring and does not enable production by default."
                .to_string(),
            format!(
                "Blockers: {}",
                if blockers.is_empty() {
                    "none".to_string()
                } else {
                    blockers.join("; ")
                }
            ),
        ],
    }
}

fn approval_decision_blocked(
    expected_scope: ApprovalScope,
    status: ApprovalStatus,
    evidence: Option<ApprovalEvidence>,
    production_flag_enabled: bool,
    blocker: &str,
) -> ApprovalDecision {
    ApprovalDecision {
        expected_scope,
        status,
        evidence,
        production_flag_enabled,
        production_apply_enabled: false,
        blockers: vec![blocker.to_string()],
        review_lines: vec![
            "Explicit approval is required before any gated production capability can be considered."
                .to_string(),
            blocker.to_string(),
        ],
    }
}

pub fn source_include_approval_flow(
    review: &SourceIncludeProductionReview,
    request: Option<&ApprovalRequest>,
) -> ApprovalDecision {
    approval_decision_for_gate(
        ApprovalScope::SourceIncludeInsertion,
        review.gate.status == SourceIncludeProductionGateStatus::ReadyButDefaultDisabled,
        review.gate.selected_target_path.as_deref(),
        None,
        request,
        false,
    )
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SourceIncludeInsertionReadiness {
    SingleRootEligible,
    SourceIncludeTargetSelectionRequired,
    ManagedTargetBlocked,
    DuplicateOrAmbiguousBlocked,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SourceIncludeInsertionReview {
    pub root_path: PathBuf,
    pub candidate_targets: Vec<PathBuf>,
    pub selected_target: Option<PathBuf>,
    pub readiness: SourceIncludeInsertionReadiness,
    pub production_insertion_enabled: bool,
    pub review_lines: Vec<String>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SourceIncludeTargetSelectionStatus {
    NoTargetSelected,
    SelectedTargetReadyForFixture,
    ManagedTargetBlocked,
    TargetNotCandidate,
    DuplicateOrAmbiguousBlocked,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SourceIncludeTargetCandidate {
    pub path: PathBuf,
    pub source_depth: usize,
    pub generated_or_script_managed: bool,
    pub symlink_or_profile_managed: bool,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SourceIncludeTargetPrecondition {
    pub root_path: PathBuf,
    pub selected_target: PathBuf,
    pub source_depth: usize,
    pub generated_or_script_managed: bool,
    pub symlink_or_profile_managed: bool,
    pub candidate_count: usize,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SourceIncludeTargetSelectionProof {
    pub status: SourceIncludeTargetSelectionStatus,
    pub precondition: Option<SourceIncludeTargetPrecondition>,
    pub fixture_plan_allowed: bool,
    pub production_insertion_enabled: bool,
    pub real_config_touched: bool,
    pub review_lines: Vec<String>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SourceIncludeSelectedTargetDryRunStatus {
    Planned,
    SelectionBlocked,
    TargetMismatch,
    InsertionPlanBlocked,
    NonFixtureTargetRefused,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SourceIncludeSelectedTargetDryRunPlan {
    pub status: SourceIncludeSelectedTargetDryRunStatus,
    pub root_path: Option<PathBuf>,
    pub selected_target: Option<PathBuf>,
    pub source_depth: Option<usize>,
    pub insertion_line: Option<String>,
    pub dry_run_preview: Option<String>,
    pub blocked_reasons: Vec<String>,
    pub production_insertion_enabled: bool,
    pub real_config_touched: bool,
    pub runtime_touched: bool,
    pub review_lines: Vec<String>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum GuardedTempExecutionStatus {
    SucceededAndRestored,
    Blocked,
    VerificationFailedRestored,
    RestoreFailed,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct GuardedTempExecutionReport {
    pub status: GuardedTempExecutionStatus,
    pub target_path: PathBuf,
    pub backup_path: Option<PathBuf>,
    pub original_hash: Option<String>,
    pub restored_hash: Option<String>,
    pub planned_line: Option<String>,
    pub mutation_verified: bool,
    pub restore_attempted: bool,
    pub restore_succeeded: bool,
    pub production_write_enabled: bool,
    pub real_config_touched: bool,
    pub runtime_touched: bool,
    pub errors: Vec<String>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CopiedConfigTreeFile {
    pub original_path: PathBuf,
    pub copied_path: PathBuf,
    pub source_depth: usize,
    pub is_symlink: bool,
    pub original_symlink_target: Option<PathBuf>,
    pub copied_symlink_target: Option<PathBuf>,
    pub original_fingerprint: String,
    pub copied_initial_fingerprint: String,
    pub generated_or_script_managed: bool,
    pub symlink_or_profile_managed: bool,
    pub target_eligible: bool,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CopiedConfigTreeSnapshot {
    pub original_root_path: PathBuf,
    pub copied_root_path: PathBuf,
    pub copy_root: PathBuf,
    pub files: Vec<CopiedConfigTreeFile>,
    pub real_config_touched: bool,
    pub runtime_touched: bool,
    pub production_behavior_enabled: bool,
    pub errors: Vec<String>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CopiedConfigTreeReport {
    pub snapshot: CopiedConfigTreeSnapshot,
    pub originals_unchanged: bool,
    pub copied_files_restored: bool,
    pub source_include_executor_restored: Option<bool>,
    pub duplicate_executor_restored: Option<bool>,
    pub structured_executor_restored: Option<bool>,
    pub profile_executor_restored: Option<bool>,
    pub review_lines: Vec<String>,
}

pub fn source_include_insertion_review(
    root_path: impl Into<PathBuf>,
    candidate_targets: Vec<PathBuf>,
    selected_target: Option<PathBuf>,
    managed_or_ambiguous: bool,
) -> SourceIncludeInsertionReview {
    let root_path = root_path.into();
    let readiness = if managed_or_ambiguous {
        SourceIncludeInsertionReadiness::ManagedTargetBlocked
    } else if candidate_targets.len() == 1 && selected_target.as_ref() == Some(&root_path) {
        SourceIncludeInsertionReadiness::SingleRootEligible
    } else if candidate_targets.len() > 1 || selected_target.is_some() {
        SourceIncludeInsertionReadiness::SourceIncludeTargetSelectionRequired
    } else {
        SourceIncludeInsertionReadiness::DuplicateOrAmbiguousBlocked
    };

    SourceIncludeInsertionReview {
        root_path,
        candidate_targets,
        selected_target,
        readiness,
        production_insertion_enabled: readiness == SourceIncludeInsertionReadiness::SingleRootEligible,
        review_lines: vec![
            "Missing/default insertion can write only a reviewed single-file root config today."
                .to_string(),
            "Source/include insertion needs explicit target selection before production activation."
                .to_string(),
            "Generated, script-managed, symlink/current-profile, duplicate, and ambiguous targets stay blocked.".to_string(),
        ],
    }
}

pub fn source_include_target_selection_fixture_proof(
    root_path: impl Into<PathBuf>,
    candidates: Vec<SourceIncludeTargetCandidate>,
    selected_target: Option<PathBuf>,
    duplicate_or_ambiguous: bool,
) -> SourceIncludeTargetSelectionProof {
    let root_path = root_path.into();
    let Some(selected_target) = selected_target else {
        return SourceIncludeTargetSelectionProof {
            status: SourceIncludeTargetSelectionStatus::NoTargetSelected,
            precondition: None,
            fixture_plan_allowed: false,
            production_insertion_enabled: false,
            real_config_touched: false,
            review_lines: vec![
                "No source/include target file is selected.".to_string(),
                "Source/include insertion remains blocked.".to_string(),
            ],
        };
    };
    let Some(candidate) = candidates
        .iter()
        .find(|candidate| candidate.path == selected_target)
    else {
        return SourceIncludeTargetSelectionProof {
            status: SourceIncludeTargetSelectionStatus::TargetNotCandidate,
            precondition: None,
            fixture_plan_allowed: false,
            production_insertion_enabled: false,
            real_config_touched: false,
            review_lines: vec![
                "Selected target is not part of the reviewed source/include graph.".to_string(),
                "Source/include insertion remains blocked.".to_string(),
            ],
        };
    };
    let precondition = SourceIncludeTargetPrecondition {
        root_path,
        selected_target: selected_target.clone(),
        source_depth: candidate.source_depth,
        generated_or_script_managed: candidate.generated_or_script_managed,
        symlink_or_profile_managed: candidate.symlink_or_profile_managed,
        candidate_count: candidates.len(),
    };
    let status = if candidate.generated_or_script_managed || candidate.symlink_or_profile_managed {
        SourceIncludeTargetSelectionStatus::ManagedTargetBlocked
    } else if duplicate_or_ambiguous {
        SourceIncludeTargetSelectionStatus::DuplicateOrAmbiguousBlocked
    } else {
        SourceIncludeTargetSelectionStatus::SelectedTargetReadyForFixture
    };
    let fixture_plan_allowed =
        status == SourceIncludeTargetSelectionStatus::SelectedTargetReadyForFixture;

    SourceIncludeTargetSelectionProof {
        status,
        precondition: Some(precondition),
        fixture_plan_allowed,
        production_insertion_enabled: false,
        real_config_touched: false,
        review_lines: vec![
            "Source/include target selection is fixture proof only.".to_string(),
            "Production source/include insertion remains disabled.".to_string(),
            "The app must not auto-select root, source, profile, or generated targets.".to_string(),
        ],
    }
}

pub fn execute_source_include_selected_target_guarded_temp(
    proof: &SourceIncludeTargetSelectionProof,
    dry_run: &SourceIncludeSelectedTargetDryRunPlan,
    guard: &ControlledLiveTestGuardReview,
    force_verification_failure: bool,
    force_restore_failure: bool,
) -> GuardedTempExecutionReport {
    let target_path = dry_run.selected_target.clone().unwrap_or_default();
    let Some(planned_line) = dry_run.insertion_line.clone() else {
        return guarded_temp_blocked(target_path, "dry-run plan has no insertion line");
    };
    if !guard.live_mutation_allowed || !guard.real_config_touch_allowed {
        return guarded_temp_blocked(
            target_path,
            "controlled live-test guard did not allow file mutation",
        );
    }
    if dry_run.status != SourceIncludeSelectedTargetDryRunStatus::Planned
        || !proof.fixture_plan_allowed
    {
        return guarded_temp_blocked(target_path, "selected source/include target is not planned");
    }
    execute_guarded_temp_line_mutation(
        target_path,
        planned_line.clone(),
        format!("\n# Guarded source/include selected-target insertion proof\n{planned_line}\n"),
        |contents| {
            contents
                .lines()
                .any(|line| line.trim() == planned_line.trim())
        },
        force_verification_failure,
        force_restore_failure,
    )
}

pub fn copy_config_tree_for_proof(
    root_path: impl AsRef<Path>,
    copy_root: impl AsRef<Path>,
) -> CopiedConfigTreeSnapshot {
    let root_path = root_path.as_ref().to_path_buf();
    let copy_root = copy_root.as_ref().to_path_buf();
    let root_parent = root_path
        .parent()
        .map(Path::to_path_buf)
        .unwrap_or_else(|| PathBuf::from("/"));
    let options = ConfigGraphOptions {
        home_dir: root_parent.parent().map(Path::to_path_buf),
        script_dirs: Vec::new(),
        max_depth: 16,
        source_follow_policy: SourceFollowPolicy::ReviewAll,
    };
    let graph = inspect_config_graph_with_options(&root_path, options);
    let copied_root_path = map_original_path_to_copy(&root_path, &root_parent, &copy_root);
    let mut files = Vec::new();
    let mut errors = Vec::new();

    for file in graph.files {
        if !file.readable {
            errors.push(format!("skipped unreadable file {}", file.path.display()));
            continue;
        }
        let copied_path = map_original_path_to_copy(&file.path, &root_parent, &copy_root);
        if let Some(parent) = copied_path.parent() {
            if let Err(error) = fs::create_dir_all(parent) {
                errors.push(format!("could not create {}: {error}", parent.display()));
                continue;
            }
        }

        let generated_or_script_managed = file.hints.iter().any(|hint| {
            matches!(
                hint.kind,
                ConfigManagementHintKind::GeneratedFile
                    | ConfigManagementHintKind::ScriptManaged
                    | ConfigManagementHintKind::ScriptReferenced
            )
        });
        let symlink_or_profile_managed = file.is_symlink
            || file.hints.iter().any(|hint| {
                matches!(
                    hint.kind,
                    ConfigManagementHintKind::CurrentProfile
                        | ConfigManagementHintKind::DesktopProfile
                        | ConfigManagementHintKind::GamingProfile
                        | ConfigManagementHintKind::LaptopProfile
                        | ConfigManagementHintKind::PerformanceProfile
                        | ConfigManagementHintKind::ModeProfile
                        | ConfigManagementHintKind::SymlinkManaged
                )
            });

        let original_fingerprint = if file.is_symlink {
            symlink_fingerprint(file.symlink_target.as_ref())
        } else {
            match fs::read(&file.path) {
                Ok(bytes) => content_fingerprint(&bytes),
                Err(error) => {
                    errors.push(format!("could not read {}: {error}", file.path.display()));
                    continue;
                }
            }
        };

        let mut copied_symlink_target = None;
        if file.is_symlink {
            #[cfg(unix)]
            {
                if let Some(target) = file.symlink_target.as_ref() {
                    let copied_target = map_original_path_to_copy(target, &root_parent, &copy_root);
                    if let Some(parent) = copied_target.parent() {
                        if let Err(error) = fs::create_dir_all(parent) {
                            errors.push(format!(
                                "could not create symlink target parent {}: {error}",
                                parent.display()
                            ));
                        }
                    }
                    if target.is_file() {
                        if let Err(error) = fs::copy(target, &copied_target) {
                            errors.push(format!(
                                "could not copy symlink target {}: {error}",
                                target.display()
                            ));
                        }
                    }
                    let _ = fs::remove_file(&copied_path);
                    match std::os::unix::fs::symlink(&copied_target, &copied_path) {
                        Ok(()) => copied_symlink_target = Some(copied_target),
                        Err(error) => errors.push(format!(
                            "could not create copied symlink {}: {error}",
                            copied_path.display()
                        )),
                    }
                }
            }
            #[cfg(not(unix))]
            {
                errors.push("symlink copy proof requires unix symlink support".to_string());
            }
        } else if let Err(error) = fs::copy(&file.path, &copied_path) {
            errors.push(format!(
                "could not copy {} to {}: {error}",
                file.path.display(),
                copied_path.display()
            ));
            continue;
        }

        let copied_initial_fingerprint = if file.is_symlink {
            symlink_fingerprint(copied_symlink_target.as_ref())
        } else {
            match fs::read(&copied_path) {
                Ok(bytes) => content_fingerprint(&bytes),
                Err(error) => {
                    errors.push(format!(
                        "could not read copy {}: {error}",
                        copied_path.display()
                    ));
                    String::new()
                }
            }
        };

        files.push(CopiedConfigTreeFile {
            original_path: file.path,
            copied_path,
            source_depth: file.source_depth,
            is_symlink: file.is_symlink,
            original_symlink_target: file.symlink_target,
            copied_symlink_target,
            original_fingerprint,
            copied_initial_fingerprint,
            generated_or_script_managed,
            symlink_or_profile_managed,
            target_eligible: !generated_or_script_managed && !symlink_or_profile_managed,
        });
    }

    CopiedConfigTreeSnapshot {
        original_root_path: root_path,
        copied_root_path,
        copy_root,
        files,
        real_config_touched: false,
        runtime_touched: false,
        production_behavior_enabled: false,
        errors,
    }
}

pub fn copied_config_tree_originals_unchanged(snapshot: &CopiedConfigTreeSnapshot) -> bool {
    snapshot.files.iter().all(|file| {
        if file.is_symlink {
            path_symlink_fingerprint(&file.original_path) == file.original_fingerprint
        } else {
            fs::read(&file.original_path)
                .map(|bytes| content_fingerprint(&bytes) == file.original_fingerprint)
                .unwrap_or(false)
        }
    })
}

pub fn copied_config_tree_files_restored(snapshot: &CopiedConfigTreeSnapshot) -> bool {
    snapshot.files.iter().all(|file| {
        if file.is_symlink {
            path_symlink_fingerprint(&file.copied_path) == file.copied_initial_fingerprint
        } else {
            fs::read(&file.copied_path)
                .map(|bytes| content_fingerprint(&bytes) == file.copied_initial_fingerprint)
                .unwrap_or(false)
        }
    })
}

pub fn copied_config_tree_report(
    snapshot: CopiedConfigTreeSnapshot,
    source_include_executor: Option<&GuardedTempExecutionReport>,
    duplicate_executor: Option<&GuardedTempExecutionReport>,
    structured_executor: Option<&GuardedTempExecutionReport>,
    profile_executor_restored: Option<bool>,
) -> CopiedConfigTreeReport {
    let originals_unchanged = copied_config_tree_originals_unchanged(&snapshot);
    let copied_files_restored = copied_config_tree_files_restored(&snapshot);
    CopiedConfigTreeReport {
        snapshot,
        originals_unchanged,
        copied_files_restored,
        source_include_executor_restored: source_include_executor.map(|report| {
            report.restore_succeeded && report.status == GuardedTempExecutionStatus::SucceededAndRestored
        }),
        duplicate_executor_restored: duplicate_executor.map(|report| {
            report.restore_succeeded && report.status == GuardedTempExecutionStatus::SucceededAndRestored
        }),
        structured_executor_restored: structured_executor.map(|report| {
            report.restore_succeeded && report.status == GuardedTempExecutionStatus::SucceededAndRestored
        }),
        profile_executor_restored,
        review_lines: vec![
            "Copied-config-tree proof runs only against temp copies.".to_string(),
            "Original real config files are fingerprinted and verified unchanged.".to_string(),
            "Production source/include, duplicate, structured, profile, runtime, and high-risk behavior remains disabled by default.".to_string(),
        ],
    }
}

fn copied_report_has_path(report: &CopiedConfigTreeReport, path: &Path) -> bool {
    report
        .snapshot
        .files
        .iter()
        .any(|file| file.copied_path == path)
}

fn copied_report_base_ready(report: &CopiedConfigTreeReport, path: &Path) -> bool {
    report.originals_unchanged
        && report.copied_files_restored
        && copied_report_has_path(report, path)
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SourceIncludeProductionGateStatus {
    NoTargetSelected,
    TargetNotInSourceGraph,
    ManagedTargetBlocked,
    DuplicateOrAmbiguousBlocked,
    MissingDryRunPlan,
    MissingCopiedProof,
    CopiedProofMismatch,
    ReadyButDefaultDisabled,
    Enabled,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SourceIncludeProductionGate {
    pub status: SourceIncludeProductionGateStatus,
    pub root_path: Option<PathBuf>,
    pub selected_target_path: Option<PathBuf>,
    pub source_depth: Option<usize>,
    pub planned_line: Option<String>,
    pub proposed_value: Option<String>,
    pub copied_proof_restored: bool,
    pub production_flag_enabled: bool,
    pub production_apply_enabled: bool,
    pub blockers: Vec<String>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SourceIncludeProductionReview {
    pub gate: SourceIncludeProductionGate,
    pub review_lines: Vec<String>,
}

pub fn source_include_production_gate_review(
    proof: &SourceIncludeTargetSelectionProof,
    dry_run: Option<&SourceIncludeSelectedTargetDryRunPlan>,
    copied_proof: Option<&CopiedConfigTreeReport>,
    production_flag_enabled: bool,
) -> SourceIncludeProductionReview {
    let mut blockers = Vec::new();
    let mut status = match proof.status {
        SourceIncludeTargetSelectionStatus::NoTargetSelected => {
            blockers.push("explicit source/include target selection is required".to_string());
            SourceIncludeProductionGateStatus::NoTargetSelected
        }
        SourceIncludeTargetSelectionStatus::TargetNotCandidate => {
            blockers.push("selected target is not in the reviewed source graph".to_string());
            SourceIncludeProductionGateStatus::TargetNotInSourceGraph
        }
        SourceIncludeTargetSelectionStatus::ManagedTargetBlocked => {
            blockers.push(
                "generated, script-managed, symlink, profile, or mode targets are blocked"
                    .to_string(),
            );
            SourceIncludeProductionGateStatus::ManagedTargetBlocked
        }
        SourceIncludeTargetSelectionStatus::DuplicateOrAmbiguousBlocked => {
            blockers.push("duplicate or ambiguous source/include target is blocked".to_string());
            SourceIncludeProductionGateStatus::DuplicateOrAmbiguousBlocked
        }
        SourceIncludeTargetSelectionStatus::SelectedTargetReadyForFixture => {
            SourceIncludeProductionGateStatus::ReadyButDefaultDisabled
        }
    };

    let copied_proof_restored = if status
        == SourceIncludeProductionGateStatus::ReadyButDefaultDisabled
    {
        match (dry_run, copied_proof) {
            (Some(dry_run), Some(report))
                if dry_run.status == SourceIncludeSelectedTargetDryRunStatus::Planned =>
            {
                if let Some(target) = dry_run.selected_target.as_ref() {
                    let ready = copied_report_base_ready(report, target)
                        && report.source_include_executor_restored == Some(true);
                    if !ready {
                        blockers.push(
                            "copied-config-tree source/include proof does not match the selected target and planned line"
                                .to_string(),
                        );
                        status = SourceIncludeProductionGateStatus::CopiedProofMismatch;
                    }
                    ready
                } else {
                    blockers.push("selected-target dry-run has no target path".to_string());
                    status = SourceIncludeProductionGateStatus::MissingDryRunPlan;
                    false
                }
            }
            (Some(_), _) => {
                blockers.push("copied-config-tree source/include proof is required".to_string());
                status = SourceIncludeProductionGateStatus::MissingCopiedProof;
                false
            }
            (None, _) => {
                blockers.push("selected-target dry-run plan is required".to_string());
                status = SourceIncludeProductionGateStatus::MissingDryRunPlan;
                false
            }
        }
    } else {
        false
    };

    if status == SourceIncludeProductionGateStatus::ReadyButDefaultDisabled
        && copied_proof_restored
        && production_flag_enabled
    {
        status = SourceIncludeProductionGateStatus::Enabled;
    } else if status == SourceIncludeProductionGateStatus::ReadyButDefaultDisabled
        && copied_proof_restored
        && !production_flag_enabled
    {
        blockers.push("source/include production insertion flag is default-disabled".to_string());
    }

    let production_apply_enabled = status == SourceIncludeProductionGateStatus::Enabled;
    let proposed_value = dry_run
        .and_then(|plan| plan.insertion_line.as_ref())
        .and_then(|line| line.split_once('='))
        .map(|(_, value)| value.trim().to_string());
    let gate = SourceIncludeProductionGate {
        status,
        root_path: dry_run.and_then(|plan| plan.root_path.clone()),
        selected_target_path: dry_run.and_then(|plan| plan.selected_target.clone()),
        source_depth: dry_run.and_then(|plan| plan.source_depth),
        planned_line: dry_run.and_then(|plan| plan.insertion_line.clone()),
        proposed_value,
        copied_proof_restored,
        production_flag_enabled,
        production_apply_enabled,
        blockers: blockers.clone(),
    };
    SourceIncludeProductionReview {
        gate,
        review_lines: vec![
            "Source/include insertion now has copied-config-tree proof as a prerequisite."
                .to_string(),
            "The selected target, source depth, planned line, and proposed value must match the copied proof.".to_string(),
            "Production source/include insertion remains default-disabled until explicit activation review.".to_string(),
            format!("Blockers: {}", if blockers.is_empty() { "none".to_string() } else { blockers.join("; ") }),
        ],
    }
}

pub fn source_include_selected_target_dry_run_plan(
    proof: &SourceIncludeTargetSelectionProof,
    insertion_plan: &MissingDefaultInsertionPlan,
) -> SourceIncludeSelectedTargetDryRunPlan {
    let Some(precondition) = proof.precondition.as_ref() else {
        return source_include_dry_run_blocked(
            SourceIncludeSelectedTargetDryRunStatus::SelectionBlocked,
            None,
            None,
            None,
            vec!["no explicit source/include target selected".to_string()],
        );
    };

    if !proof.fixture_plan_allowed {
        return source_include_dry_run_blocked(
            SourceIncludeSelectedTargetDryRunStatus::SelectionBlocked,
            Some(precondition.root_path.clone()),
            Some(precondition.selected_target.clone()),
            Some(precondition.source_depth),
            vec!["selected source/include target is not eligible for fixture dry-run".to_string()],
        );
    }

    if precondition.selected_target != insertion_plan.target_path {
        return source_include_dry_run_blocked(
            SourceIncludeSelectedTargetDryRunStatus::TargetMismatch,
            Some(precondition.root_path.clone()),
            Some(precondition.selected_target.clone()),
            Some(precondition.source_depth),
            vec!["selected target does not match the insertion plan target".to_string()],
        );
    }

    if !precondition
        .selected_target
        .starts_with(std::env::temp_dir())
    {
        return source_include_dry_run_blocked(
            SourceIncludeSelectedTargetDryRunStatus::NonFixtureTargetRefused,
            Some(precondition.root_path.clone()),
            Some(precondition.selected_target.clone()),
            Some(precondition.source_depth),
            vec![
                "source/include selected-target dry-run accepts temp fixture paths only"
                    .to_string(),
            ],
        );
    }

    if !insertion_plan.can_execute {
        return source_include_dry_run_blocked(
            SourceIncludeSelectedTargetDryRunStatus::InsertionPlanBlocked,
            Some(precondition.root_path.clone()),
            Some(precondition.selected_target.clone()),
            Some(precondition.source_depth),
            insertion_plan.blocked_reasons.clone(),
        );
    }

    SourceIncludeSelectedTargetDryRunPlan {
        status: SourceIncludeSelectedTargetDryRunStatus::Planned,
        root_path: Some(precondition.root_path.clone()),
        selected_target: Some(precondition.selected_target.clone()),
        source_depth: Some(precondition.source_depth),
        insertion_line: Some(insertion_plan.insertion_line.clone()),
        dry_run_preview: Some(format!(
            "Would insert `{}` into {} at source depth {}.",
            insertion_plan.insertion_line,
            precondition.selected_target.display(),
            precondition.source_depth
        )),
        blocked_reasons: Vec::new(),
        production_insertion_enabled: false,
        real_config_touched: false,
        runtime_touched: false,
        review_lines: vec![
            "Source/include selected-target insertion is dry-run proof only.".to_string(),
            "The exact target file and inserted line are shown before any future activation."
                .to_string(),
            "Production source/include insertion remains disabled.".to_string(),
        ],
    }
}

fn source_include_dry_run_blocked(
    status: SourceIncludeSelectedTargetDryRunStatus,
    root_path: Option<PathBuf>,
    selected_target: Option<PathBuf>,
    source_depth: Option<usize>,
    blocked_reasons: Vec<String>,
) -> SourceIncludeSelectedTargetDryRunPlan {
    SourceIncludeSelectedTargetDryRunPlan {
        status,
        root_path,
        selected_target,
        source_depth,
        insertion_line: None,
        dry_run_preview: None,
        blocked_reasons,
        production_insertion_enabled: false,
        real_config_touched: false,
        runtime_touched: false,
        review_lines: vec![
            "Source/include selected-target dry-run is blocked.".to_string(),
            "Production source/include insertion remains disabled.".to_string(),
        ],
    }
}

pub fn disabled_missing_default_insertion_review(
    plan: &MissingDefaultInsertionPlan,
) -> DisabledInsertionReview {
    DisabledInsertionReview {
        setting_id: plan.setting_id.clone(),
        target_path: plan.target_path.clone(),
        proposed_line: plan.insertion_line.clone(),
        production_apply_enabled: false,
        user_copy: "This setting uses Hyprland's default value. Production insertion is limited to reviewed single-file safe-batch targets; this scaffold stays disabled for unsupported layouts.".to_string(),
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

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum DuplicateOccurrenceReviewState {
    NoOccurrenceSelected,
    OccurrenceSelectedProductionDisabled,
    InvalidSelection,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum DuplicateOccurrenceApprovalState {
    Missing,
    Pending,
    Confirmed,
    Rejected,
    Expired,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct DuplicateOccurrenceConfirmation {
    pub setting_id: String,
    pub selected_path: Option<PathBuf>,
    pub selected_line_number: Option<usize>,
    pub selected_raw_line: Option<String>,
    pub occurrence_fingerprint: Option<String>,
    pub approval_state: DuplicateOccurrenceApprovalState,
    pub token_required: bool,
    pub token_matched: bool,
    pub safe_env_replacement_allowed: bool,
    pub production_write_enabled: bool,
    pub apply_enabled: bool,
    pub review_lines: Vec<String>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum DuplicateProductionGateStatus {
    MissingConfirmation,
    PendingConfirmation,
    ConfirmedButProductionDisabled,
    MissingCopiedProof,
    CopiedProofMismatch,
    ReadyButDefaultDisabled,
    Rejected,
    Expired,
    FingerprintMismatch,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct DuplicateOccurrencePrecondition {
    pub path: PathBuf,
    pub line_number: usize,
    pub raw_line: String,
    pub old_value: String,
    pub source_depth: usize,
    pub fingerprint: String,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct DuplicateProductionApprovalGate {
    pub setting_id: String,
    pub status: DuplicateProductionGateStatus,
    pub precondition: Option<DuplicateOccurrencePrecondition>,
    pub safe_env_replacement_allowed: bool,
    pub production_apply_enabled: bool,
    pub duplicate_write_enabled: bool,
    pub block_reason: String,
    pub review_lines: Vec<String>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct DuplicateProductionGateReview {
    pub setting_id: String,
    pub status: DuplicateProductionGateStatus,
    pub selected_path: Option<PathBuf>,
    pub selected_line_number: Option<usize>,
    pub selected_raw_line: Option<String>,
    pub old_value: Option<String>,
    pub proposed_value: Option<String>,
    pub source_depth: Option<usize>,
    pub copied_proof_restored: bool,
    pub production_flag_enabled: bool,
    pub production_apply_enabled: bool,
    pub blockers: Vec<String>,
    pub review_lines: Vec<String>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct DuplicateOccurrenceReview {
    pub setting_id: String,
    pub state: DuplicateOccurrenceReviewState,
    pub selected_path: Option<PathBuf>,
    pub selected_line_number: Option<usize>,
    pub selected_raw_line: Option<String>,
    pub selected_current_value: Option<String>,
    pub proposed_value: Option<String>,
    pub source_depth: Option<usize>,
    pub apply_enabled: bool,
    pub production_write_enabled: bool,
    pub write_execution_attempted: bool,
    pub review_lines: Vec<String>,
}

pub fn duplicate_occurrence_review(
    model: &DuplicateOccurrenceModel,
    selected_index: Option<usize>,
    proposed_value: Option<String>,
) -> DuplicateOccurrenceReview {
    let Some(selected_index) = selected_index else {
        return DuplicateOccurrenceReview {
            setting_id: model.setting_id.clone(),
            state: DuplicateOccurrenceReviewState::NoOccurrenceSelected,
            selected_path: None,
            selected_line_number: None,
            selected_raw_line: None,
            selected_current_value: None,
            proposed_value,
            source_depth: None,
            apply_enabled: false,
            production_write_enabled: false,
            write_execution_attempted: false,
            review_lines: vec![
                "No duplicate occurrence is selected.".to_string(),
                "The app will not auto-choose a duplicate line.".to_string(),
                "Apply remains blocked until manual occurrence targeting is approved.".to_string(),
            ],
        };
    };

    let Some(occurrence) = model.occurrences.get(selected_index) else {
        return DuplicateOccurrenceReview {
            setting_id: model.setting_id.clone(),
            state: DuplicateOccurrenceReviewState::InvalidSelection,
            selected_path: None,
            selected_line_number: None,
            selected_raw_line: None,
            selected_current_value: None,
            proposed_value,
            source_depth: None,
            apply_enabled: false,
            production_write_enabled: false,
            write_execution_attempted: false,
            review_lines: vec![
                "The selected duplicate occurrence is no longer available.".to_string(),
                "Apply remains blocked.".to_string(),
            ],
        };
    };

    DuplicateOccurrenceReview {
        setting_id: model.setting_id.clone(),
        state: DuplicateOccurrenceReviewState::OccurrenceSelectedProductionDisabled,
        selected_path: Some(occurrence.path.clone()),
        selected_line_number: Some(occurrence.line_number),
        selected_raw_line: Some(occurrence.raw_line.clone()),
        selected_current_value: Some(occurrence.raw_value.clone()),
        proposed_value,
        source_depth: Some(occurrence.source_depth),
        apply_enabled: false,
        production_write_enabled: false,
        write_execution_attempted: false,
        review_lines: vec![
            "A duplicate occurrence is selected for review only.".to_string(),
            "Production Apply still will not write duplicate settings.".to_string(),
            "Manual occurrence targeting needs a separate approval gate before activation."
                .to_string(),
        ],
    }
}

pub fn duplicate_occurrence_confirmation(
    occurrence: Option<&DuplicateOccurrence>,
    provided_token: Option<&str>,
    expected_token: &str,
    rejected: bool,
    expired: bool,
) -> DuplicateOccurrenceConfirmation {
    let Some(occurrence) = occurrence else {
        return DuplicateOccurrenceConfirmation {
            setting_id: String::new(),
            selected_path: None,
            selected_line_number: None,
            selected_raw_line: None,
            occurrence_fingerprint: None,
            approval_state: DuplicateOccurrenceApprovalState::Missing,
            token_required: true,
            token_matched: false,
            safe_env_replacement_allowed: false,
            production_write_enabled: false,
            apply_enabled: false,
            review_lines: vec![
                "No duplicate occurrence is selected for confirmation.".to_string(),
                "Apply remains blocked.".to_string(),
            ],
        };
    };

    let token_matched = provided_token == Some(expected_token);
    let approval_state = if expired {
        DuplicateOccurrenceApprovalState::Expired
    } else if rejected {
        DuplicateOccurrenceApprovalState::Rejected
    } else if token_matched {
        DuplicateOccurrenceApprovalState::Confirmed
    } else {
        DuplicateOccurrenceApprovalState::Pending
    };
    let occurrence_fingerprint = format!(
        "{}:{}:{}:{}",
        occurrence.path.display(),
        occurrence.line_number,
        occurrence.raw_line,
        occurrence.raw_value
    );

    DuplicateOccurrenceConfirmation {
        setting_id: occurrence.setting_id.clone(),
        selected_path: Some(occurrence.path.clone()),
        selected_line_number: Some(occurrence.line_number),
        selected_raw_line: Some(occurrence.raw_line.clone()),
        occurrence_fingerprint: Some(occurrence_fingerprint),
        approval_state,
        token_required: true,
        token_matched,
        safe_env_replacement_allowed: approval_state == DuplicateOccurrenceApprovalState::Confirmed,
        production_write_enabled: false,
        apply_enabled: false,
        review_lines: vec![
            "Manual occurrence confirmation is required before a duplicate replacement proof can run.".to_string(),
            "Production Apply remains disabled even after fixture confirmation.".to_string(),
            "The selected path, line number, raw line, and old value must still match at write time.".to_string(),
        ],
    }
}

fn duplicate_occurrence_fingerprint(occurrence: &DuplicateOccurrence) -> String {
    format!(
        "{}:{}:{}:{}",
        occurrence.path.display(),
        occurrence.line_number,
        occurrence.raw_line,
        occurrence.raw_value
    )
}

pub fn duplicate_production_approval_gate(
    occurrence: Option<&DuplicateOccurrence>,
    confirmation: Option<&DuplicateOccurrenceConfirmation>,
) -> DuplicateProductionApprovalGate {
    let precondition = occurrence.map(|occurrence| DuplicateOccurrencePrecondition {
        path: occurrence.path.clone(),
        line_number: occurrence.line_number,
        raw_line: occurrence.raw_line.clone(),
        old_value: occurrence.raw_value.clone(),
        source_depth: occurrence.source_depth,
        fingerprint: duplicate_occurrence_fingerprint(occurrence),
    });

    let status = match (occurrence, confirmation) {
        (None, _) | (_, None) => DuplicateProductionGateStatus::MissingConfirmation,
        (Some(_), Some(confirmation))
            if confirmation.approval_state == DuplicateOccurrenceApprovalState::Rejected =>
        {
            DuplicateProductionGateStatus::Rejected
        }
        (Some(_), Some(confirmation))
            if confirmation.approval_state == DuplicateOccurrenceApprovalState::Expired =>
        {
            DuplicateProductionGateStatus::Expired
        }
        (Some(_), Some(confirmation))
            if confirmation.approval_state != DuplicateOccurrenceApprovalState::Confirmed =>
        {
            DuplicateProductionGateStatus::PendingConfirmation
        }
        (Some(occurrence), Some(confirmation))
            if confirmation.occurrence_fingerprint.as_deref()
                != Some(duplicate_occurrence_fingerprint(occurrence).as_str()) =>
        {
            DuplicateProductionGateStatus::FingerprintMismatch
        }
        (Some(_), Some(_)) => DuplicateProductionGateStatus::ConfirmedButProductionDisabled,
    };

    let block_reason = match status {
        DuplicateProductionGateStatus::MissingConfirmation => {
            "No duplicate occurrence has confirmed target approval.".to_string()
        }
        DuplicateProductionGateStatus::PendingConfirmation => {
            "Duplicate occurrence approval is still pending.".to_string()
        }
        DuplicateProductionGateStatus::ConfirmedButProductionDisabled => {
            "Duplicate occurrence is confirmed for fixture proof, but production duplicate writes remain disabled.".to_string()
        }
        DuplicateProductionGateStatus::MissingCopiedProof => {
            "Copied-config-tree duplicate replacement proof is required.".to_string()
        }
        DuplicateProductionGateStatus::CopiedProofMismatch => {
            "Copied-config-tree duplicate proof does not match the confirmed occurrence.".to_string()
        }
        DuplicateProductionGateStatus::ReadyButDefaultDisabled => {
            "Duplicate occurrence is confirmed and copied proof is restored, but production duplicate writes are default-disabled.".to_string()
        }
        DuplicateProductionGateStatus::Rejected => {
            "Duplicate occurrence approval was rejected.".to_string()
        }
        DuplicateProductionGateStatus::Expired => {
            "Duplicate occurrence approval expired before Apply.".to_string()
        }
        DuplicateProductionGateStatus::FingerprintMismatch => {
            "Duplicate occurrence preconditions no longer match the confirmed target.".to_string()
        }
    };

    DuplicateProductionApprovalGate {
        setting_id: occurrence
            .map(|occurrence| occurrence.setting_id.clone())
            .or_else(|| confirmation.map(|confirmation| confirmation.setting_id.clone()))
            .unwrap_or_default(),
        status,
        precondition,
        safe_env_replacement_allowed: status
            == DuplicateProductionGateStatus::ConfirmedButProductionDisabled,
        production_apply_enabled: false,
        duplicate_write_enabled: false,
        block_reason: block_reason.clone(),
        review_lines: vec![
            "Duplicate writes need explicit occurrence target confirmation.".to_string(),
            block_reason,
            "The app will not choose the first, last, base, or profile value automatically."
                .to_string(),
        ],
    }
}

pub fn duplicate_production_gate_review(
    occurrence: Option<&DuplicateOccurrence>,
    confirmation: Option<&DuplicateOccurrenceConfirmation>,
    copied_proof: Option<&CopiedConfigTreeReport>,
    proposed_value: Option<String>,
    production_flag_enabled: bool,
) -> DuplicateProductionGateReview {
    let base_gate = duplicate_production_approval_gate(occurrence, confirmation);
    let mut status = base_gate.status;
    let mut blockers = vec![base_gate.block_reason.clone()];
    let mut copied_proof_restored = false;

    if status == DuplicateProductionGateStatus::ConfirmedButProductionDisabled {
        let Some(precondition) = base_gate.precondition.as_ref() else {
            status = DuplicateProductionGateStatus::MissingConfirmation;
            blockers.push("duplicate precondition record is missing".to_string());
            return duplicate_gate_review_from_parts(
                base_gate,
                status,
                proposed_value,
                false,
                production_flag_enabled,
                false,
                blockers,
            );
        };
        match copied_proof {
            Some(report) => {
                copied_proof_restored = copied_report_base_ready(report, &precondition.path)
                    && report.duplicate_executor_restored == Some(true);
                if copied_proof_restored {
                    status = if production_flag_enabled {
                        DuplicateProductionGateStatus::ConfirmedButProductionDisabled
                    } else {
                        DuplicateProductionGateStatus::ReadyButDefaultDisabled
                    };
                    if !production_flag_enabled {
                        blockers =
                            vec!["duplicate production write flag is default-disabled".to_string()];
                    } else {
                        blockers =
                            vec!["duplicate production Apply integration is still not wired"
                                .to_string()];
                    }
                } else {
                    status = DuplicateProductionGateStatus::CopiedProofMismatch;
                    blockers.push(
                        "copied-config-tree duplicate proof must restore the selected copied target"
                            .to_string(),
                    );
                }
            }
            None => {
                status = DuplicateProductionGateStatus::MissingCopiedProof;
                blockers.push("copied-config-tree duplicate proof is missing".to_string());
            }
        }
    }

    duplicate_gate_review_from_parts(
        base_gate,
        status,
        proposed_value,
        copied_proof_restored,
        production_flag_enabled,
        false,
        blockers,
    )
}

fn duplicate_gate_review_from_parts(
    base_gate: DuplicateProductionApprovalGate,
    status: DuplicateProductionGateStatus,
    proposed_value: Option<String>,
    copied_proof_restored: bool,
    production_flag_enabled: bool,
    production_apply_enabled: bool,
    blockers: Vec<String>,
) -> DuplicateProductionGateReview {
    let precondition = base_gate.precondition.as_ref();
    DuplicateProductionGateReview {
        setting_id: base_gate.setting_id,
        status,
        selected_path: precondition.map(|precondition| precondition.path.clone()),
        selected_line_number: precondition.map(|precondition| precondition.line_number),
        selected_raw_line: precondition.map(|precondition| precondition.raw_line.clone()),
        old_value: precondition.map(|precondition| precondition.old_value.clone()),
        proposed_value,
        source_depth: precondition.map(|precondition| precondition.source_depth),
        copied_proof_restored,
        production_flag_enabled,
        production_apply_enabled,
        blockers: blockers.clone(),
        review_lines: vec![
            "Duplicate replacement requires confirmed occurrence targeting.".to_string(),
            "Path, line number, raw line, old value, fingerprint, source depth, and copied proof must match.".to_string(),
            "Production duplicate writes remain disabled by default.".to_string(),
            format!(
                "Blockers: {}",
                if blockers.is_empty() {
                    "none".to_string()
                } else {
                    blockers.join("; ")
                }
            ),
        ],
    }
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

pub fn replace_duplicate_occurrence_with_confirmation_safe_env(
    confirmation: &DuplicateOccurrenceConfirmation,
    request: &DuplicateReplacementRequest,
    options: &DuplicateReplacementOptions,
) -> DuplicateReplacementReport {
    let gate = duplicate_production_approval_gate(Some(&request.occurrence), Some(confirmation));
    if gate.status != DuplicateProductionGateStatus::ConfirmedButProductionDisabled {
        return duplicate_failed(gate.block_reason);
    }
    replace_duplicate_occurrence_safe_env(request, options)
}

pub fn execute_duplicate_replacement_guarded_temp(
    confirmation: &DuplicateOccurrenceConfirmation,
    request: &DuplicateReplacementRequest,
    guard: &ControlledLiveTestGuardReview,
    force_verification_failure: bool,
    force_restore_failure: bool,
) -> GuardedTempExecutionReport {
    if !guard.live_mutation_allowed || !guard.real_config_touch_allowed {
        return guarded_temp_blocked(
            request.occurrence.path.clone(),
            "controlled live-test guard did not allow duplicate replacement",
        );
    }
    let gate = duplicate_production_approval_gate(Some(&request.occurrence), Some(confirmation));
    if gate.status != DuplicateProductionGateStatus::ConfirmedButProductionDisabled {
        return guarded_temp_blocked(request.occurrence.path.clone(), &gate.block_reason);
    }
    let key = request
        .occurrence
        .raw_line
        .split_once('=')
        .map(|(key, _)| key.trim().to_string())
        .unwrap_or_default();
    if key.is_empty() || request.occurrence.raw_value.trim() != request.expected_old_value.trim() {
        return guarded_temp_blocked(
            request.occurrence.path.clone(),
            "selected duplicate preconditions do not match",
        );
    }
    let planned_line = format!("{key} = {}", request.proposed_value.trim());
    let line_number = request.occurrence.line_number;
    let expected_raw_line = request.occurrence.raw_line.clone();
    execute_guarded_temp_line_replace(
        request.occurrence.path.clone(),
        line_number,
        expected_raw_line,
        planned_line,
        force_verification_failure,
        force_restore_failure,
    )
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

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct HighRiskRecoveryReview {
    pub setting_id: String,
    pub state: MockWatchdogState,
    pub production_write_enabled: bool,
    pub real_runtime_enabled: bool,
    pub review_lines: Vec<String>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RollbackProofRecord {
    pub backup_before_write_required: bool,
    pub reread_after_write_required: bool,
    pub restore_on_timeout_required: bool,
    pub reread_after_restore_required: bool,
    pub real_runtime_enabled: bool,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct HighRiskRecoveryWorkflow {
    pub setting_id: String,
    pub state: MockWatchdogState,
    pub confirmation_enabled: bool,
    pub revert_enabled: bool,
    pub production_write_enabled: bool,
    pub real_runtime_enabled: bool,
    pub rollback_proof: RollbackProofRecord,
    pub review_lines: Vec<String>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum HighRiskLiveReadinessStatus {
    NoopReadyForReview,
    RealConfigRefused,
    RuntimeMutationRefused,
    RecoveryProofMissing,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct HighRiskLiveRecoveryProtocol {
    pub setting_id: String,
    pub target_path: PathBuf,
    pub status: HighRiskLiveReadinessStatus,
    pub accepts_real_config: bool,
    pub mutating_runtime_enabled: bool,
    pub live_write_enabled: bool,
    pub no_op_harness: bool,
    pub required_manual_steps: Vec<String>,
    pub review_lines: Vec<String>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum HighRiskProductionGateStatus {
    RecoveryMissing,
    DeadManTimeoutMissing,
    RestoreCommandMissing,
    ConfigBackupMissing,
    RuntimeSnapshotMissing,
    ExplicitApprovalMissing,
    ReadinessProofMissing,
    ReadyButDefaultDisabled,
    Enabled,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct HighRiskProductionGateReview {
    pub setting_id: String,
    pub status: HighRiskProductionGateStatus,
    pub readiness_status: Option<HighRiskLiveReadinessStatus>,
    pub out_of_band_recovery_available: bool,
    pub dead_man_timeout_recorded: bool,
    pub restore_command_recorded: bool,
    pub config_backup_recorded: bool,
    pub runtime_snapshot_recorded: bool,
    pub explicit_approval_recorded: bool,
    pub production_flag_enabled: bool,
    pub production_write_enabled: bool,
    pub blockers: Vec<String>,
    pub review_lines: Vec<String>,
}

pub fn high_risk_recovery_review(
    setting_id: &str,
    watchdog: &MockWatchdog,
) -> HighRiskRecoveryReview {
    let state_line = match watchdog.state {
        MockWatchdogState::Pending => "Recovery is pending confirmation or timeout.",
        MockWatchdogState::Confirmed => "The user confirmed the mock recovery session.",
        MockWatchdogState::TimedOut => "The mock recovery session timed out.",
        MockWatchdogState::Reverted => "The mock recovery session reverted the fixture state.",
        MockWatchdogState::RecoveryFailed => "The mock recovery session failed to recover.",
    };
    HighRiskRecoveryReview {
        setting_id: setting_id.to_string(),
        state: watchdog.state,
        production_write_enabled: false,
        real_runtime_enabled: false,
        review_lines: vec![
            "High-risk/display writes need a recovery path before production Apply can use them."
                .to_string(),
            state_line.to_string(),
            "This review is non-mutating and does not reload Hyprland.".to_string(),
        ],
    }
}

pub fn high_risk_live_recovery_protocol(
    setting_id: &str,
    target_path: impl Into<PathBuf>,
    recovery_proof_available: bool,
    runtime_mutation_requested: bool,
) -> HighRiskLiveRecoveryProtocol {
    let target_path = target_path.into();
    let status = if runtime_mutation_requested {
        HighRiskLiveReadinessStatus::RuntimeMutationRefused
    } else if !target_path.starts_with(std::env::temp_dir()) {
        HighRiskLiveReadinessStatus::RealConfigRefused
    } else if !recovery_proof_available {
        HighRiskLiveReadinessStatus::RecoveryProofMissing
    } else {
        HighRiskLiveReadinessStatus::NoopReadyForReview
    };
    HighRiskLiveRecoveryProtocol {
        setting_id: setting_id.to_string(),
        target_path,
        status,
        accepts_real_config: false,
        mutating_runtime_enabled: false,
        live_write_enabled: false,
        no_op_harness: true,
        required_manual_steps: vec![
            "out-of-band recovery channel".to_string(),
            "dead-man timeout confirmation".to_string(),
            "verified backup restore path".to_string(),
            "explicit user approval before any live mutation".to_string(),
        ],
        review_lines: vec![
            "Live high-risk recovery proof is no-op only in this branch.".to_string(),
            "Real config paths and runtime mutation are refused by default.".to_string(),
            "A future sprint must prove recovery outside the graphical session before activation."
                .to_string(),
        ],
    }
}

pub fn high_risk_guarded_live_readiness_executor(
    setting_id: &str,
    target_path: impl Into<PathBuf>,
    guard: &ControlledLiveTestGuardReview,
    dead_man_timeout_recorded: bool,
    restore_command_recorded: bool,
) -> HighRiskLiveRecoveryProtocol {
    let target_path = target_path.into();
    let recovery_proof_available =
        guard.live_mutation_allowed && dead_man_timeout_recorded && restore_command_recorded;
    let runtime_mutation_requested = false;
    let mut protocol = high_risk_live_recovery_protocol(
        setting_id,
        target_path,
        recovery_proof_available,
        runtime_mutation_requested,
    );
    if !dead_man_timeout_recorded {
        protocol
            .required_manual_steps
            .push("dead-man timeout record".to_string());
    }
    if !restore_command_recorded {
        protocol
            .required_manual_steps
            .push("restore command record".to_string());
    }
    protocol.live_write_enabled = false;
    protocol.mutating_runtime_enabled = false;
    protocol
}

pub fn high_risk_production_gate_review(
    protocol: Option<&HighRiskLiveRecoveryProtocol>,
    out_of_band_recovery_available: bool,
    dead_man_timeout_recorded: bool,
    restore_command_recorded: bool,
    config_backup_recorded: bool,
    runtime_snapshot_recorded: bool,
    explicit_approval_recorded: bool,
    production_flag_enabled: bool,
) -> HighRiskProductionGateReview {
    let setting_id = protocol
        .map(|protocol| protocol.setting_id.clone())
        .unwrap_or_default();
    let mut blockers = Vec::new();
    let mut status = HighRiskProductionGateStatus::ReadyButDefaultDisabled;
    if protocol
        .map(|protocol| protocol.status != HighRiskLiveReadinessStatus::NoopReadyForReview)
        .unwrap_or(true)
    {
        status = HighRiskProductionGateStatus::ReadinessProofMissing;
        blockers.push("no-op or copied readiness proof is required".to_string());
    }
    if !out_of_band_recovery_available {
        status = HighRiskProductionGateStatus::RecoveryMissing;
        blockers.push("out-of-band recovery channel is required".to_string());
    }
    if !dead_man_timeout_recorded {
        status = HighRiskProductionGateStatus::DeadManTimeoutMissing;
        blockers.push("dead-man timeout is required".to_string());
    }
    if !restore_command_recorded {
        status = HighRiskProductionGateStatus::RestoreCommandMissing;
        blockers.push("restore command is required".to_string());
    }
    if !config_backup_recorded {
        status = HighRiskProductionGateStatus::ConfigBackupMissing;
        blockers.push("config backup is required".to_string());
    }
    if !runtime_snapshot_recorded {
        status = HighRiskProductionGateStatus::RuntimeSnapshotMissing;
        blockers.push("runtime snapshot is required".to_string());
    }
    if !explicit_approval_recorded {
        status = HighRiskProductionGateStatus::ExplicitApprovalMissing;
        blockers.push("explicit high-risk write approval is required".to_string());
    }
    if status == HighRiskProductionGateStatus::ReadyButDefaultDisabled && production_flag_enabled {
        status = HighRiskProductionGateStatus::Enabled;
    } else if status == HighRiskProductionGateStatus::ReadyButDefaultDisabled {
        blockers.push("high-risk production write flag is default-disabled".to_string());
    }
    let production_write_enabled = status == HighRiskProductionGateStatus::Enabled;
    HighRiskProductionGateReview {
        setting_id,
        status,
        readiness_status: protocol.map(|protocol| protocol.status),
        out_of_band_recovery_available,
        dead_man_timeout_recorded,
        restore_command_recorded,
        config_backup_recorded,
        runtime_snapshot_recorded,
        explicit_approval_recorded,
        production_flag_enabled,
        production_write_enabled,
        blockers: blockers.clone(),
        review_lines: vec![
            "High-risk/display writes require an out-of-band recovery path before any live mutation.".to_string(),
            "Dead-man timeout, restore command, config backup, runtime snapshot, and explicit approval are mandatory.".to_string(),
            "Production high-risk writes remain default-disabled.".to_string(),
            format!(
                "Blockers: {}",
                if blockers.is_empty() {
                    "none".to_string()
                } else {
                    blockers.join("; ")
                }
            ),
        ],
    }
}

fn structured_bind_blocked(
    target_path: PathBuf,
    line_number: usize,
    expected_old_line: &str,
    proposed_new_line: &str,
    error: &str,
) -> StructuredBindEditProof {
    StructuredBindEditProof {
        status: StructuredBindEditStatus::Blocked,
        target_path,
        line_number,
        expected_old_line: expected_old_line.to_string(),
        proposed_new_line: proposed_new_line.to_string(),
        rendered_line: None,
        comments_and_order_preserved: false,
        reread_verified: false,
        production_write_enabled: false,
        real_config_touched: false,
        errors: vec![error.to_string()],
    }
}

pub fn high_risk_recovery_workflow(
    setting_id: &str,
    watchdog: &MockWatchdog,
) -> HighRiskRecoveryWorkflow {
    let state_copy = match watchdog.state {
        MockWatchdogState::Pending => "A future high-risk Apply would wait for confirmation.",
        MockWatchdogState::Confirmed => "The mock session was confirmed.",
        MockWatchdogState::TimedOut => "The mock session timed out before confirmation.",
        MockWatchdogState::Reverted => "The mock session restored the fixture state.",
        MockWatchdogState::RecoveryFailed => {
            "The mock session recorded a recovery failure that would block production activation."
        }
    };

    HighRiskRecoveryWorkflow {
        setting_id: setting_id.to_string(),
        state: watchdog.state,
        confirmation_enabled: false,
        revert_enabled: false,
        production_write_enabled: false,
        real_runtime_enabled: false,
        rollback_proof: RollbackProofRecord {
            backup_before_write_required: true,
            reread_after_write_required: true,
            restore_on_timeout_required: true,
            reread_after_restore_required: true,
            real_runtime_enabled: false,
        },
        review_lines: vec![
            "High-risk and display/render writes remain blocked.".to_string(),
            state_copy.to_string(),
            "This workflow records the recovery contract without writing config or reloading Hyprland."
                .to_string(),
        ],
    }
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

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct StructuredEditCandidate {
    pub family_id: String,
    pub proposed_raw_line: String,
    pub accepted: bool,
    pub production_write_enabled: bool,
    pub lossless_render_required: bool,
    pub errors: Vec<String>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct StructuredFamilyReview {
    pub family_id: String,
    pub entries: Vec<StructuredFamilyEntry>,
    pub proposed_edit: Option<StructuredEditCandidate>,
    pub editor_enabled: bool,
    pub production_write_enabled: bool,
    pub raw_line_preservation_required: bool,
    pub comments_order_preservation_required: bool,
    pub invalid_input_reasons: Vec<String>,
    pub first_safe_env_write_candidate: Option<String>,
    pub review_lines: Vec<String>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum StructuredBindEditStatus {
    Succeeded,
    Blocked,
    VerificationFailed,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct StructuredBindEditProof {
    pub status: StructuredBindEditStatus,
    pub target_path: PathBuf,
    pub line_number: usize,
    pub expected_old_line: String,
    pub proposed_new_line: String,
    pub rendered_line: Option<String>,
    pub comments_and_order_preserved: bool,
    pub reread_verified: bool,
    pub production_write_enabled: bool,
    pub real_config_touched: bool,
    pub errors: Vec<String>,
}

pub fn validate_structured_edit_candidate(
    family_id: &str,
    proposed_raw_line: &str,
) -> StructuredEditCandidate {
    let trimmed = proposed_raw_line.trim();
    let mut errors = Vec::new();
    if trimmed.is_empty() {
        errors.push("structured entries cannot be blank".to_string());
    }
    if proposed_raw_line.contains('\n') || proposed_raw_line.contains('\r') {
        errors.push(
            "structured entries must be single-line until a lossless editor exists".to_string(),
        );
    }
    let expected_prefix = match family_id {
        "hl.bind" => "bind",
        "hl.monitor" => "monitor",
        "hl.windowrule" => "windowrule",
        "hl.animation" => "animation",
        "hl.curve" => "bezier",
        "hl.gesture" => "gesture",
        "hl.permission" => "permission",
        "hl.device" => "device",
        _ => "",
    };
    if expected_prefix.is_empty() {
        errors.push("unknown structured family".to_string());
    } else if !trimmed.starts_with(expected_prefix) {
        errors.push(format!(
            "structured entry for {family_id} must start with {expected_prefix}"
        ));
    }

    StructuredEditCandidate {
        family_id: family_id.to_string(),
        proposed_raw_line: proposed_raw_line.to_string(),
        accepted: errors.is_empty(),
        production_write_enabled: false,
        lossless_render_required: true,
        errors,
    }
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

pub fn structured_family_review(
    model: &StructuredFamilyModel,
    proposed_raw_line: Option<&str>,
) -> StructuredFamilyReview {
    let proposed_edit = proposed_raw_line
        .map(|raw_line| validate_structured_edit_candidate(&model.family_id, raw_line));
    let invalid_input_reasons = proposed_edit
        .as_ref()
        .map(|candidate| candidate.errors.clone())
        .unwrap_or_default();

    StructuredFamilyReview {
        family_id: model.family_id.clone(),
        entries: model.entries.clone(),
        proposed_edit,
        editor_enabled: false,
        production_write_enabled: false,
        raw_line_preservation_required: true,
        comments_order_preservation_required: true,
        invalid_input_reasons,
        first_safe_env_write_candidate: if model.family_id == "hl.bind" {
            Some("hl.bind single-line replacement after lossless render proof".to_string())
        } else {
            None
        },
        review_lines: vec![
            "Structured settings are shown read-only while lossless editing is designed."
                .to_string(),
            "Raw lines, comments, and ordering must be preserved before writes can be enabled."
                .to_string(),
            "Production structured writes remain blocked.".to_string(),
        ],
    }
}

pub fn render_structured_entry_lossless(entry: &StructuredFamilyEntry) -> String {
    entry.raw_line.clone()
}

pub fn edit_structured_bind_safe_env(
    target_path: impl AsRef<Path>,
    line_number: usize,
    expected_old_line: &str,
    proposed_new_line: &str,
) -> StructuredBindEditProof {
    let target_path = target_path.as_ref().to_path_buf();
    if !target_path.starts_with(std::env::temp_dir()) {
        return structured_bind_blocked(
            target_path,
            line_number,
            expected_old_line,
            proposed_new_line,
            "structured bind proof only accepts temp fixture paths",
        );
    }
    let candidate = validate_structured_edit_candidate("hl.bind", proposed_new_line);
    if !candidate.accepted {
        return structured_bind_blocked(
            target_path,
            line_number,
            expected_old_line,
            proposed_new_line,
            &candidate.errors.join("; "),
        );
    }
    let original = match fs::read_to_string(&target_path) {
        Ok(contents) => contents,
        Err(error) => {
            return structured_bind_blocked(
                target_path,
                line_number,
                expected_old_line,
                proposed_new_line,
                &format!("read target failed: {error}"),
            )
        }
    };
    let mut lines: Vec<String> = original.lines().map(str::to_string).collect();
    let Some(line) = lines.get_mut(line_number.saturating_sub(1)) else {
        return structured_bind_blocked(
            target_path,
            line_number,
            expected_old_line,
            proposed_new_line,
            "line number is outside target file",
        );
    };
    if line.trim() != expected_old_line.trim() {
        return structured_bind_blocked(
            target_path,
            line_number,
            expected_old_line,
            proposed_new_line,
            "expected old structured line no longer matches",
        );
    }
    *line = proposed_new_line.trim().to_string();
    let mut updated = lines.join("\n");
    if original.ends_with('\n') {
        updated.push('\n');
    }
    if let Err(error) = fs::write(&target_path, updated.as_bytes()) {
        return structured_bind_blocked(
            target_path,
            line_number,
            expected_old_line,
            proposed_new_line,
            &format!("write target failed: {error}"),
        );
    }
    let verified = parse_hyprland_config_file(&target_path)
        .map(|parsed| {
            parsed.records.iter().any(|record| {
                record.line_number == line_number
                    && record.status == ParseStatus::StructuredRaw
                    && record.normalized_setting_id.as_deref() == Some("hl.bind")
                    && record.raw_line.trim() == proposed_new_line.trim()
            })
        })
        .unwrap_or(false);
    if !verified {
        return StructuredBindEditProof {
            status: StructuredBindEditStatus::VerificationFailed,
            target_path,
            line_number,
            expected_old_line: expected_old_line.to_string(),
            proposed_new_line: proposed_new_line.to_string(),
            rendered_line: None,
            comments_and_order_preserved: false,
            reread_verified: false,
            production_write_enabled: false,
            real_config_touched: false,
            errors: vec!["structured bind reread verification failed".to_string()],
        };
    }

    StructuredBindEditProof {
        status: StructuredBindEditStatus::Succeeded,
        target_path,
        line_number,
        expected_old_line: expected_old_line.to_string(),
        proposed_new_line: proposed_new_line.to_string(),
        rendered_line: Some(proposed_new_line.trim().to_string()),
        comments_and_order_preserved: true,
        reread_verified: true,
        production_write_enabled: false,
        real_config_touched: false,
        errors: Vec::new(),
    }
}

pub fn execute_structured_bind_guarded_temp(
    target_path: impl AsRef<Path>,
    line_number: usize,
    expected_old_line: &str,
    proposed_new_line: &str,
    guard: &ControlledLiveTestGuardReview,
    force_verification_failure: bool,
    force_restore_failure: bool,
) -> GuardedTempExecutionReport {
    let target_path = target_path.as_ref().to_path_buf();
    if !guard.live_mutation_allowed || !guard.real_config_touch_allowed {
        return guarded_temp_blocked(
            target_path,
            "controlled live-test guard did not allow structured write",
        );
    }
    let candidate = validate_structured_edit_candidate("hl.bind", proposed_new_line);
    if !candidate.accepted {
        return guarded_temp_blocked(target_path, &candidate.errors.join("; "));
    }
    execute_guarded_temp_line_replace(
        target_path,
        line_number,
        expected_old_line.to_string(),
        proposed_new_line.trim().to_string(),
        force_verification_failure,
        force_restore_failure,
    )
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum StructuredProductionGateStatus {
    InvalidFamily,
    InvalidCandidate,
    MissingSelectedLine,
    MissingCopiedProof,
    CopiedProofMismatch,
    ReadyButDefaultDisabled,
    Enabled,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct StructuredProductionGateReview {
    pub family_id: String,
    pub status: StructuredProductionGateStatus,
    pub target_path: PathBuf,
    pub line_number: usize,
    pub old_raw_line: String,
    pub new_raw_line: String,
    pub copied_proof_restored: bool,
    pub comments_order_preserved: bool,
    pub production_flag_enabled: bool,
    pub production_apply_enabled: bool,
    pub blockers: Vec<String>,
    pub review_lines: Vec<String>,
}

pub fn structured_production_gate_review(
    family_id: &str,
    target_path: impl Into<PathBuf>,
    line_number: usize,
    old_raw_line: &str,
    new_raw_line: &str,
    copied_proof: Option<&CopiedConfigTreeReport>,
    comments_order_preserved: bool,
    production_flag_enabled: bool,
) -> StructuredProductionGateReview {
    let target_path = target_path.into();
    let mut blockers = Vec::new();
    let mut status = StructuredProductionGateStatus::ReadyButDefaultDisabled;
    if family_id != "hl.bind" {
        status = StructuredProductionGateStatus::InvalidFamily;
        blockers.push("this production gate currently accepts hl.bind only".to_string());
    }
    if old_raw_line.trim().is_empty() || line_number == 0 {
        status = StructuredProductionGateStatus::MissingSelectedLine;
        blockers.push("exact selected structured line and line number are required".to_string());
    }
    let candidate = validate_structured_edit_candidate(family_id, new_raw_line);
    if !candidate.accepted {
        status = StructuredProductionGateStatus::InvalidCandidate;
        blockers.extend(candidate.errors);
    }
    if !comments_order_preserved {
        status = StructuredProductionGateStatus::CopiedProofMismatch;
        blockers.push("comments/order preservation proof is required".to_string());
    }

    let copied_proof_restored = match copied_proof {
        Some(report) => {
            copied_report_base_ready(report, &target_path)
                && report.structured_executor_restored == Some(true)
        }
        None => false,
    };
    if status == StructuredProductionGateStatus::ReadyButDefaultDisabled && !copied_proof_restored {
        status = if copied_proof.is_some() {
            StructuredProductionGateStatus::CopiedProofMismatch
        } else {
            StructuredProductionGateStatus::MissingCopiedProof
        };
        blockers.push("restored copied-config-tree hl.bind proof is required".to_string());
    }
    if status == StructuredProductionGateStatus::ReadyButDefaultDisabled && production_flag_enabled
    {
        status = StructuredProductionGateStatus::Enabled;
    } else if status == StructuredProductionGateStatus::ReadyButDefaultDisabled {
        blockers.push("structured production write flag is default-disabled".to_string());
    }

    let production_apply_enabled = status == StructuredProductionGateStatus::Enabled;
    StructuredProductionGateReview {
        family_id: family_id.to_string(),
        status,
        target_path,
        line_number,
        old_raw_line: old_raw_line.to_string(),
        new_raw_line: new_raw_line.to_string(),
        copied_proof_restored,
        comments_order_preserved,
        production_flag_enabled,
        production_apply_enabled,
        blockers: blockers.clone(),
        review_lines: vec![
            "Structured production writes currently review hl.bind only.".to_string(),
            "Exact old line, new line, target file, line number, candidate validation, and copied proof are required.".to_string(),
            "Production structured writes remain default-disabled.".to_string(),
            format!(
                "Blockers: {}",
                if blockers.is_empty() {
                    "none".to_string()
                } else {
                    blockers.join("; ")
                }
            ),
        ],
    }
}

fn execute_guarded_temp_line_mutation(
    target_path: PathBuf,
    planned_line: String,
    appended_text: String,
    verify: impl Fn(&str) -> bool,
    force_verification_failure: bool,
    force_restore_failure: bool,
) -> GuardedTempExecutionReport {
    if !target_path.starts_with(std::env::temp_dir()) {
        return guarded_temp_blocked(
            target_path,
            "guarded executor accepts temp fixture paths only",
        );
    }
    let original = match fs::read(&target_path) {
        Ok(bytes) => bytes,
        Err(error) => {
            return guarded_temp_blocked(target_path, &format!("read target failed: {error}"))
        }
    };
    let original_hash = content_fingerprint(&original);
    let backup_path = target_path.with_extension("guarded-temp-backup");
    if let Err(error) = fs::write(&backup_path, &original) {
        return guarded_temp_blocked(target_path, &format!("backup write failed: {error}"));
    }
    let mut updated = String::from_utf8_lossy(&original).into_owned();
    if !updated.ends_with('\n') {
        updated.push('\n');
    }
    updated.push_str(&appended_text);
    if let Err(error) = fs::write(&target_path, updated.as_bytes()) {
        return guarded_temp_blocked(target_path, &format!("mutation write failed: {error}"));
    }
    let mutation_verified = !force_verification_failure
        && fs::read_to_string(&target_path)
            .map(|contents| verify(&contents))
            .unwrap_or(false);
    restore_guarded_temp(
        target_path,
        Some(backup_path),
        original,
        original_hash,
        Some(planned_line),
        mutation_verified,
        force_restore_failure,
    )
}

fn execute_guarded_temp_line_replace(
    target_path: PathBuf,
    line_number: usize,
    expected_old_line: String,
    planned_line: String,
    force_verification_failure: bool,
    force_restore_failure: bool,
) -> GuardedTempExecutionReport {
    if !target_path.starts_with(std::env::temp_dir()) {
        return guarded_temp_blocked(
            target_path,
            "guarded executor accepts temp fixture paths only",
        );
    }
    let original = match fs::read(&target_path) {
        Ok(bytes) => bytes,
        Err(error) => {
            return guarded_temp_blocked(target_path, &format!("read target failed: {error}"))
        }
    };
    let original_hash = content_fingerprint(&original);
    let backup_path = target_path.with_extension("guarded-temp-backup");
    if let Err(error) = fs::write(&backup_path, &original) {
        return guarded_temp_blocked(target_path, &format!("backup write failed: {error}"));
    }
    let original_text = String::from_utf8_lossy(&original).into_owned();
    let mut lines: Vec<String> = original_text.lines().map(str::to_string).collect();
    let Some(line) = lines.get_mut(line_number.saturating_sub(1)) else {
        return guarded_temp_blocked(target_path, "line number is outside target file");
    };
    if line.trim() != expected_old_line.trim() {
        return guarded_temp_blocked(target_path, "expected old line no longer matches");
    }
    *line = planned_line.clone();
    let mut updated = lines.join("\n");
    if original_text.ends_with('\n') {
        updated.push('\n');
    }
    if let Err(error) = fs::write(&target_path, updated.as_bytes()) {
        return guarded_temp_blocked(target_path, &format!("mutation write failed: {error}"));
    }
    let mutation_verified = !force_verification_failure
        && fs::read_to_string(&target_path)
            .map(|contents| {
                contents
                    .lines()
                    .nth(line_number.saturating_sub(1))
                    .map(|line| line.trim() == planned_line.trim())
                    .unwrap_or(false)
            })
            .unwrap_or(false);
    restore_guarded_temp(
        target_path,
        Some(backup_path),
        original,
        original_hash,
        Some(planned_line),
        mutation_verified,
        force_restore_failure,
    )
}

fn restore_guarded_temp(
    target_path: PathBuf,
    backup_path: Option<PathBuf>,
    original: Vec<u8>,
    original_hash: String,
    planned_line: Option<String>,
    mutation_verified: bool,
    force_restore_failure: bool,
) -> GuardedTempExecutionReport {
    let restore_attempted = true;
    let restore_succeeded = if force_restore_failure {
        false
    } else {
        fs::write(&target_path, &original)
            .and_then(|_| fs::read(&target_path))
            .map(|bytes| content_fingerprint(&bytes) == original_hash)
            .unwrap_or(false)
    };
    let restored_hash = fs::read(&target_path)
        .ok()
        .map(|bytes| content_fingerprint(&bytes));
    GuardedTempExecutionReport {
        status: if restore_succeeded {
            if mutation_verified {
                GuardedTempExecutionStatus::SucceededAndRestored
            } else {
                GuardedTempExecutionStatus::VerificationFailedRestored
            }
        } else {
            GuardedTempExecutionStatus::RestoreFailed
        },
        target_path,
        backup_path,
        original_hash: Some(original_hash),
        restored_hash,
        planned_line,
        mutation_verified,
        restore_attempted,
        restore_succeeded,
        production_write_enabled: false,
        real_config_touched: false,
        runtime_touched: false,
        errors: if restore_succeeded {
            Vec::new()
        } else {
            vec!["restore verification failed".to_string()]
        },
    }
}

fn guarded_temp_blocked(target_path: PathBuf, error: &str) -> GuardedTempExecutionReport {
    GuardedTempExecutionReport {
        status: GuardedTempExecutionStatus::Blocked,
        target_path,
        backup_path: None,
        original_hash: None,
        restored_hash: None,
        planned_line: None,
        mutation_verified: false,
        restore_attempted: false,
        restore_succeeded: false,
        production_write_enabled: false,
        real_config_touched: false,
        runtime_touched: false,
        errors: vec![error.to_string()],
    }
}

fn content_fingerprint(bytes: &[u8]) -> String {
    let mut hasher = DefaultHasher::new();
    bytes.hash(&mut hasher);
    format!("sha256-fixture-fingerprint:{:016x}", hasher.finish())
}

fn symlink_fingerprint(target: Option<&PathBuf>) -> String {
    target
        .map(|target| format!("symlink-target:{}", target.display()))
        .unwrap_or_else(|| "symlink-target:none".to_string())
}

fn path_symlink_fingerprint(path: &Path) -> String {
    fs::read_link(path)
        .ok()
        .map(|target| {
            if target.is_absolute() {
                target
            } else {
                path.parent()
                    .map(|parent| parent.join(target))
                    .unwrap_or_else(|| PathBuf::from("."))
            }
        })
        .as_ref()
        .map(|target| format!("symlink-target:{}", target.display()))
        .unwrap_or_else(|| "symlink-target:none".to_string())
}

fn map_original_path_to_copy(path: &Path, root_parent: &Path, copy_root: &Path) -> PathBuf {
    if let Ok(relative) = path.strip_prefix(root_parent) {
        copy_root.join(relative)
    } else {
        copy_root.join(
            path.file_name()
                .and_then(|name| name.to_str())
                .unwrap_or("copied-config-file"),
        )
    }
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

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ProfileSwitchReview {
    pub current_profile: Option<PathBuf>,
    pub target_profile: PathBuf,
    pub symlink_path: PathBuf,
    pub production_switch_enabled: bool,
    pub reload_after_switch_enabled: bool,
    pub review_lines: Vec<String>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ProfileSwitchSelectionReview {
    pub symlink_path: PathBuf,
    pub current_profile: Option<PathBuf>,
    pub resolved_current_target: Option<PathBuf>,
    pub selected_target_profile: Option<PathBuf>,
    pub confirmation_enabled: bool,
    pub production_switch_enabled: bool,
    pub reload_after_switch_enabled: bool,
    pub review_lines: Vec<String>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ProfileTargetReadiness {
    NoSelection,
    TargetMissing,
    TargetOutsideSafeEnv,
    SafeEnvReviewOnly,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ProfileTargetApprovalReview {
    pub selected_target_profile: Option<PathBuf>,
    pub readiness: ProfileTargetReadiness,
    pub production_switch_enabled: bool,
    pub real_session_allowed: bool,
    pub review_lines: Vec<String>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ProfileProductionGateStatus {
    NoSelection,
    MissingTarget,
    MissingSymlinkSnapshot,
    MissingCopiedProof,
    ReadyButDefaultDisabled,
    Enabled,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ProfileProductionGateReview {
    pub status: ProfileProductionGateStatus,
    pub symlink_path: PathBuf,
    pub original_symlink_target: Option<PathBuf>,
    pub selected_target_profile: Option<PathBuf>,
    pub copied_proof_restored: bool,
    pub real_session_live_proof_required: bool,
    pub production_flag_enabled: bool,
    pub production_switch_enabled: bool,
    pub blockers: Vec<String>,
    pub review_lines: Vec<String>,
}

pub fn disabled_profile_switch_review(
    symlink_path: impl Into<PathBuf>,
    current_profile: Option<PathBuf>,
    target_profile: impl Into<PathBuf>,
) -> ProfileSwitchReview {
    ProfileSwitchReview {
        current_profile,
        target_profile: target_profile.into(),
        symlink_path: symlink_path.into(),
        production_switch_enabled: false,
        reload_after_switch_enabled: false,
        review_lines: vec![
            "Profile switching is not active yet.".to_string(),
            "The safe-env proof can switch and restore temp symlinks only.".to_string(),
            "Real profile files, symlinks, scripts, and Hyprland reload stay blocked.".to_string(),
        ],
    }
}

pub fn disabled_profile_switch_selection_review(
    symlink_path: impl Into<PathBuf>,
    current_profile: Option<PathBuf>,
    resolved_current_target: Option<PathBuf>,
    selected_target_profile: Option<PathBuf>,
) -> ProfileSwitchSelectionReview {
    let selected_copy = if selected_target_profile.is_some() {
        "A target profile is selected for review only."
    } else {
        "No target profile is selected."
    };
    ProfileSwitchSelectionReview {
        symlink_path: symlink_path.into(),
        current_profile,
        resolved_current_target,
        selected_target_profile,
        confirmation_enabled: false,
        production_switch_enabled: false,
        reload_after_switch_enabled: false,
        review_lines: vec![
            selected_copy.to_string(),
            "Profile switching remains disabled for the real session.".to_string(),
            "Safe-env proof may switch and restore temp symlinks only.".to_string(),
        ],
    }
}

pub fn profile_target_approval_review(
    safe_env_root: impl AsRef<Path>,
    selected_target_profile: Option<PathBuf>,
) -> ProfileTargetApprovalReview {
    let safe_env_root = safe_env_root.as_ref();
    let readiness = match selected_target_profile.as_ref() {
        None => ProfileTargetReadiness::NoSelection,
        Some(path)
            if !path.starts_with(safe_env_root)
                || !safe_env_root.starts_with(std::env::temp_dir()) =>
        {
            ProfileTargetReadiness::TargetOutsideSafeEnv
        }
        Some(path) if !path.exists() => ProfileTargetReadiness::TargetMissing,
        Some(_) => ProfileTargetReadiness::SafeEnvReviewOnly,
    };
    ProfileTargetApprovalReview {
        selected_target_profile,
        readiness,
        production_switch_enabled: false,
        real_session_allowed: false,
        review_lines: vec![
            "Profile target review is safe-env only.".to_string(),
            "Real profile symlinks and scripts stay blocked.".to_string(),
            "Hyprland reload is not part of this review.".to_string(),
        ],
    }
}

pub fn profile_production_gate_review(
    symlink_path: impl Into<PathBuf>,
    original_symlink_target: Option<PathBuf>,
    selected_target_profile: Option<PathBuf>,
    copied_proof: Option<&CopiedConfigTreeReport>,
    production_flag_enabled: bool,
) -> ProfileProductionGateReview {
    let symlink_path = symlink_path.into();
    let mut blockers = Vec::new();
    let mut status = ProfileProductionGateStatus::ReadyButDefaultDisabled;
    if selected_target_profile.is_none() {
        status = ProfileProductionGateStatus::NoSelection;
        blockers.push("selected profile target is required".to_string());
    }
    if selected_target_profile
        .as_ref()
        .map(|target| !target.exists())
        .unwrap_or(false)
    {
        status = ProfileProductionGateStatus::MissingTarget;
        blockers.push("selected profile target must exist in the copied tree".to_string());
    }
    if original_symlink_target.is_none() {
        status = ProfileProductionGateStatus::MissingSymlinkSnapshot;
        blockers.push("original symlink target snapshot is required".to_string());
    }
    let copied_proof_restored = copied_proof
        .map(|report| {
            copied_report_base_ready(report, &symlink_path)
                && report.profile_executor_restored == Some(true)
        })
        .unwrap_or(false);
    if status == ProfileProductionGateStatus::ReadyButDefaultDisabled && !copied_proof_restored {
        status = ProfileProductionGateStatus::MissingCopiedProof;
        blockers.push("restored copied-config-tree profile symlink proof is required".to_string());
    }
    if status == ProfileProductionGateStatus::ReadyButDefaultDisabled && production_flag_enabled {
        status = ProfileProductionGateStatus::Enabled;
    } else if status == ProfileProductionGateStatus::ReadyButDefaultDisabled {
        blockers.push("profile production switch flag is default-disabled".to_string());
    }
    let production_switch_enabled = status == ProfileProductionGateStatus::Enabled;
    ProfileProductionGateReview {
        status,
        symlink_path,
        original_symlink_target,
        selected_target_profile,
        copied_proof_restored,
        real_session_live_proof_required: true,
        production_flag_enabled,
        production_switch_enabled,
        blockers: blockers.clone(),
        review_lines: vec![
            "Profile/mode switching requires copied symlink proof and separate real-session live proof.".to_string(),
            "The original symlink target must be restored exactly before any production activation.".to_string(),
            "Production profile switching remains default-disabled.".to_string(),
            format!(
                "Blockers: {}",
                if blockers.is_empty() {
                    "none".to_string()
                } else {
                    blockers.join("; ")
                }
            ),
        ],
    }
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

#[cfg(unix)]
pub fn switch_profile_symlink_guarded_temp(
    root: impl AsRef<Path>,
    current_symlink: impl AsRef<Path>,
    target_profile: impl AsRef<Path>,
    guard: &ControlledLiveTestGuardReview,
    force_restore_failure: bool,
) -> ProfileSwitchReport {
    if !guard.live_mutation_allowed || !guard.real_config_touch_allowed {
        return profile_switch_blocked(
            "controlled live-test guard did not allow profile switching",
        );
    }
    switch_profile_symlink_safe_env(root, current_symlink, target_profile, force_restore_failure)
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

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RuntimeActionPolicy {
    pub action: RuntimeAction,
    pub allowlisted_for_real_execution: bool,
    pub dry_run_allowed: bool,
    pub production_runtime_enabled: bool,
    pub reason: String,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum RuntimeCommandRisk {
    ReadOnlyStatus,
    MutatingReload,
    MutatingKeyword,
    MutatingDispatch,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RuntimeActionReview {
    pub action: RuntimeAction,
    pub policy: RuntimeActionPolicy,
    pub dry_run_result: RuntimeDryRunResult,
    pub execution_log: Vec<String>,
    pub production_execution_enabled: bool,
    pub real_command_executed: bool,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RuntimeGuardedExecutionReport {
    pub action: RuntimeAction,
    pub guard_allowed: bool,
    pub restore_command: Option<String>,
    pub real_command_executed: bool,
    pub production_runtime_enabled: bool,
    pub runtime_touched: bool,
    pub execution_log: Vec<String>,
    pub errors: Vec<String>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum RuntimeProductionGateStatus {
    ReadOnlyEvidenceMissing,
    PriorValueSnapshotMissing,
    RestoreCommandMissing,
    RecoveryPlanMissing,
    ReadyButDefaultDisabled,
    Enabled,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RuntimeProductionGateReview {
    pub action: RuntimeAction,
    pub status: RuntimeProductionGateStatus,
    pub read_only_evidence_available: bool,
    pub restore_command: Option<String>,
    pub explicit_approval_recorded: bool,
    pub production_flag_enabled: bool,
    pub production_runtime_enabled: bool,
    pub blockers: Vec<String>,
    pub review_lines: Vec<String>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum RuntimeSocketDiagnosisStatus {
    HyprctlReadOnlySucceeded,
    HyprctlSocketTimeout,
    WrongInstanceSignature,
    StaleSocket,
    RawSocketSucceededHyprctlFailed,
    PermissionMismatch,
    HyprlandProcessMissing,
    RuntimeEnvMismatch,
    UnknownFailure,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RuntimeSocketCandidate {
    pub signature: String,
    pub socket_path: PathBuf,
    pub exists: bool,
    pub hyprctl_version_succeeded: bool,
    pub raw_socket_succeeded: bool,
    pub error: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RuntimeDirectIpcReadOnlyEvidence {
    pub socket_path: PathBuf,
    pub attempted: bool,
    pub succeeded: bool,
    pub error: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RuntimeReadOnlyEvidence {
    pub hyprctl_binary_path: Option<PathBuf>,
    pub instance_signature: Option<String>,
    pub xdg_runtime_dir: Option<PathBuf>,
    pub version_succeeded: bool,
    pub monitors_json_succeeded: bool,
    pub gaps_in_succeeded: bool,
    pub gaps_out_succeeded: bool,
    pub blur_enabled_succeeded: bool,
    pub logo_disabled_succeeded: bool,
    pub raw_error_text: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RuntimeSocketDiagnosis {
    pub status: RuntimeSocketDiagnosisStatus,
    pub read_only_evidence: RuntimeReadOnlyEvidence,
    pub candidates: Vec<RuntimeSocketCandidate>,
    pub direct_ipc: RuntimeDirectIpcReadOnlyEvidence,
    pub hyprland_process_visible: bool,
    pub process_env_matches_shell: bool,
    pub root_cause: String,
    pub mutation_allowed: bool,
    pub blockers: Vec<String>,
}

pub fn runtime_socket_diagnosis(
    read_only_evidence: RuntimeReadOnlyEvidence,
    candidates: Vec<RuntimeSocketCandidate>,
    direct_ipc: RuntimeDirectIpcReadOnlyEvidence,
    hyprland_process_visible: bool,
    process_env_matches_shell: bool,
) -> RuntimeSocketDiagnosis {
    let all_hyprctl_readonly = read_only_evidence.version_succeeded
        && read_only_evidence.monitors_json_succeeded
        && read_only_evidence.gaps_in_succeeded;
    let mut blockers = Vec::new();
    let (status, root_cause) = if all_hyprctl_readonly {
        (
            RuntimeSocketDiagnosisStatus::HyprctlReadOnlySucceeded,
            "hyprctl read-only evidence succeeded for the selected runtime shell".to_string(),
        )
    } else if !hyprland_process_visible {
        blockers.push("Hyprland process is not visible from this execution context".to_string());
        if direct_ipc
            .error
            .as_deref()
            .map(|error| error.contains("Operation not permitted"))
            .unwrap_or(false)
        {
            (
                RuntimeSocketDiagnosisStatus::PermissionMismatch,
                "sandbox or permission boundary prevents direct Unix socket access".to_string(),
            )
        } else {
            (
                RuntimeSocketDiagnosisStatus::HyprlandProcessMissing,
                "Hyprland process is missing or hidden from the current process namespace"
                    .to_string(),
            )
        }
    } else if !process_env_matches_shell {
        blockers.push("Hyprland process environment does not match shell environment".to_string());
        (
            RuntimeSocketDiagnosisStatus::RuntimeEnvMismatch,
            "shell runtime variables do not match the Hyprland process environment".to_string(),
        )
    } else if direct_ipc.succeeded && !all_hyprctl_readonly {
        blockers.push("raw IPC succeeded while hyprctl read-only queries failed".to_string());
        (
            RuntimeSocketDiagnosisStatus::RawSocketSucceededHyprctlFailed,
            "raw read-only IPC works but hyprctl failed".to_string(),
        )
    } else if candidates
        .iter()
        .any(|candidate| candidate.raw_socket_succeeded)
        && !candidates
            .iter()
            .any(|candidate| candidate.hyprctl_version_succeeded)
    {
        blockers.push("candidate signature did not match a working hyprctl instance".to_string());
        (
            RuntimeSocketDiagnosisStatus::WrongInstanceSignature,
            "socket candidate exists but no matching hyprctl instance succeeded".to_string(),
        )
    } else if read_only_evidence
        .raw_error_text
        .as_deref()
        .map(|error| error.contains("Couldn't set socket timeout"))
        .unwrap_or(false)
    {
        blockers.push("hyprctl read-only command failed with socket timeout".to_string());
        (
            RuntimeSocketDiagnosisStatus::HyprctlSocketTimeout,
            "hyprctl read-only query could not configure or use the runtime socket timeout"
                .to_string(),
        )
    } else if candidates.iter().any(|candidate| candidate.exists) && !direct_ipc.succeeded {
        blockers.push("socket file exists but read-only IPC did not succeed".to_string());
        (
            RuntimeSocketDiagnosisStatus::StaleSocket,
            "socket path exists but could not be used for read-only IPC".to_string(),
        )
    } else {
        blockers.push("runtime socket failure did not match a known category".to_string());
        (
            RuntimeSocketDiagnosisStatus::UnknownFailure,
            "runtime socket failure remains unknown".to_string(),
        )
    };

    RuntimeSocketDiagnosis {
        status,
        read_only_evidence,
        candidates,
        direct_ipc,
        hyprland_process_visible,
        process_env_matches_shell,
        root_cause,
        mutation_allowed: status == RuntimeSocketDiagnosisStatus::HyprctlReadOnlySucceeded,
        blockers,
    }
}

pub fn runtime_action_policy(action: RuntimeAction) -> RuntimeActionPolicy {
    let reason = match &action {
        RuntimeAction::Status { .. } => {
            "Represented as read-only intent; this scaffold still does not execute hyprctl."
        }
        RuntimeAction::Reload => "Reload is mutating and remains disabled.",
        RuntimeAction::Keyword { .. } => "Keyword changes mutate runtime and remain disabled.",
        RuntimeAction::Dispatch { .. } => "Dispatch commands mutate runtime and remain disabled.",
    }
    .to_string();
    RuntimeActionPolicy {
        action,
        allowlisted_for_real_execution: false,
        dry_run_allowed: true,
        production_runtime_enabled: false,
        reason,
    }
}

pub fn runtime_command_risk(action: &RuntimeAction) -> RuntimeCommandRisk {
    match action {
        RuntimeAction::Status { .. } => RuntimeCommandRisk::ReadOnlyStatus,
        RuntimeAction::Reload => RuntimeCommandRisk::MutatingReload,
        RuntimeAction::Keyword { .. } => RuntimeCommandRisk::MutatingKeyword,
        RuntimeAction::Dispatch { .. } => RuntimeCommandRisk::MutatingDispatch,
    }
}

pub fn runtime_action_review(action: RuntimeAction) -> RuntimeActionReview {
    let policy = runtime_action_policy(action.clone());
    let mut executor = RuntimeDryRunExecutor::default();
    let dry_run_result = executor.evaluate(action.clone());
    RuntimeActionReview {
        action,
        policy,
        execution_log: executor
            .recorded_actions
            .iter()
            .map(|result| result.explanation.clone())
            .collect(),
        production_execution_enabled: false,
        real_command_executed: false,
        dry_run_result,
    }
}

pub fn runtime_guarded_executor(
    action: RuntimeAction,
    guard: &ControlledLiveTestGuardReview,
    prior_value_snapshot: Option<&str>,
) -> RuntimeGuardedExecutionReport {
    let risk = runtime_command_risk(&action);
    let mut executor = RuntimeDryRunExecutor::default();
    let dry_run = executor.evaluate(action.clone());
    if matches!(risk, RuntimeCommandRisk::ReadOnlyStatus) {
        return RuntimeGuardedExecutionReport {
            action,
            guard_allowed: true,
            restore_command: None,
            real_command_executed: false,
            production_runtime_enabled: false,
            runtime_touched: false,
            execution_log: executor
                .recorded_actions
                .iter()
                .map(|result| result.explanation.clone())
                .collect(),
            errors: Vec::new(),
        };
    }

    let restore_command = match (&action, prior_value_snapshot) {
        (RuntimeAction::Keyword { key, .. }, Some(value)) => {
            Some(format!("hyprctl keyword {key} {value}"))
        }
        (RuntimeAction::Reload, Some(_)) => {
            Some("restore config backup before hyprctl reload".to_string())
        }
        (RuntimeAction::Dispatch { .. }, Some(_)) => {
            Some("dispatch restore requires command-specific recovery plan".to_string())
        }
        _ => None,
    };
    let mut errors = Vec::new();
    if !guard.live_mutation_allowed || !guard.runtime_mutation_allowed {
        errors.push("controlled live-test guard did not allow runtime mutation".to_string());
    }
    if restore_command.is_none() {
        errors.push("restore command must be generated before runtime mutation".to_string());
    }
    RuntimeGuardedExecutionReport {
        action,
        guard_allowed: errors.is_empty(),
        restore_command,
        real_command_executed: false,
        production_runtime_enabled: false,
        runtime_touched: false,
        execution_log: vec![dry_run.explanation],
        errors,
    }
}

pub fn runtime_production_gate_review(
    action: RuntimeAction,
    read_only_evidence_available: bool,
    prior_value_snapshot: Option<&str>,
    command_specific_recovery_plan: Option<&str>,
    explicit_approval_recorded: bool,
    production_flag_enabled: bool,
) -> RuntimeProductionGateReview {
    let risk = runtime_command_risk(&action);
    let mut blockers = Vec::new();
    let mut status = RuntimeProductionGateStatus::ReadyButDefaultDisabled;
    if !read_only_evidence_available {
        status = RuntimeProductionGateStatus::ReadOnlyEvidenceMissing;
        blockers.push("reachable read-only runtime evidence is required".to_string());
    }

    let restore_command = match (&action, prior_value_snapshot) {
        (RuntimeAction::Keyword { key, .. }, Some(value)) => {
            Some(format!("hyprctl keyword {key} {value}"))
        }
        (RuntimeAction::Reload, Some(_)) => {
            Some("restore config backup before hyprctl reload".to_string())
        }
        (RuntimeAction::Dispatch { .. }, Some(_)) => {
            command_specific_recovery_plan.map(ToOwned::to_owned)
        }
        (RuntimeAction::Status { .. }, _) => None,
        _ => None,
    };

    if !matches!(risk, RuntimeCommandRisk::ReadOnlyStatus) {
        if prior_value_snapshot.is_none() {
            status = RuntimeProductionGateStatus::PriorValueSnapshotMissing;
            blockers.push("prior runtime value snapshot is required".to_string());
        }
        if restore_command.is_none() {
            status = RuntimeProductionGateStatus::RestoreCommandMissing;
            blockers.push("restore command must be generated before runtime mutation".to_string());
        }
        if matches!(risk, RuntimeCommandRisk::MutatingDispatch)
            && command_specific_recovery_plan.is_none()
        {
            status = RuntimeProductionGateStatus::RecoveryPlanMissing;
            blockers.push("dispatch requires command-specific recovery plan".to_string());
        }
        if !explicit_approval_recorded {
            blockers.push("explicit runtime mutation approval is required".to_string());
        }
    }

    if status == RuntimeProductionGateStatus::ReadyButDefaultDisabled && production_flag_enabled {
        status = RuntimeProductionGateStatus::Enabled;
    } else if status == RuntimeProductionGateStatus::ReadyButDefaultDisabled {
        blockers.push("runtime production mutation flag is default-disabled".to_string());
    }
    let production_runtime_enabled = status == RuntimeProductionGateStatus::Enabled;
    RuntimeProductionGateReview {
        action,
        status,
        read_only_evidence_available,
        restore_command,
        explicit_approval_recorded,
        production_flag_enabled,
        production_runtime_enabled,
        blockers: blockers.clone(),
        review_lines: vec![
            "Runtime mutation requires read-only evidence, prior snapshot, restore command, and explicit approval.".to_string(),
            "Reload requires a config-backup prerequisite; dispatch requires command-specific recovery.".to_string(),
            "Production runtime mutation remains default-disabled.".to_string(),
            format!(
                "Blockers: {}",
                if blockers.is_empty() {
                    "none".to_string()
                } else {
                    blockers.join("; ")
                }
            ),
        ],
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum RuntimeLiveRestoreStatus {
    ReadOnlyEvidenceMissing,
    PriorValueMissing,
    TemporaryValueMissing,
    RestoreCommandMissing,
    PostMutationReadbackMissing,
    PostRestoreVerificationFailed,
    LiveRestoreProven,
    LiveRestoreBlocked,
    ReadyButDefaultDisabled,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum RuntimeMutationSyntaxStatus {
    NotAttempted,
    FailedBeforeValueChange,
    MutatedAndRestored,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RuntimeMutationCommandPair {
    pub mutation_command: String,
    pub restore_command: String,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RuntimeRestoreCommandPair {
    pub setting: String,
    pub prior_value: String,
    pub temporary_value: String,
    pub commands: RuntimeMutationCommandPair,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RuntimeMutationSyntaxCandidate {
    pub syntax_name: String,
    pub command_pair: RuntimeMutationCommandPair,
    pub status: RuntimeMutationSyntaxStatus,
    pub error: Option<String>,
    pub post_mutation_value: Option<String>,
    pub post_restore_value: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RuntimeEvalSyntaxEvidence {
    pub setting: String,
    pub prior_value: String,
    pub temporary_value: String,
    pub candidates: Vec<RuntimeMutationSyntaxCandidate>,
    pub successful_syntax: Option<String>,
    pub live_restore_proven: bool,
    pub runtime_left_restored: bool,
    pub production_runtime_enabled: bool,
}

pub fn runtime_eval_syntax_evidence(
    setting: impl Into<String>,
    prior_value: impl Into<String>,
    temporary_value: impl Into<String>,
    candidates: Vec<RuntimeMutationSyntaxCandidate>,
) -> RuntimeEvalSyntaxEvidence {
    let setting = setting.into();
    let temporary_value = temporary_value.into();
    let prior_value = prior_value.into();
    let successful_syntax = candidates
        .iter()
        .find(|candidate| {
            candidate.status == RuntimeMutationSyntaxStatus::MutatedAndRestored
                && candidate.post_mutation_value.as_deref() == Some(temporary_value.as_str())
                && candidate.post_restore_value.as_deref() == Some(prior_value.as_str())
        })
        .map(|candidate| candidate.syntax_name.clone());
    let runtime_left_restored = candidates.iter().all(|candidate| {
        candidate.status != RuntimeMutationSyntaxStatus::MutatedAndRestored
            || candidate.post_restore_value.as_deref() == Some(prior_value.as_str())
    });
    RuntimeEvalSyntaxEvidence {
        setting,
        prior_value,
        temporary_value,
        live_restore_proven: successful_syntax.is_some(),
        runtime_left_restored,
        production_runtime_enabled: false,
        candidates,
        successful_syntax,
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum RuntimeApprovalReviewStatus {
    MissingLiveRestoreProof,
    FailedLiveRestoreProof,
    MissingMutationSyntaxEvidence,
    MutationSyntaxNotProven,
    WrongSetting,
    RestoreCommandMismatch,
    MissingApproval,
    WrongApprovalScope,
    ApprovalRejected,
    ApprovalExpired,
    ApprovalPending,
    MissingApprovalEvidence,
    ReadyButDefaultDisabled,
    ApprovedButDefaultDisabled,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RuntimeLiveRestoreApprovalEvidence {
    pub setting: String,
    pub prior_value: String,
    pub temporary_value: String,
    pub mutation_command: String,
    pub restore_command: String,
    pub post_mutation_readback: String,
    pub post_restore_readback: String,
    pub restoration_verified: bool,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RuntimeApprovalReview {
    pub action: RuntimeAction,
    pub status: RuntimeApprovalReviewStatus,
    pub live_restore_evidence: Option<RuntimeLiveRestoreApprovalEvidence>,
    pub approval_decision: Option<ApprovalDecision>,
    pub production_flag_enabled: bool,
    pub production_runtime_enabled: bool,
    pub blockers: Vec<String>,
    pub review_lines: Vec<String>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RuntimeApprovalEvidenceSummary {
    pub setting: String,
    pub prior_value: String,
    pub temporary_value: String,
    pub mutation_command: String,
    pub restore_command: String,
    pub post_mutation_readback: String,
    pub post_restore_readback: String,
    pub approval_status: String,
    pub production_runtime_status: String,
    pub production_runtime_enabled: bool,
}

impl RuntimeApprovalEvidenceSummary {
    pub fn user_facing_lines(&self) -> Vec<String> {
        vec![
            "Runtime approval review".to_string(),
            "Runtime changes are not enabled yet.".to_string(),
            "This setting has a proven live-restore test.".to_string(),
            "Production runtime/reload remains disabled.".to_string(),
            format!("Setting: {}", self.setting),
            format!("Prior value: {}", self.prior_value),
            format!("Temporary test value: {}", self.temporary_value),
            format!("Mutation command: {}", self.mutation_command),
            format!("Restore command: {}", self.restore_command),
            format!("Post-mutation readback: {}", self.post_mutation_readback),
            format!("Post-restore readback: {}", self.post_restore_readback),
            format!("Approval status: {}", self.approval_status),
            format!(
                "Production runtime/reload: {}",
                self.production_runtime_status
            ),
        ]
    }
}

pub fn proven_runtime_approval_evidence_summary() -> RuntimeApprovalEvidenceSummary {
    RuntimeApprovalEvidenceSummary {
        setting: "general:gaps_in".to_string(),
        prior_value: "5".to_string(),
        temporary_value: "6".to_string(),
        mutation_command: "hyprctl eval 'hl.config({ general = { gaps_in = 6 } })'".to_string(),
        restore_command: "hyprctl eval 'hl.config({ general = { gaps_in = 5 } })'".to_string(),
        post_mutation_readback: "css gap data: 6 6 6 6; set: true".to_string(),
        post_restore_readback: "css gap data: 5 5 5 5; set: true".to_string(),
        approval_status: "Approved but default-disabled".to_string(),
        production_runtime_status: "Disabled".to_string(),
        production_runtime_enabled: false,
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ApprovalCardProofRecord {
    pub source: String,
    pub status: String,
    pub fields: Vec<(String, String)>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ApprovalCardPreconditionLine {
    pub label: String,
    pub value: String,
    pub status: String,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ApprovalCardRestoreEvidence {
    pub label: String,
    pub status: String,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct DisabledApprovalCardProjection {
    pub widget_name: String,
    pub evidence_widget_name: String,
    pub disabled_action_widget_name: String,
    pub heading: String,
    pub summary_lines: Vec<String>,
    pub proof_record: ApprovalCardProofRecord,
    pub preconditions: Vec<ApprovalCardPreconditionLine>,
    pub restore_evidence: Vec<ApprovalCardRestoreEvidence>,
    pub evidence_lines: Vec<(String, String)>,
    pub blockers: Vec<String>,
    pub disabled_action_label: String,
    pub production_status: String,
    pub production_enabled: bool,
}

impl DisabledApprovalCardProjection {
    pub fn user_facing_lines(&self) -> Vec<String> {
        let mut lines = vec![self.heading.clone()];
        lines.extend(self.summary_lines.clone());
        lines.push(format!("Proof source: {}", self.proof_record.source));
        lines.push(format!("Proof status: {}", self.proof_record.status));
        for (label, value) in &self.proof_record.fields {
            lines.push(format!("Proof {label}: {value}"));
        }
        for precondition in &self.preconditions {
            lines.push(format!(
                "Precondition {}: {} ({})",
                precondition.label, precondition.value, precondition.status
            ));
        }
        for evidence in &self.restore_evidence {
            lines.push(format!(
                "Restore evidence {}: {}",
                evidence.label, evidence.status
            ));
        }
        for (label, value) in &self.evidence_lines {
            lines.push(format!("{label}: {value}"));
        }
        lines.push(format!("Production status: {}", self.production_status));
        for blocker in &self.blockers {
            lines.push(format!("Blocker: {blocker}"));
        }
        lines.push(self.disabled_action_label.clone());
        lines
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ProductionActivationDecisionStatus {
    MissingReportBackedCard,
    MissingProofSource,
    MissingProofStatus,
    MissingRequiredProofField,
    MissingPrecondition,
    MissingRestoreEvidence,
    MissingOriginalUnchangedProof,
    MissingApprovalStatus,
    ProductionAlreadyEnabledError,
    ReadyButDefaultDisabled,
    ApprovedButDefaultDisabled,
    Blocked,
}

impl ProductionActivationDecisionStatus {
    pub fn user_facing_label(&self) -> &'static str {
        match self {
            Self::MissingReportBackedCard => "Missing report-backed card",
            Self::MissingProofSource => "Missing proof source",
            Self::MissingProofStatus => "Missing proof status",
            Self::MissingRequiredProofField => "Missing required proof field",
            Self::MissingPrecondition => "Missing precondition",
            Self::MissingRestoreEvidence => "Missing restore evidence",
            Self::MissingOriginalUnchangedProof => "Missing original unchanged proof",
            Self::MissingApprovalStatus => "Missing approval status",
            Self::ProductionAlreadyEnabledError => "Production already enabled error",
            Self::ReadyButDefaultDisabled => "Ready but default-disabled",
            Self::ApprovedButDefaultDisabled => "Approved but default-disabled",
            Self::Blocked => "Blocked",
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ProductionActivationDecisionReview {
    pub widget_name: String,
    pub evidence_widget_name: String,
    pub disabled_action_widget_name: String,
    pub heading: String,
    pub input_source: String,
    pub status: ProductionActivationDecisionStatus,
    pub required_proof_summary: Vec<String>,
    pub blockers: Vec<String>,
    pub production_label: String,
    pub production_status: String,
    pub production_enabled: bool,
    pub disabled_action_label: String,
}

impl ProductionActivationDecisionReview {
    pub fn user_facing_lines(&self) -> Vec<String> {
        let mut lines = vec![
            self.heading.clone(),
            format!("Decision status: {}", self.status.user_facing_label()),
            format!("Decision input source: {}", self.input_source),
        ];
        lines.extend(
            self.required_proof_summary
                .iter()
                .map(|summary| format!("Required proof: {summary}")),
        );
        lines.extend(
            self.blockers
                .iter()
                .map(|blocker| format!("Decision blocker: {blocker}")),
        );
        lines.push(format!(
            "{}: {}",
            self.production_label, self.production_status
        ));
        lines.push(self.disabled_action_label.clone());
        lines
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ProductionActivationPathStatus {
    ActivationPathReadyButDefaultDisabled,
    ActivationPathBlocked,
    ActivationPathNeedsExplicitProductionFlag,
    ActivationPathNeedsRealBackupRestorePlan,
    ActivationPathNeedsFinalUserApproval,
}

impl ProductionActivationPathStatus {
    pub fn user_facing_label(&self) -> &'static str {
        match self {
            Self::ActivationPathReadyButDefaultDisabled => {
                "Activation path ready but default-disabled"
            }
            Self::ActivationPathBlocked => "Activation path blocked",
            Self::ActivationPathNeedsExplicitProductionFlag => {
                "Activation path needs explicit production flag"
            }
            Self::ActivationPathNeedsRealBackupRestorePlan => {
                "Activation path needs real backup/restore plan"
            }
            Self::ActivationPathNeedsFinalUserApproval => {
                "Activation path needs final user approval"
            }
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ProductionActivationRequestScope {
    SourceIncludeInsertion,
    DuplicateReplacement,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ProductionActivationRequest {
    pub scope: ProductionActivationRequestScope,
    pub user_facing_reason: String,
    pub decision_category: String,
    pub explicit_activation_token: String,
    pub backup_plan_acknowledged: bool,
    pub restore_plan_acknowledged: bool,
    pub reread_plan_acknowledged: bool,
    pub final_confirmation_acknowledged: bool,
    pub one_shot_nonce: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ProductionActivationSafetyPlan {
    pub backup_before_write_plan: Option<String>,
    pub restore_plan: Option<String>,
    pub post_write_reread_plan: Option<String>,
    pub post_restore_verification_plan: Option<String>,
    pub dry_run_summary: Option<String>,
    pub files_that_would_be_touched: Vec<String>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ProductionActivationPathReview {
    pub widget_name: String,
    pub evidence_widget_name: String,
    pub disabled_action_widget_name: String,
    pub heading: String,
    pub input_decision_status: ProductionActivationDecisionStatus,
    pub input_proof_source: String,
    pub status: ProductionActivationPathStatus,
    pub required_before_enabling: Vec<String>,
    pub blockers: Vec<String>,
    pub production_label: String,
    pub production_status: String,
    pub production_activation_enabled: bool,
    pub category_production_enabled: bool,
    pub disabled_action_label: String,
}

impl ProductionActivationPathReview {
    pub fn user_facing_lines(&self) -> Vec<String> {
        let mut lines = vec![
            self.heading.clone(),
            format!(
                "Input decision: {}",
                self.input_decision_status.user_facing_label()
            ),
            format!("Proof source: {}", self.input_proof_source),
            format!(
                "Activation path status: {}",
                self.status.user_facing_label()
            ),
        ];
        lines.extend(
            self.required_before_enabling
                .iter()
                .map(|item| format!("Required before enabling: {item}")),
        );
        lines.extend(
            self.blockers
                .iter()
                .map(|blocker| format!("Activation path blocker: {blocker}")),
        );
        lines.push(format!(
            "{}: {}",
            self.production_label, self.production_status
        ));
        lines.push(self.disabled_action_label.clone());
        lines
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ProductionExecutorWiringState {
    Unwired,
    WiredForTestingOnly,
    WiredProduction,
}

impl ProductionExecutorWiringState {
    pub fn user_facing_label(&self) -> &'static str {
        match self {
            Self::Unwired => "Unwired",
            Self::WiredForTestingOnly => "Wired for testing only",
            Self::WiredProduction => "Wired for production",
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ProductionActivationControlStatus {
    MissingActivationPath,
    MissingActivationRequest,
    MissingSafetyPlan,
    MissingBackupPlan,
    MissingRestorePlan,
    MissingRereadPlan,
    MissingFinalConfirmation,
    WrongScope,
    WrongCategory,
    ProductionFlagMustRemainFalse,
    ExecutorMustRemainUnwired,
    ValidatedButExecutorUnwired,
    Blocked,
}

impl ProductionActivationControlStatus {
    pub fn user_facing_label(&self) -> &'static str {
        match self {
            Self::MissingActivationPath => "Missing activation path",
            Self::MissingActivationRequest => "Missing activation request",
            Self::MissingSafetyPlan => "Missing safety plan",
            Self::MissingBackupPlan => "Missing backup plan",
            Self::MissingRestorePlan => "Missing restore plan",
            Self::MissingRereadPlan => "Missing reread plan",
            Self::MissingFinalConfirmation => "Missing final confirmation",
            Self::WrongScope => "Wrong scope",
            Self::WrongCategory => "Wrong category",
            Self::ProductionFlagMustRemainFalse => "Production flag must remain false",
            Self::ExecutorMustRemainUnwired => "Executor must remain unwired",
            Self::ValidatedButExecutorUnwired => "Validated but executor unwired",
            Self::Blocked => "Blocked",
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ProductionActivationControlReview {
    pub widget_name: String,
    pub evidence_widget_name: String,
    pub disabled_action_widget_name: String,
    pub heading: String,
    pub input_path_status: ProductionActivationPathStatus,
    pub input_decision_status: ProductionActivationDecisionStatus,
    pub status: ProductionActivationControlStatus,
    pub request_validation_status: String,
    pub safety_plan_validation_status: String,
    pub executor_wiring_status: ProductionExecutorWiringState,
    pub blockers: Vec<String>,
    pub production_label: String,
    pub production_status: String,
    pub production_activation_enabled: bool,
    pub category_production_enabled: bool,
    pub disabled_action_label: String,
}

impl ProductionActivationControlReview {
    pub fn user_facing_lines(&self) -> Vec<String> {
        let mut lines = vec![
            self.heading.clone(),
            format!(
                "Input path status: {}",
                self.input_path_status.user_facing_label()
            ),
            format!(
                "Input decision: {}",
                self.input_decision_status.user_facing_label()
            ),
            format!("Control status: {}", self.status.user_facing_label()),
            format!("Request validation: {}", self.request_validation_status),
            format!(
                "Safety plan validation: {}",
                self.safety_plan_validation_status
            ),
            format!(
                "Executor wiring: {}",
                self.executor_wiring_status.user_facing_label()
            ),
        ];
        lines.extend(
            self.blockers
                .iter()
                .map(|blocker| format!("Activation control blocker: {blocker}")),
        );
        lines.push(format!(
            "{}: {}",
            self.production_label, self.production_status
        ));
        lines.push(self.disabled_action_label.clone());
        lines
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ProductionActivationFormStatus {
    Empty,
    MissingRequiredFields,
    InvalidWrongScope,
    InvalidWrongCategory,
    BlockedExecutorWired,
    BlockedProductionFlagTrue,
    ValidatedForReviewOnly,
}

impl ProductionActivationFormStatus {
    pub fn user_facing_label(&self) -> &'static str {
        match self {
            Self::Empty => "Empty",
            Self::MissingRequiredFields => "Missing required fields",
            Self::InvalidWrongScope => "Invalid wrong scope",
            Self::InvalidWrongCategory => "Invalid wrong category",
            Self::BlockedExecutorWired => "Blocked because executor is wired",
            Self::BlockedProductionFlagTrue => "Blocked because production flag is true",
            Self::ValidatedForReviewOnly => "Validated for review only",
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ProductionActivationFormState {
    pub scope: Option<ProductionActivationRequestScope>,
    pub user_facing_reason: String,
    pub explicit_activation_token: String,
    pub decision_category: String,
    pub backup_plan_acknowledged: bool,
    pub restore_plan_acknowledged: bool,
    pub reread_plan_acknowledged: bool,
    pub post_restore_verification_acknowledged: bool,
    pub final_confirmation_acknowledged: bool,
    pub backup_before_write_plan: String,
    pub restore_plan: String,
    pub post_write_reread_plan: String,
    pub post_restore_verification_plan: String,
    pub dry_run_summary: String,
    pub files_that_would_be_touched: Vec<String>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ProductionActivationFormReview {
    pub widget_name: String,
    pub evidence_widget_name: String,
    pub disabled_action_widget_name: String,
    pub heading: String,
    pub form_state: ProductionActivationFormState,
    pub status: ProductionActivationFormStatus,
    pub missing_fields: Vec<String>,
    pub request_preview: Vec<String>,
    pub safety_plan_preview: Vec<String>,
    pub control_validation_status: ProductionActivationControlStatus,
    pub request_generation_status: String,
    pub safety_plan_generation_status: String,
    pub executor_wiring_status: ProductionExecutorWiringState,
    pub production_label: String,
    pub production_status: String,
    pub production_activation_enabled: bool,
    pub category_production_enabled: bool,
    pub disabled_action_label: String,
}

impl ProductionActivationFormReview {
    pub fn user_facing_lines(&self) -> Vec<String> {
        let mut lines = vec![
            self.heading.clone(),
            format!("Form status: {}", self.status.user_facing_label()),
            format!("Request generation: {}", self.request_generation_status),
            format!(
                "Safety plan generation: {}",
                self.safety_plan_generation_status
            ),
            format!(
                "Control validation: {}",
                self.control_validation_status.user_facing_label()
            ),
            format!(
                "Executor wiring: {}",
                self.executor_wiring_status.user_facing_label()
            ),
        ];
        lines.extend(
            self.missing_fields
                .iter()
                .map(|field| format!("Missing field: {field}")),
        );
        lines.extend(
            self.request_preview
                .iter()
                .map(|field| format!("Request preview: {field}")),
        );
        lines.extend(
            self.safety_plan_preview
                .iter()
                .map(|field| format!("Safety plan preview: {field}")),
        );
        lines.push(format!(
            "{}: {}",
            self.production_label, self.production_status
        ));
        lines.push(self.disabled_action_label.clone());
        lines
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ProductionActivationDraftStatus {
    EmptyDraft,
    DraftDirty,
    DraftMissingRequiredFields,
    DraftValidatedForReviewOnly,
    DraftBlockedExecutorWired,
    DraftBlockedProductionFlagTrue,
    DraftInvalidWrongScope,
    DraftInvalidWrongCategory,
}

impl ProductionActivationDraftStatus {
    pub fn user_facing_label(&self) -> &'static str {
        match self {
            Self::EmptyDraft => "Empty draft",
            Self::DraftDirty => "Draft dirty",
            Self::DraftMissingRequiredFields => "Draft missing required fields",
            Self::DraftValidatedForReviewOnly => "Draft validated for review only",
            Self::DraftBlockedExecutorWired => "Draft blocked because executor is wired",
            Self::DraftBlockedProductionFlagTrue => "Draft blocked because production flag is true",
            Self::DraftInvalidWrongScope => "Draft invalid wrong scope",
            Self::DraftInvalidWrongCategory => "Draft invalid wrong category",
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ProductionActivationDraftUpdate {
    Scope(Option<ProductionActivationRequestScope>),
    UserFacingReason(String),
    ExplicitActivationToken(String),
    DecisionCategory(String),
    BackupPlanAcknowledged(bool),
    RestorePlanAcknowledged(bool),
    RereadPlanAcknowledged(bool),
    PostRestoreVerificationAcknowledged(bool),
    FinalConfirmationAcknowledged(bool),
    BackupBeforeWritePlan(String),
    RestorePlan(String),
    PostWriteRereadPlan(String),
    PostRestoreVerificationPlan(String),
    DryRunSummary(String),
    FilesThatWouldBeTouched(Vec<String>),
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ProductionActivationDraftForm {
    pub form_state: ProductionActivationFormState,
    pub dirty: bool,
    pub persisted: bool,
    pub last_validation_status: ProductionActivationDraftStatus,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ProductionActivationDraftReview {
    pub widget_name: String,
    pub evidence_widget_name: String,
    pub disabled_update_widget_name: String,
    pub disabled_reset_widget_name: String,
    pub heading: String,
    pub draft: ProductionActivationDraftForm,
    pub status: ProductionActivationDraftStatus,
    pub form_validation_status: ProductionActivationFormStatus,
    pub control_validation_status: ProductionActivationControlStatus,
    pub dirty_state: String,
    pub persistence_status: String,
    pub request_generation_status: String,
    pub safety_plan_generation_status: String,
    pub executor_wiring_status: ProductionExecutorWiringState,
    pub production_label: String,
    pub production_status: String,
    pub production_activation_enabled: bool,
    pub category_production_enabled: bool,
    pub disabled_update_label: String,
    pub disabled_reset_label: String,
}

impl ProductionActivationDraftReview {
    pub fn user_facing_lines(&self) -> Vec<String> {
        vec![
            self.heading.clone(),
            format!("Draft status: {}", self.status.user_facing_label()),
            format!(
                "Draft validation: {}",
                self.form_validation_status.user_facing_label()
            ),
            format!("Dirty state: {}", self.dirty_state),
            self.persistence_status.clone(),
            format!(
                "Control validation: {}",
                self.control_validation_status.user_facing_label()
            ),
            format!(
                "Executor wiring: {}",
                self.executor_wiring_status.user_facing_label()
            ),
            format!("{}: {}", self.production_label, self.production_status),
            self.disabled_update_label.clone(),
            self.disabled_reset_label.clone(),
        ]
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ProductionActivationDraftEditMode {
    DisabledByDefault,
    EnabledInMemoryOnly,
}

impl ProductionActivationDraftEditMode {
    pub fn user_facing_label(&self) -> &'static str {
        match self {
            Self::DisabledByDefault => "Draft editing disabled by default",
            Self::EnabledInMemoryOnly => "Draft editing enabled in memory only",
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ProductionActivationDraftEditStatus {
    DraftEditingUnavailable,
    DraftEditingDisabledByDefault,
    DraftEditingEnabledInMemoryOnly,
    DraftEditingDirty,
    DraftEditingValidatedForReviewOnly,
    DraftEditingBlockedProductionFlagTrue,
    DraftEditingBlockedExecutorWired,
    DraftEditingPersistenceUnavailable,
}

impl ProductionActivationDraftEditStatus {
    pub fn user_facing_label(&self) -> &'static str {
        match self {
            Self::DraftEditingUnavailable => "Draft editing unavailable",
            Self::DraftEditingDisabledByDefault => "Draft editing disabled by default",
            Self::DraftEditingEnabledInMemoryOnly => "Draft editing enabled in memory only",
            Self::DraftEditingDirty => "Draft editing dirty",
            Self::DraftEditingValidatedForReviewOnly => "Draft editing validated for review only",
            Self::DraftEditingBlockedProductionFlagTrue => {
                "Draft editing blocked because production flag is true"
            }
            Self::DraftEditingBlockedExecutorWired => {
                "Draft editing blocked because executor is wired"
            }
            Self::DraftEditingPersistenceUnavailable => "Draft editing persistence unavailable",
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ProductionActivationDraftEditAction {
    EnterInMemoryEditMode,
    ApplyUpdate(ProductionActivationDraftUpdate),
    ResetToDefault,
    ExitEditMode,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ProductionActivationDraftEditState {
    pub draft: ProductionActivationDraftForm,
    pub mode: ProductionActivationDraftEditMode,
    pub dirty: bool,
    pub persisted: bool,
    pub production_action_enabled: bool,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ProductionActivationDraftEditResult {
    pub status: ProductionActivationDraftEditStatus,
    pub state: ProductionActivationDraftEditState,
    pub draft_validation_status: ProductionActivationDraftStatus,
    pub persistence_status: String,
    pub production_action_enabled: bool,
    pub executor_wired: bool,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ProductionActivationDraftEditReview {
    pub widget_name: String,
    pub evidence_widget_name: String,
    pub mode_widget_name: String,
    pub disabled_update_widget_name: String,
    pub disabled_reset_widget_name: String,
    pub heading: String,
    pub mode: ProductionActivationDraftEditMode,
    pub status: ProductionActivationDraftEditStatus,
    pub draft_status: ProductionActivationDraftStatus,
    pub form_validation_status: ProductionActivationFormStatus,
    pub control_validation_status: ProductionActivationControlStatus,
    pub dirty_state: String,
    pub persistence_status: String,
    pub executor_wiring_status: ProductionExecutorWiringState,
    pub production_label: String,
    pub production_status: String,
    pub production_activation_enabled: bool,
    pub category_production_enabled: bool,
    pub disabled_update_label: String,
    pub disabled_reset_label: String,
}

impl ProductionActivationDraftEditReview {
    pub fn user_facing_lines(&self) -> Vec<String> {
        vec![
            self.heading.clone(),
            format!("Editing mode: {}", self.mode.user_facing_label()),
            format!("Edit status: {}", self.status.user_facing_label()),
            format!("Draft dirty state: {}", self.dirty_state),
            format!(
                "Draft validation: {}",
                self.draft_status.user_facing_label()
            ),
            self.persistence_status.clone(),
            format!(
                "Form validation: {}",
                self.form_validation_status.user_facing_label()
            ),
            format!(
                "Control validation: {}",
                self.control_validation_status.user_facing_label()
            ),
            format!(
                "Executor wiring: {}",
                self.executor_wiring_status.user_facing_label()
            ),
            format!("{}: {}", self.production_label, self.production_status),
            self.disabled_update_label.clone(),
            self.disabled_reset_label.clone(),
        ]
    }
}

const DISABLED_APPROVAL_CARDS_REPORT_PATH: &str =
    "data/reports/disabled-approval-ui-cards.v0.55.2.json";
const DISABLED_APPROVAL_CARDS_REPORT_JSON: &str =
    include_str!("../data/reports/disabled-approval-ui-cards.v0.55.2.json");

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ApprovalCardReportLoadStatus {
    Loaded,
    ReportUnavailable(String),
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ApprovalCardReportSource {
    pub path: String,
    pub load_status: ApprovalCardReportLoadStatus,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ReportBackedDisabledApprovalCardProjection {
    pub source: ApprovalCardReportSource,
    pub cards: Vec<DisabledApprovalCardProjection>,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
struct SerializedDisabledApprovalCardsReport {
    cards: SerializedApprovalCards,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
struct SerializedApprovalCards {
    source_include_insertion: SerializedApprovalCardRecord,
    duplicate_replacement: SerializedApprovalCardRecord,
    structured_hl_bind_write: SerializedApprovalCardRecord,
    profile_mode_switch: SerializedApprovalCardRecord,
    high_risk_display_write: SerializedApprovalCardRecord,
    hyprland0554_migration: SerializedApprovalCardRecord,
}

#[derive(Debug, Default, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SerializedApprovalCardRecord {
    widget_name: Option<String>,
    evidence_widget_name: Option<String>,
    disabled_action_widget_name: Option<String>,
    proof_source: Option<String>,
    proof_status: Option<String>,
    proof_fields: Option<BTreeMap<String, String>>,
    preconditions: Option<BTreeMap<String, String>>,
    restore_evidence: Option<BTreeMap<String, String>>,
    production_status: Option<String>,
    active_model: Option<String>,
    migration_status: Option<String>,
    approval_status: Option<String>,
    blockers: Option<Vec<String>>,
}

pub fn load_disabled_approval_cards_from_reports() -> ReportBackedDisabledApprovalCardProjection {
    load_disabled_approval_cards_from_report_str(
        DISABLED_APPROVAL_CARDS_REPORT_PATH,
        DISABLED_APPROVAL_CARDS_REPORT_JSON,
    )
}

pub fn load_disabled_approval_cards_from_report_str(
    path: &str,
    report_json: &str,
) -> ReportBackedDisabledApprovalCardProjection {
    match serde_json::from_str::<SerializedDisabledApprovalCardsReport>(report_json) {
        Ok(report) => ReportBackedDisabledApprovalCardProjection {
            source: ApprovalCardReportSource {
                path: path.to_string(),
                load_status: ApprovalCardReportLoadStatus::Loaded,
            },
            cards: cards_from_serialized_report(report),
        },
        Err(error) => ReportBackedDisabledApprovalCardProjection {
            source: ApprovalCardReportSource {
                path: path.to_string(),
                load_status: ApprovalCardReportLoadStatus::ReportUnavailable(error.to_string()),
            },
            cards: fallback_disabled_future_approval_card_projections()
                .into_iter()
                .map(mark_card_report_unavailable)
                .collect(),
        },
    }
}

pub fn disabled_future_approval_card_projections() -> Vec<DisabledApprovalCardProjection> {
    load_disabled_approval_cards_from_reports().cards
}

pub fn production_activation_decision_reviews() -> Vec<ProductionActivationDecisionReview> {
    let report = load_disabled_approval_cards_from_reports();
    let input_source = match &report.source.load_status {
        ApprovalCardReportLoadStatus::Loaded => report.source.path.clone(),
        ApprovalCardReportLoadStatus::ReportUnavailable(error) => {
            format!("{} ({error})", report.source.path)
        }
    };
    vec![
        source_include_activation_decision_review(
            report
                .cards
                .iter()
                .find(|card| card.widget_name.contains("source-include")),
            &input_source,
        ),
        duplicate_activation_decision_review(
            report
                .cards
                .iter()
                .find(|card| card.widget_name.contains("duplicate")),
            &input_source,
        ),
    ]
}

pub fn source_include_activation_decision_review(
    card: Option<&DisabledApprovalCardProjection>,
    input_source: &str,
) -> ProductionActivationDecisionReview {
    activation_decision_review(
        card,
        input_source,
        ActivationDecisionSpec {
            widget_name: "hyprland-settings-source-include-activation-decision-disabled",
            evidence_widget_name: "hyprland-settings-source-include-activation-decision-evidence",
            disabled_action_widget_name:
                "hyprland-settings-source-include-activation-decision-enable-disabled",
            heading: "Source/include production activation decision",
            production_label: "Production source/include insertion",
            disabled_action_label: "Enable source/include production activation (planned)",
            required_proof_fields: &[
                "root config",
                "selected target",
                "source depth",
                "dry-run status",
            ],
            required_preconditions: &["planned inserted line", "proposed value"],
            required_restore_evidence: &["copied target restore"],
            original_unchanged_label: "original real config unchanged",
        },
    )
}

pub fn duplicate_activation_decision_review(
    card: Option<&DisabledApprovalCardProjection>,
    input_source: &str,
) -> ProductionActivationDecisionReview {
    activation_decision_review(
        card,
        input_source,
        ActivationDecisionSpec {
            widget_name: "hyprland-settings-duplicate-activation-decision-disabled",
            evidence_widget_name: "hyprland-settings-duplicate-activation-decision-evidence",
            disabled_action_widget_name:
                "hyprland-settings-duplicate-activation-decision-enable-disabled",
            heading: "Duplicate production activation decision",
            production_label: "Production duplicate writes",
            disabled_action_label: "Enable duplicate production activation (planned)",
            required_proof_fields: &[
                "selected occurrence",
                "target path",
                "source depth",
                "copied replacement status",
            ],
            required_preconditions: &["line number", "raw line", "old value", "proposed value"],
            required_restore_evidence: &["copied target restore"],
            original_unchanged_label: "original real config unchanged",
        },
    )
}

pub fn production_activation_path_reviews() -> Vec<ProductionActivationPathReview> {
    let report = load_disabled_approval_cards_from_reports();
    let input_source = match &report.source.load_status {
        ApprovalCardReportLoadStatus::Loaded => report.source.path.clone(),
        ApprovalCardReportLoadStatus::ReportUnavailable(error) => {
            format!("{} ({error})", report.source.path)
        }
    };
    let source_card = report
        .cards
        .iter()
        .find(|card| card.widget_name.contains("source-include"));
    let duplicate_card = report
        .cards
        .iter()
        .find(|card| card.widget_name.contains("duplicate"));
    let source_decision = source_include_activation_decision_review(source_card, &input_source);
    let duplicate_decision = duplicate_activation_decision_review(duplicate_card, &input_source);
    vec![
        source_include_activation_path_review(
            Some(&source_decision),
            source_card,
            None,
            None,
            false,
        ),
        duplicate_activation_path_review(
            Some(&duplicate_decision),
            duplicate_card,
            None,
            None,
            false,
        ),
    ]
}

pub fn source_include_activation_path_review(
    decision: Option<&ProductionActivationDecisionReview>,
    card: Option<&DisabledApprovalCardProjection>,
    request: Option<&ProductionActivationRequest>,
    safety_plan: Option<&ProductionActivationSafetyPlan>,
    production_activation_flag: bool,
) -> ProductionActivationPathReview {
    activation_path_review(
        decision,
        card,
        request,
        safety_plan,
        production_activation_flag,
        ActivationPathSpec {
            widget_name: "hyprland-settings-source-include-activation-path-disabled",
            evidence_widget_name: "hyprland-settings-source-include-activation-path-evidence",
            disabled_action_widget_name:
                "hyprland-settings-source-include-activation-path-start-disabled",
            heading: "Source/include production activation path",
            production_label: "Production source/include insertion",
            disabled_action_label: "Start source/include production activation (planned)",
            expected_scope: ProductionActivationRequestScope::SourceIncludeInsertion,
            expected_category: "sourceIncludeInsertion",
        },
    )
}

pub fn duplicate_activation_path_review(
    decision: Option<&ProductionActivationDecisionReview>,
    card: Option<&DisabledApprovalCardProjection>,
    request: Option<&ProductionActivationRequest>,
    safety_plan: Option<&ProductionActivationSafetyPlan>,
    production_activation_flag: bool,
) -> ProductionActivationPathReview {
    activation_path_review(
        decision,
        card,
        request,
        safety_plan,
        production_activation_flag,
        ActivationPathSpec {
            widget_name: "hyprland-settings-duplicate-activation-path-disabled",
            evidence_widget_name: "hyprland-settings-duplicate-activation-path-evidence",
            disabled_action_widget_name:
                "hyprland-settings-duplicate-activation-path-start-disabled",
            heading: "Duplicate production activation path",
            production_label: "Production duplicate writes",
            disabled_action_label: "Start duplicate production activation (planned)",
            expected_scope: ProductionActivationRequestScope::DuplicateReplacement,
            expected_category: "duplicateReplacement",
        },
    )
}

pub fn production_activation_control_reviews() -> Vec<ProductionActivationControlReview> {
    let report = load_disabled_approval_cards_from_reports();
    let input_source = match &report.source.load_status {
        ApprovalCardReportLoadStatus::Loaded => report.source.path.clone(),
        ApprovalCardReportLoadStatus::ReportUnavailable(error) => {
            format!("{} ({error})", report.source.path)
        }
    };
    let source_card = report
        .cards
        .iter()
        .find(|card| card.widget_name.contains("source-include"));
    let duplicate_card = report
        .cards
        .iter()
        .find(|card| card.widget_name.contains("duplicate"));
    let source_decision = source_include_activation_decision_review(source_card, &input_source);
    let duplicate_decision = duplicate_activation_decision_review(duplicate_card, &input_source);
    let source_request = production_activation_control_request(
        ProductionActivationRequestScope::SourceIncludeInsertion,
        "sourceIncludeInsertion",
    );
    let duplicate_request = production_activation_control_request(
        ProductionActivationRequestScope::DuplicateReplacement,
        "duplicateReplacement",
    );
    let safety_plan = production_activation_control_safety_plan();
    let source_path = source_include_activation_path_review(
        Some(&source_decision),
        source_card,
        Some(&source_request),
        Some(&safety_plan),
        false,
    );
    let duplicate_path = duplicate_activation_path_review(
        Some(&duplicate_decision),
        duplicate_card,
        Some(&duplicate_request),
        Some(&safety_plan),
        false,
    );
    vec![
        source_include_activation_control_review(
            Some(&source_path),
            Some(&source_request),
            Some(&safety_plan),
            ProductionExecutorWiringState::Unwired,
            false,
        ),
        duplicate_activation_control_review(
            Some(&duplicate_path),
            Some(&duplicate_request),
            Some(&safety_plan),
            ProductionExecutorWiringState::Unwired,
            false,
        ),
    ]
}

pub fn production_activation_form_reviews() -> Vec<ProductionActivationFormReview> {
    let paths = production_activation_path_reviews();
    let source_path = paths
        .iter()
        .find(|path| path.widget_name.contains("source-include"));
    let duplicate_path = paths
        .iter()
        .find(|path| path.widget_name.contains("duplicate"));
    vec![
        source_include_activation_form_review(
            source_path,
            production_activation_form_state(
                ProductionActivationRequestScope::SourceIncludeInsertion,
                "sourceIncludeInsertion",
            ),
            ProductionExecutorWiringState::Unwired,
            false,
        ),
        duplicate_activation_form_review(
            duplicate_path,
            production_activation_form_state(
                ProductionActivationRequestScope::DuplicateReplacement,
                "duplicateReplacement",
            ),
            ProductionExecutorWiringState::Unwired,
            false,
        ),
    ]
}

pub fn production_activation_draft_reviews() -> Vec<ProductionActivationDraftReview> {
    let paths = production_activation_path_reviews();
    let source_path = paths
        .iter()
        .find(|path| path.widget_name.contains("source-include"));
    let duplicate_path = paths
        .iter()
        .find(|path| path.widget_name.contains("duplicate"));
    vec![
        source_include_activation_draft_review(
            source_path,
            production_activation_draft_from_form_state(production_activation_form_state(
                ProductionActivationRequestScope::SourceIncludeInsertion,
                "sourceIncludeInsertion",
            )),
            ProductionExecutorWiringState::Unwired,
            false,
        ),
        duplicate_activation_draft_review(
            duplicate_path,
            production_activation_draft_from_form_state(production_activation_form_state(
                ProductionActivationRequestScope::DuplicateReplacement,
                "duplicateReplacement",
            )),
            ProductionExecutorWiringState::Unwired,
            false,
        ),
    ]
}

pub fn production_activation_draft_edit_reviews() -> Vec<ProductionActivationDraftEditReview> {
    let paths = production_activation_path_reviews();
    let source_path = paths
        .iter()
        .find(|path| path.widget_name.contains("source-include"));
    let duplicate_path = paths
        .iter()
        .find(|path| path.widget_name.contains("duplicate"));
    vec![
        source_include_activation_draft_edit_review(
            source_path,
            production_activation_draft_edit_state_from_draft(
                production_activation_draft_from_form_state(production_activation_form_state(
                    ProductionActivationRequestScope::SourceIncludeInsertion,
                    "sourceIncludeInsertion",
                )),
            ),
            ProductionExecutorWiringState::Unwired,
            false,
        ),
        duplicate_activation_draft_edit_review(
            duplicate_path,
            production_activation_draft_edit_state_from_draft(
                production_activation_draft_from_form_state(production_activation_form_state(
                    ProductionActivationRequestScope::DuplicateReplacement,
                    "duplicateReplacement",
                )),
            ),
            ProductionExecutorWiringState::Unwired,
            false,
        ),
    ]
}

pub fn production_activation_form_state(
    scope: ProductionActivationRequestScope,
    decision_category: &str,
) -> ProductionActivationFormState {
    ProductionActivationFormState {
        scope: Some(scope),
        user_facing_reason: "review-only activation form validation".to_string(),
        explicit_activation_token: "VALIDATE FORM ONLY - KEEP EXECUTOR UNWIRED".to_string(),
        decision_category: decision_category.to_string(),
        backup_plan_acknowledged: true,
        restore_plan_acknowledged: true,
        reread_plan_acknowledged: true,
        post_restore_verification_acknowledged: true,
        final_confirmation_acknowledged: true,
        backup_before_write_plan: "create byte-exact backup before any future production write"
            .to_string(),
        restore_plan: "restore original bytes before any review completes".to_string(),
        post_write_reread_plan: "reread target after any future write".to_string(),
        post_restore_verification_plan: "verify restored hash after restore".to_string(),
        dry_run_summary: "dry-run must show exact target, exact old state, and exact proposed line"
            .to_string(),
        files_that_would_be_touched: vec!["report-backed selected target only".to_string()],
    }
}

pub fn empty_production_activation_draft() -> ProductionActivationDraftForm {
    ProductionActivationDraftForm {
        form_state: ProductionActivationFormState {
            scope: None,
            user_facing_reason: String::new(),
            explicit_activation_token: String::new(),
            decision_category: String::new(),
            backup_plan_acknowledged: false,
            restore_plan_acknowledged: false,
            reread_plan_acknowledged: false,
            post_restore_verification_acknowledged: false,
            final_confirmation_acknowledged: false,
            backup_before_write_plan: String::new(),
            restore_plan: String::new(),
            post_write_reread_plan: String::new(),
            post_restore_verification_plan: String::new(),
            dry_run_summary: String::new(),
            files_that_would_be_touched: Vec::new(),
        },
        dirty: false,
        persisted: false,
        last_validation_status: ProductionActivationDraftStatus::EmptyDraft,
    }
}

pub fn production_activation_draft_from_form_state(
    form_state: ProductionActivationFormState,
) -> ProductionActivationDraftForm {
    ProductionActivationDraftForm {
        form_state,
        dirty: false,
        persisted: false,
        last_validation_status: ProductionActivationDraftStatus::DraftValidatedForReviewOnly,
    }
}

pub fn apply_production_activation_draft_update(
    draft: &mut ProductionActivationDraftForm,
    update: ProductionActivationDraftUpdate,
) {
    match update {
        ProductionActivationDraftUpdate::Scope(value) => draft.form_state.scope = value,
        ProductionActivationDraftUpdate::UserFacingReason(value) => {
            draft.form_state.user_facing_reason = value
        }
        ProductionActivationDraftUpdate::ExplicitActivationToken(value) => {
            draft.form_state.explicit_activation_token = value
        }
        ProductionActivationDraftUpdate::DecisionCategory(value) => {
            draft.form_state.decision_category = value
        }
        ProductionActivationDraftUpdate::BackupPlanAcknowledged(value) => {
            draft.form_state.backup_plan_acknowledged = value
        }
        ProductionActivationDraftUpdate::RestorePlanAcknowledged(value) => {
            draft.form_state.restore_plan_acknowledged = value
        }
        ProductionActivationDraftUpdate::RereadPlanAcknowledged(value) => {
            draft.form_state.reread_plan_acknowledged = value
        }
        ProductionActivationDraftUpdate::PostRestoreVerificationAcknowledged(value) => {
            draft.form_state.post_restore_verification_acknowledged = value
        }
        ProductionActivationDraftUpdate::FinalConfirmationAcknowledged(value) => {
            draft.form_state.final_confirmation_acknowledged = value
        }
        ProductionActivationDraftUpdate::BackupBeforeWritePlan(value) => {
            draft.form_state.backup_before_write_plan = value
        }
        ProductionActivationDraftUpdate::RestorePlan(value) => {
            draft.form_state.restore_plan = value
        }
        ProductionActivationDraftUpdate::PostWriteRereadPlan(value) => {
            draft.form_state.post_write_reread_plan = value
        }
        ProductionActivationDraftUpdate::PostRestoreVerificationPlan(value) => {
            draft.form_state.post_restore_verification_plan = value
        }
        ProductionActivationDraftUpdate::DryRunSummary(value) => {
            draft.form_state.dry_run_summary = value
        }
        ProductionActivationDraftUpdate::FilesThatWouldBeTouched(value) => {
            draft.form_state.files_that_would_be_touched = value
        }
    }
    draft.dirty = true;
    draft.persisted = false;
    draft.last_validation_status = ProductionActivationDraftStatus::DraftDirty;
}

pub fn reset_production_activation_draft(draft: &mut ProductionActivationDraftForm) {
    *draft = empty_production_activation_draft();
}

pub fn production_activation_draft_edit_state_from_draft(
    draft: ProductionActivationDraftForm,
) -> ProductionActivationDraftEditState {
    ProductionActivationDraftEditState {
        draft,
        mode: ProductionActivationDraftEditMode::DisabledByDefault,
        dirty: false,
        persisted: false,
        production_action_enabled: false,
    }
}

pub fn apply_production_activation_draft_edit_action(
    state: &mut ProductionActivationDraftEditState,
    action: ProductionActivationDraftEditAction,
) -> ProductionActivationDraftEditResult {
    let mut status = match action {
        ProductionActivationDraftEditAction::EnterInMemoryEditMode => {
            state.mode = ProductionActivationDraftEditMode::EnabledInMemoryOnly;
            ProductionActivationDraftEditStatus::DraftEditingEnabledInMemoryOnly
        }
        ProductionActivationDraftEditAction::ApplyUpdate(update) => {
            if state.mode == ProductionActivationDraftEditMode::EnabledInMemoryOnly {
                apply_production_activation_draft_update(&mut state.draft, update);
                state.dirty = true;
                ProductionActivationDraftEditStatus::DraftEditingDirty
            } else {
                ProductionActivationDraftEditStatus::DraftEditingDisabledByDefault
            }
        }
        ProductionActivationDraftEditAction::ResetToDefault => {
            if state.mode == ProductionActivationDraftEditMode::EnabledInMemoryOnly {
                reset_production_activation_draft(&mut state.draft);
                state.dirty = false;
                ProductionActivationDraftEditStatus::DraftEditingEnabledInMemoryOnly
            } else {
                ProductionActivationDraftEditStatus::DraftEditingDisabledByDefault
            }
        }
        ProductionActivationDraftEditAction::ExitEditMode => {
            state.mode = ProductionActivationDraftEditMode::DisabledByDefault;
            ProductionActivationDraftEditStatus::DraftEditingDisabledByDefault
        }
    };

    state.persisted = false;
    state.production_action_enabled = false;
    if state.persisted {
        status = ProductionActivationDraftEditStatus::DraftEditingPersistenceUnavailable;
    }

    ProductionActivationDraftEditResult {
        status,
        state: state.clone(),
        draft_validation_status: state.draft.last_validation_status.clone(),
        persistence_status: "In-memory only".to_string(),
        production_action_enabled: false,
        executor_wired: false,
    }
}

pub fn source_include_activation_draft_review(
    path: Option<&ProductionActivationPathReview>,
    draft: ProductionActivationDraftForm,
    executor_wiring: ProductionExecutorWiringState,
    production_activation_flag: bool,
) -> ProductionActivationDraftReview {
    activation_draft_review(
        path,
        draft,
        executor_wiring,
        production_activation_flag,
        ActivationDraftSpec {
            widget_name: "hyprland-settings-source-include-activation-draft-disabled",
            evidence_widget_name: "hyprland-settings-source-include-activation-draft-evidence",
            disabled_update_widget_name:
                "hyprland-settings-source-include-activation-draft-update-disabled",
            disabled_reset_widget_name:
                "hyprland-settings-source-include-activation-draft-reset-disabled",
            heading: "Source/include activation draft",
            production_label: "Production source/include insertion",
            disabled_update_label: "Update source/include activation draft (planned)",
            disabled_reset_label: "Reset source/include activation draft (planned)",
            expected_scope: ProductionActivationRequestScope::SourceIncludeInsertion,
        },
    )
}

pub fn source_include_activation_draft_edit_review(
    path: Option<&ProductionActivationPathReview>,
    state: ProductionActivationDraftEditState,
    executor_wiring: ProductionExecutorWiringState,
    production_activation_flag: bool,
) -> ProductionActivationDraftEditReview {
    activation_draft_edit_review(
        path,
        state,
        executor_wiring,
        production_activation_flag,
        ActivationDraftEditSpec {
            widget_name: "hyprland-settings-source-include-activation-draft-edit-disabled",
            evidence_widget_name: "hyprland-settings-source-include-activation-draft-edit-evidence",
            mode_widget_name:
                "hyprland-settings-source-include-activation-draft-edit-mode-disabled",
            disabled_update_widget_name:
                "hyprland-settings-source-include-activation-draft-edit-update-disabled",
            disabled_reset_widget_name:
                "hyprland-settings-source-include-activation-draft-edit-reset-disabled",
            heading: "Source/include activation draft editing",
            production_label: "Production source/include insertion",
            disabled_update_label: "Update source/include activation draft (planned)",
            disabled_reset_label: "Reset source/include activation draft (planned)",
            expected_scope: ProductionActivationRequestScope::SourceIncludeInsertion,
        },
    )
}

pub fn duplicate_activation_draft_review(
    path: Option<&ProductionActivationPathReview>,
    draft: ProductionActivationDraftForm,
    executor_wiring: ProductionExecutorWiringState,
    production_activation_flag: bool,
) -> ProductionActivationDraftReview {
    activation_draft_review(
        path,
        draft,
        executor_wiring,
        production_activation_flag,
        ActivationDraftSpec {
            widget_name: "hyprland-settings-duplicate-activation-draft-disabled",
            evidence_widget_name: "hyprland-settings-duplicate-activation-draft-evidence",
            disabled_update_widget_name:
                "hyprland-settings-duplicate-activation-draft-update-disabled",
            disabled_reset_widget_name:
                "hyprland-settings-duplicate-activation-draft-reset-disabled",
            heading: "Duplicate activation draft",
            production_label: "Production duplicate writes",
            disabled_update_label: "Update duplicate activation draft (planned)",
            disabled_reset_label: "Reset duplicate activation draft (planned)",
            expected_scope: ProductionActivationRequestScope::DuplicateReplacement,
        },
    )
}

pub fn duplicate_activation_draft_edit_review(
    path: Option<&ProductionActivationPathReview>,
    state: ProductionActivationDraftEditState,
    executor_wiring: ProductionExecutorWiringState,
    production_activation_flag: bool,
) -> ProductionActivationDraftEditReview {
    activation_draft_edit_review(
        path,
        state,
        executor_wiring,
        production_activation_flag,
        ActivationDraftEditSpec {
            widget_name: "hyprland-settings-duplicate-activation-draft-edit-disabled",
            evidence_widget_name: "hyprland-settings-duplicate-activation-draft-edit-evidence",
            mode_widget_name: "hyprland-settings-duplicate-activation-draft-edit-mode-disabled",
            disabled_update_widget_name:
                "hyprland-settings-duplicate-activation-draft-edit-update-disabled",
            disabled_reset_widget_name:
                "hyprland-settings-duplicate-activation-draft-edit-reset-disabled",
            heading: "Duplicate activation draft editing",
            production_label: "Production duplicate writes",
            disabled_update_label: "Update duplicate activation draft (planned)",
            disabled_reset_label: "Reset duplicate activation draft (planned)",
            expected_scope: ProductionActivationRequestScope::DuplicateReplacement,
        },
    )
}

pub fn source_include_activation_form_review(
    path: Option<&ProductionActivationPathReview>,
    form: ProductionActivationFormState,
    executor_wiring: ProductionExecutorWiringState,
    production_activation_flag: bool,
) -> ProductionActivationFormReview {
    activation_form_review(
        path,
        form,
        executor_wiring,
        production_activation_flag,
        ActivationFormSpec {
            widget_name: "hyprland-settings-source-include-activation-form-disabled",
            evidence_widget_name: "hyprland-settings-source-include-activation-form-evidence",
            disabled_action_widget_name:
                "hyprland-settings-source-include-activation-form-validate-disabled",
            heading: "Source/include activation request form",
            production_label: "Production source/include insertion",
            disabled_action_label: "Validate source/include activation form (planned)",
            expected_scope: ProductionActivationRequestScope::SourceIncludeInsertion,
            expected_category: "sourceIncludeInsertion",
        },
    )
}

pub fn duplicate_activation_form_review(
    path: Option<&ProductionActivationPathReview>,
    form: ProductionActivationFormState,
    executor_wiring: ProductionExecutorWiringState,
    production_activation_flag: bool,
) -> ProductionActivationFormReview {
    activation_form_review(
        path,
        form,
        executor_wiring,
        production_activation_flag,
        ActivationFormSpec {
            widget_name: "hyprland-settings-duplicate-activation-form-disabled",
            evidence_widget_name: "hyprland-settings-duplicate-activation-form-evidence",
            disabled_action_widget_name:
                "hyprland-settings-duplicate-activation-form-validate-disabled",
            heading: "Duplicate activation request form",
            production_label: "Production duplicate writes",
            disabled_action_label: "Validate duplicate activation form (planned)",
            expected_scope: ProductionActivationRequestScope::DuplicateReplacement,
            expected_category: "duplicateReplacement",
        },
    )
}

pub fn production_activation_control_request(
    scope: ProductionActivationRequestScope,
    decision_category: &str,
) -> ProductionActivationRequest {
    ProductionActivationRequest {
        scope,
        user_facing_reason: "final review-only production activation control validation"
            .to_string(),
        decision_category: decision_category.to_string(),
        explicit_activation_token: "VALIDATE REVIEW ONLY - KEEP PRODUCTION DISABLED".to_string(),
        backup_plan_acknowledged: true,
        restore_plan_acknowledged: true,
        reread_plan_acknowledged: true,
        final_confirmation_acknowledged: true,
        one_shot_nonce: Some("review-only-control".to_string()),
    }
}

pub fn production_activation_control_safety_plan() -> ProductionActivationSafetyPlan {
    ProductionActivationSafetyPlan {
        backup_before_write_plan: Some(
            "create byte-exact backup before any future production write".to_string(),
        ),
        restore_plan: Some("restore original bytes before any review completes".to_string()),
        post_write_reread_plan: Some("reread target after any future write".to_string()),
        post_restore_verification_plan: Some(
            "verify restored hash or symlink target after restore".to_string(),
        ),
        dry_run_summary: Some(
            "dry-run must show exact target, exact old state, and exact proposed line".to_string(),
        ),
        files_that_would_be_touched: vec!["report-backed selected target only".to_string()],
    }
}

pub fn source_include_activation_control_review(
    path: Option<&ProductionActivationPathReview>,
    request: Option<&ProductionActivationRequest>,
    safety_plan: Option<&ProductionActivationSafetyPlan>,
    executor_wiring: ProductionExecutorWiringState,
    production_activation_flag: bool,
) -> ProductionActivationControlReview {
    activation_control_review(
        path,
        request,
        safety_plan,
        executor_wiring,
        production_activation_flag,
        ActivationControlSpec {
            widget_name: "hyprland-settings-source-include-activation-control-disabled",
            evidence_widget_name: "hyprland-settings-source-include-activation-control-evidence",
            disabled_action_widget_name:
                "hyprland-settings-source-include-activation-control-validate-disabled",
            heading: "Source/include production activation control",
            production_label: "Production source/include insertion",
            disabled_action_label: "Validate source/include activation request (planned)",
            expected_scope: ProductionActivationRequestScope::SourceIncludeInsertion,
            expected_category: "sourceIncludeInsertion",
        },
    )
}

pub fn duplicate_activation_control_review(
    path: Option<&ProductionActivationPathReview>,
    request: Option<&ProductionActivationRequest>,
    safety_plan: Option<&ProductionActivationSafetyPlan>,
    executor_wiring: ProductionExecutorWiringState,
    production_activation_flag: bool,
) -> ProductionActivationControlReview {
    activation_control_review(
        path,
        request,
        safety_plan,
        executor_wiring,
        production_activation_flag,
        ActivationControlSpec {
            widget_name: "hyprland-settings-duplicate-activation-control-disabled",
            evidence_widget_name: "hyprland-settings-duplicate-activation-control-evidence",
            disabled_action_widget_name:
                "hyprland-settings-duplicate-activation-control-validate-disabled",
            heading: "Duplicate production activation control",
            production_label: "Production duplicate writes",
            disabled_action_label: "Validate duplicate activation request (planned)",
            expected_scope: ProductionActivationRequestScope::DuplicateReplacement,
            expected_category: "duplicateReplacement",
        },
    )
}

struct ActivationDraftSpec {
    widget_name: &'static str,
    evidence_widget_name: &'static str,
    disabled_update_widget_name: &'static str,
    disabled_reset_widget_name: &'static str,
    heading: &'static str,
    production_label: &'static str,
    disabled_update_label: &'static str,
    disabled_reset_label: &'static str,
    expected_scope: ProductionActivationRequestScope,
}

struct ActivationDraftEditSpec {
    widget_name: &'static str,
    evidence_widget_name: &'static str,
    mode_widget_name: &'static str,
    disabled_update_widget_name: &'static str,
    disabled_reset_widget_name: &'static str,
    heading: &'static str,
    production_label: &'static str,
    disabled_update_label: &'static str,
    disabled_reset_label: &'static str,
    expected_scope: ProductionActivationRequestScope,
}

fn activation_draft_review(
    path: Option<&ProductionActivationPathReview>,
    mut draft: ProductionActivationDraftForm,
    executor_wiring: ProductionExecutorWiringState,
    production_activation_flag: bool,
    spec: ActivationDraftSpec,
) -> ProductionActivationDraftReview {
    let form_review = match spec.expected_scope {
        ProductionActivationRequestScope::SourceIncludeInsertion => {
            source_include_activation_form_review(
                path,
                draft.form_state.clone(),
                executor_wiring,
                production_activation_flag,
            )
        }
        ProductionActivationRequestScope::DuplicateReplacement => duplicate_activation_form_review(
            path,
            draft.form_state.clone(),
            executor_wiring,
            production_activation_flag,
        ),
    };
    let status = match form_review.status {
        ProductionActivationFormStatus::Empty => ProductionActivationDraftStatus::EmptyDraft,
        ProductionActivationFormStatus::MissingRequiredFields => {
            ProductionActivationDraftStatus::DraftMissingRequiredFields
        }
        ProductionActivationFormStatus::InvalidWrongScope => {
            ProductionActivationDraftStatus::DraftInvalidWrongScope
        }
        ProductionActivationFormStatus::InvalidWrongCategory => {
            ProductionActivationDraftStatus::DraftInvalidWrongCategory
        }
        ProductionActivationFormStatus::BlockedExecutorWired => {
            ProductionActivationDraftStatus::DraftBlockedExecutorWired
        }
        ProductionActivationFormStatus::BlockedProductionFlagTrue => {
            ProductionActivationDraftStatus::DraftBlockedProductionFlagTrue
        }
        ProductionActivationFormStatus::ValidatedForReviewOnly => {
            ProductionActivationDraftStatus::DraftValidatedForReviewOnly
        }
    };
    draft.last_validation_status = status.clone();
    draft.persisted = false;
    let dirty_state = if draft.dirty {
        "Dirty in-memory draft"
    } else {
        "Clean in-memory draft"
    }
    .to_string();

    ProductionActivationDraftReview {
        widget_name: spec.widget_name.to_string(),
        evidence_widget_name: spec.evidence_widget_name.to_string(),
        disabled_update_widget_name: spec.disabled_update_widget_name.to_string(),
        disabled_reset_widget_name: spec.disabled_reset_widget_name.to_string(),
        heading: spec.heading.to_string(),
        draft,
        status,
        form_validation_status: form_review.status,
        control_validation_status: form_review.control_validation_status,
        dirty_state,
        persistence_status: "In-memory only".to_string(),
        request_generation_status: form_review.request_generation_status,
        safety_plan_generation_status: form_review.safety_plan_generation_status,
        executor_wiring_status: executor_wiring,
        production_label: spec.production_label.to_string(),
        production_status: form_review.production_status,
        production_activation_enabled: false,
        category_production_enabled: false,
        disabled_update_label: spec.disabled_update_label.to_string(),
        disabled_reset_label: spec.disabled_reset_label.to_string(),
    }
}

fn activation_draft_edit_review(
    path: Option<&ProductionActivationPathReview>,
    mut state: ProductionActivationDraftEditState,
    executor_wiring: ProductionExecutorWiringState,
    production_activation_flag: bool,
    spec: ActivationDraftEditSpec,
) -> ProductionActivationDraftEditReview {
    let draft_review = match spec.expected_scope {
        ProductionActivationRequestScope::SourceIncludeInsertion => {
            source_include_activation_draft_review(
                path,
                state.draft.clone(),
                executor_wiring,
                production_activation_flag,
            )
        }
        ProductionActivationRequestScope::DuplicateReplacement => {
            duplicate_activation_draft_review(
                path,
                state.draft.clone(),
                executor_wiring,
                production_activation_flag,
            )
        }
    };
    state.persisted = false;
    state.production_action_enabled = false;

    let status = if executor_wiring != ProductionExecutorWiringState::Unwired {
        ProductionActivationDraftEditStatus::DraftEditingBlockedExecutorWired
    } else if production_activation_flag {
        ProductionActivationDraftEditStatus::DraftEditingBlockedProductionFlagTrue
    } else if state.mode == ProductionActivationDraftEditMode::DisabledByDefault {
        ProductionActivationDraftEditStatus::DraftEditingDisabledByDefault
    } else if draft_review.status == ProductionActivationDraftStatus::DraftValidatedForReviewOnly {
        ProductionActivationDraftEditStatus::DraftEditingValidatedForReviewOnly
    } else if state.dirty || state.draft.dirty {
        ProductionActivationDraftEditStatus::DraftEditingDirty
    } else {
        ProductionActivationDraftEditStatus::DraftEditingEnabledInMemoryOnly
    };
    let dirty_state = if state.dirty || state.draft.dirty {
        "Dirty in-memory draft"
    } else {
        "Clean in-memory draft"
    }
    .to_string();

    ProductionActivationDraftEditReview {
        widget_name: spec.widget_name.to_string(),
        evidence_widget_name: spec.evidence_widget_name.to_string(),
        mode_widget_name: spec.mode_widget_name.to_string(),
        disabled_update_widget_name: spec.disabled_update_widget_name.to_string(),
        disabled_reset_widget_name: spec.disabled_reset_widget_name.to_string(),
        heading: spec.heading.to_string(),
        mode: state.mode,
        status,
        draft_status: draft_review.status,
        form_validation_status: draft_review.form_validation_status,
        control_validation_status: draft_review.control_validation_status,
        dirty_state,
        persistence_status: "In-memory only".to_string(),
        executor_wiring_status: executor_wiring,
        production_label: spec.production_label.to_string(),
        production_status: draft_review.production_status,
        production_activation_enabled: false,
        category_production_enabled: false,
        disabled_update_label: spec.disabled_update_label.to_string(),
        disabled_reset_label: spec.disabled_reset_label.to_string(),
    }
}

struct ActivationFormSpec {
    widget_name: &'static str,
    evidence_widget_name: &'static str,
    disabled_action_widget_name: &'static str,
    heading: &'static str,
    production_label: &'static str,
    disabled_action_label: &'static str,
    expected_scope: ProductionActivationRequestScope,
    expected_category: &'static str,
}

fn activation_form_review(
    path: Option<&ProductionActivationPathReview>,
    form: ProductionActivationFormState,
    executor_wiring: ProductionExecutorWiringState,
    production_activation_flag: bool,
    spec: ActivationFormSpec,
) -> ProductionActivationFormReview {
    let missing_fields = activation_form_missing_fields(&form);
    let scope_status = form
        .scope
        .as_ref()
        .map(|scope| scope == &spec.expected_scope)
        .unwrap_or(false);
    let category_status = form.decision_category == spec.expected_category;
    let request = activation_form_request(&form);
    let safety_plan = activation_form_safety_plan(&form);
    let control = match spec.expected_scope {
        ProductionActivationRequestScope::SourceIncludeInsertion => {
            source_include_activation_control_review(
                path,
                request.as_ref(),
                safety_plan.as_ref(),
                executor_wiring,
                production_activation_flag,
            )
        }
        ProductionActivationRequestScope::DuplicateReplacement => {
            duplicate_activation_control_review(
                path,
                request.as_ref(),
                safety_plan.as_ref(),
                executor_wiring,
                production_activation_flag,
            )
        }
    };

    let status = if form_is_empty(&form) {
        ProductionActivationFormStatus::Empty
    } else if !scope_status {
        ProductionActivationFormStatus::InvalidWrongScope
    } else if !category_status {
        ProductionActivationFormStatus::InvalidWrongCategory
    } else if executor_wiring != ProductionExecutorWiringState::Unwired {
        ProductionActivationFormStatus::BlockedExecutorWired
    } else if production_activation_flag {
        ProductionActivationFormStatus::BlockedProductionFlagTrue
    } else if !missing_fields.is_empty() {
        ProductionActivationFormStatus::MissingRequiredFields
    } else {
        ProductionActivationFormStatus::ValidatedForReviewOnly
    };

    let request_generation_status = if request.is_some() {
        "ProductionActivationRequest generated for review only".to_string()
    } else {
        "ProductionActivationRequest not generated".to_string()
    };
    let safety_plan_generation_status = if safety_plan.is_some() {
        "ProductionActivationSafetyPlan generated for review only".to_string()
    } else {
        "ProductionActivationSafetyPlan not generated".to_string()
    };
    let request_preview = activation_form_request_preview(&form);
    let safety_plan_preview = activation_form_safety_plan_preview(&form);

    ProductionActivationFormReview {
        widget_name: spec.widget_name.to_string(),
        evidence_widget_name: spec.evidence_widget_name.to_string(),
        disabled_action_widget_name: spec.disabled_action_widget_name.to_string(),
        heading: spec.heading.to_string(),
        form_state: form,
        status,
        missing_fields,
        request_preview,
        safety_plan_preview,
        control_validation_status: control.status,
        request_generation_status,
        safety_plan_generation_status,
        executor_wiring_status: executor_wiring,
        production_label: spec.production_label.to_string(),
        production_status: control.production_status,
        production_activation_enabled: false,
        category_production_enabled: false,
        disabled_action_label: spec.disabled_action_label.to_string(),
    }
}

fn activation_form_request(
    form: &ProductionActivationFormState,
) -> Option<ProductionActivationRequest> {
    if activation_form_missing_request_fields(form).is_empty() {
        Some(ProductionActivationRequest {
            scope: form.scope.clone()?,
            user_facing_reason: form.user_facing_reason.clone(),
            decision_category: form.decision_category.clone(),
            explicit_activation_token: form.explicit_activation_token.clone(),
            backup_plan_acknowledged: form.backup_plan_acknowledged,
            restore_plan_acknowledged: form.restore_plan_acknowledged,
            reread_plan_acknowledged: form.reread_plan_acknowledged,
            final_confirmation_acknowledged: form.final_confirmation_acknowledged
                && form.post_restore_verification_acknowledged,
            one_shot_nonce: Some("review-only-form".to_string()),
        })
    } else {
        None
    }
}

fn activation_form_safety_plan(
    form: &ProductionActivationFormState,
) -> Option<ProductionActivationSafetyPlan> {
    if activation_form_missing_safety_plan_fields(form).is_empty() {
        Some(ProductionActivationSafetyPlan {
            backup_before_write_plan: Some(form.backup_before_write_plan.clone()),
            restore_plan: Some(form.restore_plan.clone()),
            post_write_reread_plan: Some(form.post_write_reread_plan.clone()),
            post_restore_verification_plan: Some(form.post_restore_verification_plan.clone()),
            dry_run_summary: Some(form.dry_run_summary.clone()),
            files_that_would_be_touched: form.files_that_would_be_touched.clone(),
        })
    } else {
        None
    }
}

fn activation_form_missing_fields(form: &ProductionActivationFormState) -> Vec<String> {
    let mut fields = activation_form_missing_request_fields(form);
    fields.extend(activation_form_missing_safety_plan_fields(form));
    fields
}

fn activation_form_missing_request_fields(form: &ProductionActivationFormState) -> Vec<String> {
    let mut missing = Vec::new();
    if form.scope.is_none() {
        missing.push("category/scope".to_string());
    }
    if form.user_facing_reason.trim().is_empty() {
        missing.push("user-facing reason".to_string());
    }
    if form.explicit_activation_token.trim().is_empty() {
        missing.push("explicit activation phrase/token".to_string());
    }
    if form.decision_category.trim().is_empty() {
        missing.push("decision category".to_string());
    }
    if !form.backup_plan_acknowledged {
        missing.push("backup-before-write acknowledgement".to_string());
    }
    if !form.restore_plan_acknowledged {
        missing.push("restore-plan acknowledgement".to_string());
    }
    if !form.reread_plan_acknowledged {
        missing.push("post-write reread acknowledgement".to_string());
    }
    if !form.post_restore_verification_acknowledged {
        missing.push("post-restore verification acknowledgement".to_string());
    }
    if !form.final_confirmation_acknowledged {
        missing.push("final confirmation acknowledgement".to_string());
    }
    missing
}

fn activation_form_missing_safety_plan_fields(form: &ProductionActivationFormState) -> Vec<String> {
    let mut missing = Vec::new();
    if form.backup_before_write_plan.trim().is_empty() {
        missing.push("backup-before-write plan text".to_string());
    }
    if form.restore_plan.trim().is_empty() {
        missing.push("restore plan text".to_string());
    }
    if form.post_write_reread_plan.trim().is_empty() {
        missing.push("post-write reread plan text".to_string());
    }
    if form.post_restore_verification_plan.trim().is_empty() {
        missing.push("post-restore verification plan text".to_string());
    }
    if form.dry_run_summary.trim().is_empty() {
        missing.push("dry-run summary text".to_string());
    }
    if form.files_that_would_be_touched.is_empty() {
        missing.push("files-that-would-be-touched list".to_string());
    }
    missing
}

fn activation_form_request_preview(form: &ProductionActivationFormState) -> Vec<String> {
    vec![
        format!(
            "scope = {}",
            form.scope
                .as_ref()
                .map(activation_scope_label)
                .unwrap_or("missing")
        ),
        format!(
            "decision category = {}",
            missing_safe(&form.decision_category)
        ),
        format!("reason = {}", missing_safe(&form.user_facing_reason)),
        format!(
            "activation token = {}",
            missing_safe(&form.explicit_activation_token)
        ),
        format!(
            "backup/restore/reread/final acknowledgements = {}/{}/{}/{}",
            form.backup_plan_acknowledged,
            form.restore_plan_acknowledged,
            form.reread_plan_acknowledged,
            form.final_confirmation_acknowledged
        ),
    ]
}

fn activation_form_safety_plan_preview(form: &ProductionActivationFormState) -> Vec<String> {
    vec![
        format!(
            "backup plan = {}",
            missing_safe(&form.backup_before_write_plan)
        ),
        format!("restore plan = {}", missing_safe(&form.restore_plan)),
        format!(
            "post-write reread plan = {}",
            missing_safe(&form.post_write_reread_plan)
        ),
        format!(
            "post-restore verification plan = {}",
            missing_safe(&form.post_restore_verification_plan)
        ),
        format!("dry-run summary = {}", missing_safe(&form.dry_run_summary)),
        format!(
            "files that would be touched = {}",
            if form.files_that_would_be_touched.is_empty() {
                "Missing from form".to_string()
            } else {
                form.files_that_would_be_touched.join(", ")
            }
        ),
    ]
}

fn activation_scope_label(scope: &ProductionActivationRequestScope) -> &'static str {
    match scope {
        ProductionActivationRequestScope::SourceIncludeInsertion => "source/include",
        ProductionActivationRequestScope::DuplicateReplacement => "duplicate",
    }
}

fn missing_safe(value: &str) -> String {
    if value.trim().is_empty() {
        "Missing from form".to_string()
    } else {
        value.to_string()
    }
}

fn form_is_empty(form: &ProductionActivationFormState) -> bool {
    form.scope.is_none()
        && form.user_facing_reason.is_empty()
        && form.explicit_activation_token.is_empty()
        && form.decision_category.is_empty()
        && !form.backup_plan_acknowledged
        && !form.restore_plan_acknowledged
        && !form.reread_plan_acknowledged
        && !form.post_restore_verification_acknowledged
        && !form.final_confirmation_acknowledged
        && form.backup_before_write_plan.is_empty()
        && form.restore_plan.is_empty()
        && form.post_write_reread_plan.is_empty()
        && form.post_restore_verification_plan.is_empty()
        && form.dry_run_summary.is_empty()
        && form.files_that_would_be_touched.is_empty()
}

struct ActivationPathSpec {
    widget_name: &'static str,
    evidence_widget_name: &'static str,
    disabled_action_widget_name: &'static str,
    heading: &'static str,
    production_label: &'static str,
    disabled_action_label: &'static str,
    expected_scope: ProductionActivationRequestScope,
    expected_category: &'static str,
}

fn activation_path_review(
    decision: Option<&ProductionActivationDecisionReview>,
    card: Option<&DisabledApprovalCardProjection>,
    request: Option<&ProductionActivationRequest>,
    safety_plan: Option<&ProductionActivationSafetyPlan>,
    production_activation_flag: bool,
    spec: ActivationPathSpec,
) -> ProductionActivationPathReview {
    let mut blockers = Vec::new();
    let input_decision_status = decision
        .map(|decision| decision.status.clone())
        .unwrap_or(ProductionActivationDecisionStatus::MissingReportBackedCard);
    let input_proof_source = card
        .map(|card| card.proof_record.source.clone())
        .unwrap_or_else(|| "Missing report-backed approval card".to_string());
    let production_status = decision
        .map(|decision| decision.production_status.clone())
        .or_else(|| card.map(|card| card.production_status.clone()))
        .unwrap_or_else(|| "Disabled".to_string());

    if decision.is_none() {
        blockers.push("activation decision review is missing".to_string());
    } else if input_decision_status
        != ProductionActivationDecisionStatus::ApprovedButDefaultDisabled
    {
        blockers.push(format!(
            "input decision must be ApprovedButDefaultDisabled, got {}",
            input_decision_status.user_facing_label()
        ));
    }
    if card.is_none() || missing_report_value(&input_proof_source) {
        blockers.push("report-backed approval card proof is missing".to_string());
    }
    if production_status != "Disabled" {
        blockers.push(format!(
            "production status must be Disabled, got {production_status}"
        ));
    }
    if production_activation_flag {
        blockers.push(
            "production activation flag was true; this default-disabled review refuses enablement"
                .to_string(),
        );
    } else {
        blockers.push("category-specific production activation flag is false".to_string());
    }

    match request {
        Some(request) => {
            if request.scope != spec.expected_scope {
                blockers
                    .push("activation request scope does not match decision category".to_string());
            }
            if request.decision_category != spec.expected_category {
                blockers.push("activation request decision category does not match".to_string());
            }
            if request.explicit_activation_token.trim().is_empty() {
                blockers.push("explicit production activation token is missing".to_string());
            }
            if request.user_facing_reason.trim().is_empty() {
                blockers.push("user-facing activation reason is missing".to_string());
            }
            if !request.backup_plan_acknowledged {
                blockers.push("backup-before-write plan acknowledgement is missing".to_string());
            }
            if !request.restore_plan_acknowledged {
                blockers.push("restore plan acknowledgement is missing".to_string());
            }
            if !request.reread_plan_acknowledged {
                blockers.push("post-write reread plan acknowledgement is missing".to_string());
            }
            if !request.final_confirmation_acknowledged {
                blockers.push("final confirmation acknowledgement is missing".to_string());
            }
        }
        None => blockers.push("explicit production activation request is missing".to_string()),
    }

    match safety_plan {
        Some(plan) => {
            if plan
                .backup_before_write_plan
                .as_deref()
                .is_none_or(str::is_empty)
            {
                blockers.push("backup-before-write plan is missing".to_string());
            }
            if plan.restore_plan.as_deref().is_none_or(str::is_empty) {
                blockers.push("restore plan is missing".to_string());
            }
            if plan
                .post_write_reread_plan
                .as_deref()
                .is_none_or(str::is_empty)
            {
                blockers.push("post-write reread plan is missing".to_string());
            }
            if plan
                .post_restore_verification_plan
                .as_deref()
                .is_none_or(str::is_empty)
            {
                blockers.push("post-restore verification plan is missing".to_string());
            }
            if plan.dry_run_summary.as_deref().is_none_or(str::is_empty) {
                blockers.push("dry-run summary is missing".to_string());
            }
            if plan.files_that_would_be_touched.is_empty() {
                blockers.push("files that would be touched are missing".to_string());
            }
        }
        None => blockers.push("backup/restore/reread safety plan is missing".to_string()),
    }

    let status = activation_path_status(&blockers, production_activation_flag);
    ProductionActivationPathReview {
        widget_name: spec.widget_name.to_string(),
        evidence_widget_name: spec.evidence_widget_name.to_string(),
        disabled_action_widget_name: spec.disabled_action_widget_name.to_string(),
        heading: spec.heading.to_string(),
        input_decision_status,
        input_proof_source,
        status,
        required_before_enabling: vec![
            "explicit production activation request".to_string(),
            "explicit user approval".to_string(),
            "category-specific production activation flag".to_string(),
            "backup-before-write plan".to_string(),
            "restore plan".to_string(),
            "post-write reread plan".to_string(),
            "post-restore verification plan".to_string(),
            "dry-run summary".to_string(),
            "clear list of files that would be touched".to_string(),
            "final confirmation".to_string(),
        ],
        blockers,
        production_label: spec.production_label.to_string(),
        production_status,
        production_activation_enabled: false,
        category_production_enabled: false,
        disabled_action_label: spec.disabled_action_label.to_string(),
    }
}

fn activation_path_status(
    blockers: &[String],
    production_activation_flag: bool,
) -> ProductionActivationPathStatus {
    if blockers.iter().any(|blocker| {
        blocker.contains("production status must be Disabled")
            || blocker.contains("production activation flag was true")
            || blocker.contains("input decision must be ApprovedButDefaultDisabled")
            || blocker.contains("activation decision review is missing")
            || blocker.contains("report-backed approval card proof is missing")
    }) {
        return ProductionActivationPathStatus::ActivationPathBlocked;
    }
    if !production_activation_flag {
        return ProductionActivationPathStatus::ActivationPathNeedsExplicitProductionFlag;
    }
    if blockers.iter().any(|blocker| {
        blocker.contains("backup")
            || blocker.contains("restore")
            || blocker.contains("reread")
            || blocker.contains("dry-run")
            || blocker.contains("files that would be touched")
    }) {
        return ProductionActivationPathStatus::ActivationPathNeedsRealBackupRestorePlan;
    }
    if blockers.iter().any(|blocker| {
        blocker.contains("request")
            || blocker.contains("approval")
            || blocker.contains("confirmation")
            || blocker.contains("token")
            || blocker.contains("reason")
    }) {
        return ProductionActivationPathStatus::ActivationPathNeedsFinalUserApproval;
    }
    ProductionActivationPathStatus::ActivationPathReadyButDefaultDisabled
}

struct ActivationControlSpec {
    widget_name: &'static str,
    evidence_widget_name: &'static str,
    disabled_action_widget_name: &'static str,
    heading: &'static str,
    production_label: &'static str,
    disabled_action_label: &'static str,
    expected_scope: ProductionActivationRequestScope,
    expected_category: &'static str,
}

fn activation_control_review(
    path: Option<&ProductionActivationPathReview>,
    request: Option<&ProductionActivationRequest>,
    safety_plan: Option<&ProductionActivationSafetyPlan>,
    executor_wiring: ProductionExecutorWiringState,
    production_activation_flag: bool,
    spec: ActivationControlSpec,
) -> ProductionActivationControlReview {
    let mut blockers = Vec::new();
    let input_path_status = path
        .map(|path| path.status.clone())
        .unwrap_or(ProductionActivationPathStatus::ActivationPathBlocked);
    let input_decision_status = path
        .map(|path| path.input_decision_status.clone())
        .unwrap_or(ProductionActivationDecisionStatus::MissingReportBackedCard);
    let production_status = path
        .map(|path| path.production_status.clone())
        .unwrap_or_else(|| "Disabled".to_string());

    if path.is_none() {
        blockers.push("activation path review is missing".to_string());
    }
    if input_decision_status != ProductionActivationDecisionStatus::ApprovedButDefaultDisabled {
        blockers.push(format!(
            "input decision must be ApprovedButDefaultDisabled, got {}",
            input_decision_status.user_facing_label()
        ));
    }
    if production_status != "Disabled" {
        blockers.push(format!(
            "production status must be Disabled, got {production_status}"
        ));
    }
    if production_activation_flag
        || path
            .map(|path| path.production_activation_enabled || path.category_production_enabled)
            .unwrap_or(false)
    {
        blockers
            .push("production flags must remain false for final control validation".to_string());
    }

    let request_validation_status = validate_activation_control_request(
        request,
        &mut blockers,
        &spec.expected_scope,
        spec.expected_category,
    );
    let safety_plan_validation_status =
        validate_activation_control_safety_plan(safety_plan, &mut blockers);

    if executor_wiring != ProductionExecutorWiringState::Unwired {
        blockers.push("production executor must remain unwired".to_string());
    }

    let status = activation_control_status(&blockers, request, safety_plan, executor_wiring);
    ProductionActivationControlReview {
        widget_name: spec.widget_name.to_string(),
        evidence_widget_name: spec.evidence_widget_name.to_string(),
        disabled_action_widget_name: spec.disabled_action_widget_name.to_string(),
        heading: spec.heading.to_string(),
        input_path_status,
        input_decision_status,
        status,
        request_validation_status,
        safety_plan_validation_status,
        executor_wiring_status: executor_wiring,
        blockers,
        production_label: spec.production_label.to_string(),
        production_status,
        production_activation_enabled: false,
        category_production_enabled: false,
        disabled_action_label: spec.disabled_action_label.to_string(),
    }
}

fn validate_activation_control_request(
    request: Option<&ProductionActivationRequest>,
    blockers: &mut Vec<String>,
    expected_scope: &ProductionActivationRequestScope,
    expected_category: &str,
) -> String {
    let Some(request) = request else {
        blockers.push("explicit activation request is missing".to_string());
        return "Missing activation request".to_string();
    };

    let mut missing = Vec::new();
    if &request.scope != expected_scope {
        blockers.push("activation request scope does not match control category".to_string());
        missing.push("wrong scope");
    }
    if request.decision_category != expected_category {
        blockers.push("activation request decision category does not match control".to_string());
        missing.push("wrong category");
    }
    if request.user_facing_reason.trim().is_empty() {
        blockers.push("activation request reason is missing".to_string());
        missing.push("reason");
    }
    if request.explicit_activation_token.trim().is_empty() {
        blockers.push("activation token is missing".to_string());
        missing.push("token");
    }
    if !request.backup_plan_acknowledged {
        blockers.push("backup acknowledgement is missing".to_string());
        missing.push("backup acknowledgement");
    }
    if !request.restore_plan_acknowledged {
        blockers.push("restore acknowledgement is missing".to_string());
        missing.push("restore acknowledgement");
    }
    if !request.reread_plan_acknowledged {
        blockers.push("reread acknowledgement is missing".to_string());
        missing.push("reread acknowledgement");
    }
    if !request.final_confirmation_acknowledged {
        blockers.push("final confirmation acknowledgement is missing".to_string());
        missing.push("final confirmation");
    }

    if missing.is_empty() {
        "Complete activation request".to_string()
    } else {
        format!("Incomplete activation request: {}", missing.join(", "))
    }
}

fn validate_activation_control_safety_plan(
    safety_plan: Option<&ProductionActivationSafetyPlan>,
    blockers: &mut Vec<String>,
) -> String {
    let Some(plan) = safety_plan else {
        blockers.push("activation safety plan is missing".to_string());
        return "Missing safety plan".to_string();
    };

    let mut missing = Vec::new();
    if plan
        .backup_before_write_plan
        .as_deref()
        .is_none_or(str::is_empty)
    {
        blockers.push("backup-before-write plan is missing".to_string());
        missing.push("backup-before-write plan");
    }
    if plan.restore_plan.as_deref().is_none_or(str::is_empty) {
        blockers.push("restore plan is missing".to_string());
        missing.push("restore plan");
    }
    if plan
        .post_write_reread_plan
        .as_deref()
        .is_none_or(str::is_empty)
    {
        blockers.push("post-write reread plan is missing".to_string());
        missing.push("post-write reread plan");
    }
    if plan
        .post_restore_verification_plan
        .as_deref()
        .is_none_or(str::is_empty)
    {
        blockers.push("post-restore verification plan is missing".to_string());
        missing.push("post-restore verification plan");
    }
    if plan.dry_run_summary.as_deref().is_none_or(str::is_empty) {
        blockers.push("dry-run summary is missing".to_string());
        missing.push("dry-run summary");
    }
    if plan.files_that_would_be_touched.is_empty() {
        blockers.push("files that would be touched list is missing".to_string());
        missing.push("files that would be touched");
    }

    if missing.is_empty() {
        "Complete safety plan".to_string()
    } else {
        format!("Incomplete safety plan: {}", missing.join(", "))
    }
}

fn activation_control_status(
    blockers: &[String],
    request: Option<&ProductionActivationRequest>,
    safety_plan: Option<&ProductionActivationSafetyPlan>,
    executor_wiring: ProductionExecutorWiringState,
) -> ProductionActivationControlStatus {
    if blockers
        .iter()
        .any(|blocker| blocker.contains("activation path review is missing"))
    {
        return ProductionActivationControlStatus::MissingActivationPath;
    }
    if request.is_none() {
        return ProductionActivationControlStatus::MissingActivationRequest;
    }
    if blockers
        .iter()
        .any(|blocker| blocker.contains("scope does not match"))
    {
        return ProductionActivationControlStatus::WrongScope;
    }
    if blockers
        .iter()
        .any(|blocker| blocker.contains("decision category does not match"))
    {
        return ProductionActivationControlStatus::WrongCategory;
    }
    if blockers
        .iter()
        .any(|blocker| blocker.contains("production flags must remain false"))
    {
        return ProductionActivationControlStatus::ProductionFlagMustRemainFalse;
    }
    if executor_wiring != ProductionExecutorWiringState::Unwired {
        return ProductionActivationControlStatus::ExecutorMustRemainUnwired;
    }
    if safety_plan.is_none() {
        return ProductionActivationControlStatus::MissingSafetyPlan;
    }
    if blockers.iter().any(|blocker| {
        blocker.contains("backup acknowledgement")
            || blocker.contains("backup-before-write plan is missing")
    }) {
        return ProductionActivationControlStatus::MissingBackupPlan;
    }
    if blockers.iter().any(|blocker| {
        blocker.contains("restore acknowledgement") || blocker == "restore plan is missing"
    }) {
        return ProductionActivationControlStatus::MissingRestorePlan;
    }
    if blockers.iter().any(|blocker| {
        blocker.contains("reread acknowledgement") || blocker.contains("reread plan is missing")
    }) {
        return ProductionActivationControlStatus::MissingRereadPlan;
    }
    if blockers.iter().any(|blocker| {
        blocker.contains("final confirmation")
            || blocker.contains("activation token")
            || blocker.contains("reason is missing")
    }) {
        return ProductionActivationControlStatus::MissingFinalConfirmation;
    }
    if blockers.iter().any(|blocker| {
        blocker.contains("post-restore verification")
            || blocker.contains("dry-run summary")
            || blocker.contains("files that would be touched")
    }) {
        return ProductionActivationControlStatus::MissingSafetyPlan;
    }
    if blockers.iter().any(|blocker| {
        blocker.contains("input decision must be ApprovedButDefaultDisabled")
            || blocker.contains("production status must be Disabled")
    }) {
        return ProductionActivationControlStatus::Blocked;
    }
    ProductionActivationControlStatus::ValidatedButExecutorUnwired
}

struct ActivationDecisionSpec {
    widget_name: &'static str,
    evidence_widget_name: &'static str,
    disabled_action_widget_name: &'static str,
    heading: &'static str,
    production_label: &'static str,
    disabled_action_label: &'static str,
    required_proof_fields: &'static [&'static str],
    required_preconditions: &'static [&'static str],
    required_restore_evidence: &'static [&'static str],
    original_unchanged_label: &'static str,
}

fn activation_decision_review(
    card: Option<&DisabledApprovalCardProjection>,
    input_source: &str,
    spec: ActivationDecisionSpec,
) -> ProductionActivationDecisionReview {
    let Some(card) = card else {
        return blocked_activation_decision_review(
            input_source,
            spec,
            ProductionActivationDecisionStatus::MissingReportBackedCard,
            vec!["report-backed approval card is missing".to_string()],
        );
    };

    let mut blockers = Vec::new();
    let mut status = ProductionActivationDecisionStatus::ReadyButDefaultDisabled;
    if card.production_enabled {
        status = ProductionActivationDecisionStatus::ProductionAlreadyEnabledError;
        blockers.push("production_enabled was true in report-backed card data".to_string());
    }
    if missing_report_value(&card.proof_record.source) {
        status = first_blocking_status(
            status,
            ProductionActivationDecisionStatus::MissingProofSource,
        );
        blockers.push("proof source is missing from report-backed card data".to_string());
    }
    if missing_report_value(&card.proof_record.status) {
        status = first_blocking_status(
            status,
            ProductionActivationDecisionStatus::MissingProofStatus,
        );
        blockers.push("proof status is missing from report-backed card data".to_string());
    }
    for required in spec.required_proof_fields {
        if missing_labeled_value(&card.proof_record.fields, required) {
            status = first_blocking_status(
                status,
                ProductionActivationDecisionStatus::MissingRequiredProofField,
            );
            blockers.push(format!("required proof field missing: {required}"));
        }
    }
    for required in spec.required_preconditions {
        if missing_precondition(&card.preconditions, required) {
            status = first_blocking_status(
                status,
                ProductionActivationDecisionStatus::MissingPrecondition,
            );
            blockers.push(format!("required precondition missing: {required}"));
        }
    }
    for required in spec.required_restore_evidence {
        if missing_restore_evidence(&card.restore_evidence, required) {
            status = first_blocking_status(
                status,
                ProductionActivationDecisionStatus::MissingRestoreEvidence,
            );
            blockers.push(format!("required restore evidence missing: {required}"));
        }
    }
    if missing_restore_evidence(&card.restore_evidence, spec.original_unchanged_label) {
        status = first_blocking_status(
            status,
            ProductionActivationDecisionStatus::MissingOriginalUnchangedProof,
        );
        blockers.push(format!(
            "required original unchanged proof missing: {}",
            spec.original_unchanged_label
        ));
    }
    let approval_status = card
        .evidence_lines
        .iter()
        .find(|(label, _)| label.eq_ignore_ascii_case("Approval status"))
        .map(|(_, value)| value.as_str());
    if approval_status.is_none_or(missing_report_value) {
        status = first_blocking_status(
            status,
            ProductionActivationDecisionStatus::MissingApprovalStatus,
        );
        blockers.push("approval status is missing from report-backed card data".to_string());
    }
    if !card.production_status.eq_ignore_ascii_case("Disabled") {
        status = first_blocking_status(status, ProductionActivationDecisionStatus::Blocked);
        blockers.push(format!(
            "production status must be Disabled, got {}",
            card.production_status
        ));
    }
    if blockers.is_empty()
        && approval_status
            .map(|value| value.to_ascii_lowercase().contains("approved"))
            .unwrap_or(false)
    {
        status = ProductionActivationDecisionStatus::ApprovedButDefaultDisabled;
    }

    ProductionActivationDecisionReview {
        widget_name: spec.widget_name.to_string(),
        evidence_widget_name: spec.evidence_widget_name.to_string(),
        disabled_action_widget_name: spec.disabled_action_widget_name.to_string(),
        heading: spec.heading.to_string(),
        input_source: input_source.to_string(),
        status,
        required_proof_summary: activation_required_proof_summary(card, &spec),
        blockers,
        production_label: spec.production_label.to_string(),
        production_status: card.production_status.clone(),
        production_enabled: false,
        disabled_action_label: spec.disabled_action_label.to_string(),
    }
}

fn blocked_activation_decision_review(
    input_source: &str,
    spec: ActivationDecisionSpec,
    status: ProductionActivationDecisionStatus,
    blockers: Vec<String>,
) -> ProductionActivationDecisionReview {
    ProductionActivationDecisionReview {
        widget_name: spec.widget_name.to_string(),
        evidence_widget_name: spec.evidence_widget_name.to_string(),
        disabled_action_widget_name: spec.disabled_action_widget_name.to_string(),
        heading: spec.heading.to_string(),
        input_source: input_source.to_string(),
        status,
        required_proof_summary: Vec::new(),
        blockers,
        production_label: spec.production_label.to_string(),
        production_status: "Disabled".to_string(),
        production_enabled: false,
        disabled_action_label: spec.disabled_action_label.to_string(),
    }
}

fn activation_required_proof_summary(
    card: &DisabledApprovalCardProjection,
    spec: &ActivationDecisionSpec,
) -> Vec<String> {
    let mut summary = vec![
        format!("proof source = {}", card.proof_record.source),
        format!("proof status = {}", card.proof_record.status),
    ];
    summary.extend(spec.required_proof_fields.iter().map(|label| {
        format!(
            "{label} = {}",
            labeled_value(&card.proof_record.fields, label).unwrap_or("Missing from report")
        )
    }));
    summary.extend(spec.required_preconditions.iter().map(|label| {
        format!(
            "{label} = {}",
            precondition_status(&card.preconditions, label).unwrap_or("Missing from report")
        )
    }));
    summary.extend(spec.required_restore_evidence.iter().map(|label| {
        format!(
            "{label} = {}",
            restore_status(&card.restore_evidence, label).unwrap_or("Missing from report")
        )
    }));
    summary.push(format!(
        "{} = {}",
        spec.original_unchanged_label,
        restore_status(&card.restore_evidence, spec.original_unchanged_label)
            .unwrap_or("Missing from report")
    ));
    summary
}

fn first_blocking_status(
    current: ProductionActivationDecisionStatus,
    next: ProductionActivationDecisionStatus,
) -> ProductionActivationDecisionStatus {
    match current {
        ProductionActivationDecisionStatus::ReadyButDefaultDisabled
        | ProductionActivationDecisionStatus::ApprovedButDefaultDisabled => next,
        _ => current,
    }
}

fn missing_report_value(value: &str) -> bool {
    let value = value.trim();
    value.is_empty()
        || value.eq_ignore_ascii_case("Missing from report")
        || value.eq_ignore_ascii_case("Report unavailable")
        || value.eq_ignore_ascii_case("Not proven yet")
}

fn missing_labeled_value(values: &[(String, String)], label: &str) -> bool {
    labeled_value(values, label).is_none_or(missing_report_value)
}

fn labeled_value<'a>(values: &'a [(String, String)], label: &str) -> Option<&'a str> {
    values
        .iter()
        .find(|(candidate, _)| candidate.eq_ignore_ascii_case(label))
        .map(|(_, value)| value.as_str())
}

fn missing_precondition(values: &[ApprovalCardPreconditionLine], label: &str) -> bool {
    precondition_status(values, label).is_none_or(missing_report_value)
}

fn precondition_status<'a>(
    values: &'a [ApprovalCardPreconditionLine],
    label: &str,
) -> Option<&'a str> {
    values
        .iter()
        .find(|candidate| candidate.label.eq_ignore_ascii_case(label))
        .map(|candidate| candidate.status.as_str())
}

fn missing_restore_evidence(values: &[ApprovalCardRestoreEvidence], label: &str) -> bool {
    restore_status(values, label).is_none_or(missing_report_value)
}

fn restore_status<'a>(values: &'a [ApprovalCardRestoreEvidence], label: &str) -> Option<&'a str> {
    values
        .iter()
        .find(|candidate| candidate.label.eq_ignore_ascii_case(label))
        .map(|candidate| candidate.status.as_str())
}

fn cards_from_serialized_report(
    report: SerializedDisabledApprovalCardsReport,
) -> Vec<DisabledApprovalCardProjection> {
    let mut fallback = fallback_disabled_future_approval_card_projections().into_iter();
    [
        report.cards.source_include_insertion,
        report.cards.duplicate_replacement,
        report.cards.structured_hl_bind_write,
        report.cards.profile_mode_switch,
        report.cards.high_risk_display_write,
        report.cards.hyprland0554_migration,
    ]
    .into_iter()
    .map(|record| {
        let fallback_card = fallback
            .next()
            .expect("fallback card inventory must match serialized report order");
        card_from_serialized_record(record, fallback_card)
    })
    .collect()
}

fn card_from_serialized_record(
    record: SerializedApprovalCardRecord,
    mut fallback: DisabledApprovalCardProjection,
) -> DisabledApprovalCardProjection {
    let evidence_lines = serialized_evidence_lines(&record, &fallback);
    fallback.widget_name = record.widget_name.unwrap_or(fallback.widget_name);
    fallback.evidence_widget_name = record
        .evidence_widget_name
        .unwrap_or(fallback.evidence_widget_name);
    fallback.disabled_action_widget_name = record
        .disabled_action_widget_name
        .unwrap_or(fallback.disabled_action_widget_name);
    fallback.proof_record = ApprovalCardProofRecord {
        source: record
            .proof_source
            .unwrap_or_else(|| "Missing from report".to_string()),
        status: record
            .proof_status
            .unwrap_or_else(|| "Missing from report".to_string()),
        fields: map_to_labeled_pairs(record.proof_fields),
    };
    fallback.evidence_lines = evidence_lines;
    fallback.preconditions =
        map_to_precondition_lines(record.preconditions, &fallback.preconditions);
    fallback.restore_evidence = map_to_restore_evidence(record.restore_evidence);
    fallback.blockers = record
        .blockers
        .filter(|blockers| !blockers.is_empty())
        .unwrap_or_else(|| fallback.blockers.clone());
    fallback.production_status = record
        .production_status
        .unwrap_or_else(|| "Missing from report".to_string());
    fallback.production_enabled = false;
    fallback
}

fn mark_card_report_unavailable(
    mut card: DisabledApprovalCardProjection,
) -> DisabledApprovalCardProjection {
    card.proof_record.source = "Report unavailable".to_string();
    card.proof_record.status = "Report unavailable".to_string();
    card.evidence_lines.push((
        "Report load status".to_string(),
        "Report unavailable".to_string(),
    ));
    card.production_enabled = false;
    card
}

fn serialized_evidence_lines(
    record: &SerializedApprovalCardRecord,
    fallback: &DisabledApprovalCardProjection,
) -> Vec<(String, String)> {
    let mut lines = Vec::new();
    if let Some(fields) = &record.proof_fields {
        lines.extend(
            fields
                .iter()
                .map(|(key, value)| (labelize_key(key), value.clone())),
        );
    }
    if let Some(preconditions) = &record.preconditions {
        lines.extend(
            preconditions
                .iter()
                .map(|(key, value)| (format!("{} status", labelize_key(key)), value.clone())),
        );
    }
    if let Some(restore_evidence) = &record.restore_evidence {
        lines.extend(
            restore_evidence
                .iter()
                .map(|(key, value)| (format!("{} status", labelize_key(key)), value.clone())),
        );
    }
    lines.push((
        "Approval status".to_string(),
        record
            .approval_status
            .clone()
            .unwrap_or_else(|| approval_status_from_fallback(fallback)),
    ));
    if let Some(active_model) = &record.active_model {
        lines.push(("Current active app model".to_string(), active_model.clone()));
    }
    if let Some(migration_status) = &record.migration_status {
        lines.push(("Migration status".to_string(), migration_status.clone()));
    }
    lines.push((
        production_label_for_widget(&fallback.widget_name),
        record
            .production_status
            .clone()
            .unwrap_or_else(|| "Missing from report".to_string()),
    ));
    if lines.is_empty() {
        lines.push(("Report data".to_string(), "Missing from report".to_string()));
    }
    lines
}

fn approval_status_from_fallback(card: &DisabledApprovalCardProjection) -> String {
    card.evidence_lines
        .iter()
        .find(|(label, _)| label == "Approval status")
        .map(|(_, value)| value.clone())
        .unwrap_or_else(|| "Missing from report".to_string())
}

fn production_label_for_widget(widget_name: &str) -> String {
    if widget_name.contains("source-include") {
        "Production source/include insertion".to_string()
    } else if widget_name.contains("duplicate") {
        "Production duplicate writes".to_string()
    } else if widget_name.contains("structured") {
        "Production structured writes".to_string()
    } else if widget_name.contains("profile") {
        "Production profile switching".to_string()
    } else if widget_name.contains("high-risk") {
        "Production high-risk/display writes".to_string()
    } else if widget_name.contains("0554") {
        "Production migration activation".to_string()
    } else {
        "Production status".to_string()
    }
}

fn map_to_labeled_pairs(map: Option<BTreeMap<String, String>>) -> Vec<(String, String)> {
    match map {
        Some(values) if !values.is_empty() => values
            .into_iter()
            .map(|(key, value)| (labelize_key(&key), value))
            .collect(),
        _ => vec![("Report data".to_string(), "Missing from report".to_string())],
    }
}

fn map_to_precondition_lines(
    map: Option<BTreeMap<String, String>>,
    fallback: &[ApprovalCardPreconditionLine],
) -> Vec<ApprovalCardPreconditionLine> {
    match map {
        Some(values) if !values.is_empty() => values
            .into_iter()
            .map(|(key, status)| {
                let label = labelize_key(&key);
                let value = fallback
                    .iter()
                    .find(|line| line.label.eq_ignore_ascii_case(&label))
                    .map(|line| line.value.clone())
                    .unwrap_or_else(|| status.clone());
                ApprovalCardPreconditionLine {
                    label,
                    value,
                    status,
                }
            })
            .collect(),
        _ => vec![ApprovalCardPreconditionLine {
            label: "Report data".to_string(),
            value: "Missing from report".to_string(),
            status: "Missing from report".to_string(),
        }],
    }
}

fn map_to_restore_evidence(
    map: Option<BTreeMap<String, String>>,
) -> Vec<ApprovalCardRestoreEvidence> {
    match map {
        Some(values) if !values.is_empty() => values
            .into_iter()
            .map(|(key, status)| ApprovalCardRestoreEvidence {
                label: labelize_key(&key),
                status,
            })
            .collect(),
        _ => vec![ApprovalCardRestoreEvidence {
            label: "Report data".to_string(),
            status: "Missing from report".to_string(),
        }],
    }
}

fn labelize_key(key: &str) -> String {
    match key {
        "dryRunStatus" => return "dry-run status".to_string(),
        "copiedReplacementStatus" => return "copied replacement status".to_string(),
        "copiedEditStatus" => return "copied edit status".to_string(),
        "commentOrderPreservation" => return "comment/order preservation".to_string(),
        "outOfBandRecovery" => return "out-of-band recovery".to_string(),
        "deadManTimeout" => return "dead-man timeout".to_string(),
        "runtimeReadOnlyEvidence" => return "runtime read-only evidence".to_string(),
        "lowRiskRuntimeLiveRestoreProof" => {
            return "low-risk runtime live-restore proof".to_string()
        }
        "insufficiencyReason" => return "insufficiency reason".to_string(),
        "packageMetadataEvidence" => return "package metadata evidence".to_string(),
        "currentActiveAppModel" => return "current active app model".to_string(),
        "official0554ExportBundle" => return "official 0.55.4 export bundle".to_string(),
        "rowCountDiff" => return "row-count diff".to_string(),
        "writeSafetyReview" => return "write-safety review".to_string(),
        "safeEnvEvidence" => return "safe-env evidence".to_string(),
        _ => {}
    }
    let mut label = String::new();
    let mut previous_was_space = true;
    for character in key.chars() {
        if character == '_' || character == '-' {
            label.push(' ');
            previous_was_space = true;
        } else if character.is_uppercase() {
            if !previous_was_space {
                label.push(' ');
            }
            label.push(character.to_ascii_lowercase());
            previous_was_space = false;
        } else {
            label.push(character);
            previous_was_space = false;
        }
    }
    match label.as_str() {
        "official0554 export bundle" => "official 0.55.4 export bundle".to_string(),
        "hyprland0554 migration" => "Hyprland 0.55.4 migration".to_string(),
        other => other.to_string(),
    }
}

pub fn fallback_disabled_future_approval_card_projections() -> Vec<DisabledApprovalCardProjection> {
    vec![
        DisabledApprovalCardProjection {
            widget_name: "hyprland-settings-source-include-approval-review-disabled".to_string(),
            evidence_widget_name: "hyprland-settings-source-include-approval-evidence".to_string(),
            disabled_action_widget_name:
                "hyprland-settings-source-include-approval-enable-disabled".to_string(),
            heading: "Source/include approval review".to_string(),
            summary_lines: vec![
                "Source/include production insertion is not enabled yet.".to_string(),
                "Copied-config-tree proof exists.".to_string(),
                "Production connected-file insertion remains disabled.".to_string(),
            ],
            proof_record: ApprovalCardProofRecord {
                source: "copied-config-tree proof".to_string(),
                status: "copied_config_tree_proven".to_string(),
                fields: vec![
                    (
                        "root config".to_string(),
                        "copied-config-tree root fixture".to_string(),
                    ),
                    (
                        "selected target".to_string(),
                        "copied source/include target fixture".to_string(),
                    ),
                    ("source depth".to_string(), "1".to_string()),
                    (
                        "dry-run status".to_string(),
                        "selected target plan accepted for copied tree".to_string(),
                    ),
                ],
            },
            preconditions: vec![
                ApprovalCardPreconditionLine {
                    label: "planned inserted line".to_string(),
                    value: "normal scalar setting line from selected-target dry-run proof"
                        .to_string(),
                    status: "matched copied proof".to_string(),
                },
                ApprovalCardPreconditionLine {
                    label: "proposed value".to_string(),
                    value: "reviewed normal scalar value".to_string(),
                    status: "normal scalar only".to_string(),
                },
            ],
            restore_evidence: vec![
                ApprovalCardRestoreEvidence {
                    label: "copied target restore".to_string(),
                    status: "restored byte-for-byte".to_string(),
                },
                ApprovalCardRestoreEvidence {
                    label: "original real config unchanged".to_string(),
                    status: "verified unchanged".to_string(),
                },
            ],
            evidence_lines: vec![
                (
                    "Root config".to_string(),
                    "copied-config-tree root fixture".to_string(),
                ),
                (
                    "Selected target".to_string(),
                    "copied source/include target fixture".to_string(),
                ),
                ("Source depth".to_string(), "1".to_string()),
                (
                    "Planned inserted line".to_string(),
                    "normal scalar setting line from selected-target dry-run proof".to_string(),
                ),
                (
                    "Proposed value".to_string(),
                    "reviewed normal scalar value".to_string(),
                ),
                (
                    "Copied-config-tree proof status".to_string(),
                    "copied_config_tree_proven".to_string(),
                ),
                (
                    "Approval status".to_string(),
                    "Approved but default-disabled".to_string(),
                ),
                (
                    "Production source/include insertion".to_string(),
                    "Disabled".to_string(),
                ),
            ],
            blockers: vec![
                "production flag remains false".to_string(),
                "real connected-file insertion still needs explicit activation".to_string(),
            ],
            disabled_action_label: "Enable source/include insertion (planned)".to_string(),
            production_status: "Disabled".to_string(),
            production_enabled: false,
        },
        DisabledApprovalCardProjection {
            widget_name: "hyprland-settings-duplicate-approval-review-disabled".to_string(),
            evidence_widget_name: "hyprland-settings-duplicate-approval-evidence".to_string(),
            disabled_action_widget_name: "hyprland-settings-duplicate-approval-enable-disabled"
                .to_string(),
            heading: "Duplicate approval review".to_string(),
            summary_lines: vec![
                "Duplicate production writes are not enabled yet.".to_string(),
                "Copied-config-tree proof exists.".to_string(),
                "Production duplicate replacement remains disabled.".to_string(),
            ],
            proof_record: ApprovalCardProofRecord {
                source: "copied-config-tree proof".to_string(),
                status: "copied_config_tree_proven".to_string(),
                fields: vec![
                    (
                        "selected occurrence".to_string(),
                        "confirmed copied occurrence".to_string(),
                    ),
                    (
                        "target path".to_string(),
                        "copied duplicate target fixture".to_string(),
                    ),
                    ("source depth".to_string(), "1".to_string()),
                    (
                        "copied replacement status".to_string(),
                        "selected duplicate replaced and reread in copied tree".to_string(),
                    ),
                ],
            },
            preconditions: vec![
                ApprovalCardPreconditionLine {
                    label: "line number".to_string(),
                    value: "exact precondition line".to_string(),
                    status: "matched copied occurrence".to_string(),
                },
                ApprovalCardPreconditionLine {
                    label: "raw line".to_string(),
                    value: "raw duplicate line from copied proof".to_string(),
                    status: "matched fingerprint".to_string(),
                },
                ApprovalCardPreconditionLine {
                    label: "old value".to_string(),
                    value: "copied proof old value".to_string(),
                    status: "matched old-value precondition".to_string(),
                },
                ApprovalCardPreconditionLine {
                    label: "proposed value".to_string(),
                    value: "copied proof proposed value".to_string(),
                    status: "review only".to_string(),
                },
            ],
            restore_evidence: vec![
                ApprovalCardRestoreEvidence {
                    label: "copied target restore".to_string(),
                    status: "restored byte-for-byte".to_string(),
                },
                ApprovalCardRestoreEvidence {
                    label: "original real config unchanged".to_string(),
                    status: "verified unchanged".to_string(),
                },
            ],
            evidence_lines: vec![
                (
                    "Selected occurrence".to_string(),
                    "confirmed copied occurrence".to_string(),
                ),
                (
                    "Target path".to_string(),
                    "copied duplicate target fixture".to_string(),
                ),
                (
                    "Line number".to_string(),
                    "exact precondition line".to_string(),
                ),
                (
                    "Raw line".to_string(),
                    "raw duplicate line from copied proof".to_string(),
                ),
                (
                    "Old value".to_string(),
                    "copied proof old value".to_string(),
                ),
                (
                    "Proposed value".to_string(),
                    "copied proof proposed value".to_string(),
                ),
                (
                    "Fingerprint/precondition status".to_string(),
                    "matching copied proof precondition".to_string(),
                ),
                (
                    "Copied proof status".to_string(),
                    "copied_config_tree_proven".to_string(),
                ),
                (
                    "Approval status".to_string(),
                    "Approved but default-disabled".to_string(),
                ),
                (
                    "Production duplicate writes".to_string(),
                    "Disabled".to_string(),
                ),
            ],
            blockers: vec![
                "production duplicate write flag remains false".to_string(),
                "Apply is not wired to duplicate replacement".to_string(),
            ],
            disabled_action_label: "Enable duplicate replacement (planned)".to_string(),
            production_status: "Disabled".to_string(),
            production_enabled: false,
        },
        DisabledApprovalCardProjection {
            widget_name: "hyprland-settings-structured-approval-review-disabled".to_string(),
            evidence_widget_name: "hyprland-settings-structured-approval-evidence".to_string(),
            disabled_action_widget_name: "hyprland-settings-structured-approval-enable-disabled"
                .to_string(),
            heading: "Structured hl.bind approval review".to_string(),
            summary_lines: vec![
                "Structured production writes are not enabled yet.".to_string(),
                "Copied-config-tree proof exists.".to_string(),
                "Production hl.bind editing remains disabled.".to_string(),
            ],
            proof_record: ApprovalCardProofRecord {
                source: "copied-config-tree proof".to_string(),
                status: "copied_config_tree_proven".to_string(),
                fields: vec![
                    (
                        "target file".to_string(),
                        "copied bind target fixture".to_string(),
                    ),
                    ("source depth".to_string(), "1".to_string()),
                    (
                        "copied edit status".to_string(),
                        "selected hl.bind line edited and reread in copied tree".to_string(),
                    ),
                    (
                        "comment/order preservation".to_string(),
                        "comments and order preserved".to_string(),
                    ),
                ],
            },
            preconditions: vec![
                ApprovalCardPreconditionLine {
                    label: "line number".to_string(),
                    value: "selected bind line".to_string(),
                    status: "matched copied proof".to_string(),
                },
                ApprovalCardPreconditionLine {
                    label: "old raw line".to_string(),
                    value: "original bind raw line from copied proof".to_string(),
                    status: "matched stale-line precondition".to_string(),
                },
                ApprovalCardPreconditionLine {
                    label: "proposed raw line".to_string(),
                    value: "candidate bind raw line from copied proof".to_string(),
                    status: "valid hl.bind candidate".to_string(),
                },
            ],
            restore_evidence: vec![
                ApprovalCardRestoreEvidence {
                    label: "copied target restore".to_string(),
                    status: "restored byte-for-byte".to_string(),
                },
                ApprovalCardRestoreEvidence {
                    label: "original real config unchanged".to_string(),
                    status: "verified unchanged".to_string(),
                },
            ],
            evidence_lines: vec![
                (
                    "Target file".to_string(),
                    "copied bind target fixture".to_string(),
                ),
                ("Line number".to_string(), "selected bind line".to_string()),
                (
                    "Old raw line".to_string(),
                    "original bind raw line from copied proof".to_string(),
                ),
                (
                    "Proposed raw line".to_string(),
                    "candidate bind raw line from copied proof".to_string(),
                ),
                (
                    "Candidate validation status".to_string(),
                    "valid hl.bind candidate".to_string(),
                ),
                (
                    "Comment/order preservation status".to_string(),
                    "comments and order preserved".to_string(),
                ),
                (
                    "Copied proof status".to_string(),
                    "copied_config_tree_proven".to_string(),
                ),
                (
                    "Approval status".to_string(),
                    "Approved but default-disabled".to_string(),
                ),
                (
                    "Production structured writes".to_string(),
                    "Disabled".to_string(),
                ),
            ],
            blockers: vec![
                "production structured write flag remains false".to_string(),
                "Apply is not wired to structured-family writes".to_string(),
            ],
            disabled_action_label: "Enable structured write (planned)".to_string(),
            production_status: "Disabled".to_string(),
            production_enabled: false,
        },
        DisabledApprovalCardProjection {
            widget_name: "hyprland-settings-profile-approval-review-disabled".to_string(),
            evidence_widget_name: "hyprland-settings-profile-approval-evidence".to_string(),
            disabled_action_widget_name: "hyprland-settings-profile-approval-enable-disabled"
                .to_string(),
            heading: "Profile/mode approval review".to_string(),
            summary_lines: vec![
                "Real profile switching is not enabled yet.".to_string(),
                "Copied symlink proof exists.".to_string(),
                "Real profile/mode switching remains disabled.".to_string(),
            ],
            proof_record: ApprovalCardProofRecord {
                source: "copied-config-tree profile/symlink proof".to_string(),
                status: "copied_config_tree_proven".to_string(),
                fields: vec![
                    (
                        "current symlink".to_string(),
                        "copied current.conf symlink".to_string(),
                    ),
                    (
                        "original target".to_string(),
                        "original copied symlink target".to_string(),
                    ),
                    (
                        "proposed target".to_string(),
                        "selected copied profile target".to_string(),
                    ),
                    (
                        "copied switch status".to_string(),
                        "temp symlink switched to selected copied target".to_string(),
                    ),
                ],
            },
            preconditions: vec![
                ApprovalCardPreconditionLine {
                    label: "selected target".to_string(),
                    value: "selected copied profile target".to_string(),
                    status: "inside copied tree".to_string(),
                },
                ApprovalCardPreconditionLine {
                    label: "original symlink target".to_string(),
                    value: "original copied symlink target".to_string(),
                    status: "snapshot recorded".to_string(),
                },
            ],
            restore_evidence: vec![
                ApprovalCardRestoreEvidence {
                    label: "copied symlink restore".to_string(),
                    status: "restored original copied target".to_string(),
                },
                ApprovalCardRestoreEvidence {
                    label: "real symlink untouched".to_string(),
                    status: "verified untouched".to_string(),
                },
            ],
            evidence_lines: vec![
                (
                    "Current symlink".to_string(),
                    "copied current.conf symlink".to_string(),
                ),
                (
                    "Original target".to_string(),
                    "original copied symlink target".to_string(),
                ),
                (
                    "Proposed target".to_string(),
                    "selected copied profile target".to_string(),
                ),
                (
                    "Copied symlink proof status".to_string(),
                    "copied_config_tree_proven".to_string(),
                ),
                (
                    "Restore proof status".to_string(),
                    "copied symlink restored".to_string(),
                ),
                (
                    "Approval status".to_string(),
                    "Approved but default-disabled".to_string(),
                ),
                (
                    "Production profile switching".to_string(),
                    "Disabled".to_string(),
                ),
            ],
            blockers: vec![
                "real-session live symlink restore proof is still required".to_string(),
                "production profile switching flag remains false".to_string(),
            ],
            disabled_action_label: "Enable profile switching (planned)".to_string(),
            production_status: "Disabled".to_string(),
            production_enabled: false,
        },
        DisabledApprovalCardProjection {
            widget_name: "hyprland-settings-high-risk-approval-review-disabled".to_string(),
            evidence_widget_name: "hyprland-settings-high-risk-approval-evidence".to_string(),
            disabled_action_widget_name: "hyprland-settings-high-risk-approval-enable-disabled"
                .to_string(),
            heading: "High-risk/display approval review".to_string(),
            summary_lines: vec![
                "High-risk display writes are not enabled yet.".to_string(),
                "Runtime live-restore proof is available for a low-risk setting.".to_string(),
                "That proof is not enough to enable high-risk/display writes.".to_string(),
                "Required before high-risk/display activation:".to_string(),
            ],
            proof_record: ApprovalCardProofRecord {
                source: "high-risk readiness gate".to_string(),
                status: "blocked_recovery_proof_missing".to_string(),
                fields: vec![
                    (
                        "runtime read-only evidence".to_string(),
                        "succeeded outside sandbox".to_string(),
                    ),
                    (
                        "low-risk runtime live-restore proof".to_string(),
                        "general:gaps_in restored after hl.config eval proof".to_string(),
                    ),
                    (
                        "insufficiency reason".to_string(),
                        "low-risk runtime proof does not prove display recovery".to_string(),
                    ),
                ],
            },
            preconditions: vec![
                ApprovalCardPreconditionLine {
                    label: "out-of-band recovery".to_string(),
                    value: "missing".to_string(),
                    status: "blocks activation".to_string(),
                },
                ApprovalCardPreconditionLine {
                    label: "dead-man timeout".to_string(),
                    value: "missing".to_string(),
                    status: "blocks activation".to_string(),
                },
                ApprovalCardPreconditionLine {
                    label: "restore command".to_string(),
                    value: "required".to_string(),
                    status: "not proven for display risk".to_string(),
                },
                ApprovalCardPreconditionLine {
                    label: "config backup".to_string(),
                    value: "required".to_string(),
                    status: "not attached to high-risk live proof".to_string(),
                },
                ApprovalCardPreconditionLine {
                    label: "runtime snapshot".to_string(),
                    value: "required".to_string(),
                    status: "not attached to high-risk live proof".to_string(),
                },
            ],
            restore_evidence: vec![ApprovalCardRestoreEvidence {
                label: "high-risk restoration".to_string(),
                status: "not proven; no display mutation attempted".to_string(),
            }],
            evidence_lines: vec![
                ("Out-of-band recovery".to_string(), "Missing".to_string()),
                ("Dead-man timeout".to_string(), "Missing".to_string()),
                ("Restore command".to_string(), "Required".to_string()),
                ("Config backup".to_string(), "Required".to_string()),
                ("Runtime snapshot".to_string(), "Required".to_string()),
                ("Explicit approval".to_string(), "Required".to_string()),
                (
                    "Approval status".to_string(),
                    "Blocked by missing recovery proof".to_string(),
                ),
                (
                    "Production high-risk/display writes".to_string(),
                    "Disabled".to_string(),
                ),
            ],
            blockers: vec![
                "out-of-band recovery proof is missing".to_string(),
                "dead-man restore proof is missing".to_string(),
            ],
            disabled_action_label: "Enable high-risk/display writes (planned)".to_string(),
            production_status: "Disabled".to_string(),
            production_enabled: false,
        },
        DisabledApprovalCardProjection {
            widget_name: "hyprland-settings-0554-approval-review-disabled".to_string(),
            evidence_widget_name: "hyprland-settings-0554-approval-evidence".to_string(),
            disabled_action_widget_name: "hyprland-settings-0554-approval-enable-disabled"
                .to_string(),
            heading: "Hyprland 0.55.4 migration review".to_string(),
            summary_lines: vec![
                "Hyprland 0.55.4 activation is not enabled yet.".to_string(),
                "Runtime version evidence exists.".to_string(),
                "Package metadata evidence exists.".to_string(),
                "These are advisory only.".to_string(),
                "Required before activation:".to_string(),
            ],
            proof_record: ApprovalCardProofRecord {
                source: "runtime/package/trusted-data records".to_string(),
                status: "blocked_missing_trusted_exports".to_string(),
                fields: vec![
                    (
                        "runtime version evidence".to_string(),
                        "Hyprland 0.55.4 commit a0136d8c04687bb36eb8a28eb9d1ff92aea99704"
                            .to_string(),
                    ),
                    (
                        "package metadata evidence".to_string(),
                        "hyprland 0.55.4-1".to_string(),
                    ),
                    (
                        "current active app model".to_string(),
                        "v0.55.2".to_string(),
                    ),
                    (
                        "advisory evidence status".to_string(),
                        "runtime/package evidence cannot activate migration".to_string(),
                    ),
                ],
            },
            preconditions: vec![
                ApprovalCardPreconditionLine {
                    label: "official 0.55.4 export bundle".to_string(),
                    value: "missing".to_string(),
                    status: "blocks activation".to_string(),
                },
                ApprovalCardPreconditionLine {
                    label: "row-count diff".to_string(),
                    value: "missing".to_string(),
                    status: "blocks activation".to_string(),
                },
                ApprovalCardPreconditionLine {
                    label: "write-safety review".to_string(),
                    value: "missing".to_string(),
                    status: "blocks activation".to_string(),
                },
                ApprovalCardPreconditionLine {
                    label: "safe-env evidence".to_string(),
                    value: "missing".to_string(),
                    status: "blocks activation".to_string(),
                },
                ApprovalCardPreconditionLine {
                    label: "explicit approval".to_string(),
                    value: "required".to_string(),
                    status: "not sufficient without trusted inputs".to_string(),
                },
            ],
            restore_evidence: vec![ApprovalCardRestoreEvidence {
                label: "migration activation".to_string(),
                status: "inactive; v0.55.2 remains active".to_string(),
            }],
            evidence_lines: vec![
                (
                    "Official 0.55.4 export bundle".to_string(),
                    "Missing".to_string(),
                ),
                ("Row-count diff".to_string(), "Missing".to_string()),
                ("Write-safety review".to_string(), "Missing".to_string()),
                ("Safe-env evidence".to_string(), "Missing".to_string()),
                ("Explicit approval".to_string(), "Required".to_string()),
                (
                    "Current active app model".to_string(),
                    "v0.55.2".to_string(),
                ),
                ("Migration status".to_string(), "Inactive".to_string()),
                (
                    "Production migration activation".to_string(),
                    "Disabled".to_string(),
                ),
            ],
            blockers: vec![
                "trusted official 0.55.4 export bundle is missing".to_string(),
                "row-count diff, write-safety review, and safe-env evidence are missing"
                    .to_string(),
            ],
            disabled_action_label: "Enable 0.55.4 migration (planned)".to_string(),
            production_status: "Disabled".to_string(),
            production_enabled: false,
        },
    ]
}

pub fn runtime_live_restore_approval_review(
    action: RuntimeAction,
    live_restore_proof: Option<&RuntimeLiveRestoreProof>,
    syntax_evidence: Option<&RuntimeEvalSyntaxEvidence>,
    approval_request: Option<&ApprovalRequest>,
    production_flag_enabled: bool,
) -> RuntimeApprovalReview {
    let mut blockers = Vec::new();
    let setting = match &action {
        RuntimeAction::Keyword { key, .. } => key.clone(),
        RuntimeAction::Reload => "reload".to_string(),
        RuntimeAction::Dispatch { command } => format!("dispatch:{command}"),
        RuntimeAction::Status { query } => format!("status:{query}"),
    };

    let Some(proof) = live_restore_proof else {
        return runtime_approval_review_blocked(
            action,
            RuntimeApprovalReviewStatus::MissingLiveRestoreProof,
            None,
            None,
            production_flag_enabled,
            "proven runtime live-restore proof is required",
        );
    };
    if proof.status != RuntimeLiveRestoreStatus::LiveRestoreProven
        || !proof.restored
        || proof.production_runtime_enabled
    {
        return runtime_approval_review_blocked(
            action,
            RuntimeApprovalReviewStatus::FailedLiveRestoreProof,
            None,
            None,
            production_flag_enabled,
            "runtime live-restore proof must be proven, restored, and production-disabled",
        );
    }

    let Some(syntax) = syntax_evidence else {
        return runtime_approval_review_blocked(
            action,
            RuntimeApprovalReviewStatus::MissingMutationSyntaxEvidence,
            None,
            None,
            production_flag_enabled,
            "runtime mutation syntax evidence is required",
        );
    };
    if syntax.setting != setting {
        return runtime_approval_review_blocked(
            action,
            RuntimeApprovalReviewStatus::WrongSetting,
            None,
            None,
            production_flag_enabled,
            "runtime syntax evidence setting does not match the requested action",
        );
    }
    if !syntax.live_restore_proven || !syntax.runtime_left_restored {
        return runtime_approval_review_blocked(
            action,
            RuntimeApprovalReviewStatus::MutationSyntaxNotProven,
            None,
            None,
            production_flag_enabled,
            "runtime mutation syntax did not prove mutation and restore",
        );
    }

    let Some(successful_syntax) = syntax.successful_syntax.as_deref() else {
        return runtime_approval_review_blocked(
            action,
            RuntimeApprovalReviewStatus::MutationSyntaxNotProven,
            None,
            None,
            production_flag_enabled,
            "no successful runtime mutation syntax is recorded",
        );
    };
    let Some(success_candidate) = syntax.candidates.iter().find(|candidate| {
        candidate.syntax_name == successful_syntax
            && candidate.status == RuntimeMutationSyntaxStatus::MutatedAndRestored
    }) else {
        return runtime_approval_review_blocked(
            action,
            RuntimeApprovalReviewStatus::MutationSyntaxNotProven,
            None,
            None,
            production_flag_enabled,
            "successful runtime mutation syntax candidate is missing",
        );
    };

    if proof.restore_command.as_deref()
        != Some(success_candidate.command_pair.restore_command.as_str())
    {
        return runtime_approval_review_blocked(
            action,
            RuntimeApprovalReviewStatus::RestoreCommandMismatch,
            None,
            None,
            production_flag_enabled,
            "live restore proof restore command does not match the proven syntax restore command",
        );
    }
    if proof.prior_value.as_deref() != Some(syntax.prior_value.as_str())
        || proof.temporary_value.as_deref() != Some(syntax.temporary_value.as_str())
        || proof.post_mutation_value.as_deref() != Some(syntax.temporary_value.as_str())
        || proof.post_restore_value.as_deref() != Some(syntax.prior_value.as_str())
    {
        return runtime_approval_review_blocked(
            action,
            RuntimeApprovalReviewStatus::FailedLiveRestoreProof,
            None,
            None,
            production_flag_enabled,
            "live restore proof readbacks do not match mutation syntax evidence",
        );
    }

    let evidence = RuntimeLiveRestoreApprovalEvidence {
        setting: setting.clone(),
        prior_value: syntax.prior_value.clone(),
        temporary_value: syntax.temporary_value.clone(),
        mutation_command: success_candidate.command_pair.mutation_command.clone(),
        restore_command: success_candidate.command_pair.restore_command.clone(),
        post_mutation_readback: proof.post_mutation_value.clone().unwrap_or_default(),
        post_restore_readback: proof.post_restore_value.clone().unwrap_or_default(),
        restoration_verified: proof.restored,
    };

    let decision = approval_decision_for_gate(
        ApprovalScope::RuntimeKeyword,
        true,
        None,
        Some(evidence.mutation_command.as_str()),
        approval_request,
        false,
    );
    let status = match decision.status {
        ApprovalStatus::MissingEvidence if approval_request.is_none() => {
            RuntimeApprovalReviewStatus::MissingApproval
        }
        ApprovalStatus::MissingEvidence => RuntimeApprovalReviewStatus::MissingApprovalEvidence,
        ApprovalStatus::WrongScope => RuntimeApprovalReviewStatus::WrongApprovalScope,
        ApprovalStatus::Rejected => RuntimeApprovalReviewStatus::ApprovalRejected,
        ApprovalStatus::Expired => RuntimeApprovalReviewStatus::ApprovalExpired,
        ApprovalStatus::Pending => RuntimeApprovalReviewStatus::ApprovalPending,
        ApprovalStatus::ReadyButDefaultDisabled | ApprovalStatus::ApprovedButDefaultDisabled => {
            RuntimeApprovalReviewStatus::ApprovedButDefaultDisabled
        }
        ApprovalStatus::Enabled => RuntimeApprovalReviewStatus::ApprovedButDefaultDisabled,
    };
    blockers.extend(decision.blockers.clone());
    if production_flag_enabled {
        blockers.push(
            "runtime production activation is not wired in this default-disabled review"
                .to_string(),
        );
    }
    if status == RuntimeApprovalReviewStatus::ApprovedButDefaultDisabled
        && !blockers
            .iter()
            .any(|blocker| blocker.contains("default-disabled"))
    {
        blockers.push("runtime production mutation flag is default-disabled".to_string());
    }

    RuntimeApprovalReview {
        action,
        status,
        live_restore_evidence: Some(evidence),
        approval_decision: Some(decision),
        production_flag_enabled,
        production_runtime_enabled: false,
        blockers: blockers.clone(),
        review_lines: vec![
            "Runtime approval review consumes the proven hl.config eval live-restore proof."
                .to_string(),
            "The approval must name the exact runtime command, old value, proposed value, and restore plan."
                .to_string(),
            "Production runtime/reload remains disabled by default.".to_string(),
            format!(
                "Blockers: {}",
                if blockers.is_empty() {
                    "none".to_string()
                } else {
                    blockers.join("; ")
                }
            ),
        ],
    }
}

fn runtime_approval_review_blocked(
    action: RuntimeAction,
    status: RuntimeApprovalReviewStatus,
    live_restore_evidence: Option<RuntimeLiveRestoreApprovalEvidence>,
    approval_decision: Option<ApprovalDecision>,
    production_flag_enabled: bool,
    blocker: &str,
) -> RuntimeApprovalReview {
    RuntimeApprovalReview {
        action,
        status,
        live_restore_evidence,
        approval_decision,
        production_flag_enabled,
        production_runtime_enabled: false,
        blockers: vec![blocker.to_string()],
        review_lines: vec![
            "Runtime approval review is blocked.".to_string(),
            blocker.to_string(),
            "Production runtime/reload remains disabled.".to_string(),
        ],
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RuntimeLiveRestoreProof {
    pub action: RuntimeAction,
    pub status: RuntimeLiveRestoreStatus,
    pub prior_value: Option<String>,
    pub temporary_value: Option<String>,
    pub restore_command: Option<String>,
    pub post_mutation_value: Option<String>,
    pub post_restore_value: Option<String>,
    pub real_command_executed: bool,
    pub runtime_touched: bool,
    pub restored: bool,
    pub production_runtime_enabled: bool,
    pub blockers: Vec<String>,
    pub review_lines: Vec<String>,
}

pub fn runtime_live_restore_proof_review(
    action: RuntimeAction,
    read_only_evidence_available: bool,
    prior_value: Option<&str>,
    temporary_value: Option<&str>,
    post_mutation_value: Option<&str>,
    post_restore_value: Option<&str>,
    live_mutation_executed: bool,
) -> RuntimeLiveRestoreProof {
    let mut blockers = Vec::new();
    let mut status = RuntimeLiveRestoreStatus::ReadyButDefaultDisabled;
    if !read_only_evidence_available {
        status = RuntimeLiveRestoreStatus::ReadOnlyEvidenceMissing;
        blockers.push("read-only runtime evidence is required before live mutation".to_string());
    }
    if prior_value.is_none() {
        status = RuntimeLiveRestoreStatus::PriorValueMissing;
        blockers.push("prior runtime value is required".to_string());
    }
    if temporary_value.is_none() {
        status = RuntimeLiveRestoreStatus::TemporaryValueMissing;
        blockers.push("temporary runtime value is required".to_string());
    }
    let restore_command = match (&action, prior_value) {
        (RuntimeAction::Keyword { key, .. }, Some(value)) => {
            Some(format!("hyprctl keyword {key} {value}"))
        }
        _ => None,
    };
    if restore_command.is_none() {
        status = RuntimeLiveRestoreStatus::RestoreCommandMissing;
        blockers.push("restore command must be generated before mutation".to_string());
    }
    if live_mutation_executed && post_mutation_value.is_none() {
        status = RuntimeLiveRestoreStatus::PostMutationReadbackMissing;
        blockers.push("post-mutation readback is required".to_string());
    }
    let restored =
        live_mutation_executed && prior_value.is_some() && post_restore_value == prior_value;
    if live_mutation_executed && !restored {
        status = RuntimeLiveRestoreStatus::PostRestoreVerificationFailed;
        blockers.push("post-restore readback did not match the prior value".to_string());
    }
    if blockers.is_empty() && live_mutation_executed && restored {
        status = RuntimeLiveRestoreStatus::LiveRestoreProven;
    } else if blockers.is_empty() {
        status = RuntimeLiveRestoreStatus::ReadyButDefaultDisabled;
        blockers.push("runtime live mutation was not executed in this proof".to_string());
    }

    RuntimeLiveRestoreProof {
        action,
        status,
        prior_value: prior_value.map(ToOwned::to_owned),
        temporary_value: temporary_value.map(ToOwned::to_owned),
        restore_command,
        post_mutation_value: post_mutation_value.map(ToOwned::to_owned),
        post_restore_value: post_restore_value.map(ToOwned::to_owned),
        real_command_executed: live_mutation_executed,
        runtime_touched: live_mutation_executed,
        restored,
        production_runtime_enabled: false,
        blockers: blockers.clone(),
        review_lines: vec![
            "Runtime live-restore proof requires prior value, generated restore command, post-mutation readback, and post-restore verification.".to_string(),
            "Production runtime mutation remains default-disabled even after proof.".to_string(),
            format!(
                "Blockers: {}",
                if blockers.is_empty() {
                    "none".to_string()
                } else {
                    blockers.join("; ")
                }
            ),
        ],
    }
}

pub fn runtime_live_restore_attempt_review(
    action: RuntimeAction,
    read_only_evidence_available: bool,
    prior_value: Option<&str>,
    temporary_value: Option<&str>,
    restore_command: Option<&str>,
    mutation_command: Option<&str>,
    mutation_command_succeeded: bool,
    post_mutation_value: Option<&str>,
    post_restore_value: Option<&str>,
) -> RuntimeLiveRestoreProof {
    let mut blockers = Vec::new();
    let mut status = RuntimeLiveRestoreStatus::ReadyButDefaultDisabled;
    if !read_only_evidence_available {
        status = RuntimeLiveRestoreStatus::ReadOnlyEvidenceMissing;
        blockers.push("read-only runtime evidence is required before live mutation".to_string());
    }
    if prior_value.is_none() {
        status = RuntimeLiveRestoreStatus::PriorValueMissing;
        blockers.push("prior runtime value is required".to_string());
    }
    if temporary_value.is_none() {
        status = RuntimeLiveRestoreStatus::TemporaryValueMissing;
        blockers.push("temporary runtime value is required".to_string());
    }
    if restore_command.is_none() {
        status = RuntimeLiveRestoreStatus::RestoreCommandMissing;
        blockers.push("restore command must be generated before mutation".to_string());
    }
    if mutation_command.is_none() {
        status = RuntimeLiveRestoreStatus::LiveRestoreBlocked;
        blockers.push("mutation command must be explicit before mutation".to_string());
    }
    if mutation_command.is_some()
        && restore_command.is_some()
        && read_only_evidence_available
        && prior_value.is_some()
        && temporary_value.is_some()
        && !mutation_command_succeeded
    {
        status = RuntimeLiveRestoreStatus::LiveRestoreBlocked;
        blockers
            .push("runtime mutation command failed before a value change was verified".to_string());
    }
    if mutation_command_succeeded && post_mutation_value.is_none() {
        status = RuntimeLiveRestoreStatus::PostMutationReadbackMissing;
        blockers.push("post-mutation readback is required".to_string());
    }
    let restored =
        mutation_command_succeeded && prior_value.is_some() && post_restore_value == prior_value;
    if mutation_command_succeeded && !restored {
        status = RuntimeLiveRestoreStatus::PostRestoreVerificationFailed;
        blockers.push("post-restore readback did not match the prior value".to_string());
    }
    if blockers.is_empty() && mutation_command_succeeded && restored {
        status = RuntimeLiveRestoreStatus::LiveRestoreProven;
    }

    RuntimeLiveRestoreProof {
        action,
        status,
        prior_value: prior_value.map(ToOwned::to_owned),
        temporary_value: temporary_value.map(ToOwned::to_owned),
        restore_command: restore_command.map(ToOwned::to_owned),
        post_mutation_value: post_mutation_value.map(ToOwned::to_owned),
        post_restore_value: post_restore_value.map(ToOwned::to_owned),
        real_command_executed: mutation_command_succeeded,
        runtime_touched: mutation_command_succeeded,
        restored,
        production_runtime_enabled: false,
        blockers: blockers.clone(),
        review_lines: vec![
            "Runtime live-restore attempts require read-only evidence, prior value, generated restore command, mutation command, readback, and restore verification.".to_string(),
            "Failed mutation syntax is recorded as blocked and must not enable production runtime mutation.".to_string(),
            format!(
                "Blockers: {}",
                if blockers.is_empty() {
                    "none".to_string()
                } else {
                    blockers.join("; ")
                }
            ),
        ],
    }
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

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct VersionedDataBundle {
    pub version: String,
    pub readable_rows: usize,
    pub writable_rows: usize,
    pub blocked_rows: usize,
    pub default_model: bool,
    pub trusted_source: bool,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct MigrationComparisonReview {
    pub current_default: VersionedDataBundle,
    pub requested_version: String,
    pub requested_bundle: Option<VersionedDataBundle>,
    pub trusted_source_requirement_met: bool,
    pub missing_proof: Vec<String>,
    pub migration_enabled: bool,
    pub production_default_changed: bool,
    pub review_lines: Vec<String>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct DisabledMigrationReview {
    pub current_default: VersionedDataBundle,
    pub requested_version: String,
    pub migration_enabled: bool,
    pub review_lines: Vec<String>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct TrustedExportRequirement {
    pub requested_version: String,
    pub has_official_export: bool,
    pub has_row_count_diff: bool,
    pub has_write_safety_review: bool,
    pub has_safe_env_evidence: bool,
    pub can_activate: bool,
    pub missing_inputs: Vec<String>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct LocalHyprlandVersionEvidence {
    pub requested_version: String,
    pub installed_package_version: Option<String>,
    pub runtime_binary_version: Option<String>,
    pub official_export_available: bool,
    pub row_count_diff_available: bool,
    pub write_safety_review_available: bool,
    pub safe_env_evidence_available: bool,
    pub user_approval_recorded: bool,
    pub activation_allowed: bool,
    pub production_default_changed: bool,
    pub evidence_lines: Vec<String>,
    pub missing_inputs: Vec<String>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum HyprlandVersionActivationStatus {
    PartialEvidenceOnly,
    MissingOfficialExport,
    MissingRowCountDiff,
    MissingWriteSafetyReview,
    MissingSafeEnvEvidence,
    MissingUserApproval,
    ReadyButDefaultDisabled,
    Enabled,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct HyprlandVersionActivationGate {
    pub requested_version: String,
    pub current_default_version: String,
    pub status: HyprlandVersionActivationStatus,
    pub local_package_evidence: Option<String>,
    pub local_runtime_evidence: Option<String>,
    pub official_export_available: bool,
    pub row_count_diff_available: bool,
    pub write_safety_review_available: bool,
    pub safe_env_evidence_available: bool,
    pub user_approval_recorded: bool,
    pub activation_flag_enabled: bool,
    pub migration_activated: bool,
    pub production_default_changed: bool,
    pub blockers: Vec<String>,
    pub review_lines: Vec<String>,
}

pub fn current_v0552_data_bundle() -> VersionedDataBundle {
    VersionedDataBundle {
        version: "0.55.2".to_string(),
        readable_rows: 341,
        writable_rows: 341,
        blocked_rows: 0,
        default_model: true,
        trusted_source: true,
    }
}

pub fn hyprland_version_activation_gate(
    evidence: &LocalHyprlandVersionEvidence,
    activation_flag_enabled: bool,
) -> HyprlandVersionActivationGate {
    let current_default = current_v0552_data_bundle();
    let mut blockers = Vec::new();
    let mut status = HyprlandVersionActivationStatus::ReadyButDefaultDisabled;
    if evidence.requested_version == current_default.version {
        status = HyprlandVersionActivationStatus::Enabled;
    } else {
        let has_only_advisory_evidence = (evidence.installed_package_version.is_some()
            || evidence.runtime_binary_version.is_some())
            && (!evidence.official_export_available
                || !evidence.row_count_diff_available
                || !evidence.write_safety_review_available
                || !evidence.safe_env_evidence_available);
        if has_only_advisory_evidence {
            blockers.push("local package/runtime version evidence is advisory only".to_string());
            status = HyprlandVersionActivationStatus::PartialEvidenceOnly;
        }
        if !evidence.official_export_available {
            blockers.push("trusted official export bundle is required".to_string());
            status = HyprlandVersionActivationStatus::MissingOfficialExport;
        }
        if !evidence.row_count_diff_available {
            blockers.push("row-count diff against v0.55.2 is required".to_string());
            status = HyprlandVersionActivationStatus::MissingRowCountDiff;
        }
        if !evidence.write_safety_review_available {
            blockers.push("write-safety review is required".to_string());
            status = HyprlandVersionActivationStatus::MissingWriteSafetyReview;
        }
        if !evidence.safe_env_evidence_available {
            blockers.push("safe-env evidence matrix is required".to_string());
            status = HyprlandVersionActivationStatus::MissingSafeEnvEvidence;
        }
        if !evidence.user_approval_recorded {
            blockers.push("explicit user approval is required".to_string());
            status = HyprlandVersionActivationStatus::MissingUserApproval;
        }
        if blockers.is_empty() && activation_flag_enabled {
            status = HyprlandVersionActivationStatus::Enabled;
        } else if blockers.is_empty() {
            status = HyprlandVersionActivationStatus::ReadyButDefaultDisabled;
            blockers.push("migration activation flag is default-disabled".to_string());
        }
    }

    let migration_activated = evidence.requested_version == current_default.version
        || status == HyprlandVersionActivationStatus::Enabled && activation_flag_enabled;
    HyprlandVersionActivationGate {
        requested_version: evidence.requested_version.clone(),
        current_default_version: current_default.version,
        status,
        local_package_evidence: evidence.installed_package_version.clone(),
        local_runtime_evidence: evidence.runtime_binary_version.clone(),
        official_export_available: evidence.official_export_available,
        row_count_diff_available: evidence.row_count_diff_available,
        write_safety_review_available: evidence.write_safety_review_available,
        safe_env_evidence_available: evidence.safe_env_evidence_available,
        user_approval_recorded: evidence.user_approval_recorded,
        activation_flag_enabled,
        migration_activated,
        production_default_changed: false,
        blockers: blockers.clone(),
        review_lines: vec![
            "Hyprland v0.55.2 remains the active/default app data bundle.".to_string(),
            "Package/runtime version evidence is advisory and cannot replace trusted official exports.".to_string(),
            "0.55.4 activation requires official exports, row diff, write-safety review, safe-env evidence, and user approval.".to_string(),
            format!(
                "Blockers: {}",
                if blockers.is_empty() {
                    "none".to_string()
                } else {
                    blockers.join("; ")
                }
            ),
        ],
    }
}

pub fn duplicate_approval_flow(
    review: &DuplicateProductionGateReview,
    request: Option<&ApprovalRequest>,
) -> ApprovalDecision {
    approval_decision_for_gate(
        ApprovalScope::DuplicateReplacement,
        review.status == DuplicateProductionGateStatus::ReadyButDefaultDisabled,
        review.selected_path.as_deref(),
        None,
        request,
        false,
    )
}

pub fn structured_approval_flow(
    review: &StructuredProductionGateReview,
    request: Option<&ApprovalRequest>,
) -> ApprovalDecision {
    approval_decision_for_gate(
        ApprovalScope::StructuredHlBindWrite,
        review.status == StructuredProductionGateStatus::ReadyButDefaultDisabled,
        Some(&review.target_path),
        None,
        request,
        false,
    )
}

pub fn profile_approval_flow(
    review: &ProfileProductionGateReview,
    request: Option<&ApprovalRequest>,
) -> ApprovalDecision {
    approval_decision_for_gate(
        ApprovalScope::ProfileModeSwitch,
        review.status == ProfileProductionGateStatus::ReadyButDefaultDisabled,
        Some(&review.symlink_path),
        None,
        request,
        false,
    )
}

pub fn runtime_approval_flow(
    review: &RuntimeProductionGateReview,
    request: Option<&ApprovalRequest>,
) -> ApprovalDecision {
    let expected_command = match &review.action {
        RuntimeAction::Keyword { key, value } => Some(format!("hyprctl keyword {key} {value}")),
        RuntimeAction::Reload => Some("hyprctl reload".to_string()),
        RuntimeAction::Dispatch { command } => Some(format!("hyprctl dispatch {command}")),
        RuntimeAction::Status { query } => Some(format!("hyprctl {query}")),
    };
    let expected_scope = match review.action {
        RuntimeAction::Reload => ApprovalScope::RuntimeReload,
        _ => ApprovalScope::RuntimeKeyword,
    };
    approval_decision_for_gate(
        expected_scope,
        review.status == RuntimeProductionGateStatus::ReadyButDefaultDisabled,
        None,
        expected_command.as_deref(),
        request,
        false,
    )
}

pub fn high_risk_approval_flow(
    review: &HighRiskProductionGateReview,
    request: Option<&ApprovalRequest>,
) -> ApprovalDecision {
    approval_decision_for_gate(
        ApprovalScope::HighRiskDisplayWrite,
        review.status == HighRiskProductionGateStatus::ReadyButDefaultDisabled,
        None,
        Some(review.setting_id.as_str()),
        request,
        false,
    )
}

pub fn hyprland_0554_approval_flow(
    review: &HyprlandVersionActivationGate,
    request: Option<&ApprovalRequest>,
) -> ApprovalDecision {
    approval_decision_for_gate(
        ApprovalScope::Hyprland0554Migration,
        review.status == HyprlandVersionActivationStatus::ReadyButDefaultDisabled,
        None,
        Some("hyprland_0554_migration"),
        request,
        false,
    )
}

pub fn local_hyprland_version_evidence(
    requested_version: &str,
    installed_package_version: Option<&str>,
    runtime_binary_version: Option<&str>,
    official_export_available: bool,
    row_count_diff_available: bool,
    write_safety_review_available: bool,
    safe_env_evidence_available: bool,
    user_approval_recorded: bool,
) -> LocalHyprlandVersionEvidence {
    let requirement = trusted_export_requirement(
        requested_version,
        official_export_available,
        row_count_diff_available,
        write_safety_review_available,
        safe_env_evidence_available,
    );
    let mut missing_inputs = requirement.missing_inputs;
    if requested_version != "0.55.2" && !user_approval_recorded {
        missing_inputs.push("explicit user approval".to_string());
    }
    if requested_version != "0.55.2" && installed_package_version.is_none() {
        missing_inputs.push("local installed package version evidence".to_string());
    }
    if requested_version != "0.55.2" && runtime_binary_version.is_none() {
        missing_inputs.push("read-only runtime binary version evidence".to_string());
    }

    let activation_allowed = requested_version == "0.55.2" || missing_inputs.is_empty();
    let mut evidence_lines = vec![
        "Hyprland v0.55.2 remains the active/default app data bundle.".to_string(),
        "Local package/runtime version evidence is advisory and cannot replace trusted exports."
            .to_string(),
    ];
    if let Some(version) = installed_package_version {
        evidence_lines.push(format!("Installed package version observed: {version}."));
    }
    if let Some(version) = runtime_binary_version {
        evidence_lines.push(format!("Runtime binary version observed: {version}."));
    }
    if !activation_allowed {
        evidence_lines.push(
            "Requested migration stays inactive until every trusted input exists.".to_string(),
        );
    }

    LocalHyprlandVersionEvidence {
        requested_version: requested_version.to_string(),
        installed_package_version: installed_package_version.map(ToOwned::to_owned),
        runtime_binary_version: runtime_binary_version.map(ToOwned::to_owned),
        official_export_available,
        row_count_diff_available,
        write_safety_review_available,
        safe_env_evidence_available,
        user_approval_recorded,
        activation_allowed,
        production_default_changed: false,
        evidence_lines,
        missing_inputs,
    }
}

pub fn disabled_migration_review(requested_version: &str) -> DisabledMigrationReview {
    DisabledMigrationReview {
        current_default: current_v0552_data_bundle(),
        requested_version: requested_version.to_string(),
        migration_enabled: false,
        review_lines: vec![
            "The app still defaults to Hyprland v0.55.2 data/model.".to_string(),
            "A newer runtime package is not enough to migrate app data.".to_string(),
            "Trusted official exports and comparison tests are required before activation."
                .to_string(),
        ],
    }
}

pub fn trusted_export_requirement(
    requested_version: &str,
    has_official_export: bool,
    has_row_count_diff: bool,
    has_write_safety_review: bool,
    has_safe_env_evidence: bool,
) -> TrustedExportRequirement {
    let mut missing_inputs = Vec::new();
    if requested_version != "0.55.2" && !has_official_export {
        missing_inputs.push("trusted official export".to_string());
    }
    if requested_version != "0.55.2" && !has_row_count_diff {
        missing_inputs.push("row-count diff against v0.55.2".to_string());
    }
    if requested_version != "0.55.2" && !has_write_safety_review {
        missing_inputs.push("write-safety review".to_string());
    }
    if requested_version != "0.55.2" && !has_safe_env_evidence {
        missing_inputs.push("safe-env GTK evidence".to_string());
    }
    TrustedExportRequirement {
        requested_version: requested_version.to_string(),
        has_official_export,
        has_row_count_diff,
        has_write_safety_review,
        has_safe_env_evidence,
        can_activate: requested_version == "0.55.2" || missing_inputs.is_empty(),
        missing_inputs,
    }
}

pub fn migration_comparison_review(
    requested_version: &str,
    trusted_export_available: bool,
) -> MigrationComparisonReview {
    let current_default = current_v0552_data_bundle();
    let requested_bundle = if requested_version == current_default.version {
        Some(current_default.clone())
    } else {
        None
    };
    let mut missing_proof = Vec::new();
    if requested_version != current_default.version {
        missing_proof.push("trusted official export for requested version".to_string());
        missing_proof.push("row-count diff against v0.55.2".to_string());
        missing_proof.push("write-safety classification review".to_string());
        missing_proof.push("GTK safe-env evidence matrix for requested bundle".to_string());
    }
    if !trusted_export_available && requested_version != current_default.version {
        missing_proof.push("trusted source confirmation".to_string());
    }

    MigrationComparisonReview {
        current_default,
        requested_version: requested_version.to_string(),
        requested_bundle,
        trusted_source_requirement_met: requested_version == "0.55.2" || trusted_export_available,
        missing_proof,
        migration_enabled: false,
        production_default_changed: false,
        review_lines: vec![
            "Hyprland v0.55.2 remains the active app data bundle.".to_string(),
            "Newer runtime/package versions are assessed side by side only.".to_string(),
            "Migration cannot activate without trusted exports and comparison proof.".to_string(),
        ],
    }
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
