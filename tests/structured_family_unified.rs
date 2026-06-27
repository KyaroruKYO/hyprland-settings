use std::fs;
use std::path::{Path, PathBuf};
use std::time::{SystemTime, UNIX_EPOCH};

use hyprland_settings::config_parser::parse_hyprland_config_file;
use hyprland_settings::current_config::CurrentConfigSnapshot;
use hyprland_settings::structured_family::{
    build_structured_family_temp_write_plan, prove_fixture_parse_render_reread,
    prove_structured_family_draft_rendered_record_plans,
    prove_structured_family_draft_rendered_record_render_reread,
    prove_structured_family_record_draft_reset, prove_structured_family_temp_write_plan,
    render_draft_rendered_record_fixture_text, render_structured_family_projection,
    reset_structured_family_record_draft, structured_family_draft_rendered_record_plans,
    structured_family_kind_from_id, structured_family_record_draft_gtk_bindings,
    structured_family_record_drafts, structured_family_record_editor_forms,
    structured_family_render_target_allowed, update_structured_family_record_draft_field,
    update_structured_family_record_draft_gtk_binding, validate_structured_family_projection,
    StructuredFamilyDraftRenderedRecordRenderRereadStatus,
    StructuredFamilyDraftRenderedRecordStatus, StructuredFamilyKind,
    StructuredFamilyRecordDraftGtkBindingStatus, StructuredFamilyRecordDraftStatus,
    StructuredFamilyRecordEditorStatus, StructuredFamilyStatus,
    StructuredFamilyTempWritePlanStatus, StructuredFamilyValidationStatus,
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
fn all_structured_family_record_draft_gtk_bindings_are_disabled_and_memory_only() {
    for family in StructuredFamilyKind::ALL {
        let snapshot = snapshot_for_family(family);
        let projection = snapshot
            .structured_family_projections()
            .into_iter()
            .find(|projection| projection.family == family)
            .expect("projection should exist");
        let forms = structured_family_record_editor_forms(&projection);
        let drafts = structured_family_record_drafts(&forms);
        let bindings = structured_family_record_draft_gtk_bindings(&drafts);

        assert_eq!(bindings.len(), drafts.len());
        assert_eq!(bindings.len(), projection.record_count());
        assert!(!bindings.is_empty());
        assert!(bindings.iter().any(|binding| {
            binding.raw_fallback_status
                == StructuredFamilyRecordDraftStatus::RawFallbackRequired.as_str()
        }));

        for (index, binding) in bindings.iter().enumerate() {
            let draft = &drafts[index];
            assert_eq!(binding.family, family);
            assert_eq!(binding.record_index, draft.record_index);
            assert_eq!(binding.source_path, draft.source_path);
            assert_eq!(binding.line_number, draft.line_number);
            assert_eq!(binding.fields.len(), draft.draft_fields.len());
            assert_eq!(
                binding.binding_status,
                StructuredFamilyRecordDraftGtkBindingStatus::ProjectionReady
            );
            assert_eq!(
                binding.review_status,
                StructuredFamilyRecordDraftGtkBindingStatus::ReviewOnly
            );
            assert_eq!(
                binding.created_status,
                StructuredFamilyRecordDraftGtkBindingStatus::CreatedInMemory
            );
            assert!(!binding.widget_sensitive);
            assert_eq!(
                binding.action_policy,
                StructuredFamilyRecordDraftGtkBindingStatus::ActionsDisabled
            );
            assert_eq!(
                binding.write_policy,
                StructuredFamilyRecordDraftGtkBindingStatus::WritesBlockedByDefault
            );
            assert_eq!(
                binding.persistence_policy,
                StructuredFamilyRecordDraftGtkBindingStatus::PersistenceForbidden
            );
            assert!(binding.fields.iter().all(|field| !field.widget_sensitive));
            assert!(binding.fields.iter().all(|field| {
                field.binding_status == StructuredFamilyRecordDraftGtkBindingStatus::Disabled
            }));
            assert!(!binding.real_config_touched);
            assert!(!binding.runtime_mutated);
            assert!(!binding.hyprctl_reload_run);
            assert!(!binding.production_executor_wired);
            assert!(!binding.draft_written_to_disk);

            let field_name = draft
                .draft_fields
                .iter()
                .find(|field| field.name != "raw line")
                .or_else(|| draft.draft_fields.first())
                .expect("draft should expose fields")
                .name
                .clone();
            let update = update_structured_family_record_draft_gtk_binding(
                draft,
                &field_name,
                "fixture-gtk-binding",
            );
            assert_eq!(
                update.update_status,
                StructuredFamilyRecordDraftGtkBindingStatus::CanUpdateMemoryOnly
            );
            assert_eq!(
                update.dirty_state_recomputed,
                StructuredFamilyRecordDraftGtkBindingStatus::DirtyStateRecomputed
            );
            assert_eq!(
                update.validation_recomputed,
                StructuredFamilyRecordDraftGtkBindingStatus::ValidationRecomputed
            );
            assert_eq!(
                update.raw_fallback_preserved,
                StructuredFamilyRecordDraftGtkBindingStatus::RawFallbackPreserved
            );
            assert_eq!(
                update.updated_draft.dirty_state,
                StructuredFamilyRecordDraftStatus::Dirty
            );
            assert!(update.reset_restores_original_fields);
            assert!(!update.real_config_touched);
            assert!(!update.runtime_mutated);
            assert!(!update.hyprctl_reload_run);
            assert!(!update.production_executor_wired);
            assert!(!update.draft_written_to_disk);
        }
    }
}

#[test]
fn all_structured_family_drafts_build_fixture_only_rendered_record_plans() {
    for family in StructuredFamilyKind::ALL {
        let snapshot = snapshot_for_family(family);
        let projection = snapshot
            .structured_family_projections()
            .into_iter()
            .find(|projection| projection.family == family)
            .expect("projection should exist");
        let forms = structured_family_record_editor_forms(&projection);
        let drafts = structured_family_record_drafts(&forms);
        let plans = structured_family_draft_rendered_record_plans(&drafts);

        assert_eq!(plans.len(), drafts.len());
        assert_eq!(plans.len(), projection.record_count());
        assert!(!plans.is_empty());
        assert!(plans.iter().any(|plan| {
            plan.raw_fallback_status
                == StructuredFamilyDraftRenderedRecordStatus::RawFallbackPreserved
        }));
        assert!(plans.iter().any(|plan| {
            plan.unsupported_not_proven_status
                == StructuredFamilyDraftRenderedRecordStatus::UnsupportedNotProvenYet
        }));

        for (index, plan) in plans.iter().enumerate() {
            let draft = &drafts[index];
            assert_eq!(plan.family, family);
            assert_eq!(plan.record_index, draft.record_index);
            assert_eq!(plan.source_path, draft.source_path);
            assert_eq!(plan.line_number, draft.line_number);
            assert_eq!(plan.raw_original_line, draft.raw_original_line);
            assert_eq!(plan.parsed_key, draft.parsed_key);
            assert_eq!(plan.draft_fields, draft.draft_fields);
            assert_eq!(
                plan.plan_status,
                StructuredFamilyDraftRenderedRecordStatus::PlanReady
            );
            assert_eq!(
                plan.review_status,
                StructuredFamilyDraftRenderedRecordStatus::ReviewOnly
            );
            assert_eq!(
                plan.created_status,
                StructuredFamilyDraftRenderedRecordStatus::CreatedInMemory
            );
            assert_eq!(
                plan.fixture_only_status,
                StructuredFamilyDraftRenderedRecordStatus::FixtureOnly
            );
            assert_eq!(
                plan.action_policy,
                StructuredFamilyDraftRenderedRecordStatus::ActionsDisabled
            );
            assert_eq!(
                plan.write_policy,
                StructuredFamilyDraftRenderedRecordStatus::WritesBlockedByDefault
            );
            assert_eq!(
                plan.persistence_policy,
                StructuredFamilyDraftRenderedRecordStatus::PersistenceForbidden
            );
            assert_eq!(
                plan.real_config_target_policy,
                StructuredFamilyDraftRenderedRecordStatus::RealConfigTargetForbidden
            );
            assert!(!plan.field_map.is_empty());
            assert!(plan
                .field_map
                .iter()
                .all(|field| field.status
                    == StructuredFamilyDraftRenderedRecordStatus::FieldMapReady));
            assert!(!plan.rendered_record_preview.trim().is_empty());
            if plan.unsupported_reason.is_some() {
                assert_eq!(
                    plan.rendered_record_preview,
                    plan.raw_original_line,
                    "unsupported plans must preserve raw line for {}",
                    family.family_id()
                );
                assert_eq!(
                    plan.rendered_record_syntax_status,
                    StructuredFamilyDraftRenderedRecordStatus::UnsupportedNotProvenYet
                );
            } else {
                assert_eq!(
                    plan.rendered_record_syntax_status,
                    StructuredFamilyDraftRenderedRecordStatus::SyntaxProjected
                );
            }
            assert!(!plan.draft_written_to_disk);
            assert!(!plan.rendered_record_written_to_disk);
            assert!(!plan.real_config_touched);
            assert!(!plan.runtime_mutated);
            assert!(!plan.hyprctl_reload_run);
            assert!(!plan.production_executor_wired);
        }

        let proof = prove_structured_family_draft_rendered_record_plans(&drafts);
        assert_eq!(proof.family, family);
        assert_eq!(proof.plan_count, drafts.len());
        assert_eq!(proof.draft_count, drafts.len());
        assert!(proof.field_map_count >= drafts.len());
        assert!(proof.raw_fallback_plan_count > 0);
        assert!(proof.unsupported_not_proven_plan_count > 0);
        assert_eq!(
            proof.fixture_only_status,
            StructuredFamilyDraftRenderedRecordStatus::FixtureOnly
        );
        assert_eq!(
            proof.write_policy,
            StructuredFamilyDraftRenderedRecordStatus::WritesBlockedByDefault
        );
        assert_eq!(
            proof.persistence_policy,
            StructuredFamilyDraftRenderedRecordStatus::PersistenceForbidden
        );
        assert_eq!(
            proof.real_config_target_policy,
            StructuredFamilyDraftRenderedRecordStatus::RealConfigTargetForbidden
        );
        assert!(!proof.draft_written_to_disk);
        assert!(!proof.rendered_record_written_to_disk);
        assert!(!proof.real_config_touched);
        assert!(!proof.runtime_mutated);
        assert!(!proof.hyprctl_reload_run);
        assert!(!proof.production_executor_wired);
    }
}

#[test]
fn all_structured_family_rendered_record_plans_render_and_reread_temp_fixtures_only() {
    for family in StructuredFamilyKind::ALL {
        let snapshot = snapshot_for_family(family);
        let projection = snapshot
            .structured_family_projections()
            .into_iter()
            .find(|projection| projection.family == family)
            .expect("projection should exist");
        let forms = structured_family_record_editor_forms(&projection);
        let drafts = structured_family_record_drafts(&forms);
        let plans = structured_family_draft_rendered_record_plans(&drafts);
        let output_path = temp_output_path(family);

        let rendered_text = render_draft_rendered_record_fixture_text(&plans);
        assert!(!rendered_text.trim().is_empty());
        assert!(structured_family_render_target_allowed(&output_path));

        let proof =
            prove_structured_family_draft_rendered_record_render_reread(&plans, &output_path);
        assert_eq!(proof.family, family);
        assert_eq!(proof.source_draft_count, drafts.len());
        assert_eq!(proof.source_plan_count, plans.len());
        assert_eq!(proof.rendered_fixture_path, output_path);
        assert_eq!(proof.rendered_fixture_text, rendered_text);
        assert_eq!(proof.reread_projection_family, family);
        assert_eq!(proof.reread_record_count, plans.len());
        assert!(proof.family_preserved);
        assert!(proof.record_count_preserved);
        assert!(proof.field_map_preserved);
        assert!(proof.raw_fallback_preserved);
        assert!(proof.unsupported_not_proven_preserved);
        assert!(proof.field_map_count >= plans.len());
        assert!(proof.raw_fallback_plan_count > 0);
        assert!(proof.unsupported_not_proven_plan_count > 0);
        assert_eq!(
            proof.render_reread_status,
            StructuredFamilyDraftRenderedRecordRenderRereadStatus::ReviewOnly
        );
        assert_eq!(
            proof.rendered_temp_fixture_status,
            StructuredFamilyDraftRenderedRecordRenderRereadStatus::RenderedToTempFixture
        );
        assert_eq!(
            proof.reread_status,
            StructuredFamilyDraftRenderedRecordRenderRereadStatus::RereadFromTempFixture
        );
        assert_eq!(
            proof.family_preservation_status,
            StructuredFamilyDraftRenderedRecordRenderRereadStatus::FamilyPreserved
        );
        assert_eq!(
            proof.record_count_preservation_status,
            StructuredFamilyDraftRenderedRecordRenderRereadStatus::RecordCountPreserved
        );
        assert_eq!(
            proof.field_map_preservation_status,
            StructuredFamilyDraftRenderedRecordRenderRereadStatus::FieldMapPreserved
        );
        assert_eq!(
            proof.raw_fallback_preservation_status,
            StructuredFamilyDraftRenderedRecordRenderRereadStatus::RawFallbackPreserved
        );
        assert_eq!(
            proof.unsupported_not_proven_preservation_status,
            StructuredFamilyDraftRenderedRecordRenderRereadStatus::UnsupportedNotProvenYet
        );
        assert_eq!(
            proof.fixture_only_status,
            StructuredFamilyDraftRenderedRecordRenderRereadStatus::FixtureOnly
        );
        assert_eq!(
            proof.action_policy,
            StructuredFamilyDraftRenderedRecordRenderRereadStatus::ActionsDisabled
        );
        assert_eq!(
            proof.write_policy,
            StructuredFamilyDraftRenderedRecordRenderRereadStatus::WritesBlockedByDefault
        );
        assert_eq!(
            proof.persistence_policy,
            StructuredFamilyDraftRenderedRecordRenderRereadStatus::PersistenceForbidden
        );
        assert_eq!(
            proof.real_config_target_policy,
            StructuredFamilyDraftRenderedRecordRenderRereadStatus::RealConfigTargetForbidden
        );
        assert!(!proof.draft_written_to_disk);
        assert!(proof.rendered_record_written_to_temp_fixture);
        assert!(!proof.rendered_record_written_to_real_config);
        assert!(!proof.real_config_touched);
        assert!(!proof.runtime_mutated);
        assert!(!proof.hyprctl_reload_run);
        assert!(!proof.production_executor_wired);

        fs::remove_file(output_path).expect("render/reread proof should clean up temp fixture");
    }
}

#[test]
fn structured_family_rendered_record_reread_proof_rejects_real_config_targets() {
    let family = StructuredFamilyKind::Monitor;
    let snapshot = snapshot_for_family(family);
    let projection = snapshot
        .structured_family_projections()
        .into_iter()
        .find(|projection| projection.family == family)
        .expect("projection should exist");
    let forms = structured_family_record_editor_forms(&projection);
    let drafts = structured_family_record_drafts(&forms);
    let plans = structured_family_draft_rendered_record_plans(&drafts);
    let real_config = PathBuf::from("/home/kyo/.config/hypr/hyprland.conf");

    let proof = prove_structured_family_draft_rendered_record_render_reread(&plans, &real_config);
    assert_eq!(
        proof.render_reread_status,
        StructuredFamilyDraftRenderedRecordRenderRereadStatus::RealConfigTargetForbidden
    );
    assert!(!proof.rendered_record_written_to_temp_fixture);
    assert!(!proof.rendered_record_written_to_real_config);
    assert!(!proof.real_config_touched);
    assert!(!proof.runtime_mutated);
    assert!(!proof.hyprctl_reload_run);
    assert!(!proof.production_executor_wired);
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
        "hyprland-settings-structured-family-record-draft-binding-section",
        "hyprland-settings-structured-family-hl-monitor-record-draft-binding",
        "hyprland-settings-structured-family-hl-bind-record-draft-binding",
        "hyprland-settings-structured-family-hl-animation-record-draft-binding",
        "hyprland-settings-structured-family-hl-curve-record-draft-binding",
        "hyprland-settings-structured-family-hl-gesture-record-draft-binding",
        "hyprland-settings-structured-family-hl-device-record-draft-binding",
        "hyprland-settings-structured-family-hl-permission-record-draft-binding",
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
        "Disabled live GTK draft-field binding",
        "Draft-field binding projection ready",
        "Draft-field widgets insensitive",
        "Draft-field update is memory-only",
        "Draft dirty state recomputed",
        "Draft validation recomputed",
        "Raw fallback preserved",
        "Draft binding actions disabled",
        "Draft binding persistence forbidden",
        "Update monitor draft field (not available)",
        "Update bind draft field (not available)",
        "Update animation draft field (not available)",
        "Update curve draft field (not available)",
        "Update gesture draft field (not available)",
        "Update device draft field (not available)",
        "Update permission draft field (not available)",
        "Reset structured-family GTK draft binding (not available)",
        "Persist structured-family GTK draft binding (not available)",
        "Apply structured-family GTK draft binding to real config (not available)",
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
fn structured_family_record_draft_binding_section_has_no_write_reload_persistence_or_executor_handlers(
) {
    let window = fs::read_to_string("src/ui/window.rs").expect("window source should read");
    let section_start = window
        .find("fn structured_family_record_draft_binding_section")
        .expect("record draft binding section should exist");
    let section_end = window[section_start..]
        .find("fn disabled_future_approval_cards_section")
        .map(|offset| section_start + offset)
        .expect("record draft binding section should end before future approval cards");
    let section = &window[section_start..section_end];

    for forbidden in [
        "connect_clicked",
        "apply_setting_change",
        "write_flow",
        "hyprctl reload",
        "Command::",
        "File::create",
        "write_all",
        "serde_json::to_writer",
    ] {
        assert!(
            !section.contains(forbidden),
            "record draft binding section must not contain {forbidden}"
        );
    }
    assert!(section.contains("set_sensitive(false)"));
}

#[test]
fn structured_family_draft_rendered_record_planning_has_no_write_reload_or_persistence_calls() {
    let structured_family = fs::read_to_string("src/structured_family.rs")
        .expect("structured family source should read");
    let section_start = structured_family
        .find("pub fn structured_family_draft_rendered_record_plans")
        .expect("draft rendered record planning function should exist");
    let section_end = structured_family[section_start..]
        .find("pub fn structured_family_render_target_allowed")
        .map(|offset| section_start + offset)
        .expect("planning section should end before render target path guard");
    let section = &structured_family[section_start..section_end];

    for forbidden in [
        "apply_setting_change",
        "write_flow",
        "hyprctl reload",
        "Command::",
        "File::create",
        "fs::write",
        "write_all",
        "serde_json::to_writer",
        "/home/kyo/.config/hypr/hyprland.conf",
        "~/.config/hypr",
    ] {
        assert!(
            !section.contains(forbidden),
            "draft-to-rendered-record planning must not contain {forbidden}"
        );
    }
}

#[test]
fn structured_family_rendered_record_reread_proof_has_path_guard_and_no_real_write_path() {
    let structured_family = fs::read_to_string("src/structured_family.rs")
        .expect("structured family source should read");
    let section_start = structured_family
        .find("pub fn prove_structured_family_draft_rendered_record_render_reread")
        .expect("render/reread proof function should exist");
    let section_end = structured_family[section_start..]
        .find("pub fn render_draft_rendered_record_fixture_text")
        .map(|offset| section_start + offset)
        .expect("render/reread proof section should end before render helper");
    let section = &structured_family[section_start..section_end];

    assert!(section.contains("structured_family_render_target_allowed"));
    assert!(section.contains("parse_hyprland_config_file"));
    assert!(section.contains("fs::write"));
    for forbidden in [
        "apply_setting_change",
        "write_flow",
        "hyprctl reload",
        "Command::",
        "File::create",
        "write_all",
        "serde_json::to_writer",
        "/home/kyo/.config/hypr/hyprland.conf",
        "~/.config/hypr",
    ] {
        assert!(
            !section.contains(forbidden),
            "render/reread proof must not contain {forbidden}"
        );
    }
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
        "data/reports/structured-family-record-draft-gtk-binding.v0.55.2.json",
        "data/reports/structured-family-draft-rendered-record-plan.v0.55.2.json",
        "data/reports/structured-family-draft-rendered-record-render-reread.v0.55.2.json",
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
        "Add fixture-only structured-family draft rendered-record diff/review summary while keeping real writes blocked."
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
        "Add fixture-only structured-family draft rendered-record diff/review summary while keeping real writes blocked."
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
        "Add fixture-only structured-family draft rendered-record diff/review summary while keeping real writes blocked."
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
        "Add fixture-only structured-family draft rendered-record diff/review summary while keeping real writes blocked."
    );
}

#[test]
fn structured_family_record_draft_gtk_binding_report_preserves_review_only_policy() {
    let report: serde_json::Value = serde_json::from_str(
        &fs::read_to_string("data/reports/structured-family-record-draft-gtk-binding.v0.55.2.json")
            .expect("record draft GTK binding report should read"),
    )
    .expect("record draft GTK binding report should be valid JSON");
    assert_eq!(
        report["artifactKind"],
        "structured-family-record-draft-gtk-binding"
    );
    assert_eq!(report["realConfigTouched"], false);
    assert_eq!(report["runtimeMutated"], false);
    assert_eq!(report["hyprctlReloadRun"], false);
    assert_eq!(report["productionBehaviorEnabled"], false);
    assert_eq!(report["productionExecutorWired"], false);
    assert_eq!(report["draftWrittenToDisk"], false);
    assert_eq!(
        report["ui"]["sectionWidgetName"],
        "hyprland-settings-structured-family-record-draft-binding-section"
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
            report["bindingStatusByFamily"][family],
            "StructuredFamilyRecordDraftGtkBindingProjectionReady"
        );
        assert!(report["bindingWidgetByFamily"][family]
            .as_str()
            .expect("binding widget should be text")
            .contains("-record-draft-binding"));
        assert_eq!(
            report["insensitiveWidgetStatusByFamily"][family],
            "StructuredFamilyRecordDraftGtkBindingDisabled"
        );
        assert_eq!(
            report["dirtyStateRecomputeStatusByFamily"][family],
            "StructuredFamilyRecordDraftGtkBindingDirtyStateRecomputed"
        );
        assert_eq!(
            report["validationRecomputeStatusByFamily"][family],
            "StructuredFamilyRecordDraftGtkBindingValidationRecomputed"
        );
        assert_eq!(
            report["resetProofStatusByFamily"][family],
            "StructuredFamilyRecordDraftResetRestoredOriginalFields"
        );
        assert_eq!(
            report["rawFallbackStatusByFamily"][family],
            "StructuredFamilyRecordDraftGtkBindingRawFallbackPreserved"
        );
        assert_eq!(
            report["actionPolicyByFamily"][family],
            "StructuredFamilyRecordDraftGtkBindingActionsDisabled"
        );
        assert_eq!(
            report["writePolicyByFamily"][family],
            "StructuredFamilyRecordDraftGtkBindingWritesBlockedByDefault"
        );
        assert_eq!(
            report["persistencePolicyByFamily"][family],
            "StructuredFamilyRecordDraftGtkBindingPersistenceForbidden"
        );
    }
    assert_eq!(
        report["nextRecommendedWork"],
        "Add fixture-only structured-family draft rendered-record diff/review summary while keeping real writes blocked."
    );
}

