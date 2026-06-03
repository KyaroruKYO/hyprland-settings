use std::collections::BTreeSet;
use std::path::Path;

use anyhow::Result;
use hyprland_settings::export::ExportBundle;
use hyprland_settings::metadata::resolve_metadata_path_with_env;
use hyprland_settings::validation::validate_bundle;

const REQUIRED_STRUCTURED_FAMILIES: &[&str] = &[
    "hl.curve",
    "hl.animation",
    "hl.monitor",
    "hl.bind",
    "hl.device",
    "hl.gesture",
    "hl.permission",
];

const REQUIRED_VALUE_FAMILIES: &[&str] = &[
    "color",
    "vector_tuple",
    "gradient",
    "path_shader_path",
    "regex_rule_like_string",
    "mixed_freeform_string",
    "bezierish_numeric_list",
    "none",
    "structured_curve_animation",
];

fn load_default_bundle() -> Result<ExportBundle> {
    let resolution = resolve_metadata_path_with_env(None, None)?;
    ExportBundle::load(Path::new(&resolution.export_dir))
}

fn inventory_row_ids(bundle: &ExportBundle) -> BTreeSet<&str> {
    bundle
        .inventory
        .settings
        .iter()
        .map(|entry| entry.row_id.as_str())
        .collect()
}

#[test]
fn loads_default_export_bundle() -> Result<()> {
    load_default_bundle()?;
    Ok(())
}

#[test]
fn validates_default_export_bundle() -> Result<()> {
    let bundle = load_default_bundle()?;
    validate_bundle(&bundle)?;
    Ok(())
}

#[test]
fn summary_counts_match_expected() -> Result<()> {
    let bundle = load_default_bundle()?;
    let summary = validate_bundle(&bundle)?;

    assert_eq!(summary.inventory_rows, 341);
    assert_eq!(summary.official_scalar_covered, 341);
    assert_eq!(summary.official_scalar_total, 341);
    assert_eq!(summary.read_allowlist_rows, 232);
    assert_eq!(summary.non_read_rows, 109);
    assert_eq!(summary.preview_parser_needed_rows, 37);
    assert_eq!(summary.report_only_high_risk_rows, 72);
    assert_eq!(summary.safe_parsed_preview_candidates, 16);
    assert_eq!(summary.warning_preview_candidates, 16);
    assert_eq!(summary.deferred_parser_rows, 5);
    assert_eq!(summary.structured_family_count, 7);

    Ok(())
}

#[test]
fn required_animation_rows_have_expected_presence() -> Result<()> {
    let bundle = load_default_bundle()?;
    let row_ids = inventory_row_ids(&bundle);

    assert!(!row_ids.contains("animations.global_speed"));
    assert!(!row_ids.contains("animations.style"));
    assert!(row_ids.contains("animations.enabled"));
    assert!(row_ids.contains("animations.workspace_wraparound"));

    Ok(())
}

#[test]
fn write_candidate_remains_pending_only() -> Result<()> {
    let bundle = load_default_bundle()?;
    let candidates = &bundle.write_safety.active_candidates;

    assert_eq!(candidates.len(), 1);
    assert_eq!(candidates[0].row_id, "windows.snap.enabled");
    assert!(!candidates[0].executable);
    assert!(!candidates[0].command_generation_allowed);

    Ok(())
}

#[test]
fn structured_families_are_complete() -> Result<()> {
    let bundle = load_default_bundle()?;
    let family_ids: BTreeSet<_> = bundle
        .structured_families
        .families
        .iter()
        .map(|entry| entry.family_id.as_str())
        .collect();

    assert_eq!(family_ids.len(), REQUIRED_STRUCTURED_FAMILIES.len());
    for required in REQUIRED_STRUCTURED_FAMILIES {
        assert!(family_ids.contains(required), "{required} missing");
    }

    Ok(())
}

#[test]
fn value_families_are_complete() -> Result<()> {
    let bundle = load_default_bundle()?;
    let value_family_ids: BTreeSet<_> = bundle
        .value_families
        .items
        .iter()
        .map(|entry| entry.value_family.as_str())
        .collect();

    assert_eq!(value_family_ids.len(), REQUIRED_VALUE_FAMILIES.len());
    for required in REQUIRED_VALUE_FAMILIES {
        assert!(value_family_ids.contains(required), "{required} missing");
    }

    Ok(())
}
