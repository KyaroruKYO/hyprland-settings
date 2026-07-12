use std::fs;
use std::path::{Path, PathBuf};
use std::time::{SystemTime, UNIX_EPOCH};

use hyprland_settings::config_parser::parse_hyprland_config_file;
use hyprland_settings::current_config::CurrentConfigSnapshot;
use hyprland_settings::structured_family::{
    accept_structured_family_draft_rendered_record_confirmation,
    build_structured_family_temp_write_plan, prove_fixture_parse_render_reread,
    prove_structured_family_draft_rendered_record_plans,
    prove_structured_family_draft_rendered_record_render_reread,
    prove_structured_family_record_draft_reset, prove_structured_family_temp_write_plan,
    reject_structured_family_draft_rendered_record_confirmation,
    render_draft_rendered_record_fixture_text, render_structured_family_projection,
    reset_structured_family_record_draft, structured_family_draft_rendered_record_approval_draft,
    structured_family_draft_rendered_record_confirmation_invalidation_reasons,
    structured_family_draft_rendered_record_confirmation_request,
    structured_family_draft_rendered_record_diff_review_summary,
    structured_family_draft_rendered_record_final_executor_readiness_audit,
    structured_family_draft_rendered_record_final_executor_readiness_audit_blockers,
    structured_family_draft_rendered_record_final_executor_readiness_audit_with_links,
    structured_family_draft_rendered_record_plans,
    structured_family_draft_rendered_record_staged_apply_blockers,
    structured_family_draft_rendered_record_staged_apply_dry_run_report,
    structured_family_draft_rendered_record_staged_apply_plan,
    structured_family_draft_rendered_record_staged_apply_rollback_recovery_review,
    structured_family_executor_architecture_implementation_plan,
    structured_family_internal_safe_write_architecture_plan, structured_family_kind_from_id,
    structured_family_production_activation_design,
    structured_family_production_activation_planning_scope,
    structured_family_real_write_activation_requirements_audit,
    structured_family_record_draft_gtk_bindings, structured_family_record_drafts,
    structured_family_record_editor_forms, structured_family_render_target_allowed,
    update_structured_family_record_draft_field, update_structured_family_record_draft_gtk_binding,
    validate_structured_family_projection, StructuredFamilyDraftRenderedRecordApprovalStatus,
    StructuredFamilyDraftRenderedRecordConfirmationInvalidationReason,
    StructuredFamilyDraftRenderedRecordDiffReviewStatus,
    StructuredFamilyDraftRenderedRecordFinalExecutorReadinessBlocker,
    StructuredFamilyDraftRenderedRecordFinalExecutorReadinessDecision,
    StructuredFamilyDraftRenderedRecordFinalExecutorReadinessFinding,
    StructuredFamilyDraftRenderedRecordFinalExecutorReadinessStatus,
    StructuredFamilyDraftRenderedRecordRenderRereadStatus,
    StructuredFamilyDraftRenderedRecordRollbackRecoveryBlocker,
    StructuredFamilyDraftRenderedRecordRollbackRecoveryRequirement,
    StructuredFamilyDraftRenderedRecordStagedApplyBlocker,
    StructuredFamilyDraftRenderedRecordStagedApplyDryRunStatus,
    StructuredFamilyDraftRenderedRecordStagedApplyRollbackRecoveryStatus,
    StructuredFamilyDraftRenderedRecordStagedApplyStatus,
    StructuredFamilyDraftRenderedRecordStatus, StructuredFamilyKind,
    StructuredFamilyRealWriteActivationAuditStatus, StructuredFamilyRecordDraftGtkBindingStatus,
    StructuredFamilyRecordDraftStatus, StructuredFamilyRecordEditorStatus, StructuredFamilyStatus,
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
fn all_structured_family_rendered_record_plans_build_in_memory_diff_review_summaries() {
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
        let proof =
            prove_structured_family_draft_rendered_record_render_reread(&plans, &output_path);

        let summary = structured_family_draft_rendered_record_diff_review_summary(&plans, &proof);
        assert_eq!(summary.family, family);
        assert_eq!(summary.source_draft_count, drafts.len());
        assert_eq!(summary.source_plan_count, plans.len());
        assert_eq!(summary.review_entry_count, plans.len());
        assert_eq!(summary.entries.len(), plans.len());
        assert_eq!(
            summary.render_reread_proof_status,
            StructuredFamilyDraftRenderedRecordDiffReviewStatus::RenderRereadProofLinked
        );
        assert_eq!(
            summary.diff_review_status,
            StructuredFamilyDraftRenderedRecordDiffReviewStatus::ReviewOnly
        );
        assert_eq!(
            summary.review_summary_status,
            StructuredFamilyDraftRenderedRecordDiffReviewStatus::ReviewSummaryReady
        );
        assert_eq!(
            summary.field_diff_status,
            StructuredFamilyDraftRenderedRecordDiffReviewStatus::FieldDiffReady
        );
        assert_eq!(
            summary.raw_fallback_review_status,
            StructuredFamilyDraftRenderedRecordDiffReviewStatus::RawFallbackPreserved
        );
        assert_eq!(
            summary.unsupported_not_proven_review_status,
            StructuredFamilyDraftRenderedRecordDiffReviewStatus::UnsupportedNotProvenYet
        );
        assert_eq!(
            summary.fixture_only_status,
            StructuredFamilyDraftRenderedRecordDiffReviewStatus::FixtureOnly
        );
        assert_eq!(
            summary.action_policy,
            StructuredFamilyDraftRenderedRecordDiffReviewStatus::ActionsDisabled
        );
        assert_eq!(
            summary.write_policy,
            StructuredFamilyDraftRenderedRecordDiffReviewStatus::WritesBlockedByDefault
        );
        assert_eq!(
            summary.persistence_policy,
            StructuredFamilyDraftRenderedRecordDiffReviewStatus::PersistenceForbidden
        );
        assert_eq!(
            summary.real_config_target_policy,
            StructuredFamilyDraftRenderedRecordDiffReviewStatus::RealConfigTargetForbidden
        );
        assert!(summary.field_diff_count >= plans.len());
        assert_eq!(
            summary.raw_fallback_entry_count,
            proof.raw_fallback_plan_count
        );
        assert_eq!(
            summary.unsupported_not_proven_entry_count,
            proof.unsupported_not_proven_plan_count
        );
        assert!(summary.summary_text.contains(family.family_id()));
        assert!(summary.risk_summary.contains("real writes"));
        assert!(!summary.draft_written_to_disk);
        assert!(!summary.diff_summary_written_to_disk);
        assert!(summary.rendered_record_written_to_temp_fixture);
        assert!(!summary.rendered_record_written_to_real_config);
        assert!(!summary.real_config_touched);
        assert!(!summary.runtime_mutated);
        assert!(!summary.hyprctl_reload_run);
        assert!(!summary.production_executor_wired);

        for (index, entry) in summary.entries.iter().enumerate() {
            let plan = &plans[index];
            assert_eq!(entry.family, family);
            assert_eq!(entry.record_index, plan.record_index);
            assert_eq!(entry.source_path, plan.source_path);
            assert_eq!(entry.line_number, plan.line_number);
            assert_eq!(entry.original_raw_line, plan.raw_original_line);
            assert_eq!(entry.rendered_record_preview, plan.rendered_record_preview);
            assert!(!entry.field_diffs.is_empty());
            assert_eq!(
                entry.field_diff_status,
                StructuredFamilyDraftRenderedRecordDiffReviewStatus::FieldDiffReady
            );
            assert_eq!(
                entry.rendered_preview_compared_status,
                StructuredFamilyDraftRenderedRecordDiffReviewStatus::RenderedPreviewCompared
            );
            assert_eq!(
                entry.original_raw_preserved_status,
                StructuredFamilyDraftRenderedRecordDiffReviewStatus::OriginalRawPreserved
            );
            assert_eq!(
                entry.review_decision_status,
                StructuredFamilyDraftRenderedRecordDiffReviewStatus::ReviewSummaryReady
            );
            if plan.unsupported_reason.is_some() {
                assert_eq!(
                    entry.raw_fallback_status,
                    StructuredFamilyDraftRenderedRecordDiffReviewStatus::RawFallbackPreserved
                );
                assert_eq!(
                    entry.unsupported_not_proven_status,
                    StructuredFamilyDraftRenderedRecordDiffReviewStatus::UnsupportedNotProvenYet
                );
                assert!(entry.not_safe_for_full_synthesis);
            }
        }

        fs::remove_file(output_path).expect("diff/review summary proof should clean up temp file");
    }
}

#[test]
fn structured_family_diff_review_summaries_detect_model_only_modified_drafts() {
    for family in StructuredFamilyKind::ALL {
        let snapshot = snapshot_for_family(family);
        let projection = snapshot
            .structured_family_projections()
            .into_iter()
            .find(|projection| projection.family == family)
            .expect("projection should exist");
        let forms = structured_family_record_editor_forms(&projection);
        let drafts = structured_family_record_drafts(&forms);
        let supported_index = drafts
            .iter()
            .position(|draft| draft.unsupported_reason.is_none())
            .expect("fixture should include one supported record");
        let field_name = drafts[supported_index]
            .draft_fields
            .iter()
            .find(|field| field.name != "raw line" && field.name != "parsed key")
            .expect("supported draft should expose a family-specific field")
            .name
            .clone();

        let baseline_plans = structured_family_draft_rendered_record_plans(&drafts);
        let baseline_output_path = temp_output_path(family);
        let baseline_proof = prove_structured_family_draft_rendered_record_render_reread(
            &baseline_plans,
            &baseline_output_path,
        );
        let baseline_summary = structured_family_draft_rendered_record_diff_review_summary(
            &baseline_plans,
            &baseline_proof,
        );

        let mut modified_drafts = drafts.clone();
        modified_drafts[supported_index] = update_structured_family_record_draft_field(
            &modified_drafts[supported_index],
            &field_name,
            changed_fixture_value_for_family(family, &field_name),
        );
        let modified_plans = structured_family_draft_rendered_record_plans(&modified_drafts);
        let modified_output_path = temp_output_path(family);
        let modified_proof = prove_structured_family_draft_rendered_record_render_reread(
            &modified_plans,
            &modified_output_path,
        );
        let modified_summary = structured_family_draft_rendered_record_diff_review_summary(
            &modified_plans,
            &modified_proof,
        );

        assert!(modified_summary.changed_entry_count >= baseline_summary.changed_entry_count);
        assert!(
            modified_summary.changed_field_diff_count > baseline_summary.changed_field_diff_count,
            "model-only field update should increase changed field-diff count for {}",
            family.family_id()
        );
        assert!(modified_summary.entries[supported_index].changed);
        assert!(modified_summary.entries[supported_index]
            .field_diffs
            .iter()
            .any(|field_diff| field_diff.field_name == field_name && field_diff.changed));
        assert!(!modified_summary.draft_written_to_disk);
        assert!(!modified_summary.diff_summary_written_to_disk);
        assert!(!modified_summary.rendered_record_written_to_real_config);
        assert!(!modified_summary.real_config_touched);
        assert!(!modified_summary.runtime_mutated);
        assert!(!modified_summary.hyprctl_reload_run);
        assert!(!modified_summary.production_executor_wired);

        fs::remove_file(baseline_output_path)
            .expect("baseline diff/review proof should clean up temp file");
        fs::remove_file(modified_output_path)
            .expect("modified diff/review proof should clean up temp file");
    }
}

#[test]
fn all_structured_family_diff_reviews_create_fixture_only_approval_confirmations() {
    for family in StructuredFamilyKind::ALL {
        let summary = diff_review_summary_for_family(family);
        let approval = structured_family_draft_rendered_record_approval_draft(&summary);

        assert_eq!(approval.family, family);
        assert_eq!(approval.source_draft_count, summary.source_draft_count);
        assert_eq!(approval.source_plan_count, summary.source_plan_count);
        assert_eq!(approval.review_entry_count, summary.review_entry_count);
        assert_eq!(approval.changed_entry_count, summary.changed_entry_count);
        assert_eq!(approval.noop_entry_count, summary.noop_entry_count);
        assert_eq!(
            approval.raw_fallback_entry_count,
            summary.raw_fallback_entry_count
        );
        assert_eq!(
            approval.unsupported_not_proven_entry_count,
            summary.unsupported_not_proven_entry_count
        );
        assert_eq!(approval.field_diff_count, summary.field_diff_count);
        assert!(approval.diff_review_summary_linked);
        assert!(approval.render_reread_proof_linked);
        assert_eq!(
            approval.approval_status,
            StructuredFamilyDraftRenderedRecordApprovalStatus::ReviewOnly
        );
        assert_eq!(
            approval.confirmation_status,
            StructuredFamilyDraftRenderedRecordApprovalStatus::ConfirmationDraftReady
        );
        assert_eq!(
            approval.fixture_only_status,
            StructuredFamilyDraftRenderedRecordApprovalStatus::FixtureOnly
        );
        assert_eq!(
            approval.action_policy,
            StructuredFamilyDraftRenderedRecordApprovalStatus::ActionsDisabled
        );
        assert_eq!(
            approval.write_policy,
            StructuredFamilyDraftRenderedRecordApprovalStatus::WritesBlockedByDefault
        );
        assert_eq!(
            approval.persistence_policy,
            StructuredFamilyDraftRenderedRecordApprovalStatus::PersistenceForbidden
        );
        assert_eq!(
            approval.real_config_target_policy,
            StructuredFamilyDraftRenderedRecordApprovalStatus::RealConfigTargetForbidden
        );
        assert_eq!(
            approval.production_executor_policy,
            StructuredFamilyDraftRenderedRecordApprovalStatus::ProductionExecutorForbidden
        );
        assert_eq!(
            approval.raw_fallback_acknowledged,
            approval.raw_fallback_entry_count == 0
        );
        assert_eq!(
            approval.unsupported_not_proven_acknowledged,
            approval.unsupported_not_proven_entry_count == 0
        );
        assert!(approval
            .summary_text
            .contains("fixture-only next-stage review only"));
        assert!(approval
            .risk_summary
            .contains("does not authorize real config writes"));
        assert!(!approval.draft_written_to_disk);
        assert!(!approval.approval_written_to_disk);
        assert!(!approval.confirmation_written_to_disk);
        assert!(!approval.rendered_record_written_to_real_config);
        assert!(!approval.real_config_touched);
        assert!(!approval.runtime_mutated);
        assert!(!approval.hyprctl_reload_run);
        assert!(!approval.production_executor_wired);

        let request = structured_family_draft_rendered_record_confirmation_request(&approval);
        let accepted =
            accept_structured_family_draft_rendered_record_confirmation(&approval, &request);
        assert_eq!(accepted.family, family);
        assert_eq!(
            accepted.confirmation_status,
            StructuredFamilyDraftRenderedRecordApprovalStatus::ConfirmationAcceptedInMemory
        );
        assert!(accepted.confirmation_accepted_in_memory);
        assert!(!accepted.confirmation_rejected_in_memory);
        assert!(accepted.confirmation_invalidation_reasons.is_empty());
        assert!(accepted.changed_entries_acknowledged);
        assert!(accepted.noop_entries_acknowledged);
        assert!(accepted.raw_fallback_acknowledged);
        assert!(accepted.unsupported_not_proven_acknowledged);
        assert_eq!(
            accepted.fixture_only_status,
            StructuredFamilyDraftRenderedRecordApprovalStatus::FixtureOnly
        );
        assert_eq!(
            accepted.write_policy,
            StructuredFamilyDraftRenderedRecordApprovalStatus::WritesBlockedByDefault
        );
        assert_eq!(
            accepted.persistence_policy,
            StructuredFamilyDraftRenderedRecordApprovalStatus::PersistenceForbidden
        );
        assert_eq!(
            accepted.real_config_target_policy,
            StructuredFamilyDraftRenderedRecordApprovalStatus::RealConfigTargetForbidden
        );
        assert_eq!(
            accepted.production_executor_policy,
            StructuredFamilyDraftRenderedRecordApprovalStatus::ProductionExecutorForbidden
        );
        assert!(!accepted.draft_written_to_disk);
        assert!(!accepted.approval_written_to_disk);
        assert!(!accepted.confirmation_written_to_disk);
        assert!(!accepted.rendered_record_written_to_real_config);
        assert!(!accepted.real_config_touched);
        assert!(!accepted.runtime_mutated);
        assert!(!accepted.hyprctl_reload_run);
        assert!(!accepted.production_executor_wired);

        let rejected =
            reject_structured_family_draft_rendered_record_confirmation(&approval, &request);
        assert_eq!(
            rejected.confirmation_status,
            StructuredFamilyDraftRenderedRecordApprovalStatus::ConfirmationRejectedInMemory
        );
        assert!(!rejected.confirmation_accepted_in_memory);
        assert!(rejected.confirmation_rejected_in_memory);
        assert!(!rejected.rendered_record_written_to_real_config);
        assert!(!rejected.real_config_touched);
        assert!(!rejected.runtime_mutated);
        assert!(!rejected.hyprctl_reload_run);
        assert!(!rejected.production_executor_wired);
    }
}

#[test]
fn structured_family_approval_confirmations_report_required_invalidation_reasons() {
    let approval = structured_family_draft_rendered_record_approval_draft(
        &diff_review_summary_for_family(StructuredFamilyKind::Bind),
    );
    let valid_request = structured_family_draft_rendered_record_confirmation_request(&approval);

    assert_confirmation_invalid_reason(
        &approval,
        valid_request.clone(),
        |request| request.family = StructuredFamilyKind::Monitor,
        StructuredFamilyDraftRenderedRecordConfirmationInvalidationReason::MismatchedFamily,
    );
    assert_confirmation_invalid_reason(
        &approval,
        valid_request.clone(),
        |request| request.source_plan_count += 1,
        StructuredFamilyDraftRenderedRecordConfirmationInvalidationReason::MismatchedSourcePlanCount,
    );
    assert_confirmation_invalid_reason(
        &approval,
        valid_request.clone(),
        |request| request.review_entry_count += 1,
        StructuredFamilyDraftRenderedRecordConfirmationInvalidationReason::MismatchedReviewEntryCount,
    );
    assert_confirmation_invalid_reason(
        &approval,
        valid_request.clone(),
        |request| request.changed_entry_count += 1,
        StructuredFamilyDraftRenderedRecordConfirmationInvalidationReason::MismatchedChangedEntryCount,
    );
    assert_confirmation_invalid_reason(
        &approval,
        valid_request.clone(),
        |request| request.raw_fallback_entry_count += 1,
        StructuredFamilyDraftRenderedRecordConfirmationInvalidationReason::MismatchedRawFallbackCount,
    );
    assert_confirmation_invalid_reason(
        &approval,
        valid_request.clone(),
        |request| request.unsupported_not_proven_entry_count += 1,
        StructuredFamilyDraftRenderedRecordConfirmationInvalidationReason::MismatchedUnsupportedNotProvenCount,
    );
    assert_confirmation_invalid_reason(
        &approval,
        valid_request.clone(),
        |request| request.diff_review_summary_linked = false,
        StructuredFamilyDraftRenderedRecordConfirmationInvalidationReason::MissingDiffReviewSummary,
    );
    assert_confirmation_invalid_reason(
        &approval,
        valid_request.clone(),
        |request| request.render_reread_proof_linked = false,
        StructuredFamilyDraftRenderedRecordConfirmationInvalidationReason::MissingRenderRereadProofLink,
    );
    assert_confirmation_invalid_reason(
        &approval,
        valid_request.clone(),
        |request| request.raw_fallback_acknowledged = false,
        StructuredFamilyDraftRenderedRecordConfirmationInvalidationReason::RawFallbackRequiresAcknowledgement,
    );
    assert_confirmation_invalid_reason(
        &approval,
        valid_request.clone(),
        |request| request.unsupported_not_proven_acknowledged = false,
        StructuredFamilyDraftRenderedRecordConfirmationInvalidationReason::UnsupportedNotProvenRequiresAcknowledgement,
    );
    assert_confirmation_invalid_reason(
        &approval,
        valid_request.clone(),
        |request| request.real_config_target_forbidden = false,
        StructuredFamilyDraftRenderedRecordConfirmationInvalidationReason::RealConfigTargetNotAllowed,
    );
    assert_confirmation_invalid_reason(
        &approval,
        valid_request.clone(),
        |request| request.persistence_forbidden = false,
        StructuredFamilyDraftRenderedRecordConfirmationInvalidationReason::PersistenceNotAllowed,
    );
    assert_confirmation_invalid_reason(
        &approval,
        valid_request.clone(),
        |request| request.runtime_mutation_forbidden = false,
        StructuredFamilyDraftRenderedRecordConfirmationInvalidationReason::RuntimeMutationNotAllowed,
    );
    assert_confirmation_invalid_reason(
        &approval,
        valid_request.clone(),
        |request| request.hyprland_reload_forbidden = false,
        StructuredFamilyDraftRenderedRecordConfirmationInvalidationReason::HyprlandReloadNotAllowed,
    );
    assert_confirmation_invalid_reason(
        &approval,
        valid_request,
        |request| request.production_executor_forbidden = false,
        StructuredFamilyDraftRenderedRecordConfirmationInvalidationReason::ProductionExecutorNotAllowed,
    );
}