#[test]
fn structured_family_draft_rendered_record_plan_report_preserves_fixture_only_policy() {
    let report: serde_json::Value = serde_json::from_str(
        &fs::read_to_string(
            "data/reports/structured-family-draft-rendered-record-plan.v0.55.2.json",
        )
        .expect("draft rendered record plan report should read"),
    )
    .expect("draft rendered record plan report should be valid JSON");
    assert_eq!(
        report["artifactKind"],
        "structured-family-draft-rendered-record-plan"
    );
    assert_eq!(report["draftWrittenToDisk"], false);
    assert_eq!(report["renderedRecordWrittenToDisk"], false);
    assert_eq!(report["realConfigTouched"], false);
    assert_eq!(report["runtimeMutated"], false);
    assert_eq!(report["hyprctlReloadRun"], false);
    assert_eq!(report["productionBehaviorEnabled"], false);
    assert_eq!(report["productionExecutorWired"], false);
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
            report["planStatusByFamily"][family],
            "StructuredFamilyDraftRenderedRecordPlanReady"
        );
        assert_eq!(
            report["fieldMapStatusByFamily"][family],
            "StructuredFamilyDraftRenderedRecordFieldMapReady"
        );
        assert_eq!(
            report["renderedPreviewStatusByFamily"][family],
            "StructuredFamilyDraftRenderedRecordSyntaxProjected"
        );
        assert_eq!(
            report["rawFallbackStatusByFamily"][family],
            "StructuredFamilyDraftRenderedRecordRawFallbackPreserved"
        );
        assert_eq!(
            report["unsupportedNotProvenStatusByFamily"][family],
            "StructuredFamilyDraftRenderedRecordUnsupportedNotProvenYet"
        );
        assert_eq!(
            report["actionPolicyByFamily"][family],
            "StructuredFamilyDraftRenderedRecordActionsDisabled"
        );
        assert_eq!(
            report["writePolicyByFamily"][family],
            "StructuredFamilyDraftRenderedRecordWritesBlockedByDefault"
        );
        assert_eq!(
            report["persistencePolicyByFamily"][family],
            "StructuredFamilyDraftRenderedRecordPersistenceForbidden"
        );
        assert_eq!(
            report["realConfigTargetPolicyByFamily"][family],
            "StructuredFamilyDraftRenderedRecordRealConfigTargetForbidden"
        );
    }
    assert_eq!(
        report["nextRecommendedWork"],
        "Add fixture-only structured-family draft rendered-record diff/review summary while keeping real writes blocked."
    );
}

