use std::fs;
use std::path::{Path, PathBuf};
use std::time::{SystemTime, UNIX_EPOCH};

use hyprland_settings::config_parser::parse_hyprland_config_file;
use hyprland_settings::current_config::CurrentConfigSnapshot;
use hyprland_settings::structured_family::{
    prove_fixture_parse_render_reread, render_structured_family_projection,
    structured_family_kind_from_id, StructuredFamilyKind, StructuredFamilyStatus,
};

const FIXTURE_DIR: &str = "tests/fixtures/structured_families";

#[test]
fn all_structured_family_fixtures_parse_into_review_only_projections() {
    for family in StructuredFamilyKind::ALL {
        let snapshot = snapshot_for_family(family);
        let projections = snapshot.structured_family_projections();
        let projection = projections
            .iter()
            .find(|projection| projection.family == family)
            .expect("projection should exist for every family");

        assert_eq!(projection.family_id, family.family_id());
        assert_eq!(
            projection.projection_status,
            StructuredFamilyStatus::ReadOnlyProjectionReady
        );
        assert_eq!(
            projection.editor_status,
            StructuredFamilyStatus::EditorScaffoldReady
        );
        assert_eq!(
            projection.fixture_parse_proof_status,
            StructuredFamilyStatus::FixtureParseProofReady
        );
        assert_eq!(
            projection.fixture_render_proof_status,
            StructuredFamilyStatus::FixtureRenderProofReady
        );
        assert_eq!(
            projection.write_status,
            StructuredFamilyStatus::WritesBlockedByDefault
        );
        assert!(!projection.records.is_empty());
        assert!(
            projection.unproven_record_count() > 0,
            "fixture should retain one not-proven raw example for {}",
            family.family_id()
        );
        assert!(projection
            .records
            .iter()
            .any(|record| record.validation_status == "not proven yet"));
    }
}

#[test]
fn all_structured_family_fixtures_render_and_reread_without_real_config_or_runtime() {
    for family in StructuredFamilyKind::ALL {
        let snapshot = snapshot_for_family(family);
        let original = snapshot
            .structured_family_projections()
            .into_iter()
            .find(|projection| projection.family == family)
            .expect("original projection should exist");

        let rendered = render_structured_family_projection(&original);
        let output_path = temp_output_path(family);
        fs::write(&output_path, rendered).expect("temp fixture render should write");
        let reread = CurrentConfigSnapshot::from_parsed(
            parse_hyprland_config_file(&output_path).expect("rendered fixture should parse"),
        )
        .structured_family_projections()
        .into_iter()
        .find(|projection| projection.family == family)
        .expect("reread projection should exist");

        let proof = prove_fixture_parse_render_reread(&original, &reread);
        assert_eq!(
            proof.parse_status,
            StructuredFamilyStatus::FixtureParseProofReady
        );
        assert_eq!(
            proof.render_status,
            StructuredFamilyStatus::FixtureRenderProofReady
        );
        assert!(proof.family_identity_preserved);
        assert_eq!(proof.original_record_count, proof.rendered_record_count);
        assert!(!proof.real_config_touched);
        assert!(!proof.runtime_mutated);
        assert!(!proof.hyprctl_reload_run);

        fs::remove_file(output_path).expect("temp fixture render should clean up");
    }
}

#[test]
fn structured_family_kinds_cover_required_ids_and_widget_names() {
    let required = [
        (
            "hl.monitor",
            "hyprland-settings-structured-family-hl-monitor-card",
        ),
        (
            "hl.bind",
            "hyprland-settings-structured-family-hl-bind-card",
        ),
        (
            "hl.animation",
            "hyprland-settings-structured-family-hl-animation-card",
        ),
        (
            "hl.curve",
            "hyprland-settings-structured-family-hl-curve-card",
        ),
        (
            "hl.gesture",
            "hyprland-settings-structured-family-hl-gesture-card",
        ),
        (
            "hl.device",
            "hyprland-settings-structured-family-hl-device-card",
        ),
        (
            "hl.permission",
            "hyprland-settings-structured-family-hl-permission-card",
        ),
    ];

    for (family_id, widget_name) in required {
        let family = structured_family_kind_from_id(family_id).expect("family id should map");
        assert_eq!(family.family_id(), family_id);
        assert_eq!(family.card_widget_name(), widget_name);
        assert_eq!(
            family.review_button_label().starts_with("Review "),
            true,
            "review controls are review-only labels"
        );
    }
}

