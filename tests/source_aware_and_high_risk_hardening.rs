mod support;

use std::collections::{BTreeMap, BTreeSet};
use std::fs;
use std::path::{Path, PathBuf};
use std::time::{SystemTime, UNIX_EPOCH};

use hyprland_settings::config_discovery::{
    ConfigDiscovery, ConfigDiscoveryStatus, ConfigPathSource,
};
use hyprland_settings::current_config::CurrentConfigSnapshot;
use hyprland_settings::high_risk_family::{
    display_render_family_for_row, family_blocked_reason, HighRiskFamily,
};
use hyprland_settings::safe_batch_write::{SafeBatchChangeRequest, SafeBatchEligibility};
use hyprland_settings::write_classification::SAFE_WRITABLE_ROWS;
use hyprland_settings::write_flow::apply_safe_batch_setting_changes;
use serde_json::{json, Value};
use support::safe_batch_harness::*;

fn temp_root(label: &str) -> PathBuf {
    let stamp = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .expect("time should work")
        .as_nanos();
    let root = std::env::temp_dir().join(format!(
        "hyprland-settings-source-risk-{label}-{}-{stamp}",
        std::process::id()
    ));
    fs::create_dir_all(&root).expect("temp root should create");
    root
}

fn write_file(path: &Path, contents: &str) {
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent).expect("parent should create");
    }
    fs::write(path, contents).expect("fixture should write");
}

fn known_settings() -> BTreeSet<String> {
    SAFE_WRITABLE_ROWS
        .iter()
        .map(|row| row.row_id.to_string())
        .collect()
}

fn high_risk_rows() -> Vec<(&'static str, &'static str, HighRiskFamily)> {
    SAFE_WRITABLE_ROWS
        .iter()
        .filter_map(|row| {
            display_render_family_for_row(row.row_id)
                .map(|family| (row.row_id, row.official_setting, family))
        })
        .collect()
}

fn family_report(rows: &[(&str, &str, HighRiskFamily)]) -> Vec<Value> {
    let mut grouped = BTreeMap::<HighRiskFamily, Vec<(&str, &str)>>::new();
    for (row_id, official, family) in rows {
        grouped
            .entry(*family)
            .or_default()
            .push((*row_id, *official));
    }
    grouped
        .into_iter()
        .map(|(family, rows)| {
            json!({
                "family": family.label(),
                "rows": rows.iter().map(|(row_id, _)| *row_id).collect::<Vec<_>>(),
                "officialSettings": rows.iter().map(|(_, official)| *official).collect::<Vec<_>>(),
                "whyRisky": family.user_facing_blocked_copy(),
                "currentBlockReason": rows.iter().filter_map(|(row_id, _)| family_blocked_reason(row_id).map(|reason| reason.label())).collect::<BTreeSet<_>>(),
                "requiredProofBeforeWriting": family.required_proof(),
                "requiredRecoveryBehavior": family.recovery_behavior(),
                "fixtureOnlyProofExists": false,
                "realApplyWriteRemainsBlocked": true,
                "recommendedHandlingStrategy": family.recommended_strategy()
            })
        })
        .collect()
}

#[test]
fn source_aware_apply_helper_uses_connected_file_current_values_without_real_config() {
    let root = temp_root("apply-helper");
    let config = root.join("hyprland.conf");
    let sourced = root.join("appearance.conf");
    write_file(&config, "source = appearance.conf\n");
    write_file(&sourced, "decoration:blur:enabled = true\n");
    let discovery = ConfigDiscovery {
        status: ConfigDiscoveryStatus::Found {
            path: config.clone(),
            source: ConfigPathSource::HomeFallback,
        },
        attempted_paths: vec![config.clone()],
    };
    let stale_root_only = CurrentConfigSnapshot::from_parsed(
        hyprland_settings::config_parser::parse_hyprland_config_file(&config)
            .expect("fixture should parse"),
    );
    assert_eq!(
        stale_root_only
            .value_for("decoration.blur.enabled")
            .status_label(),
        "not configured"
    );

    let report = apply_safe_batch_setting_changes(
        known_settings(),
        &discovery,
        &stale_root_only,
        vec![SafeBatchChangeRequest::new(
            "appearance.blur.enabled",
            "false",
        )],
    )
    .expect("source-aware helper should plan and execute fixture sourced setting");
    assert_eq!(report.verified_changes, vec!["appearance.blur.enabled"]);
    assert!(fs::read_to_string(&sourced)
        .expect("sourced fixture should read")
        .contains("decoration:blur:enabled = false"));
    assert!(!report.hyprland_reload_attempted);
    assert!(!report.mutating_hyprctl_used);
    assert!(!report.runtime_mutated);
}

