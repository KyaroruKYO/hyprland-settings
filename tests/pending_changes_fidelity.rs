//! Guards for the pending-changes fidelity pass: the reference-matched
//! unsaved-changes experience (row accents, header chip, bottom action
//! bar, hidden review page with diff preview) cannot silently regress,
//! and its save path stays the existing gated one.

use std::fs;

use hyprland_settings::pending_changes_ui::{
    next_save_config_text, pending_change_kind, pending_change_subtitle, pending_group_caption,
    pending_summary_title, unified_diff, DiffLineKind, NextSaveChange,
};
use hyprland_settings::ux_presentation::{page_for_row, SIDEBAR_PAGE_LAYOUT};

fn window_source() -> String {
    fs::read_to_string("src/ui/window.rs").expect("window source reads")
}

fn fn_slice<'a>(source: &'a str, name: &str) -> &'a str {
    let start = source
        .find(&format!("fn {name}("))
        .unwrap_or_else(|| panic!("fn {name} exists"));
    let next = source[start + 3..]
        .find("\nfn ")
        .map(|offset| start + 3 + offset)
        .unwrap_or(source.len());
    &source[start..next]
}

#[test]
fn summary_and_row_text_match_the_reference_shapes() {
    assert_eq!(pending_summary_title(1), "1 unsaved change");
    assert_eq!(pending_summary_title(4), "4 unsaved changes");
    assert_eq!(pending_group_caption(1), "1 change");
    assert_eq!(pending_group_caption(3), "3 changes");
    // Colon-form config key plus staged value.
    assert_eq!(
        pending_change_subtitle("general.allow_tearing", "true"),
        "general:allow_tearing · set to true"
    );
    // The pill reflects what the save would do to the config file.
    assert_eq!(pending_change_kind(true), "Modified");
    assert_eq!(pending_change_kind(false), "Added");
}

#[test]
fn unified_diff_reports_hunks_and_counts() {
    let old_text = "a\nb\nc\nd\ne\nf\ng\n";
    let new_text = "a\nb\nc\nX\ne\nf\ng\n";
    let diff = unified_diff(old_text, new_text, "old.conf", "old.conf (next save)");
    assert_eq!(diff.added, 1);
    assert_eq!(diff.removed, 1);
    assert!(diff
        .lines
        .iter()
        .any(|line| line.kind == DiffLineKind::Meta && line.text == "--- old.conf"));
    assert!(diff
        .lines
        .iter()
        .any(|line| line.kind == DiffLineKind::Meta && line.text == "+++ old.conf (next save)"));
    assert!(diff
        .lines
        .iter()
        .any(|line| line.kind == DiffLineKind::Hunk && line.text.starts_with("@@ -")));
    assert!(diff
        .lines
        .iter()
        .any(|line| line.kind == DiffLineKind::Added && line.text == "+X"));
    assert!(diff
        .lines
        .iter()
        .any(|line| line.kind == DiffLineKind::Removed && line.text == "-d"));
    // Context lines surround the change.
    assert!(diff
        .lines
        .iter()
        .any(|line| line.kind == DiffLineKind::Context && line.text == " c"));

    // Identical texts produce an empty diff (the section hides).
    let same = unified_diff(old_text, old_text, "old.conf", "old.conf (next save)");
    assert!(same.is_empty());
    assert!(same.lines.is_empty());
}

#[test]
fn next_save_text_uses_the_real_writer_helpers_read_only() {
    // Replace an existing line (the row's current line in the target) and
    // append a missing key — the same operations the gated save performs.
    // Colon-form scalar lines are the shape the writer supports.
    let original = "# managed\ngeneral:gaps_in = 5\n";
    let changes = [NextSaveChange {
        setting_id: "appearance.gaps_in".to_string(),
        proposed_value: "12".to_string(),
        line_in_target: Some(2),
    }];
    let next = next_save_config_text(original, &changes).expect("replace renders");
    assert!(next.contains("gaps_in = 12"));
    assert!(!next.contains("gaps_in = 5"));

    let appended = next_save_config_text(
        original,
        &[NextSaveChange {
            setting_id: "appearance.border_size".to_string(),
            proposed_value: "3".to_string(),
            line_in_target: None,
        }],
    )
    .expect("append renders");
    assert!(appended.contains("general:border_size = 3"));
    // Read-only path: unknown settings fail closed instead of guessing.
    assert!(next_save_config_text(
        original,
        &[NextSaveChange {
            setting_id: "not.a.row".to_string(),
            proposed_value: "1".to_string(),
            line_in_target: None,
        }],
    )
    .is_err());
}

