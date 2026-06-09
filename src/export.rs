use std::path::{Path, PathBuf};

use anyhow::{Context, Result};
use serde::Deserialize;

pub const HYPRLAND_VERSION: &str = "0.55.2";
pub const SCHEMA_VERSION: u32 = 1;

const MANIFEST_FILE: &str = "hyprland-settings-export-manifest.v0.55.2.json";
const INVENTORY_FILE: &str = "hyprland-settings-inventory.v0.55.2.json";
const READ_ALLOWLIST_FILE: &str = "hyprland-settings-read-allowlist.v0.55.2.json";
const NON_READ_FILE: &str = "hyprland-settings-non-read-classifications.v0.55.2.json";
const VALUE_FAMILIES_FILE: &str = "hyprland-settings-value-families.v0.55.2.json";
const PREVIEW_CANDIDATES_FILE: &str = "hyprland-settings-preview-candidates.v0.55.2.json";
const WRITE_SAFETY_FILE: &str = "hyprland-settings-write-safety.v0.55.2.json";
const STRUCTURED_FAMILIES_FILE: &str = "hyprland-structured-families-design-index.v0.55.2.json";
const SOURCE_PROVENANCE_FILE: &str = "hyprland-settings-source-provenance-index.v0.55.2.json";

#[derive(Debug)]
pub struct ExportBundle {
    pub export_dir: PathBuf,
    pub manifest: Manifest,
    pub inventory: InventoryArtifact,
    pub read_allowlist: ReadAllowlistArtifact,
    pub non_read: NonReadArtifact,
    pub value_families: ValueFamiliesArtifact,
    pub preview_candidates: PreviewCandidatesArtifact,
    pub write_safety: WriteSafetyArtifact,
    pub structured_families: StructuredFamiliesArtifact,
    pub source_provenance: SourceProvenanceArtifact,
}

impl ExportBundle {
    pub fn load(export_dir: &Path) -> Result<Self> {
        Ok(Self {
            export_dir: export_dir.to_path_buf(),
            manifest: load_json(export_dir, MANIFEST_FILE)?,
            inventory: load_json(export_dir, INVENTORY_FILE)?,
            read_allowlist: load_json(export_dir, READ_ALLOWLIST_FILE)?,
            non_read: load_json(export_dir, NON_READ_FILE)?,
            value_families: load_json(export_dir, VALUE_FAMILIES_FILE)?,
            preview_candidates: load_json(export_dir, PREVIEW_CANDIDATES_FILE)?,
            write_safety: load_json(export_dir, WRITE_SAFETY_FILE)?,
            structured_families: load_json(export_dir, STRUCTURED_FAMILIES_FILE)?,
            source_provenance: load_json(export_dir, SOURCE_PROVENANCE_FILE)?,
        })
    }
}

fn load_json<T: for<'de> Deserialize<'de>>(export_dir: &Path, file_name: &str) -> Result<T> {
    let path = export_dir.join(file_name);
    let text = std::fs::read_to_string(&path)
        .with_context(|| format!("failed to read export artifact {}", path.display()))?;
    serde_json::from_str(&text)
        .with_context(|| format!("failed to parse export artifact {}", path.display()))
}

#[derive(Debug, Deserialize)]
pub struct Manifest {
    #[serde(rename = "hyprlandVersion")]
    pub hyprland_version: String,
    #[serde(rename = "schemaVersion")]
    pub schema_version: u32,
    #[serde(rename = "artifactKind")]
    pub artifact_kind: String,
    pub counts: ManifestCounts,
}

#[derive(Debug, Deserialize)]
pub struct ManifestCounts {
    #[serde(rename = "inventoryRows")]
    pub inventory_rows: usize,
    #[serde(rename = "officialScalarCoverage")]
    pub official_scalar_coverage: OfficialScalarCoverage,
    #[serde(rename = "readAllowlistRows")]
    pub read_allowlist_rows: usize,
    #[serde(rename = "nonReadClassificationRows")]
    pub non_read_classification_rows: usize,
    #[serde(rename = "previewParserNeededRows")]
    pub preview_parser_needed_rows: usize,
    #[serde(rename = "reportOnlyHighRiskRows")]
    pub report_only_high_risk_rows: usize,
    #[serde(rename = "safeParsedPreviewCandidates")]
    pub safe_parsed_preview_candidates: usize,
    #[serde(rename = "warningPreviewCandidates")]
    pub warning_preview_candidates: usize,
    #[serde(rename = "deferredParserRows")]
    pub deferred_parser_rows: usize,
    #[serde(rename = "activeWriteCandidates")]
    pub active_write_candidates: usize,
    #[serde(rename = "activeWriteCandidateIds")]
    pub active_write_candidate_ids: Vec<String>,
}

