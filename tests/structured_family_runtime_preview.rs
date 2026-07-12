use std::fs;

use hyprland_settings::structured_family_runtime_preview::{
    parse_animation_leaf, parse_bezier_points, proven_family_record_proof,
    structured_family_runtime_preview_profile, structured_family_runtime_preview_profiles,
    structured_family_runtime_preview_summary, StructuredFamilyRuntimePreviewCapability,
    PROVEN_FAMILY_RECORD_PROOFS,
};

const CAPABILITY_REPORT: &str =
    "data/reports/structured-family-runtime-preview-capability.v0.55.2.json";
const PROOF_STATUS_REPORT: &str =
    "data/reports/structured-family-runtime-preview-proof-status.v0.55.2.json";

#[test]
fn all_seven_families_are_classified_with_full_strategies() {
    let profiles = structured_family_runtime_preview_profiles();
    assert_eq!(profiles.len(), 7);
    for family_id in [
        "hl.monitor",
        "hl.bind",
        "hl.animation",
        "hl.curve",
        "hl.gesture",
        "hl.device",
        "hl.permission",
    ] {
        let profile = structured_family_runtime_preview_profile(family_id)
            .unwrap_or_else(|| panic!("missing profile for {family_id}"));
        assert!(!profile.runtime_command_strategy.is_empty());
        assert!(!profile.original_capture_strategy.is_empty());
        assert!(!profile.revert_strategy.is_empty());
        assert!(!profile.verification_strategy.is_empty());
        assert!(!profile.ui_status.is_empty());
        assert!(!profile.next_proof_needed.is_empty());
        assert!(!profile.evidence.is_empty());
        assert!(profile.dead_man_required, "every family stays supervised");
        if !profile.capability.live_previewable() {
            assert!(
                profile.blocked_reason.is_some()
                    || profile.capability == StructuredFamilyRuntimePreviewCapability::NotProvenYet,
                "{family_id} needs a blocked reason or not-proven status"
            );
        }
    }

    let summary = structured_family_runtime_preview_summary();
    assert_eq!(summary.families_total, 7);
    assert_eq!(summary.families_classified, 7);
    assert_eq!(
        summary.live_preview_supported_with_dead_man, 2,
        "hl.animation and hl.curve are proven for modify-existing supervised preview"
    );
    assert_eq!(summary.blocked_high_risk, 4);
    assert_eq!(summary.blocked_no_verification_mechanism, 1);
    assert_eq!(summary.not_proven_yet, 0);
}

#[test]
fn family_promotion_requires_a_recorded_proof_receipt() {
    for profile in structured_family_runtime_preview_profiles() {
        if profile.capability.live_previewable() {
            let proof = proven_family_record_proof(profile.family_id)
                .unwrap_or_else(|| panic!("{} armed without receipt", profile.family_id));
            assert!(!proof.record.is_empty());
            assert!(!proof.original.is_empty());
            assert!(!proof.preview.is_empty());
            assert!(proof.verification.contains("zero residue"));
            assert!(
                profile.scope.contains("modify-existing"),
                "{} must be scoped to modify-existing records",
                profile.family_id
            );
        }
    }
    assert_eq!(PROVEN_FAMILY_RECORD_PROOFS.len(), 2);

    // The high-risk families can never be promoted by these receipts.
    for family_id in ["hl.monitor", "hl.bind", "hl.device", "hl.permission"] {
        let profile = structured_family_runtime_preview_profile(family_id).expect("profile");
        assert_eq!(
            profile.capability,
            StructuredFamilyRuntimePreviewCapability::BlockedHighRisk,
            "{family_id} stays blocked high-risk"
        );
        assert!(proven_family_record_proof(family_id).is_none());
    }
    let gesture = structured_family_runtime_preview_profile("hl.gesture").expect("profile");
    assert_eq!(
        gesture.capability,
        StructuredFamilyRuntimePreviewCapability::BlockedNoVerificationMechanism
    );
}

#[test]
fn animations_listing_parsers_read_leaves_and_curves() {
    let listing = "animations:\n\n\tname: global\n\t\toverriden: 1\n\t\tbezier: default\n\t\tenabled: 1\n\t\tspeed: 8.00\n\t\tstyle: \n\n\tname: zoomFactor\n\t\toverriden: 0\n\t\tbezier: \n\t\tenabled: -1\n\t\tspeed: 0.00\n\t\tstyle: \n\nbeziers:\n\n\tname: default\n\t\tX0: 0.00\n\t\tY0: 0.75\n\t\tX1: 0.15\n\t\tY1: 1.00\n";
    let (enabled, speed, bezier) = parse_animation_leaf(listing, "global").expect("global parses");
    assert_eq!(
        (enabled.as_str(), speed.as_str(), bezier.as_str()),
        ("1", "8.00", "default")
    );
    let (enabled, speed, bezier) =
        parse_animation_leaf(listing, "zoomFactor").expect("zoomFactor parses");
    assert_eq!(
        (enabled.as_str(), speed.as_str(), bezier.as_str()),
        ("-1", "0.00", "")
    );
    assert!(parse_animation_leaf(listing, "missing").is_none());

    let (x0, y0, x1, y1) = parse_bezier_points(listing, "default").expect("default parses");
    assert_eq!((x0, y0, x1, y1), (0.0, 0.75, 0.15, 1.0));
    assert!(parse_bezier_points(listing, "missing").is_none());
}