#[test]
fn structured_family_ui_source_exposes_all_cards_without_write_handlers() {
    let window = fs::read_to_string("src/ui/window.rs").expect("window source should read");
    let model = fs::read_to_string("src/ui/model.rs").expect("model source should read");
    let structured_family = fs::read_to_string("src/structured_family.rs")
        .expect("structured family source should read");

    assert!(window.contains("hyprland-settings-structured-family-section"));
    for widget_name in [
        "hyprland-settings-structured-family-hl-monitor-card",
        "hyprland-settings-structured-family-hl-bind-card",
        "hyprland-settings-structured-family-hl-animation-card",
        "hyprland-settings-structured-family-hl-curve-card",
        "hyprland-settings-structured-family-hl-gesture-card",
        "hyprland-settings-structured-family-hl-device-card",
        "hyprland-settings-structured-family-hl-permission-card",
    ] {
        assert!(
            model.contains(widget_name)
                || window.contains(widget_name)
                || structured_family.contains(widget_name)
        );
    }
    for copy in [
        "Structured family editors",
        "These editors are available as review-only projections.",
        "Writes are blocked by default.",
        "Real config writes are not active.",
        "Structured family editor projection cannot write real config.",
        "Structured family editor projection cannot reload Hyprland.",
        "Structured family editor projection cannot mutate runtime.",
    ] {
        assert!(window.contains(copy));
    }

    let section_start = window
        .find("fn structured_family_editor_section")
        .expect("structured family section should exist");
    let section_end = window[section_start..]
        .find("fn disabled_future_approval_cards_section")
        .map(|offset| section_start + offset)
        .expect("section should end before future approval cards");
    let section = &window[section_start..section_end];
    assert!(!section.contains("connect_clicked"));
    assert!(!section.contains("apply_setting_change"));
    assert!(!section.contains("hyprctl"));
    assert!(!section.contains("Command::"));
}

#[test]
fn structured_family_reports_and_continuation_scan_exist() {
    for path in [
        "data/reports/structured-family-editors-unified.v0.55.2.json",
        "data/reports/project-area-continuation-scan.v0.55.2.json",
        "data/reports/current-project-handoff.v0.55.2.json",
    ] {
        assert!(
            Path::new(path).exists(),
            "expected report artifact to exist: {path}"
        );
    }
}

#[test]
fn project_area_continuation_scan_classifies_every_required_area() {
    let report: serde_json::Value = serde_json::from_str(
        &fs::read_to_string("data/reports/project-area-continuation-scan.v0.55.2.json")
            .expect("scan report should read"),
    )
    .expect("scan report should be valid JSON");
    let areas = report["areas"]
        .as_array()
        .expect("areas should be an array");

    for required in [
        "Core app shell / UI / navigation",
        "Config discovery / source-aware model",
        "341-row read/write model",
        "Safe normal-scalar writes",
        "Release packaging/tag/artifacts",
        "Missing/default insertion",
        "Duplicate resolution",
        "High-risk/display recovery",
        "Structured-family editors/writes",
        "Profile/mode switching",
        "Runtime/reload integration",
        "Hyprland 0.55.4 migration",
    ] {
        assert!(
            areas
                .iter()
                .any(|area| area["areaName"].as_str() == Some(required)),
            "missing continuation scan area {required}"
        );
    }

    let structured = areas
        .iter()
        .find(|area| area["areaName"].as_str() == Some("Structured-family editors/writes"))
        .expect("structured-family area should exist");
    assert_eq!(structured["classification"], "can_continue_now");
    assert_eq!(structured["canContinueNow"], true);
    assert!(structured["mustNotDo"]
        .as_str()
        .expect("mustNotDo should be text")
        .contains("do not enable real structured-family writes"));

    let missing = areas
        .iter()
        .find(|area| area["areaName"].as_str() == Some("Missing/default insertion"))
        .expect("missing/default area should exist");
    assert_eq!(missing["classification"], "capped");
    assert_eq!(missing["canContinueNow"], false);

    let duplicate = areas
        .iter()
        .find(|area| area["areaName"].as_str() == Some("Duplicate resolution"))
        .expect("duplicate area should exist");
    assert_eq!(duplicate["classification"], "capped");
    assert_eq!(duplicate["canContinueNow"], false);

    let handoff: serde_json::Value = serde_json::from_str(
        &fs::read_to_string("data/reports/current-project-handoff.v0.55.2.json")
            .expect("current handoff should read"),
    )
    .expect("current handoff should be valid JSON");
    assert_eq!(
        handoff["activeNextWork"],
        "Add family-specific temp-fixture write plans and validators for structured-family editors while keeping production writes blocked by default."
    );
    assert_eq!(
        handoff["safetyBoundaries"]["structuredFamilyWritesEnabled"],
        false
    );
}

fn snapshot_for_family(family: StructuredFamilyKind) -> CurrentConfigSnapshot {
    CurrentConfigSnapshot::from_parsed(
        parse_hyprland_config_file(fixture_path(family)).expect("fixture should parse"),
    )
}

fn fixture_path(family: StructuredFamilyKind) -> PathBuf {
    let name = match family {
        StructuredFamilyKind::Monitor => "hl_monitor.conf",
        StructuredFamilyKind::Bind => "hl_bind.conf",
        StructuredFamilyKind::Animation => "hl_animation.conf",
        StructuredFamilyKind::Curve => "hl_curve.conf",
        StructuredFamilyKind::Gesture => "hl_gesture.conf",
        StructuredFamilyKind::Device => "hl_device.conf",
        StructuredFamilyKind::Permission => "hl_permission.conf",
    };
    Path::new(FIXTURE_DIR).join(name)
}

fn temp_output_path(family: StructuredFamilyKind) -> PathBuf {
    let nanos = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .expect("clock should be after epoch")
        .as_nanos();
    std::env::temp_dir().join(format!(
        "hyprland-settings-structured-family-{}-{nanos}.conf",
        family.family_id().replace('.', "-")
    ))
}