#[derive(Debug, Deserialize)]
pub struct OfficialScalarCoverage {
    pub covered: usize,
    pub total: usize,
}

#[derive(Debug, Deserialize)]
pub struct InventoryArtifact {
    #[serde(rename = "hyprlandVersion")]
    pub hyprland_version: String,
    #[serde(rename = "schemaVersion")]
    pub schema_version: u32,
    #[serde(rename = "artifactKind")]
    pub artifact_kind: String,
    pub counts: InventoryCounts,
    pub tabs: Vec<TabEntry>,
    pub settings: Vec<InventoryEntry>,
}

#[derive(Debug, Deserialize)]
pub struct InventoryCounts {
    pub rows: usize,
    #[serde(rename = "officialScalarCoverage")]
    pub official_scalar_coverage: OfficialScalarCoverage,
}

#[derive(Debug, Deserialize)]
pub struct InventoryEntry {
    #[serde(rename = "rowId")]
    pub row_id: String,
    #[serde(rename = "officialSetting")]
    pub official_setting: String,
    #[serde(rename = "tabId")]
    pub tab_id: String,
    #[serde(rename = "tabLabel")]
    pub tab_label: String,
    pub subsection: String,
    #[serde(rename = "rowOrder")]
    pub row_order: usize,
    pub label: String,
    pub description: String,
    #[serde(rename = "controlKind")]
    pub control_kind: String,
    #[serde(rename = "valueFamily")]
    pub value_family: String,
    #[serde(rename = "parserStatus")]
    pub parser_status: String,
    #[serde(rename = "structuredFamily")]
    pub structured_family: Option<bool>,
    #[serde(rename = "defaultConfigPresence")]
    pub default_config_presence: String,
    #[serde(rename = "readSupport")]
    pub read_support: String,
    #[serde(rename = "writeSupport")]
    pub write_support: String,
    #[serde(rename = "riskClass")]
    pub risk_class: String,
    #[serde(rename = "previewStatus")]
    pub preview_status: String,
    #[serde(rename = "reportOnly")]
    pub report_only: bool,
}

#[derive(Debug, Deserialize)]
pub struct TabEntry {
    #[serde(rename = "tabId")]
    pub tab_id: String,
    #[serde(rename = "tabLabel")]
    pub tab_label: String,
    #[serde(rename = "tabOrder")]
    pub tab_order: usize,
    #[serde(rename = "rowCount")]
    pub row_count: usize,
}

#[derive(Debug, Deserialize)]
pub struct ReadAllowlistArtifact {
    #[serde(rename = "hyprlandVersion")]
    pub hyprland_version: String,
    #[serde(rename = "schemaVersion")]
    pub schema_version: u32,
    #[serde(rename = "artifactKind")]
    pub artifact_kind: String,
    pub counts: RowsCount,
    pub items: Vec<RowItem>,
}

#[derive(Debug, Deserialize)]
pub struct RowsCount {
    pub rows: usize,
}

#[derive(Debug, Deserialize)]
pub struct RowItem {
    #[serde(rename = "rowId")]
    pub row_id: String,
    #[serde(rename = "officialSetting")]
    pub official_setting: Option<String>,
}

#[derive(Debug, Deserialize)]
pub struct NonReadArtifact {
    #[serde(rename = "hyprlandVersion")]
    pub hyprland_version: String,
    #[serde(rename = "schemaVersion")]
    pub schema_version: u32,
    #[serde(rename = "artifactKind")]
    pub artifact_kind: String,
    pub counts: NonReadCounts,
    pub items: Vec<NonReadEntry>,
}

