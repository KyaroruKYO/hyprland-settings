//! Presentation-adoption guards: every scalar row resolves a friendly
//! label through the adoption layer, matched labels carry verified
//! provenance, unmatched rows keep their official label untouched, raw
//! keys and every classification/save gate are unchanged, and the hidden
//! Ctrl+F search is wired. Normal tests only.

use std::collections::BTreeSet;
use std::fs;

use hyprland_settings::presentation_labels::{display_label_for_row, ROW_DISPLAY_LABELS};
use hyprland_settings::ux_presentation::{choice_display_label, resolved_row_label};
use hyprland_settings::write_classification::SAFE_WRITABLE_ROWS;

#[test]
fn adoption_counts_and_provenance_are_pinned() {
    // 127 matched labels over the 341-row model; 214 rows stay unmatched
    // (reported, never guessed).
    assert_eq!(SAFE_WRITABLE_ROWS.len(), 341);
    assert_eq!(ROW_DISPLAY_LABELS.len(), 127);

    let row_index: std::collections::BTreeMap<&str, &str> = SAFE_WRITABLE_ROWS
        .iter()
        .map(|row| (row.row_id, row.official_setting))
        .collect();

    let mut seen = BTreeSet::new();
    for entry in ROW_DISPLAY_LABELS {
        // Unique, existing rows only.
        assert!(seen.insert(entry.row_id), "duplicate {}", entry.row_id);
        let official = row_index
            .get(entry.row_id)
            .unwrap_or_else(|| panic!("{} is not a model row", entry.row_id));

        // Provenance: the recorded reference key normalizes to the row's
        // raw official Hyprland setting key, which is preserved unchanged.
        assert_eq!(
            entry.reference_key.replace(':', "."),
            **official,
            "reference key must match the official setting for {}",
            entry.row_id
        );

        // Labels are short, friendly, and non-empty.
        assert!(!entry.label.trim().is_empty());
        assert!(entry.label.chars().count() <= 60);
    }
}

#[test]
fn every_row_resolves_a_label_and_unmatched_rows_keep_official_metadata() {
    let mut matched = 0;
    let mut unmatched = 0;
    for row in SAFE_WRITABLE_ROWS {
        // The official metadata label is what the model passes at runtime;
        // a sentinel proves the fallback path byte-exactly.
        let resolved = resolved_row_label(row.row_id, "OFFICIAL-METADATA-LABEL");
        assert!(
            !resolved.trim().is_empty(),
            "{} resolves a label",
            row.row_id
        );
        if display_label_for_row(row.row_id).is_some() {
            matched += 1;
        } else {
            unmatched += 1;
            // Unmatched rows keep the official metadata label untouched.
            assert_eq!(
                resolved, "OFFICIAL-METADATA-LABEL",
                "{} must not be guessed",
                row.row_id
            );
        }
    }
    assert_eq!(matched, 127);
    assert_eq!(unmatched, 214);
}

#[test]
fn dropdown_display_labels_never_change_raw_values() {
    // Presentation form only; numbers and empty values pass through.
    assert_eq!(choice_display_label("follow_mouse"), "Follow mouse");
    assert_eq!(choice_display_label("no-cond"), "No cond");
    assert_eq!(choice_display_label("2"), "2");
    assert_eq!(choice_display_label(" 0.5 "), "0.5");
    assert_eq!(choice_display_label(""), "");

    // The dropdown model keys stay the raw values: the combo stores
    // raw_value as the id and applies active_id, never the display text.
    let window = fs::read_to_string("src/ui/window.rs").expect("window reads");
    assert!(window.contains("combo.append(Some(raw_value), label)"));
    let projection =
        fs::read_to_string("src/runtime_preview_ui_projection.rs").expect("projection reads");
    assert!(projection.contains("choice_display_label(choice.raw_value)"));
    assert!(projection.contains("(choice.raw_value.to_string(), display)"));
}

