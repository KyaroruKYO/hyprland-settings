use std::fs;

use hyprland_settings::safe_batch_write::{
    safe_batch_write_user_facing_lines, SafeBatchEligibility,
};
use hyprland_settings::write_classification::SAFE_WRITABLE_ROWS;

fn read_json(path: &str) -> serde_json::Value {
    serde_json::from_slice(&fs::read(path).expect("report should exist"))
        .expect("report should parse")
}

#[test]
fn future_capability_reports_exist_and_keep_unsafe_production_tracks_disabled() {
    let reports = [
        "data/reports/future-capability-duplicate-resolution.v0.55.2.json",
        "data/reports/future-capability-high-risk-recovery.v0.55.2.json",
        "data/reports/future-capability-structured-families.v0.55.2.json",
        "data/reports/future-capability-profile-mode-switching.v0.55.2.json",
        "data/reports/future-capability-runtime-reload.v0.55.2.json",
        "data/reports/future-capability-hyprland-0554-migration.json",
    ];

    for report_path in reports {
        let report = read_json(report_path);
        assert_eq!(
            report["startingCommit"],
            "895b67281f7551789e5b4a07c0ea849db1eab622"
        );
        assert_eq!(report["whetherRealConfigTouched"], false);
        if report_path == "data/reports/future-capability-runtime-reload.v0.55.2.json" {
            assert_eq!(report["whetherRuntimeTouched"], true);
        } else {
            assert_eq!(report["whetherRuntimeTouched"], false);
        }
        assert_eq!(report["whetherProductionBehaviorEnabled"], false);
        assert_ne!(report["implementationStatus"], "implemented_and_enabled");
    }

    let insertion =
        read_json("data/reports/future-capability-missing-default-insertion.v0.55.2.json");
    assert_eq!(insertion["implementationStatus"], "implemented_and_enabled");
    assert_eq!(
        insertion["safetyBoundaries"]["productionInsertionEnabled"],
        true
    );
    assert_eq!(insertion["whetherRealConfigTouched"], false);
    assert_eq!(insertion["whetherRuntimeTouched"], false);
}

#[test]
fn marathon_summary_attempts_all_tracks_and_preserves_release_scope() {
    let summary = read_json("data/reports/future-capability-marathon-summary.v0.55.2.json");
    assert_eq!(summary["branch"], "future-capability-marathon");
    assert_eq!(
        summary["startingCommit"],
        "895b67281f7551789e5b4a07c0ea849db1eab622"
    );
    assert_eq!(summary["safeReleaseScopePreserved"], true);
    assert_eq!(summary["v0552ModelPreserved"], true);
    assert_eq!(summary["hyprland0554MigrationActivated"], false);
    assert_eq!(summary["unsafeProductionBehaviorEnabled"], false);
    assert_eq!(summary["distV010Modified"], false);

    let phases = summary["phasesAttempted"]
        .as_array()
        .expect("phasesAttempted should be an array");
    assert_eq!(phases.len(), 7);
    assert!(summary["phasesCompleted"]
        .as_array()
        .expect("phasesCompleted should be an array")
        .iter()
        .any(|phase| phase == "runtime_dry_run_boundary"));
    assert!(summary["phasesBlocked"]
        .as_array()
        .expect("phasesBlocked should be an array")
        .iter()
        .any(|phase| phase == "production_duplicate_resolution"));
}

#[test]
fn handoff_identifies_next_concrete_work_without_enabling_runtime_paths() {
    let handoff = read_json("data/reports/future-capability-marathon-handoff.v0.55.2.json");
    assert_eq!(handoff["currentBranch"], "future-capability-marathon");
    assert_eq!(handoff["runtimeTouched"], true);
    assert_eq!(handoff["realConfigTouched"], false);
    assert_eq!(
        handoff["nextExactPhaseToContinue"],
        "Extend the disabled approval UI pattern to source/include, duplicate, structured, profile, high-risk, and 0.55.4 review cards without enabling production behavior."
    );
    assert!(handoff["recommendedNextCodexPrompt"]
        .as_str()
        .expect("prompt should be text")
        .contains("Extend the disabled approval UI pattern"));
}

#[test]
fn active_safe_batch_copy_still_blocks_future_tracks() {
    assert_eq!(SAFE_WRITABLE_ROWS.len(), 341);
    assert!(safe_batch_write_user_facing_lines()
        .iter()
        .any(|line| line.contains("Safe batch write")));
    assert_eq!(
        SafeBatchEligibility::BlockedMissingLine.user_facing_blocked_copy(),
        "Blocked: this setting uses Hyprland's default value, and this config layout is not safe for automatic insertion."
    );
    assert!(SafeBatchEligibility::BlockedDuplicateConflict
        .user_facing_blocked_copy()
        .contains("appears in more than one place"));
    assert!(SafeBatchEligibility::BlockedHighRisk
        .user_facing_blocked_copy()
        .contains("family-specific recovery path"));
    assert!(SafeBatchEligibility::BlockedStructuredFamily
        .user_facing_blocked_copy()
        .contains("structured settings are not part"));
    assert!(SafeBatchEligibility::BlockedProfileModeSwitch
        .user_facing_blocked_copy()
        .contains("profile and mode switching"));
}

