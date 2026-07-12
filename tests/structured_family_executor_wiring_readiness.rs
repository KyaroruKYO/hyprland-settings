use std::fs;

use hyprland_settings::structured_family_executor_wiring::{
    build_executor_wiring_boundary, build_executor_wiring_candidate,
    build_executor_wiring_readiness, executor_wiring_readiness_report,
    executor_wiring_rejection_reasons, executor_wiring_source_guards,
    validate_executor_wiring_preflight, verify_executor_wiring_approval_state,
    StructuredFamilyExecutorWiringRejectionReason, StructuredFamilyExecutorWiringStatus,
};

#[test]
fn wiring_readiness_is_planning_only_and_rejects_by_default() {
    let readiness = build_executor_wiring_readiness();
    assert_eq!(
        readiness.status,
        StructuredFamilyExecutorWiringStatus::PlanningOnly
    );
    assert!(readiness.planning_only);
    assert!(!readiness.boundaries.is_empty());
    assert!(!readiness.candidates.is_empty());
    assert!(!readiness.source_guards.is_empty());

    let preflight = validate_executor_wiring_preflight(&readiness);
    assert_eq!(
        preflight.status,
        StructuredFamilyExecutorWiringStatus::RejectedByDefault
    );
    assert!(!preflight.passed);
    assert!(!preflight.wiring_may_proceed);
    assert!(preflight
        .rejection_reasons
        .contains(&StructuredFamilyExecutorWiringRejectionReason::ExecutorWiringNotApproved));

    let approvals = verify_executor_wiring_approval_state(&readiness);
    assert!(approvals.executor_wiring_planning_approved);
    assert!(!approvals.executor_wiring_approved);
    assert!(!approvals.executor_wired);
    assert!(!approvals.real_write_scope_approved);
    assert!(!approvals.gui_real_write_controls_approved);
    assert!(!approvals.production_activation_approved);
    assert!(!approvals.first_real_config_write_approved);
}

#[test]
fn wiring_readiness_report_keeps_every_activation_flag_false() {
    let readiness = build_executor_wiring_readiness();
    let report = executor_wiring_readiness_report(&readiness);

    assert!(report.executor_wiring_planning_approved);
    assert!(report.executor_wiring_readiness_model_added);
    assert!(report.executor_wiring_boundary_defined);
    assert!(report.executor_wiring_source_guards_added);

    assert!(!report.executor_wiring_approved);
    assert!(!report.executor_wired);
    assert!(!report.executor_reachable_from_ui);
    assert!(!report.executor_reachable_from_write_flow);
    assert!(!report.executor_reachable_from_apply_setting_change);
    assert!(!report.real_write_path_enabled);
    assert!(!report.real_config_target_enabled);
    assert!(!report.backup_creation_enabled);
    assert!(!report.restore_execution_enabled);
    assert!(!report.rollback_execution_enabled);
    assert!(!report.hyprland_reload_enabled);
    assert!(!report.runtime_mutation_enabled);
    assert!(!report.first_real_config_write_approved);
    assert!(!report.gui_real_write_controls_enabled);
    assert!(!report.activation_subset_selected);
    assert!(!report.production_ready);
    assert!(report.family_ranking_excluded);
    assert!(!report.rejection_reasons.is_empty());
}

#[test]
fn wiring_readiness_exposes_all_default_rejection_reasons() {
    let reasons = executor_wiring_rejection_reasons();
    let expected = [
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
    ];
    for reason in expected {
        assert!(
            reasons.contains(&reason),
            "missing wiring rejection reason {}",
            reason.as_str()
        );
    }
}

#[test]
fn wiring_boundaries_cover_every_gate_and_forbid_wiring() {
    let boundaries = build_executor_wiring_boundary();
    let expected_ids = [
        "executor-scaffold-boundary",
        "write-flow-boundary",
        "apply-setting-change-boundary",
        "ui-reachability-boundary",
        "filesystem-boundary",
        "backup-restore-boundary",
        "rollback-recovery-boundary",
        "reload-runtime-boundary",
    ];
    for id in expected_ids {
        let boundary = boundaries
            .iter()
            .find(|boundary| boundary.boundary_id == id)
            .unwrap_or_else(|| panic!("missing wiring boundary {id}"));
        assert!(
            !boundary.wiring_allowed,
            "boundary {id} must forbid wiring by default"
        );
        assert!(
            boundary.crossing_requires_explicit_approval,
            "boundary {id} must require explicit approval"
        );
    }
}

