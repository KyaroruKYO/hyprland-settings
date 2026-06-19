mod support;

use hyprland_settings::safe_batch_write::{
    execute_safe_batch_write_plan, SafeBatchChangeRequest, SafeBatchEligibility,
    SafeBatchExecutionOptions, SafeBatchWriteStatus,
};
use hyprland_settings::write_classification::SAFE_WRITABLE_ROWS;
use serde_json::{json, Value};
use support::safe_batch_harness::*;

fn blocker_count(audit: &Value, key: &str) -> u64 {
    audit["blocked"][key].as_u64().unwrap_or(0)
}

fn report_validation_pending() -> Value {
    json!({
        "cargoFmt": "pending",
        "cargoFmtCheck": "pending",
        "cargoCheck": "pending",
        "cargoTest": "pending",
        "cargoBuildRelease": "pending",
        "jqReports": "pending",
        "gitDiffCheck": "pending",
        "gitStatusShort": "pending"
    })
}

#[test]
fn real_config_blocker_analysis_separates_missing_unmapped_and_redacts_local_detail() {
    let audit = real_config_readonly_audit();
    assert_eq!(audit["performed"], true);
    assert_eq!(audit["realUserConfigEdited"], false);
    assert_eq!(audit["realBackupsCreated"], false);
    assert_eq!(audit["privacy"]["pathsRedacted"], true);
    assert_eq!(audit["privacy"]["rawSourceLinesRedacted"], true);
    assert_eq!(audit["privacy"]["localScriptEvidenceRedacted"], true);
    assert_eq!(audit["rootPath"], "<hypr-config>/hyprland.conf");
    assert!(!audit.to_string().contains("/home/kyo/.config/hypr"));

    let missing = blocker_count(&audit, "blocked_missing_line");
    let duplicate = blocker_count(&audit, "blocked_duplicate_conflict");
    let high_risk = blocker_count(&audit, "blocked_high_risk");
    let display_render = blocker_count(&audit, "blocked_display_render_risk");
    let missing_subtypes = audit["missingLineSubtypes"]
        .as_object()
        .expect("missing subtypes should be reported");
    let subtype_total = missing_subtypes
        .values()
        .map(|value| value.as_u64().unwrap_or(0))
        .sum::<u64>();
    assert_eq!(subtype_total, missing);
    assert!(missing > 0);
    assert!(duplicate > 0);
    assert!(high_risk > 0);
    assert!(display_render > 0);

    let configured_but_unmapped = audit["configuredButNotMappedToSafeCurrentSource"]
        .as_array()
        .map(Vec::len)
        .unwrap_or(0);
    let truly_not_configured = audit["trulyNotConfiguredDefaultRows"]
        .as_array()
        .map(Vec::len)
        .unwrap_or(0);

    let report = json!({
        "schemaVersion": "1.0",
        "artifactKind": "safe-batch-real-config-blocker-analysis",
        "generatedAt": "2026-06-18T00:00:00-07:00",
        "startingCommit": "4249144f74e95c803e0bde3c2b4e55571aabe870",
        "eligibleSafeBatchWrites": audit["eligibleSafeBatchWrites"],
        "blockedCounts": audit["blocked"],
        "missingLineBlockers": missing,
        "duplicateConflictBlockers": duplicate,
        "highRiskBlockers": high_risk,
        "displayRenderRiskBlockers": display_render,
        "scriptManagedBlockers": blocker_count(&audit, "blocked_script_managed"),
        "symlinkManagedBlockers": blocker_count(&audit, "blocked_symlink_managed"),
        "trulyNotConfiguredRows": truly_not_configured,
        "configuredButNotMappedRows": configured_but_unmapped,
        "aliasesOrCategoryMismatchesThatCouldBeFixedInCode": audit["configuredButNotMappedToSafeCurrentSource"],
        "intentionallyBlockedRows": {
            "highRisk": high_risk,
            "displayRenderRisk": display_render,
            "duplicateConflicts": duplicate,
            "scriptSymlinkManaged": blocker_count(&audit, "blocked_script_managed") + blocker_count(&audit, "blocked_symlink_managed")
        },
        "blockerDetails": audit["blockerDetails"],
        "duplicateConflictDetails": audit["duplicateConflictDetails"],
        "managedHints": audit["managedHints"],
        "safety": {
            "realUserConfigEdited": false,
            "realBackupsCreated": false,
            "applyClicked": false,
            "hyprlandReloaded": false,
            "mutatingHyprctlUsed": false,
            "runtimeMutated": false
        }
    });
    write_report("safe-batch-real-config-readonly-audit.v0.55.2.json", &audit);
    write_report(
        "safe-batch-real-config-blocker-analysis.v0.55.2.json",
        &report,
    );
}

