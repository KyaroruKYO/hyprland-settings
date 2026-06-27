use std::fs;
use std::path::{Path, PathBuf};
use std::time::{SystemTime, UNIX_EPOCH};

use hyprland_settings::config_parser::parse_hyprland_config_file;
use hyprland_settings::current_config::CurrentConfigSnapshot;
use hyprland_settings::structured_family::{
    build_structured_family_temp_write_plan, prove_fixture_parse_render_reread,
    prove_structured_family_record_draft_reset, prove_structured_family_temp_write_plan,
    render_structured_family_projection, reset_structured_family_record_draft,
    structured_family_kind_from_id, structured_family_record_drafts,
    structured_family_record_editor_forms, structured_family_render_target_allowed,
    update_structured_family_record_draft_field, validate_structured_family_projection,
    StructuredFamilyKind, StructuredFamilyRecordDraftStatus, StructuredFamilyRecordEditorStatus,
    StructuredFamilyStatus, StructuredFamilyTempWritePlanStatus, StructuredFamilyValidationStatus,
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
fn all_structured_family_validators_classify_fixture_records_conservatively() {
    for family in StructuredFamilyKind::ALL {
        let snapshot = snapshot_for_family(family);
        let projection = snapshot
            .structured_family_projections()
            .into_iter()
            .find(|projection| projection.family == family)
            .expect("projection should exist");
        let validation = validate_structured_family_projection(&projection);

        assert_eq!(validation.family, family);
        assert_eq!(
            validation.status,
            StructuredFamilyValidationStatus::NotProvenYet,
            "each fixture intentionally includes an incomplete/raw form for {}",
            family.family_id()
        );
        assert!(validation.passed_count > 0);
        assert!(validation.not_proven_count > 0);
        assert_eq!(validation.failed_count, 0);
        assert!(!validation.real_config_touched);
        assert!(!validation.runtime_mutated);
        assert!(!validation.hyprctl_reload_run);
        assert!(!validation.production_write_enabled);
        assert!(!validation.production_executor_wired);
        assert!(validation
            .issues
            .iter()
            .any(|issue| issue.status == StructuredFamilyValidationStatus::NotProvenYet));
    }
}

#[test]
fn all_structured_family_temp_write_plans_validate_render_and_reread_temp_fixtures_only() {
    for family in StructuredFamilyKind::ALL {
        let snapshot = snapshot_for_family(family);
        let projection = snapshot
            .structured_family_projections()
            .into_iter()
            .find(|projection| projection.family == family)
            .expect("projection should exist");
        let output_path = temp_output_path(family);

        let plan = build_structured_family_temp_write_plan(
            &projection,
            fixture_path(family),
            output_path.clone(),
        );
        assert_eq!(plan.family, family);
        assert_eq!(plan.records_planned, projection.record_count());
        assert_eq!(
            plan.plan_status,
            StructuredFamilyTempWritePlanStatus::Validated
        );
        assert_eq!(
            plan.path_guard_status,
            StructuredFamilyTempWritePlanStatus::Ready
        );
        assert_eq!(
            plan.validation_status,
            StructuredFamilyValidationStatus::NotProvenYet
        );
        assert!(!plan.real_config_touched);
        assert!(!plan.runtime_mutated);
        assert!(!plan.hyprctl_reload_run);
        assert!(!plan.production_write_enabled);
        assert!(!plan.production_executor_wired);

        let proof = prove_structured_family_temp_write_plan(
            &projection,
            fixture_path(family),
            &output_path,
        );
        assert_eq!(
            proof.plan_status,
            StructuredFamilyTempWritePlanStatus::Validated
        );
        assert_eq!(
            proof.render_status,
            StructuredFamilyTempWritePlanStatus::RenderedToTempFixture
        );
        assert_eq!(
            proof.reread_status,
            StructuredFamilyTempWritePlanStatus::RereadVerified
        );
        assert!(proof.family_identity_preserved);
        assert!(
            proof.record_count_preserved,
            "{}",
            proof.record_count_explanation
        );
        assert_eq!(proof.original_record_count, projection.record_count());
        assert_eq!(proof.reread_record_count, projection.record_count());
        assert!(!proof.real_config_touched);
        assert!(!proof.runtime_mutated);
        assert!(!proof.hyprctl_reload_run);
        assert!(!proof.production_write_enabled);
        assert!(!proof.production_executor_wired);

        fs::remove_file(output_path).expect("temp fixture render should clean up");
    }
}

#[test]
fn structured_family_temp_write_path_guard_rejects_real_config_targets() {
    let real_config = PathBuf::from("/home/kyo/.config/hypr/hyprland.conf");
    assert!(!structured_family_render_target_allowed(&real_config));

    let family = StructuredFamilyKind::Bind;
    let snapshot = snapshot_for_family(family);
    let projection = snapshot
        .structured_family_projections()
        .into_iter()
        .find(|projection| projection.family == family)
        .expect("projection should exist");
    let plan =
        build_structured_family_temp_write_plan(&projection, fixture_path(family), &real_config);
    assert_eq!(
        plan.path_guard_status,
        StructuredFamilyTempWritePlanStatus::BlockedFromRealConfig
    );
    assert_eq!(
        plan.plan_status,
        StructuredFamilyTempWritePlanStatus::BlockedFromRealConfig
    );

    let proof =
        prove_structured_family_temp_write_plan(&projection, fixture_path(family), &real_config);
    assert_eq!(
        proof.path_guard_status,
        StructuredFamilyTempWritePlanStatus::BlockedFromRealConfig
    );
    assert!(!proof.real_config_touched);
    assert!(!proof.runtime_mutated);
    assert!(!proof.hyprctl_reload_run);
    assert!(!proof.production_write_enabled);
    assert!(!proof.production_executor_wired);
}

#[test]
fn all_structured_family_record_editor_forms_project_fixture_records_read_only() {
    for family in StructuredFamilyKind::ALL {
        let snapshot = snapshot_for_family(family);
        let projection = snapshot
            .structured_family_projections()
            .into_iter()
            .find(|projection| projection.family == family)
            .expect("projection should exist");
        let forms = structured_family_record_editor_forms(&projection);

        assert_eq!(forms.len(), projection.record_count());
        assert!(!forms.is_empty());
        assert!(forms.iter().any(|form| {
            form.raw_fallback_status
                == StructuredFamilyRecordEditorStatus::RawFallbackRequired.as_str()
        }));

        for (index, form) in forms.iter().enumerate() {
            let record = &projection.records[index];
            assert_eq!(form.family, family);
            assert_eq!(form.record_index, index);
            assert_eq!(form.source_path, record.source_path);
            assert_eq!(form.line_number, record.line_number);
            assert_eq!(form.raw_line, record.raw_line);
            assert_eq!(form.parsed_key, record.parsed_key);
            assert_eq!(form.validation_status, record.validation_status);
            assert_eq!(form.unsupported_reason, record.unsupported_reason);
            assert!(form.fields.iter().any(|field| field.name == "raw line"));
            assert!(form.fields.iter().any(|field| field.name == "parsed key"));
            assert!(form.fields.iter().any(|field| !field.value.is_empty()));
            assert!(form.fields.iter().all(|field| !field.editable));
            assert_eq!(
                form.projection_status,
                StructuredFamilyRecordEditorStatus::ProjectionReady
            );
            assert_eq!(
                form.review_status,
                StructuredFamilyRecordEditorStatus::ReviewOnly
            );
            assert_eq!(
                form.action_policy,
                StructuredFamilyRecordEditorStatus::ActionsDisabled
            );
            assert_eq!(
                form.write_blocked_status,
                StructuredFamilyRecordEditorStatus::WritesBlockedByDefault
            );
            assert_eq!(
                form.temp_fixture_plan_status,
                StructuredFamilyTempWritePlanStatus::Validated
            );
            assert!(!form.real_config_touched);
            assert!(!form.runtime_mutated);
            assert!(!form.hyprctl_reload_run);
            assert!(!form.production_executor_wired);
        }
    }
}

#[test]
fn all_structured_family_record_drafts_track_dirty_state_and_reset_in_memory_only() {
    for family in StructuredFamilyKind::ALL {
        let snapshot = snapshot_for_family(family);
        let projection = snapshot
            .structured_family_projections()
            .into_iter()
            .find(|projection| projection.family == family)
            .expect("projection should exist");
        let forms = structured_family_record_editor_forms(&projection);
        let drafts = structured_family_record_drafts(&forms);

        assert_eq!(drafts.len(), projection.record_count());
        assert_eq!(drafts.len(), forms.len());
        assert!(!drafts.is_empty());
        assert!(drafts.iter().any(|draft| {
            draft.raw_fallback_status
                == StructuredFamilyRecordDraftStatus::RawFallbackRequired.as_str()
        }));

        for (index, draft) in drafts.iter().enumerate() {
            let form = &forms[index];
            assert_eq!(draft.family, family);
            assert_eq!(draft.record_index, form.record_index);
            assert_eq!(draft.source_path, form.source_path);
            assert_eq!(draft.line_number, form.line_number);
            assert_eq!(draft.raw_original_line, form.raw_line);
            assert_eq!(draft.parsed_key, form.parsed_key);
            assert_eq!(draft.unsupported_reason, form.unsupported_reason);
            assert_eq!(draft.original_fields, draft.draft_fields);
            assert_eq!(draft.dirty_state, StructuredFamilyRecordDraftStatus::Clean);
            assert_eq!(
                draft.validation_status,
                StructuredFamilyRecordDraftStatus::ValidationReady
            );
            assert_eq!(
                draft.action_policy,
                StructuredFamilyRecordDraftStatus::ActionsDisabled
            );
            assert_eq!(
                draft.write_blocked_status,
                StructuredFamilyRecordDraftStatus::WritesBlockedByDefault
            );
            assert_eq!(
                draft.persistence_policy,
                StructuredFamilyRecordDraftStatus::PersistenceForbidden
            );
            assert_eq!(
                draft.created_status,
                StructuredFamilyRecordDraftStatus::CreatedInMemory
            );
            assert!(draft.draft_fields.iter().all(|field| !field.editable));
            assert!(!draft.real_config_touched);
            assert!(!draft.runtime_mutated);
            assert!(!draft.hyprctl_reload_run);
            assert!(!draft.production_executor_wired);
            assert!(!draft.draft_written_to_disk);

            let field_name = draft
                .draft_fields
                .iter()
                .find(|field| field.name != "raw line")
                .or_else(|| draft.draft_fields.first())
                .expect("draft should expose fields")
                .name
                .clone();
            let updated =
                update_structured_family_record_draft_field(draft, &field_name, "fixture-draft");
            assert_eq!(
                updated.dirty_state,
                StructuredFamilyRecordDraftStatus::Dirty
            );
            assert!(updated
                .draft_fields
                .iter()
                .any(|field| field.name == field_name && field.dirty));
            if updated.unsupported_reason.is_some() {
                assert_eq!(
                    updated.validation_status,
                    StructuredFamilyRecordDraftStatus::ValidationWarning
                );
            } else {
                assert_eq!(
                    updated.validation_status,
                    StructuredFamilyRecordDraftStatus::ValidationPassed
                );
            }
            assert!(!updated.real_config_touched);
            assert!(!updated.runtime_mutated);
            assert!(!updated.hyprctl_reload_run);
            assert!(!updated.production_executor_wired);
            assert!(!updated.draft_written_to_disk);

            let reset = reset_structured_family_record_draft(&updated);
            assert_eq!(reset.draft_fields, draft.original_fields);
            assert_eq!(reset.dirty_state, StructuredFamilyRecordDraftStatus::Clean);
            assert_eq!(
                reset.validation_status,
                StructuredFamilyRecordDraftStatus::ValidationReady
            );
            assert!(!reset.draft_written_to_disk);

            let proof = prove_structured_family_record_draft_reset(&updated);
            assert!(proof.original_fields_restored);
            assert_eq!(
                proof.dirty_state_after_reset,
                StructuredFamilyRecordDraftStatus::Clean
            );
            assert!(!proof.real_config_touched);
            assert!(!proof.runtime_mutated);
            assert!(!proof.hyprctl_reload_run);
            assert!(!proof.production_executor_wired);
            assert!(!proof.draft_written_to_disk);
        }
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
        "hyprland-settings-structured-family-record-editor-section",
        "hyprland-settings-structured-family-hl-monitor-record-editor",
        "hyprland-settings-structured-family-hl-bind-record-editor",
        "hyprland-settings-structured-family-hl-animation-record-editor",
        "hyprland-settings-structured-family-hl-curve-record-editor",
        "hyprland-settings-structured-family-hl-gesture-record-editor",
        "hyprland-settings-structured-family-hl-device-record-editor",
        "hyprland-settings-structured-family-hl-permission-record-editor",
        "hyprland-settings-structured-family-record-draft-section",
        "hyprland-settings-structured-family-hl-monitor-record-draft",
        "hyprland-settings-structured-family-hl-bind-record-draft",
        "hyprland-settings-structured-family-hl-animation-record-draft",
        "hyprland-settings-structured-family-hl-curve-record-draft",
        "hyprland-settings-structured-family-hl-gesture-record-draft",
        "hyprland-settings-structured-family-hl-device-record-draft",
        "hyprland-settings-structured-family-hl-permission-record-draft",
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
        "Family-specific validator",
        "Temp-fixture write plan",
        "Temp-fixture write plan validated",
        "Temp-fixture render/reread verified",
        "Production writes blocked by default",
        "Real config target not allowed",
        "Runtime mutation not allowed",
        "Hyprland reload not allowed",
        "Structured family editor projection cannot write real config.",
        "Structured family editor projection cannot reload Hyprland.",
        "Structured family editor projection cannot mutate runtime.",
        "Review-only per-record editor forms",
        "Record editor projection ready",
        "Family-specific fields projected",
        "Raw fallback required where not proven",
        "Editor actions disabled",
        "Real writes blocked by default",
        "source path",
        "line number",
        "raw line",
        "validation status",
        "field count",
        "raw fallback status",
        "write policy",
        "Apply structured-family record change (not available)",
        "Render structured-family record to real config (not available)",
        "Review-only structured-family record drafts",
        "Draft projection ready",
        "Draft created in memory only",
        "Draft starts clean",
        "Draft dirty state tracked",
        "Draft reset proof available",
        "Draft validation ready",
        "Draft actions disabled",
        "Draft persistence forbidden",
        "Update monitor draft (not available)",
        "Update bind draft (not available)",
        "Update animation draft (not available)",
        "Update curve draft (not available)",
        "Update gesture draft (not available)",
        "Update device draft (not available)",
        "Update permission draft (not available)",
        "Reset structured-family draft (not available)",
        "Persist structured-family draft (not available)",
        "Apply structured-family draft to real config (not available)",
    ] {
        assert!(
            window.contains(copy) || model.contains(copy) || structured_family.contains(copy),
            "expected structured-family UI/model source to contain {copy}"
        );
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
fn structured_family_record_draft_section_has_no_write_reload_persistence_or_executor_handlers() {
    let window = fs::read_to_string("src/ui/window.rs").expect("window source should read");
    let section_start = window
        .find("fn structured_family_record_draft_section")
        .expect("record draft section should exist");
    let section_end = window[section_start..]
        .find("fn disabled_future_approval_cards_section")
        .map(|offset| section_start + offset)
        .expect("record draft section should end before future approval cards");
    let section = &window[section_start..section_end];

    for forbidden in [
        "connect_clicked",
        "apply_setting_change",
        "write_flow",
        "hyprctl",
        "hyprctl reload",
        "Command::",
        "File::create",
        "write_all",
        "serde_json::to_writer",
    ] {
        assert!(
            !section.contains(forbidden),
            "record draft section must not contain {forbidden}"
        );
    }
    assert!(section.contains("set_sensitive(false)"));
}

#[test]
fn structured_family_record_editor_section_has_no_write_reload_or_executor_handlers() {
    let window = fs::read_to_string("src/ui/window.rs").expect("window source should read");
    let section_start = window
        .find("fn structured_family_record_editor_section")
        .expect("record editor section should exist");
    let section_end = window[section_start..]
        .find("fn disabled_future_approval_cards_section")
        .map(|offset| section_start + offset)
        .expect("record editor section should end before future approval cards");
    let section = &window[section_start..section_end];

    for forbidden in [
        "connect_clicked",
        "apply_setting_change",
        "write_flow",
        "hyprctl",
        "hyprctl reload",
        "Command::",
        "File::create",
        "write_all",
    ] {
        assert!(
            !section.contains(forbidden),
            "record editor section must not contain {forbidden}"
        );
    }
    assert!(section.contains("set_sensitive(false)"));
}

#[test]
fn structured_family_temp_write_source_has_no_production_write_or_reload_integration() {
    let structured_family = fs::read_to_string("src/structured_family.rs")
        .expect("structured family source should read");
    let window = fs::read_to_string("src/ui/window.rs").expect("window source should read");
    let section_start = window
        .find("fn structured_family_editor_section")
        .expect("structured family section should exist");
    let section_end = window[section_start..]
        .find("fn disabled_future_approval_cards_section")
        .map(|offset| section_start + offset)
        .expect("section should end before future approval cards");
    let section = &window[section_start..section_end];

    for forbidden in [
        "apply_setting_change",
        "apply_safe_batch_setting_changes",
        "apply_scalar_write_plan",
        "execute_safe_batch_write_plan",
        "write_flow::",
        "hyprctl reload",
        "Command::new",
        "File::create",
        "write_all",
    ] {
        assert!(
            !structured_family.contains(forbidden),
            "structured-family temp write plan must not contain production integration: {forbidden}"
        );
        assert!(
            !section.contains(forbidden),
            "structured-family UI section must not contain production integration: {forbidden}"
        );
    }
}

#[test]
fn structured_family_reports_and_continuation_scan_exist() {
    for path in [
        "data/reports/structured-family-editors-unified.v0.55.2.json",
        "data/reports/structured-family-temp-write-plans.v0.55.2.json",
        "data/reports/structured-family-record-editor-forms.v0.55.2.json",
        "data/reports/structured-family-record-draft-model.v0.55.2.json",
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
        "Add disabled live GTK draft-field binding for structured-family record drafts while keeping persistence and real writes blocked."
    );
    assert_eq!(
        handoff["safetyBoundaries"]["structuredFamilyWritesEnabled"],
        false
    );
}

#[test]
fn structured_family_temp_write_plan_report_preserves_safety_boundaries() {
    let report: serde_json::Value = serde_json::from_str(
        &fs::read_to_string("data/reports/structured-family-temp-write-plans.v0.55.2.json")
            .expect("temp write plan report should read"),
    )
    .expect("temp write plan report should be valid JSON");
    assert_eq!(report["artifactKind"], "structured-family-temp-write-plans");
    assert_eq!(
        report["pathGuardStatus"]["realHyprConfigTargetsRejected"],
        true
    );
    assert_eq!(
        report["productionWritePolicy"]["status"],
        "StructuredFamilyProductionWritesBlockedByDefault"
    );
    assert_eq!(
        report["productionWritePolicy"]["realStructuredFamilyWritesEnabled"],
        false
    );
    assert_eq!(report["realConfigTouched"], false);
    assert_eq!(report["runtimeMutated"], false);
    assert_eq!(report["hyprctlReloadRun"], false);
    assert_eq!(report["productionBehaviorEnabled"], false);
    for family in [
        "hl.monitor",
        "hl.bind",
        "hl.animation",
        "hl.curve",
        "hl.gesture",
        "hl.device",
        "hl.permission",
    ] {
        assert_eq!(
            report["tempWritePlanStatusByFamily"][family],
            "StructuredFamilyTempWritePlanValidated"
        );
        assert!(report["fixtureRenderRereadStatusByFamily"][family]
            .as_str()
            .expect("render/reread status should be text")
            .contains("StructuredFamilyTempWritePlanRereadVerified"));
    }
    assert_eq!(
        report["nextRecommendedWork"],
        "Add disabled live GTK draft-field binding for structured-family record drafts while keeping persistence and real writes blocked."
    );
}

#[test]
fn structured_family_record_editor_forms_report_preserves_review_only_policy() {
    let report: serde_json::Value = serde_json::from_str(
        &fs::read_to_string("data/reports/structured-family-record-editor-forms.v0.55.2.json")
            .expect("record editor forms report should read"),
    )
    .expect("record editor forms report should be valid JSON");
    assert_eq!(
        report["artifactKind"],
        "structured-family-record-editor-forms"
    );
    assert_eq!(report["realConfigTouched"], false);
    assert_eq!(report["runtimeMutated"], false);
    assert_eq!(report["hyprctlReloadRun"], false);
    assert_eq!(report["productionBehaviorEnabled"], false);
    assert_eq!(report["productionExecutorWired"], false);
    assert_eq!(
        report["ui"]["sectionWidgetName"],
        "hyprland-settings-structured-family-record-editor-section"
    );
    assert_eq!(report["ui"]["controlsConnectedToMutationHandlers"], false);
    for family in [
        "hl.monitor",
        "hl.bind",
        "hl.animation",
        "hl.curve",
        "hl.gesture",
        "hl.device",
        "hl.permission",
    ] {
        assert_eq!(
            report["recordEditorStatusByFamily"][family],
            "StructuredFamilyRecordEditorProjectionReady"
        );
        assert_eq!(
            report["actionPolicyByFamily"][family],
            "StructuredFamilyRecordEditorActionsDisabled"
        );
        assert_eq!(
            report["writePolicyByFamily"][family],
            "StructuredFamilyRecordEditorWritesBlockedByDefault"
        );
        assert!(report["recordEditorWidgetByFamily"][family]
            .as_str()
            .expect("record editor widget should be text")
            .contains("-record-editor"));
        assert!(report["rawFallbackStatusByFamily"][family]
            .as_str()
            .expect("raw fallback status should be text")
            .contains("StructuredFamilyRecordEditorRawFallbackRequired"));
    }
    assert_eq!(
        report["nextRecommendedWork"],
        "Add disabled live GTK draft-field binding for structured-family record drafts while keeping persistence and real writes blocked."
    );
}

#[test]
fn structured_family_record_draft_model_report_preserves_review_only_policy() {
    let report: serde_json::Value = serde_json::from_str(
        &fs::read_to_string("data/reports/structured-family-record-draft-model.v0.55.2.json")
            .expect("record draft model report should read"),
    )
    .expect("record draft model report should be valid JSON");
    assert_eq!(
        report["artifactKind"],
        "structured-family-record-draft-model"
    );
    assert_eq!(report["realConfigTouched"], false);
    assert_eq!(report["runtimeMutated"], false);
    assert_eq!(report["hyprctlReloadRun"], false);
    assert_eq!(report["productionBehaviorEnabled"], false);
    assert_eq!(report["productionExecutorWired"], false);
    assert_eq!(report["draftWrittenToDisk"], false);
    for family in [
        "hl.monitor",
        "hl.bind",
        "hl.animation",
        "hl.curve",
        "hl.gesture",
        "hl.device",
        "hl.permission",
    ] {
        assert_eq!(
            report["draftStatusByFamily"][family],
            "StructuredFamilyRecordDraftProjectionReady"
        );
        assert_eq!(
            report["dirtyStateStatusByFamily"][family],
            "StructuredFamilyRecordDraftClean initially; StructuredFamilyRecordDraftDirty after model-only update"
        );
        assert_eq!(
            report["resetProofStatusByFamily"][family],
            "StructuredFamilyRecordDraftResetRestoredOriginalFields"
        );
        assert_eq!(
            report["actionPolicyByFamily"][family],
            "StructuredFamilyRecordDraftActionsDisabled"
        );
        assert_eq!(
            report["writePolicyByFamily"][family],
            "StructuredFamilyRecordDraftWritesBlockedByDefault"
        );
        assert_eq!(
            report["persistencePolicyByFamily"][family],
            "StructuredFamilyRecordDraftPersistenceForbidden"
        );
    }
    assert_eq!(
        report["nextRecommendedWork"],
        "Add disabled live GTK draft-field binding for structured-family record drafts while keeping persistence and real writes blocked."
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