#[test]
fn all_structured_family_accepted_confirmations_create_fixture_only_staged_apply_plans() {
    for family in StructuredFamilyKind::ALL {
        let summary = diff_review_summary_for_family(family);
        let approval = structured_family_draft_rendered_record_approval_draft(&summary);
        let request = structured_family_draft_rendered_record_confirmation_request(&approval);
        let accepted =
            accept_structured_family_draft_rendered_record_confirmation(&approval, &request);
        let plan = structured_family_draft_rendered_record_staged_apply_plan(&accepted, &summary);

        assert_eq!(plan.family, family);
        assert_eq!(plan.source_draft_count, summary.source_draft_count);
        assert_eq!(plan.source_plan_count, summary.source_plan_count);
        assert_eq!(plan.review_entry_count, summary.review_entry_count);
        assert_eq!(plan.changed_entry_count, summary.changed_entry_count);
        assert_eq!(plan.noop_entry_count, summary.noop_entry_count);
        assert_eq!(
            plan.raw_fallback_entry_count,
            summary.raw_fallback_entry_count
        );
        assert_eq!(
            plan.unsupported_not_proven_entry_count,
            summary.unsupported_not_proven_entry_count
        );
        assert_eq!(plan.field_diff_count, summary.field_diff_count);
        assert!(plan.accepted_confirmation_linked);
        assert!(plan.diff_review_summary_linked);
        assert!(plan.render_reread_proof_linked);
        assert_eq!(plan.stage_count, 7);
        assert_eq!(
            plan.operation_count,
            plan.changed_operation_count
                + plan.noop_operation_count
                + plan.raw_fallback_preservation_operation_count
                + plan.unsupported_not_proven_preservation_operation_count
        );
        assert_eq!(plan.changed_operation_count, summary.changed_entry_count);
        assert_eq!(plan.noop_operation_count, summary.noop_entry_count);
        assert_eq!(
            plan.raw_fallback_preservation_operation_count,
            summary.raw_fallback_entry_count
        );
        assert_eq!(
            plan.unsupported_not_proven_preservation_operation_count,
            summary.unsupported_not_proven_entry_count
        );
        assert_eq!(plan.stages.len(), plan.stage_count);
        assert_eq!(plan.operations.len(), plan.operation_count);
        assert!(plan.blockers.is_empty());
        assert_eq!(
            plan.staged_apply_status,
            StructuredFamilyDraftRenderedRecordStagedApplyStatus::PlanReady
        );
        assert_eq!(
            plan.preflight_stage,
            StructuredFamilyDraftRenderedRecordStagedApplyStatus::PreflightReady
        );
        assert_eq!(
            plan.review_stage,
            StructuredFamilyDraftRenderedRecordStagedApplyStatus::DiffReviewLinked
        );
        assert_eq!(
            plan.render_preview_stage,
            StructuredFamilyDraftRenderedRecordStagedApplyStatus::RenderRereadLinked
        );
        assert_eq!(
            plan.raw_fallback_preservation_stage,
            StructuredFamilyDraftRenderedRecordStagedApplyStatus::RawFallbackPreserved
        );
        assert_eq!(
            plan.unsupported_not_proven_preservation_stage,
            StructuredFamilyDraftRenderedRecordStagedApplyStatus::UnsupportedNotProvenPreserved
        );
        assert_eq!(
            plan.dry_run_only_apply_stage,
            StructuredFamilyDraftRenderedRecordStagedApplyStatus::DryRunOnly
        );
        assert_eq!(
            plan.rollback_plan_stage,
            StructuredFamilyDraftRenderedRecordStagedApplyStatus::RollbackPlanReady
        );
        assert_eq!(
            plan.fixture_only_status,
            StructuredFamilyDraftRenderedRecordStagedApplyStatus::FixtureOnly
        );
        assert_eq!(
            plan.action_policy,
            StructuredFamilyDraftRenderedRecordStagedApplyStatus::ActionsDisabled
        );
        assert_eq!(
            plan.write_policy,
            StructuredFamilyDraftRenderedRecordStagedApplyStatus::WritesBlockedByDefault
        );
        assert_eq!(
            plan.persistence_policy,
            StructuredFamilyDraftRenderedRecordStagedApplyStatus::PersistenceForbidden
        );
        assert_eq!(
            plan.real_config_target_policy,
            StructuredFamilyDraftRenderedRecordStagedApplyStatus::RealConfigTargetForbidden
        );
        assert_eq!(
            plan.production_executor_policy,
            StructuredFamilyDraftRenderedRecordStagedApplyStatus::ProductionExecutorForbidden
        );
        assert_eq!(
            plan.executor_availability_status,
            StructuredFamilyDraftRenderedRecordStagedApplyStatus::ProductionExecutorForbidden
        );
        assert!(plan.summary_text.contains("staged apply plan generated"));
        assert!(plan.risk_summary.contains("review-only and in memory"));
        assert!(!plan.draft_written_to_disk);
        assert!(!plan.staged_apply_plan_written_to_disk);
        assert!(!plan.staged_apply_executed);
        assert!(!plan.rendered_record_written_to_real_config);
        assert!(!plan.real_config_touched);
        assert!(!plan.runtime_mutated);
        assert!(!plan.hyprctl_reload_run);
        assert!(!plan.production_executor_wired);
    }
}

#[test]
fn structured_family_staged_apply_plans_block_rejected_invalid_and_unsafe_confirmations() {
    let summary = diff_review_summary_for_family(StructuredFamilyKind::Bind);
    let approval = structured_family_draft_rendered_record_approval_draft(&summary);
    let request = structured_family_draft_rendered_record_confirmation_request(&approval);
    let accepted = accept_structured_family_draft_rendered_record_confirmation(&approval, &request);
    let rejected = reject_structured_family_draft_rendered_record_confirmation(&approval, &request);
    let invalid_request = {
        let mut request = request.clone();
        request.changed_entry_count += 1;
        request
    };
    let invalid =
        accept_structured_family_draft_rendered_record_confirmation(&approval, &invalid_request);

    let rejected_plan =
        structured_family_draft_rendered_record_staged_apply_plan(&rejected, &summary);
    assert!(rejected_plan
        .blockers
        .contains(&StructuredFamilyDraftRenderedRecordStagedApplyBlocker::RejectedConfirmation));
    assert_eq!(
        rejected_plan.staged_apply_status,
        StructuredFamilyDraftRenderedRecordStagedApplyStatus::RejectedConfirmationBlocked
    );
    assert!(!rejected_plan.staged_apply_executed);
    assert!(!rejected_plan.real_config_touched);
    assert!(!rejected_plan.runtime_mutated);
    assert!(!rejected_plan.hyprctl_reload_run);
    assert!(!rejected_plan.production_executor_wired);

    let invalid_plan =
        structured_family_draft_rendered_record_staged_apply_plan(&invalid, &summary);
    assert!(invalid_plan
        .blockers
        .contains(&StructuredFamilyDraftRenderedRecordStagedApplyBlocker::InvalidConfirmation));
    assert_eq!(
        invalid_plan.staged_apply_status,
        StructuredFamilyDraftRenderedRecordStagedApplyStatus::InvalidConfirmationBlocked
    );
    assert!(!invalid_plan.staged_apply_executed);
    assert!(!invalid_plan.rendered_record_written_to_real_config);

    assert_staged_apply_blocker(
        &accepted,
        &summary,
        |confirmation| confirmation.confirmation_accepted_in_memory = false,
        StructuredFamilyDraftRenderedRecordStagedApplyBlocker::MissingAcceptedConfirmation,
    );
    assert_staged_apply_blocker(
        &accepted,
        &summary,
        |confirmation| confirmation.diff_review_summary_linked = false,
        StructuredFamilyDraftRenderedRecordStagedApplyBlocker::MissingDiffReviewSummary,
    );
    assert_staged_apply_blocker(
        &accepted,
        &summary,
        |confirmation| confirmation.render_reread_proof_linked = false,
        StructuredFamilyDraftRenderedRecordStagedApplyBlocker::MissingRenderRereadProof,
    );
    assert_staged_apply_blocker(
        &accepted,
        &summary,
        |confirmation| confirmation.raw_fallback_acknowledged = false,
        StructuredFamilyDraftRenderedRecordStagedApplyBlocker::MissingApprovalAcknowledgement,
    );
    assert_staged_apply_blocker(
        &accepted,
        &summary,
        |confirmation| confirmation.unsupported_not_proven_acknowledged = false,
        StructuredFamilyDraftRenderedRecordStagedApplyBlocker::MissingApprovalAcknowledgement,
    );
    assert_staged_apply_blocker(
        &accepted,
        &summary,
        |confirmation| {
            confirmation.real_config_target_policy =
                StructuredFamilyDraftRenderedRecordApprovalStatus::Ready
        },
        StructuredFamilyDraftRenderedRecordStagedApplyBlocker::RealConfigTargetNotAllowed,
    );
    assert_staged_apply_blocker(
        &accepted,
        &summary,
        |confirmation| {
            confirmation.persistence_policy =
                StructuredFamilyDraftRenderedRecordApprovalStatus::Ready
        },
        StructuredFamilyDraftRenderedRecordStagedApplyBlocker::PersistenceNotAllowed,
    );
    assert_staged_apply_blocker(
        &accepted,
        &summary,
        |confirmation| confirmation.runtime_mutated = true,
        StructuredFamilyDraftRenderedRecordStagedApplyBlocker::RuntimeMutationNotAllowed,
    );
    assert_staged_apply_blocker(
        &accepted,
        &summary,
        |confirmation| confirmation.hyprctl_reload_run = true,
        StructuredFamilyDraftRenderedRecordStagedApplyBlocker::HyprlandReloadNotAllowed,
    );
    assert_staged_apply_blocker(
        &accepted,
        &summary,
        |confirmation| {
            confirmation.production_executor_policy =
                StructuredFamilyDraftRenderedRecordApprovalStatus::Ready
        },
        StructuredFamilyDraftRenderedRecordStagedApplyBlocker::ProductionExecutorNotAllowed,
    );
}

#[test]
fn all_structured_family_staged_apply_plans_create_fixture_only_dry_run_reports() {
    for family in StructuredFamilyKind::ALL {
        let summary = diff_review_summary_for_family(family);
        let approval = structured_family_draft_rendered_record_approval_draft(&summary);
        let request = structured_family_draft_rendered_record_confirmation_request(&approval);
        let accepted =
            accept_structured_family_draft_rendered_record_confirmation(&approval, &request);
        let plan = structured_family_draft_rendered_record_staged_apply_plan(&accepted, &summary);
        let dry_run = structured_family_draft_rendered_record_staged_apply_dry_run_report(&plan);

        assert_eq!(dry_run.family, family);
        assert_eq!(dry_run.source_draft_count, plan.source_draft_count);
        assert_eq!(dry_run.source_plan_count, plan.source_plan_count);
        assert_eq!(dry_run.review_entry_count, plan.review_entry_count);
        assert_eq!(dry_run.changed_entry_count, plan.changed_entry_count);
        assert_eq!(dry_run.noop_entry_count, plan.noop_entry_count);
        assert_eq!(
            dry_run.raw_fallback_entry_count,
            plan.raw_fallback_entry_count
        );
        assert_eq!(
            dry_run.unsupported_not_proven_entry_count,
            plan.unsupported_not_proven_entry_count
        );
        assert_eq!(dry_run.field_diff_count, plan.field_diff_count);
        assert!(dry_run.staged_apply_plan_linked);
        assert_eq!(dry_run.stage_count, plan.stage_count);
        assert_eq!(dry_run.operation_count, plan.operation_count);
        assert_eq!(
            dry_run.changed_operation_count,
            plan.changed_operation_count
        );
        assert_eq!(dry_run.noop_operation_count, plan.noop_operation_count);
        assert_eq!(
            dry_run.raw_fallback_preservation_operation_count,
            plan.raw_fallback_preservation_operation_count
        );
        assert_eq!(
            dry_run.unsupported_not_proven_preservation_operation_count,
            plan.unsupported_not_proven_preservation_operation_count
        );
        assert_eq!(dry_run.entries.len(), plan.operation_count);
        assert_eq!(dry_run.blocked_plan_count, 0);
        assert!(dry_run.executor_unavailable_by_design);
        assert_eq!(
            dry_run.dry_run_report_status,
            StructuredFamilyDraftRenderedRecordStagedApplyDryRunStatus::ReportReady
        );
        assert_eq!(
            dry_run.plan_status,
            StructuredFamilyDraftRenderedRecordStagedApplyDryRunStatus::PlanLinked
        );
        assert_eq!(
            dry_run.stage_summary_status,
            StructuredFamilyDraftRenderedRecordStagedApplyDryRunStatus::StagesSummarized
        );
        assert_eq!(
            dry_run.operation_summary_status,
            StructuredFamilyDraftRenderedRecordStagedApplyDryRunStatus::OperationsSummarized
        );
        assert_eq!(
            dry_run.changed_operation_summary_status,
            StructuredFamilyDraftRenderedRecordStagedApplyDryRunStatus::ChangedOperationsSummarized
        );
        assert_eq!(
            dry_run.noop_operation_summary_status,
            StructuredFamilyDraftRenderedRecordStagedApplyDryRunStatus::NoopOperationsSummarized
        );
        assert_eq!(
            dry_run.raw_fallback_preservation_summary_status,
            StructuredFamilyDraftRenderedRecordStagedApplyDryRunStatus::RawFallbackPreserved
        );
        assert_eq!(
            dry_run.unsupported_not_proven_preservation_summary_status,
            StructuredFamilyDraftRenderedRecordStagedApplyDryRunStatus::UnsupportedNotProvenPreserved
        );
        assert_eq!(
            dry_run.executor_availability_status,
            StructuredFamilyDraftRenderedRecordStagedApplyDryRunStatus::ExecutorUnavailable
        );
        assert_eq!(
            dry_run.dry_run_execution_status,
            StructuredFamilyDraftRenderedRecordStagedApplyDryRunStatus::NotExecuted
        );
        assert_eq!(
            dry_run.fixture_only_status,
            StructuredFamilyDraftRenderedRecordStagedApplyDryRunStatus::FixtureOnly
        );
        assert_eq!(
            dry_run.action_policy,
            StructuredFamilyDraftRenderedRecordStagedApplyDryRunStatus::ActionsDisabled
        );
        assert_eq!(
            dry_run.write_policy,
            StructuredFamilyDraftRenderedRecordStagedApplyDryRunStatus::WritesBlockedByDefault
        );
        assert_eq!(
            dry_run.persistence_policy,
            StructuredFamilyDraftRenderedRecordStagedApplyDryRunStatus::PersistenceForbidden
        );
        assert_eq!(
            dry_run.real_config_target_policy,
            StructuredFamilyDraftRenderedRecordStagedApplyDryRunStatus::RealConfigTargetForbidden
        );
        assert_eq!(
            dry_run.production_executor_policy,
            StructuredFamilyDraftRenderedRecordStagedApplyDryRunStatus::ProductionExecutorForbidden
        );
        assert!(dry_run.summary_text.contains("dry-run report generated"));
        assert!(dry_run.summary_text.contains("dry-run executed false"));
        assert!(dry_run
            .risk_summary
            .contains("executor unavailable by design"));
        assert!(!dry_run.draft_written_to_disk);
        assert!(!dry_run.dry_run_report_written_to_disk);
        assert!(!dry_run.staged_apply_plan_written_to_disk);
        assert!(!dry_run.staged_apply_executed);
        assert!(!dry_run.dry_run_executed);
        assert!(!dry_run.rendered_record_written_to_real_config);
        assert!(!dry_run.real_config_touched);
        assert!(!dry_run.runtime_mutated);
        assert!(!dry_run.hyprctl_reload_run);
        assert!(!dry_run.production_executor_wired);
    }
}