#[test]
fn wiring_candidates_remain_unwired_unapproved_and_not_family_specific() {
    let boundaries = build_executor_wiring_boundary();
    let candidates = build_executor_wiring_candidate();
    assert!(!candidates.is_empty());
    for candidate in &candidates {
        assert!(
            !candidate.wired,
            "candidate {} must remain unwired",
            candidate.candidate_id
        );
        assert!(
            !candidate.approved,
            "candidate {} must remain unapproved",
            candidate.candidate_id
        );
        assert!(
            !candidate.family_specific,
            "candidate {} must not be family-specific",
            candidate.candidate_id
        );
        assert!(
            !candidate.candidate_id.contains("hl."),
            "candidate {} must not name a family",
            candidate.candidate_id
        );
        assert!(
            !candidate.description.contains("hl."),
            "candidate {} description must not name a family",
            candidate.candidate_id
        );
        assert!(
            boundaries
                .iter()
                .any(|boundary| boundary.boundary_id == candidate.boundary_id),
            "candidate {} must reference a defined boundary",
            candidate.candidate_id
        );
    }
}

#[test]
fn wiring_source_guards_are_all_test_enforced() {
    let guards = executor_wiring_source_guards();
    let expected_ids = [
        "no-executor-call",
        "no-write-flow-call",
        "no-apply-setting-change-call",
        "no-filesystem-write",
        "no-real-config-reference",
        "no-command-runner",
        "no-gtk-controls",
        "no-approval-flag-flip",
    ];
    for id in expected_ids {
        let guard = guards
            .iter()
            .find(|guard| guard.guard_id == id)
            .unwrap_or_else(|| panic!("missing wiring source guard {id}"));
        assert!(guard.enforced_by_test, "guard {id} must be test-enforced");
    }
}

#[test]
fn wiring_readiness_source_has_no_mutating_or_wiring_paths() {
    let source = fs::read_to_string("src/structured_family_executor_wiring.rs")
        .expect("wiring-readiness source should read");

    for forbidden in [
        "execute_safe_write_scaffold",
        "structured_family_safe_write",
        "write_flow::",
        "crate::write_flow",
        "apply_setting_change(",
        "hyprctl",
        ".config/hypr",
        "std::fs",
        "std::process",
        "fs::write",
        "File::create",
        "write_all",
        "serde_json::to_writer",
        "Command::",
        "gtk::",
        "gtk4::",
        "adw::",
        "Button::",
        "connect_clicked",
        "executor_wiring_approved: true",
        "executor_wired: true",
        "real_write_path_enabled: true",
        "real_config_target_enabled: true",
        "backup_creation_enabled: true",
        "restore_execution_enabled: true",
        "rollback_execution_enabled: true",
        "hyprland_reload_enabled: true",
        "runtime_mutation_enabled: true",
        "first_real_config_write_approved: true",
        "gui_real_write_controls_enabled: true",
        "production_activation_approved: true",
        "activation_subset_selected: true",
        "production_ready: true",
        "wired: true",
        "wiring_allowed: true",
        "wiring_may_proceed: true",
    ] {
        assert!(
            !source.contains(forbidden),
            "wiring-readiness source must not contain {forbidden}"
        );
    }
}

#[test]
fn wiring_readiness_module_is_unreachable_from_runtime_write_and_ui_paths() {
    for path in [
        "src/main.rs",
        "src/write_flow.rs",
        "src/structured_family.rs",
        "src/structured_family_safe_write.rs",
        "src/ui/mod.rs",
        "src/ui/app.rs",
        "src/ui/model.rs",
        "src/ui/window.rs",
    ] {
        let source =
            fs::read_to_string(path).unwrap_or_else(|_| panic!("{path} source should read"));
        assert!(
            !source.contains("structured_family_executor_wiring"),
            "{path} must not reference the wiring-readiness module"
        );
        assert!(
            !source.contains("executor_wiring_readiness_report"),
            "{path} must not call wiring-readiness functions"
        );
    }
}

#[test]
fn executor_scaffold_remains_unwired_after_wiring_readiness_work() {
    let scaffold_callers = [
        "src/main.rs",
        "src/write_flow.rs",
        "src/ui/mod.rs",
        "src/ui/app.rs",
        "src/ui/model.rs",
        "src/ui/window.rs",
    ];
    for path in scaffold_callers {
        let source =
            fs::read_to_string(path).unwrap_or_else(|_| panic!("{path} source should read"));
        assert!(
            !source.contains("execute_safe_write_scaffold"),
            "{path} must not call the executor scaffold"
        );
        assert!(
            !source.contains("structured_family_safe_write"),
            "{path} must not import the executor scaffold"
        );
    }
}

