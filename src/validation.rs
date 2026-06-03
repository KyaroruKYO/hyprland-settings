use std::collections::{BTreeSet, HashMap};

use anyhow::{anyhow, bail, ensure, Result};

use crate::export::{ExportBundle, HYPRLAND_VERSION, SCHEMA_VERSION};

const EXPECTED_INVENTORY_ROWS: usize = 341;
const EXPECTED_OFFICIAL_SCALAR_COVERED: usize = 341;
const EXPECTED_OFFICIAL_SCALAR_TOTAL: usize = 341;
const EXPECTED_READ_ALLOWLIST_ROWS: usize = 232;
const EXPECTED_NON_READ_ROWS: usize = 109;
const EXPECTED_PREVIEW_PARSER_NEEDED_ROWS: usize = 37;
const EXPECTED_REPORT_ONLY_HIGH_RISK_ROWS: usize = 72;
const EXPECTED_SAFE_PARSED_PREVIEW_CANDIDATES: usize = 16;
const EXPECTED_WARNING_PREVIEW_CANDIDATES: usize = 16;
const EXPECTED_DEFERRED_PARSER_ROWS: usize = 5;
const EXPECTED_ACTIVE_WRITE_CANDIDATES: usize = 1;
const EXPECTED_ACTIVE_WRITE_CANDIDATE: &str = "windows.snap.enabled";

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

#[derive(Debug)]
pub struct ValidationSummary {
    pub inventory_rows: usize,
    pub official_scalar_covered: usize,
    pub official_scalar_total: usize,
    pub read_allowlist_rows: usize,
    pub non_read_rows: usize,
    pub preview_parser_needed_rows: usize,
    pub report_only_high_risk_rows: usize,
    pub safe_parsed_preview_candidates: usize,
    pub warning_preview_candidates: usize,
    pub deferred_parser_rows: usize,
    pub active_write_candidate_ids: Vec<String>,
    pub structured_family_count: usize,
}

pub fn validate_bundle(bundle: &ExportBundle) -> Result<ValidationSummary> {
    validate_header(
        "manifest",
        &bundle.manifest.hyprland_version,
        bundle.manifest.schema_version,
        &bundle.manifest.artifact_kind,
        "export-manifest",
    )?;
    validate_header(
        "inventory",
        &bundle.inventory.hyprland_version,
        bundle.inventory.schema_version,
        &bundle.inventory.artifact_kind,
        "inventory",
    )?;
    validate_header(
        "read allowlist",
        &bundle.read_allowlist.hyprland_version,
        bundle.read_allowlist.schema_version,
        &bundle.read_allowlist.artifact_kind,
        "read-allowlist",
    )?;
    validate_header(
        "non-read classifications",
        &bundle.non_read.hyprland_version,
        bundle.non_read.schema_version,
        &bundle.non_read.artifact_kind,
        "non-read-classifications",
    )?;
    validate_header(
        "value families",
        &bundle.value_families.hyprland_version,
        bundle.value_families.schema_version,
        &bundle.value_families.artifact_kind,
        "value-families",
    )?;
    validate_header(
        "preview candidates",
        &bundle.preview_candidates.hyprland_version,
        bundle.preview_candidates.schema_version,
        &bundle.preview_candidates.artifact_kind,
        "preview-candidates",
    )?;
    validate_header(
        "write safety",
        &bundle.write_safety.hyprland_version,
        bundle.write_safety.schema_version,
        &bundle.write_safety.artifact_kind,
        "write-safety",
    )?;
    validate_header(
        "structured families",
        &bundle.structured_families.hyprland_version,
        bundle.structured_families.schema_version,
        &bundle.structured_families.artifact_kind,
        "structured-families-design-index",
    )?;
    validate_header(
        "source provenance",
        &bundle.source_provenance.hyprland_version,
        bundle.source_provenance.schema_version,
        &bundle.source_provenance.artifact_kind,
        "source-provenance-index",
    )?;

    let inventory_ids = collect_unique_inventory_ids(bundle)?;
    let read_ids = collect_row_ids(
        "read allowlist",
        bundle
            .read_allowlist
            .items
            .iter()
            .map(|entry| entry.row_id.as_str()),
    )?;
    let non_read_ids = collect_row_ids(
        "non-read classifications",
        bundle
            .non_read
            .items
            .iter()
            .map(|entry| entry.row_id.as_str()),
    )?;
    let preview_ids = collect_row_ids(
        "preview candidates",
        bundle
            .preview_candidates
            .items
            .iter()
            .map(|entry| entry.row_id.as_str()),
    )?;
    let write_ids = collect_row_ids(
        "write safety active candidates",
        bundle
            .write_safety
            .active_candidates
            .iter()
            .map(|entry| entry.row_id.as_str()),
    )?;

    validate_counts(bundle)?;
    validate_required_rows(&inventory_ids)?;
    validate_row_links(
        &inventory_ids,
        &read_ids,
        &non_read_ids,
        &preview_ids,
        &write_ids,
    )?;
    validate_official_setting_consistency(bundle)?;
    validate_write_safety(bundle)?;
    validate_structured_families(bundle)?;
    validate_value_families(bundle)?;

    Ok(ValidationSummary {
        inventory_rows: bundle.inventory.settings.len(),
        official_scalar_covered: bundle.inventory.counts.official_scalar_coverage.covered,
        official_scalar_total: bundle.inventory.counts.official_scalar_coverage.total,
        read_allowlist_rows: bundle.read_allowlist.items.len(),
        non_read_rows: bundle.non_read.items.len(),
        preview_parser_needed_rows: bundle
            .non_read
            .items
            .iter()
            .filter(|entry| entry.unresolved_category == "preview-parser-needed")
            .count(),
        report_only_high_risk_rows: bundle
            .non_read
            .items
            .iter()
            .filter(|entry| entry.unresolved_category == "report-only-high-risk")
            .count(),
        safe_parsed_preview_candidates: bundle
            .preview_candidates
            .items
            .iter()
            .filter(|entry| entry.status == "safe-parsed-preview")
            .count(),
        warning_preview_candidates: bundle
            .preview_candidates
            .items
            .iter()
            .filter(|entry| entry.status == "warning-preview")
            .count(),
        deferred_parser_rows: bundle
            .preview_candidates
            .items
            .iter()
            .filter(|entry| entry.status == "deferred-parser-preview")
            .count(),
        active_write_candidate_ids: bundle
            .write_safety
            .active_candidates
            .iter()
            .map(|entry| entry.row_id.clone())
            .collect(),
        structured_family_count: bundle.structured_families.families.len(),
    })
}