#[derive(Debug, Deserialize)]
pub struct NonReadCounts {
    pub rows: usize,
    #[serde(rename = "previewParserNeededRows")]
    pub preview_parser_needed_rows: usize,
    #[serde(rename = "reportOnlyHighRiskRows")]
    pub report_only_high_risk_rows: usize,
}

#[derive(Debug, Deserialize)]
pub struct NonReadEntry {
    #[serde(rename = "rowId")]
    pub row_id: String,
    #[serde(rename = "officialSetting")]
    pub official_setting: Option<String>,
    #[serde(rename = "unresolvedCategory")]
    pub unresolved_category: String,
}

#[derive(Debug, Deserialize)]
pub struct ValueFamiliesArtifact {
    #[serde(rename = "hyprlandVersion")]
    pub hyprland_version: String,
    #[serde(rename = "schemaVersion")]
    pub schema_version: u32,
    #[serde(rename = "artifactKind")]
    pub artifact_kind: String,
    pub counts: FamiliesCount,
    pub items: Vec<ValueFamilyEntry>,
}

#[derive(Debug, Deserialize)]
pub struct FamiliesCount {
    pub families: usize,
}

#[derive(Debug, Deserialize)]
pub struct ValueFamilyEntry {
    #[serde(rename = "valueFamily")]
    pub value_family: String,
}

#[derive(Debug, Deserialize)]
pub struct PreviewCandidatesArtifact {
    #[serde(rename = "hyprlandVersion")]
    pub hyprland_version: String,
    #[serde(rename = "schemaVersion")]
    pub schema_version: u32,
    #[serde(rename = "artifactKind")]
    pub artifact_kind: String,
    pub counts: PreviewCounts,
    pub items: Vec<PreviewCandidateEntry>,
}

#[derive(Debug, Deserialize)]
pub struct PreviewCounts {
    pub rows: usize,
    #[serde(rename = "safeParsedPreviewCandidates")]
    pub safe_parsed_preview_candidates: usize,
    #[serde(rename = "warningPreviewCandidates")]
    pub warning_preview_candidates: usize,
    #[serde(rename = "deferredParserRows")]
    pub deferred_parser_rows: usize,
}

#[derive(Debug, Deserialize)]
pub struct PreviewCandidateEntry {
    #[serde(rename = "rowId")]
    pub row_id: String,
    pub status: String,
}

#[derive(Debug, Deserialize)]
pub struct WriteSafetyArtifact {
    #[serde(rename = "hyprlandVersion")]
    pub hyprland_version: String,
    #[serde(rename = "schemaVersion")]
    pub schema_version: u32,
    #[serde(rename = "artifactKind")]
    pub artifact_kind: String,
    pub counts: WriteSafetyCounts,
    #[serde(rename = "activeCandidates")]
    pub active_candidates: Vec<WriteCandidateEntry>,
}

#[derive(Debug, Deserialize)]
pub struct WriteSafetyCounts {
    #[serde(rename = "activeCandidates")]
    pub active_candidates: usize,
}

#[derive(Debug, Deserialize)]
pub struct WriteCandidateEntry {
    #[serde(rename = "rowId")]
    pub row_id: String,
    #[serde(rename = "officialSetting")]
    pub official_setting: String,
    pub executable: bool,
    #[serde(rename = "commandGenerationAllowed")]
    pub command_generation_allowed: bool,
    #[serde(rename = "targetMode")]
    pub target_mode: String,
}

#[derive(Debug, Deserialize)]
pub struct StructuredFamiliesArtifact {
    #[serde(rename = "hyprlandVersion")]
    pub hyprland_version: String,
    #[serde(rename = "schemaVersion")]
    pub schema_version: u32,
    #[serde(rename = "artifactKind")]
    pub artifact_kind: String,
    pub counts: FamiliesCount,
    pub families: Vec<StructuredFamilyEntry>,
}

#[derive(Debug, Deserialize)]
pub struct StructuredFamilyEntry {
    #[serde(rename = "familyId")]
    pub family_id: String,
}

#[derive(Debug, Deserialize)]
pub struct SourceProvenanceArtifact {
    #[serde(rename = "hyprlandVersion")]
    pub hyprland_version: String,
    #[serde(rename = "schemaVersion")]
    pub schema_version: u32,
    #[serde(rename = "artifactKind")]
    pub artifact_kind: String,
}
