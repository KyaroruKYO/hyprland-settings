use std::fs;

use hyprland_settings::safe_batch_write::{
    safe_batch_write_user_facing_lines, SafeBatchEligibility,
};
use hyprland_settings::write_classification::SAFE_WRITABLE_ROWS;
use serde_json::json;

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
    assert_eq!(handoff["runtimeTouched"], false);
    assert_eq!(handoff["realConfigTouched"], false);
    assert_eq!(
        handoff["nextExactPhaseToContinue"],
        "Define future explicit production flag and executor-wiring opt-in implementation requirements without enabling production."
    );
    assert!(handoff["recommendedNextCodexPrompt"]
        .as_str()
        .expect("prompt should be text")
        .contains("executor-wiring opt-in"));
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

    let disabled_cards = read_json("data/reports/disabled-approval-ui-cards.v0.55.2.json");
    assert_eq!(disabled_cards["projectDataVersion"], "v0.55.2");
    assert_eq!(disabled_cards["uiSurface"]["implemented"], true);
    assert_eq!(
        disabled_cards["uiSurface"]["sectionWidgetName"],
        "hyprland-settings-disabled-approval-cards-section"
    );
    assert_eq!(
        disabled_cards["uiSurface"]["allPlannedEnableControlsSensitive"],
        false
    );
    assert_eq!(disabled_cards["uiSurface"]["deepDataImplemented"], true);
    for key in [
        "sourceIncludeInsertion",
        "duplicateReplacement",
        "structuredHlBindWrite",
        "profileModeSwitch",
        "highRiskDisplayWrite",
        "hyprland0554Migration",
    ] {
        assert_eq!(
            disabled_cards["cards"][key]["status"],
            "implemented_but_disabled"
        );
        assert_eq!(disabled_cards["cards"][key]["productionStatus"], "Disabled");
    }
    assert_eq!(
        disabled_cards["cards"]["sourceIncludeInsertion"]["proofSource"],
        "copied-config-tree proof"
    );
    assert_eq!(
        disabled_cards["cards"]["duplicateReplacement"]["proofFields"]["copiedReplacementStatus"],
        "selected duplicate replaced and reread in copied tree"
    );
    assert_eq!(
        disabled_cards["cards"]["structuredHlBindWrite"]["restoreEvidence"]["copiedTargetRestore"],
        "restored byte-for-byte"
    );
    assert_eq!(
        disabled_cards["cards"]["profileModeSwitch"]["restoreEvidence"]["realSymlinkUntouched"],
        "verified untouched"
    );
    assert_eq!(
        disabled_cards["cards"]["highRiskDisplayWrite"]["proofFields"]["insufficiencyReason"],
        "low-risk runtime proof does not prove display recovery"
    );
    assert_eq!(
        disabled_cards["cards"]["hyprland0554Migration"]["proofFields"]["currentActiveAppModel"],
        "v0.55.2"
    );
    assert_eq!(
        disabled_cards["safety"]["unsafeProductionBehaviorEnabled"],
        false
    );
    assert_eq!(
        disabled_cards["safety"]["hyprland0554MigrationActivated"],
        false
    );

    let deep_cards = read_json("data/reports/deep-approval-card-data.v0.55.2.json");
    assert_eq!(deep_cards["projectDataVersion"], "v0.55.2");
    assert_eq!(
        deep_cards["implementation"]["deepProjectionModelExists"],
        true
    );
    assert_eq!(deep_cards["implementation"]["uiRendersProofRecords"], true);
    assert_eq!(deep_cards["implementation"]["allCardsRemainDisabled"], true);
    for key in [
        "sourceIncludeInsertion",
        "duplicateReplacement",
        "structuredHlBindWrite",
        "profileModeSwitch",
        "highRiskDisplayWrite",
        "hyprland0554Migration",
    ] {
        assert_eq!(deep_cards["deepCards"][key]["productionEnabled"], false);
        assert!(
            deep_cards["deepCards"][key]["requiredFieldsDisplayed"]
                .as_array()
                .expect("deep card fields")
                .len()
                >= 6
        );
    }
    assert_eq!(deep_cards["safety"]["runtimeMutatedThisSprint"], false);
    assert_eq!(deep_cards["safety"]["v0552DefaultPreserved"], true);

    let report_backed = read_json("data/reports/report-backed-approval-card-data.v0.55.2.json");
    assert_eq!(report_backed["projectDataVersion"], "v0.55.2");
    assert_eq!(report_backed["implementation"]["typedAdapterExists"], true);
    assert_eq!(
        report_backed["implementation"]["cardsLoadedFromSerializedReport"],
        true
    );
    assert_eq!(
        report_backed["implementation"]["uiRendersReportBackedProjection"],
        true
    );
    for key in [
        "sourceIncludeInsertion",
        "duplicateReplacement",
        "structuredHlBindWrite",
        "profileModeSwitch",
        "highRiskDisplayWrite",
        "hyprland0554Migration",
    ] {
        assert_eq!(report_backed["cards"][key]["reportBacked"], true);
        assert_eq!(report_backed["cards"][key]["productionEnabled"], false);
        assert_eq!(
            report_backed["screenshotLevelAssertions"][key], true,
            "{key} should have screenshot-level assertion coverage"
        );
    }
    assert_eq!(
        report_backed["safety"]["unsafeProductionBehaviorEnabled"],
        false
    );

    let activation_decision =
        read_json("data/reports/default-disabled-production-activation-decision.v0.55.2.json");
    assert_eq!(activation_decision["projectDataVersion"], "v0.55.2");
    assert_eq!(
        activation_decision["implementation"]["consumesReportBackedApprovalCardData"],
        true
    );
    for key in ["sourceIncludeInsertion", "duplicateReplacement"] {
        assert_eq!(
            activation_decision["decisions"][key]["status"],
            "ApprovedButDefaultDisabled"
        );
        assert_eq!(
            activation_decision["decisions"][key]["productionEnabled"],
            false
        );
        assert_eq!(
            activation_decision["decisions"][key]["productionFlag"],
            false
        );
        assert_eq!(
            activation_decision["decisions"][key]["productionStatus"],
            "Disabled"
        );
        assert_eq!(activation_decision["screenshotLevelAssertions"][key], true);
    }
    assert_eq!(
        activation_decision["safety"]["unsafeProductionBehaviorEnabled"],
        false
    );

    let activation_path =
        read_json("data/reports/default-disabled-production-activation-path.v0.55.2.json");
    assert_eq!(activation_path["projectDataVersion"], "v0.55.2");
    assert_eq!(
        activation_path["implementation"]["activationPathModelExists"],
        true
    );
    assert_eq!(
        activation_path["implementation"]["activationRequestModelExists"],
        true
    );
    assert_eq!(
        activation_path["implementation"]["activationSafetyPlanModelExists"],
        true
    );
    for key in ["sourceIncludeInsertion", "duplicateReplacement"] {
        assert_eq!(
            activation_path["paths"][key]["inputDecisionStatus"],
            "ApprovedButDefaultDisabled"
        );
        assert_eq!(
            activation_path["paths"][key]["status"],
            "ActivationPathNeedsExplicitProductionFlag"
        );
        assert_eq!(activation_path["paths"][key]["productionEnabled"], false);
        assert_eq!(
            activation_path["paths"][key]["productionActivationFlag"],
            false
        );
        assert_eq!(
            activation_path["paths"][key]["categoryProductionFlag"],
            false
        );
        assert_eq!(
            activation_path["paths"][key]["productionStatus"],
            "Disabled"
        );
        assert_eq!(activation_path["screenshotLevelAssertions"][key], true);
    }
    assert_eq!(
        activation_path["safety"]["unsafeProductionBehaviorEnabled"],
        false
    );

    let activation_control =
        read_json("data/reports/default-disabled-production-activation-control.v0.55.2.json");
    assert_eq!(activation_control["projectDataVersion"], "v0.55.2");
    assert_eq!(
        activation_control["implementation"]["activationControlModelExists"],
        true
    );
    assert_eq!(
        activation_control["implementation"]["executorWiringModelExists"],
        true
    );
    assert_eq!(
        activation_control["implementation"]["uiSurfaceExists"],
        true
    );
    for key in ["sourceIncludeInsertion", "duplicateReplacement"] {
        assert_eq!(
            activation_control["controls"][key]["inputPathStatus"],
            "ActivationPathNeedsExplicitProductionFlag"
        );
        assert_eq!(
            activation_control["controls"][key]["status"],
            "ValidatedButExecutorUnwired"
        );
        assert_eq!(
            activation_control["controls"][key]["requestValidationStatus"],
            "Complete activation request"
        );
        assert_eq!(
            activation_control["controls"][key]["safetyPlanValidationStatus"],
            "Complete safety plan"
        );
        assert_eq!(
            activation_control["controls"][key]["executorWiringStatus"],
            "Unwired"
        );
        assert_eq!(
            activation_control["controls"][key]["productionEnabled"],
            false
        );
        assert_eq!(activation_control["controls"][key]["productionFlag"], false);
        assert_eq!(
            activation_control["controls"][key]["productionStatus"],
            "Disabled"
        );
        assert_eq!(activation_control["screenshotLevelAssertions"][key], true);
    }
    assert_eq!(
        activation_control["safety"]["unsafeProductionBehaviorEnabled"],
        false
    );
    assert_eq!(
        activation_control["safety"]["sourceIncludeExecutorWired"],
        false
    );
    assert_eq!(
        activation_control["safety"]["duplicateExecutorWired"],
        false
    );

    let activation_form =
        read_json("data/reports/default-disabled-production-activation-form.v0.55.2.json");
    assert_eq!(activation_form["projectDataVersion"], "v0.55.2");
    assert_eq!(
        activation_form["implementation"]["activationFormStateMachineExists"],
        true
    );
    assert_eq!(
        activation_form["implementation"]["requestGenerationExists"],
        true
    );
    assert_eq!(
        activation_form["implementation"]["safetyPlanGenerationExists"],
        true
    );
    assert_eq!(
        activation_form["implementation"]["controlValidationFromFormExists"],
        true
    );
    assert_eq!(
        activation_form["implementation"]["realDisabledGtkFormFieldsExist"],
        true
    );
    assert_eq!(
        activation_form["implementation"]["fieldsAreReadOnlyOrDisabled"],
        true
    );
    for key in ["sourceIncludeInsertion", "duplicateReplacement"] {
        assert_eq!(
            activation_form["forms"][key]["status"],
            "ValidatedForReviewOnly"
        );
        assert_eq!(
            activation_form["forms"][key]["requestGenerationStatus"],
            "ProductionActivationRequest generated for review only"
        );
        assert_eq!(
            activation_form["forms"][key]["safetyPlanGenerationStatus"],
            "ProductionActivationSafetyPlan generated for review only"
        );
        assert_eq!(
            activation_form["forms"][key]["controlValidationStatus"],
            "ValidatedButExecutorUnwired"
        );
        assert_eq!(
            activation_form["forms"][key]["executorWiringStatus"],
            "Unwired"
        );
        assert_eq!(activation_form["forms"][key]["productionEnabled"], false);
        assert_eq!(activation_form["forms"][key]["productionFlag"], false);
        assert_eq!(activation_form["forms"][key]["executorWired"], false);
        assert_eq!(activation_form["forms"][key]["reviewOnly"], true);
        assert!(activation_form["forms"][key]["fieldWidgets"].is_object());
        assert_eq!(activation_form["screenshotLevelAssertions"][key], true);
    }
    assert_eq!(
        activation_form["safety"]["unsafeProductionBehaviorEnabled"],
        false
    );
    assert_eq!(
        activation_form["safety"]["sourceIncludeExecutorWired"],
        false
    );
    assert_eq!(activation_form["safety"]["duplicateExecutorWired"], false);

    let activation_form_fields =
        read_json("data/reports/default-disabled-production-activation-form-fields.v0.55.2.json");
    assert_eq!(activation_form_fields["projectDataVersion"], "v0.55.2");
    assert_eq!(
        activation_form_fields["implementation"]["sourceIncludeRealDisabledGtkFormFieldsExist"],
        true
    );
    assert_eq!(
        activation_form_fields["implementation"]["duplicateRealDisabledGtkFormFieldsExist"],
        true
    );
    assert_eq!(
        activation_form_fields["implementation"]["entryFieldsUseEditableFalse"],
        true
    );
    assert_eq!(
        activation_form_fields["implementation"]["textViewFieldsUseEditableFalse"],
        true
    );
    assert_eq!(
        activation_form_fields["implementation"]["checkFieldsUseSensitiveFalse"],
        true
    );
    for key in ["sourceIncludeInsertion", "duplicateReplacement"] {
        assert_eq!(
            activation_form_fields["forms"][key]["status"],
            "ValidatedForReviewOnly"
        );
        assert_eq!(
            activation_form_fields["forms"][key]["controlValidationStatus"],
            "ValidatedButExecutorUnwired"
        );
        assert_eq!(
            activation_form_fields["forms"][key]["executorWiringStatus"],
            "Unwired"
        );
        assert_eq!(
            activation_form_fields["forms"][key]["productionStatus"],
            "Disabled"
        );
        assert_eq!(
            activation_form_fields["forms"][key]["productionEnabled"],
            false
        );
        assert_eq!(
            activation_form_fields["forms"][key]["productionFlag"],
            false
        );
        assert_eq!(activation_form_fields["forms"][key]["executorWired"], false);
        assert!(activation_form_fields["forms"][key]["fieldWidgets"].is_object());
        assert!(activation_form_fields["forms"][key]["visibleFieldLabels"]
            .as_array()
            .expect("visible field labels")
            .contains(&json!("User-facing reason")));
        assert_eq!(
            activation_form_fields["screenshotLevelAssertions"][key],
            true
        );
    }
    assert_eq!(
        activation_form_fields["safety"]["unsafeProductionBehaviorEnabled"],
        false
    );
    assert_eq!(
        activation_form_fields["safety"]["sourceIncludeExecutorWired"],
        false
    );
    assert_eq!(
        activation_form_fields["safety"]["duplicateExecutorWired"],
        false
    );

    let activation_draft =
        read_json("data/reports/default-disabled-production-activation-draft.v0.55.2.json");
    assert_eq!(activation_draft["projectDataVersion"], "v0.55.2");
    assert_eq!(
        activation_draft["implementation"]["activationDraftModelExists"],
        true
    );
    assert_eq!(
        activation_draft["implementation"]["draftStoreIsInMemoryOnly"],
        true
    );
    assert_eq!(
        activation_draft["implementation"]["diskPersistenceAdded"],
        false
    );
    for key in ["sourceIncludeInsertion", "duplicateReplacement"] {
        assert_eq!(
            activation_draft["drafts"][key]["status"],
            "DraftValidatedForReviewOnly"
        );
        assert_eq!(
            activation_draft["drafts"][key]["persistenceStatus"],
            "In-memory only"
        );
        assert_eq!(
            activation_draft["drafts"][key]["updateResetStatus"],
            "Modeled and tested in memory only"
        );
        assert_eq!(
            activation_draft["drafts"][key]["controlValidationStatus"],
            "ValidatedButExecutorUnwired"
        );
        assert_eq!(
            activation_draft["drafts"][key]["executorWiringStatus"],
            "Unwired"
        );
        assert_eq!(
            activation_draft["drafts"][key]["productionStatus"],
            "Disabled"
        );
        assert_eq!(activation_draft["drafts"][key]["productionEnabled"], false);
        assert_eq!(activation_draft["drafts"][key]["productionFlag"], false);
        assert_eq!(activation_draft["drafts"][key]["executorWired"], false);
        assert_eq!(
            activation_draft["drafts"][key]["draftPersistsToDisk"],
            false
        );
        assert_eq!(activation_draft["screenshotLevelAssertions"][key], true);
    }
    assert_eq!(
        activation_draft["safety"]["unsafeProductionBehaviorEnabled"],
        false
    );
    assert_eq!(activation_draft["safety"]["diskPersistenceAdded"], false);
    assert_eq!(
        activation_draft["safety"]["sourceIncludeExecutorWired"],
        false
    );
    assert_eq!(activation_draft["safety"]["duplicateExecutorWired"], false);

    let activation_draft_edit =
        read_json("data/reports/default-disabled-production-activation-draft-edit.v0.55.2.json");
    assert_eq!(activation_draft_edit["projectDataVersion"], "v0.55.2");
    assert_eq!(
        activation_draft_edit["implementation"]["sourceIncludeDraftEditPlumbingExists"],
        true
    );
    assert_eq!(
        activation_draft_edit["implementation"]["duplicateDraftEditPlumbingExists"],
        true
    );
    assert_eq!(
        activation_draft_edit["implementation"]["draftEditingDisabledByDefault"],
        true
    );
    assert_eq!(
        activation_draft_edit["implementation"]["draftEditingCanBeModeledInMemoryOnly"],
        true
    );
    assert_eq!(
        activation_draft_edit["implementation"]["diskPersistenceAdded"],
        false
    );
    for key in ["sourceIncludeInsertion", "duplicateReplacement"] {
        assert_eq!(
            activation_draft_edit["draftEdits"][key]["defaultMode"],
            "DraftEditingDisabledByDefault"
        );
        assert_eq!(
            activation_draft_edit["draftEdits"][key]["memoryOnlyMode"],
            "DraftEditingEnabledInMemoryOnly"
        );
        assert_eq!(
            activation_draft_edit["draftEdits"][key]["validatedStatus"],
            "DraftEditingValidatedForReviewOnly"
        );
        assert_eq!(
            activation_draft_edit["draftEdits"][key]["formValidationStatus"],
            "ValidatedForReviewOnly"
        );
        assert_eq!(
            activation_draft_edit["draftEdits"][key]["controlValidationStatus"],
            "ValidatedButExecutorUnwired"
        );
        assert_eq!(
            activation_draft_edit["draftEdits"][key]["persistenceStatus"],
            "In-memory only"
        );
        assert_eq!(
            activation_draft_edit["draftEdits"][key]["executorWiringStatus"],
            "Unwired"
        );
        assert_eq!(
            activation_draft_edit["draftEdits"][key]["productionStatus"],
            "Disabled"
        );
        assert_eq!(
            activation_draft_edit["draftEdits"][key]["productionEnabled"],
            false
        );
        assert_eq!(
            activation_draft_edit["draftEdits"][key]["productionFlag"],
            false
        );
        assert_eq!(
            activation_draft_edit["draftEdits"][key]["executorWired"],
            false
        );
        assert_eq!(
            activation_draft_edit["draftEdits"][key]["draftPersistsToDisk"],
            false
        );
        assert_eq!(
            activation_draft_edit["screenshotLevelAssertions"][key],
            true
        );
    }
    assert_eq!(
        activation_draft_edit["safety"]["unsafeProductionBehaviorEnabled"],
        false
    );
    assert_eq!(
        activation_draft_edit["safety"]["diskPersistenceAdded"],
        false
    );
    assert_eq!(
        activation_draft_edit["safety"]["sourceIncludeExecutorWired"],
        false
    );
    assert_eq!(
        activation_draft_edit["safety"]["duplicateExecutorWired"],
        false
    );

    let activation_live_draft_edit = read_json(
        "data/reports/default-disabled-production-activation-live-draft-edit.v0.55.2.json",
    );
    assert_eq!(activation_live_draft_edit["projectDataVersion"], "v0.55.2");
    assert_eq!(
        activation_live_draft_edit["implementation"]["sourceIncludeLiveDraftEditBridgeExists"],
        true
    );
    assert_eq!(
        activation_live_draft_edit["implementation"]["duplicateLiveDraftEditBridgeExists"],
        true
    );
    assert_eq!(
        activation_live_draft_edit["implementation"]["gtkFieldChangedHandlersUpdateMemoryOnly"],
        true
    );
    assert_eq!(
        activation_live_draft_edit["implementation"]["textViewBufferHandlersUpdateMemoryOnly"],
        true
    );
    assert_eq!(
        activation_live_draft_edit["implementation"]["checkButtonHandlersUpdateMemoryOnly"],
        true
    );
    assert_eq!(
        activation_live_draft_edit["implementation"]["resetHandlersUpdateMemoryOnly"],
        true
    );
    assert_eq!(
        activation_live_draft_edit["implementation"]["diskPersistenceAdded"],
        false
    );
    for key in ["sourceIncludeInsertion", "duplicateReplacement"] {
        assert_eq!(
            activation_live_draft_edit["liveDraftEditBridges"][key]["initialStatus"],
            "GtkBridgeEnabledInMemoryOnly"
        );
        assert_eq!(
            activation_live_draft_edit["liveDraftEditBridges"][key]["memoryUpdateStatus"],
            "GtkBridgeMemoryUpdated"
        );
        assert_eq!(
            activation_live_draft_edit["liveDraftEditBridges"][key]["resetStatus"],
            "GtkBridgeResetInMemoryOnly"
        );
        assert_eq!(
            activation_live_draft_edit["liveDraftEditBridges"][key]["formValidationStatus"],
            "ValidatedForReviewOnly"
        );
        assert_eq!(
            activation_live_draft_edit["liveDraftEditBridges"][key]["controlValidationStatus"],
            "ValidatedButExecutorUnwired"
        );
        assert_eq!(
            activation_live_draft_edit["liveDraftEditBridges"][key]["executorWiringStatus"],
            "Unwired"
        );
        assert_eq!(
            activation_live_draft_edit["liveDraftEditBridges"][key]["productionStatus"],
            "Disabled"
        );
        assert_eq!(
            activation_live_draft_edit["liveDraftEditBridges"][key]["productionEnabled"],
            false
        );
        assert_eq!(
            activation_live_draft_edit["liveDraftEditBridges"][key]["productionFlag"],
            false
        );
        assert_eq!(
            activation_live_draft_edit["liveDraftEditBridges"][key]["executorWired"],
            false
        );
        assert_eq!(
            activation_live_draft_edit["liveDraftEditBridges"][key]["draftPersistsToDisk"],
            false
        );
        assert_eq!(
            activation_live_draft_edit["screenshotLevelAssertions"][key],
            true
        );
    }
    assert_eq!(
        activation_live_draft_edit["safety"]["unsafeProductionBehaviorEnabled"],
        false
    );
    assert_eq!(
        activation_live_draft_edit["safety"]["diskPersistenceAdded"],
        false
    );
    assert_eq!(
        activation_live_draft_edit["safety"]["sourceIncludeExecutorWired"],
        false
    );
    assert_eq!(
        activation_live_draft_edit["safety"]["duplicateExecutorWired"],
        false
    );

    let activation_persistence_boundary = read_json(
        "data/reports/default-disabled-production-activation-draft-persistence-boundary.v0.55.2.json",
    );
    assert_eq!(
        activation_persistence_boundary["projectDataVersion"],
        "v0.55.2"
    );
    assert_eq!(
        activation_persistence_boundary["chosenOption"],
        "persistence boundary"
    );
    assert_eq!(
        activation_persistence_boundary["implementation"]["sourceIncludePersistenceBoundaryExists"],
        true
    );
    assert_eq!(
        activation_persistence_boundary["implementation"]["duplicatePersistenceBoundaryExists"],
        true
    );
    assert_eq!(
        activation_persistence_boundary["implementation"]["draftPersistenceEnabled"],
        false
    );
    assert_eq!(
        activation_persistence_boundary["implementation"]["draftDataWrittenToDisk"],
        false
    );
    assert_eq!(
        activation_persistence_boundary["implementation"]["storagePathCreated"],
        false
    );
    assert_eq!(
        activation_persistence_boundary["implementation"]["serializerOrWritePathAdded"],
        false
    );
    for key in ["sourceIncludeInsertion", "duplicateReplacement"] {
        assert_eq!(
            activation_persistence_boundary["boundaries"][key]["status"],
            "PersistenceForbiddenByDefault"
        );
        assert_eq!(
            activation_persistence_boundary["boundaries"][key]["persistenceEnabled"],
            false
        );
        assert_eq!(
            activation_persistence_boundary["boundaries"][key]["draftWrittenToDisk"],
            false
        );
        assert_eq!(
            activation_persistence_boundary["boundaries"][key]["storagePath"],
            "none"
        );
        assert_eq!(
            activation_persistence_boundary["boundaries"][key]["serializerCalled"],
            false
        );
        assert_eq!(
            activation_persistence_boundary["boundaries"][key]["executorWiringStatus"],
            "Unwired"
        );
        assert_eq!(
            activation_persistence_boundary["boundaries"][key]["productionStatus"],
            "Disabled"
        );
        assert_eq!(
            activation_persistence_boundary["boundaries"][key]["productionFlag"],
            false
        );
        assert_eq!(
            activation_persistence_boundary["screenshotLevelAssertions"][key],
            true
        );
    }
    assert_eq!(
        activation_persistence_boundary["safety"]["unsafeProductionBehaviorEnabled"],
        false
    );
    assert_eq!(
        activation_persistence_boundary["safety"]["diskPersistenceAdded"],
        false
    );
    assert_eq!(
        activation_persistence_boundary["safety"]["sourceIncludeExecutorWired"],
        false
    );
    assert_eq!(
        activation_persistence_boundary["safety"]["duplicateExecutorWired"],
        false
    );
    assert_eq!(
        activation_persistence_boundary["branchState"]["branchCapped"],
        false
    );

    let activation_safety_gates =
        read_json("data/reports/default-disabled-production-activation-safety-gates.v0.55.2.json");
    assert_eq!(activation_safety_gates["projectDataVersion"], "v0.55.2");
    assert_eq!(
        activation_safety_gates["implementation"]
            ["sourceIncludeProductionActivationSafetyGateExists"],
        true
    );
    assert_eq!(
        activation_safety_gates["implementation"]["duplicateProductionActivationSafetyGateExists"],
        true
    );
    assert_eq!(
        activation_safety_gates["implementation"]
            ["sourceIncludeProductionActivationBlockedByDefault"],
        false
    );
    assert_eq!(
        activation_safety_gates["implementation"]["duplicateProductionActivationBlockedByDefault"],
        false
    );
    for key in ["sourceIncludeInsertion", "duplicateReplacement"] {
        assert_eq!(
            activation_safety_gates["gates"][key]["status"],
            "ProductionActivationProofPartiallySatisfiedButDefaultDisabled"
        );
        assert_eq!(
            activation_safety_gates["gates"][key]["byteExactBackupProof"],
            "satisfied in copied fixture"
        );
        assert_eq!(
            activation_safety_gates["gates"][key]["writePlan"],
            "satisfied in copied fixture"
        );
        assert_eq!(
            activation_safety_gates["gates"][key]["rereadPlan"],
            "satisfied in copied fixture"
        );
        assert_eq!(
            activation_safety_gates["gates"][key]["restorePlan"],
            "satisfied in copied fixture"
        );
        assert_eq!(
            activation_safety_gates["gates"][key]["noAutoApplyProof"],
            "satisfied by report-backed evidence"
        );
        assert_eq!(
            activation_safety_gates["gates"][key]["persistedDraftAutoApplyProof"],
            "satisfied by report-backed evidence"
        );
        assert_eq!(
            activation_safety_gates["gates"][key]["executorWiringStatus"],
            "Unwired"
        );
        assert_eq!(
            activation_safety_gates["gates"][key]["productionFlag"],
            false
        );
        assert_eq!(
            activation_safety_gates["gates"][key]["executorWired"],
            false
        );
        assert_eq!(
            activation_safety_gates["gates"][key]["productionWriteExecuted"],
            false
        );
    }

    let activation_safety_proof = read_json(
        "data/reports/default-disabled-production-activation-safety-gate-proof.v0.55.2.json",
    );
    assert_eq!(activation_safety_proof["projectDataVersion"], "v0.55.2");
    assert_eq!(
        activation_safety_proof["implementation"]
            ["sourceIncludeProductionActivationSafetyProofExists"],
        true
    );
    assert_eq!(
        activation_safety_proof["implementation"]["duplicateProductionActivationSafetyProofExists"],
        true
    );
    for key in ["sourceIncludeInsertion", "duplicateReplacement"] {
        assert_eq!(
            activation_safety_proof["proofs"][key]["status"],
            "ProductionActivationProofPartiallySatisfiedButDefaultDisabled"
        );
        assert_eq!(
            activation_safety_proof["proofs"][key]["byteExactBackupProof"],
            "ProofSatisfiedInCopiedFixture"
        );
        assert_eq!(
            activation_safety_proof["proofs"][key]["preWriteSnapshotProof"],
            "ProofSatisfiedInCopiedFixture"
        );
        assert_eq!(
            activation_safety_proof["proofs"][key]["targetIdentityProof"],
            "ProofSatisfiedInCopiedFixture"
        );
        assert_eq!(
            activation_safety_proof["proofs"][key]["dryRunDiffProof"],
            "ProofSatisfiedInCopiedFixture"
        );
        assert_eq!(
            activation_safety_proof["proofs"][key]["rereadRestorePostRestoreProof"],
            "ProofSatisfiedInCopiedFixture"
        );
        assert_eq!(
            activation_safety_proof["proofs"][key]["noAutoApplyProof"],
            "ProofSatisfiedByReportBackedEvidence"
        );
        assert_eq!(
            activation_safety_proof["proofs"][key]["persistedDraftAutoApplyProof"],
            "ProofSatisfiedByReportBackedEvidence"
        );
        assert_eq!(
            activation_safety_proof["proofs"][key]["finalApproval"],
            "ProofStillRequiresExplicitUserApproval"
        );
        assert_eq!(
            activation_safety_proof["proofs"][key]["executorWiringStatus"],
            "Unwired"
        );
        assert_eq!(
            activation_safety_proof["proofs"][key]["productionFlag"],
            false
        );
        assert_eq!(
            activation_safety_proof["proofs"][key]["realConfigTouched"],
            false
        );
        assert_eq!(
            activation_safety_proof["proofs"][key]["runtimeMutated"],
            false
        );
    }
    assert_eq!(
        activation_safety_gates["draftPersistenceBoundary"]["status"],
        "PersistenceForbiddenByDefault"
    );
    assert_eq!(
        activation_safety_gates["safety"]["unsafeProductionBehaviorEnabled"],
        false
    );
    assert_eq!(
        activation_safety_gates["safety"]["diskPersistenceAdded"],
        false
    );
    assert_eq!(
        activation_safety_gates["safety"]["sourceIncludeExecutorWired"],
        false
    );
    assert_eq!(
        activation_safety_gates["safety"]["duplicateExecutorWired"],
        false
    );

    let activation_final_decisions = read_json(
        "data/reports/default-disabled-production-activation-final-decisions.v0.55.2.json",
    );
    assert_eq!(activation_final_decisions["projectDataVersion"], "v0.55.2");
    assert_eq!(
        activation_final_decisions["implementation"]["sourceIncludeFinalDecisionReviewExists"],
        true
    );
    assert_eq!(
        activation_final_decisions["implementation"]["duplicateFinalDecisionReviewExists"],
        true
    );
    for key in ["sourceIncludeInsertion", "duplicateReplacement"] {
        assert_eq!(
            activation_final_decisions["decisions"][key]["status"],
            "FinalDecisionProofSatisfiedButDecisionsMissing"
        );
        assert_eq!(
            activation_final_decisions["decisions"][key]["finalApproval"],
            "FinalDecisionRequiresExplicitUserApproval"
        );
        assert_eq!(
            activation_final_decisions["decisions"][key]["productionFlagDecision"],
            "FinalDecisionRequiresProductionFlagOptIn"
        );
        assert_eq!(
            activation_final_decisions["decisions"][key]["executorWiringDecision"],
            "FinalDecisionRequiresExecutorWiringOptIn"
        );
        assert_eq!(
            activation_final_decisions["decisions"][key]["liveProductionDryRunPolicy"],
            "FinalDecisionRequiresLiveProductionDryRunPolicy"
        );
        assert_eq!(
            activation_final_decisions["decisions"][key]["copiedFixtureProof"],
            "ProductionActivationProofPartiallySatisfiedButDefaultDisabled"
        );
        assert_eq!(
            activation_final_decisions["decisions"][key]["draftPersistenceBoundary"],
            "PersistenceForbiddenByDefault"
        );
        assert_eq!(
            activation_final_decisions["decisions"][key]["executorWiringStatus"],
            "Unwired"
        );
        assert_eq!(
            activation_final_decisions["decisions"][key]["productionFlag"],
            false
        );
        assert_eq!(
            activation_final_decisions["decisions"][key]["productionWriteExecuted"],
            false
        );
        assert_eq!(
            activation_final_decisions["decisions"][key]["realConfigTouched"],
            false
        );
        assert_eq!(
            activation_final_decisions["decisions"][key]["runtimeMutated"],
            false
        );
    }
    assert_eq!(
        activation_final_decisions["negativeProofs"]
            ["copiedFixtureProofAloneCannotApproveProduction"],
        true
    );
    assert_eq!(
        activation_final_decisions["negativeProofs"]
            ["copiedFixtureProofAloneCannotSetProductionFlag"],
        true
    );
    assert_eq!(
        activation_final_decisions["negativeProofs"]["copiedFixtureProofAloneCannotWireExecutor"],
        true
    );
    assert_eq!(
        activation_final_decisions["negativeProofs"]
            ["copiedFixtureProofAloneCannotAuthorizeLiveDryRun"],
        true
    );
    assert_eq!(
        activation_final_decisions["safety"]["unsafeProductionBehaviorEnabled"],
        false
    );

    let approval_ux_and_dry_run = read_json(
        "data/reports/default-disabled-production-activation-approval-ux-and-dry-run-policy.v0.55.2.json",
    );
    assert_eq!(approval_ux_and_dry_run["projectDataVersion"], "v0.55.2");
    assert_eq!(
        approval_ux_and_dry_run["implementation"]["sourceIncludeApprovalUxReviewExists"],
        true
    );
    assert_eq!(
        approval_ux_and_dry_run["implementation"]["duplicateApprovalUxReviewExists"],
        true
    );
    assert_eq!(
        approval_ux_and_dry_run["implementation"]["sourceIncludeLiveDryRunPolicyReviewExists"],
        true
    );
    assert_eq!(
        approval_ux_and_dry_run["implementation"]["duplicateLiveDryRunPolicyReviewExists"],
        true
    );
    for key in ["sourceIncludeInsertion", "duplicateReplacement"] {
        assert_eq!(
            approval_ux_and_dry_run["reviews"][key]["approvalUxStatus"],
            "ApprovalUxDesignedButDisabled"
        );
        assert_eq!(
            approval_ux_and_dry_run["reviews"][key]["dryRunPolicyStatus"],
            "DryRunPolicyDesignedButDisabled"
        );
        assert_eq!(
            approval_ux_and_dry_run["reviews"][key]["explicitFinalApprovalRequired"],
            true
        );
        assert_eq!(
            approval_ux_and_dry_run["reviews"][key]["typedConfirmationRequired"],
            true
        );
        assert_eq!(
            approval_ux_and_dry_run["reviews"][key]["productionFlagOptInRequired"],
            true
        );
        assert_eq!(
            approval_ux_and_dry_run["reviews"][key]["executorWiringOptInRequired"],
            true
        );
        assert_eq!(
            approval_ux_and_dry_run["reviews"][key]["liveDryRunCannotRunByDefault"],
            true
        );
        assert_eq!(
            approval_ux_and_dry_run["reviews"][key]["liveDryRunCannotTouchRealConfigByDefault"],
            true
        );
        assert_eq!(
            approval_ux_and_dry_run["reviews"][key]["liveDryRunCannotReloadHyprlandByDefault"],
            true
        );
        assert_eq!(
            approval_ux_and_dry_run["reviews"][key]["liveDryRunCannotMutateRuntimeByDefault"],
            true
        );
        assert_eq!(
            approval_ux_and_dry_run["reviews"][key]["executorWiringStatus"],
            "Unwired"
        );
        assert_eq!(
            approval_ux_and_dry_run["reviews"][key]["productionFlag"],
            false
        );
        assert_eq!(
            approval_ux_and_dry_run["reviews"][key]["productionWriteExecuted"],
            false
        );
        assert_eq!(
            approval_ux_and_dry_run["reviews"][key]["realConfigTouched"],
            false
        );
        assert_eq!(
            approval_ux_and_dry_run["reviews"][key]["runtimeMutated"],
            false
        );
    }
    assert_eq!(
        approval_ux_and_dry_run["negativeProofs"]["approvalUxDesignAloneCannotApproveProduction"],
        true
    );
    assert_eq!(
        approval_ux_and_dry_run["negativeProofs"]
            ["dryRunPolicyDesignAloneCannotAuthorizeLiveDryRun"],
        true
    );
    assert_eq!(
        approval_ux_and_dry_run["safety"]["unsafeProductionBehaviorEnabled"],
        false
    );

    let dependency_scan =
        read_json("data/reports/future-capability-remaining-dependency-scan.v0.55.2.json");
    assert_eq!(dependency_scan["projectDataVersion"], "v0.55.2");
    assert_eq!(dependency_scan["scanComplete"], true);
    for key in [
        "coreAppShellUiNavigation",
        "configDiscoverySourceAwareModel",
        "rowReadWriteModel",
        "safeNormalScalarWrites",
        "releasePackagingTagArtifacts",
        "missingDefaultInsertion",
        "duplicateResolution",
        "highRiskDisplayRecovery",
        "structuredFamilyEditorsWrites",
        "profileModeSwitching",
        "runtimeReloadIntegration",
        "hyprland0554Migration",
    ] {
        assert!(dependency_scan["classifications"][key]["classification"].is_string());
        assert!(dependency_scan["classifications"][key]["evidence"].is_string());
    }
    assert_eq!(
        dependency_scan["classifications"]["missingDefaultInsertion"]["classification"],
        "blocked by production activation"
    );
    assert_eq!(
        dependency_scan["classifications"]["duplicateResolution"]["classification"],
        "blocked by production activation"
    );
    assert_eq!(
        dependency_scan["classifications"]["highRiskDisplayRecovery"]["classification"],
        "blocked by high-risk recovery proof"
    );
    assert_eq!(
        dependency_scan["classifications"]["hyprland0554Migration"]["classification"],
        "blocked by missing official data/export"
    );
    assert_eq!(
        dependency_scan["safeIndependentExtraWork"]["attempted"],
        true
    );
    assert_eq!(
        dependency_scan["safeIndependentExtraWork"]["completed"],
        "production activation safety gate proof"
    );
    assert_eq!(
        dependency_scan["safety"]["unsafeProductionBehaviorEnabled"],
        false
    );
    assert_eq!(dependency_scan["safety"]["diskPersistenceAdded"], false);
    assert_eq!(
        dependency_scan["safety"]["sourceIncludeExecutorWired"],
        false
    );
    assert_eq!(dependency_scan["safety"]["duplicateExecutorWired"], false);

    let gtk_cards =
        read_json("data/reports/gtk-safe-env-disabled-approval-card-proof.v0.55.2.json");
    assert_eq!(gtk_cards["projectDataVersion"], "v0.55.2");
    assert_eq!(
        gtk_cards["assertionMethod"],
        "screenshot_plus_accessibility_tree_text_not_ocr"
    );
    assert_eq!(gtk_cards["allHeadingsFound"], true);
    assert_eq!(gtk_cards["allProductionDisabledTextFound"], true);
    assert_eq!(gtk_cards["allDisabledActionsFound"], true);
    for key in [
        "sourceIncludeInsertion",
        "duplicateReplacement",
        "structuredHlBindWrite",
        "profileModeSwitch",
        "highRiskDisplayWrite",
        "hyprland0554Migration",
    ] {
        assert_eq!(
            gtk_cards["approvalCardResults"][key]["headingProof"],
            "live_gtk_atspi_proof"
        );
        assert_eq!(
            gtk_cards["approvalCardResults"][key]["productionDisabledProof"],
            "live_gtk_atspi_proof"
        );
        assert_eq!(
            gtk_cards["approvalCardResults"][key]["disabledActionProof"],
            "live_gtk_atspi_proof"
        );
    }
    for key in ["sourceIncludeInsertion", "duplicateReplacement"] {
        assert_eq!(
            gtk_cards["activationDecisionResults"][key]["headingProof"],
            "live_gtk_atspi_proof"
        );
        assert_eq!(
            gtk_cards["activationDecisionResults"][key]["productionDisabledProof"],
            "live_gtk_atspi_proof"
        );
        assert_eq!(
            gtk_cards["activationDecisionResults"][key]["disabledActionProof"],
            "live_gtk_atspi_proof"
        );
    }
    for key in ["sourceIncludeInsertion", "duplicateReplacement"] {
        assert_eq!(
            gtk_cards["activationPathResults"][key]["headingProof"],
            "live_gtk_atspi_proof"
        );
        assert_eq!(
            gtk_cards["activationPathResults"][key]["productionDisabledProof"],
            "live_gtk_atspi_proof"
        );
        assert_eq!(
            gtk_cards["activationPathResults"][key]["disabledActionProof"],
            "live_gtk_atspi_proof"
        );
    }
    for key in ["sourceIncludeInsertion", "duplicateReplacement"] {
        assert_eq!(
            gtk_cards["activationControlResults"][key]["headingProof"],
            "live_gtk_atspi_proof"
        );
        assert_eq!(
            gtk_cards["activationControlResults"][key]["productionDisabledProof"],
            "live_gtk_atspi_proof"
        );
        assert_eq!(
            gtk_cards["activationControlResults"][key]["executorWiringProof"],
            "live_gtk_atspi_proof"
        );
        assert_eq!(
            gtk_cards["activationControlResults"][key]["disabledActionProof"],
            "live_gtk_atspi_proof"
        );
    }
    for key in ["sourceIncludeInsertion", "duplicateReplacement"] {
        assert_eq!(
            gtk_cards["activationFormResults"][key]["headingProof"],
            "live_gtk_atspi_proof"
        );
        assert_eq!(
            gtk_cards["activationFormResults"][key]["productionDisabledProof"],
            "live_gtk_atspi_proof"
        );
        assert_eq!(
            gtk_cards["activationFormResults"][key]["executorWiringProof"],
            "live_gtk_atspi_proof"
        );
        assert_eq!(
            gtk_cards["activationFormResults"][key]["disabledActionProof"],
            "live_gtk_atspi_proof"
        );
    }
    assert_eq!(gtk_cards["safety"]["runtimeMutated"], false);
    assert_eq!(gtk_cards["safety"]["hyprlandReloaded"], false);
}