#[test]
fn executor_wiring_readiness_plan_report_preserves_non_approval_state() {
    let report: serde_json::Value = serde_json::from_str(
        &fs::read_to_string(
            "data/reports/structured-family-executor-wiring-readiness-plan.v0.55.2.json",
        )
        .expect("wiring-readiness plan report should read"),
    )
    .expect("wiring-readiness plan report should be valid JSON");

    assert_eq!(
        report["artifactKind"],
        "structured-family-executor-wiring-readiness-plan"
    );
    assert_eq!(report["executorWiringPlanningApproved"], true);
    assert_eq!(report["executorWiringReadinessModelAdded"], true);
    assert_eq!(report["executorWiringBoundaryDefined"], true);
    assert_eq!(report["executorWiringSourceGuardsAdded"], true);
    assert_eq!(report["actualExecutorImplementationApproved"], true);
    assert_eq!(report["executorImplementationApproved"], true);
    assert_eq!(report["executorScaffoldImplemented"], true);
    assert_eq!(report["executorScaffoldInert"], true);
    assert_eq!(report["executorScaffoldUnwired"], true);
    assert_eq!(report["familyRankingExcluded"], true);

    for key in [
        "executorWiringApproved",
        "executorWired",
        "realWriteScopeApproved",
        "guiRealWriteControlsApproved",
        "productionActivationApproved",
        "realWritePathEnabled",
        "realConfigTargetEnabled",
        "backupCreationEnabled",
        "restoreExecutionEnabled",
        "rollbackExecutionEnabled",
        "hyprctlReloadEnabled",
        "runtimeMutationEnabled",
        "firstRealConfigWriteApproved",
        "guiRealWriteControlsEnabled",
        "activationSubsetSelected",
    ] {
        assert_eq!(report[key], false, "{key} should remain false");
    }
    assert_eq!(
        report["productionReadinessDecision"],
        "not production ready"
    );

    for key in [
        "wiringReadinessModules",
        "wiringReadinessTypes",
        "wiringReadinessFunctions",
        "defaultWiringRejectionReasons",
        "wiringBoundaryPlan",
        "wiringReadinessBehavior",
        "uiReachabilityBoundary",
        "writeFlowBoundary",
        "applySettingChangeBoundary",
        "executorScaffoldBoundary",
        "filesystemBoundary",
        "backupRestoreBoundary",
        "rollbackRecoveryBoundary",
        "reloadRuntimeBoundary",
        "sourceGuardPlan",
        "futureApprovalGates",
        "filesChanged",
        "testsAdded",
    ] {
        assert!(
            !report[key]
                .as_array()
                .expect("wiring-readiness report field should be array")
                .is_empty(),
            "{key} should be populated"
        );
    }

    for reason in [
        "ExecutorWiringPlanningOnly",
        "ExecutorWiringNotApproved",
        "ExecutorWiredFalse",
        "ExecutorReachabilityNotApproved",
        "WriteFlowBoundaryNotApproved",
        "ApplySettingChangeBoundaryNotApproved",
        "UiReachabilityNotApproved",
        "RealWriteScopeNotApproved",
        "RealConfigTargetNotApproved",
        "BackupExecutionNotApproved",
        "RestoreExecutionNotApproved",
        "RollbackExecutionNotApproved",
        "HyprlandReloadNotApproved",
        "RuntimeMutationNotApproved",
        "FirstRealConfigWriteNotApproved",
        "GuiRealWriteControlsNotApproved",
        "ProductionReadinessNotApproved",
    ] {
        assert!(
            report["defaultWiringRejectionReasons"]
                .as_array()
                .expect("defaultWiringRejectionReasons should be array")
                .iter()
                .any(|value| value.as_str() == Some(reason)),
            "missing default wiring rejection reason {reason}"
        );
    }

    let next = report["nextRecommendedWork"]
        .as_str()
        .expect("nextRecommendedWork should be text");
    assert_eq!(
        next,
        "Stop for explicit user decision: approve or reject actual executor wiring scaffold."
    );
    assert!(!next.contains("automatically"));
    assert!(!next.contains("GUI"));
    assert!(!next.contains("real write"));
    assert!(!next.contains("wire the executor"));
}