#[test]
fn family_preview_sources_and_harness_stay_guarded() {
    let module = fs::read_to_string("src/structured_family_runtime_preview.rs")
        .expect("module source reads");
    for forbidden in [
        "Command::new",
        "std::process",
        "fs::write",
        "File::create",
        ".config/hypr",
        "hyprctl reload",
        "write_flow::",
        "apply_setting_change(",
    ] {
        assert!(
            !module.contains(forbidden),
            "family preview module must not contain {forbidden}"
        );
    }
    let harness = fs::read_to_string("tests/structured_family_runtime_preview_live.rs")
        .expect("harness source reads");
    assert!(harness.contains("#[ignore"));
    assert!(harness.contains("HYPRLAND_SETTINGS_RUN_STRUCTURED_RUNTIME_PREVIEW_LIVE"));
    assert!(harness.contains("HYPRLAND_SETTINGS_STRUCTURED_FAMILY"));
    assert!(!harness.contains("hyprctl reload"));
    assert!(!harness.contains("\"reload\""));
    // The harness only touches records proven to exist with exact restore:
    // no monitor/bind/device/permission mutation appears anywhere.
    for forbidden in [
        "hl.monitor",
        "hl.bind",
        "hl.device",
        "hl.permission",
        "hl.gesture",
    ] {
        assert!(
            !harness.contains(forbidden),
            "harness must not touch {forbidden}"
        );
    }
    // UI code never builds family runtime commands.
    let window = fs::read_to_string("src/ui/window.rs").expect("window source reads");
    for forbidden in ["hl.animation(", "hl.curve(", "hl.config"] {
        assert!(
            !window.contains(forbidden),
            "UI must not contain {forbidden}"
        );
    }
}

#[test]
fn family_capability_reports_are_generated_and_consistent() {
    #[derive(serde::Serialize)]
    struct CapabilityReport {
        #[serde(rename = "artifactKind")]
        artifact_kind: &'static str,
        #[serde(rename = "projectDataVersion")]
        project_data_version: &'static str,
        summary:
            hyprland_settings::structured_family_runtime_preview::StructuredFamilyRuntimePreviewSummary,
        profiles: Vec<
            hyprland_settings::structured_family_runtime_preview::StructuredFamilyRuntimePreviewProfile,
        >,
    }
    let report = CapabilityReport {
        artifact_kind: "structured-family-runtime-preview-capability",
        project_data_version: "v0.55.2",
        summary: structured_family_runtime_preview_summary(),
        profiles: structured_family_runtime_preview_profiles(),
    };
    let mut rendered = serde_json::to_string_pretty(&report).expect("serializes");
    rendered.push('\n');
    fs::write(CAPABILITY_REPORT, &rendered).expect("capability report writes");
    let parsed: serde_json::Value = serde_json::from_str(&rendered).expect("parses");
    assert_eq!(parsed["summary"]["families_classified"], 7);
    assert_eq!(parsed["profiles"].as_array().expect("array").len(), 7);

    #[derive(serde::Serialize)]
    struct ProofStatusReport {
        #[serde(rename = "artifactKind")]
        artifact_kind: &'static str,
        #[serde(rename = "projectDataVersion")]
        project_data_version: &'static str,
        #[serde(rename = "runtimeLiveProofsRun")]
        runtime_live_proofs_run: usize,
        #[serde(rename = "runtimeLiveProofsPassed")]
        runtime_live_proofs_passed: usize,
        #[serde(rename = "runtimeLiveProofsFailed")]
        runtime_live_proofs_failed: usize,
        #[serde(rename = "provenFamilyRecordProofs")]
        proven: Vec<hyprland_settings::structured_family_runtime_preview::ProvenFamilyRecordProof>,
        #[serde(rename = "harness")]
        harness: &'static str,
    }
    let proof_report = ProofStatusReport {
        artifact_kind: "structured-family-runtime-preview-proof-status",
        project_data_version: "v0.55.2",
        runtime_live_proofs_run: PROVEN_FAMILY_RECORD_PROOFS.len(),
        runtime_live_proofs_passed: PROVEN_FAMILY_RECORD_PROOFS.len(),
        runtime_live_proofs_failed: 0,
        proven: PROVEN_FAMILY_RECORD_PROOFS.to_vec(),
        harness: "tests/structured_family_runtime_preview_live.rs (ignored, env-gated)",
    };
    let mut rendered = serde_json::to_string_pretty(&proof_report).expect("serializes");
    rendered.push('\n');
    fs::write(PROOF_STATUS_REPORT, &rendered).expect("proof status report writes");
}
