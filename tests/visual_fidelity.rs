//! Visual-fidelity acceptance guards: the specific visible failures from
//! the fidelity review cannot return. Normal tests only.

use std::fs;

use hyprland_settings::ux_presentation::{page_claims_row_in_tab, page_spec, section_for_row};

#[test]
fn no_generic_color_button_label_remains() {
    let window = fs::read_to_string("src/ui/window.rs").expect("window reads");
    assert!(
        !window.contains("\"Color…\"") && !window.contains("\"Color...\""),
        "no normal color row may show a generic Color button"
    );
    // The unparseable fallback is a checkered swatch, not a text button.
    assert!(window.contains("color_swatch_area(\"\", 44, 26)"));
}

#[test]
fn general_page_has_separate_hyprmod_sections() {
    // Curated placement: the five target sections, never a combined one.
    assert_eq!(
        section_for_row("general.gaps_in", "General", "General"),
        "Gaps"
    );
    assert_eq!(
        section_for_row("general.border_size", "General", "General"),
        "Borders"
    );
    assert_eq!(
        section_for_row("general.col.active_border", "General Col", "General"),
        "Border Colors"
    );
    assert_eq!(
        section_for_row("general.layout", "General", "General"),
        "Layout"
    );
    assert_eq!(
        section_for_row("general.snap.enabled", "General Snap", "General"),
        "Snap"
    );
    let presentation = fs::read_to_string("src/ux_presentation.rs").expect("reads");
    assert!(!presentation.contains("Gaps & Borders"));

    // The layout/snap/tearing keys reach General across tabs.
    let general = page_spec("general").expect("general page");
    assert!(page_claims_row_in_tab(
        general,
        "windows-layout",
        "general.layout"
    ));
    assert!(page_claims_row_in_tab(
        general,
        "windows-layout",
        "general.snap.enabled"
    ));
    assert!(page_claims_row_in_tab(
        general,
        "windows-layout",
        "general.allow_tearing"
    ));
    // And the Windows & Layout rest page no longer claims them.
    let rest = page_spec("windows-layout").expect("rest page");
    assert!(!page_claims_row_in_tab(
        rest,
        "windows-layout",
        "general.layout"
    ));
}

#[test]
fn header_title_follows_the_selected_page() {
    let window = fs::read_to_string("src/ui/window.rs").expect("window reads");
    assert!(window.contains("header_title.set_title(&item.label)"));
    assert!(
        !window.contains("adw::WindowTitle::new(\"Hyprland Settings\""),
        "the app name must not be the dominant page header"
    );
    // Settings pages show no duplicate giant content title.
    assert!(window.contains("tab_title.set_visible(false)"));
}

#[test]
fn normal_rows_show_no_routine_status_text() {
    let window = fs::read_to_string("src/ui/window.rs").expect("window reads");
    let row = &window[window.find("fn build_setting_row").expect("fn")
        ..window.find("fn inline_preview_apply").expect("next")];
    // Only the compact badge path exists; no chip label or current-status
    // string is rendered inline.
    assert!(row.contains("row_badge"));
    assert!(!row.contains("friendly_row_current_status(setting)"));
    assert!(!row.contains("status_chip_for_row(&setting.row_id).label()"));
    // The badge vocabulary excludes routine states.
    use hyprland_settings::ux_presentation::{row_badge, StatusChip};
    assert_eq!(row_badge(StatusChip::LivePreview, false), None);
    assert_eq!(row_badge(StatusChip::SaveOnly, false), None);
    assert_eq!(row_badge(StatusChip::Blocked, false), Some("Blocked"));
    assert_eq!(
        row_badge(StatusChip::HardwareRequired, false),
        Some("Hardware required")
    );
    assert_eq!(
        row_badge(StatusChip::LivePreview, true),
        Some("Needs attention")
    );
}

#[test]
fn search_is_a_sidebar_icon_not_a_text_button() {
    let window = fs::read_to_string("src/ui/window.rs").expect("window reads");
    assert!(!window.contains("ToggleButton::with_label(\"Search\")"));
    assert!(window.contains("search_toggle.set_icon_name(\"system-search-symbolic\")"));
    assert!(window.contains("sidebar_header.append(&search_toggle)"));
    // Ctrl+F wiring unchanged.
    assert!(window.contains("gtk::gdk::Key::f"));
    assert!(window.contains("search_toggle.set_active(true)"));
}

#[test]
fn color_picker_has_palette_and_custom_views() {
    let window = fs::read_to_string("src/ui/window.rs").expect("window reads");
    let picker = &window[window.find("fn open_color_stop_picker").expect("fn")
        ..window.find("fn attach_raw_color_entry").expect("next")];
    for expected in [
        "Pick a Color",
        "Cancel",
        "Select",
        "hyprland-settings-color-palette-grid",
        "hyprland-settings-color-custom-view",
        "hyprland-settings-color-sv-area",
        "hyprland-settings-color-hue",
        "hyprland-settings-color-alpha",
        "hsv_to_rgb",
        "render_color_like",
    ] {
        assert!(picker.contains(expected), "picker missing {expected}");
    }
    // Sidebar rows carry icons.
    assert!(window.contains("crate::ux_presentation::page_icon(&item.id)"));
}
