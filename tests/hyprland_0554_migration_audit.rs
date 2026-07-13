use std::collections::HashSet;
use std::fs;

use hyprland_settings::write_classification::{source_backed_numeric_bounds, SAFE_WRITABLE_ROWS};

const CAPTURED_DESCRIPTIONS: &str =
    "data/exports/hyprland-0.55.4/hyprctl-descriptions.v0.55.4.json";

/// The 0.55.4 migration audit, pinned as a regression test against the
/// captured trusted export (`hyprctl -j descriptions` from the official
/// 0.55.4 binary). To refresh the capture:
/// `hyprctl -j descriptions > data/exports/hyprland-0.55.4/hyprctl-descriptions.v0.55.4.json`
#[test]
fn v0552_model_is_name_and_bounds_compatible_with_captured_0554_descriptions() {
    let descriptions: serde_json::Value = serde_json::from_str(
        &fs::read_to_string(CAPTURED_DESCRIPTIONS).expect("captured descriptions read"),
    )
    .expect("captured descriptions parse");
    let live: Vec<(String, Option<f64>, Option<f64>)> = descriptions
        .as_array()
        .expect("array")
        .iter()
        .map(|entry| {
            (
                entry["name"].as_str().expect("name").replace(':', "."),
                entry["min"].as_f64(),
                entry["max"].as_f64(),
            )
        })
        .collect();

    let live_names: HashSet<&str> = live.iter().map(|(name, _, _)| name.as_str()).collect();
    let model_names: HashSet<&str> = SAFE_WRITABLE_ROWS
        .iter()
        .map(|row| row.official_setting)
        .collect();

    assert_eq!(live.len(), 341, "0.55.4 exposes 341 options");
    assert_eq!(model_names.len(), 341, "the model has 341 settings");
    let added: Vec<_> = live_names.difference(&model_names).collect();
    let removed: Vec<_> = model_names.difference(&live_names).collect();
    assert!(
        added.is_empty(),
        "options in 0.55.4 missing from the model: {added:?}"
    );
    assert!(
        removed.is_empty(),
        "model options absent in 0.55.4: {removed:?}"
    );

    // Numeric bounds in the model must match the live binary's bounds.
    let mut compared = 0;
    for row in SAFE_WRITABLE_ROWS {
        if let Some(bounds) = source_backed_numeric_bounds(row.row_id) {
            if let Some((_, Some(min), Some(max))) = live
                .iter()
                .find(|(name, _, _)| name == row.official_setting)
            {
                compared += 1;
                assert!(
                    (bounds.min - min).abs() < 1e-9 && (bounds.max - max).abs() < 1e-9,
                    "{}: model bounds [{}, {}] differ from 0.55.4 [{min}, {max}]",
                    row.official_setting,
                    bounds.min,
                    bounds.max
                );
            }
        }
    }
    assert!(
        compared >= 70,
        "expected to compare most bounded rows, got {compared}"
    );
}