#[test]
fn structured_family_draft_rendered_record_render_reread_report_preserves_fixture_only_policy() {
    let report: serde_json::Value = serde_json::from_str(
        &fs::read_to_string(
            "data/reports/structured-family-draft-rendered-record-render-reread.v0.55.2.json",
        )
        .expect("draft rendered record render/reread report should read"),
    )
    .expect("draft rendered record render/reread report should be valid JSON");
    assert_eq!(
        report["artifactKind"],
        "structured-family-draft-rendered-record-render-reread"
    );
    assert_eq!(report["draftWrittenToDisk"], false);
    assert_eq!(report["renderedRecordWrittenToTempFixture"], true);
    assert_eq!(report["renderedRecordWrittenToRealConfig"], false);
    assert_eq!(report["realConfigTouched"], false);
    assert_eq!(report["runtimeMutated"], false);
    assert_eq!(report["hyprctlReloadRun"], false);
    assert_eq!(report["productionBehaviorEnabled"], false);
    assert_eq!(report["productionExecutorWired"], false);
    assert_eq!(report["gtkEvidenceRoot"], "not-run-no-visible-ui-change");
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
            report["renderRereadStatusByFamily"][family],
            "StructuredFamilyDraftRenderedRecordRenderRereadReviewOnly"
        );
        assert_eq!(
            report["renderedTempFixtureStatusByFamily"][family],
            "StructuredFamilyDraftRenderedRecordRenderedToTempFixture"
        );
        assert_eq!(
            report["rereadStatusByFamily"][family],
            "StructuredFamilyDraftRenderedRecordRereadFromTempFixture"
        );
        assert_eq!(
            report["familyPreservationStatusByFamily"][family],
            "StructuredFamilyDraftRenderedRecordFamilyPreserved"
        );
        assert_eq!(
            report["recordCountPreservationStatusByFamily"][family],
            "StructuredFamilyDraftRenderedRecordRecordCountPreserved"
        );
        assert_eq!(
            report["fieldMapPreservationStatusByFamily"][family],
            "StructuredFamilyDraftRenderedRecordFieldMapPreserved"
        );
        assert_eq!(
            report["rawFallbackPreservationStatusByFamily"][family],
            "StructuredFamilyDraftRenderedRecordRawFallbackPreserved"
        );
        assert_eq!(
            report["unsupportedNotProvenPreservationStatusByFamily"][family],
            "StructuredFamilyDraftRenderedRecordUnsupportedNotProvenYet"
        );
        assert_eq!(
            report["actionPolicyByFamily"][family],
            "StructuredFamilyDraftRenderedRecordActionsDisabled"
        );
        assert_eq!(
            report["writePolicyByFamily"][family],
            "StructuredFamilyDraftRenderedRecordWritesBlockedByDefault"
        );
        assert_eq!(
            report["persistencePolicyByFamily"][family],
            "StructuredFamilyDraftRenderedRecordPersistenceForbidden"
        );
        assert_eq!(
            report["realConfigTargetPolicyByFamily"][family],
            "StructuredFamilyDraftRenderedRecordRealConfigTargetForbidden"
        );
    }
    assert_eq!(
        report["nextRecommendedWork"],
        "Add fixture-only structured-family draft rendered-record diff/review summary while keeping real writes blocked."
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
