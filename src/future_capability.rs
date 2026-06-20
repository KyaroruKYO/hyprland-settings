use std::collections::hash_map::DefaultHasher;
use std::fs;
use std::hash::{Hash, Hasher};
use std::path::{Path, PathBuf};

use anyhow::{anyhow, Result};

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