#[test]
fn duplicate_missing_and_managed_blockers_explain_why_apply_is_blocked() {
    let audit = real_config_readonly_audit();
    let duplicate_details = audit["duplicateConflictDetails"]
        .as_array()
        .expect("duplicate details should be present");
    let blur_enabled = duplicate_details
        .iter()
        .find(|detail| detail["rowId"] == "appearance.blur.enabled")
        .expect("appearance blur enabled duplicate should be reported");
    assert!(
        blur_enabled["occurrences"]
            .as_array()
            .expect("occurrences should be reported")
            .len()
            >= 2
    );
    assert!(blur_enabled["whyApplyIsBlocked"]
        .as_str()
        .unwrap()
        .contains("will not silently choose one target"));
    assert!(blur_enabled["manualResolution"]
        .as_str()
        .unwrap()
        .contains("Remove or consolidate"));

    let (_root, duplicate_plan) = blocked_plan_for(SafeBatchEligibility::BlockedDuplicateConflict);
    assert!(!duplicate_plan.can_execute);
    assert!(duplicate_plan
        .blocked_changes
        .iter()
        .any(|change| change.user_facing_copy.contains("Resolve the duplicate")));
    let duplicate_report =
        execute_safe_batch_write_plan(&duplicate_plan, &SafeBatchExecutionOptions::default());
    assert_eq!(duplicate_report.status, SafeBatchWriteStatus::Blocked);
    assert!(duplicate_report.backups.is_empty());

    let (_root, missing_plan) = blocked_plan_for(SafeBatchEligibility::BlockedMissingLine);
    assert!(!missing_plan.can_execute);
    assert!(missing_plan.blocked_changes.iter().any(|change| change
        .user_facing_copy
        .contains("not safe for automatic insertion")));

    for category in [
        SafeBatchEligibility::BlockedGeneratedFile,
        SafeBatchEligibility::BlockedScriptManaged,
        SafeBatchEligibility::BlockedSymlinkManaged,
        SafeBatchEligibility::BlockedHighRisk,
        SafeBatchEligibility::BlockedDisplayRenderRisk,
        SafeBatchEligibility::BlockedStructuredFamily,
    ] {
        let (_root, plan) = blocked_plan_for(category);
        assert!(
            !plan.can_execute,
            "{} must remain blocked",
            category.label()
        );
        assert!(plan
            .blocked_changes
            .iter()
            .any(|change| change.reason == category));
        let report = execute_safe_batch_write_plan(&plan, &SafeBatchExecutionOptions::default());
        assert_eq!(report.status, SafeBatchWriteStatus::Blocked);
        assert!(report.backups.is_empty());
    }
}

