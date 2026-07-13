//! Pins the release decision: a decision report exists, its hard safety
//! boundaries are recorded as respected, and no release action was taken
//! from this branch.

use std::fs;

const REPORT: &str = "data/reports/release-decision.v0.55.2.json";
const DOC: &str = "docs/RELEASE-DECISION.md";

#[test]
fn release_decision_report_exists_with_safe_values() {
    let report: serde_json::Value =
        serde_json::from_str(&fs::read_to_string(REPORT).expect("report reads"))
            .expect("report parses");
    assert_eq!(
        report["artifactKind"].as_str(),
        Some("release-decision-report")
    );
    // The decision was resolved by user approval: v0.2.0 released.
    assert_eq!(
        report["releaseDecisionStatus"].as_str(),
        Some("approved and released as v0.2.0")
    );
    assert_eq!(report["resolution"]["userApproved"].as_bool(), Some(true));
    assert!(report["resolution"]["manualRcTest"]
        .as_str()
        .unwrap_or("")
        .starts_with("passed"));
    assert_eq!(report["releaseTagCreated"].as_bool(), Some(true));
    assert_eq!(report["releaseArtifactsPublished"].as_bool(), Some(true));
    assert_eq!(report["mergedToMain"].as_bool(), Some(true));
    // The original decision event took no release action (historical
    // block), and the v0.1.0 release stayed untouched throughout.
    let boundaries = &report["hardBoundariesRespected"];
    for key in [
        "noTagCreated",
        "noGithubReleaseCreated",
        "noMergeToMain",
        "noArtifactsPublished",
        "v010Unchanged",
        "distV010Unchanged",
    ] {
        assert_eq!(boundaries[key].as_bool(), Some(true), "{key} must be true");
    }
    assert!(report["hardBoundariesNote"]
        .as_str()
        .unwrap_or("")
        .contains("original decision event"));
    // The existing release is referenced, not modified.
    assert_eq!(report["existingRelease"]["tag"].as_str(), Some("v0.1.0"));
    assert_eq!(
        report["existingRelease"]["unchangedByThisDecision"].as_bool(),
        Some(true)
    );
    // Blockers are recorded: the release needs the user.
    let blockers = report["remainingBlockersForRelease"]
        .as_array()
        .expect("blockers");
    assert!(!blockers.is_empty());
    assert!(blockers
        .iter()
        .any(|blocker| blocker.as_str().unwrap_or("").contains("user approval")));
}

#[test]
fn release_candidate_materials_exist_without_release_automation() {
    let doc = fs::read_to_string(DOC).expect("doc reads");
    for section in [
        "## Release notes draft",
        "## Changelog draft",
        "## Artifact checklist",
        "## Test checklist",
        "## Manual test plan",
        "## Known limitations",
    ] {
        assert!(doc.contains(section), "doc must contain {section}");
    }
    // The doc records the original decision and its resolution.
    assert!(doc.contains("Ready pending user approval"));
    assert!(doc.contains("No tag was created"));
    assert!(doc.contains("## Resolution (2026-07-13)"));
    assert!(doc.contains("Approved and released as v0.2.0"));

    // No release automation exists in the repo that tags or publishes:
    // the only release-related script material is the documented manual
    // checklist above.
    for script in fs::read_dir("tools").expect("tools dir") {
        let path = script.expect("entry").path();
        if path.is_file() {
            let contents = fs::read_to_string(&path).unwrap_or_default();
            assert!(
                !contents.contains("git tag ") && !contents.contains("gh release create"),
                "{} must not tag or publish releases",
                path.display()
            );
        }
    }
}
