//! Backend guards for the save-only pending ledger (Scope B).
//!
//! These prove the classifier admits exactly the safe save-only rows and
//! rejects high-risk, hardware-gated, dead-man, not-proven, and
//! live-preview rows; that the staged ledger tracks pending state with the
//! same semantic comparison as the live-preview ledger; and that staging
//! performs no runtime mutation (the module compiles and runs with no
//! `hyprctl` runner anywhere in reach).

use hyprland_settings::save_only_pending::{
    is_save_only_editable, save_only_control_kind, SaveOnlyControlKind, SaveOnlyDraft,
    SaveOnlyLedger,
};
use hyprland_settings::ux_presentation::{status_chip_for_row, StatusChip};
use hyprland_settings::write_classification::SAFE_WRITABLE_ROWS;

/// Rows the reliability matrix live-verified as StatusChip::SaveOnly. A
/// representative slice across every value grammar the bucket contains.
const SAVE_ONLY_ROWS: &[&str] = &[
    "misc.middle_click_paste",        // Boolean -> Switch
    "misc.focus_on_activate",         // Boolean -> Switch
    "misc.vrr",                       // FiniteChoice -> Dropdown
    "misc.on_focus_under_fullscreen", // FiniteChoice -> Dropdown
    "binds.drag_threshold",           // Number -> Spin
    "gestures.close_max_timeout",     // Number -> Spin
    "input.kb_layout",                // SourceBacked -> Entry
    "input.kb_options",               // SourceBacked -> Entry
    "misc.font_family",               // LineSafeString -> Entry
    "misc.swallow_regex",             // RegexString -> Entry
];

/// Rows that must never become save-only-editable, one per excluded bucket.
const NOT_SAVE_ONLY_ROWS: &[&str] = &[
    "decoration.screen_shader",          // blocked high-risk
    "misc.disable_autoreload",           // blocked high-risk
    "input.resolve_binds_by_sym",        // hardware-gated
    "input.rotation",                    // hardware-gated
    "layout.single_window_aspect_ratio", // not proven yet
    "animations.enabled",                // dead-man supervised (LivePreview chip, no inline)
    "appearance.dim.modal",              // live-preview editable (preview ledger owns it)
];

#[test]
fn classifier_admits_the_save_only_bucket() {
    for row_id in SAVE_ONLY_ROWS {
        assert_eq!(
            status_chip_for_row(row_id),
            StatusChip::SaveOnly,
            "{row_id} should classify as SaveOnly"
        );
        assert!(
            is_save_only_editable(row_id),
            "{row_id} should be save-only-editable"
        );
        assert!(
            save_only_control_kind(row_id).is_some(),
            "{row_id} should map to a concrete control"
        );
    }
}

#[test]
fn classifier_rejects_unsafe_and_live_rows() {
    for row_id in NOT_SAVE_ONLY_ROWS {
        assert!(
            !is_save_only_editable(row_id),
            "{row_id} must NOT be save-only-editable"
        );
    }
}

#[test]
fn control_kind_matches_value_grammar() {
    assert_eq!(
        save_only_control_kind("misc.middle_click_paste"),
        Some(SaveOnlyControlKind::Switch)
    );
    assert_eq!(
        save_only_control_kind("misc.vrr"),
        Some(SaveOnlyControlKind::Dropdown)
    );
    assert_eq!(
        save_only_control_kind("binds.drag_threshold"),
        Some(SaveOnlyControlKind::Spin)
    );
    assert_eq!(
        save_only_control_kind("input.kb_layout"),
        Some(SaveOnlyControlKind::Entry)
    );
}

/// Every row the app classifies SaveOnly, that is not live-previewable and
/// is saveable, must be admitted by the save-only editable predicate — the
/// pending target rule ("every editable save-only row has a staged pending
/// state") holds with no silent gaps.
#[test]
fn every_save_only_saveable_row_is_editable() {
    let mut admitted = 0usize;
    for row in SAFE_WRITABLE_ROWS.iter() {
        if status_chip_for_row(row.row_id) != StatusChip::SaveOnly {
            continue;
        }
        // The bucket the reliability matrix measured: 48 rows, all saveable.
        assert!(
            is_save_only_editable(row.row_id),
            "{} is SaveOnly but not save-only-editable — a silent gap",
            row.row_id
        );
        admitted += 1;
    }
    assert_eq!(admitted, 48, "expected exactly the 48 SaveOnly rows");
}

#[test]
fn ledger_stage_and_clear_track_pending() {
    let mut ledger = SaveOnlyLedger::new();
    assert!(ledger.is_empty());
    ledger.stage(SaveOnlyDraft {
        row_id: "misc.middle_click_paste".into(),
        official_setting: "misc:middle_click_paste".into(),
        page_id: Some("system".into()),
        original_value: "false".into(),
        staged_value: "true".into(),
        config_has_line: false,
    });
    assert_eq!(ledger.pending().len(), 1);
    assert_eq!(ledger.len(), 1);
    assert!(ledger.get("misc.middle_click_paste").is_some());

    // Clearing removes it and the pending set empties.
    assert!(ledger.clear("misc.middle_click_paste"));
    assert!(ledger.pending().is_empty());
    assert!(ledger.is_empty());
    // Clearing an absent row is a no-op.
    assert!(!ledger.clear("misc.middle_click_paste"));
}

#[test]
fn restaging_to_original_is_not_pending() {
    let mut ledger = SaveOnlyLedger::new();
    // Stage a change...
    ledger.stage(SaveOnlyDraft {
        row_id: "binds.drag_threshold".into(),
        official_setting: "binds:drag_threshold".into(),
        page_id: Some("keybinds".into()),
        original_value: "0".into(),
        staged_value: "5".into(),
        config_has_line: false,
    });
    assert_eq!(ledger.pending().len(), 1);
    // ...then restage back to the original. The draft still exists but is
    // not pending (no unsaved change), and stage() replaced rather than
    // duplicated.
    ledger.stage(SaveOnlyDraft {
        row_id: "binds.drag_threshold".into(),
        official_setting: "binds:drag_threshold".into(),
        page_id: Some("keybinds".into()),
        original_value: "0".into(),
        staged_value: "0".into(),
        config_has_line: false,
    });
    assert_eq!(ledger.len(), 1, "restage replaces, never duplicates");
    assert!(
        ledger.pending().is_empty(),
        "back-to-original is not pending"
    );
}

#[test]
fn pending_uses_semantic_equality() {
    let mut ledger = SaveOnlyLedger::new();
    // "true" vs "1" are the same boolean value — not a pending change.
    ledger.stage(SaveOnlyDraft {
        row_id: "misc.middle_click_paste".into(),
        official_setting: "misc:middle_click_paste".into(),
        page_id: Some("system".into()),
        original_value: "1".into(),
        staged_value: "true".into(),
        config_has_line: true,
    });
    assert!(
        ledger.pending().is_empty(),
        "semantic-equal spellings are not a pending change"
    );
}

#[test]
fn clear_all_empties_every_draft() {
    let mut ledger = SaveOnlyLedger::new();
    for (row, val) in [("misc.vrr", "2"), ("misc.font_family", "Sans")] {
        ledger.stage(SaveOnlyDraft {
            row_id: row.into(),
            official_setting: row.replace('.', ":"),
            page_id: None,
            original_value: "orig".into(),
            staged_value: val.into(),
            config_has_line: false,
        });
    }
    assert_eq!(ledger.pending().len(), 2);
    ledger.clear_all();
    assert!(ledger.is_empty());
    assert!(ledger.pending().is_empty());
}
