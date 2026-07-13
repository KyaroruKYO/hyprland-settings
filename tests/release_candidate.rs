//! Pins the v0.2.0-rc.1 release-candidate state: the candidate is prepared
//! locally, no tag was created, nothing was published, main is untouched,
//! and v0.1.0 / dist/v0.1.0 are unchanged (checksum-verified). Read-only:
//! this test never builds artifacts, never tags, and never mutates
//! anything.

use std::fs;
use std::process::Command;

const REPORT: &str = "data/reports/release-candidate-v0.2.0-rc.1.v0.55.2.json";
const MANIFEST: &str = "data/reports/v0.2.0-rc.1-release-artifact-manifest.json";
const DOC: &str = "docs/RELEASE-CANDIDATE-v0.2.0-rc.1.md";

#[test]
fn release_candidate_report_records_prepared_state_without_release_actions() {
    let report: serde_json::Value =
        serde_json::from_str(&fs::read_to_string(REPORT).expect("report reads"))
            .expect("report parses");
    assert_eq!(
        report["artifactKind"].as_str(),
        Some("release-candidate-report")
    );
    assert_eq!(report["releaseCandidate"].as_str(), Some("v0.2.0-rc.1"));
    // The candidate takes no release action.
    assert_eq!(report["releaseTagCreated"].as_bool(), Some(false));
    assert_eq!(report["releaseTagPushed"].as_bool(), Some(false));
    assert_eq!(report["finalReleaseTagCreated"].as_bool(), Some(false));
    assert_eq!(report["releaseArtifactsPublished"].as_bool(), Some(false));
    assert_eq!(report["mainChanged"].as_bool(), Some(false));
    assert_eq!(report["v010Changed"].as_bool(), Some(false));
    assert_eq!(report["distV010Changed"].as_bool(), Some(false));
    // Artifacts are local-only, built by the guarded builder.
    assert_eq!(report["releaseArtifactsCreated"].as_bool(), Some(true));
    assert_eq!(report["artifactPath"].as_str(), Some("dist/v0.2.0-rc.1/"));
    // The remaining steps stay the user's.
    let remaining = report["remainingBeforeFinalRelease"]
        .as_array()
        .expect("remaining steps");
    assert!(remaining
        .iter()
        .any(|step| step.as_str().unwrap_or("").contains("user approval")));

    let manifest: serde_json::Value =
        serde_json::from_str(&fs::read_to_string(MANIFEST).expect("manifest reads"))
            .expect("manifest parses");
    assert_eq!(manifest["tagCreated"].as_bool(), Some(false));
    assert_eq!(manifest["finalReleaseTagCreated"].as_bool(), Some(false));
    assert_eq!(manifest["publicReleasePublished"].as_bool(), Some(false));
    assert_eq!(manifest["packageArtifactPublished"].as_bool(), Some(false));
    assert_eq!(manifest["distV010Untouched"].as_bool(), Some(true));
}

#[test]
fn release_candidate_version_metadata_is_consistent() {
    let cargo_toml = fs::read_to_string("Cargo.toml").expect("Cargo.toml reads");
    assert!(
        cargo_toml.contains("version = \"0.2.0-rc.1\""),
        "Cargo.toml carries the RC version"
    );
    let cargo_lock = fs::read_to_string("Cargo.lock").expect("Cargo.lock reads");
    assert!(cargo_lock.contains("version = \"0.2.0-rc.1\""));

    let changelog = fs::read_to_string("CHANGELOG.md").expect("changelog reads");
    assert!(changelog.contains("## 0.2.0-rc.1"));
    assert!(changelog.contains("## 0.1.0"));

    let metainfo =
        fs::read_to_string("data/metainfo/io.github.kyarorukyo.hyprlandsettings.metainfo.xml")
            .expect("metainfo reads");
    assert!(metainfo.contains("<release type=\"development\" version=\"0.2.0~rc.1\""));
    assert!(metainfo.contains("<release version=\"0.1.0\""));

    let doc = fs::read_to_string(DOC).expect("doc reads");
    for section in [
        "## Why no RC tag was created",
        "## Completed RC checklist",
        "## Remaining before final v0.2.0",
        "## Manual test plan",
        "## Known limitations",
        "## Upgrade notes",
        "## Rollback notes",
        "## Artifact manifest",
    ] {
        assert!(doc.contains(section), "doc must contain {section}");
    }
    assert!(fs::read_to_string("docs/releases/v0.2.0-rc.1.md")
        .expect("release notes read")
        .contains("release candidate"));
}

#[test]
fn release_candidate_builder_is_build_only() {
    let builder = fs::read_to_string("tools/build_release_candidate_artifacts.sh")
        .expect("builder script reads");
    // Build-only: the builder never tags, pushes, publishes, or touches the
    // compositor. ("git tag " / "gh release create" absence over all of
    // tools/ is additionally pinned by tests/release_decision.rs.)
    for forbidden in ["git push", "gh release", "gh api", "hyprctl"] {
        assert!(
            !builder.contains(forbidden),
            "builder must not contain {forbidden}"
        );
    }
    assert!(builder.contains("refusing: Cargo.toml version is"));
    assert!(builder.contains("already exists; remove it manually"));
    assert!(builder.contains("sha256sum --check --quiet dist/v0.1.0/SHA256SUMS"));

    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;
        let mode = fs::metadata("tools/build_release_candidate_artifacts.sh")
            .expect("metadata")
            .permissions()
            .mode();
        assert_ne!(mode & 0o111, 0, "builder script is executable");
    }
}

#[test]
fn protected_release_state_is_unchanged() {
    // dist/v0.1.0 checksums still match their recorded values.
    if std::path::Path::new("dist/v0.1.0/SHA256SUMS").exists() {
        let output = Command::new("sha256sum")
            .args(["--check", "--quiet", "dist/v0.1.0/SHA256SUMS"])
            .output()
            .expect("sha256sum runs");
        assert!(
            output.status.success(),
            "dist/v0.1.0 checksums must still match: {}",
            String::from_utf8_lossy(&output.stderr)
        );
    }

    // dist/v0.2.0-rc.1 artifacts, when present, match their checksums.
    if std::path::Path::new("dist/v0.2.0-rc.1/SHA256SUMS").exists() {
        let output = Command::new("sha256sum")
            .args(["--check", "--quiet", "dist/v0.2.0-rc.1/SHA256SUMS"])
            .output()
            .expect("sha256sum runs");
        assert!(
            output.status.success(),
            "dist/v0.2.0-rc.1 checksums must match: {}",
            String::from_utf8_lossy(&output.stderr)
        );
    }

    // No final v0.2.0 tag exists (read-only listing).
    let output = Command::new("git")
        .args(["tag", "-l", "v0.2.0"])
        .output()
        .expect("git runs");
    assert!(
        String::from_utf8_lossy(&output.stdout).trim().is_empty(),
        "the final v0.2.0 tag must not exist without user approval"
    );

    // The v0.1.0 tag still points at its recorded commit.
    let output = Command::new("git")
        .args(["rev-parse", "v0.1.0^{commit}"])
        .output()
        .expect("git runs");
    assert_eq!(
        String::from_utf8_lossy(&output.stdout).trim(),
        "74efa283a3d7259b8e15b1c93cad43942d9f9a1a",
        "v0.1.0 must stay on its recorded commit"
    );
}