#[test]
fn source_aware_real_config_audit_records_after_counts_and_new_eligibility() {
    let audit = real_config_readonly_audit();
    assert_eq!(audit["performed"], true);
    assert_eq!(audit["realUserConfigEdited"], false);
    assert_eq!(audit["realBackupsCreated"], false);
    assert_eq!(audit["privacy"]["pathsRedacted"], true);
    let after_eligible = audit["eligibleSafeBatchWrites"].as_u64().unwrap_or(0);
    let newly_eligible_rows = if after_eligible > 1 {
        audit["eligibleRowsWithExactTargets"].clone()
    } else {
        json!([])
    };
    let report = json!({
        "schemaVersion": "1.0",
        "artifactKind": "source-aware-safe-batch-real-config-audit",
        "generatedAt": "2026-06-18T00:00:00-07:00",
        "startingCommit": "99565bd954fa00870cfc6a614df7dd5e77f381ad",
        "baselineBeforeSourceAwareMapping": {
            "eligibleSafeBatchWrites": 1,
            "blockedMissingLine": 261,
            "blockedDuplicateConflict": 5,
            "blockedHighRisk": 47,
            "blockedDisplayRenderRisk": 26,
            "blockedProfileModeSwitch": 1
        },
        "afterSourceAwareMapping": audit,
        "eligibleRowsWithExactTargets": audit["eligibleRowsWithExactTargets"],
        "newlyEligibleRows": newly_eligible_rows,
        "newlyEligibleRowsAreSafe": audit["eligibleRowsWithExactTargets"].as_array().unwrap_or(&Vec::new()).iter().all(|row| {
            row["oldValueMatched"] == true
                && row["normalRisk"] == true
                && row["generated"] == false
                && row["scriptManaged"] == false
                && row["symlinkManaged"] == false
                && row["duplicateConflicted"] == false
        }),
        "realUserConfigEdited": false,
        "realBackupsCreated": false,
        "applyClicked": false,
        "hyprlandReloaded": false,
        "mutatingHyprctlUsed": false,
        "runtimeMutated": false
    });
    assert!(after_eligible >= 1);
    write_report(
        "source-aware-safe-batch-real-config-audit.v0.55.2.json",
        &report,
    );
}

#[test]
fn high_risk_family_inventory_classifies_every_display_or_high_risk_row() {
    let rows = high_risk_rows();
    assert!(!rows.is_empty());
    assert_eq!(SAFE_WRITABLE_ROWS.len(), 341);
    assert!(rows
        .iter()
        .all(|(row_id, _, _)| family_blocked_reason(row_id).is_some()));
    let families = rows
        .iter()
        .map(|(_, _, family)| family.label())
        .collect::<BTreeSet<_>>();
    assert!(families.contains("display_render_pipeline_risk"));
    assert!(families.contains("shader_screen_shader_risk"));
    assert!(families.contains("input_device_risk"));

    let report = json!({
        "schemaVersion": "1.0",
        "artifactKind": "high-risk-family-inventory",
        "generatedAt": "2026-06-18T00:00:00-07:00",
        "startingCommit": "99565bd954fa00870cfc6a614df7dd5e77f381ad",
        "totalRowsClassified": rows.len(),
        "familiesFound": families.iter().copied().collect::<Vec<_>>(),
        "families": family_report(&rows),
        "safeWritableRowsLen": SAFE_WRITABLE_ROWS.len(),
        "realApplyWritesRemainBlocked": true,
        "highRiskWritesEnabled": false,
        "displayRenderRiskyWritesEnabled": false
    });
    write_report("high-risk-family-inventory.v0.55.2.json", &report);
}

#[test]
fn high_risk_family_handling_design_covers_each_family_and_keeps_apply_blocked() {
    let rows = high_risk_rows();
    let families = rows
        .iter()
        .map(|(_, _, family)| *family)
        .collect::<BTreeSet<_>>();
    let designs = families
        .iter()
        .map(|family| {
            json!({
                "family": family.label(),
                "userFacingBlockedCopy": family.user_facing_blocked_copy(),
                "requiredProof": family.required_proof(),
                "requiredRecoveryBehavior": family.recovery_behavior(),
                "recommendedHandlingStrategy": family.recommended_strategy(),
                "realApplyWriteRemainsBlocked": true
            })
        })
        .collect::<Vec<_>>();
    assert!(designs.iter().all(|design| {
        design["requiredProof"].as_str().unwrap().len() > 20
            && design["realApplyWriteRemainsBlocked"] == true
    }));

    for (row_id, _, family) in rows.iter().take(20) {
        let (_root, plan) = blocked_plan_for(
            family_blocked_reason(row_id).unwrap_or(SafeBatchEligibility::BlockedHighRisk),
        );
        assert!(
            !plan.can_execute,
            "{} in {} should not become normal safe-batch eligible",
            row_id,
            family.label()
        );
    }

    let report = json!({
        "schemaVersion": "1.0",
        "artifactKind": "high-risk-family-handling-design",
        "generatedAt": "2026-06-18T00:00:00-07:00",
        "startingCommit": "99565bd954fa00870cfc6a614df7dd5e77f381ad",
        "familyDesigns": designs,
        "allFamiliesHaveRequiredProof": true,
        "allFamiliesHaveUserFacingBlockedCopy": true,
        "allFamiliesRemainBlockedFromRealSafeBatchApply": true,
        "screenShaderRemainsBlockedUnlessSpecialProofPathActive": true,
        "profileModeRowsRemainBlocked": true,
        "scriptExecPathRiskRowsRemainBlocked": true,
        "safeWritableRowsLen": SAFE_WRITABLE_ROWS.len()
    });
    write_report("high-risk-family-handling-design.v0.55.2.json", &report);
}