#[test]
fn structured_family_staged_apply_dry_run_reports_summarize_blocked_plans() {
    let summary = diff_review_summary_for_family(StructuredFamilyKind::Bind);
    let approval = structured_family_draft_rendered_record_approval_draft(&summary);
    let request = structured_family_draft_rendered_record_confirmation_request(&approval);
    let accepted = accept_structured_family_draft_rendered_record_confirmation(&approval, &request);
    let rejected = reject_structured_family_draft_rendered_record_confirmation(&approval, &request);
    let invalid_request = {
        let mut request = request.clone();
        request.changed_entry_count += 1;
        request
    };
    let invalid =
        accept_structured_family_draft_rendered_record_confirmation(&approval, &invalid_request);

    for (confirmation, expected) in [
        (
            rejected,
            StructuredFamilyDraftRenderedRecordStagedApplyBlocker::RejectedConfirmation,
        ),
        (
            invalid,
            StructuredFamilyDraftRenderedRecordStagedApplyBlocker::InvalidConfirmation,
        ),
    ] {
        let plan =
            structured_family_draft_rendered_record_staged_apply_plan(&confirmation, &summary);
        let dry_run = structured_family_draft_rendered_record_staged_apply_dry_run_report(&plan);
        assert!(dry_run.blockers.contains(&expected));
        assert_eq!(dry_run.blocked_plan_count, 1);
        assert_eq!(
            dry_run.dry_run_report_status,
            StructuredFamilyDraftRenderedRecordStagedApplyDryRunStatus::BlockedPlanSummarized
        );
        assert!(!dry_run.staged_apply_executed);
        assert!(!dry_run.dry_run_executed);
        assert!(!dry_run.real_config_touched);
        assert!(!dry_run.runtime_mutated);
        assert!(!dry_run.hyprctl_reload_run);
        assert!(!dry_run.production_executor_wired);
    }

    assert_staged_apply_dry_run_blocker(
        &accepted,
        &summary,
        |confirmation| confirmation.confirmation_accepted_in_memory = false,
        StructuredFamilyDraftRenderedRecordStagedApplyBlocker::MissingAcceptedConfirmation,
    );
    assert_staged_apply_dry_run_blocker(
        &accepted,
        &summary,
        |confirmation| confirmation.diff_review_summary_linked = false,
        StructuredFamilyDraftRenderedRecordStagedApplyBlocker::MissingDiffReviewSummary,
    );
    assert_staged_apply_dry_run_blocker(
        &accepted,
        &summary,
        |confirmation| confirmation.render_reread_proof_linked = false,
        StructuredFamilyDraftRenderedRecordStagedApplyBlocker::MissingRenderRereadProof,
    );
    assert_staged_apply_dry_run_blocker(
        &accepted,
        &summary,
        |confirmation| {
            confirmation.real_config_target_policy =
                StructuredFamilyDraftRenderedRecordApprovalStatus::Ready
        },
        StructuredFamilyDraftRenderedRecordStagedApplyBlocker::RealConfigTargetNotAllowed,
    );
    assert_staged_apply_dry_run_blocker(
        &accepted,
        &summary,
        |confirmation| {
            confirmation.persistence_policy =
                StructuredFamilyDraftRenderedRecordApprovalStatus::Ready
        },
        StructuredFamilyDraftRenderedRecordStagedApplyBlocker::PersistenceNotAllowed,
    );
    assert_staged_apply_dry_run_blocker(
        &accepted,
        &summary,
        |confirmation| confirmation.runtime_mutated = true,
        StructuredFamilyDraftRenderedRecordStagedApplyBlocker::RuntimeMutationNotAllowed,
    );
    assert_staged_apply_dry_run_blocker(
        &accepted,
        &summary,
        |confirmation| confirmation.hyprctl_reload_run = true,
        StructuredFamilyDraftRenderedRecordStagedApplyBlocker::HyprlandReloadNotAllowed,
    );
    assert_staged_apply_dry_run_blocker(
        &accepted,
        &summary,
        |confirmation| {
            confirmation.production_executor_policy =
                StructuredFamilyDraftRenderedRecordApprovalStatus::Ready
        },
        StructuredFamilyDraftRenderedRecordStagedApplyBlocker::ProductionExecutorNotAllowed,
    );
}

#[test]
fn all_structured_family_dry_run_reports_create_fixture_only_rollback_recovery_reviews() {
    for family in StructuredFamilyKind::ALL {
        let summary = diff_review_summary_for_family(family);
        let approval = structured_family_draft_rendered_record_approval_draft(&summary);
        let request = structured_family_draft_rendered_record_confirmation_request(&approval);
        let accepted =
            accept_structured_family_draft_rendered_record_confirmation(&approval, &request);
        let plan = structured_family_draft_rendered_record_staged_apply_plan(&accepted, &summary);
        let dry_run = structured_family_draft_rendered_record_staged_apply_dry_run_report(&plan);
        let review =
            structured_family_draft_rendered_record_staged_apply_rollback_recovery_review(&dry_run);

        assert_eq!(review.family, family);
        assert_eq!(review.source_draft_count, dry_run.source_draft_count);
        assert_eq!(review.source_plan_count, dry_run.source_plan_count);
        assert_eq!(review.review_entry_count, dry_run.review_entry_count);
        assert_eq!(review.changed_entry_count, dry_run.changed_entry_count);
        assert_eq!(review.noop_entry_count, dry_run.noop_entry_count);
        assert_eq!(
            review.raw_fallback_entry_count,
            dry_run.raw_fallback_entry_count
        );
        assert_eq!(
            review.unsupported_not_proven_entry_count,
            dry_run.unsupported_not_proven_entry_count
        );
        assert_eq!(review.field_diff_count, dry_run.field_diff_count);
        assert!(review.dry_run_report_linked);
        assert!(review.staged_apply_plan_linked);
        assert_eq!(review.stage_count, dry_run.stage_count);
        assert_eq!(review.operation_count, dry_run.operation_count);
        assert_eq!(
            review.changed_operation_count,
            dry_run.changed_operation_count
        );
        assert_eq!(review.noop_operation_count, dry_run.noop_operation_count);
        assert_eq!(
            review.raw_fallback_preservation_operation_count,
            dry_run.raw_fallback_preservation_operation_count
        );
        assert_eq!(
            review.unsupported_not_proven_preservation_operation_count,
            dry_run.unsupported_not_proven_preservation_operation_count
        );
        assert_eq!(review.blocked_plan_count, dry_run.blocked_plan_count);
        assert_eq!(review.rollback_review_entry_count, dry_run.entries.len());
        assert!(review.recovery_requirement_count >= 9);
        assert_eq!(review.backup_requirement_count, 1);
        assert_eq!(review.restore_requirement_count, 1);
        assert_eq!(review.blocked_recovery_reason_count, 0);
        assert!(review.executor_unavailable_by_design);
        assert!(review.recovery_requirements.contains(
            &StructuredFamilyDraftRenderedRecordRollbackRecoveryRequirement::BackupRequiredBeforeFutureApply
        ));
        assert!(review.recovery_requirements.contains(
            &StructuredFamilyDraftRenderedRecordRollbackRecoveryRequirement::RestoreRequiredBeforeFutureRecovery
        ));
        assert!(review.recovery_requirements.contains(
            &StructuredFamilyDraftRenderedRecordRollbackRecoveryRequirement::DryRunMustRemainNotExecuted
        ));
        assert!(review.recovery_requirements.contains(
            &StructuredFamilyDraftRenderedRecordRollbackRecoveryRequirement::StagedApplyMustRemainNotExecuted
        ));
        if dry_run.raw_fallback_entry_count > 0 {
            assert!(review.recovery_requirements.contains(
                &StructuredFamilyDraftRenderedRecordRollbackRecoveryRequirement::RawFallbackRequiresPreservation
            ));
        }
        if dry_run.unsupported_not_proven_entry_count > 0 {
            assert!(review.recovery_requirements.contains(
                &StructuredFamilyDraftRenderedRecordRollbackRecoveryRequirement::UnsupportedNotProvenRequiresPreservation
            ));
        }
        assert_eq!(
            review.rollback_recovery_review_status,
            StructuredFamilyDraftRenderedRecordStagedApplyRollbackRecoveryStatus::ReviewReady
        );
        assert_eq!(
            review.dry_run_link_status,
            StructuredFamilyDraftRenderedRecordStagedApplyRollbackRecoveryStatus::DryRunLinked
        );
        assert_eq!(
            review.staged_apply_plan_link_status,
            StructuredFamilyDraftRenderedRecordStagedApplyRollbackRecoveryStatus::PlanLinked
        );
        assert_eq!(
            review.rollback_plan_summary_status,
            StructuredFamilyDraftRenderedRecordStagedApplyRollbackRecoveryStatus::RollbackPlanSummarized
        );
        assert_eq!(
            review.recovery_path_summary_status,
            StructuredFamilyDraftRenderedRecordStagedApplyRollbackRecoveryStatus::RecoveryPathSummarized
        );
        assert_eq!(
            review.backup_requirement_status,
            StructuredFamilyDraftRenderedRecordStagedApplyRollbackRecoveryStatus::BackupRequirementReady
        );
        assert_eq!(
            review.restore_requirement_status,
            StructuredFamilyDraftRenderedRecordStagedApplyRollbackRecoveryStatus::RestoreRequirementReady
        );
        assert_eq!(
            review.blocked_plan_preservation_status,
            StructuredFamilyDraftRenderedRecordStagedApplyRollbackRecoveryStatus::BlockedPlanPreserved
        );
        assert_eq!(
            review.executor_availability_status,
            StructuredFamilyDraftRenderedRecordStagedApplyRollbackRecoveryStatus::ExecutorUnavailable
        );
        assert_eq!(
            review.execution_status,
            StructuredFamilyDraftRenderedRecordStagedApplyRollbackRecoveryStatus::NotExecuted
        );
        assert_eq!(
            review.fixture_only_status,
            StructuredFamilyDraftRenderedRecordStagedApplyRollbackRecoveryStatus::FixtureOnly
        );
        assert_eq!(
            review.action_policy,
            StructuredFamilyDraftRenderedRecordStagedApplyRollbackRecoveryStatus::ActionsDisabled
        );
        assert_eq!(
            review.write_policy,
            StructuredFamilyDraftRenderedRecordStagedApplyRollbackRecoveryStatus::WritesBlockedByDefault
        );
        assert_eq!(
            review.persistence_policy,
            StructuredFamilyDraftRenderedRecordStagedApplyRollbackRecoveryStatus::PersistenceForbidden
        );
        assert_eq!(
            review.real_config_target_policy,
            StructuredFamilyDraftRenderedRecordStagedApplyRollbackRecoveryStatus::RealConfigTargetForbidden
        );
        assert_eq!(
            review.production_executor_policy,
            StructuredFamilyDraftRenderedRecordStagedApplyRollbackRecoveryStatus::ProductionExecutorForbidden
        );
        assert!(review
            .summary_text
            .contains("rollback/recovery review generated"));
        assert!(review.summary_text.contains("rollback executed false"));
        assert!(review.summary_text.contains("recovery executed false"));
        assert!(review.summary_text.contains("backup created false"));
        assert!(review.summary_text.contains("restore executed false"));
        assert!(review
            .risk_summary
            .contains("executor unavailable by design"));
        assert!(!review.draft_written_to_disk);
        assert!(!review.rollback_recovery_review_written_to_disk);
        assert!(!review.dry_run_report_written_to_disk);
        assert!(!review.staged_apply_plan_written_to_disk);
        assert!(!review.staged_apply_executed);
        assert!(!review.dry_run_executed);
        assert!(!review.rollback_executed);
        assert!(!review.recovery_executed);
        assert!(!review.backup_created);
        assert!(!review.restore_executed);
        assert!(!review.rendered_record_written_to_real_config);
        assert!(!review.real_config_touched);
        assert!(!review.runtime_mutated);
        assert!(!review.hyprctl_reload_run);
        assert!(!review.production_executor_wired);
    }
}

#[test]
fn structured_family_rollback_recovery_reviews_preserve_blocked_dry_run_reports() {
    let summary = diff_review_summary_for_family(StructuredFamilyKind::Bind);
    let approval = structured_family_draft_rendered_record_approval_draft(&summary);
    let request = structured_family_draft_rendered_record_confirmation_request(&approval);
    let accepted = accept_structured_family_draft_rendered_record_confirmation(&approval, &request);
    let rejected = reject_structured_family_draft_rendered_record_confirmation(&approval, &request);
    let invalid_request = {
        let mut request = request.clone();
        request.changed_entry_count += 1;
        request
    };
    let invalid =
        accept_structured_family_draft_rendered_record_confirmation(&approval, &invalid_request);

    for confirmation in [rejected, invalid] {
        let plan =
            structured_family_draft_rendered_record_staged_apply_plan(&confirmation, &summary);
        let dry_run = structured_family_draft_rendered_record_staged_apply_dry_run_report(&plan);
        let review =
            structured_family_draft_rendered_record_staged_apply_rollback_recovery_review(&dry_run);
        assert_eq!(
            review.rollback_recovery_review_status,
            StructuredFamilyDraftRenderedRecordStagedApplyRollbackRecoveryStatus::BlockedPlanPreserved
        );
        assert_eq!(review.blocked_plan_count, 1);
        assert!(review.recovery_blockers.contains(
            &StructuredFamilyDraftRenderedRecordRollbackRecoveryBlocker::BlockedStagedApplyPlan
        ));
        assert_eq!(review.staged_apply_blockers, dry_run.blockers);
        assert!(!review.staged_apply_executed);
        assert!(!review.dry_run_executed);
        assert!(!review.rollback_executed);
        assert!(!review.recovery_executed);
        assert!(!review.backup_created);
        assert!(!review.restore_executed);
        assert!(!review.real_config_touched);
        assert!(!review.runtime_mutated);
        assert!(!review.hyprctl_reload_run);
        assert!(!review.production_executor_wired);
    }

    assert_rollback_recovery_blocker(
        &accepted,
        &summary,
        |dry_run| dry_run.staged_apply_plan_linked = false,
        StructuredFamilyDraftRenderedRecordRollbackRecoveryBlocker::MissingStagedApplyPlanLink,
    );
    assert_rollback_recovery_blocker(
        &accepted,
        &summary,
        |dry_run| {
            dry_run.blocked_plan_count = 1;
            dry_run
                .blockers
                .push(StructuredFamilyDraftRenderedRecordStagedApplyBlocker::RejectedConfirmation);
        },
        StructuredFamilyDraftRenderedRecordRollbackRecoveryBlocker::BlockedStagedApplyPlan,
    );
    assert_rollback_recovery_blocker(
        &accepted,
        &summary,
        |dry_run| {
            dry_run.real_config_target_policy =
                StructuredFamilyDraftRenderedRecordStagedApplyDryRunStatus::Ready
        },
        StructuredFamilyDraftRenderedRecordRollbackRecoveryBlocker::RealConfigTargetNotAllowed,
    );
    assert_rollback_recovery_blocker(
        &accepted,
        &summary,
        |dry_run| {
            dry_run.persistence_policy =
                StructuredFamilyDraftRenderedRecordStagedApplyDryRunStatus::Ready
        },
        StructuredFamilyDraftRenderedRecordRollbackRecoveryBlocker::PersistenceNotAllowed,
    );
    assert_rollback_recovery_blocker(
        &accepted,
        &summary,
        |dry_run| dry_run.runtime_mutated = true,
        StructuredFamilyDraftRenderedRecordRollbackRecoveryBlocker::RuntimeMutationNotAllowed,
    );
    assert_rollback_recovery_blocker(
        &accepted,
        &summary,
        |dry_run| dry_run.hyprctl_reload_run = true,
        StructuredFamilyDraftRenderedRecordRollbackRecoveryBlocker::HyprlandReloadNotAllowed,
    );
    assert_rollback_recovery_blocker(
        &accepted,
        &summary,
        |dry_run| {
            dry_run.production_executor_policy =
                StructuredFamilyDraftRenderedRecordStagedApplyDryRunStatus::Ready
        },
        StructuredFamilyDraftRenderedRecordRollbackRecoveryBlocker::ProductionExecutorNotAllowed,
    );
}

#[test]
fn all_structured_family_rollback_recovery_reviews_create_final_executor_readiness_audits() {
    for family in StructuredFamilyKind::ALL {
        let summary = diff_review_summary_for_family(family);
        let approval = structured_family_draft_rendered_record_approval_draft(&summary);
        let request = structured_family_draft_rendered_record_confirmation_request(&approval);
        let accepted =
            accept_structured_family_draft_rendered_record_confirmation(&approval, &request);
        let plan = structured_family_draft_rendered_record_staged_apply_plan(&accepted, &summary);
        let dry_run = structured_family_draft_rendered_record_staged_apply_dry_run_report(&plan);
        let review =
            structured_family_draft_rendered_record_staged_apply_rollback_recovery_review(&dry_run);
        let audit = structured_family_draft_rendered_record_final_executor_readiness_audit(&review);

        assert_eq!(audit.family, family);
        assert_eq!(audit.source_draft_count, review.source_draft_count);
        assert_eq!(audit.source_plan_count, review.source_plan_count);
        assert_eq!(audit.review_entry_count, review.review_entry_count);
        assert_eq!(audit.changed_entry_count, review.changed_entry_count);
        assert_eq!(audit.noop_entry_count, review.noop_entry_count);
        assert_eq!(
            audit.raw_fallback_entry_count,
            review.raw_fallback_entry_count
        );
        assert_eq!(
            audit.unsupported_not_proven_entry_count,
            review.unsupported_not_proven_entry_count
        );
        assert_eq!(audit.field_diff_count, review.field_diff_count);
        assert!(audit.rollback_recovery_review_linked);
        assert!(audit.dry_run_report_linked);
        assert!(audit.staged_apply_plan_linked);
        assert_eq!(audit.stage_count, review.stage_count);
        assert_eq!(audit.operation_count, review.operation_count);
        assert_eq!(
            audit.changed_operation_count,
            review.changed_operation_count
        );
        assert_eq!(audit.noop_operation_count, review.noop_operation_count);
        assert_eq!(
            audit.raw_fallback_preservation_operation_count,
            review.raw_fallback_preservation_operation_count
        );
        assert_eq!(
            audit.unsupported_not_proven_preservation_operation_count,
            review.unsupported_not_proven_preservation_operation_count
        );
        assert_eq!(
            audit.recovery_requirement_count,
            review.recovery_requirement_count
        );
        assert_eq!(
            audit.backup_requirement_count,
            review.backup_requirement_count
        );
        assert_eq!(
            audit.restore_requirement_count,
            review.restore_requirement_count
        );
        assert_eq!(audit.blocked_plan_count, review.blocked_plan_count);
        assert!(audit.executor_unavailable_by_design);
        assert!(audit.final_audit_finding_count >= 15);
        assert!(audit.production_blocker_count >= 10);
        assert_eq!(audit.fixture_only_proof_count, 13);
        assert_eq!(
            audit.final_audit_status,
            StructuredFamilyDraftRenderedRecordFinalExecutorReadinessStatus::AuditReady
        );
        assert_eq!(
            audit.rollback_recovery_link_status,
            StructuredFamilyDraftRenderedRecordFinalExecutorReadinessStatus::RollbackRecoveryLinked
        );
        assert_eq!(
            audit.dry_run_link_status,
            StructuredFamilyDraftRenderedRecordFinalExecutorReadinessStatus::DryRunLinked
        );
        assert_eq!(
            audit.staged_apply_plan_link_status,
            StructuredFamilyDraftRenderedRecordFinalExecutorReadinessStatus::StagedApplyLinked
        );
        assert_eq!(
            audit.proof_chain_status,
            StructuredFamilyDraftRenderedRecordFinalExecutorReadinessStatus::ProofChainComplete
        );
        assert_eq!(
            audit.fixture_only_completion_status,
            StructuredFamilyDraftRenderedRecordFinalExecutorReadinessStatus::FixtureOnlyComplete
        );
        assert_eq!(
            audit.production_activation_requirement_status,
            StructuredFamilyDraftRenderedRecordFinalExecutorReadinessStatus::ProductionActivationRequired
        );
        assert_eq!(
            audit.executor_implementation_status,
            StructuredFamilyDraftRenderedRecordFinalExecutorReadinessStatus::ExecutorNotImplemented
        );
        assert_eq!(
            audit.executor_wiring_status,
            StructuredFamilyDraftRenderedRecordFinalExecutorReadinessStatus::ExecutorNotWired
        );
        assert_eq!(
            audit.real_write_boundary_status,
            StructuredFamilyDraftRenderedRecordFinalExecutorReadinessStatus::RealWritesBlocked
        );
        assert_eq!(
            audit.runtime_mutation_boundary_status,
            StructuredFamilyDraftRenderedRecordFinalExecutorReadinessStatus::RuntimeMutationBlocked
        );
        assert_eq!(
            audit.reload_boundary_status,
            StructuredFamilyDraftRenderedRecordFinalExecutorReadinessStatus::ReloadBlocked
        );
        assert_eq!(
            audit.backup_restore_implementation_status,
            StructuredFamilyDraftRenderedRecordFinalExecutorReadinessStatus::BackupRestoreNotImplemented
        );
        assert_eq!(
            audit.blocked_plan_preservation_status,
            StructuredFamilyDraftRenderedRecordFinalExecutorReadinessStatus::BlockedPlanPreserved
        );
        assert_eq!(
            audit.production_readiness_decision,
            StructuredFamilyDraftRenderedRecordFinalExecutorReadinessDecision::NotProductionReady
        );
        assert!(audit.findings.contains(
            &StructuredFamilyDraftRenderedRecordFinalExecutorReadinessFinding::FixturePipelineComplete
        ));
        assert!(audit.findings.contains(
            &StructuredFamilyDraftRenderedRecordFinalExecutorReadinessFinding::ProductionActivationRequired
        ));
        assert!(audit.findings.contains(
            &StructuredFamilyDraftRenderedRecordFinalExecutorReadinessFinding::ExecutorNotImplemented
        ));
        assert!(audit.findings.contains(
            &StructuredFamilyDraftRenderedRecordFinalExecutorReadinessFinding::ExecutorNotWired
        ));
        assert!(audit.findings.contains(
            &StructuredFamilyDraftRenderedRecordFinalExecutorReadinessFinding::UserDecisionRequiredBeforeProduction
        ));
        assert!(audit.findings.contains(
            &StructuredFamilyDraftRenderedRecordFinalExecutorReadinessFinding::Hyprland0554MigrationNotActive
        ));
        if review.raw_fallback_entry_count > 0 {
            assert!(audit.findings.contains(
                &StructuredFamilyDraftRenderedRecordFinalExecutorReadinessFinding::RawFallbackPreservationRequired
            ));
        }
        if review.unsupported_not_proven_entry_count > 0 {
            assert!(audit.findings.contains(
                &StructuredFamilyDraftRenderedRecordFinalExecutorReadinessFinding::UnsupportedNotProvenPreservationRequired
            ));
        }
        assert!(audit
            .summary_text
            .contains("final executor-readiness audit generated: true"));
        assert!(audit
            .summary_text
            .contains("fixture-only pipeline complete: true"));
        assert!(audit
            .summary_text
            .contains("production activation required: true"));
        assert!(audit
            .summary_text
            .contains("production activation approved: false"));
        assert!(audit.summary_text.contains("executor implemented: false"));
        assert!(audit.summary_text.contains("executor wired: false"));
        assert!(audit
            .summary_text
            .contains("real config writes remain blocked"));
        assert!(audit
            .risk_summary
            .contains("fixture-only readiness does not imply production readiness"));
        assert!(!audit.draft_written_to_disk);
        assert!(!audit.final_audit_written_to_disk);
        assert!(!audit.rollback_recovery_review_written_to_disk);
        assert!(!audit.dry_run_report_written_to_disk);
        assert!(!audit.staged_apply_plan_written_to_disk);
        assert!(!audit.staged_apply_executed);
        assert!(!audit.dry_run_executed);
        assert!(!audit.rollback_executed);
        assert!(!audit.recovery_executed);
        assert!(!audit.backup_created);
        assert!(!audit.restore_executed);
        assert!(!audit.executor_implemented);
        assert!(!audit.executor_wired);
        assert!(!audit.production_activation_approved);
        assert!(!audit.rendered_record_written_to_real_config);
        assert!(!audit.real_config_touched);
        assert!(!audit.runtime_mutated);
        assert!(!audit.hyprctl_reload_run);
        assert!(!audit.production_behavior_enabled);
        assert!(!audit.production_executor_wired);
    }
}

