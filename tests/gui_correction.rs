//! Correction-pass guards: tooltips stay out of the normal UI, section
//! headings live outside the cards, the sidebar uses the target page
//! names, Startup pages exist, the Bezier editor lives under Animations,
//! color rows are stop-based, every scalar row stays reachable exactly
//! once, and all seven families are mapped — with zero classification
//! change. Normal tests only.

use std::collections::BTreeSet;
use std::fs;
use std::path::Path;

use anyhow::Result;
use hyprland_settings::config_discovery::{ConfigDiscovery, ConfigDiscoveryStatus};
use hyprland_settings::current_config::CurrentConfigSnapshot;
use hyprland_settings::export::ExportBundle;
use hyprland_settings::metadata::resolve_metadata_path_with_env;
use hyprland_settings::ui::model::UiProjection;
use hyprland_settings::ux_presentation::{
    page_claims_row_in_tab, page_source_tabs, page_spec, section_display_name, SIDEBAR_PAGE_LAYOUT,
};
use hyprland_settings::validation::validate_bundle;
use hyprland_settings::write_classification::SAFE_WRITABLE_ROWS;

fn load_projection() -> Result<UiProjection> {
    let resolution = resolve_metadata_path_with_env(None, None)?;
    let bundle = ExportBundle::load(Path::new(&resolution.export_dir))?;
    let summary = validate_bundle(&bundle)?;
    Ok(UiProjection::from_bundle(
        &bundle,
        &summary,
        ConfigDiscovery {
            status: ConfigDiscoveryStatus::Missing,
            attempted_paths: Vec::new(),
        },
        CurrentConfigSnapshot::read_unavailable("test fixture has no live config"),
    ))
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
fn normal_ui_builders_carry_no_tooltips() {
    let window = fs::read_to_string("src/ui/window.rs").expect("window reads");
    // Every normal user-facing builder is tooltip-free. The only remaining
    // tooltip call sites are the documented harness/accessibility
    // exceptions (sidebar navigation names, setting-row accessibility
    // text, detail-pane identification) and the review-card descriptors on
    // the Safety Details surfaces.
    // Identification for the harness moved to accessibility labels: the
    // row builder carries no tooltip at all, only an accessible name.
    let row_builder = fn_slice(&window, "build_setting_row");
    assert_eq!(row_builder.matches("set_tooltip_text").count(), 0);
    assert!(row_builder.contains("setting_row_accessibility_text"));
    assert!(row_builder.contains("gtk::accessible::Property::Label"));

    for builder in [
        "build_setting_row",
        "build_sidebar",
        "build_detail_panel",
        "attach_inline_row_control",
        "attach_inline_color_control",
        "open_color_stop_picker",
        "attach_raw_color_entry",
        "build_profiles_view",
        "build_layouts_view",
        "empty_state_view",
        "structured_locked_list_view",
        "build_dashboard_view",
        "build_dashboard_card",
        "dashboard_cards",
        "build_config_view",
        "append_animations_bezier_row",
        "append_animation_record_groups",
        "animation_record_row",
        "animation_record_menu_box",
        "open_bezier_editor_dialog",
        "bezier_graph_area",
        "render_settings_view",
        "append_structured_entries_card",
        "inline_preview_apply",
    ] {
        assert!(
            !fn_slice(&window, builder).contains("set_tooltip_text"),
            "normal UI builder {builder} must not set tooltips"
        );
    }
    // Identification still exists — as accessible names, not tooltips.
    assert!(window.contains("Navigation: "));
    assert!(window.contains("setting_row_accessibility_text"));
    assert!(window.contains("detail_pane_accessibility_text"));
}

#[test]
fn section_headings_render_outside_the_cards() {
    let window = fs::read_to_string("src/ui/window.rs").expect("window reads");
    let render = fn_slice(&window, "render_settings_view");
    // Heading label appended to the sections column, then a separate
    // boxed-list card of rows appended after it — never a header row
    // inside the card.
    assert!(render.contains("sections_box.append(&heading)"));
    assert!(render.contains("sections_box.append(&list)"));
    assert!(render.contains("list.add_css_class(\"boxed-list\")"));
    assert!(render.contains("section_for_row"));
    assert!(
        !render.contains("set_header_func"),
        "section headings are standalone labels, not list headers"
    );

    // The known awkward generated names map to natural titles.
    assert_eq!(
        section_display_name("Decoration Blur", "Decoration"),
        "Blur"
    );
    assert_eq!(
        section_display_name("Decoration Shadow", "Decoration"),
        "Shadow"
    );
    assert_eq!(
        section_display_name("General Col", "General"),
        "Border Colors"
    );
    assert_eq!(section_display_name("General Snap", "General"), "Snap");
    assert_eq!(
        section_display_name("Decoration", "Decoration"),
        "Rounding and Opacity"
    );
}

#[test]
fn sidebar_matches_the_target_page_names() {
    let labels: Vec<&str> = SIDEBAR_PAGE_LAYOUT
        .iter()
        .flat_map(|category| category.pages.iter().map(|page| page.label))
        .collect();
    for expected in [
        "General",
        "Decoration",
        "Animations",
        "Cursor",
        "Keybinds",
        "Devices",
        "Gestures",
        "Monitors",
        "Workspaces",
        "Layouts",
        "Window Rules",
        "Layer Rules",
        "Autostart",
        "Env Variables",
        "XWayland",
        "Ecosystem",
        "Profiles",
        "Settings",
    ] {
        assert!(labels.contains(&expected), "sidebar page {expected} exists");
    }
    // Correct capitalization is data, not chance.
    assert!(!labels.contains(&"Xwayland"));
    assert!(!labels.contains(&"Env variables"));
    assert!(!labels.contains(&"Appearance"));
    assert!(!labels.contains(&"Config"));
}

#[test]
fn startup_pages_exist_and_add_no_write_paths() {
    let window = fs::read_to_string("src/ui/window.rs").expect("window reads");
    assert!(window.contains("(\"autostart\", autostart_view.clone())"));
    assert!(window.contains("(\"env-variables\", env_variables_view.clone())"));
    assert!(window.contains("hyprland-settings-autostart-content"));
    assert!(window.contains("hyprland-settings-env-variables-content"));
    // Honest empty states: entries are explained, nothing is invented or
    // editable, and the locked list builder has no write affordance.
    assert!(window.contains("A safe read-only view is not available yet"));
    let locked = fn_slice(&window, "structured_locked_list_view");
    for forbidden in ["connect_clicked", "Entry::new", "gated_", "fs::write"] {
        assert!(
            !locked.contains(forbidden),
            "locked list must not contain {forbidden}"
        );
    }
}

#[test]
fn bezier_editor_lives_under_animations_not_settings() {
    let window = fs::read_to_string("src/ui/window.rs").expect("window reads");
    // Animations page: editor entry row (icon + chevron) and record rows
    // with menu buttons.
    let bezier_row = fn_slice(&window, "append_animations_bezier_row");
    assert!(bezier_row.contains("Bezier Curve Editor"));
    assert!(bezier_row.contains("Create and manage animation curves"));
    assert!(bezier_row.contains("draw-arc-symbolic"));
    assert!(bezier_row.contains("go-next-symbolic"));
    assert!(bezier_row.contains("open_bezier_editor_dialog"));
    let record_row = fn_slice(&window, "animation_record_row");
    assert!(record_row.contains("open-menu-symbolic"));
    assert!(record_row.contains("hyprland-settings-animation-menu-"));

    // The Settings (config) page no longer hosts the pickers.
    let config = fn_slice(&window, "build_config_view");
    assert!(!config.contains("structured_family_preview_controls_section"));
    assert!(!config.contains("Bezier"));

    // The record menu drives only the proven gates: fixed existing record,
    // existing curves, style untouched, gated save.
    let menu = fn_slice(&window, "animation_record_menu_box");
    assert!(menu.contains("FamilyRecordPreviewController::new_live"));
    assert!(menu.contains("save_picked_record_with_precondition_live"));
    assert!(menu.contains("save_precondition"));
    assert!(menu.contains("PickedRecordValues::AnimationRecord"));
    assert!(!menu.contains("style"));

    // The editor dialog shows the curve graph and the proven curve picker
    // — presented in-window, never as a separate toplevel.
    let editor = fn_slice(&window, "open_bezier_editor_dialog");
    assert!(editor.contains("bezier_graph_area()"));
    assert!(editor.contains("curve_record_picker_group"));
    assert!(editor.contains("adw::Dialog::new()"));
    assert!(!editor.contains("gtk::Window::new()"));
}

#[test]
fn color_rows_are_stop_based_with_discard_and_angle() {
    let window = fs::read_to_string("src/ui/window.rs").expect("window reads");
    let color = fn_slice(&window, "attach_inline_color_control");
    // Stop swatches, per-stop remove, add-stop, angle stepper, discard
    // back-arrow, all rebuilt from the raw token state.
    assert!(color.contains("-stop-"));
    assert!(color.contains("-remove-"));
    assert!(color.contains("-add-stop"));
    assert!(color.contains("-angle"));
    assert!(color.contains("-discard"));
    assert!(color.contains("edit-undo-symbolic"));
    assert!(color.contains("edit-clear-symbolic"));
    assert!(color.contains("list-add-symbolic"));
    assert!(color.contains("token_count > 1"));
    assert!(color.contains("attach_raw_color_entry"));
    // Swatches are checkered for alpha visibility.
    let swatch = fn_slice(&window, "live_swatch_area");
    assert!(swatch.contains("draw_checkerboard"));
    // The per-stop picker validates and preserves the raw token.
    let picker = fn_slice(&window, "open_color_stop_picker");
    assert!(picker.contains("parse_hyprland_color"));
    assert!(picker.contains("Cancel"));
}

#[test]
fn every_scalar_row_is_reachable_exactly_once() -> Result<()> {
    let projection = load_projection()?;
    let mut claimed: BTreeSet<String> = BTreeSet::new();
    let mut duplicates = Vec::new();
    for category in SIDEBAR_PAGE_LAYOUT {
        for page in category.pages {
            for tab in page_source_tabs(page) {
                for setting in projection.settings_for_tab(tab) {
                    if page_claims_row_in_tab(page, tab, &setting.official_setting) {
                        if !claimed.insert(setting.row_id.clone()) {
                            duplicates.push((page.id, setting.row_id.clone()));
                        }
                    }
                }
            }
        }
    }
    assert!(duplicates.is_empty(), "rows claimed twice: {duplicates:?}");
    assert_eq!(
        claimed.len(),
        341,
        "every scalar row lands on exactly one page"
    );
    assert_eq!(SAFE_WRITABLE_ROWS.len(), 341);
    Ok(())
}

#[test]
fn all_seven_families_are_mapped_to_pages() {
    let window = fs::read_to_string("src/ui/window.rs").expect("window reads");
    // Read-only structured entries render on the matching pages; the
    // animation/curve families live on Animations (picker + editor).
    let mapping = fn_slice(&window, "page_structured_family");
    for (page, family) in [
        ("keybinds", "hl.bind"),
        ("monitors", "hl.monitor"),
        ("gestures", "hl.gesture"),
        ("devices", "hl.device"),
        ("ecosystem", "hl.permission"),
    ] {
        assert!(mapping.contains(page), "family page {page} mapped");
        assert!(mapping.contains(family), "family {family} mapped");
    }
    assert!(
        window.contains("\"hl.windowrule\""),
        "window rules page shows entries"
    );
    let groups = fn_slice(&window, "append_animation_record_groups");
    assert!(groups.contains("list_animation_records_live"));
    let editor = fn_slice(&window, "open_bezier_editor_dialog");
    assert!(editor.contains("curve_record_picker_group"));

    // Page specs exist for the two standalone family shells too.
    assert!(page_spec("window-rules").is_some());
    assert!(page_spec("layer-rules").is_some());
}
