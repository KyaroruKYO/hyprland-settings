//! Inert executor wiring-readiness models for structured families.
//!
//! This module is planning/readiness-only. It defines the boundaries that any
//! future executor wiring must respect, and it reports that no wiring exists.
//! Nothing in this module calls the executor scaffold, the scalar write flow,
//! the UI, the filesystem, or any command runner. Every readiness function
//! reports executor wiring approved false and executor wired false.

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum StructuredFamilyExecutorWiringStatus {
    PlanningOnly,
    ReadinessRecorded,
    Unwired,
    NotApproved,
    RejectedByDefault,
}

impl StructuredFamilyExecutorWiringStatus {
    pub fn as_str(self) -> &'static str {
        match self {
            Self::PlanningOnly => "StructuredFamilyExecutorWiringPlanningOnly",
            Self::ReadinessRecorded => "StructuredFamilyExecutorWiringReadinessRecorded",
            Self::Unwired => "StructuredFamilyExecutorWiringUnwired",
            Self::NotApproved => "StructuredFamilyExecutorWiringNotApproved",
            Self::RejectedByDefault => "StructuredFamilyExecutorWiringRejectedByDefault",
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum StructuredFamilyExecutorWiringRejectionReason {
    ExecutorWiringPlanningOnly,
    ExecutorWiringNotApproved,
    ExecutorWiredFalse,
    ExecutorReachabilityNotApproved,
    WriteFlowBoundaryNotApproved,
    ApplySettingChangeBoundaryNotApproved,
    UiReachabilityNotApproved,
    RealWriteScopeNotApproved,
    RealConfigTargetNotApproved,
    BackupExecutionNotApproved,
    RestoreExecutionNotApproved,
    RollbackExecutionNotApproved,
    HyprlandReloadNotApproved,
    RuntimeMutationNotApproved,
    FirstRealConfigWriteNotApproved,
    GuiRealWriteControlsNotApproved,
    ProductionReadinessNotApproved,
}

impl StructuredFamilyExecutorWiringRejectionReason {
    pub fn as_str(self) -> &'static str {
        match self {
            Self::ExecutorWiringPlanningOnly => "ExecutorWiringPlanningOnly",
            Self::ExecutorWiringNotApproved => "ExecutorWiringNotApproved",
            Self::ExecutorWiredFalse => "ExecutorWiredFalse",
            Self::ExecutorReachabilityNotApproved => "ExecutorReachabilityNotApproved",
            Self::WriteFlowBoundaryNotApproved => "WriteFlowBoundaryNotApproved",
            Self::ApplySettingChangeBoundaryNotApproved => "ApplySettingChangeBoundaryNotApproved",
            Self::UiReachabilityNotApproved => "UiReachabilityNotApproved",
            Self::RealWriteScopeNotApproved => "RealWriteScopeNotApproved",
            Self::RealConfigTargetNotApproved => "RealConfigTargetNotApproved",
            Self::BackupExecutionNotApproved => "BackupExecutionNotApproved",
            Self::RestoreExecutionNotApproved => "RestoreExecutionNotApproved",
            Self::RollbackExecutionNotApproved => "RollbackExecutionNotApproved",
            Self::HyprlandReloadNotApproved => "HyprlandReloadNotApproved",
            Self::RuntimeMutationNotApproved => "RuntimeMutationNotApproved",
            Self::FirstRealConfigWriteNotApproved => "FirstRealConfigWriteNotApproved",
            Self::GuiRealWriteControlsNotApproved => "GuiRealWriteControlsNotApproved",
            Self::ProductionReadinessNotApproved => "ProductionReadinessNotApproved",
        }
    }
}

/// A boundary that any future executor wiring must not cross without a
/// separate explicit approval. Boundaries are universal; they are never
/// family-specific and never rank families.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct StructuredFamilyExecutorWiringBoundary {
    pub boundary_id: &'static str,
    pub description: &'static str,
    pub wiring_allowed: bool,
    pub crossing_requires_explicit_approval: bool,
    pub rejection_reason: StructuredFamilyExecutorWiringRejectionReason,
}