#[test]
fn structured_family_final_executor_readiness_audits_preserve_blockers_and_forbidden_attempts() {
    let summary = diff_review_summary_for_family(StructuredFamilyKind::Bind);
    let approval = structured_family_draft_rendered_record_approval_draft(&summary);
    let request = structured_family_draft_rendered_record_confirmation_request(&approval);
    let accepted = accept_structured_family_draft_rendered_record_confirmation(&approval, &request);
    let rejected = reject_structured_family_draft_rendered_record_confirmation(&approval, &request);
    let rejected_plan =
        structured_family_draft_rendered_record_staged_apply_plan(&rejected, &summary);
    let rejected_dry_run =
        structured_family_draft_rendered_record_staged_apply_dry_run_report(&rejected_plan);
    let rejected_review =
        structured_family_draft_rendered_record_staged_apply_rollback_recovery_review(
            &rejected_dry_run,
        );
    let rejected_audit =
        structured_family_draft_rendered_record_final_executor_readiness_audit(&rejected_review);
    assert_eq!(
        rejected_audit.final_audit_status,
        StructuredFamilyDraftRenderedRecordFinalExecutorReadinessStatus::BlockedPlanPreserved
    );
    assert_eq!(
        rejected_audit.production_readiness_decision,
        StructuredFamilyDraftRenderedRecordFinalExecutorReadinessDecision::NotProductionReady
    );
    assert!(rejected_audit.findings.contains(
        &StructuredFamilyDraftRenderedRecordFinalExecutorReadinessFinding::BlockedPlanPreserved
    ));
    assert!(rejected_audit.blockers.contains(
        &StructuredFamilyDraftRenderedRecordFinalExecutorReadinessBlocker::BlockedStagedApplyPlan
    ));

    let plan = structured_family_draft_rendered_record_staged_apply_plan(&accepted, &summary);
    let dry_run = structured_family_draft_rendered_record_staged_apply_dry_run_report(&plan);
    let review =
        structured_family_draft_rendered_record_staged_apply_rollback_recovery_review(&dry_run);

    assert_final_executor_readiness_blocker(
        &review,
        |review| review.dry_run_report_linked = false,
        StructuredFamilyDraftRenderedRecordFinalExecutorReadinessBlocker::MissingDryRunLink,
    );
    assert_final_executor_readiness_blocker(
        &review,
        |review| review.staged_apply_plan_linked = false,
        StructuredFamilyDraftRenderedRecordFinalExecutorReadinessBlocker::MissingStagedApplyPlanLink,
    );
    assert_final_executor_readiness_blocker(
        &review,
        |review| {
            review.real_config_target_policy =
                StructuredFamilyDraftRenderedRecordStagedApplyRollbackRecoveryStatus::Ready
        },
        StructuredFamilyDraftRenderedRecordFinalExecutorReadinessBlocker::RealConfigTargetNotAllowed,
    );
    assert_final_executor_readiness_blocker(
        &review,
        |review| {
            review.persistence_policy =
                StructuredFamilyDraftRenderedRecordStagedApplyRollbackRecoveryStatus::Ready
        },
        StructuredFamilyDraftRenderedRecordFinalExecutorReadinessBlocker::PersistenceNotAllowed,
    );
    assert_final_executor_readiness_blocker(
        &review,
        |review| review.runtime_mutated = true,
        StructuredFamilyDraftRenderedRecordFinalExecutorReadinessBlocker::RuntimeMutationNotAllowed,
    );
    assert_final_executor_readiness_blocker(
        &review,
        |review| review.hyprctl_reload_run = true,
        StructuredFamilyDraftRenderedRecordFinalExecutorReadinessBlocker::HyprlandReloadNotAllowed,
    );
    assert_final_executor_readiness_blocker(
        &review,
        |review| {
            review.production_executor_policy =
                StructuredFamilyDraftRenderedRecordStagedApplyRollbackRecoveryStatus::Ready
        },
        StructuredFamilyDraftRenderedRecordFinalExecutorReadinessBlocker::ProductionExecutorNotAllowed,
    );

    let missing_rollback_link_audit =
        structured_family_draft_rendered_record_final_executor_readiness_audit_with_links(
            &review, false, true, true,
        );
    assert!(missing_rollback_link_audit.blockers.contains(
        &StructuredFamilyDraftRenderedRecordFinalExecutorReadinessBlocker::MissingRollbackRecoveryLink
    ));
    assert_eq!(
        missing_rollback_link_audit.final_audit_status,
        StructuredFamilyDraftRenderedRecordFinalExecutorReadinessStatus::BlockedPlanPreserved
    );

    let mut attempted_activation =
        structured_family_draft_rendered_record_final_executor_readiness_audit(&review);
    attempted_activation.production_activation_approved = true;
    attempted_activation.executor_implemented = true;
    attempted_activation.executor_wired = true;
    let attempted_activation_blockers =
        structured_family_draft_rendered_record_final_executor_readiness_audit_blockers(
            &attempted_activation,
        );
    assert!(attempted_activation_blockers.contains(
        &StructuredFamilyDraftRenderedRecordFinalExecutorReadinessBlocker::ProductionActivationNotAllowed
    ));
    assert!(attempted_activation_blockers.contains(
        &StructuredFamilyDraftRenderedRecordFinalExecutorReadinessBlocker::ExecutorImplementationNotAllowed
    ));
    assert!(attempted_activation_blockers.contains(
        &StructuredFamilyDraftRenderedRecordFinalExecutorReadinessBlocker::ExecutorWiringNotAllowed
    ));
    assert!(
        attempted_activation.production_readiness_decision
            == StructuredFamilyDraftRenderedRecordFinalExecutorReadinessDecision::NotProductionReady
    );
    assert!(attempted_activation
        .findings
        .contains(&StructuredFamilyDraftRenderedRecordFinalExecutorReadinessFinding::ProductionActivationRequired));
    assert!(attempted_activation.findings.contains(
        &StructuredFamilyDraftRenderedRecordFinalExecutorReadinessFinding::ExecutorNotImplemented
    ));
    assert!(attempted_activation.findings.contains(
        &StructuredFamilyDraftRenderedRecordFinalExecutorReadinessFinding::ExecutorNotWired
    ));
}

#[test]
fn structured_family_real_write_activation_requirements_audit_lists_required_gates_only() {
    let audit = structured_family_real_write_activation_requirements_audit();

    assert_eq!(
        audit.activation_status,
        StructuredFamilyRealWriteActivationAuditStatus::ProductionActivationDecisionRequired
    );
    assert_eq!(
        audit.executor_status,
        StructuredFamilyRealWriteActivationAuditStatus::ExecutorNotImplemented
    );
    assert_eq!(
        audit.real_write_boundary_status,
        StructuredFamilyRealWriteActivationAuditStatus::BlockedByDefault
    );
    assert_eq!(
        audit.backup_restore_boundary_status,
        StructuredFamilyRealWriteActivationAuditStatus::BackupRestoreProofMissing
    );
    assert_eq!(
        audit.reload_boundary_status,
        StructuredFamilyRealWriteActivationAuditStatus::HyprlandReloadForbidden
    );
    assert_eq!(
        audit.runtime_mutation_boundary_status,
        StructuredFamilyRealWriteActivationAuditStatus::RuntimeMutationForbidden
    );
    assert_eq!(
        audit.family_ranking_excluded,
        StructuredFamilyRealWriteActivationAuditStatus::FamilyRankingExcludedByUser
    );

    for required in [
        "explicit user production activation decision",
        "explicit activation scope document",
        "real config target policy",
        "source/include target policy",
        "atomic write strategy",
        "pre-write validation",
        "post-write reread validation",
        "Hyprland verify-config or equivalent validation strategy",
        "reload policy",
        "runtime mutation policy",
        "backup creation policy",
        "restore policy",
        "rollback policy",
        "failure recovery policy",
        "audit logging policy",
        "manual confirmation policy",
        "production executor implementation",
        "production executor wiring",
        "production executor tests",
        "UI confirmation gate",
        "dry-run to real-write transition gate",
        "blocked-plan rejection gate",
        "unsupported/not-proven preservation gate",
        "raw fallback preservation gate",
        "version compatibility gate for Hyprland 0.55.2 vs live 0.55.4",
    ] {
        assert!(
            audit.real_write_activation_requirements.contains(&required),
            "missing real-write activation requirement {required}"
        );
    }
    for missing in [
        "no real backup file creation proof",
        "no real backup location policy proof",
        "no backup integrity hash proof",
        "no backup reread proof",
        "no real restore execution proof",
        "no restore target validation proof",
        "no restore reread proof",
        "no rollback file creation proof",
        "no rollback execution proof",
        "no failed-write recovery proof",
        "no interrupted-write recovery proof",
        "no partial-write recovery proof",
        "no post-restore Hyprland validation proof",
        "no post-restore reload/restart policy proof",
        "no user-visible recovery instructions proof",
    ] {
        assert!(
            audit.missing_backup_restore_proof.contains(&missing),
            "missing backup/restore proof item {missing}"
        );
    }
    for gate in [
        "approve entering production activation planning",
        "approve exact real-write activation scope",
        "approve config target path",
        "approve backup location and retention policy",
        "approve restore policy",
        "approve reload policy",
        "approve runtime mutation policy",
        "approve executor implementation",
        "approve executor wiring",
        "approve first real config write",
        "approve fallback behavior for unsupported/not-proven records",
        "approve raw fallback preservation behavior",
        "approve rollback procedure",
        "approve recovery procedure",
        "approve emergency stop procedure",
        "approve whether Hyprland 0.55.4 migration must happen before production activation",
    ] {
        assert!(
            audit.required_user_approval_gates.contains(&gate),
            "missing user approval gate {gate}"
        );
    }

    assert_eq!(
        audit.excluded_by_user,
        vec![
            "which families are safest",
            "which families should stay blocked",
            "family-by-family activation ranking",
            "activation subset recommendation",
        ]
    );
    assert_eq!(audit.real_write_activation_requirements.len(), 25);
    assert_eq!(audit.missing_backup_restore_proof.len(), 15);
    assert_eq!(audit.required_user_approval_gates.len(), 16);
    assert!(!audit.production_activation_approved);
    assert!(!audit.executor_implemented);
    assert!(!audit.executor_wired);
    assert!(!audit.real_write_path_enabled);
    assert!(!audit.real_config_target_enabled);
    assert!(!audit.backup_creation_enabled);
    assert!(!audit.restore_execution_enabled);
    assert!(!audit.rollback_execution_enabled);
    assert!(!audit.hyprctl_reload_enabled);
    assert!(!audit.runtime_mutation_enabled);
    assert!(!audit.first_real_config_write_approved);
    assert_eq!(
        audit.next_recommended_work,
        "Wait for explicit user approval of production activation planning scope before designing any real-write executor."
    );

    let audited_lists = [
        audit.real_write_activation_requirements.as_slice(),
        audit.missing_backup_restore_proof.as_slice(),
        audit.required_user_approval_gates.as_slice(),
    ];
    for item in audited_lists.into_iter().flatten() {
        assert!(
            !item.contains("safest")
                && !item.contains("should stay blocked")
                && !item.contains("activation subset")
                && !item.contains("ranking"),
            "requirements audit must not recommend ranked family activation: {item}"
        );
    }
}

#[test]
fn structured_family_real_write_activation_requirements_has_no_write_reload_or_executor_calls() {
    let structured_family = fs::read_to_string("src/structured_family.rs")
        .expect("structured family source should read");
    let section_start = structured_family
        .find("pub fn structured_family_real_write_activation_requirements_audit")
        .expect("activation requirements audit function should exist");
    let section_end = structured_family[section_start..]
        .find("pub fn structured_family_kind_from_id")
        .map(|offset| section_start + offset)
        .expect("activation requirements audit section should end before kind helper");
    let section = &structured_family[section_start..section_end];

    assert!(section.contains("StructuredFamilyRealWriteActivationRequirementsAudit"));
    for forbidden in [
        "apply_setting_change",
        "write_flow",
        "Command::",
        "fs::write",
        "File::create",
        "write_all",
        "serde_json::to_writer",
        "/home/kyo/.config/hypr/hyprland.conf",
        "~/.config/hypr",
        "production_activation_approved: true",
        "executor_implemented: true",
        "executor_wired: true",
        "real_write_path_enabled: true",
    ] {
        assert!(
            !section.contains(forbidden),
            "activation requirements audit model must not contain {forbidden}"
        );
    }
}

#[test]
fn structured_family_production_activation_planning_scope_is_planning_only() {
    let scope = structured_family_production_activation_planning_scope();

    assert_eq!(
        scope.user_decision,
        "Option B: production activation planning scope only"
    );
    assert!(scope.planning_scope_approved);
    assert!(!scope.implementation_scope_approved);
    assert!(!scope.real_write_scope_approved);
    assert_eq!(
        scope.excluded_by_user,
        vec![
            "family safety ranking",
            "safest-family recommendation",
            "families that should stay blocked",
            "limited activation subset selection",
            "broad activation selection",
            "first family selection",
            "first record selection",
        ]
    );
    for item in [
        "production activation planning document",
        "executor architecture design requirements",
        "backup and restore design requirements",
        "rollback and recovery design requirements",
        "validation evidence design requirements",
        "manual approval checkpoint design",
        "future implementation stop-gate design",
    ] {
        assert!(
            scope.approved_planning_scope.contains(&item),
            "missing approved planning scope item {item}"
        );
    }
    for item in [
        "executor implementation",
        "executor wiring",
        "real config writes",
        "real backup creation",
        "real restore execution",
        "rollback execution",
        "Hyprland reload",
        "runtime mutation",
        "first real config write",
        "family ranking",
        "activation subset selection",
    ] {
        assert!(
            scope.not_approved_scope.contains(&item),
            "missing not-approved scope item {item}"
        );
    }
    assert!(!scope.production_activation_approved);
    assert!(!scope.executor_implemented);
    assert!(!scope.executor_wired);
    assert!(!scope.real_write_path_enabled);
    assert!(!scope.real_config_target_enabled);
    assert!(!scope.backup_creation_enabled);
    assert!(!scope.restore_execution_enabled);
    assert!(!scope.rollback_execution_enabled);
    assert!(!scope.hyprctl_reload_enabled);
    assert!(!scope.runtime_mutation_enabled);
    assert!(!scope.first_real_config_write_approved);
    assert!(scope.family_ranking_excluded);
    assert!(!scope.activation_subset_selected);
    assert_eq!(scope.production_readiness_decision, "not production ready");
    assert_eq!(
        scope.next_recommended_work,
        "Create a planning-only structured-family production activation design document that does not implement or wire an executor."
    );
    assert_eq!(scope.future_implementation_stop_gates.len(), 6);
    assert!(scope
        .future_implementation_stop_gates
        .iter()
        .all(|gate| gate.contains("must stop before")));
}

