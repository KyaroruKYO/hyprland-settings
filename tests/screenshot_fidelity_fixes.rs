//! Guards for the screenshot-grounded fidelity fixes
//! (docs/MANUAL-VISUAL-REVIEW-FINDINGS.md): the visible failures the
//! user-captured review screenshots proved cannot silently return.

use std::fs;

use hyprland_settings::ux_presentation::{
    animation_record_display_name, animation_record_subtitle, page_claims_row_in_tab, page_spec,
    picker_palette_columns, section_for_row, ANIMATION_RECORD_GROUPS,
};

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
fn bezier_editor_is_an_in_window_dialog_not_a_toplevel() {
    let window = window_source();
    let editor = fn_slice(&window, "open_bezier_editor_dialog");
    // adw::Dialog renders inside the parent window — a tiling compositor
    // never sees a second client to tile.
    assert!(editor.contains("adw::Dialog::new()"));
    assert!(editor.contains("dialog.present(Some(parent))"));
    assert!(!editor.contains("gtk::Window::new()"));
    assert!(!editor.contains("window.present()"));
    // The old separate-window opener is gone entirely.
    assert!(!window.contains("fn open_bezier_editor_window"));
}

#[test]
fn animations_page_uses_the_six_reference_sections() {
    // Scalar rows head the page under General; the record groups follow in
    // the reference order.
    assert_eq!(
        section_for_row("animations.enabled", "Animations", "Animations"),
        "General"
    );
    let group_names: Vec<&str> = ANIMATION_RECORD_GROUPS
        .iter()
        .map(|(name, _)| *name)
        .collect();
    assert_eq!(
        group_names,
        vec![
            "Global",
            "Windows & Layers",
            "Fading",
            "Workspaces",
            "Other"
        ]
    );
    // The curated records land where the reference puts them.
    let find = |record: &str| {
        ANIMATION_RECORD_GROUPS
            .iter()
            .find(|(_, names)| names.contains(&record))
            .map(|(group, _)| *group)
    };
    assert_eq!(find("global"), Some("Global"));
    assert_eq!(find("windows"), Some("Windows & Layers"));
    assert_eq!(find("layers"), Some("Windows & Layers"));
    assert_eq!(find("fade"), Some("Fading"));
    assert_eq!(find("workspaces"), Some("Workspaces"));
    for other in ["border", "borderangle", "zoomFactor", "monitorAdded"] {
        assert_eq!(find(other), Some("Other"), "{other} lands in Other");
    }
    // Friendly names, not raw record identifiers.
    assert_eq!(animation_record_display_name("borderangle"), "Border Angle");
    assert_eq!(animation_record_display_name("zoomFactor"), "Zoom Factor");
    assert_eq!(
        animation_record_display_name("monitorAdded"),
        "Monitor Added"
    );
}

#[test]
fn animation_rows_show_friendly_subtitles_not_raw_record_text() {
    // The friendly subtitle shapes seen in the reference.
    assert_eq!(
        animation_record_subtitle("1", "4.00", "easeOutQuint", "", true),
        "4.0ds · easeOutQuint"
    );
    assert_eq!(
        animation_record_subtitle("1", "3.00", "easeOutQuint", "fade", true),
        "3.0ds · easeOutQuint · fade"
    );
    assert_eq!(
        animation_record_subtitle("0", "1.00", "", "", true),
        "disabled · 1.0ds · default"
    );
    assert_eq!(
        animation_record_subtitle("1", "8.00", "", "", false),
        "inherited · 8.0ds · default"
    );

    // The page builders never render the raw "(enabled 1, bezier …)" text.
    let window = window_source();
    for builder in [
        "append_animations_bezier_row",
        "append_animation_record_groups",
        "animation_record_row",
    ] {
        let slice = fn_slice(&window, builder);
        assert!(
            !slice.contains("current_value_text"),
            "{builder} must not render raw record text"
        );
        assert!(
            !slice.contains("(enabled"),
            "{builder} must not render raw enabled flags"
        );
    }
}

#[test]
fn animations_page_carries_no_proof_or_safety_prose() {
    let window = window_source();
    for builder in [
        "append_animations_bezier_row",
        "append_animation_record_groups",
        "animation_record_row",
    ] {
        let slice = fn_slice(&window, builder);
        for prose in [
            "Preview with recovery",
            "Keep changes",
            "Revert now",
            "Save previewed value",
            "Supervised preview",
            "Safe Live Save Mode",
            "Save writes once",
            "not yet supported",
        ] {
            assert!(
                !slice.contains(prose),
                "{builder} must not put {prose:?} on the normal page"
            );
        }
    }
    // The workbench (with those controls) lives on Safety Details instead.
    let safety = fn_slice(&window, "build_safety_details_view");
    assert!(safety.contains("structured_family_preview_controls_section(model)"));
}

#[test]
fn color_picker_is_an_opaque_dialog_not_a_popover() {
    let window = window_source();
    let picker = fn_slice(&window, "open_color_stop_picker");
    assert!(picker.contains("adw::Dialog::new()"));
    assert!(picker.contains("dialog.present(Some(parent))"));
    assert!(
        !picker.contains("gtk::Popover::new()"),
        "the main color picker must not be a popover"
    );
    // Header buttons are real buttons; Cancel is never a flat text label.
    assert!(picker.contains("gtk::Button::with_label(\"Cancel\")"));
    assert!(!picker.contains("cancel.add_css_class(\"flat\")"));
}