fn validate_header(
    label: &str,
    hyprland_version: &str,
    schema_version: u32,
    artifact_kind: &str,
    expected_kind: &str,
) -> Result<()> {
    ensure!(
        hyprland_version == HYPRLAND_VERSION,
        "{label}: expected Hyprland version {HYPRLAND_VERSION}, got {hyprland_version}"
    );
    ensure!(
        schema_version == SCHEMA_VERSION,
        "{label}: expected schema version {SCHEMA_VERSION}, got {schema_version}"
    );
    ensure!(
        artifact_kind == expected_kind,
        "{label}: expected artifact kind {expected_kind}, got {artifact_kind}"
    );
    Ok(())
}

fn validate_counts(bundle: &ExportBundle) -> Result<()> {
    ensure_eq(
        "manifest inventory rows",
        bundle.manifest.counts.inventory_rows,
        EXPECTED_INVENTORY_ROWS,
    )?;
    ensure_eq(
        "inventory rows",
        bundle.inventory.settings.len(),
        EXPECTED_INVENTORY_ROWS,
    )?;
    ensure_eq(
        "inventory counts.rows",
        bundle.inventory.counts.rows,
        EXPECTED_INVENTORY_ROWS,
    )?;
    ensure_eq(
        "official scalar coverage covered",
        bundle.inventory.counts.official_scalar_coverage.covered,
        EXPECTED_OFFICIAL_SCALAR_COVERED,
    )?;
    ensure_eq(
        "official scalar coverage total",
        bundle.inventory.counts.official_scalar_coverage.total,
        EXPECTED_OFFICIAL_SCALAR_TOTAL,
    )?;
    ensure_eq(
        "manifest official scalar coverage covered",
        bundle.manifest.counts.official_scalar_coverage.covered,
        EXPECTED_OFFICIAL_SCALAR_COVERED,
    )?;
    ensure_eq(
        "manifest official scalar coverage total",
        bundle.manifest.counts.official_scalar_coverage.total,
        EXPECTED_OFFICIAL_SCALAR_TOTAL,
    )?;
    ensure_eq(
        "read allowlist rows",
        bundle.read_allowlist.items.len(),
        EXPECTED_READ_ALLOWLIST_ROWS,
    )?;
    ensure_eq(
        "manifest readAllowlistRows",
        bundle.manifest.counts.read_allowlist_rows,
        EXPECTED_READ_ALLOWLIST_ROWS,
    )?;
    ensure_eq(
        "read allowlist counts.rows",
        bundle.read_allowlist.counts.rows,
        EXPECTED_READ_ALLOWLIST_ROWS,
    )?;
    ensure_eq(
        "non-read rows",
        bundle.non_read.items.len(),
        EXPECTED_NON_READ_ROWS,
    )?;
    ensure_eq(
        "manifest nonReadClassificationRows",
        bundle.manifest.counts.non_read_classification_rows,
        EXPECTED_NON_READ_ROWS,
    )?;
    ensure_eq(
        "non-read counts.rows",
        bundle.non_read.counts.rows,
        EXPECTED_NON_READ_ROWS,
    )?;
    ensure_eq(
        "preview/parser-needed rows",
        bundle
            .non_read
            .items
            .iter()
            .filter(|entry| entry.unresolved_category == "preview-parser-needed")
            .count(),
        EXPECTED_PREVIEW_PARSER_NEEDED_ROWS,
    )?;
    ensure_eq(
        "manifest previewParserNeededRows",
        bundle.manifest.counts.preview_parser_needed_rows,
        EXPECTED_PREVIEW_PARSER_NEEDED_ROWS,
    )?;
    ensure_eq(
        "non-read counts.previewParserNeededRows",
        bundle.non_read.counts.preview_parser_needed_rows,
        EXPECTED_PREVIEW_PARSER_NEEDED_ROWS,
    )?;
    ensure_eq(
        "report-only/high-risk rows",
        bundle
            .non_read
            .items
            .iter()
            .filter(|entry| entry.unresolved_category == "report-only-high-risk")
            .count(),
        EXPECTED_REPORT_ONLY_HIGH_RISK_ROWS,
    )?;
    ensure_eq(
        "manifest reportOnlyHighRiskRows",
        bundle.manifest.counts.report_only_high_risk_rows,
        EXPECTED_REPORT_ONLY_HIGH_RISK_ROWS,
    )?;
    ensure_eq(
        "non-read counts.reportOnlyHighRiskRows",
        bundle.non_read.counts.report_only_high_risk_rows,
        EXPECTED_REPORT_ONLY_HIGH_RISK_ROWS,
    )?;
    ensure_eq(
        "safe parsed-preview candidates",
        bundle
            .preview_candidates
            .items
            .iter()
            .filter(|entry| entry.status == "safe-parsed-preview")
            .count(),
        EXPECTED_SAFE_PARSED_PREVIEW_CANDIDATES,
    )?;
    ensure_eq(
        "manifest safeParsedPreviewCandidates",
        bundle.manifest.counts.safe_parsed_preview_candidates,
        EXPECTED_SAFE_PARSED_PREVIEW_CANDIDATES,
    )?;
    ensure_eq(
        "preview counts.safeParsedPreviewCandidates",
        bundle
            .preview_candidates
            .counts
            .safe_parsed_preview_candidates,
        EXPECTED_SAFE_PARSED_PREVIEW_CANDIDATES,
    )?;
    ensure_eq(
        "warning-preview candidates",
        bundle
            .preview_candidates
            .items
            .iter()
            .filter(|entry| entry.status == "warning-preview")
            .count(),
        EXPECTED_WARNING_PREVIEW_CANDIDATES,
    )?;
    ensure_eq(
        "manifest warningPreviewCandidates",
        bundle.manifest.counts.warning_preview_candidates,
        EXPECTED_WARNING_PREVIEW_CANDIDATES,
    )?;
    ensure_eq(
        "preview counts.warningPreviewCandidates",
        bundle.preview_candidates.counts.warning_preview_candidates,
        EXPECTED_WARNING_PREVIEW_CANDIDATES,
    )?;
    ensure_eq(
        "deferred parser rows",
        bundle
            .preview_candidates
            .items
            .iter()
            .filter(|entry| entry.status == "deferred-parser-preview")
            .count(),
        EXPECTED_DEFERRED_PARSER_ROWS,
    )?;
    ensure_eq(
        "manifest deferredParserRows",
        bundle.manifest.counts.deferred_parser_rows,
        EXPECTED_DEFERRED_PARSER_ROWS,
    )?;
    ensure_eq(
        "preview counts.deferredParserRows",
        bundle.preview_candidates.counts.deferred_parser_rows,
        EXPECTED_DEFERRED_PARSER_ROWS,
    )?;
    ensure_eq(
        "preview counts.rows",
        bundle.preview_candidates.counts.rows,
        EXPECTED_PREVIEW_PARSER_NEEDED_ROWS,
    )?;
    ensure_eq(
        "active write candidates",
        bundle.write_safety.active_candidates.len(),
        EXPECTED_ACTIVE_WRITE_CANDIDATES,
    )?;
    ensure_eq(
        "manifest activeWriteCandidates",
        bundle.manifest.counts.active_write_candidates,
        EXPECTED_ACTIVE_WRITE_CANDIDATES,
    )?;
    ensure_eq(
        "write counts.activeCandidates",
        bundle.write_safety.counts.active_candidates,
        EXPECTED_ACTIVE_WRITE_CANDIDATES,
    )?;
    ensure_eq(
        "structured family count",
        bundle.structured_families.families.len(),
        REQUIRED_STRUCTURED_FAMILIES.len(),
    )?;
    ensure_eq(
        "structured counts.families",
        bundle.structured_families.counts.families,
        REQUIRED_STRUCTURED_FAMILIES.len(),
    )?;
    ensure_eq(
        "value family count",
        bundle.value_families.items.len(),
        bundle.value_families.counts.families,
    )?;
    Ok(())
}