#[test]
fn structured_family_production_activation_planning_scope_has_no_write_reload_or_executor_calls() {
    let structured_family = fs::read_to_string("src/structured_family.rs")
        .expect("structured family source should read");
    let section_start = structured_family
        .find("pub fn structured_family_production_activation_planning_scope")
        .expect("activation planning scope function should exist");
    let section_end = structured_family[section_start..]
        .find("pub fn structured_family_kind_from_id")
        .map(|offset| section_start + offset)
        .expect("activation planning scope section should end before kind helper");
    let section = &structured_family[section_start..section_end];

    assert!(section.contains("StructuredFamilyProductionActivationPlanningScope"));
    for forbidden in [
        "apply_setting_change",
        "write_flow",
        "Command::",
        "fs::write(",
        "File::create",
        "write_all",
        "serde_json::to_writer",
        "apply_setting_change(",
        "write_flow::",
        "hyprctl reload",
        "gtk::",
        "Button::",
        "connect_clicked",
        "/home/kyo/.config/hypr/hyprland.conf",
        "~/.config/hypr",
        "production_activation_approved: true",
        "executor_implemented: true",
        "executor_wired: true",
        "real_write_path_enabled: true",
    ] {
        assert!(
            !section.contains(forbidden),
            "activation planning scope model must not contain {forbidden}"
        );
    }
}

#[test]
fn structured_family_production_activation_design_is_design_only() {
    let design = structured_family_production_activation_design();

    assert_eq!(
        design.user_decision,
        "Option B: production activation planning scope only"
    );
    assert!(design.design_only);
    assert!(design.planning_scope_approved);
    assert!(!design.implementation_scope_approved);
    assert!(!design.real_write_scope_approved);
    assert_eq!(
        design.excluded_by_user,
        vec![
            "family safety ranking",
            "safest-family recommendation",
            "families that should stay blocked",
            "limited activation subset selection",
            "broad activation selection",
            "first family selection",
            "first record selection",
        ]
    );
    for item in [
        "review-only design artifact",
        "future production activation phases",
        "separation between design, executor implementation, executor wiring, and first real write",
        "no automatic transition from planning to implementation",
        "source/include and duplicate activation remain separate scopes",
    ] {
        assert!(
            design.architecture_design.contains(&item),
            "missing architecture design item {item}"
        );
    }
    for item in [
        "executor is future-only",
        "executor must not be reachable from current UI",
        "executor must require separate implementation approval",
        "executor wiring must require separate wiring approval",
        "executor must reject blocked plans",
        "executor must reject unsupported/not-proven records",
        "executor must preserve raw fallback behavior",
        "executor must require manual confirmation before any future real write",
    ] {
        assert!(
            design.executor_boundary_design.contains(&item),
            "missing executor boundary design item {item}"
        );
    }
    for item in [
        "real config target must be explicitly approved",
        "source/include target policy must be explicitly approved",
        "write target must be reread before write",
        "write target must be validated after write",
        "ambiguous target must block activation",
        "generated/script-managed target must block or require explicit policy",
        "Hyprland 0.55.2 model versus live 0.55.4 compatibility must be resolved before write activation",
    ] {
        assert!(
            design.config_target_policy_design.contains(&item),
            "missing config target policy design item {item}"
        );
    }
    for item in [
        "backup location policy",
        "backup retention policy",
        "backup integrity hash proof",
        "backup reread proof",
        "restore target validation proof",
        "restore reread proof",
        "post-restore Hyprland validation proof",
        "post-restore reload or restart policy approval gate",
    ] {
        assert!(
            design.backup_restore_design.contains(&item),
            "missing backup/restore design item {item}"
        );
    }
    for item in [
        "rollback file policy",
        "failed-write recovery path",
        "interrupted-write recovery path",
        "partial-write recovery path",
        "restore failure recovery path",
        "emergency stop path",
        "user-visible recovery instructions",
    ] {
        assert!(
            design.rollback_recovery_design.contains(&item),
            "missing rollback/recovery design item {item}"
        );
    }
    for item in [
        "pre-write parser validation",
        "pre-write semantic validation",
        "fixture render/reread validation",
        "temporary config validation",
        "Hyprland verify-config or equivalent validation",
        "manual diff review",
        "post-write reread validation",
        "post-write config verification",
        "post-write rollback availability check",
    ] {
        assert!(
            design.validation_sequence_design.contains(&item),
            "missing validation sequence design item {item}"
        );
    }
    for item in [
        "confirm exact activation scope",
        "confirm exact config target",
        "confirm backup location and retention policy",
        "confirm reload policy",
        "confirm runtime mutation policy",
        "confirm rollback/recovery policy",
        "confirm first real config write",
    ] {
        assert!(
            design.manual_confirmation_design.contains(&item),
            "missing manual confirmation design item {item}"
        );
    }
    for item in [
        "planned operation summary",
        "pre-write evidence summary",
        "backup artifact summary",
        "write target summary",
        "validation result summary",
        "manual approval summary",
        "failure and recovery summary",
        "no secret or sensitive data leakage",
    ] {
        assert!(
            design.audit_logging_design.contains(&item),
            "missing audit logging design item {item}"
        );
    }
    for item in [
        "stop before executor implementation",
        "stop before executor wiring",
        "stop before backup execution",
        "stop before restore execution",
        "stop before real config write",
        "stop before reload",
        "stop before runtime mutation",
        "stop on failed preflight",
        "stop on failed validation",
        "stop on ambiguous target",
        "stop on unsupported/not-proven record",
    ] {
        assert!(
            design.emergency_stop_design.contains(&item),
            "missing emergency stop design item {item}"
        );
    }
    assert!(design
        .version_compatibility_design
        .contains(&"Hyprland 0.55.4 migration remains separate explicit scope"));
    assert!(!design.production_activation_approved);
    assert!(!design.executor_implemented);
    assert!(!design.executor_wired);
    assert!(!design.real_write_path_enabled);
    assert!(!design.real_config_target_enabled);
    assert!(!design.backup_creation_enabled);
    assert!(!design.restore_execution_enabled);
    assert!(!design.rollback_execution_enabled);
    assert!(!design.hyprctl_reload_enabled);
    assert!(!design.runtime_mutation_enabled);
    assert!(!design.first_real_config_write_approved);
    assert!(design.family_ranking_excluded);
    assert!(!design.activation_subset_selected);
    assert_eq!(design.production_readiness_decision, "not production ready");
    assert_eq!(
        design.next_recommended_work,
        "Stop for explicit user decision: approve or reject a future executor architecture implementation-planning sprint."
    );
}

#[test]
fn structured_family_production_activation_design_has_no_write_reload_or_executor_calls() {
    let structured_family = fs::read_to_string("src/structured_family.rs")
        .expect("structured family source should read");
    let section_start = structured_family
        .find("pub fn structured_family_production_activation_design")
        .expect("activation design function should exist");
    let section_end = structured_family[section_start..]
        .find("pub fn structured_family_kind_from_id")
        .map(|offset| section_start + offset)
        .expect("activation design section should end before kind helper");
    let section = &structured_family[section_start..section_end];

    assert!(section.contains("StructuredFamilyProductionActivationDesign"));
    for forbidden in [
        "apply_setting_change",
        "write_flow",
        "Command::",
        "fs::write(",
        "File::create",
        "write_all",
        "serde_json::to_writer",
        "/home/kyo/.config/hypr/hyprland.conf",
        "~/.config/hypr",
        "production_activation_approved: true",
        "executor_implemented: true",
        "executor_wired: true",
        "real_write_path_enabled: true",
    ] {
        assert!(
            !section.contains(forbidden),
            "activation design model must not contain {forbidden}"
        );
    }
}

#[test]
fn structured_family_internal_safe_write_architecture_plan_is_planning_only() {
    let plan = structured_family_internal_safe_write_architecture_plan();

    assert_eq!(
        plan.user_decision,
        "Plan the internal safe-write architecture before designing or wiring user-facing controls for real writes."
    );
    assert!(plan.architecture_planning_approved);
    assert!(plan.planning_only);
    assert!(!plan.implementation_scope_approved);
    assert!(!plan.executor_implementation_approved);
    assert!(!plan.executor_wiring_approved);
    assert!(!plan.real_write_scope_approved);
    assert!(!plan.gui_real_write_controls_approved);
    assert_eq!(
        plan.excluded_by_user,
        vec![
            "family safety ranking",
            "safest-family recommendation",
            "families that should stay blocked",
            "limited activation subset selection",
            "broad activation selection",
            "first family selection",
            "first record selection",
            "GUI real-write controls",
            "real apply button wiring",
        ]
    );

    for item in [
        "planning-only internal architecture artifact",
        "separate review-only model from future execution model",
        "separate draft editing from future write approval",
        "separate rendered-record generation from future real config write",
        "separate future executor implementation from future executor wiring",
        "separate future executor wiring from first real config write",
        "keep source/include and duplicate production activation as separate scopes",
    ] {
        assert!(
            plan.safe_write_architecture_plan.contains(&item),
            "missing safe-write architecture item {item}"
        );
    }
    for item in [
        "read current config snapshot",
        "project structured-family records",
        "edit in-memory draft",
        "render candidate record",
        "reread candidate from fixture or temporary config",
        "review diff",
        "collect manual approvals",
        "validate target policy",
        "validate backup policy",
        "validate rollback policy",
        "validate version compatibility",
        "future executor boundary",
        "future first-write stop gate",
        "post-write reread and verification gate",
    ] {
        assert!(
            plan.pipeline_stage_plan.contains(&item),
            "missing pipeline stage item {item}"
        );
    }
    for item in [
        "StructuredFamilyDraft remains in-memory until separate persistence approval",
        "RenderedRecordCandidate represents candidate text only",
        "SafeWritePlan is future-only and not executable in this sprint",
        "SafeWritePreflightReport is future-only evidence summary",
        "SafeWriteBackupPlan is future-only and does not create backups in this sprint",
        "SafeWriteRollbackPlan is future-only and does not create rollback files in this sprint",
        "SafeWriteApprovalState is future-only and requires explicit user approval",
        "SafeWriteExecutionReceipt is future-only and cannot exist until real writes are approved",
    ] {
        assert!(
            plan.boundary_object_plan.contains(&item),
            "missing boundary object item {item}"
        );
    }
    for item in [
        "executor module is future-only",
        "executor is not reachable from current UI",
        "executor is not wired to structured-family records",
        "executor is not called by scalar apply helper",
        "executor is not called by scalar write flow",
        "executor cannot run without explicit implementation approval",
        "executor cannot be wired without explicit wiring approval",
        "executor cannot write without first-real-write approval",
        "executor rejects blocked plans",
        "executor rejects unsupported/not-proven records",
        "executor preserves raw fallback behavior",
    ] {
        assert!(
            plan.executor_boundary_plan.contains(&item),
            "missing executor boundary item {item}"
        );
    }
    for item in [
        "parser validation before future execution",
        "semantic validation before future execution",
        "target reread before future execution",
        "candidate render/reread before future execution",
        "manual diff review before future execution",
        "Hyprland verify-config or equivalent before future execution",
        "post-write reread after future execution",
        "post-write verification after future execution",
        "rollback availability check before future execution",
    ] {
        assert!(
            plan.validation_gate_plan.contains(&item),
            "missing validation gate item {item}"
        );
    }
    assert!(plan
        .backup_restore_gate_plan
        .contains(&"restore execution requires separate approval"));
    assert!(plan
        .rollback_recovery_gate_plan
        .contains(&"rollback execution requires separate approval"));
    assert!(plan
        .audit_log_plan
        .contains(&"sensitive data redaction rule"));
    assert!(plan
        .emergency_stop_plan
        .contains(&"stop before GUI controls"));
    assert!(plan
        .ui_reachability_boundary
        .contains(&"no GTK control that can trigger real writes"));
    assert!(plan
        .future_approval_gates
        .contains(&"approve GUI real-write control design before GUI design starts"));

    assert!(!plan.production_activation_approved);
    assert!(!plan.executor_implemented);
    assert!(!plan.executor_wired);
    assert!(!plan.real_write_path_enabled);
    assert!(!plan.real_config_target_enabled);
    assert!(!plan.backup_creation_enabled);
    assert!(!plan.restore_execution_enabled);
    assert!(!plan.rollback_execution_enabled);
    assert!(!plan.hyprctl_reload_enabled);
    assert!(!plan.runtime_mutation_enabled);
    assert!(!plan.first_real_config_write_approved);
    assert!(!plan.gui_real_write_controls_enabled);
    assert!(plan.family_ranking_excluded);
    assert!(!plan.activation_subset_selected);
    assert_eq!(plan.production_readiness_decision, "not production ready");
    assert_eq!(
        plan.next_recommended_work,
        "Stop for explicit user decision: approve or reject future executor architecture implementation planning."
    );
}

#[test]
fn structured_family_internal_safe_write_architecture_plan_has_no_write_reload_or_ui_wiring() {
    let structured_family = fs::read_to_string("src/structured_family.rs")
        .expect("structured family source should read");
    let section_start = structured_family
        .find("pub fn structured_family_internal_safe_write_architecture_plan")
        .expect("internal safe-write architecture plan function should exist");
    let section_end = structured_family[section_start..]
        .find("pub fn structured_family_kind_from_id")
        .map(|offset| section_start + offset)
        .expect("internal safe-write architecture section should end before kind helper");
    let section = &structured_family[section_start..section_end];

    assert!(section.contains("StructuredFamilyInternalSafeWriteArchitecturePlan"));
    for forbidden in [
        "Command::",
        "fs::write(",
        "File::create",
        "write_all",
        "serde_json::to_writer",
        "/home/kyo/.config/hypr/hyprland.conf",
        "~/.config/hypr",
        "production_activation_approved: true",
        "executor_implemented: true",
        "executor_wired: true",
        "real_write_path_enabled: true",
        "gui_real_write_controls_enabled: true",
    ] {
        assert!(
            !section.contains(forbidden),
            "internal safe-write architecture plan must not contain {forbidden}"
        );
    }
}

#[test]
fn structured_family_executor_architecture_implementation_plan_is_planning_only() {
    let plan = structured_family_executor_architecture_implementation_plan();

    assert_eq!(
        plan.user_decision,
        "Approve future executor architecture implementation planning."
    );
    assert!(plan.executor_architecture_implementation_planning_approved);
    assert!(plan.planning_only);
    assert!(!plan.actual_executor_implementation_approved);
    assert!(!plan.executor_implementation_approved);
    assert!(!plan.executor_wiring_approved);
    assert!(!plan.real_write_scope_approved);
    assert!(!plan.gui_real_write_controls_approved);
    assert_eq!(
        plan.excluded_by_user,
        vec![
            "actual executor implementation",
            "executor wiring",
            "real config writes",
            "GUI real-write controls",
            "real apply button wiring",
            "family safety ranking",
            "safest-family recommendation",
            "families that should stay blocked",
            "limited activation subset selection",
            "broad activation selection",
            "first family selection",
            "first record selection",
        ]
    );

    for item in [
        "future module name only, not created as executable module",
        "future structured_family_safe_write_executor module boundary",
        "future structured_family_safe_write_plan model boundary",
        "future structured_family_safe_write_preflight model boundary",
        "future structured_family_safe_write_backup model boundary",
        "future structured_family_safe_write_rollback model boundary",
        "future structured_family_safe_write_audit model boundary",
        "all future modules require separate implementation approval before creation",
    ] {
        assert!(
            plan.future_module_plan.contains(&item),
            "missing future module item {item}"
        );
    }
    for item in [
        "StructuredFamilySafeWritePlan future type",
        "StructuredFamilySafeWritePreflight future type",
        "StructuredFamilySafeWriteTargetPolicy future type",
        "StructuredFamilySafeWriteBackupPlan future type",
        "StructuredFamilySafeWriteRollbackPlan future type",
        "StructuredFamilySafeWriteApprovalState future type",
        "StructuredFamilySafeWriteExecutionReceipt future type",
        "StructuredFamilySafeWriteAuditRecord future type",
        "StructuredFamilySafeWriteEmergencyStop future type",
    ] {
        assert!(
            plan.future_type_plan.contains(&item),
            "missing future type item {item}"
        );
    }
    for item in [
        "future build_safe_write_plan function",
        "future validate_safe_write_preflight function",
        "future validate_safe_write_target_policy function",
        "future prepare_safe_write_backup_plan function",
        "future prepare_safe_write_rollback_plan function",
        "future verify_manual_approval_state function",
        "future execute_safe_write function",
        "future verify_safe_write_result function",
        "future emit_safe_write_audit_record function",
        "future emergency_stop_reason function",
    ] {
        assert!(
            plan.future_function_plan.contains(&item),
            "missing future function item {item}"
        );
    }
    assert!(plan
        .future_interface_boundary_plan
        .contains(&"executor boundary remains unreachable from current UI"));
    assert!(plan
        .future_interface_boundary_plan
        .contains(&"executor boundary remains disconnected from scalar apply helper"));
    assert!(plan
        .future_interface_boundary_plan
        .contains(&"executor boundary remains disconnected from scalar write flow"));
    assert!(plan
        .future_executor_input_plan
        .contains(&"validated unsupported/not-proven preservation report"));
    assert!(plan
        .future_executor_output_plan
        .contains(&"future emergency stop report on failure"));
    assert!(plan
        .future_validation_object_plan
        .contains(&"Hyprland verify-config or equivalent object"));
    assert!(plan
        .future_backup_restore_object_plan
        .contains(&"restore approval object"));
    assert!(plan
        .future_rollback_recovery_object_plan
        .contains(&"rollback approval object"));
    assert!(plan
        .future_audit_object_plan
        .contains(&"sensitive data redaction rule"));
    assert!(plan
        .future_test_plan
        .contains(&"tests for no GUI real-write controls before GUI approval"));
    assert!(plan
        .future_source_guard_plan
        .contains(&"guard against command runner in planning-only section"));
    assert!(plan
        .ui_reachability_boundary
        .contains(&"no GTK control that can trigger real writes"));
    assert!(plan.future_approval_gates.contains(
        &"approve actual executor implementation scaffold before any executor module is created"
    ));

    assert!(!plan.production_activation_approved);
    assert!(!plan.executor_implemented);
    assert!(!plan.executor_wired);
    assert!(!plan.real_write_path_enabled);
    assert!(!plan.real_config_target_enabled);
    assert!(!plan.backup_creation_enabled);
    assert!(!plan.restore_execution_enabled);
    assert!(!plan.rollback_execution_enabled);
    assert!(!plan.hyprctl_reload_enabled);
    assert!(!plan.runtime_mutation_enabled);
    assert!(!plan.first_real_config_write_approved);
    assert!(!plan.gui_real_write_controls_enabled);
    assert!(plan.family_ranking_excluded);
    assert!(!plan.activation_subset_selected);
    assert_eq!(plan.production_readiness_decision, "not production ready");
    assert_eq!(
        plan.next_recommended_work,
        "Stop for explicit user decision: approve or reject actual executor implementation scaffold."
    );
}

