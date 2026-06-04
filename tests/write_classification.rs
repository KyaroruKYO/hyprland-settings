use std::path::Path;

use anyhow::Result;
use hyprland_settings::export::ExportBundle;
use hyprland_settings::metadata::resolve_metadata_path_with_env;
use hyprland_settings::write_classification::{
    classify_inventory_entry, is_safe_writable_setting, ScalarWriteStatus, SAFE_WRITABLE_ROWS,
};

fn load_bundle() -> Result<ExportBundle> {
    let resolution = resolve_metadata_path_with_env(None, None)?;
    ExportBundle::load(Path::new(&resolution.export_dir))
}

#[test]
fn every_inventory_row_has_write_classification() -> Result<()> {
    let bundle = load_bundle()?;
    let classifications = bundle
        .inventory
        .settings
        .iter()
        .map(classify_inventory_entry)
        .collect::<Vec<_>>();

    assert_eq!(classifications.len(), 341);
    assert_eq!(
        classifications
            .iter()
            .filter(|classification| classification.status == ScalarWriteStatus::SafeWritable)
            .count(),
        SAFE_WRITABLE_ROWS.len()
    );
    for classification in &classifications {
        if classification.status == ScalarWriteStatus::SafeWritable {
            assert!(classification.blocker.is_none());
            assert!(is_safe_writable_setting(&classification.row_id));
        } else {
            assert!(
                classification.blocker.is_some(),
                "{} must explain why it is not writable",
                classification.row_id
            );
        }
    }

    Ok(())
}

#[test]
fn safe_writable_rows_are_the_reviewed_toggle_subset() {
    let row_ids = SAFE_WRITABLE_ROWS
        .iter()
        .map(|row| row.row_id)
        .collect::<Vec<_>>();

    assert_eq!(
        row_ids,
        vec![
            "appearance.blur.enabled",
            "appearance.blur.size",
            "appearance.blur.brightness",
            "appearance.blur.contrast",
            "appearance.shadow.enabled",
            "appearance.shadow.range",
            "appearance.shadow.render_power",
            "decoration.shadow.color",
            "decoration.shadow.color_inactive",
            "decoration.shadow.offset",
            "appearance.gaps_in",
            "appearance.gaps_out",
            "appearance.border_size",
            "appearance.rounding",
            "appearance.active_opacity",
            "appearance.inactive_opacity",
            "animations.enabled",
            "windows.snap.enabled",
            "windows.snap.window_gap",
            "windows.snap.monitor_gap",
            "input.pointer_sensitivity",
            "input.accel_profile",
            "input.tablet.region_position",
            "input.tablet.region_size",
            "input.tablet.active_area_size",
            "input.tablet.active_area_position",
            "decoration.glow.color",
            "decoration.glow.color_inactive",
            "group.groupbar.text_color",
            "group.groupbar.text_color_inactive",
            "group.groupbar.text_color_locked_active",
            "group.groupbar.text_color_locked_inactive",
            "group.groupbar.font_family",
            "misc.col.splash",
            "misc.background_color",
            "misc.font_family",
            "misc.splash_font_family",
            "layout.single_window_aspect_ratio"
        ]
    );
    assert!(is_safe_writable_setting("animations.enabled"));
    assert!(is_safe_writable_setting("appearance.blur.size"));
    assert!(!is_safe_writable_setting("appearance.glow.range"));
}
