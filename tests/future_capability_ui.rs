use std::fs;

use hyprland_settings::write_classification::SAFE_WRITABLE_ROWS;

fn source_slice<'a>(source: &'a str, start: &str, end: &str) -> &'a str {
    let start = source
        .find(start)
        .expect("source start marker should exist");
    let end = source[start..]
        .find(end)
        .map(|offset| start + offset)
        .expect("source end marker should exist");
    &source[start..end]
}

#[test]
fn duplicate_occurrence_selector_is_visible_read_only_and_disabled() {
    let source = fs::read_to_string("src/ui/window.rs").expect("window source should read");
    let selector = source_slice(
        &source,
        "fn append_duplicate_occurrence_selector",
        "fn append_session_value_projection_summary",
    );

    for expected in [
        "hyprland-settings-duplicate-occurrence-selector-disabled",
        "hyprland-settings-duplicate-occurrence-choice-disabled",
        "Duplicate occurrences",
        "The app will not auto-choose a duplicate line.",
        "Duplicate writes stay blocked until manual occurrence selection is reviewed.",
        "Pre-Apply duplicate approval review",
        "No duplicate target is confirmed for production.",
        "Production duplicate Apply remains disabled.",
        "Approval state:",
        "Precondition fingerprint:",
        "Block reason:",
        "Confirm duplicate target (planned)",
        "hyprland-settings-duplicate-production-confirm-disabled",
        "Choose this occurrence (planned)",
        "File:",
        "Line:",
        "Value:",
        "Source depth:",
        "Raw line:",
        "confirm.set_sensitive(false)",
        "choose.set_sensitive(false)",
    ] {
        assert!(
            selector.contains(expected),
            "missing duplicate selector source: {expected}"
        );
    }

    for forbidden in [
        "execute_missing_default_insertion_plan",
        "replace_duplicate_occurrence_safe_env",
        "apply",
        "hyprctl",
        "reload",
    ] {
        assert!(
            !selector.contains(forbidden),
            "duplicate selector must not invoke production or runtime behavior: {forbidden}"
        );
    }
}

#[test]
fn layered_occurrences_expose_raw_line_and_source_depth_for_selector() {
    let source =
        fs::read_to_string("src/config_layered_values.rs").expect("layered source should read");

    assert!(source.contains("pub raw_line: String"));
    assert!(source.contains("pub source_depth: usize"));
    assert!(source.contains("raw_line: record.raw_line.clone()"));
    assert!(source.contains("source_depth: file.source_depth"));
    assert_eq!(SAFE_WRITABLE_ROWS.len(), 341);
}

#[test]
fn source_include_insertion_target_review_is_visible_read_only_and_disabled() {
    let source = fs::read_to_string("src/ui/window.rs").expect("window source should read");
    let review_source = source_slice(
        &source,
        "fn append_source_include_insertion_target_review",
        "fn source_include_readiness_label",
    );

    for expected in [
        "hyprland-settings-source-include-insertion-review-disabled",
        "hyprland-settings-source-include-target-candidate-disabled",
        "hyprland-settings-source-include-target-choice-disabled",
        "hyprland-settings-source-include-target-selection-disabled",
        "Source/include insertion target review",
        "Source/include insertion is not active yet.",
        "The app will not pick a connected file automatically.",
        "hyprland-settings-source-include-selected-target-dry-run-preview-disabled",
        "Selected-target insertion dry-run preview",
        "Root path",
        "Selected target path",
        "Source depth",
        "Planned inserted line",
        "Dry-run status",
        "Production source/include insertion remains disabled.",
        "Run selected-target insertion (planned)",
        "hyprland-settings-source-include-selected-target-run-disabled",
        "Candidate target files",
        "Use this target (planned)",
        "Choose target file (planned)",
        "run_selected.set_sensitive(false)",
        "choose.set_sensitive(false)",
        "choose_target.set_sensitive(false)",
    ] {
        assert!(
            review_source.contains(expected),
            "missing source/include insertion target review source: {expected}"
        );
    }

    for forbidden in [
        "execute_missing_default_insertion_plan",
        "replace_duplicate_occurrence_safe_env",
        "apply_setting_change",
        "hyprctl",
        "reload",
    ] {
        assert!(
            !review_source.contains(forbidden),
            "source/include target review must not invoke production or runtime behavior: {forbidden}"
        );
    }
}