#[test]
fn changed_rows_carry_the_pending_accent() {
    let window = window_source();
    let row_builder = fn_slice(&window, "build_setting_row");
    assert!(row_builder.contains("register_pending_row_widget"));
    assert!(row_builder.contains("hyprland-settings-row-pending"));
    // Amber left-edge accent, reference treatment.
    assert!(window
        .contains(".hyprland-settings-row-pending { border-left: 3px solid @warning_bg_color; }"));
    // Live updates flow through the one notifier.
    assert!(fn_slice(&window, "notify_pending_changed").contains("hyprland-settings-row-pending"));
}

#[test]
fn header_chip_appears_only_with_pending_changes() {
    let window = window_source();
    assert!(window.contains("hyprland-settings-pending-chip"));
    assert!(window.contains("view-list-symbolic"));
    assert!(window.contains("header.pack_end(&pending_chip)"));
    assert!(window.contains("pending_chip.set_visible(false)"));
    // Count + visibility come from the ledger; the chip hides on the
    // review page itself.
    assert!(window.contains("pending_chip_count.set_label(&count.to_string())"));
    assert!(window.contains("count > 0 && selected_tab_id.borrow().as_str() != PENDING_ID"));
    // Amber pill styling.
    assert!(window.contains(
        ".hyprland-settings-pending-chip { background-color: alpha(@warning_bg_color, 0.18);"
    ));
}

#[test]
fn sidebar_rows_carry_per_page_pending_badges() {
    let window = window_source();
    let sidebar = fn_slice(&window, "build_sidebar");
    assert!(sidebar.contains("hyprland-settings-sidebar-badge"));
    assert!(sidebar.contains("badge.set_visible(false)"));
    assert!(window.contains("badge.set_visible(page_count > 0)"));
    // Rows map to pages through the shared claim logic.
    assert_eq!(
        page_for_row("appearance", "general.gaps_in").map(|page| page.id),
        Some("general")
    );
    assert_eq!(
        page_for_row("display", "xwayland.enabled").map(|page| page.id),
        Some("xwayland")
    );
}

#[test]
fn bottom_bar_matches_the_reference_and_keeps_the_gated_save() {
    let window = window_source();
    let bar = fn_slice(&window, "build_pending_bottom_bar");
    assert!(bar.contains("Unsaved changes — applied live, not saved to disk"));
    assert!(bar.contains("dialog-warning-symbolic"));
    assert!(bar.contains("Discard"));
    assert!(bar.contains("Save now"));
    assert!(bar.contains("adw::SplitButton"));
    assert!(bar.contains("Save as new profile"));
    // The profile action ships disabled: no invented behavior.
    assert!(bar.contains("action.set_enabled(false)"));
    assert!(bar.contains("Changes saved to disk"));
    // Save now = the existing per-row gated save, nothing else.
    assert!(bar.contains("gated_scalar_save_live"));
    assert!(!bar.contains("fs::write"));
    assert!(!bar.contains("apply_setting_change("));
    // Discard reverts through each controller.
    assert!(bar.contains(".revert()"));
    // Slide-up revealer like the reference banner.
    assert!(bar.contains("RevealerTransitionType::SlideUp"));
}