#[test]
fn mixed_safe_and_blocked_batch_still_does_not_partially_apply() {
    let rows = harness_rows();
    let (_root, current, graph, mut changes, _executed_rows) =
        build_fixture_for_rows(&rows, "mixed-blocked-hardening");
    changes.truncate(1);
    changes.push(SafeBatchChangeRequest::new("cursor.invisible", "1"));
    let plan = plan_for("mixed-blocked-hardening", &current, &graph, changes);
    assert!(!plan.can_execute);
    assert_eq!(plan.eligible_changes.len(), 1);
    assert_eq!(plan.blocked_changes.len(), 1);
    assert!(plan
        .cannot_execute_reasons
        .iter()
        .any(|reason| reason.contains("partial apply is not enabled")));
    let report = execute_safe_batch_write_plan(&plan, &SafeBatchExecutionOptions::default());
    assert_eq!(report.status, SafeBatchWriteStatus::Blocked);
    assert!(report.backups.is_empty());
}

#[test]
fn ui_readonly_smoke_report_uses_source_proof_without_apply() {
    let safe_batch_copy = hyprland_settings::safe_batch_write::safe_batch_write_user_facing_lines();
    assert!(source_contains_safe_batch_ui_copy());
    assert!(safe_batch_copy
        .iter()
        .any(|line| line.contains("back up files before writing")));
    assert!(SafeBatchEligibility::BlockedDuplicateConflict
        .user_facing_blocked_copy()
        .contains("appears in more than one place"));
    assert!(SafeBatchEligibility::BlockedMissingLine
        .user_facing_blocked_copy()
        .contains("Hyprland's default value"));
    assert!(SafeBatchEligibility::BlockedScriptManaged
        .user_facing_blocked_copy()
        .contains("changed by a script"));
    assert!(SafeBatchEligibility::BlockedSymlinkManaged
        .user_facing_blocked_copy()
        .contains("symlink or current-profile"));

    let report = json!({
        "schemaVersion": "1.0",
        "artifactKind": "safe-batch-real-ui-readonly-smoke",
        "generatedAt": "2026-06-18T00:00:00-07:00",
        "startingCommit": "4249144f74e95c803e0bde3c2b4e55571aabe870",
        "liveUiSmokePerformed": false,
        "reasonLiveUiNotPerformed": "Source-level UI/accessibility wording proof was used to avoid any chance of interacting with Apply during blocker hardening.",
        "sourceLevelUiSmokePerformed": true,
        "safeBatchWriteStatusVisibleInSource": true,
        "duplicateConflictBlockedCopyVisible": SafeBatchEligibility::BlockedDuplicateConflict.user_facing_blocked_copy(),
        "missingLineDefaultBlockedCopyVisible": SafeBatchEligibility::BlockedMissingLine.user_facing_blocked_copy(),
        "scriptManagedBlockedCopyVisible": SafeBatchEligibility::BlockedScriptManaged.user_facing_blocked_copy(),
        "symlinkManagedBlockedCopyVisible": SafeBatchEligibility::BlockedSymlinkManaged.user_facing_blocked_copy(),
        "highRiskBlockedCopyVisible": SafeBatchEligibility::BlockedHighRisk.user_facing_blocked_copy(),
        "applyClicked": false,
        "writesTriggered": false,
        "oneTargetWordingInNewSafeBatchPath": false
    });
    write_report("safe-batch-real-ui-readonly-smoke.v0.55.2.json", &report);
}