#[test]
fn copied_config_tree_report_records_restored_proofs_without_production_enablement() {
    let report = read_json("data/reports/copied-config-tree-proof.v0.55.2.json");
    assert_eq!(report["projectDataVersion"], "v0.55.2");
    assert_eq!(
        report["countsAfter"],
        "341 readable / 341 writable / 0 blocked"
    );
    assert_eq!(report["copiedConfigTreeHarness"]["implemented"], true);
    assert_eq!(
        report["proofs"]["sourceIncludeSelectedTargetCopiedTree"]["status"],
        "copied_config_tree_proven"
    );
    assert_eq!(
        report["proofs"]["duplicateReplacementCopiedTree"]["status"],
        "copied_config_tree_proven"
    );
    assert_eq!(
        report["proofs"]["structuredHlBindCopiedTree"]["status"],
        "copied_config_tree_proven"
    );
    assert_eq!(
        report["proofs"]["profileModeCopiedTree"]["tempSymlinkSwitchSucceededAndRestored"],
        true
    );
    assert_eq!(report["restoration"]["realConfigTouched"], false);
    assert_eq!(report["restoration"]["runtimeTouched"], false);
    assert_eq!(
        report["productionGates"]["sourceIncludeInsertionDefaultEnabled"],
        false
    );
    assert_eq!(
        report["productionGates"]["duplicateWritesDefaultEnabled"],
        false
    );
    assert_eq!(
        report["productionGates"]["hyprland0554MigrationDefaultEnabled"],
        false
    );
}

#[test]
fn production_gate_readiness_report_keeps_all_future_gates_default_disabled() {
    let report = read_json("data/reports/production-gate-readiness.v0.55.2.json");
    assert_eq!(report["projectDataVersion"], "v0.55.2");
    assert_eq!(
        report["countsPreserved"],
        "341 readable / 341 writable / 0 blocked"
    );
    assert_eq!(report["proofAvailable"]["copiedConfigTreeProof"], true);
    assert_eq!(report["proofAvailable"]["realRuntimeMutationProof"], true);
    assert_eq!(
        report["gateDefaults"]["sourceIncludeInsertionEnabledByDefault"],
        false
    );
    assert_eq!(
        report["gateDefaults"]["duplicateWritesEnabledByDefault"],
        false
    );
    assert_eq!(
        report["gateDefaults"]["structuredWritesEnabledByDefault"],
        false
    );
    assert_eq!(
        report["gateDefaults"]["hyprland0554MigrationEnabledByDefault"],
        false
    );
    assert!(report["defaultDisabledGateReviewImplemented"]
        .as_array()
        .expect("gate review list")
        .iter()
        .any(|item| item == "duplicate occurrence replacement"));
    assert!(report["explicitApprovalFlowImplemented"]
        .as_array()
        .expect("approval flow list")
        .iter()
        .any(|item| item == "runtime_keyword"));
    assert_eq!(report["releaseBoundariesPreserved"]["mainModified"], false);
    assert_eq!(
        report["releaseBoundariesPreserved"]["v0552DefaultPreserved"],
        true
    );
}

#[test]
fn default_disabled_gate_and_runtime_evidence_reports_keep_production_disabled() {
    let gates = read_json("data/reports/default-disabled-production-gates.v0.55.2.json");
    assert_eq!(gates["projectDataVersion"], "v0.55.2");
    assert_eq!(
        gates["sourceIncludeProductionGate"]["status"],
        "ready_but_default_disabled"
    );
    assert_eq!(
        gates["duplicateProductionGate"]["status"],
        "ready_but_default_disabled"
    );
    assert_eq!(
        gates["structuredHlBindProductionGate"]["status"],
        "ready_but_default_disabled"
    );
    assert_eq!(gates["safety"]["sourceIncludeInsertionEnabled"], false);
    assert_eq!(gates["safety"]["duplicateWritesEnabled"], false);
    assert_eq!(gates["safety"]["structuredWritesEnabled"], false);
    assert_eq!(gates["safety"]["v0552DefaultPreserved"], true);
    assert_eq!(gates["approvalFlow"]["implemented"], true);
    assert_eq!(
        gates["approvalFlow"]["approvalDoesNotEnableProductionByDefault"],
        true
    );

    let runtime = read_json("data/reports/runtime-readonly-evidence.v0.55.2.json");
    assert_eq!(runtime["evidence"]["hyprctlBinaryFound"], true);
    assert_eq!(runtime["evidence"]["readOnlyHyprctlQueriesSucceeded"], true);
    assert_eq!(runtime["evidence"]["mutatingHyprctlRun"], true);
    assert_eq!(runtime["evidence"]["runtimeTouched"], true);
    assert_eq!(runtime["productionGate"]["runtimeMutationEnabled"], false);
    assert!(runtime["commands"]["hyprctlGetoptionGeneralGapsOut"]
        .as_str()
        .expect("gaps_out evidence should be text")
        .contains("css gap data: 10 10 10 10"));
}