#[test]
fn source_include_insertion_target_review_is_called_from_detail_edit_section() {
    let source = fs::read_to_string("src/ui/window.rs").expect("window source should read");
    let edit_source = source_slice(
        &source,
        "append_detail_section(detail_content, \"Edit\"",
        "append_detail_section(detail_content, \"Safety\"",
    );

    assert!(edit_source.contains("append_source_include_insertion_target_review"));
    assert!(source.contains("source_include_insertion_review("));
    assert!(
        source.contains("SourceIncludeInsertionReadiness::SourceIncludeTargetSelectionRequired")
    );
    assert!(source.contains("SourceIncludeInsertionReadiness::ManagedTargetBlocked"));
}

#[test]
fn runtime_approval_review_surface_displays_live_restore_evidence_and_stays_disabled() {
    let source = fs::read_to_string("src/ui/window.rs").expect("window source should read");
    let model_source = fs::read_to_string("src/future_capability.rs")
        .expect("future capability source should read");
    let review_source = source_slice(
        &source,
        "fn append_runtime_approval_review_surface",
        "fn append_source_include_insertion_target_review",
    );

    for expected in [
        "hyprland-settings-runtime-approval-review-disabled",
        "hyprland-settings-runtime-live-restore-evidence",
        "hyprland-settings-runtime-approval-enable-disabled",
        "Runtime approval review",
        "Runtime changes are not enabled yet.",
        "This setting has a proven live-restore test.",
        "Production runtime/reload remains disabled.",
        "Setting",
        "Prior value",
        "Temporary test value",
        "Mutation command",
        "Restore command",
        "Post-mutation readback",
        "Post-restore readback",
        "Approval status",
        "Production runtime/reload",
        "Enable runtime apply (planned)",
        "enable.set_sensitive(false)",
        "proven_runtime_approval_evidence_summary",
    ] {
        assert!(
            review_source.contains(expected),
            "missing runtime approval review source: {expected}"
        );
    }

    for expected in [
        "general:gaps_in",
        "hyprctl eval 'hl.config({ general = { gaps_in = 6 } })'",
        "hyprctl eval 'hl.config({ general = { gaps_in = 5 } })'",
        "css gap data: 6 6 6 6; set: true",
        "css gap data: 5 5 5 5; set: true",
        "Approved but default-disabled",
        "Disabled",
    ] {
        assert!(
            model_source.contains(expected),
            "missing runtime approval projection source: {expected}"
        );
    }

    for forbidden in [
        "runtime_live_restore_attempt_review(",
        "runtime_guarded_executor(",
        "runtime_production_enabled = true",
        "production_runtime_enabled = true",
        "apply_setting_change",
        "hyprctl reload",
        "enable.set_sensitive(true)",
    ] {
        assert!(
            !review_source.contains(forbidden),
            "runtime approval review must not enable or execute production runtime behavior: {forbidden}"
        );
    }
}

#[test]
fn runtime_approval_review_surface_is_called_from_detail_edit_section() {
    let source = fs::read_to_string("src/ui/window.rs").expect("window source should read");
    let edit_source = source_slice(
        &source,
        "append_detail_section(detail_content, \"Edit\"",
        "append_detail_section(detail_content, \"Safety\"",
    );

    assert!(edit_source.contains("append_runtime_approval_review_surface"));
    assert!(source.contains("proven_runtime_approval_evidence_summary"));
}