#[test]
fn structured_family_executor_architecture_implementation_plan_has_no_write_reload_or_ui_wiring() {
    let structured_family = fs::read_to_string("src/structured_family.rs")
        .expect("structured family source should read");
    let section_start = structured_family
        .find("pub fn structured_family_executor_architecture_implementation_plan")
        .expect("executor architecture implementation plan function should exist");
    let section_end = structured_family[section_start..]
        .find("pub fn structured_family_kind_from_id")
        .map(|offset| section_start + offset)
        .expect("executor architecture implementation section should end before kind helper");
    let section = &structured_family[section_start..section_end];

    assert!(section.contains("StructuredFamilyExecutorArchitectureImplementationPlan"));
    for forbidden in [
        "apply_setting_change(",
        "write_flow::",
        "Command::",
        "hyprctl reload",
        "fs::write(",
        "File::create",
        "write_all",
        "serde_json::to_writer",
        "/home/kyo/.config/hypr/hyprland.conf",
        "~/.config/hypr",
        "gtk::",
        "Button::",
        "connect_clicked",
        "production_activation_approved: true",
        "actual_executor_implementation_approved: true",
        "executor_implementation_approved: true",
        "executor_implemented: true",
        "executor_wiring_approved: true",
        "executor_wired: true",
        "real_write_path_enabled: true",
        "gui_real_write_controls_enabled: true",
    ] {
        assert!(
            !section.contains(forbidden),
            "executor architecture implementation plan must not contain {forbidden}"
        );
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
fn structured_family_rendered_record_diff_review_has_no_write_reload_or_persistence_calls() {
    let structured_family = fs::read_to_string("src/structured_family.rs")
        .expect("structured family source should read");
    let section_start = structured_family
        .find("pub fn structured_family_draft_rendered_record_diff_review_summary")
        .expect("diff/review summary function should exist");
    let section_end = structured_family[section_start..]
        .find("fn structured_record_from_raw")
        .map(|offset| section_start + offset)
        .expect("diff/review summary section should end before parser helpers");
    let section = &structured_family[section_start..section_end];

    assert!(section.contains("StructuredFamilyDraftRenderedRecordDiffReviewSummary"));
    for forbidden in [
        "apply_setting_change",
        "write_flow",
        "hyprctl reload",
        "Command::",
        "fs::write",
        "File::create",
        "write_all",
        "serde_json::to_writer",
        "/home/kyo/.config/hypr/hyprland.conf",
        "~/.config/hypr",
    ] {
        assert!(
            !section.contains(forbidden),
            "diff/review summary must not contain {forbidden}"
        );
    }
}

#[test]
fn structured_family_rendered_record_approval_confirmation_has_no_write_reload_or_persistence_calls(
) {
    let structured_family = fs::read_to_string("src/structured_family.rs")
        .expect("structured family source should read");
    let section_start = structured_family
        .find("pub fn structured_family_draft_rendered_record_approval_draft")
        .expect("approval/confirmation function should exist");
    let section_end = structured_family[section_start..]
        .find("fn structured_record_from_raw")
        .map(|offset| section_start + offset)
        .expect("approval/confirmation section should end before parser helpers");
    let section = &structured_family[section_start..section_end];

    assert!(section.contains("StructuredFamilyDraftRenderedRecordApprovalDraft"));
    assert!(section.contains("StructuredFamilyDraftRenderedRecordConfirmation"));
    for forbidden in [
        "apply_setting_change",
        "write_flow",
        "hyprctl reload",
        "Command::",
        "fs::write",
        "File::create",
        "write_all",
        "serde_json::to_writer",
        "/home/kyo/.config/hypr/hyprland.conf",
        "~/.config/hypr",
    ] {
        assert!(
            !section.contains(forbidden),
            "approval/confirmation model must not contain {forbidden}"
        );
    }
}

#[test]
fn structured_family_rendered_record_staged_apply_plan_has_no_write_reload_or_persistence_calls() {
    let structured_family = fs::read_to_string("src/structured_family.rs")
        .expect("structured family source should read");
    let section_start = structured_family
        .find("pub fn structured_family_draft_rendered_record_staged_apply_plan")
        .expect("staged apply plan function should exist");
    let section_end = structured_family[section_start..]
        .find("fn structured_record_from_raw")
        .map(|offset| section_start + offset)
        .expect("staged apply section should end before parser helpers");
    let section = &structured_family[section_start..section_end];

    assert!(section.contains("StructuredFamilyDraftRenderedRecordStagedApplyPlan"));
    for forbidden in [
        "apply_setting_change",
        "write_flow",
        "hyprctl reload",
        "Command::",
        "fs::write",
        "File::create",
        "write_all",
        "serde_json::to_writer",
        "/home/kyo/.config/hypr/hyprland.conf",
        "~/.config/hypr",
    ] {
        assert!(
            !section.contains(forbidden),
            "staged apply plan model must not contain {forbidden}"
        );
    }
}

#[test]
fn structured_family_rendered_record_staged_apply_dry_run_has_no_write_reload_or_persistence_calls()
{
    let structured_family = fs::read_to_string("src/structured_family.rs")
        .expect("structured family source should read");
    let section_start = structured_family
        .find("pub fn structured_family_draft_rendered_record_staged_apply_dry_run_report")
        .expect("staged apply dry-run report function should exist");
    let section_end = structured_family[section_start..]
        .find("fn structured_record_from_raw")
        .map(|offset| section_start + offset)
        .expect("dry-run report section should end before parser helpers");
    let section = &structured_family[section_start..section_end];

    assert!(section.contains("StructuredFamilyDraftRenderedRecordStagedApplyDryRunReport"));
    for forbidden in [
        "apply_setting_change",
        "write_flow",
        "hyprctl reload",
        "Command::",
        "fs::write",
        "File::create",
        "write_all",
        "serde_json::to_writer",
        "/home/kyo/.config/hypr/hyprland.conf",
        "~/.config/hypr",
    ] {
        assert!(
            !section.contains(forbidden),
            "staged apply dry-run report model must not contain {forbidden}"
        );
    }
}

#[test]
fn structured_family_rendered_record_staged_apply_rollback_recovery_has_no_write_reload_or_persistence_calls(
) {
    let structured_family = fs::read_to_string("src/structured_family.rs")
        .expect("structured family source should read");
    let section_start = structured_family
        .find(
            "pub fn structured_family_draft_rendered_record_staged_apply_rollback_recovery_review",
        )
        .expect("rollback/recovery review function should exist");
    let section_end = structured_family[section_start..]
        .find("fn structured_record_from_raw")
        .map(|offset| section_start + offset)
        .expect("rollback/recovery review section should end before parser helpers");
    let section = &structured_family[section_start..section_end];

    assert!(
        section.contains("StructuredFamilyDraftRenderedRecordStagedApplyRollbackRecoveryReview")
    );
    for forbidden in [
        "apply_setting_change",
        "write_flow",
        "hyprctl reload",
        "Command::",
        "fs::write",
        "File::create",
        "write_all",
        "serde_json::to_writer",
        "/home/kyo/.config/hypr/hyprland.conf",
        "~/.config/hypr",
    ] {
        assert!(
            !section.contains(forbidden),
            "rollback/recovery review model must not contain {forbidden}"
        );
    }
}

#[test]
fn structured_family_rendered_record_final_executor_readiness_has_no_write_reload_or_persistence_calls(
) {
    let structured_family = fs::read_to_string("src/structured_family.rs")
        .expect("structured family source should read");
    let section_start = structured_family
        .find("pub fn structured_family_draft_rendered_record_final_executor_readiness_audit")
        .expect("final executor-readiness audit function should exist");
    let section_end = structured_family[section_start..]
        .find("fn structured_record_from_raw")
        .map(|offset| section_start + offset)
        .expect("final executor-readiness section should end before parser helpers");
    let section = &structured_family[section_start..section_end];

    assert!(section.contains("StructuredFamilyDraftRenderedRecordFinalExecutorReadinessAudit"));
    for forbidden in [
        "apply_setting_change",
        "write_flow",
        "hyprctl reload",
        "Command::",
        "fs::write",
        "File::create",
        "write_all",
        "serde_json::to_writer",
        "/home/kyo/.config/hypr/hyprland.conf",
        "~/.config/hypr",
    ] {
        assert!(
            !section.contains(forbidden),
            "final executor-readiness audit model must not contain {forbidden}"
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
        "data/reports/structured-family-draft-rendered-record-diff-review.v0.55.2.json",
        "data/reports/structured-family-draft-rendered-record-approval-confirmation.v0.55.2.json",
        "data/reports/structured-family-rendered-record-staged-apply-plan.v0.55.2.json",
        "data/reports/structured-family-rendered-record-staged-apply-dry-run.v0.55.2.json",
        "data/reports/structured-family-rendered-record-staged-apply-rollback-recovery.v0.55.2.json",
        "data/reports/structured-family-rendered-record-final-executor-readiness.v0.55.2.json",
        "data/reports/structured-family-real-write-activation-requirements.v0.55.2.json",
        "data/reports/structured-family-production-activation-planning-scope.v0.55.2.json",
        "data/reports/structured-family-production-activation-design.v0.55.2.json",
        "data/reports/structured-family-internal-safe-write-architecture-plan.v0.55.2.json",
        "data/reports/structured-family-executor-architecture-implementation-plan.v0.55.2.json",
        "data/reports/structured-family-executor-implementation-scaffold.v0.55.2.json",
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
    assert_eq!(
        structured["classification"],
        "blocked_by_executor_wiring_readiness_plan_pending_actual_wiring_scaffold_decision"
    );
    assert_eq!(structured["canContinueNow"], false);
    assert!(structured["currentStatus"]
        .as_str()
        .expect("currentStatus should be text")
        .contains("executor scaffold implemented true"));
    assert!(structured["currentStatus"]
        .as_str()
        .expect("currentStatus should be text")
        .contains("executor wiring readiness model added true"));
    assert_eq!(
        structured["safeNextWork"],
        "stop for explicit user decision on actual executor wiring scaffold"
    );
    assert!(structured["mustNotDo"]
        .as_str()
        .expect("mustNotDo should be text")
        .contains("do not wire executor"));

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
        "Stop for explicit user decision: approve or reject actual executor wiring scaffold."
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
        "Stop for explicit user decision: structured-family production activation remains blocked until the user approves real-write activation scope."
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
        "Stop for explicit user decision: structured-family production activation remains blocked until the user approves real-write activation scope."
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
        "Stop for explicit user decision: structured-family production activation remains blocked until the user approves real-write activation scope."
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
        "Stop for explicit user decision: structured-family production activation remains blocked until the user approves real-write activation scope."
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
        "Stop for explicit user decision: structured-family production activation remains blocked until the user approves real-write activation scope."
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
        "Stop for explicit user decision: structured-family production activation remains blocked until the user approves real-write activation scope."
    );
}

#[test]
fn structured_family_draft_rendered_record_diff_review_report_preserves_fixture_only_policy() {
    let report: serde_json::Value = serde_json::from_str(
        &fs::read_to_string(
            "data/reports/structured-family-draft-rendered-record-diff-review.v0.55.2.json",
        )
        .expect("draft rendered record diff/review report should read"),
    )
    .expect("draft rendered record diff/review report should be valid JSON");
    assert_eq!(
        report["artifactKind"],
        "structured-family-draft-rendered-record-diff-review"
    );
    assert_eq!(report["draftWrittenToDisk"], false);
    assert_eq!(report["diffSummaryWrittenToDisk"], false);
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
            report["diffReviewStatusByFamily"][family],
            "StructuredFamilyDraftRenderedRecordDiffReviewReviewOnly"
        );
        assert_eq!(
            report["reviewSummaryStatusByFamily"][family],
            "StructuredFamilyDraftRenderedRecordReviewSummaryReady"
        );
        assert_eq!(
            report["fieldDiffStatusByFamily"][family],
            "StructuredFamilyDraftRenderedRecordFieldDiffReady"
        );
        assert_eq!(
            report["renderRereadProofLinkStatusByFamily"][family],
            "StructuredFamilyDraftRenderedRecordRenderRereadProofLinked"
        );
        assert_eq!(
            report["changedEntryStatusByFamily"][family],
            "StructuredFamilyDraftRenderedRecordChanged"
        );
        assert_eq!(
            report["noopEntryStatusByFamily"][family],
            "StructuredFamilyDraftRenderedRecordNoop"
        );
        assert_eq!(
            report["rawFallbackReviewStatusByFamily"][family],
            "StructuredFamilyDraftRenderedRecordRawFallbackPreserved"
        );
        assert_eq!(
            report["unsupportedNotProvenReviewStatusByFamily"][family],
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
        "Stop for explicit user decision: structured-family production activation remains blocked until the user approves real-write activation scope."
    );
}

#[test]
fn structured_family_draft_rendered_record_approval_confirmation_report_preserves_fixture_only_policy(
) {
    let report: serde_json::Value = serde_json::from_str(
        &fs::read_to_string(
            "data/reports/structured-family-draft-rendered-record-approval-confirmation.v0.55.2.json",
        )
        .expect("draft rendered record approval/confirmation report should read"),
    )
    .expect("draft rendered record approval/confirmation report should be valid JSON");
    assert_eq!(
        report["artifactKind"],
        "structured-family-draft-rendered-record-approval-confirmation"
    );
    assert_eq!(report["draftWrittenToDisk"], false);
    assert_eq!(report["approvalWrittenToDisk"], false);
    assert_eq!(report["confirmationWrittenToDisk"], false);
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
            report["approvalStatusByFamily"][family],
            "StructuredFamilyDraftRenderedRecordApprovalReviewOnly"
        );
        assert_eq!(
            report["confirmationDraftStatusByFamily"][family],
            "StructuredFamilyDraftRenderedRecordConfirmationDraftReady"
        );
        assert_eq!(
            report["acceptedConfirmationStatusByFamily"][family],
            "StructuredFamilyDraftRenderedRecordConfirmationAcceptedInMemory"
        );
        assert_eq!(
            report["rejectedConfirmationStatusByFamily"][family],
            "StructuredFamilyDraftRenderedRecordConfirmationRejectedInMemory"
        );
        assert_eq!(
            report["invalidConfirmationStatusByFamily"][family],
            "StructuredFamilyDraftRenderedRecordConfirmationInvalidated"
        );
        assert_eq!(
            report["diffReviewLinkStatusByFamily"][family],
            "StructuredFamilyDraftRenderedRecordDiffReviewLinked"
        );
        assert_eq!(
            report["renderRereadProofLinkStatusByFamily"][family],
            "StructuredFamilyDraftRenderedRecordRenderRereadProofLinked"
        );
        assert_eq!(
            report["changedEntriesAcknowledgementStatusByFamily"][family],
            "StructuredFamilyDraftRenderedRecordChangedEntriesAcknowledged"
        );
        assert_eq!(
            report["noopEntriesAcknowledgementStatusByFamily"][family],
            "StructuredFamilyDraftRenderedRecordNoopEntriesAcknowledged"
        );
        assert_eq!(
            report["rawFallbackAcknowledgementStatusByFamily"][family],
            "StructuredFamilyDraftRenderedRecordRawFallbackAcknowledged"
        );
        assert_eq!(
            report["unsupportedNotProvenAcknowledgementStatusByFamily"][family],
            "StructuredFamilyDraftRenderedRecordUnsupportedNotProvenAcknowledged"
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
        assert_eq!(
            report["productionExecutorPolicyByFamily"][family],
            "StructuredFamilyDraftRenderedRecordProductionExecutorForbidden"
        );
    }
    assert_eq!(
        report["nextRecommendedWork"],
        "Stop for explicit user decision: structured-family production activation remains blocked until the user approves real-write activation scope."
    );
}

#[test]
fn structured_family_rendered_record_staged_apply_plan_report_preserves_fixture_only_policy() {
    let report: serde_json::Value = serde_json::from_str(
        &fs::read_to_string(
            "data/reports/structured-family-rendered-record-staged-apply-plan.v0.55.2.json",
        )
        .expect("staged apply plan report should read"),
    )
    .expect("staged apply plan report should be valid JSON");
    assert_eq!(
        report["artifactKind"],
        "structured-family-rendered-record-staged-apply-plan"
    );
    assert_eq!(report["draftWrittenToDisk"], false);
    assert_eq!(report["stagedApplyPlanWrittenToDisk"], false);
    assert_eq!(report["stagedApplyExecuted"], false);
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
            report["stagedApplyStatusByFamily"][family],
            "StructuredFamilyDraftRenderedRecordStagedApplyReviewOnly"
        );
        assert_eq!(
            report["stagedApplyPlanStatusByFamily"][family],
            "StructuredFamilyDraftRenderedRecordStagedApplyPlanReady"
        );
        assert_eq!(
            report["operationStatusByFamily"][family],
            "StructuredFamilyDraftRenderedRecordStagedApplyOperationsReady"
        );
        assert_eq!(
            report["preflightStageStatusByFamily"][family],
            "StructuredFamilyDraftRenderedRecordStagedApplyPreflightReady"
        );
        assert_eq!(
            report["reviewStageStatusByFamily"][family],
            "StructuredFamilyDraftRenderedRecordStagedApplyDiffReviewLinked"
        );
        assert_eq!(
            report["renderPreviewStageStatusByFamily"][family],
            "StructuredFamilyDraftRenderedRecordStagedApplyRenderRereadLinked"
        );
        assert_eq!(
            report["rawFallbackPreservationStageStatusByFamily"][family],
            "StructuredFamilyDraftRenderedRecordStagedApplyRawFallbackPreserved"
        );
        assert_eq!(
            report["unsupportedNotProvenPreservationStageStatusByFamily"][family],
            "StructuredFamilyDraftRenderedRecordStagedApplyUnsupportedNotProvenPreserved"
        );
        assert_eq!(
            report["dryRunOnlyApplyStageStatusByFamily"][family],
            "StructuredFamilyDraftRenderedRecordStagedApplyDryRunOnly"
        );
        assert_eq!(
            report["rollbackPlanStatusByFamily"][family],
            "StructuredFamilyDraftRenderedRecordStagedApplyRollbackPlanReady"
        );
        assert_eq!(
            report["acceptedConfirmationLinkStatusByFamily"][family],
            "StructuredFamilyDraftRenderedRecordStagedApplyConfirmationLinked"
        );
        assert_eq!(
            report["diffReviewLinkStatusByFamily"][family],
            "StructuredFamilyDraftRenderedRecordStagedApplyDiffReviewLinked"
        );
        assert_eq!(
            report["renderRereadProofLinkStatusByFamily"][family],
            "StructuredFamilyDraftRenderedRecordStagedApplyRenderRereadLinked"
        );
        assert_eq!(
            report["blockedPlanStatusByFamily"][family],
            "StructuredFamilyDraftRenderedRecordStagedApplyRejectedConfirmationBlocked; StructuredFamilyDraftRenderedRecordStagedApplyInvalidConfirmationBlocked"
        );
        assert_eq!(
            report["executorAvailabilityStatusByFamily"][family],
            "StructuredFamilyDraftRenderedRecordProductionExecutorForbidden"
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
        assert_eq!(
            report["productionExecutorPolicyByFamily"][family],
            "StructuredFamilyDraftRenderedRecordProductionExecutorForbidden"
        );
    }
    assert_eq!(
        report["nextRecommendedWork"],
        "Stop for explicit user decision: structured-family production activation remains blocked until the user approves real-write activation scope."
    );
}