/// A future wiring call-site candidate described for planning only. Candidates
/// are boundary-level integration points, not families, records, or subsets.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct StructuredFamilyExecutorWiringCandidate {
    pub candidate_id: &'static str,
    pub boundary_id: &'static str,
    pub description: &'static str,
    pub wired: bool,
    pub approved: bool,
    pub family_specific: bool,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct StructuredFamilyExecutorWiringApprovalState {
    pub status: StructuredFamilyExecutorWiringStatus,
    pub executor_wiring_planning_approved: bool,
    pub executor_wiring_approved: bool,
    pub executor_wired: bool,
    pub real_write_scope_approved: bool,
    pub gui_real_write_controls_approved: bool,
    pub production_activation_approved: bool,
    pub first_real_config_write_approved: bool,
    pub rejection_reasons: Vec<StructuredFamilyExecutorWiringRejectionReason>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct StructuredFamilyExecutorWiringPreflight {
    pub status: StructuredFamilyExecutorWiringStatus,
    pub passed: bool,
    pub wiring_may_proceed: bool,
    pub rejection_reasons: Vec<StructuredFamilyExecutorWiringRejectionReason>,
}

/// A source-level guard that future regression tests must keep enforcing on
/// this module. Patterns are described in prose so this planning module never
/// embeds a callable form of a forbidden path.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct StructuredFamilyExecutorWiringSourceGuard {
    pub guard_id: &'static str,
    pub forbidden_behavior: &'static str,
    pub enforced_by_test: bool,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct StructuredFamilyExecutorWiringReadiness {
    pub readiness_id: &'static str,
    pub status: StructuredFamilyExecutorWiringStatus,
    pub planning_only: bool,
    pub boundaries: Vec<StructuredFamilyExecutorWiringBoundary>,
    pub candidates: Vec<StructuredFamilyExecutorWiringCandidate>,
    pub source_guards: Vec<StructuredFamilyExecutorWiringSourceGuard>,
    pub default_rejection_reasons: Vec<StructuredFamilyExecutorWiringRejectionReason>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct StructuredFamilyExecutorWiringReadinessReport {
    pub status: StructuredFamilyExecutorWiringStatus,
    pub executor_wiring_planning_approved: bool,
    pub executor_wiring_readiness_model_added: bool,
    pub executor_wiring_boundary_defined: bool,
    pub executor_wiring_source_guards_added: bool,
    pub executor_wiring_approved: bool,
    pub executor_wired: bool,
    pub executor_reachable_from_ui: bool,
    pub executor_reachable_from_write_flow: bool,
    pub executor_reachable_from_apply_setting_change: bool,
    pub real_write_path_enabled: bool,
    pub real_config_target_enabled: bool,
    pub backup_creation_enabled: bool,
    pub restore_execution_enabled: bool,
    pub rollback_execution_enabled: bool,
    pub hyprland_reload_enabled: bool,
    pub runtime_mutation_enabled: bool,
    pub first_real_config_write_approved: bool,
    pub gui_real_write_controls_enabled: bool,
    pub family_ranking_excluded: bool,
    pub activation_subset_selected: bool,
    pub production_ready: bool,
    pub rejection_reasons: Vec<StructuredFamilyExecutorWiringRejectionReason>,
}

pub fn executor_wiring_rejection_reasons() -> Vec<StructuredFamilyExecutorWiringRejectionReason> {
    vec![
        StructuredFamilyExecutorWiringRejectionReason::ExecutorWiringPlanningOnly,
        StructuredFamilyExecutorWiringRejectionReason::ExecutorWiringNotApproved,
        StructuredFamilyExecutorWiringRejectionReason::ExecutorWiredFalse,
        StructuredFamilyExecutorWiringRejectionReason::ExecutorReachabilityNotApproved,
        StructuredFamilyExecutorWiringRejectionReason::WriteFlowBoundaryNotApproved,
        StructuredFamilyExecutorWiringRejectionReason::ApplySettingChangeBoundaryNotApproved,
        StructuredFamilyExecutorWiringRejectionReason::UiReachabilityNotApproved,
        StructuredFamilyExecutorWiringRejectionReason::RealWriteScopeNotApproved,
        StructuredFamilyExecutorWiringRejectionReason::RealConfigTargetNotApproved,
        StructuredFamilyExecutorWiringRejectionReason::BackupExecutionNotApproved,
        StructuredFamilyExecutorWiringRejectionReason::RestoreExecutionNotApproved,
        StructuredFamilyExecutorWiringRejectionReason::RollbackExecutionNotApproved,
        StructuredFamilyExecutorWiringRejectionReason::HyprlandReloadNotApproved,
        StructuredFamilyExecutorWiringRejectionReason::RuntimeMutationNotApproved,
        StructuredFamilyExecutorWiringRejectionReason::FirstRealConfigWriteNotApproved,
        StructuredFamilyExecutorWiringRejectionReason::GuiRealWriteControlsNotApproved,
        StructuredFamilyExecutorWiringRejectionReason::ProductionReadinessNotApproved,
    ]
}

pub fn build_executor_wiring_boundary() -> Vec<StructuredFamilyExecutorWiringBoundary> {
    vec![
        StructuredFamilyExecutorWiringBoundary {
            boundary_id: "executor-scaffold-boundary",
            description: "The inert safe-write executor scaffold must stay unwired; no code path may make its execute entry point reachable.",
            wiring_allowed: false,
            crossing_requires_explicit_approval: true,
            rejection_reason:
                StructuredFamilyExecutorWiringRejectionReason::ExecutorWiringNotApproved,
        },
        StructuredFamilyExecutorWiringBoundary {
            boundary_id: "write-flow-boundary",
            description: "The scalar write flow module must not import or call structured-family executor code.",
            wiring_allowed: false,
            crossing_requires_explicit_approval: true,
            rejection_reason:
                StructuredFamilyExecutorWiringRejectionReason::WriteFlowBoundaryNotApproved,
        },
        StructuredFamilyExecutorWiringBoundary {
            boundary_id: "apply-setting-change-boundary",
            description: "The scalar apply entry point must remain scalar-only and must not dispatch structured-family executor work.",
            wiring_allowed: false,
            crossing_requires_explicit_approval: true,
            rejection_reason:
                StructuredFamilyExecutorWiringRejectionReason::ApplySettingChangeBoundaryNotApproved,
        },
        StructuredFamilyExecutorWiringBoundary {
            boundary_id: "ui-reachability-boundary",
            description: "No UI model, window, or control may reach executor or wiring-readiness code; no user-facing real-write control may be designed or wired.",
            wiring_allowed: false,
            crossing_requires_explicit_approval: true,
            rejection_reason:
                StructuredFamilyExecutorWiringRejectionReason::UiReachabilityNotApproved,
        },
        StructuredFamilyExecutorWiringBoundary {
            boundary_id: "filesystem-boundary",
            description: "No real config file, include file, or user config directory may be read for mutation or written; no filesystem write API may be called.",
            wiring_allowed: false,
            crossing_requires_explicit_approval: true,
            rejection_reason:
                StructuredFamilyExecutorWiringRejectionReason::RealConfigTargetNotApproved,
        },
        StructuredFamilyExecutorWiringBoundary {
            boundary_id: "backup-restore-boundary",
            description: "No real backup may be created and no restore may be executed.",
            wiring_allowed: false,
            crossing_requires_explicit_approval: true,
            rejection_reason:
                StructuredFamilyExecutorWiringRejectionReason::BackupExecutionNotApproved,
        },
        StructuredFamilyExecutorWiringBoundary {
            boundary_id: "rollback-recovery-boundary",
            description: "No rollback file may be created and no rollback or recovery may be executed.",
            wiring_allowed: false,
            crossing_requires_explicit_approval: true,
            rejection_reason:
                StructuredFamilyExecutorWiringRejectionReason::RollbackExecutionNotApproved,
        },
        StructuredFamilyExecutorWiringBoundary {
            boundary_id: "reload-runtime-boundary",
            description: "No Hyprland reload may run and no compositor runtime state may be mutated; no command runner may be invoked.",
            wiring_allowed: false,
            crossing_requires_explicit_approval: true,
            rejection_reason:
                StructuredFamilyExecutorWiringRejectionReason::HyprlandReloadNotApproved,
        },
    ]
}

pub fn build_executor_wiring_candidate() -> Vec<StructuredFamilyExecutorWiringCandidate> {
    vec![
        StructuredFamilyExecutorWiringCandidate {
            candidate_id: "future-structured-family-write-coordinator",
            boundary_id: "executor-scaffold-boundary",
            description: "A future dedicated coordinator module, separate from the scalar write flow, would be the only allowed caller of the executor scaffold.",
            wired: false,
            approved: false,
            family_specific: false,
        },
        StructuredFamilyExecutorWiringCandidate {
            candidate_id: "future-approval-gate-adapter",
            boundary_id: "executor-scaffold-boundary",
            description: "A future adapter would translate explicit user approval state into executor preflight input; it must fail closed on any missing approval.",
            wired: false,
            approved: false,
            family_specific: false,
        },
        StructuredFamilyExecutorWiringCandidate {
            candidate_id: "future-write-flow-isolation-check",
            boundary_id: "write-flow-boundary",
            description: "Any future wiring must add regression proof that the scalar write flow remains isolated from structured-family execution.",
            wired: false,
            approved: false,
            family_specific: false,
        },
        StructuredFamilyExecutorWiringCandidate {
            candidate_id: "future-ui-review-projection",
            boundary_id: "ui-reachability-boundary",
            description: "Any future UI surface stays review-only until GUI real-write controls are separately approved; no control may trigger execution.",
            wired: false,
            approved: false,
            family_specific: false,
        },
    ]
}

pub fn executor_wiring_source_guards() -> Vec<StructuredFamilyExecutorWiringSourceGuard> {
    vec![
        StructuredFamilyExecutorWiringSourceGuard {
            guard_id: "no-executor-call",
            forbidden_behavior: "calling the safe-write executor scaffold execute entry point",
            enforced_by_test: true,
        },
        StructuredFamilyExecutorWiringSourceGuard {
            guard_id: "no-write-flow-call",
            forbidden_behavior: "importing or calling the scalar write flow module",
            enforced_by_test: true,
        },
        StructuredFamilyExecutorWiringSourceGuard {
            guard_id: "no-apply-setting-change-call",
            forbidden_behavior: "calling the scalar apply entry point",
            enforced_by_test: true,
        },
        StructuredFamilyExecutorWiringSourceGuard {
            guard_id: "no-filesystem-write",
            forbidden_behavior:
                "calling filesystem write APIs (fs write, file create, write all, serde to_writer)",
            enforced_by_test: true,
        },
        StructuredFamilyExecutorWiringSourceGuard {
            guard_id: "no-real-config-reference",
            forbidden_behavior:
                "referencing the user's real Hyprland config path or config directory",
            enforced_by_test: true,
        },
        StructuredFamilyExecutorWiringSourceGuard {
            guard_id: "no-command-runner",
            forbidden_behavior:
                "spawning processes or running compositor control commands, including reload",
            enforced_by_test: true,
        },
        StructuredFamilyExecutorWiringSourceGuard {
            guard_id: "no-gtk-controls",
            forbidden_behavior: "creating GTK widgets, buttons, or click handlers for real writes",
            enforced_by_test: true,
        },
        StructuredFamilyExecutorWiringSourceGuard {
            guard_id: "no-approval-flag-flip",
            forbidden_behavior:
                "setting any wiring, activation, or real-write approval flag to true",
            enforced_by_test: true,
        },
    ]
}

pub fn build_executor_wiring_readiness() -> StructuredFamilyExecutorWiringReadiness {
    StructuredFamilyExecutorWiringReadiness {
        readiness_id: "structured-family-executor-wiring-readiness",
        status: StructuredFamilyExecutorWiringStatus::PlanningOnly,
        planning_only: true,
        boundaries: build_executor_wiring_boundary(),
        candidates: build_executor_wiring_candidate(),
        source_guards: executor_wiring_source_guards(),
        default_rejection_reasons: executor_wiring_rejection_reasons(),
    }
}

pub fn verify_executor_wiring_approval_state(
    _readiness: &StructuredFamilyExecutorWiringReadiness,
) -> StructuredFamilyExecutorWiringApprovalState {
    StructuredFamilyExecutorWiringApprovalState {
        status: StructuredFamilyExecutorWiringStatus::NotApproved,
        executor_wiring_planning_approved: true,
        executor_wiring_approved: false,
        executor_wired: false,
        real_write_scope_approved: false,
        gui_real_write_controls_approved: false,
        production_activation_approved: false,
        first_real_config_write_approved: false,
        rejection_reasons: executor_wiring_rejection_reasons(),
    }
}

pub fn validate_executor_wiring_preflight(
    readiness: &StructuredFamilyExecutorWiringReadiness,
) -> StructuredFamilyExecutorWiringPreflight {
    let mut rejection_reasons = executor_wiring_rejection_reasons();
    if !readiness.planning_only {
        rejection_reasons
            .push(StructuredFamilyExecutorWiringRejectionReason::ExecutorWiringPlanningOnly);
    }
    StructuredFamilyExecutorWiringPreflight {
        status: StructuredFamilyExecutorWiringStatus::RejectedByDefault,
        passed: false,
        wiring_may_proceed: false,
        rejection_reasons,
    }
}

pub fn executor_wiring_readiness_report(
    readiness: &StructuredFamilyExecutorWiringReadiness,
) -> StructuredFamilyExecutorWiringReadinessReport {
    StructuredFamilyExecutorWiringReadinessReport {
        status: StructuredFamilyExecutorWiringStatus::ReadinessRecorded,
        executor_wiring_planning_approved: true,
        executor_wiring_readiness_model_added: true,
        executor_wiring_boundary_defined: !readiness.boundaries.is_empty(),
        executor_wiring_source_guards_added: !readiness.source_guards.is_empty(),
        executor_wiring_approved: false,
        executor_wired: false,
        executor_reachable_from_ui: false,
        executor_reachable_from_write_flow: false,
        executor_reachable_from_apply_setting_change: false,
        real_write_path_enabled: false,
        real_config_target_enabled: false,
        backup_creation_enabled: false,
        restore_execution_enabled: false,
        rollback_execution_enabled: false,
        hyprland_reload_enabled: false,
        runtime_mutation_enabled: false,
        first_real_config_write_approved: false,
        gui_real_write_controls_enabled: false,
        family_ranking_excluded: true,
        activation_subset_selected: false,
        production_ready: false,
        rejection_reasons: executor_wiring_rejection_reasons(),
    }
}
