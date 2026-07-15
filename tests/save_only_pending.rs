//! Backend guards for the save-only pending ledger (Scope B, broadened).
//!
//! Save-only is a deferred, gated config write, so it safely covers far more
//! than the rows that live-preview: any row that is not live-previewable,
//! not armable under the dead-man countdown, not production-gated, has a
//! concrete control, and has a gated write path is save-only-editable. These
//! guards prove the classifier admits that broad set, still rejects
//! live/armable rows, and — the one hard safety line — rejects the
//! production-gated high-risk settings that need a recovery-plan proof to
//! save.

use hyprland_settings::runtime_preview_dead_man::dead_man_ui_state;
use hyprland_settings::runtime_preview_ui_projection::runtime_preview_ui_row_state;
use hyprland_settings::save_only_pending::{
    is_save_only_editable, save_only_control_kind, SaveOnlyControlKind, SaveOnlyDraft,
    SaveOnlyLedger,
};
use hyprland_settings::write_classification::{
    is_high_risk_gated_writable_setting, SAFE_WRITABLE_ROWS,
};

/// A representative slice of rows that must be save-only-editable, across
/// every reason a row is not live-previewable.
const SAVE_ONLY_ROWS: &[&str] = &[
    "misc.middle_click_paste",           // no live setter (Boolean)
    "misc.vrr",                          // needs reload (FiniteChoice)
    "binds.drag_threshold",              // no live setter (Number)
    "input.kb_layout",                   // needs relog (SourceBacked)
    "misc.swallow_regex",                // no live setter (RegexString)
    "decoration.screen_shader",          // non-gated high-risk (Path)
    "misc.disable_autoreload",           // non-gated high-risk (Boolean)
    "input.resolve_binds_by_sym",        // was hardware-gated (Boolean)
    "input.rotation",                    // was hardware-gated
    "input.touchpad.natural_scroll",     // was hardware-gated (Boolean)
    "layout.single_window_aspect_ratio", // was not-proven
    "scrolling.wrap_focus",              // was not-proven
];

/// Rows that must never be save-only-editable: production-gated high-risk
/// (need a recovery-plan proof to save), live-preview rows, and dead-man
/// rows this machine can arm (they use the modal).
const NOT_SAVE_ONLY_ROWS: &[&str] = &[
    "render.direct_scanout",      // production-gated (display/render)
    "cursor.no_hardware_cursors", // production-gated (cursor/input)
    "debug.overlay",              // production-gated (debug/crash)
    "xwayland.enabled",           // production-gated
    "animations.enabled",         // dead-man armable -> modal
    "appearance.dim.modal",       // live-preview -> preview ledger
];

#[test]
fn classifier_admits_the_broad_save_only_set() {
    for row_id in SAVE_ONLY_ROWS {
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
fn classifier_rejects_gated_live_and_armable_rows() {
    for row_id in NOT_SAVE_ONLY_ROWS {
        assert!(
            !is_save_only_editable(row_id),
            "{row_id} must NOT be save-only-editable"
        );
    }
}

/// The one hard safety line: no production-gated high-risk setting is ever
/// save-only-editable (it would be editable-but-unsavable without a
/// recovery-plan proof, and the plain save-only path cannot supply one).
#[test]
fn production_gated_rows_are_never_save_only_editable() {
    for row in SAFE_WRITABLE_ROWS.iter() {
        if is_high_risk_gated_writable_setting(row.row_id) {
            assert!(
                !is_save_only_editable(row.row_id),
                "production-gated {} must not be save-only-editable",
                row.row_id
            );
        }
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

/// Every row that is not live-previewable, not armable, not production-gated,
/// and has a control + gated write path must be save-only-editable — no
/// silent gaps. The count is the measured save-only set.
#[test]
fn every_non_gated_non_live_row_with_a_control_is_editable() {
    let mut admitted = 0usize;
    for row in SAFE_WRITABLE_ROWS.iter() {
        let state = runtime_preview_ui_row_state(row.row_id);
        let live = state.as_ref().map(|s| s.preview_enabled).unwrap_or(false);
        let armable = dead_man_ui_state(row.row_id)
            .map(|d| d.arm_enabled)
            .unwrap_or(false);
        let saveable = state
            .as_ref()
            .map(|s| s.save_state.available())
            .unwrap_or(false);
        let has_control = save_only_control_kind(row.row_id).is_some();
        let gated = is_high_risk_gated_writable_setting(row.row_id);
        if !live && !armable && !gated && has_control && saveable {
            assert!(
                is_save_only_editable(row.row_id),
                "{} should be save-only-editable (a silent gap)",
                row.row_id
            );
            admitted += 1;
        }
    }
    assert_eq!(admitted, 117, "expected the measured save-only set");
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

    assert!(ledger.clear("misc.middle_click_paste"));
    assert!(ledger.pending().is_empty());
    assert!(ledger.is_empty());
    assert!(!ledger.clear("misc.middle_click_paste"));
}

#[test]
fn restaging_to_original_is_not_pending() {
    let mut ledger = SaveOnlyLedger::new();
    ledger.stage(SaveOnlyDraft {
        row_id: "binds.drag_threshold".into(),
        official_setting: "binds:drag_threshold".into(),
        page_id: Some("keybinds".into()),
        original_value: "0".into(),
        staged_value: "5".into(),
        config_has_line: false,
    });
    assert_eq!(ledger.pending().len(), 1);
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