#[test]
fn structured_family_rendered_record_staged_apply_dry_run_report_preserves_fixture_only_policy() {
    let report: serde_json::Value = serde_json::from_str(
        &fs::read_to_string(
            "data/reports/structured-family-rendered-record-staged-apply-dry-run.v0.55.2.json",
        )
        .expect("staged apply dry-run report should read"),
    )
    .expect("staged apply dry-run report should be valid JSON");
    assert_eq!(
        report["artifactKind"],
        "structured-family-rendered-record-staged-apply-dry-run"
    );
    assert_eq!(report["draftWrittenToDisk"], false);
    assert_eq!(report["dryRunReportWrittenToDisk"], false);
    assert_eq!(report["stagedApplyPlanWrittenToDisk"], false);
    assert_eq!(report["stagedApplyExecuted"], false);
    assert_eq!(report["dryRunExecuted"], false);
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
            report["dryRunStatusByFamily"][family],
            "StructuredFamilyDraftRenderedRecordStagedApplyDryRunReviewOnly"
        );
        assert_eq!(
            report["dryRunReportStatusByFamily"][family],
            "StructuredFamilyDraftRenderedRecordStagedApplyDryRunReportReady"
        );
        assert_eq!(
            report["stagedApplyPlanLinkStatusByFamily"][family],
            "StructuredFamilyDraftRenderedRecordStagedApplyDryRunPlanLinked"
        );
        assert_eq!(
            report["stageSummaryStatusByFamily"][family],
            "StructuredFamilyDraftRenderedRecordStagedApplyDryRunStagesSummarized"
        );
        assert_eq!(
            report["operationSummaryStatusByFamily"][family],
            "StructuredFamilyDraftRenderedRecordStagedApplyDryRunOperationsSummarized"
        );
        assert_eq!(
            report["changedOperationSummaryStatusByFamily"][family],
            "StructuredFamilyDraftRenderedRecordStagedApplyDryRunChangedOperationsSummarized"
        );
        assert_eq!(
            report["noopOperationSummaryStatusByFamily"][family],
            "StructuredFamilyDraftRenderedRecordStagedApplyDryRunNoopOperationsSummarized"
        );
        assert_eq!(
            report["rawFallbackPreservationSummaryStatusByFamily"][family],
            "StructuredFamilyDraftRenderedRecordStagedApplyDryRunRawFallbackPreserved"
        );
        assert_eq!(
            report["unsupportedNotProvenPreservationSummaryStatusByFamily"][family],
            "StructuredFamilyDraftRenderedRecordStagedApplyDryRunUnsupportedNotProvenPreserved"
        );
        assert_eq!(
            report["blockedPlanSummaryStatusByFamily"][family],
            "StructuredFamilyDraftRenderedRecordStagedApplyDryRunBlockedPlanSummarized"
        );
        assert_eq!(
            report["executorAvailabilityStatusByFamily"][family],
            "StructuredFamilyDraftRenderedRecordStagedApplyDryRunExecutorUnavailable"
        );
        assert_eq!(
            report["dryRunExecutionStatusByFamily"][family],
            "StructuredFamilyDraftRenderedRecordStagedApplyDryRunNotExecuted"
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
        assert_eq!(
            report["productionExecutorPolicyByFamily"][family],
            "StructuredFamilyDraftRenderedRecordProductionExecutorForbidden"
        );
    }
    assert_eq!(
        report["nextRecommendedWork"],
        "Stop for explicit user decision: structured-family production activation remains blocked until the user approves real-write activation scope."
    );
}

#[test]
fn structured_family_rendered_record_staged_apply_rollback_recovery_report_preserves_fixture_only_policy(
) {
    let report: serde_json::Value = serde_json::from_str(
        &fs::read_to_string(
            "data/reports/structured-family-rendered-record-staged-apply-rollback-recovery.v0.55.2.json",
        )
        .expect("staged apply rollback/recovery report should read"),
    )
    .expect("staged apply rollback/recovery report should be valid JSON");
    assert_eq!(
        report["artifactKind"],
        "structured-family-rendered-record-staged-apply-rollback-recovery"
    );
    assert_eq!(report["draftWrittenToDisk"], false);
    assert_eq!(report["rollbackRecoveryReviewWrittenToDisk"], false);
    assert_eq!(report["dryRunReportWrittenToDisk"], false);
    assert_eq!(report["stagedApplyPlanWrittenToDisk"], false);
    assert_eq!(report["stagedApplyExecuted"], false);
    assert_eq!(report["dryRunExecuted"], false);
    assert_eq!(report["rollbackExecuted"], false);
    assert_eq!(report["recoveryExecuted"], false);
    assert_eq!(report["backupCreated"], false);
    assert_eq!(report["restoreExecuted"], false);
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
            report["rollbackRecoveryStatusByFamily"][family],
            "StructuredFamilyDraftRenderedRecordStagedApplyRollbackRecoveryReviewOnly"
        );
        assert_eq!(
            report["rollbackRecoveryReviewStatusByFamily"][family],
            "StructuredFamilyDraftRenderedRecordStagedApplyRollbackRecoveryReviewReady"
        );
        assert_eq!(
            report["dryRunReportLinkStatusByFamily"][family],
            "StructuredFamilyDraftRenderedRecordStagedApplyRollbackRecoveryDryRunLinked"
        );
        assert_eq!(
            report["stagedApplyPlanLinkStatusByFamily"][family],
            "StructuredFamilyDraftRenderedRecordStagedApplyRollbackRecoveryPlanLinked"
        );
        assert_eq!(
            report["rollbackPlanSummaryStatusByFamily"][family],
            "StructuredFamilyDraftRenderedRecordStagedApplyRollbackRecoveryRollbackPlanSummarized"
        );
        assert_eq!(
            report["recoveryPathSummaryStatusByFamily"][family],
            "StructuredFamilyDraftRenderedRecordStagedApplyRollbackRecoveryRecoveryPathSummarized"
        );
        assert_eq!(
            report["backupRequirementStatusByFamily"][family],
            "StructuredFamilyDraftRenderedRecordStagedApplyRollbackRecoveryBackupRequirementReady"
        );
        assert_eq!(
            report["restoreRequirementStatusByFamily"][family],
            "StructuredFamilyDraftRenderedRecordStagedApplyRollbackRecoveryRestoreRequirementReady"
        );
        assert_eq!(
            report["blockedPlanPreservationStatusByFamily"][family],
            "StructuredFamilyDraftRenderedRecordStagedApplyRollbackRecoveryBlockedPlanPreserved"
        );
        assert_eq!(
            report["executorAvailabilityStatusByFamily"][family],
            "StructuredFamilyDraftRenderedRecordStagedApplyRollbackRecoveryExecutorUnavailable"
        );
        assert_eq!(
            report["executionStatusByFamily"][family],
            "StructuredFamilyDraftRenderedRecordStagedApplyRollbackRecoveryNotExecuted"
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
        assert_eq!(
            report["productionExecutorPolicyByFamily"][family],
            "StructuredFamilyDraftRenderedRecordProductionExecutorForbidden"
        );
    }
    assert_eq!(
        report["nextRecommendedWork"],
        "Stop for explicit user decision: structured-family production activation remains blocked until the user approves real-write activation scope."
    );
}

#[test]
fn structured_family_rendered_record_final_executor_readiness_report_preserves_fixture_only_policy()
{
    let report: serde_json::Value = serde_json::from_str(
        &fs::read_to_string(
            "data/reports/structured-family-rendered-record-final-executor-readiness.v0.55.2.json",
        )
        .expect("final executor-readiness report should read"),
    )
    .expect("final executor-readiness report should be valid JSON");
    assert_eq!(
        report["artifactKind"],
        "structured-family-rendered-record-final-executor-readiness"
    );
    assert_eq!(report["draftWrittenToDisk"], false);
    assert_eq!(report["finalAuditWrittenToDisk"], false);
    assert_eq!(report["rollbackRecoveryReviewWrittenToDisk"], false);
    assert_eq!(report["dryRunReportWrittenToDisk"], false);
    assert_eq!(report["stagedApplyPlanWrittenToDisk"], false);
    assert_eq!(report["stagedApplyExecuted"], false);
    assert_eq!(report["dryRunExecuted"], false);
    assert_eq!(report["rollbackExecuted"], false);
    assert_eq!(report["recoveryExecuted"], false);
    assert_eq!(report["backupCreated"], false);
    assert_eq!(report["restoreExecuted"], false);
    assert_eq!(report["executorImplemented"], false);
    assert_eq!(report["executorWired"], false);
    assert_eq!(report["productionActivationApproved"], false);
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
            report["finalExecutorReadinessStatusByFamily"][family],
            "StructuredFamilyDraftRenderedRecordFinalExecutorReadinessReviewOnly"
        );
        assert_eq!(
            report["finalAuditStatusByFamily"][family],
            "StructuredFamilyDraftRenderedRecordFinalExecutorReadinessAuditReady"
        );
        assert_eq!(
            report["rollbackRecoveryLinkStatusByFamily"][family],
            "StructuredFamilyDraftRenderedRecordFinalExecutorReadinessRollbackRecoveryLinked"
        );
        assert_eq!(
            report["dryRunLinkStatusByFamily"][family],
            "StructuredFamilyDraftRenderedRecordFinalExecutorReadinessDryRunLinked"
        );
        assert_eq!(
            report["stagedApplyPlanLinkStatusByFamily"][family],
            "StructuredFamilyDraftRenderedRecordFinalExecutorReadinessStagedApplyLinked"
        );
        assert_eq!(
            report["proofChainStatusByFamily"][family],
            "StructuredFamilyDraftRenderedRecordFinalExecutorReadinessProofChainComplete"
        );
        assert_eq!(
            report["fixtureOnlyCompletionStatusByFamily"][family],
            "StructuredFamilyDraftRenderedRecordFinalExecutorReadinessFixtureOnlyComplete"
        );
        assert_eq!(
            report["productionActivationRequirementStatusByFamily"][family],
            "StructuredFamilyDraftRenderedRecordFinalExecutorReadinessProductionActivationRequired"
        );
        assert_eq!(
            report["executorImplementationStatusByFamily"][family],
            "StructuredFamilyDraftRenderedRecordFinalExecutorReadinessExecutorNotImplemented"
        );
        assert_eq!(
            report["executorWiringStatusByFamily"][family],
            "StructuredFamilyDraftRenderedRecordFinalExecutorReadinessExecutorNotWired"
        );
        assert_eq!(
            report["realWriteBoundaryStatusByFamily"][family],
            "StructuredFamilyDraftRenderedRecordFinalExecutorReadinessRealWritesBlocked"
        );
        assert_eq!(
            report["runtimeMutationBoundaryStatusByFamily"][family],
            "StructuredFamilyDraftRenderedRecordFinalExecutorReadinessRuntimeMutationBlocked"
        );
        assert_eq!(
            report["reloadBoundaryStatusByFamily"][family],
            "StructuredFamilyDraftRenderedRecordFinalExecutorReadinessReloadBlocked"
        );
        assert_eq!(
            report["backupRestoreImplementationStatusByFamily"][family],
            "StructuredFamilyDraftRenderedRecordFinalExecutorReadinessBackupRestoreNotImplemented"
        );
        assert_eq!(
            report["blockedPlanPreservationStatusByFamily"][family],
            "StructuredFamilyDraftRenderedRecordFinalExecutorReadinessBlockedPlanPreserved"
        );
        assert_eq!(
            report["productionReadinessDecisionByFamily"][family],
            "StructuredFamilyDraftRenderedRecordFinalExecutorReadinessNotProductionReady"
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
        assert_eq!(
            report["productionExecutorPolicyByFamily"][family],
            "StructuredFamilyDraftRenderedRecordProductionExecutorForbidden"
        );
    }
    assert_eq!(
        report["nextRecommendedWork"],
        "Wait for explicit user approval of production activation planning scope before designing any real-write executor."
    );
}

#[test]
fn structured_family_real_write_activation_requirements_report_preserves_non_approval_state() {
    let report: serde_json::Value = serde_json::from_str(
        &fs::read_to_string(
            "data/reports/structured-family-real-write-activation-requirements.v0.55.2.json",
        )
        .expect("real-write activation requirements report should read"),
    )
    .expect("real-write activation requirements report should be valid JSON");
    assert_eq!(
        report["artifactKind"],
        "structured-family-real-write-activation-requirements"
    );
    assert_eq!(
        report["excludedByUser"],
        serde_json::json!([
            "which families are safest",
            "which families should stay blocked",
            "family-by-family activation ranking",
            "activation subset recommendation"
        ])
    );
    assert_eq!(
        report["activationStatus"],
        "StructuredFamilyProductionActivationDecisionRequired"
    );
    assert_eq!(
        report["executorStatus"],
        "StructuredFamilyRealWriteExecutorNotImplemented"
    );
    assert_eq!(
        report["realWriteBoundaryStatus"],
        "StructuredFamilyRealWriteBlockedByDefault"
    );
    assert_eq!(
        report["backupRestoreBoundaryStatus"],
        "StructuredFamilyBackupRestoreProofMissing"
    );
    assert_eq!(
        report["reloadBoundaryStatus"],
        "StructuredFamilyHyprlandReloadForbidden"
    );
    assert_eq!(
        report["runtimeMutationBoundaryStatus"],
        "StructuredFamilyRuntimeMutationForbidden"
    );
    assert_eq!(
        report["familyRankingExcluded"],
        "StructuredFamilyFamilyRankingExcludedByUser"
    );
    assert_eq!(report["productionActivationApproved"], false);
    assert_eq!(report["executorImplemented"], false);
    assert_eq!(report["executorWired"], false);
    assert_eq!(report["realWritePathEnabled"], false);
    assert_eq!(report["realConfigTargetEnabled"], false);
    assert_eq!(report["backupCreationEnabled"], false);
    assert_eq!(report["restoreExecutionEnabled"], false);
    assert_eq!(report["rollbackExecutionEnabled"], false);
    assert_eq!(report["hyprctlReloadEnabled"], false);
    assert_eq!(report["runtimeMutationEnabled"], false);
    assert_eq!(report["firstRealConfigWriteApproved"], false);
    assert_eq!(
        report["realWriteActivationRequirements"]
            .as_array()
            .expect("requirements should be array")
            .len(),
        25
    );
    assert_eq!(
        report["missingBackupRestoreProof"]
            .as_array()
            .expect("missing proof should be array")
            .len(),
        15
    );
    assert_eq!(
        report["requiredUserApprovalGates"]
            .as_array()
            .expect("approval gates should be array")
            .len(),
        16
    );
    assert!(report["realWriteActivationRequirements"]
        .as_array()
        .expect("requirements should be array")
        .iter()
        .any(|item| item.as_str() == Some("explicit user production activation decision")));
    assert!(report["missingBackupRestoreProof"]
        .as_array()
        .expect("missing proof should be array")
        .iter()
        .any(|item| item.as_str() == Some("no real backup file creation proof")));
    assert!(report["requiredUserApprovalGates"]
        .as_array()
        .expect("approval gates should be array")
        .iter()
        .any(|item| item.as_str() == Some("approve first real config write")));

    for key in [
        "realWriteActivationRequirements",
        "missingBackupRestoreProof",
        "requiredUserApprovalGates",
    ] {
        for item in report[key].as_array().expect("audit list should be array") {
            let text = item.as_str().expect("audit list item should be string");
            assert!(
                !text.contains("safest")
                    && !text.contains("should stay blocked")
                    && !text.contains("activation subset")
                    && !text.contains("ranking"),
                "requirements report must not recommend ranked family activation: {text}"
            );
        }
    }
    assert_eq!(
        report["nextRecommendedWork"],
        "Wait for explicit user approval of production activation planning scope before designing any real-write executor."
    );
}

#[test]
fn structured_family_production_activation_planning_scope_report_is_planning_only() {
    let report: serde_json::Value = serde_json::from_str(
        &fs::read_to_string(
            "data/reports/structured-family-production-activation-planning-scope.v0.55.2.json",
        )
        .expect("production activation planning scope report should read"),
    )
    .expect("production activation planning scope report should be valid JSON");

    assert_eq!(
        report["artifactKind"],
        "structured-family-production-activation-planning-scope"
    );
    assert_eq!(
        report["userDecision"],
        "Option B: production activation planning scope only"
    );
    assert_eq!(report["planningScopeApproved"], true);
    assert_eq!(report["implementationScopeApproved"], false);
    assert_eq!(report["realWriteScopeApproved"], false);
    assert_eq!(
        report["excludedByUser"],
        serde_json::json!([
            "family safety ranking",
            "safest-family recommendation",
            "families that should stay blocked",
            "limited activation subset selection",
            "broad activation selection",
            "first family selection",
            "first record selection"
        ])
    );
    for key in [
        "productionActivationApproved",
        "executorImplemented",
        "executorWired",
        "realWritePathEnabled",
        "realConfigTargetEnabled",
        "backupCreationEnabled",
        "restoreExecutionEnabled",
        "rollbackExecutionEnabled",
        "hyprctlReloadEnabled",
        "runtimeMutationEnabled",
        "firstRealConfigWriteApproved",
        "activationSubsetSelected",
    ] {
        assert_eq!(report[key], false, "{key} should remain false");
    }
    assert_eq!(report["familyRankingExcluded"], true);
    assert_eq!(
        report["productionReadinessDecision"],
        "not production ready"
    );

    for item in [
        "executor implementation",
        "executor wiring",
        "real config writes",
        "real backup creation",
        "real restore execution",
        "rollback execution",
        "Hyprland reload",
        "runtime mutation",
        "first real config write",
        "family ranking",
        "activation subset selection",
    ] {
        assert!(
            report["notApprovedScope"]
                .as_array()
                .expect("notApprovedScope should be array")
                .iter()
                .any(|value| value.as_str() == Some(item)),
            "notApprovedScope missing {item}"
        );
    }
    assert_eq!(
        report["futureImplementationStopGates"]
            .as_array()
            .expect("stop gates should be array")
            .len(),
        6
    );
    assert!(report["futureImplementationStopGates"]
        .as_array()
        .expect("stop gates should be array")
        .iter()
        .all(|gate| gate
            .as_str()
            .expect("stop gate should be string")
            .contains("must stop before")));
    assert_eq!(
        report["nextRecommendedWork"],
        "Create a planning-only structured-family production activation design document that does not implement or wire an executor."
    );
}

