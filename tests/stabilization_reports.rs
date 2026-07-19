use std::fs;

use anyhow::Result;
use serde_json::Value;

fn read_json(path: &str) -> Result<Value> {
    Ok(serde_json::from_str(&fs::read_to_string(path)?)?)
}

#[test]
fn required_stabilization_reports_record_fail_closed_state() -> Result<()> {
    let transaction = read_json("data/reports/save-state-transaction-stabilization.v0.55.2.json")?;
    assert_eq!(transaction["markSavedAfterDurableSuccess"], true);
    assert_eq!(transaction["failedSavePendingStatePreserved"], true);
    assert_eq!(transaction["failedSaveRuntimeRecoveryPreserved"], true);
    assert_eq!(transaction["partialCommitPossible"], false);

    let write = read_json("data/reports/write-drift-atomic-backup-hardening.v0.55.2.json")?;
    for key in [
        "scalarReplacementDriftProtected",
        "missingInsertionDriftProtected",
        "structuredFamilyDriftProtected",
        "sourceGraphDriftProtected",
        "sameDirectoryExclusiveTemp",
        "targetIdentityRevalidated",
        "commitBoundaryRaceProtected",
        "exclusiveBackupCreation",
        "automaticRestoreOnVerificationFailure",
    ] {
        assert_eq!(write[key], true, "{key} must remain proven");
    }
    assert_eq!(write["backupDirectoryMode"], "0700");
    assert_eq!(write["backupFileMode"], "0600");

    let hermetic = read_json("data/reports/hermetic-test-stabilization.v0.55.2.json")?;
    for key in [
        "normalTestsReadRealConfig",
        "normalTestsRunHyprctl",
        "normalTestsWriteActiveConfig",
        "normalTestsMutateRuntime",
        "normalTestsRewriteTrackedReports",
    ] {
        assert_eq!(hermetic[key], false, "{key} must remain false");
    }
    Ok(())
}

#[test]
fn current_docs_distinguish_release_and_unreleased_implementation() -> Result<()> {
    let readme = fs::read_to_string("README.md")?;
    for required in [
        "`v0.2.0` is published",
        "unreleased post-v0.2.0 work",
        "290 scalar rows exposed as editable",
        "51 high-risk scalar rows deliberately blocked",
        "`hl.animation` and `hl.curve`",
        "Safe Live Save Mode",
        "guarded, reversible runtime mutation",
        "never runs `hyprctl reload`",
        "Save all atomically",
    ] {
        assert!(readme.contains(required), "README missing: {required}");
    }
    for stale in [
        "341 writable\n- 0 blocked",
        "Packaging and release artifacts are not finalized",
        "Structured-family writes and profile/mode switching are intentionally blocked",
        "does not switch profiles, change symlinks, reload Hyprland, or run mutating `hyprctl`",
    ] {
        assert!(
            !readme.contains(stale),
            "README retained stale claim: {stale}"
        );
    }

    let handoff = fs::read_to_string("docs/CURRENT-PROJECT-HANDOFF.md")?;
    assert!(handoff.contains("## Stabilization Outcome"));
    assert!(handoff.contains("Multi-file batches reject before writing"));
    assert!(!handoff.contains("Save now iterates pending rows"));
    assert!(!handoff.contains("partial failures reported"));

    let status = fs::read_to_string("docs/PROJECT-STATUS.md")?;
    assert!(status.contains("published `v0.2.0` release"));
    assert!(status.contains("ACLs, extended attributes, and timestamps"));
    Ok(())
}
