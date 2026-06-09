use std::path::Path;

use anyhow::Result;
use hyprland_settings::export::ExportBundle;
use hyprland_settings::metadata::resolve_metadata_path_with_env;
use hyprland_settings::write_classification::{
    classify_inventory_entry, is_safe_writable_setting, ScalarWriteStatus, SAFE_WRITABLE_ROWS,
    SOURCE_BACKED_INPUT_ROWS,
};
use serde_json::Value;

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
fn safe_writable_rows_include_config_persistence_verified_rows() -> Result<()> {
    let row_ids = SAFE_WRITABLE_ROWS
        .iter()
        .map(|row| row.row_id)
        .collect::<std::collections::BTreeSet<_>>();
    let batch_a: Value = serde_json::from_str(include_str!(
        "../data/reports/batch-a-config-persistence-candidates.v0.55.2.json"
    ))?;
    let remaining_completion: Value = serde_json::from_str(include_str!(
        "../data/reports/remaining-scalar-completion.v0.55.2.json"
    ))?;

    assert_eq!(SAFE_WRITABLE_ROWS.len(), 340);
    for row in batch_a["rows"]
        .as_array()
        .expect("Batch A rows should be an array")
    {
        let row_id = row["rowId"].as_str().expect("rowId should exist");
        assert!(row_ids.contains(row_id), "{row_id} should be safe writable");
    }
    for row in remaining_completion["rows"]
        .as_array()
        .expect("remaining completion rows should be an array")
        .iter()
        .filter(|row| row["enabled"].as_bool() == Some(true))
    {
        let row_id = row["rowId"].as_str().expect("rowId should exist");
        assert!(row_ids.contains(row_id), "{row_id} should be safe writable");
    }
    assert!(is_safe_writable_setting("animations.enabled"));
    assert!(is_safe_writable_setting("appearance.blur.size"));
    assert!(is_safe_writable_setting("appearance.blur.passes"));
    assert!(is_safe_writable_setting("general.no_focus_fallback"));
    assert!(is_safe_writable_setting("misc.disable_hyprland_logo"));
    assert!(is_safe_writable_setting("input.follow_mouse"));
    assert!(is_safe_writable_setting("input.scroll_method"));
    assert!(is_safe_writable_setting("input.touchdevice.output"));
    assert!(is_safe_writable_setting("input.tablet.output"));
    assert!(is_safe_writable_setting("layout.selection"));
    assert!(is_safe_writable_setting("master.new_status"));
    assert!(is_safe_writable_setting("scrolling.direction"));
    for row_id in SOURCE_BACKED_INPUT_ROWS {
        assert!(
            is_safe_writable_setting(row_id),
            "{row_id} should be source-backed writable"
        );
    }
    assert!(!is_safe_writable_setting(
        "input.keyboard.resolve_binds_by_sym"
    ));
    Ok(())
}
