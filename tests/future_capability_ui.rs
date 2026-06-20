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
        "Candidate target files",
        "Use this target (planned)",
        "Choose target file (planned)",
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