#[test]
fn real_config_hardening_summary_preserves_write_safety() {
    assert_eq!(SAFE_WRITABLE_ROWS.len(), 341);
    let audit = real_config_readonly_audit();
    let before_blockers = json!({
        "blocked_display_render_risk": 26,
        "blocked_duplicate_conflict": 5,
        "blocked_high_risk": 47,
        "blocked_missing_line": 261,
        "blocked_profile_mode_switch": 1
    });
    let after_eligible = audit["eligibleSafeBatchWrites"].as_u64().unwrap_or(0);
    assert_eq!(after_eligible, 1);
    assert_eq!(audit["realUserConfigEdited"], false);
    assert_eq!(audit["realBackupsCreated"], false);

    let report = json!({
        "schemaVersion": "1.0",
        "artifactKind": "safe-batch-real-config-hardening",
        "generatedAt": "2026-06-18T00:00:00-07:00",
        "startingCommit": "4249144f74e95c803e0bde3c2b4e55571aabe870",
        "goal": "Harden real-config safe-batch eligibility and blocker explanations before broad trust.",
        "filesChanged": [
            "src/safe_batch_write.rs",
            "tests/support/safe_batch_harness.rs",
            "tests/safe_batch_real_config_hardening.rs",
            "data/reports/safe-batch-real-config-readonly-audit.v0.55.2.json",
            "data/reports/safe-batch-real-config-blocker-analysis.v0.55.2.json",
            "data/reports/safe-batch-real-ui-readonly-smoke.v0.55.2.json",
            "data/reports/safe-batch-real-config-hardening.v0.55.2.json",
            "docs/SAFE-BATCH-REAL-CONFIG-HARDENING-REVIEW-LOG.md"
        ],
        "baselineRealConfigEligibility": 1,
        "afterRealConfigEligibility": after_eligible,
        "blockerCountsBefore": before_blockers,
        "blockerCountsAfter": audit["blocked"],
        "newlyEligibleRows": [],
        "duplicateConflictImprovements": {
            "allDuplicateRowsListed": true,
            "occurrencesIncludeRedactedFileLineAndValue": true,
            "applyStillBlocked": true,
            "manualResolutionCopyAdded": true
        },
        "missingLineImprovements": {
            "defaultRowsSeparatedFromConfiguredButUnmapped": true,
            "safeInsertionStillUnsupported": true,
            "applyStillBlocked": true,
            "userFacingCopyAdded": true
        },
        "scriptSymlinkManagedImprovements": {
            "managedHintsRedacted": true,
            "scriptManagedCopyExplainsNoWrite": true,
            "symlinkManagedCopyExplainsNoWrite": true
        },
        "privacyReview": {
            "realConfigReadonlyAuditRedacted": true,
            "deepHarnessSummaryContainsNoRawLocalPaths": true,
            "rawSourceLinesOmittedFromCommittedRealConfigReports": true,
            "localScriptEvidenceRedacted": true
        },
        "uiSmokeReview": {
            "liveUiSmokePerformed": false,
            "sourceLevelUiSmokePerformed": true,
            "applyClicked": false,
            "writesTriggered": false
        },
        "testsAdded": ["tests/safe_batch_real_config_hardening.rs"],
        "testsUpdated": ["tests/support/safe_batch_harness.rs"],
        "safetyBoundaries": {
            "realUserConfigEdited": false,
            "realUserConfigBackupsCreated": false,
            "applyClicked": false,
            "hyprlandReloaded": false,
            "mutatingHyprctlUsed": false,
            "scriptsExecuted": false,
            "luaExecuted": false,
            "runtimeMutated": false,
            "highRiskWritesEnabled": false,
            "displayRenderRiskyWritesEnabled": false,
            "structuredFamilyWritesEnabled": false,
            "generatedScriptSymlinkManagedWritesEnabled": false,
            "duplicateConflictsStillBlockApply": true,
            "missingLineDefaultSettingsStillBlockApply": true
        },
        "countsBefore": {"readable": 341, "writable": 341, "blocked": 0},
        "countsAfter": {"readable": 341, "writable": 341, "blocked": 0},
        "validation": report_validation_pending(),
        "proofUsed": [
            "read-only real config audit",
            "safe-batch blocked-category execution tests",
            "source-level UI wording smoke",
            "redacted duplicate occurrence report"
        ],
        "proofStillMissing": [
            "source/include-aware current-value mapping for Apply is still not proven",
            "safe insertion for default-only settings is not designed",
            "duplicate auto-resolution is intentionally not implemented",
            "live GTK smoke was not run in this hardening sprint"
        ],
        "nextRecommendedSprint": "Design source/include-aware current-value mapping for safe-batch Apply, still without insertion or duplicate auto-resolution."
    });
    write_report("safe-batch-real-config-hardening.v0.55.2.json", &report);
}