#[test]
fn disabled_future_approval_cards_are_visible_and_non_mutating() {
    let source = fs::read_to_string("src/ui/window.rs").expect("window source should read");
    let model_source = fs::read_to_string("src/future_capability.rs")
        .expect("future capability source should read");
    let section_source = source_slice(
        &source,
        "fn disabled_future_approval_cards_section",
        "fn append_connected_file_details",
    );

    for expected in [
        "hyprland-settings-disabled-approval-cards-section",
        "Future approval reviews",
        "These review cards show proof and blockers for future capabilities.",
        "All planned enable controls are disabled.",
        "disabled_future_approval_card",
        "Proof source",
        "Proof status",
        "Preconditions",
        "Restore and unchanged evidence",
        "production_activation_decision_reviews_section",
        "hyprland-settings-production-activation-decision-section",
        "Future production activation decision reviews",
        "production_activation_decision_review_card",
        "production_activation_path_reviews_section",
        "hyprland-settings-production-activation-path-section",
        "Future production activation paths",
        "production_activation_path_review_card",
        "production_activation_control_reviews_section",
        "hyprland-settings-production-activation-control-section",
        "Final production activation controls",
        "production_activation_control_review_card",
        "production_activation_form_reviews_section",
        "hyprland-settings-production-activation-form-section",
        "Review-only activation request forms",
        "production_activation_form_review_card",
        "production_activation_draft_reviews_section",
        "hyprland-settings-production-activation-draft-section",
        "In-memory activation drafts",
        "production_activation_draft_review_card",
        "production_activation_draft_edit_reviews_section",
        "hyprland-settings-production-activation-draft-edit-section",
        "Activation draft editing",
        "production_activation_draft_edit_review_card",
        "Decision status",
        "Decision input source",
        "Required proof summary",
        "Decision blockers",
        "Activation path status",
        "Required before enabling",
        "Activation path blockers",
        "Control status",
        "Request validation",
        "Safety plan validation",
        "Executor wiring",
        "Form status",
        "Draft status",
        "Draft validation",
        "Dirty state",
        "Editing mode",
        "Draft dirty state",
        "In-memory only",
        "Disabled activation form fields",
        "append_disabled_activation_form_fields",
        "append_disabled_activation_text_field",
        "append_disabled_activation_multiline_field",
        "append_disabled_activation_check_field",
        "gtk::Entry::new",
        "gtk::TextView::new",
        "gtk::CheckButton::with_label",
        "entry.set_editable(false)",
        "entry.set_sensitive(false)",
        "text.set_editable(false)",
        "text.set_cursor_visible(false)",
        "text.set_sensitive(false)",
        "check.set_sensitive(false)",
        "Scope/category",
        "User-facing reason",
        "Explicit activation phrase/token",
        "Decision category",
        "Backup-before-write acknowledgement",
        "Restore-plan acknowledgement",
        "Post-write reread acknowledgement",
        "Post-restore verification acknowledgement",
        "Final confirmation acknowledgement",
        "Backup-before-write plan",
        "Restore plan",
        "Post-write reread plan",
        "Post-restore verification plan",
        "Dry-run summary",
        "Files that would be touched",
        "scope-field",
        "reason-field",
        "token-field",
        "decision-category-field",
        "backup-check",
        "restore-check",
        "reread-check",
        "post-restore-check",
        "final-check",
        "backup-plan-field",
        "restore-plan-field",
        "reread-plan-field",
        "post-restore-plan-field",
        "dry-run-summary-field",
        "touched-files-field",
        "Required fields",
        "Request preview",
        "Safety plan preview",
        "enable.set_sensitive(false)",
        "start.set_sensitive(false)",
        "validate.set_sensitive(false)",
        "update.set_sensitive(false)",
        "reset.set_sensitive(false)",
    ] {
        assert!(
            section_source.contains(expected),
            "missing disabled approval cards UI source: {expected}"
        );
    }

    for expected in [
        "DISABLED_APPROVAL_CARDS_REPORT_JSON",
        "load_disabled_approval_cards_from_reports",
        "load_disabled_approval_cards_from_report_str",
        "ReportBackedDisabledApprovalCardProjection",
        "SerializedApprovalCardRecord",
        "Missing from report",
        "hyprland-settings-source-include-approval-review-disabled",
        "hyprland-settings-source-include-approval-evidence",
        "hyprland-settings-source-include-approval-enable-disabled",
        "Source/include approval review",
        "Copied-config-tree proof exists.",
        "selected target plan accepted for copied tree",
        "copied target restore",
        "original real config unchanged",
        "Production source/include insertion",
        "Enable source/include insertion (planned)",
        "hyprland-settings-duplicate-approval-review-disabled",
        "hyprland-settings-duplicate-approval-evidence",
        "hyprland-settings-duplicate-approval-enable-disabled",
        "Duplicate approval review",
        "Fingerprint/precondition status",
        "selected duplicate replaced and reread in copied tree",
        "matched fingerprint",
        "matched old-value precondition",
        "Production duplicate writes",
        "Enable duplicate replacement (planned)",
        "ProductionActivationDecisionReview",
        "production_activation_decision_reviews",
        "source_include_activation_decision_review",
        "duplicate_activation_decision_review",
        "ProductionActivationPathReview",
        "production_activation_path_reviews",
        "source_include_activation_path_review",
        "duplicate_activation_path_review",
        "ProductionActivationControlReview",
        "ProductionActivationControlStatus",
        "ProductionExecutorWiringState",
        "production_activation_control_reviews",
        "source_include_activation_control_review",
        "duplicate_activation_control_review",
        "ProductionActivationFormReview",
        "ProductionActivationFormStatus",
        "production_activation_form_reviews",
        "source_include_activation_form_review",
        "duplicate_activation_form_review",
        "ProductionActivationDraftForm",
        "ProductionActivationDraftEditReview",
        "ProductionActivationDraftEditState",
        "ProductionActivationDraftEditMode",
        "ProductionActivationDraftEditStatus",
        "ProductionActivationDraftEditAction",
        "ProductionActivationDraftReview",
        "ProductionActivationDraftStatus",
        "ProductionActivationDraftUpdate",
        "production_activation_draft_edit_reviews",
        "source_include_activation_draft_edit_review",
        "duplicate_activation_draft_edit_review",
        "apply_production_activation_draft_edit_action",
        "production_activation_draft_edit_state_from_draft",
        "production_activation_draft_reviews",
        "source_include_activation_draft_review",
        "duplicate_activation_draft_review",
        "apply_production_activation_draft_update",
        "reset_production_activation_draft",
        "hyprland-settings-source-include-activation-decision-disabled",
        "hyprland-settings-source-include-activation-decision-evidence",
        "hyprland-settings-source-include-activation-decision-enable-disabled",
        "Source/include production activation decision",
        "Enable source/include production activation (planned)",
        "hyprland-settings-duplicate-activation-decision-disabled",
        "hyprland-settings-duplicate-activation-decision-evidence",
        "hyprland-settings-duplicate-activation-decision-enable-disabled",
        "Duplicate production activation decision",
        "Enable duplicate production activation (planned)",
        "hyprland-settings-source-include-activation-path-disabled",
        "hyprland-settings-source-include-activation-path-evidence",
        "hyprland-settings-source-include-activation-path-start-disabled",
        "hyprland-settings-source-include-activation-control-disabled",
        "hyprland-settings-source-include-activation-control-evidence",
        "hyprland-settings-source-include-activation-control-validate-disabled",
        "Source/include production activation control",
        "Validate source/include activation request (planned)",
        "hyprland-settings-source-include-activation-form-disabled",
        "hyprland-settings-source-include-activation-form-evidence",
        "hyprland-settings-source-include-activation-form-validate-disabled",
        "Source/include activation request form",
        "Validate source/include activation form (planned)",
        "hyprland-settings-source-include-activation-draft-disabled",
        "hyprland-settings-source-include-activation-draft-evidence",
        "hyprland-settings-source-include-activation-draft-update-disabled",
        "hyprland-settings-source-include-activation-draft-reset-disabled",
        "Source/include activation draft",
        "Update source/include activation draft (planned)",
        "Reset source/include activation draft (planned)",
        "hyprland-settings-source-include-activation-draft-edit-disabled",
        "hyprland-settings-source-include-activation-draft-edit-evidence",
        "hyprland-settings-source-include-activation-draft-edit-mode-disabled",
        "hyprland-settings-source-include-activation-draft-edit-update-disabled",
        "hyprland-settings-source-include-activation-draft-edit-reset-disabled",
        "Source/include activation draft editing",
        "hyprland-settings-duplicate-activation-form-disabled",
        "hyprland-settings-duplicate-activation-form-evidence",
        "hyprland-settings-duplicate-activation-form-validate-disabled",
        "Duplicate activation request form",
        "Validate duplicate activation form (planned)",
        "hyprland-settings-duplicate-activation-draft-disabled",
        "hyprland-settings-duplicate-activation-draft-evidence",
        "hyprland-settings-duplicate-activation-draft-update-disabled",
        "hyprland-settings-duplicate-activation-draft-reset-disabled",
        "Duplicate activation draft",
        "Update duplicate activation draft (planned)",
        "Reset duplicate activation draft (planned)",
        "hyprland-settings-duplicate-activation-draft-edit-disabled",
        "hyprland-settings-duplicate-activation-draft-edit-evidence",
        "hyprland-settings-duplicate-activation-draft-edit-mode-disabled",
        "hyprland-settings-duplicate-activation-draft-edit-update-disabled",
        "hyprland-settings-duplicate-activation-draft-edit-reset-disabled",
        "Duplicate activation draft editing",
        "Source/include production activation path",
        "Start source/include production activation (planned)",
        "hyprland-settings-source-include-activation-control-disabled",
        "hyprland-settings-source-include-activation-control-evidence",
        "hyprland-settings-source-include-activation-control-validate-disabled",
        "Source/include production activation control",
        "Validate source/include activation request (planned)",
        "hyprland-settings-duplicate-activation-path-disabled",
        "hyprland-settings-duplicate-activation-path-evidence",
        "hyprland-settings-duplicate-activation-path-start-disabled",
        "Duplicate production activation path",
        "Start duplicate production activation (planned)",
        "hyprland-settings-duplicate-activation-control-disabled",
        "hyprland-settings-duplicate-activation-control-evidence",
        "hyprland-settings-duplicate-activation-control-validate-disabled",
        "Duplicate production activation control",
        "Validate duplicate activation request (planned)",
        "ValidatedButExecutorUnwired",
        "ExecutorMustRemainUnwired",
        "ProductionFlagMustRemainFalse",
        "Complete activation request",
        "Complete safety plan",
        "Unwired",
        "ActivationPathNeedsExplicitProductionFlag",
        "ActivationPathReadyButDefaultDisabled",
        "ReadyButDefaultDisabled",
        "ApprovedButDefaultDisabled",
        "ProductionAlreadyEnabledError",
        "hyprland-settings-structured-approval-review-disabled",
        "hyprland-settings-structured-approval-evidence",
        "hyprland-settings-structured-approval-enable-disabled",
        "Structured hl.bind approval review",
        "Old raw line",
        "Proposed raw line",
        "Candidate validation status",
        "selected hl.bind line edited and reread in copied tree",
        "comments and order preserved",
        "restored byte-for-byte",
        "Production structured writes",
        "Enable structured write (planned)",
        "hyprland-settings-profile-approval-review-disabled",
        "hyprland-settings-profile-approval-evidence",
        "hyprland-settings-profile-approval-enable-disabled",
        "Profile/mode approval review",
        "Current symlink",
        "Original target",
        "Proposed target",
        "Restore proof status",
        "copied-config-tree profile/symlink proof",
        "temp symlink switched to selected copied target",
        "real symlink untouched",
        "Production profile switching",
        "Enable profile switching (planned)",
        "hyprland-settings-high-risk-approval-review-disabled",
        "hyprland-settings-high-risk-approval-evidence",
        "hyprland-settings-high-risk-approval-enable-disabled",
        "High-risk/display approval review",
        "Out-of-band recovery",
        "Dead-man timeout",
        "Runtime live-restore proof is available for a low-risk setting.",
        "That proof is not enough to enable high-risk/display writes.",
        "high-risk readiness gate",
        "succeeded outside sandbox",
        "general:gaps_in restored after hl.config eval proof",
        "low-risk runtime proof does not prove display recovery",
        "Production high-risk/display writes",
        "Enable high-risk/display writes (planned)",
        "hyprland-settings-0554-approval-review-disabled",
        "hyprland-settings-0554-approval-evidence",
        "hyprland-settings-0554-approval-enable-disabled",
        "Hyprland 0.55.4 migration review",
        "Runtime version evidence exists.",
        "Package metadata evidence exists.",
        "These are advisory only.",
        "runtime/package/trusted-data records",
        "Hyprland 0.55.4 commit a0136d8c04687bb36eb8a28eb9d1ff92aea99704",
        "hyprland 0.55.4-1",
        "runtime/package evidence cannot activate migration",
        "Official 0.55.4 export bundle",
        "Row-count diff",
        "Write-safety review",
        "Safe-env evidence",
        "Current active app model",
        "v0.55.2",
        "Migration status",
        "Inactive",
        "Enable 0.55.4 migration (planned)",
    ] {
        assert!(
            model_source.contains(expected),
            "missing disabled approval card projection source: {expected}"
        );
    }

    let config_source = source_slice(&source, "fn build_config_view", "fn config_path_summary");
    assert!(config_source.contains("disabled_future_approval_cards_section"));

    for forbidden in [
        "execute_source_include_selected_target_guarded_temp",
        "execute_duplicate_replacement_guarded_temp",
        "execute_structured_bind_guarded_temp",
        "switch_profile_symlink_guarded_temp",
        "high_risk_guarded_live_readiness_executor",
        "hyprland_version_activation_gate(",
        "apply_setting_change",
        "hyprctl",
        "reload",
        "enable.set_sensitive(true)",
    ] {
        assert!(
            !section_source.contains(forbidden),
            "disabled approval cards must not invoke production behavior: {forbidden}"
        );
    }
}