#[test]
fn pending_changes_page_is_hidden_and_structured_like_the_reference() {
    let window = window_source();
    // Hidden tab: reachable from the chip, never a sidebar entry.
    assert!(window.contains("const PENDING_ID: &str = \"pending-changes\""));
    let in_sidebar = SIDEBAR_PAGE_LAYOUT
        .iter()
        .flat_map(|category| category.pages.iter())
        .any(|page| page.id == "pending-changes");
    assert!(!in_sidebar, "Pending Changes must not be a sidebar page");
    assert!(window.contains("(PENDING_ID, pending_view.clone())"));
    assert!(window.contains("header_title.set_title(\"Pending Changes\")"));

    let page = fn_slice(&window, "build_pending_changes_view");
    // Large unsaved-change count header.
    assert!(page.contains("pending_summary_title"));
    assert!(page.contains("title-2"));
    // Calm empty state.
    assert!(page.contains("No Pending Changes"));
    assert!(page.contains("emblem-ok-symbolic"));
    // Grouped rows: icon, friendly label, key · value subtitle, state
    // pill, revert, navigation chevron.
    assert!(page.contains("pending_group_caption"));
    assert!(page.contains("pending_change_subtitle"));
    assert!(page.contains("pending_change_kind"));
    assert!(page.contains("hyprland-settings-pending-badge-added"));
    assert!(page.contains("hyprland-settings-pending-badge-modified"));
    assert!(page.contains("edit-undo-symbolic"));
    assert!(page.contains("go-next-symbolic"));
    // Config diff preview with file header and +/− counts.
    assert!(page.contains("Config diff preview"));
    assert!(page.contains("unified_diff"));
    assert!(page.contains("next_save_config_text"));
    assert!(page.contains("hyprland-settings-diff-count-added"));
    assert!(page.contains("hyprland-settings-diff-count-removed"));
    // Review only: the page itself never writes.
    assert!(!page.contains("gated_scalar_save_live"));
    assert!(!page.contains("fs::write"));
}

#[test]
fn unset_switches_seed_from_official_defaults_not_flip_suggestions() {
    // The edit projection's proposed_value for booleans is a flip
    // suggestion (next_bool_value); seeding a switch from it rendered
    // unset rows inverted, and toggling then produced runtime no-ops the
    // pending ledger (correctly) ignored. Unset switches now seed from
    // the pinned trusted 0.55.4 defaults capture.
    use hyprland_settings::official_defaults::official_default_value;
    assert_eq!(
        official_default_value("decoration.dim_inactive"),
        Some("false")
    );
    assert_eq!(
        official_default_value("general.snap.border_overlap"),
        Some("false")
    );
    assert_eq!(official_default_value("general.border_size"), Some("1"));
    assert_eq!(official_default_value("not.a.setting"), None);

    let window = window_source();
    // Seeding now goes runtime-first through the shared helper; the
    // official default remains the fallback when runtime and config are
    // silent (see tests/pending_changes_reliability.rs for the ordering
    // guard).
    let inline = fn_slice(&window, "attach_inline_row_control");
    assert!(inline.contains("runtime_seed_initial_value"));
    assert!(fn_slice(&window, "runtime_seed_initial_value").contains("official_default_value"));
    assert_eq!(window.matches("official_default_value(").count() >= 2, true);
}

#[test]
fn preview_plumbing_feeds_the_ledger_without_new_write_paths() {
    let window = window_source();
    let apply = fn_slice(&window, "inline_preview_apply");
    assert!(apply.contains("register_pending_controller"));
    assert!(apply.contains("notify_pending_changed"));
    // Pending = previewing live with a value that differs from original.
    let snapshots = fn_slice(&window, "pending_change_snapshots");
    assert!(snapshots.contains("PreviewingLive"));
    assert!(snapshots.contains("original_runtime_value"));
    assert!(snapshots.contains("last_applied_value"));
    // Semantic no-op comparison replaced plain string equality.
    assert!(snapshots.contains("values_semantically_equal"));
    // The scalar-write preview helper is read-only.
    let scalar = fs::read_to_string("src/scalar_write.rs").expect("scalar_write reads");
    let preview = &scalar[scalar
        .find("fn preview_scalar_change_text")
        .expect("preview fn")
        ..scalar
            .find("pub fn apply_scalar_write_plan")
            .expect("apply fn")];
    assert!(!preview.contains("fs::write"));
    assert!(!preview.contains("atomic_write"));
}