#[test]
fn palette_view_is_stacked_hue_columns_with_custom_swatches() {
    // Nine columns of five shades, reference-style.
    let palette = picker_palette_columns();
    assert_eq!(palette.len(), 9, "nine palette columns");
    for column in &palette {
        assert_eq!(column.len(), 5, "five shades per column");
    }
    // Ends with light and dark neutral columns (white-ish top, black last).
    let light = &palette[7][0];
    assert!(light.red > 0xe0 && light.green > 0xe0 && light.blue > 0xe0);
    let black = &palette[8][4];
    assert_eq!((black.red, black.green, black.blue), (0, 0, 0));

    let window = window_source();
    let picker = fn_slice(&window, "open_color_stop_picker");
    assert!(picker.contains("picker_palette_columns"));
    // Contiguous stacks: rounded ends only on the first and last shade.
    assert!(picker.contains("index == 0"));
    assert!(picker.contains("index + 1 == shade_count"));
    // Custom row: plus tile, session-remembered swatches, selected check.
    assert!(picker.contains("hyprland-settings-color-open-custom"));
    assert!(picker.contains("hyprland-settings-color-custom-swatch-row"));
    assert!(picker.contains("CUSTOM_PICKER_COLORS"));
    assert!(picker.contains("Selected checkmark"));
}

#[test]
fn custom_view_has_vertical_hue_smooth_sv_and_checkerboard_alpha() {
    let window = window_source();
    let picker = fn_slice(&window, "open_color_stop_picker");
    // Eyedropper placeholder | preview | hex top row.
    assert!(picker.contains("hyprland-settings-color-eyedropper"));
    assert!(picker.contains("color-select-symbolic"));
    // Vertical rainbow hue bar (gradient runs down the bar).
    let hue_section = &picker[picker.find("hyprland-settings-color-hue").expect("hue")..];
    assert!(hue_section.contains("LinearGradient::new(0.0, 0.0, 0.0, height)"));
    // Continuous SV plane: gradient-composed, not stepped cells.
    assert!(picker.contains("saturation_gradient"));
    assert!(picker.contains("value_gradient"));
    assert!(
        !picker.contains("let steps ="),
        "the SV area must not render stepped cells"
    );
    // Crosshair lines span the SV area.
    assert!(picker.contains("context.move_to(marker_x, 0.0)"));
    assert!(picker.contains("context.move_to(0.0, marker_y)"));
    // Checkerboard alpha slider.
    let alpha_section = &picker[picker.find("hyprland-settings-color-alpha").expect("alpha")..];
    assert!(alpha_section.contains("draw_checkerboard"));
}

#[test]
fn color_row_reads_swatch_remove_angle_then_plus() {
    let window = window_source();
    let color = fn_slice(&window, "attach_inline_color_control");
    // Bigger rounded tiles.
    assert!(color.contains("color_swatch_area(token, 44, 26)"));
    // Control order: stops (with removes), then angle, then add-stop last.
    let angle_position = color.find("-angle\"").expect("angle control");
    let add_position = color.find("-add-stop\"").expect("add control");
    assert!(
        angle_position < add_position,
        "the angle spinner renders before the trailing add button"
    );
    // Rounded checkered tiles come from the shared swatch painter.
    let swatch = fn_slice(&window, "live_swatch_area");
    assert!(swatch.contains("rounded_rect_path"));
    assert!(swatch.contains("draw_checkerboard"));
}

#[test]
fn numeric_value_entries_render_as_compact_spinners() {
    let window = window_source();
    let control = fn_slice(&window, "attach_inline_row_control");
    // Scalar numeric ValueEntry rows become −/+ spinners; non-scalar text
    // keeps the entry. Integer values render without decimals.
    assert!(control.contains("initial_value.trim().parse::<f64>()"));
    assert!(control.contains("value.fract() == 0.0"));
    assert!(control.contains("format!(\"{}\", value as i64)"));
    // The Slider arm reads integers as integers over wide ranges.
    assert!(control.contains("maximum - minimum > 3.0 && integral_value"));
}

#[test]
fn xwayland_and_ecosystem_pages_claim_their_rows() {
    // The rows live in the display/permissions model tabs; the pages must
    // claim them from there (the old system-tab claim matched nothing and
    // silently hid both pages).
    let xwayland = page_spec("xwayland").expect("xwayland page");
    assert_eq!(xwayland.source_tab, Some("display"));
    assert!(page_claims_row_in_tab(
        xwayland,
        "display",
        "xwayland.enabled"
    ));
    assert!(page_claims_row_in_tab(
        xwayland,
        "display",
        "xwayland.force_zero_scaling"
    ));
    let ecosystem = page_spec("ecosystem").expect("ecosystem page");
    assert_eq!(ecosystem.source_tab, Some("permissions"));
    assert!(page_claims_row_in_tab(
        ecosystem,
        "permissions",
        "ecosystem.no_update_news"
    ));
    // The Monitors rest page no longer swallows the xwayland rows.
    let monitors = page_spec("monitors").expect("monitors page");
    assert!(!page_claims_row_in_tab(
        monitors,
        "display",
        "xwayland.enabled"
    ));
}
