//! Pins the Hyprland 0.55.4 export refresh workflow: the script stays
//! read-only against the compositor, the pinned capture is trusted and
//! intact, and the refresh report records a reproducible zero-drift
//! refresh with the migration audit rerun.

use std::fs;

const SCRIPT: &str = "tools/refresh_hyprland_descriptions_export.sh";
const REPORT: &str = "data/reports/hyprland-0.55.4-export-refresh.v0.55.2.json";
const CAPTURE: &str = "data/exports/hyprland-0.55.4/hyprctl-descriptions.v0.55.4.json";
const VERSION_FILE: &str = "data/exports/hyprland-0.55.4/hyprland-version.txt";

#[test]
fn refresh_script_exists_and_stays_read_only() {
    let script = fs::read_to_string(SCRIPT).expect("script reads");

    // The only hyprctl commands are the two read-only ones.
    for required in ["hyprctl version", "hyprctl -j descriptions"] {
        assert!(script.contains(required), "script must use {required}");
    }
    for forbidden in [
        "hyprctl reload",
        "hyprctl dispatch",
        "hyprctl keyword",
        "hyprctl setcursor",
        "hyprctl switchxkblayout",
        "hyprctl output",
        "hyprctl seterror",
        "hyprctl eval",
    ] {
        assert!(
            !script.contains(forbidden),
            "script must not contain {forbidden}"
        );
    }

    // Another live version must never overwrite the pinned capture.
    assert!(script.contains("the pinned 0.55.4 capture is preserved"));
    assert!(script.contains("handEditedRows: false"));

    // The pinned migration audit is rerun as part of the refresh.
    assert!(script.contains("cargo test --test hyprland_0554_migration_audit"));

    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;
        let mode = fs::metadata(SCRIPT).expect("metadata").permissions().mode();
        assert!(mode & 0o111 != 0, "script is executable");
    }
}

#[test]
fn pinned_capture_and_version_metadata_stay_consistent() {
    let capture: serde_json::Value =
        serde_json::from_str(&fs::read_to_string(CAPTURE).expect("capture reads"))
            .expect("capture parses");
    assert_eq!(
        capture.as_array().expect("array").len(),
        341,
        "the trusted 0.55.4 capture holds all 341 options"
    );
    let version = fs::read_to_string(VERSION_FILE).expect("version file reads");
    assert!(
        version.contains("Hyprland 0.55.4"),
        "the capture metadata names the pinned version"
    );
    assert!(
        version.contains("v0.55.4"),
        "the capture metadata names the pinned tag"
    );
}

#[test]
fn refresh_report_records_a_reproducible_zero_drift_refresh() {
    let report: serde_json::Value =
        serde_json::from_str(&fs::read_to_string(REPORT).expect("report reads"))
            .expect("report parses");
    assert_eq!(
        report["artifactKind"].as_str(),
        Some("hyprland-0554-export-refresh-report")
    );
    assert_eq!(report["projectDataVersion"].as_str(), Some("v0.55.2"));
    assert_eq!(report["captureMatchedPinnedVersion"].as_bool(), Some(true));
    assert_eq!(report["pinnedCapturePreserved"].as_bool(), Some(true));
    assert_eq!(report["capturedRowCount"].as_u64(), Some(341));
    assert_eq!(report["migrationTestPassed"].as_bool(), Some(true));
    assert_eq!(report["hyprctlReloadRan"].as_bool(), Some(false));
    assert_eq!(report["runtimeMutationRan"].as_bool(), Some(false));
    assert_eq!(report["handEditedRows"].as_bool(), Some(false));

    // The rerun refresh against the same live binary found zero drift.
    let diff = &report["diff"];
    assert_eq!(diff["previousCaptureAvailable"].as_bool(), Some(true));
    assert_eq!(diff["previousRowCount"].as_u64(), Some(341));
    assert_eq!(diff["newRowCount"].as_u64(), Some(341));
    assert_eq!(diff["rowsAdded"].as_array().map(Vec::len), Some(0));
    assert_eq!(diff["rowsRemoved"].as_array().map(Vec::len), Some(0));
    assert_eq!(diff["rowsChangedBounds"].as_array().map(Vec::len), Some(0));
}
