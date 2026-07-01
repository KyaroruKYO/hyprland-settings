use std::fs;

use hyprland_settings::structured_family_safe_write::{
    build_safe_write_plan_scaffold, emergency_stop_reason_scaffold,
    emit_safe_write_audit_record_scaffold, execute_safe_write_scaffold,
    prepare_safe_write_backup_plan_scaffold, prepare_safe_write_rollback_plan_scaffold,
    structured_family_safe_write_default_rejection_reasons, validate_safe_write_preflight_scaffold,
    validate_safe_write_target_policy_scaffold, verify_manual_approval_state_scaffold,
    verify_safe_write_result_scaffold, StructuredFamilySafeWriteRejectionReason,
    StructuredFamilySafeWriteScaffoldStatus,
};

#[test]
fn executor_scaffold_rejects_by_default_and_never_executes() {
    let plan = build_safe_write_plan_scaffold();
    assert_eq!(
        plan.scaffold_status,
        StructuredFamilySafeWriteScaffoldStatus::Inert
    );
    assert!(plan.review_only);
    assert!(!plan.executable);

    let preflight = validate_safe_write_preflight_scaffold(&plan);
    assert_eq!(
        preflight.status,
        StructuredFamilySafeWriteScaffoldStatus::RejectedByDefault
    );
    assert!(!preflight.passed);

    let target_policy = validate_safe_write_target_policy_scaffold(&plan);
    assert!(!target_policy.real_config_target_enabled);

    let backup_plan = prepare_safe_write_backup_plan_scaffold(&plan);
    assert!(!backup_plan.backup_creation_enabled);
    assert!(!backup_plan.restore_execution_enabled);

    let rollback_plan = prepare_safe_write_rollback_plan_scaffold(&plan);
    assert!(!rollback_plan.rollback_execution_enabled);

    let approvals = verify_manual_approval_state_scaffold(&plan);
    assert!(!approvals.executor_wiring_approved);
    assert!(!approvals.real_write_scope_approved);
    assert!(!approvals.first_real_config_write_approved);
    assert!(!approvals.gui_real_write_controls_approved);

    let receipt = execute_safe_write_scaffold(&plan);
    assert_eq!(
        receipt.status,
        StructuredFamilySafeWriteScaffoldStatus::RejectedByDefault
    );
    assert!(!receipt.executed);
    assert!(!receipt.real_config_touched);
    assert!(!receipt.backup_created);
    assert!(!receipt.restore_executed);
    assert!(!receipt.rollback_executed);
    assert!(!receipt.reload_run);
    assert!(!receipt.runtime_mutated);

    let result = verify_safe_write_result_scaffold(&receipt);
    assert!(!result.passed);
    let audit = emit_safe_write_audit_record_scaffold(&receipt);
    assert!(!audit.execution_recorded);
    assert!(audit.summary.contains("rejected by default"));

    let stop = emergency_stop_reason_scaffold();
    assert_eq!(
        stop.reason,
        StructuredFamilySafeWriteRejectionReason::ExecutorWiringNotApproved
    );
}

#[test]
fn executor_scaffold_exposes_all_required_rejection_reasons() {
    let reasons = structured_family_safe_write_default_rejection_reasons();
    let expected = [
        StructuredFamilySafeWriteRejectionReason::ExecutorWiringNotApproved,
        StructuredFamilySafeWriteRejectionReason::RealWriteScopeNotApproved,
        StructuredFamilySafeWriteRejectionReason::RealConfigTargetNotApproved,
        StructuredFamilySafeWriteRejectionReason::BackupExecutionNotApproved,
        StructuredFamilySafeWriteRejectionReason::RestoreExecutionNotApproved,
        StructuredFamilySafeWriteRejectionReason::RollbackExecutionNotApproved,
        StructuredFamilySafeWriteRejectionReason::HyprlandReloadNotApproved,
        StructuredFamilySafeWriteRejectionReason::RuntimeMutationNotApproved,
        StructuredFamilySafeWriteRejectionReason::FirstRealConfigWriteNotApproved,
        StructuredFamilySafeWriteRejectionReason::GuiRealWriteControlsNotApproved,
        StructuredFamilySafeWriteRejectionReason::UnsupportedOrNotProvenRecord,
        StructuredFamilySafeWriteRejectionReason::BlockedPlan,
        StructuredFamilySafeWriteRejectionReason::ActivationSubsetNotSelected,
        StructuredFamilySafeWriteRejectionReason::ProductionReadinessNotApproved,
    ];

    for reason in expected {
        assert!(
            reasons.contains(&reason),
            "missing rejection reason {}",
            reason.as_str()
        );
    }
}