fn validate_required_rows(inventory_ids: &BTreeSet<String>) -> Result<()> {
    ensure!(
        !inventory_ids.contains("animations.global_speed"),
        "animations.global_speed must remain absent"
    );
    ensure!(
        !inventory_ids.contains("animations.style"),
        "animations.style must remain absent"
    );
    ensure!(
        inventory_ids.contains("animations.enabled"),
        "animations.enabled must remain present"
    );
    ensure!(
        inventory_ids.contains("animations.workspace_wraparound"),
        "animations.workspace_wraparound must remain present"
    );
    Ok(())
}

fn validate_row_links(
    inventory_ids: &BTreeSet<String>,
    read_ids: &BTreeSet<String>,
    non_read_ids: &BTreeSet<String>,
    preview_ids: &BTreeSet<String>,
    write_ids: &BTreeSet<String>,
) -> Result<()> {
    ensure_subset("read allowlist", read_ids, inventory_ids)?;
    ensure_subset("non-read classifications", non_read_ids, inventory_ids)?;
    ensure_subset("preview candidates", preview_ids, inventory_ids)?;
    ensure_subset("write safety active candidates", write_ids, inventory_ids)?;

    let overlap: Vec<_> = read_ids.intersection(non_read_ids).cloned().collect();
    ensure!(overlap.is_empty(), "read/non-read overlap: {overlap:?}");

    let union: BTreeSet<_> = read_ids.union(non_read_ids).cloned().collect();
    let missing: Vec<_> = inventory_ids.difference(&union).cloned().collect();
    ensure!(
        missing.is_empty(),
        "inventory rows without read or non-read status: {missing:?}"
    );
    Ok(())
}

