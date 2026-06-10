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
fn connected_file_detail_helpers_exist() {
    let source = fs::read_to_string("src/ui/window.rs").expect("window source should read");

    for helper in [
        "append_connected_file_details",
        "connected_file_reason",
        "connected_file_readable_label",
        "connected_file_source_summary",
        "source_reference_for_file",
        "paths_match_file",
        "connected_file_notes",
        "friendly_connected_file_note",
    ] {
        assert!(
            source.contains(helper),
            "missing connected-file detail helper: {helper}"
        );
    }

    assert!(source.contains("gtk::Expander::new(Some(\"Details\"))"));
}

#[test]
fn connected_file_detail_copy_is_user_facing() {
    let source = fs::read_to_string("src/ui/window.rs").expect("window source should read");
    let detail_source = source_slice(
        &source,
        "fn append_connected_file_details",
        "fn connected_file_title",
    );

    for copy in [
        "Why this file is listed",
        "Role",
        "Readable",
        "Symlink",
        "Points to",
        "Connected from",
        "Notes",
        "This is the config file the app is currently reviewing.",
        "This file is connected from another config file.",
        "This is the selected config root.",
        "The app found this file while reviewing connected configs.",
        "No extra notes were detected.",
    ] {
        assert!(
            detail_source.contains(copy),
            "missing connected-file detail copy: {copy}"
        );
    }

    for avoided in [
        "source graph",
        "symlink provenance",
        "duplicate scalar conflict",
        "ambiguous write target",
        "parser normalization",
        "canonical path",
    ] {
        assert!(
            !detail_source.contains(avoided),
            "main connected-file detail copy should not expose technical wording: {avoided}"
        );
    }
}

#[test]
fn connected_file_note_copy_covers_generated_script_and_profile_hints() {
    let source = fs::read_to_string("src/ui/window.rs").expect("window source should read");
    let notes_source = source_slice(
        &source,
        "fn connected_file_notes",
        "fn append_connected_file_issue_warnings",
    );

    for copy in [
        "This file appears to be generated",
        "This file may be changed by scripts",
        "This file is symlinked",
        "This file looks like a profile file",
    ] {
        assert!(
            notes_source.contains(copy),
            "missing connected-file note copy: {copy}"
        );
    }
}

#[test]
fn source_relationship_uses_graph_source_references_without_raw_contents() {
    let source = fs::read_to_string("src/ui/window.rs").expect("window source should read");
    let source_summary = source_slice(
        &source,
        "fn connected_file_source_summary",
        "fn connected_file_notes",
    );

    assert!(source_summary.contains("source_reference_for_file"));
    assert!(source_summary.contains("source.line_number"));
    assert!(source_summary.contains("source.source_file"));
    assert!(!source_summary.contains("source.raw_line"));
    assert!(!source_summary.contains("source.raw_target"));
}

#[test]
fn connected_file_detail_report_preserves_counts_and_boundaries() {
    let report: serde_json::Value = serde_json::from_slice(
        &fs::read("data/reports/connected-file-detail-ui.v0.55.2.json")
            .expect("report should exist"),
    )
    .expect("report should parse");

    assert_eq!(report["countsBefore"]["readableRows"], 341);
    assert_eq!(report["countsBefore"]["writableRows"], 341);
    assert_eq!(report["countsBefore"]["blockedRows"], 0);
    assert_eq!(report["countsAfter"]["readableRows"], 341);
    assert_eq!(report["countsAfter"]["writableRows"], 341);
    assert_eq!(report["countsAfter"]["blockedRows"], 0);
    assert_eq!(report["safetyBoundaries"]["appWriteModelChanged"], false);
    assert_eq!(SAFE_WRITABLE_ROWS.len(), 341);
}