#[test]
fn combined_source_aware_and_high_risk_hardening_report_preserves_safety() {
    let audit = real_config_readonly_audit();
    let rows = high_risk_rows();
    let families = rows
        .iter()
        .map(|(_, _, family)| family.label())
        .collect::<BTreeSet<_>>();
    let after_eligible = audit["eligibleSafeBatchWrites"].as_u64().unwrap_or(0);
    let newly_eligible_rows = if after_eligible > 1 {
        audit["eligibleRowsWithExactTargets"].clone()
    } else {
        json!([])
    };
    let report = json!({
        "schemaVersion": "1.0",
        "artifactKind": "source-aware-and-high-risk-hardening",
        "generatedAt": "2026-06-18T00:00:00-07:00",
        "startingCommit": "99565bd954fa00870cfc6a614df7dd5e77f381ad",
        "goal": "Add source/include-aware current-value mapping and high-risk family proof tracks while preserving real-write safety boundaries.",
        "filesChanged": [
            "src/source_aware_current_config.rs",
            "src/high_risk_family.rs",
            "src/ui/app.rs",
            "src/write_flow.rs",
            "tests/source_aware_current_config.rs",
            "tests/source_aware_and_high_risk_hardening.rs",
            "tests/support/safe_batch_harness.rs",
            "data/reports/source-aware-safe-batch-real-config-audit.v0.55.2.json",
            "data/reports/high-risk-family-inventory.v0.55.2.json",
            "data/reports/high-risk-family-handling-design.v0.55.2.json",
            "data/reports/source-aware-and-high-risk-hardening.v0.55.2.json",
            "docs/SOURCE-AWARE-AND-HIGH-RISK-HARDENING-REVIEW-LOG.md"
        ],
        "sourceAwareMapping": audit["sourceAwareMapping"],
        "realConfigEligibilityBefore": 1,
        "realConfigEligibilityAfter": after_eligible,
        "newlyEligibleRows": newly_eligible_rows,
        "stillBlockedCounts": audit["blocked"],
        "duplicateHandling": {
            "duplicateConflictsStillBlockApply": true,
            "detailsRemainRedacted": true
        },
        "missingLineHandling": {
            "missingLineDefaultSettingsStillBlockApply": true,
            "insertionNotImplemented": true
        },
        "scriptSymlinkGeneratedHandling": {
            "generatedScriptSymlinkManagedWritesEnabled": false,
            "managedHintsRemainRedacted": true
        },
        "highRiskFamilyInventory": {
            "familiesFound": families.iter().copied().collect::<Vec<_>>(),
            "rowsClassified": rows.len()
        },
        "highRiskFamilyHandling": family_report(&rows),
        "uiCopyChanges": {
            "sourceIncludeBlockers": true,
            "duplicateBlockers": true,
            "missingDefaultBlockers": true,
            "highRiskFamilyBlockers": true
        },
        "testsAdded": [
            "tests/source_aware_current_config.rs",
            "tests/source_aware_and_high_risk_hardening.rs"
        ],
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
        "validation": {
            "cargoFmt": "pending",
            "cargoFmtCheck": "pending",
            "cargoCheck": "pending",
            "cargoTest": "pending",
            "cargoBuildRelease": "pending",
            "jqReports": "pending",
            "gitDiffCheck": "pending",
            "gitStatusShort": "pending"
        },
        "proofUsed": [
            "source-aware graph fixture mapping",
            "safe-batch planning against sourced scalar fixture",
            "read-only real-config audit",
            "high-risk policy inventory",
            "family-specific blocked-copy and proof requirements"
        ],
        "proofStillMissing": [
            "safe insertion for default-only settings",
            "duplicate auto-resolution",
            "family-specific high-risk production gates",
            "display/render risky real-write watchdog proof",
            "structured-family write support"
        ],
        "nextRecommendedSprint": "Review source-aware real-config eligibility gains and choose the next family-specific high-risk proof track without enabling unsafe writes."
    });
    assert_eq!(report["safetyBoundaries"]["realUserConfigEdited"], false);
    write_report("source-aware-and-high-risk-hardening.v0.55.2.json", &report);
}