fn validate_official_setting_consistency(bundle: &ExportBundle) -> Result<()> {
    let official_by_row: HashMap<_, _> = bundle
        .inventory
        .settings
        .iter()
        .map(|entry| (entry.row_id.as_str(), entry.official_setting.as_str()))
        .collect();

    for entry in &bundle.read_allowlist.items {
        ensure_official_matches(
            "read allowlist",
            &official_by_row,
            &entry.row_id,
            entry.official_setting.as_deref(),
        )?;
    }
    for entry in &bundle.non_read.items {
        ensure_official_matches(
            "non-read classifications",
            &official_by_row,
            &entry.row_id,
            entry.official_setting.as_deref(),
        )?;
    }
    for entry in &bundle.write_safety.active_candidates {
        ensure_official_matches(
            "write safety",
            &official_by_row,
            &entry.row_id,
            Some(entry.official_setting.as_str()),
        )?;
    }
    Ok(())
}

fn validate_write_safety(bundle: &ExportBundle) -> Result<()> {
    let ids: Vec<_> = bundle
        .write_safety
        .active_candidates
        .iter()
        .map(|entry| entry.row_id.as_str())
        .collect();
    ensure!(
        ids == vec![EXPECTED_ACTIVE_WRITE_CANDIDATE],
        "active write candidates must be [{EXPECTED_ACTIVE_WRITE_CANDIDATE}], got {ids:?}"
    );
    let manifest_ids: Vec<_> = bundle
        .manifest
        .counts
        .active_write_candidate_ids
        .iter()
        .map(String::as_str)
        .collect();
    ensure!(
        manifest_ids == vec![EXPECTED_ACTIVE_WRITE_CANDIDATE],
        "manifest active write candidates must be [{EXPECTED_ACTIVE_WRITE_CANDIDATE}], got {manifest_ids:?}"
    );

    for entry in &bundle.write_safety.active_candidates {
        ensure!(!entry.executable, "{} must be non-executable", entry.row_id);
        ensure!(
            !entry.command_generation_allowed,
            "{} must not allow command generation",
            entry.row_id
        );
    }
    Ok(())
}

