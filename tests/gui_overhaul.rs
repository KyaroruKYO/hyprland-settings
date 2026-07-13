//! GUI overhaul guards: centered settings-first shell, on-demand detail,
//! inline controls, color/gradient presentation, upgraded search, the
//! Profiles empty state, the merged Layouts page, and formatting-only
//! label fallbacks — all with zero behavior or classification change.
//! Normal tests only.

use std::path::Path;
use std::{collections::BTreeSet, fs};

use anyhow::Result;
use hyprland_settings::config_discovery::{ConfigDiscovery, ConfigDiscoveryStatus};
use hyprland_settings::current_config::CurrentConfigSnapshot;
use hyprland_settings::export::ExportBundle;
use hyprland_settings::metadata::resolve_metadata_path_with_env;
use hyprland_settings::runtime_preview_ui_projection::{
    runtime_preview_ui_row_state, RuntimePreviewUiControlKind,
};
use hyprland_settings::search::search_projection;
use hyprland_settings::ui::model::UiProjection;
use hyprland_settings::ux_presentation::{
    fallback_display_label, parse_hyprland_color, parse_hyprland_gradient, ParsedColor,
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

#[test]
fn inline_controls_cover_every_default_previewable_control_kind() {
    // Every default-previewable row resolves a control kind the inline
    // builder handles; nothing else gets an inline control.
    let mut inline = 0;
    let mut chip_only = 0;
    for row in SAFE_WRITABLE_ROWS {
        let Some(state) = runtime_preview_ui_row_state(row.row_id) else {
            chip_only += 1;
            continue;
        };
        if state.preview_enabled {
            inline += 1;
            assert!(
                !matches!(state.control_kind, RuntimePreviewUiControlKind::NoControl),
                "{} is previewable but has no control kind",
                row.row_id
            );
        } else {
            chip_only += 1;
        }
    }
    assert_eq!(inline + chip_only, 341);
    assert_eq!(
        inline, 135,
        "the 135 default-previewable rows get inline controls"
    );

    let window = fs::read_to_string("src/ui/window.rs").expect("window reads");
    assert!(window.contains("fn attach_inline_row_control"));
    assert!(window.contains("fn inline_preview_apply"));
    // Inline controls ride the existing reversible preview path only.
    assert!(window.contains("RuntimePreviewUiController::new_live(&row_id)"));
    assert!(window.contains("register_preview_controller(&controller)"));
}

#[test]
fn color_rows_are_fully_accounted_for_with_swatch_and_picker() {
    // Account for every color-capable row.
    let color_rows: Vec<&str> = SAFE_WRITABLE_ROWS
        .iter()
        .filter(|row| {
            runtime_preview_ui_row_state(row.row_id)
                .map(|state| state.control_kind == RuntimePreviewUiControlKind::ColorEntry)
                .unwrap_or(false)
        })
        .map(|row| row.row_id)
        .collect();
    assert_eq!(
        color_rows.len(),
        22,
        "22 color-capable rows: {color_rows:?}"
    );

    // Swatch + validated picker exist and fail closed.
    let window = fs::read_to_string("src/ui/window.rs").expect("window reads");
    assert!(window.contains("fn attach_inline_color_control"));
    assert!(window.contains("fn color_swatch_area"));
    assert!(window.contains("fn live_swatch_area"));
    assert!(window.contains("Apply preview"));
    assert!(window.contains("apply_button.set_sensitive(valid)"));

    // Color parsing: valid forms parse, invalid ones fail closed.
    assert_eq!(
        parse_hyprland_color("rgba(33ccffee)"),
        Some(ParsedColor {
            red: 0x33,
            green: 0xcc,
            blue: 0xff,
            alpha: 0xee
        })
    );
    assert_eq!(
        parse_hyprland_color("rgb(11ee11)"),
        Some(ParsedColor {
            red: 0x11,
            green: 0xee,
            blue: 0x11,
            alpha: 0xff
        })
    );
    assert_eq!(
        parse_hyprland_color("0x80ff0000"),
        Some(ParsedColor {
            red: 0xff,
            green: 0x00,
            blue: 0x00,
            alpha: 0x80
        })
    );
    for invalid in [
        "rgba(33ccff)",
        "rgb(11ee11aa)",
        "0xff00",
        "red",
        "rgba(gggggggg)",
        "",
        "rgba(33ccffee) extra",
    ] {
        assert_eq!(
            parse_hyprland_color(invalid),
            None,
            "{invalid} must fail closed"
        );
    }
}

#[test]
fn gradients_parse_only_supported_syntax_and_fail_closed() {
    // Two or more colors with an optional trailing angle are gradient-like.
    let gradient = parse_hyprland_gradient("rgba(33ccffee) rgba(00ff99ee) 45deg")
        .expect("two-color gradient parses");
    assert_eq!(gradient.0.len(), 2);
    assert_eq!(gradient.1, Some(45));
    assert!(parse_hyprland_gradient("rgb(112233) 0xffaabbcc").is_some());

    // Everything else fails closed: no invented syntax.
    for invalid in [
        "rgba(33ccffee)",                       // single color is not a gradient
        "rgba(33ccffee) 45deg rgba(00ff99ee)",  // color after angle
        "rgba(33ccffee) sideways",              // unknown token
        "45deg",                                // angle only
        "",                                     // empty
        "rgba(33ccffee) rgba(00ff99ee) 45degg", // bad angle token
    ] {
        assert!(
            parse_hyprland_gradient(invalid).is_none(),
            "{invalid} must fail closed"
        );
    }
}

#[test]
fn search_matches_friendly_labels_raw_keys_and_categories() -> Result<()> {
    let projection = load_projection()?;

    let expect_hit = |query: &str, row_id: &str| {
        let view = search_projection(&projection, "appearance", query);
        assert!(
            view.results
                .iter()
                .any(|result| result.setting.row_id == row_id),
            "query {query:?} should find {row_id}"
        );
    };

    // Friendly label, description word, raw key in both spellings,
    // separator/case normalization, and category/group terms.
    expect_hit("Inner gaps", "appearance.gaps_in");
    expect_hit("inner GAPS", "appearance.gaps_in");
    expect_hit("general.gaps_in", "appearance.gaps_in");
    expect_hit("general:gaps_in", "appearance.gaps_in");
    expect_hit("gaps", "appearance.gaps_in");
    expect_hit("blur", "appearance.blur.enabled");
    expect_hit("Enable blur", "appearance.blur.enabled");
    expect_hit("decoration:blur:enabled", "appearance.blur.enabled");

    // The whole model is indexed.
    let all = search_projection(&projection, "appearance", "");
    assert!(all.results.len() >= 40, "page view renders its rows");
    let everywhere = search_projection(&projection, "appearance", "hyprland");
    assert!(everywhere.results.len() <= 341);
    Ok(())
}

#[test]
fn profiles_page_is_a_friendly_inert_empty_state() {
    let window = fs::read_to_string("src/ui/window.rs").expect("window reads");
    let profiles = &window[window
        .find("fn build_profiles_view")
        .expect("profiles view")
        ..window.find("fn build_layouts_view").expect("layouts view")];
    assert!(profiles.contains("No Profiles"));
    assert!(profiles.contains("Switching is not enabled yet"));
    assert!(profiles.contains("Save Current as Profile"));
    assert!(profiles.contains("action.set_sensitive(false)"));
    // No production behavior: nothing here touches files, symlinks, or the
    // compositor.
    for forbidden in ["symlink", "fs::", "connect_clicked", "hyprctl"] {
        assert!(
            !profiles.contains(forbidden),
            "profiles view must not contain {forbidden}"
        );
    }
}

#[test]
fn layouts_page_merges_dwindle_master_scrolling_presentation_only() -> Result<()> {
    let window = fs::read_to_string("src/ui/window.rs").expect("window reads");
    let layouts = &window[window.find("fn build_layouts_view").expect("layouts view")
        ..window.find("fn config_path_summary").expect("end")];
    for tab in ["Dwindle", "Master", "Scrolling"] {
        assert!(layouts.contains(tab), "layouts page has the {tab} tab");
    }
    assert!(layouts.contains("build_setting_row(result, false)"));

    // The rows the page shows are the same model rows with the same raw
    // keys and classifications (presentation only). Dwindle and master
    // rows exist; scrolling may not on this model version.
    let projection = load_projection()?;
    let layout_rows: Vec<_> = projection
        .settings_for_tab("windows-layout")
        .into_iter()
        .filter(|setting| {
            setting.row_id.starts_with("dwindle.")
                || setting.row_id.starts_with("master.")
                || setting.row_id.starts_with("scrolling.")
        })
        .collect();
    assert!(
        layout_rows.len() >= 20,
        "the merged page reaches the layout rows ({} found)",
        layout_rows.len()
    );
    for setting in &layout_rows {
        assert!(!setting.official_setting.is_empty());
    }
    Ok(())
}

#[test]
fn fallback_labels_are_formatting_only() {
    // The page-name prefix is stripped; nothing else changes; labels that
    // are only the page name stay untouched.
    assert_eq!(
        fallback_display_label("Appearance Blur Enabled", "Appearance"),
        "Blur Enabled"
    );
    assert_eq!(
        fallback_display_label("Cursor Hide On Key Press", "Cursor"),
        "Hide On Key Press"
    );
    assert_eq!(
        fallback_display_label("Appearance", "Appearance"),
        "Appearance"
    );
    assert_eq!(
        fallback_display_label("Unrelated Label", "Appearance"),
        "Unrelated Label"
    );
}

#[test]
fn overhaul_changes_no_classification_and_adds_no_unsafe_paths() {
    // Classification counts unchanged.
    assert_eq!(SAFE_WRITABLE_ROWS.len(), 341);
    let ids: BTreeSet<&str> = SAFE_WRITABLE_ROWS.iter().map(|row| row.row_id).collect();
    assert_eq!(ids.len(), 341);

    // The presentation additions introduce no process/reload/write paths.
    let presentation = fs::read_to_string("src/ux_presentation.rs").expect("presentation reads");
    for forbidden in ["Command::new", "std::process", "fs::", "hyprctl", "gated_"] {
        assert!(
            !presentation.contains(forbidden),
            "presentation module must not contain {forbidden}"
        );
    }
    let window = fs::read_to_string("src/ui/window.rs").expect("window reads");
    assert!(window.contains("gated_scalar_save_live("));
    assert!(!window.contains("apply_setting_change("));
    assert!(!window.contains("\"reload\""));
}
