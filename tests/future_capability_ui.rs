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