#[test]
fn executor_scaffold_source_has_no_mutating_paths_or_ui_wiring() {
    let source = fs::read_to_string("src/structured_family_safe_write.rs")
        .expect("safe-write scaffold source should read");

    for forbidden in [
        "apply_setting_change(",
        "write_flow::",
        "hyprctl reload",
        "hyprctl ",
        "/home/kyo/.config/hypr/hyprland.conf",
        "~/.config/hypr",
        "fs::write",
        "File::create",
        "write_all",
        "serde_json::to_writer",
        "Command::",
        "gtk::",
        "Button::",
        "connect_clicked",
        "productionActivationApproved: true",
        "executorWiringApproved: true",
        "executorWired: true",
        "realWritePathEnabled: true",
        "guiRealWriteControlsEnabled: true",
    ] {
        assert!(
            !source.contains(forbidden),
            "safe-write scaffold source must not contain {forbidden}"
        );
    }
}

#[test]
fn executor_scaffold_is_unreachable_from_write_flow_and_ui() {
    let write_flow =
        fs::read_to_string("src/write_flow.rs").expect("write_flow source should read");
    assert!(!write_flow.contains("structured_family_safe_write"));
    assert!(!write_flow.contains("execute_safe_write_scaffold"));

    let ui_model = fs::read_to_string("src/ui/model.rs").expect("ui model source should read");
    assert!(!ui_model.contains("structured_family_safe_write"));
    assert!(!ui_model.contains("execute_safe_write_scaffold"));

    let ui_window = fs::read_to_string("src/ui/window.rs").expect("ui window source should read");
    assert!(!ui_window.contains("structured_family_safe_write"));
    assert!(!ui_window.contains("execute_safe_write_scaffold"));
}

#[test]
fn executor_implementation_scaffold_report_preserves_non_approval_state() {
    let report: serde_json::Value = serde_json::from_str(
        &fs::read_to_string(
            "data/reports/structured-family-executor-implementation-scaffold.v0.55.2.json",
        )
        .expect("executor scaffold report should read"),
    )
    .expect("executor scaffold report should be valid JSON");

    assert_eq!(
        report["artifactKind"],
        "structured-family-executor-implementation-scaffold"
    );
    assert_eq!(report["actualExecutorImplementationApproved"], true);
    assert_eq!(report["executorImplementationApproved"], true);
    assert_eq!(report["executorScaffoldImplemented"], true);
    assert_eq!(report["executorScaffoldInert"], true);
    assert_eq!(report["executorScaffoldUnwired"], true);
    assert_eq!(report["executorWiringApproved"], false);
    assert_eq!(report["executorWired"], false);
    assert_eq!(report["realWriteScopeApproved"], false);
    assert_eq!(report["guiRealWriteControlsApproved"], false);

    for key in [
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
    assert_eq!(report["familyRankingExcluded"], true);
    assert_eq!(
        report["productionReadinessDecision"],
        "not production ready"
    );

    for key in [
        "scaffoldModules",
        "scaffoldTypes",
        "scaffoldFunctions",
        "defaultRejectionReasons",
        "scaffoldBehavior",
        "uiReachabilityBoundary",
        "writeFlowBoundary",
        "applySettingChangeBoundary",
        "filesystemBoundary",
        "backupRestoreBoundary",
        "rollbackRecoveryBoundary",
        "reloadRuntimeBoundary",
        "futureApprovalGates",
    ] {
        assert!(
            !report[key]
                .as_array()
                .expect("scaffold report field should be array")
                .is_empty(),
            "{key} should be populated"
        );
    }

    assert!(report["defaultRejectionReasons"]
        .as_array()
        .expect("defaultRejectionReasons should be array")
        .iter()
        .any(|value| value.as_str() == Some("ExecutorWiringNotApproved")));
    assert!(report["scaffoldBehavior"]
        .as_array()
        .expect("scaffoldBehavior should be array")
        .iter()
        .any(|value| value.as_str() == Some("execute scaffold rejects by default")));

    let next = report["nextRecommendedWork"]
        .as_str()
        .expect("nextRecommendedWork should be text");
    assert_eq!(
        next,
        "Stop for explicit user decision: approve or reject executor wiring planning."
    );
    assert!(!next.contains("automatically"));
    assert!(!next.contains("GUI"));
    assert!(!next.contains("real writes"));
}