fn validate_value_families(bundle: &ExportBundle) -> Result<()> {
    let value_family_ids = collect_row_ids(
        "value families",
        bundle
            .value_families
            .items
            .iter()
            .map(|entry| entry.value_family.as_str()),
    )?;

    ensure_eq(
        "value family required count",
        value_family_ids.len(),
        REQUIRED_VALUE_FAMILIES.len(),
    )?;
    for required in REQUIRED_VALUE_FAMILIES {
        ensure!(
            value_family_ids.contains(*required),
            "value family {required} missing"
        );
    }
    Ok(())
}

fn validate_structured_families(bundle: &ExportBundle) -> Result<()> {
    let family_ids = collect_row_ids(
        "structured families",
        bundle
            .structured_families
            .families
            .iter()
            .map(|entry| entry.family_id.as_str()),
    )?;

    for required in REQUIRED_STRUCTURED_FAMILIES {
        ensure!(
            family_ids.contains(*required),
            "structured family {required} missing"
        );
    }
    Ok(())
}

fn collect_unique_inventory_ids(bundle: &ExportBundle) -> Result<BTreeSet<String>> {
    collect_row_ids(
        "inventory",
        bundle
            .inventory
            .settings
            .iter()
            .map(|entry| entry.row_id.as_str()),
    )
}

fn collect_row_ids<'a>(
    label: &str,
    ids: impl Iterator<Item = &'a str>,
) -> Result<BTreeSet<String>> {
    let mut seen = BTreeSet::new();
    let mut duplicates = Vec::new();
    for id in ids {
        if id.is_empty() {
            bail!("{label}: blank row id");
        }
        if !seen.insert(id.to_string()) {
            duplicates.push(id.to_string());
        }
    }
    ensure!(
        duplicates.is_empty(),
        "{label}: duplicate row IDs {duplicates:?}"
    );
    Ok(seen)
}

fn ensure_subset(label: &str, child: &BTreeSet<String>, parent: &BTreeSet<String>) -> Result<()> {
    let orphan_ids: Vec<_> = child.difference(parent).cloned().collect();
    ensure!(
        orphan_ids.is_empty(),
        "{label}: orphan row IDs {orphan_ids:?}"
    );
    Ok(())
}

fn ensure_official_matches(
    label: &str,
    official_by_row: &HashMap<&str, &str>,
    row_id: &str,
    official_setting: Option<&str>,
) -> Result<()> {
    let expected = official_by_row
        .get(row_id)
        .ok_or_else(|| anyhow!("{label}: {row_id} missing from inventory"))?;
    let actual =
        official_setting.ok_or_else(|| anyhow!("{label}: {row_id} missing official setting"))?;
    ensure!(
        actual == *expected,
        "{label}: {row_id} official setting mismatch: {actual} != {expected}"
    );
    Ok(())
}

fn ensure_eq(label: &str, actual: usize, expected: usize) -> Result<()> {
    ensure!(
        actual == expected,
        "{label}: got {actual}, expected {expected}"
    );
    Ok(())
}