#[test]
fn structured_family_production_activation_design_report_is_design_only() {
    let report: serde_json::Value = serde_json::from_str(
        &fs::read_to_string(
            "data/reports/structured-family-production-activation-design.v0.55.2.json",
        )
        .expect("production activation design report should read"),
    )
    .expect("production activation design report should be valid JSON");

    assert_eq!(
        report["artifactKind"],
        "structured-family-production-activation-design"
    );
    assert_eq!(
        report["userDecision"],
        "Option B: production activation planning scope only"
    );
    assert_eq!(report["designOnly"], true);
    assert_eq!(report["planningScopeApproved"], true);
    assert_eq!(report["implementationScopeApproved"], false);
    assert_eq!(report["realWriteScopeApproved"], false);
    assert_eq!(
        report["excludedByUser"],
        serde_json::json!([
            "family safety ranking",
            "safest-family recommendation",
            "families that should stay blocked",
            "limited activation subset selection",
            "broad activation selection",
            "first family selection",
            "first record selection"
        ])
    );
    for key in [
        "productionActivationApproved",
        "executorImplemented",
        "executorWired",
        "realWritePathEnabled",
        "realConfigTargetEnabled",
        "backupCreationEnabled",
        "restoreExecutionEnabled",
        "rollbackExecutionEnabled",
        "hyprctlReloadEnabled",
        "runtimeMutationEnabled",
        "firstRealConfigWriteApproved",
        "activationSubsetSelected",
    ] {
        assert_eq!(report[key], false, "{key} should remain false");
    }
    assert_eq!(report["familyRankingExcluded"], true);
    assert_eq!(
        report["productionReadinessDecision"],
        "not production ready"
    );

    for key in [
        "architectureDesign",
        "executorBoundaryDesign",
        "configTargetPolicyDesign",
        "backupRestoreDesign",
        "rollbackRecoveryDesign",
        "validationSequenceDesign",
        "manualConfirmationDesign",
        "auditLoggingDesign",
        "emergencyStopDesign",
        "futureImplementationStopGates",
    ] {
        assert!(
            !report[key]
                .as_array()
                .expect("design field should be array")
                .is_empty(),
            "{key} should be populated"
        );
    }
    assert!(report["architectureDesign"]
        .as_array()
        .expect("architectureDesign should be array")
        .iter()
        .any(|value| value.as_str() == Some("review-only design artifact")));
    assert!(report["executorBoundaryDesign"]
        .as_array()
        .expect("executorBoundaryDesign should be array")
        .iter()
        .any(|value| value.as_str() == Some("executor is future-only")));
    assert!(report["futureImplementationStopGates"]
        .as_array()
        .expect("futureImplementationStopGates should be array")
        .iter()
        .any(|gate| gate
            .as_str()
            .expect("stop gate should be text")
            .contains("must stop before implementing an executor")));
    assert_eq!(
        report["nextRecommendedWork"],
        "Stop for explicit user decision: approve or reject a future executor architecture implementation-planning sprint."
    );
}

#[test]
fn structured_family_internal_safe_write_architecture_plan_report_is_planning_only() {
    let report: serde_json::Value = serde_json::from_str(
        &fs::read_to_string(
            "data/reports/structured-family-internal-safe-write-architecture-plan.v0.55.2.json",
        )
        .expect("internal safe-write architecture plan report should read"),
    )
    .expect("internal safe-write architecture plan report should be valid JSON");

    assert_eq!(
        report["artifactKind"],
        "structured-family-internal-safe-write-architecture-plan"
    );
    assert_eq!(report["architecturePlanningApproved"], true);
    assert_eq!(report["planningOnly"], true);
    assert_eq!(report["implementationScopeApproved"], false);
    assert_eq!(report["executorImplementationApproved"], false);
    assert_eq!(report["executorWiringApproved"], false);
    assert_eq!(report["realWriteScopeApproved"], false);
    assert_eq!(report["guiRealWriteControlsApproved"], false);
    assert_eq!(
        report["excludedByUser"],
        serde_json::json!([
            "family safety ranking",
            "safest-family recommendation",
            "families that should stay blocked",
            "limited activation subset selection",
            "broad activation selection",
            "first family selection",
            "first record selection",
            "GUI real-write controls",
            "real apply button wiring"
        ])
    );
    for key in [
        "productionActivationApproved",
        "executorImplemented",
        "executorWired",
        "realWritePathEnabled",
        "realConfigTargetEnabled",
        "backupCreationEnabled",
        "restoreExecutionEnabled",
        "rollbackExecutionEnabled",
        "hyprctlReloadEnabled",
        "runtimeMutationEnabled",
        "firstRealConfigWriteApproved",
        "guiRealWriteControlsEnabled",
        "activationSubsetSelected",
    ] {
        assert_eq!(report[key], false, "{key} should remain false");
    }
    assert_eq!(report["familyRankingExcluded"], true);
    assert_eq!(
        report["productionReadinessDecision"],
        "not production ready"
    );

    for key in [
        "safeWriteArchitecturePlan",
        "pipelineStagePlan",
        "boundaryObjectPlan",
        "executorBoundaryPlan",
        "validationGatePlan",
        "backupRestoreGatePlan",
        "rollbackRecoveryGatePlan",
        "auditLogPlan",
        "emergencyStopPlan",
        "uiReachabilityBoundary",
        "futureApprovalGates",
    ] {
        assert!(
            !report[key]
                .as_array()
                .expect("planning field should be array")
                .is_empty(),
            "{key} should be populated"
        );
    }
    assert!(report["safeWriteArchitecturePlan"]
        .as_array()
        .expect("safeWriteArchitecturePlan should be array")
        .iter()
        .any(|value| value.as_str() == Some("planning-only internal architecture artifact")));
    assert!(report["uiReachabilityBoundary"]
        .as_array()
        .expect("uiReachabilityBoundary should be array")
        .iter()
        .any(|value| value.as_str() == Some("no GTK control that can trigger real writes")));
    let next = report["nextRecommendedWork"]
        .as_str()
        .expect("nextRecommendedWork should be text");
    assert_eq!(
        next,
        "Stop for explicit user decision: approve or reject future executor architecture implementation planning."
    );
    assert!(!next.contains("implementing an executor"));
    assert!(!next.contains("GUI controls"));
}

#[test]
fn structured_family_executor_architecture_implementation_plan_report_is_planning_only() {
    let report: serde_json::Value = serde_json::from_str(
        &fs::read_to_string(
            "data/reports/structured-family-executor-architecture-implementation-plan.v0.55.2.json",
        )
        .expect("executor architecture implementation plan report should read"),
    )
    .expect("executor architecture implementation plan report should be valid JSON");

    assert_eq!(
        report["artifactKind"],
        "structured-family-executor-architecture-implementation-plan"
    );
    assert_eq!(
        report["executorArchitectureImplementationPlanningApproved"],
        true
    );
    assert_eq!(report["planningOnly"], true);
    assert_eq!(report["actualExecutorImplementationApproved"], false);
    assert_eq!(report["executorImplementationApproved"], false);
    assert_eq!(report["executorWiringApproved"], false);
    assert_eq!(report["realWriteScopeApproved"], false);
    assert_eq!(report["guiRealWriteControlsApproved"], false);
    assert_eq!(
        report["excludedByUser"],
        serde_json::json!([
            "actual executor implementation",
            "executor wiring",
            "real config writes",
            "GUI real-write controls",
            "real apply button wiring",
            "family safety ranking",
            "safest-family recommendation",
            "families that should stay blocked",
            "limited activation subset selection",
            "broad activation selection",
            "first family selection",
            "first record selection"
        ])
    );
    for key in [
        "productionActivationApproved",
        "executorImplemented",
        "executorWired",
        "realWritePathEnabled",
        "realConfigTargetEnabled",
        "backupCreationEnabled",
        "restoreExecutionEnabled",
        "rollbackExecutionEnabled",
        "hyprctlReloadEnabled",
        "runtimeMutationEnabled",
        "firstRealConfigWriteApproved",
        "guiRealWriteControlsEnabled",
        "activationSubsetSelected",
    ] {
        assert_eq!(report[key], false, "{key} should remain false");
    }
    assert_eq!(report["familyRankingExcluded"], true);
    assert_eq!(
        report["productionReadinessDecision"],
        "not production ready"
    );

    for key in [
        "futureModulePlan",
        "futureTypePlan",
        "futureFunctionPlan",
        "futureInterfaceBoundaryPlan",
        "futureExecutorInputPlan",
        "futureExecutorOutputPlan",
        "futureValidationObjectPlan",
        "futureBackupRestoreObjectPlan",
        "futureRollbackRecoveryObjectPlan",
        "futureAuditObjectPlan",
        "futureTestPlan",
        "futureSourceGuardPlan",
        "uiReachabilityBoundary",
        "futureApprovalGates",
    ] {
        assert!(
            !report[key]
                .as_array()
                .expect("planning field should be array")
                .is_empty(),
            "{key} should be populated"
        );
    }
    assert!(report["futureModulePlan"]
        .as_array()
        .expect("futureModulePlan should be array")
        .iter()
        .any(|value| value.as_str()
            == Some("future structured_family_safe_write_executor module boundary")));
    assert!(report["futureFunctionPlan"]
        .as_array()
        .expect("futureFunctionPlan should be array")
        .iter()
        .any(|value| value.as_str() == Some("future execute_safe_write function")));
    assert!(report["futureInterfaceBoundaryPlan"]
        .as_array()
        .expect("futureInterfaceBoundaryPlan should be array")
        .iter()
        .any(|value| value.as_str()
            == Some("executor boundary remains disconnected from apply_setting_change")));
    assert!(report["futureSourceGuardPlan"]
        .as_array()
        .expect("futureSourceGuardPlan should be array")
        .iter()
        .any(|value| value.as_str()
            == Some("guard against connect_clicked real-write wiring in planning-only section")));
    assert!(report["uiReachabilityBoundary"]
        .as_array()
        .expect("uiReachabilityBoundary should be array")
        .iter()
        .any(|value| value.as_str() == Some("no GTK control that can trigger real writes")));

    let next = report["nextRecommendedWork"]
        .as_str()
        .expect("nextRecommendedWork should be text");
    assert_eq!(
        next,
        "Stop for explicit user decision: approve or reject actual executor implementation scaffold."
    );
    assert!(!next.contains("automatically"));
    assert!(!next.contains("GUI controls"));
}

fn diff_review_summary_for_family(
    family: StructuredFamilyKind,
) -> hyprland_settings::structured_family::StructuredFamilyDraftRenderedRecordDiffReviewSummary {
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
    let proof = prove_structured_family_draft_rendered_record_render_reread(&plans, &output_path);
    let summary = structured_family_draft_rendered_record_diff_review_summary(&plans, &proof);
    let _ = fs::remove_file(output_path);
    summary
}

fn assert_confirmation_invalid_reason(
    approval: &hyprland_settings::structured_family::StructuredFamilyDraftRenderedRecordApprovalDraft,
    mut request: hyprland_settings::structured_family::StructuredFamilyDraftRenderedRecordConfirmationRequest,
    mutate: impl FnOnce(
        &mut hyprland_settings::structured_family::StructuredFamilyDraftRenderedRecordConfirmationRequest,
    ),
    expected: StructuredFamilyDraftRenderedRecordConfirmationInvalidationReason,
) {
    mutate(&mut request);
    let reasons = structured_family_draft_rendered_record_confirmation_invalidation_reasons(
        approval, &request,
    );
    assert!(
        reasons.contains(&expected),
        "expected invalidation reason {} in {:?}",
        expected.as_str(),
        reasons
    );
    let confirmation =
        accept_structured_family_draft_rendered_record_confirmation(approval, &request);
    assert_eq!(
        confirmation.confirmation_status,
        StructuredFamilyDraftRenderedRecordApprovalStatus::ConfirmationInvalidated
    );
    assert!(!confirmation.confirmation_accepted_in_memory);
    assert!(!confirmation.rendered_record_written_to_real_config);
    assert!(!confirmation.real_config_touched);
    assert!(!confirmation.runtime_mutated);
    assert!(!confirmation.hyprctl_reload_run);
    assert!(!confirmation.production_executor_wired);
}

fn assert_staged_apply_blocker(
    confirmation: &hyprland_settings::structured_family::StructuredFamilyDraftRenderedRecordConfirmation,
    summary: &hyprland_settings::structured_family::StructuredFamilyDraftRenderedRecordDiffReviewSummary,
    mutate: impl FnOnce(
        &mut hyprland_settings::structured_family::StructuredFamilyDraftRenderedRecordConfirmation,
    ),
    expected: StructuredFamilyDraftRenderedRecordStagedApplyBlocker,
) {
    let mut confirmation = confirmation.clone();
    mutate(&mut confirmation);
    let blockers =
        structured_family_draft_rendered_record_staged_apply_blockers(&confirmation, summary);
    assert!(
        blockers.contains(&expected),
        "expected staged apply blocker {} in {:?}",
        expected.as_str(),
        blockers
    );
    let plan = structured_family_draft_rendered_record_staged_apply_plan(&confirmation, summary);
    assert!(plan.blockers.contains(&expected));
    assert!(!plan.accepted_confirmation_linked);
    assert!(!plan.staged_apply_plan_written_to_disk);
    assert!(!plan.staged_apply_executed);
    assert!(!plan.rendered_record_written_to_real_config);
    assert!(!plan.real_config_touched);
    assert!(!plan.runtime_mutated);
    assert!(!plan.hyprctl_reload_run);
    assert!(!plan.production_executor_wired);
}

fn assert_staged_apply_dry_run_blocker(
    confirmation: &hyprland_settings::structured_family::StructuredFamilyDraftRenderedRecordConfirmation,
    summary: &hyprland_settings::structured_family::StructuredFamilyDraftRenderedRecordDiffReviewSummary,
    mutate: impl FnOnce(
        &mut hyprland_settings::structured_family::StructuredFamilyDraftRenderedRecordConfirmation,
    ),
    expected: StructuredFamilyDraftRenderedRecordStagedApplyBlocker,
) {
    let mut confirmation = confirmation.clone();
    mutate(&mut confirmation);
    let plan = structured_family_draft_rendered_record_staged_apply_plan(&confirmation, summary);
    let dry_run = structured_family_draft_rendered_record_staged_apply_dry_run_report(&plan);
    assert!(
        dry_run.blockers.contains(&expected),
        "expected dry-run blocker {} in {:?}",
        expected.as_str(),
        dry_run.blockers
    );
    assert_eq!(dry_run.blocked_plan_count, 1);
    assert_eq!(
        dry_run.dry_run_report_status,
        StructuredFamilyDraftRenderedRecordStagedApplyDryRunStatus::BlockedPlanSummarized
    );
    assert!(!dry_run.draft_written_to_disk);
    assert!(!dry_run.dry_run_report_written_to_disk);
    assert!(!dry_run.staged_apply_plan_written_to_disk);
    assert!(!dry_run.staged_apply_executed);
    assert!(!dry_run.dry_run_executed);
    assert!(!dry_run.rendered_record_written_to_real_config);
    assert!(!dry_run.real_config_touched);
    assert!(!dry_run.runtime_mutated);
    assert!(!dry_run.hyprctl_reload_run);
    assert!(!dry_run.production_executor_wired);
}

fn assert_rollback_recovery_blocker(
    confirmation: &hyprland_settings::structured_family::StructuredFamilyDraftRenderedRecordConfirmation,
    summary: &hyprland_settings::structured_family::StructuredFamilyDraftRenderedRecordDiffReviewSummary,
    mutate: impl FnOnce(
        &mut hyprland_settings::structured_family::StructuredFamilyDraftRenderedRecordStagedApplyDryRunReport,
    ),
    expected: StructuredFamilyDraftRenderedRecordRollbackRecoveryBlocker,
) {
    let plan = structured_family_draft_rendered_record_staged_apply_plan(confirmation, summary);
    let mut dry_run = structured_family_draft_rendered_record_staged_apply_dry_run_report(&plan);
    mutate(&mut dry_run);
    let review =
        structured_family_draft_rendered_record_staged_apply_rollback_recovery_review(&dry_run);
    assert!(
        review.recovery_blockers.contains(&expected),
        "expected rollback/recovery blocker {} in {:?}",
        expected.as_str(),
        review.recovery_blockers
    );
    assert_eq!(
        review.rollback_recovery_review_status,
        StructuredFamilyDraftRenderedRecordStagedApplyRollbackRecoveryStatus::BlockedPlanPreserved
    );
    assert!(!review.draft_written_to_disk);
    assert!(!review.rollback_recovery_review_written_to_disk);
    assert!(!review.dry_run_report_written_to_disk);
    assert!(!review.staged_apply_plan_written_to_disk);
    assert!(!review.staged_apply_executed);
    assert!(!review.dry_run_executed);
    assert!(!review.rollback_executed);
    assert!(!review.recovery_executed);
    assert!(!review.backup_created);
    assert!(!review.restore_executed);
    assert!(!review.rendered_record_written_to_real_config);
    assert!(!review.real_config_touched);
    assert!(!review.runtime_mutated);
    assert!(!review.hyprctl_reload_run);
    assert!(!review.production_executor_wired);
}

fn assert_final_executor_readiness_blocker(
    review: &hyprland_settings::structured_family::StructuredFamilyDraftRenderedRecordStagedApplyRollbackRecoveryReview,
    mutate: impl FnOnce(
        &mut hyprland_settings::structured_family::StructuredFamilyDraftRenderedRecordStagedApplyRollbackRecoveryReview,
    ),
    expected: StructuredFamilyDraftRenderedRecordFinalExecutorReadinessBlocker,
) {
    let mut review = review.clone();
    mutate(&mut review);
    let audit = structured_family_draft_rendered_record_final_executor_readiness_audit(&review);
    assert!(
        audit.blockers.contains(&expected),
        "expected final executor-readiness blocker {} in {:?}",
        expected.as_str(),
        audit.blockers
    );
    assert_eq!(
        audit.final_audit_status,
        StructuredFamilyDraftRenderedRecordFinalExecutorReadinessStatus::BlockedPlanPreserved
    );
    assert_eq!(
        audit.production_readiness_decision,
        StructuredFamilyDraftRenderedRecordFinalExecutorReadinessDecision::NotProductionReady
    );
    assert!(!audit.draft_written_to_disk);
    assert!(!audit.final_audit_written_to_disk);
    assert!(!audit.rollback_recovery_review_written_to_disk);
    assert!(!audit.dry_run_report_written_to_disk);
    assert!(!audit.staged_apply_plan_written_to_disk);
    assert!(!audit.staged_apply_executed);
    assert!(!audit.dry_run_executed);
    assert!(!audit.rollback_executed);
    assert!(!audit.recovery_executed);
    assert!(!audit.backup_created);
    assert!(!audit.restore_executed);
    assert!(!audit.executor_implemented);
    assert!(!audit.executor_wired);
    assert!(!audit.production_activation_approved);
    assert!(!audit.rendered_record_written_to_real_config);
    assert!(!audit.real_config_touched);
    assert!(!audit.runtime_mutated);
    assert!(!audit.hyprctl_reload_run);
    assert!(!audit.production_behavior_enabled);
    assert!(!audit.production_executor_wired);
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

fn changed_fixture_value_for_family(
    family: StructuredFamilyKind,
    field_name: &str,
) -> &'static str {
    match family {
        StructuredFamilyKind::Monitor => match field_name {
            "resolution" => "2560x1440@60",
            "position" => "100x100",
            "scale" => "1.25",
            _ => "DP-1",
        },
        StructuredFamilyKind::Bind => match field_name {
            "key" => "Space",
            "dispatcher" => "exec",
            "argument" => "kitty",
            _ => "SUPER_SHIFT",
        },
        StructuredFamilyKind::Animation => match field_name {
            "enabled" => "0",
            "bezier/curve reference" => "snappy",
            "speed" => "9",
            "style" => "slide",
            _ => "windows-diff",
        },
        StructuredFamilyKind::Curve => match field_name {
            "x1" => "0.33",
            "y1" => "0.77",
            "x2" => "0.44",
            "y2" => "0.99",
            _ => "snappy-diff",
        },
        StructuredFamilyKind::Gesture => match field_name {
            "direction" => "vertical",
            "dispatcher/action" => "workspace",
            "argument" => "+1",
            _ => "4",
        },
        StructuredFamilyKind::Device => match field_name {
            "option value" => "0.4",
            _ => "sensitivity",
        },
        StructuredFamilyKind::Permission => match field_name {
            "permission key" => "screencopy",
            "permission value/action" => "deny",
            _ => "class:test-app",
        },
    }
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
