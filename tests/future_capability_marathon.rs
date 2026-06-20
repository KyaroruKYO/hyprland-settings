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
        assert_eq!(report["whetherRuntimeTouched"], false);
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
        "Wire source/include selected-target dry-run preview into the disabled detail UI and add a temp-fixture guarded live-test executor for non-real config paths."
    );
    assert!(handoff["recommendedNextCodexPrompt"]
        .as_str()
        .expect("prompt should be text")
        .contains("source/include selected-target dry-run preview"));
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