#[test]
fn explicit_approval_and_live_restore_reports_record_default_disabled_runtime_path() {
    let approval = read_json("data/reports/explicit-approval-flow.v0.55.2.json");
    assert_eq!(approval["projectDataVersion"], "v0.55.2");
    assert_eq!(approval["approvalDefaults"]["productionFlagDefault"], false);
    assert_eq!(
        approval["scopeCoverage"]["runtime_keyword"]["status"],
        "implemented_but_disabled"
    );
    assert!(approval["tests"]
        .as_array()
        .expect("tests")
        .iter()
        .any(|item| item == "explicit_approval_flow_blocks_missing_wrong_expired_rejected_and_incomplete_evidence"));

    let live_restore = read_json("data/reports/runtime-live-restore-proof.v0.55.2.json");
    assert_eq!(live_restore["projectDataVersion"], "v0.55.2");
    assert_eq!(
        live_restore["liveShellProof"]["status"],
        "live_restore_proven"
    );
    assert_eq!(live_restore["liveShellProof"]["mutatingHyprctlRun"], true);
    assert_eq!(live_restore["liveShellProof"]["runtimeTouched"], true);
    assert_eq!(live_restore["liveShellProof"]["restorationVerified"], true);
    assert_eq!(
        live_restore["modelProof"]["statusWithSimulatedRestore"],
        "live_restore_proven"
    );
    assert_eq!(
        live_restore["liveRestoreProof"]["postMutationReadback"],
        "css gap data: 6 6 6 6; set: true"
    );
    assert_eq!(
        live_restore["productionGate"]["runtimeMutationEnabled"],
        false
    );

    let socket = read_json("data/reports/runtime-socket-diagnosis.v0.55.2.json");
    assert_eq!(socket["projectDataVersion"], "v0.55.2");
    assert_eq!(socket["sandboxDiagnosis"]["status"], "permission_mismatch");
    assert_eq!(
        socket["realSessionDiagnosis"]["status"],
        "hyprctl_readonly_succeeded"
    );
    assert_eq!(socket["liveRestoreAttempt"]["runtimeLeftUnchanged"], true);
    assert_eq!(
        socket["liveRestoreAttempt"]["luaConfigEvalMutation"],
        "succeeded: hyprctl eval 'hl.config({ general = { gaps_in = 6 } })'"
    );
    assert_eq!(socket["productionGate"]["runtimeReloadEnabled"], false);

    let syntax = read_json("data/reports/runtime-mutation-syntax-proof.v0.55.2.json");
    assert_eq!(
        syntax["successfulMutationSyntax"],
        "hyprctl eval 'hl.config({ general = { gaps_in = 6 } })'"
    );
    assert_eq!(syntax["runtimeRestored"], true);
    assert_eq!(syntax["productionRuntimeEnabled"], false);

    let approval = read_json("data/reports/runtime-approval-live-restore-gate.v0.55.2.json");
    assert_eq!(approval["projectDataVersion"], "v0.55.2");
    assert_eq!(
        approval["runtimeApprovalReview"]["status"],
        "approved_but_default_disabled"
    );
    assert_eq!(
        approval["runtimeApprovalReview"]["productionRuntimeEnabled"],
        false
    );
    assert_eq!(
        approval["runtimeApprovalReview"]["requiresLiveRestoreProof"],
        true
    );
    assert_eq!(
        approval["runtimeApprovalReview"]["successfulMutationSyntax"],
        "hyprctl eval 'hl.config({ general = { gaps_in = VALUE } })'"
    );

    let ui_surface = read_json("data/reports/runtime-approval-ui-surface.v0.55.2.json");
    assert_eq!(ui_surface["projectDataVersion"], "v0.55.2");
    assert_eq!(ui_surface["uiSurface"]["implemented"], true);
    assert_eq!(
        ui_surface["uiSurface"]["widgetName"],
        "hyprland-settings-runtime-approval-review-disabled"
    );
    assert_eq!(
        ui_surface["uiSurface"]["disabledActionWidgetName"],
        "hyprland-settings-runtime-approval-enable-disabled"
    );
    assert_eq!(ui_surface["uiSurface"]["disabledActionSensitive"], false);
    assert_eq!(
        ui_surface["visibleEvidence"]["mutationCommand"],
        "hyprctl eval 'hl.config({ general = { gaps_in = 6 } })'"
    );
    assert_eq!(
        ui_surface["visibleEvidence"]["restoreCommand"],
        "hyprctl eval 'hl.config({ general = { gaps_in = 5 } })'"
    );
    assert_eq!(
        ui_surface["visibleEvidence"]["approvalStatus"],
        "Approved but default-disabled"
    );
    assert_eq!(ui_surface["safety"]["runtimeMutatedThisSprint"], false);
    assert_eq!(
        ui_surface["safety"]["runtimeReloadProductionEnabled"],
        false
    );
}