#[test]
fn presentation_layer_changes_no_behavior_or_classification() {
    // The label table is data plus one lookup: no file access, no process
    // spawning, no runtime commands, no save paths.
    let labels = fs::read_to_string("src/presentation_labels.rs").expect("labels read");
    for forbidden in [
        "fs::",
        "Command::new",
        "std::process",
        "hyprctl",
        "gated_",
        "apply_setting_change",
    ] {
        assert!(
            !labels.contains(forbidden),
            "labels module must not contain {forbidden}"
        );
    }

    // The UI resolves labels through the presentation layer in both the
    // list rows and the detail heading: matched labels first, otherwise
    // the official label with only the page-name prefix stripped (a
    // formatting-only fallback — nothing is guessed).
    let window = fs::read_to_string("src/ui/window.rs").expect("window reads");
    assert!(window.contains("display_label_for_row(&setting.row_id)"));
    assert!(window.contains("display_label_for_row(&detail.row_id)"));
    assert!(window.contains("fallback_display_label(&setting.label, &setting.tab_label)"));
    assert!(window.contains("fallback_display_label(&detail.label, &detail.tab_label)"));
}

#[test]
fn hidden_search_is_wired_for_ctrl_f_and_escape() {
    let window = fs::read_to_string("src/ui/window.rs").expect("window reads");
    // Hidden by default behind the Search toggle; Ctrl+F and Escape route
    // through the toggle so every path behaves identically; Esc inside the
    // entry uses the same clear-and-hide path.
    assert!(window.contains("search_entry.set_visible(false)"));
    assert!(window.contains("hyprland-settings-search-toggle"));
    assert!(window.contains("Search settings (Ctrl+F)"));
    assert!(window.contains("gtk::EventControllerKey::new()"));
    assert!(window.contains("gtk::gdk::Key::f"));
    assert!(window.contains("search_toggle.set_active(true)"));
    assert!(window.contains("gtk::gdk::Key::Escape"));
    assert!(window.contains("search_toggle.set_active(false)"));
    assert!(window.contains("entry.grab_focus()"));
    assert!(window.contains("connect_stop_search"));
    assert!(window.contains("window.add_controller(key_controller)"));
}

#[test]
fn config_page_picker_card_stays_quiet() {
    let window = fs::read_to_string("src/ui/window.rs").expect("window reads");
    let picker_card_start = window
        .find("fn structured_family_preview_controls_section")
        .expect("picker card fn");
    let picker_card_end = picker_card_start
        + window[picker_card_start..]
            .find("fn structured_family_runtime_preview_status_section")
            .expect("next fn");
    let picker_card = &window[picker_card_start..picker_card_end];

    // Quiet card: friendly title, short intro, gate chip pointer — the
    // proof prose lives on the Safety Details page instead.
    assert!(picker_card.contains("Animations & curves"));
    assert!(picker_card.contains("SAVE_GATE_CHIP"));
    assert!(picker_card.contains("Safety Details page"));
    for wall_text in [
        "zero-residue",
        "live proof",
        "readback, preview it live under",
    ] {
        assert!(
            !picker_card.contains(wall_text),
            "picker card must not contain proof-wall text: {wall_text}"
        );
    }
}

#[test]
fn adoption_report_records_unchanged_safety() {
    let report: serde_json::Value = serde_json::from_str(
        &fs::read_to_string("data/reports/hyprmod-full-presentation-adoption.v0.55.2.json")
            .expect("report reads"),
    )
    .expect("report parses");
    assert_eq!(report["scalarRowsTotal"].as_i64(), Some(341));
    assert_eq!(report["matchedRows"].as_i64(), Some(127));
    assert_eq!(report["adoptedLabels"].as_i64(), Some(127));
    assert_eq!(report["unmatchedRows"].as_i64(), Some(214));
    assert_eq!(report["rowsBehaviorChanged"].as_i64(), Some(0));
    assert_eq!(
        report["rowsWriteOrSafetyClassificationChanged"].as_i64(),
        Some(0)
    );
    assert_eq!(report["safeWritableRowCountChanged"].as_bool(), Some(false));
    assert_eq!(report["newRuntimeMutationPathAdded"].as_bool(), Some(false));
    assert_eq!(report["hyprctlReloadPathAdded"].as_bool(), Some(false));
    // Description adoption is a recorded licensing decision, not silence.
    assert_eq!(report["adoptedDescriptions"].as_i64(), Some(0));
    assert!(report["descriptionPolicy"]
        .as_str()
        .unwrap_or("")
        .contains("GPL"));
}
