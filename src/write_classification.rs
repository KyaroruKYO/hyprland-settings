use crate::export::InventoryEntry;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ScalarWriteClassification {
    pub row_id: String,
    pub official_setting: String,
    pub config_key: String,
    pub status: ScalarWriteStatus,
    pub blocker: Option<String>,
    pub value_kind: ScalarWriteValueKind,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ScalarWriteStatus {
    SafeWritable,
    BlockedHighRisk,
    BlockedParserNeeded,
    BlockedValidatorNeeded,
    BlockedStructured,
    BlockedAmbiguousKey,
    BlockedSpecialSemantics,
    BlockedManualReview,
    Unsupported,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ScalarWriteValueKind {
    Boolean,
    Number,
    Percent,
    StringLike,
    ComplexRaw,
    Unknown,
}

pub const SAFE_WRITABLE_TOGGLE_ROWS: &[(&str, &str)] = &[
    ("appearance.blur.enabled", "decoration.blur.enabled"),
    ("appearance.shadow.enabled", "decoration.shadow.enabled"),
    ("animations.enabled", "animations.enabled"),
    ("windows.snap.enabled", "general.snap.enabled"),
];

pub fn classify_inventory_entry(entry: &InventoryEntry) -> ScalarWriteClassification {
    let value_kind = value_kind_for_control(&entry.control_kind, &entry.value_family);
    let (status, blocker) = if safe_writable_official_setting(&entry.row_id).is_some() {
        (ScalarWriteStatus::SafeWritable, None)
    } else if entry.structured_family.unwrap_or(false) {
        (
            ScalarWriteStatus::BlockedStructured,
            Some("structured-family metadata is not writable through scalar path".to_string()),
        )
    } else if entry.report_only {
        (
            ScalarWriteStatus::BlockedHighRisk,
            Some(format!("report-only/high-risk row: {}", entry.risk_class)),
        )
    } else if entry.parser_status != "scalar-current-value-parser" {
        (
            ScalarWriteStatus::BlockedParserNeeded,
            Some(format!(
                "semantic parser needed before safe write: {}",
                entry.parser_status
            )),
        )
    } else if entry.risk_class != "safe" {
        (
            ScalarWriteStatus::BlockedManualReview,
            Some(format!(
                "manual review required for risk class: {}",
                entry.risk_class
            )),
        )
    } else if value_kind != ScalarWriteValueKind::Boolean {
        (
            ScalarWriteStatus::BlockedValidatorNeeded,
            Some("safe numeric/string validation metadata is not available yet".to_string()),
        )
    } else {
        (
            ScalarWriteStatus::BlockedManualReview,
            Some("not selected for the initial safe toggle write allowlist".to_string()),
        )
    };

    ScalarWriteClassification {
        row_id: entry.row_id.clone(),
        official_setting: entry.official_setting.clone(),
        config_key: config_key_from_official_setting(&entry.official_setting),
        status,
        blocker,
        value_kind,
    }
}

pub fn is_safe_writable_setting(row_id: &str) -> bool {
    safe_writable_official_setting(row_id).is_some()
}

pub fn safe_writable_official_setting(row_id: &str) -> Option<&'static str> {
    SAFE_WRITABLE_TOGGLE_ROWS
        .iter()
        .find(|(candidate, _)| *candidate == row_id)
        .map(|(_, official_setting)| *official_setting)
}

pub fn config_key_from_official_setting(setting: &str) -> String {
    setting.replace('.', ":")
}

pub fn value_kind_for_control(control_kind: &str, value_family: &str) -> ScalarWriteValueKind {
    match (control_kind, value_family) {
        ("toggle", "none") => ScalarWriteValueKind::Boolean,
        ("slider" | "number-input", "none") => ScalarWriteValueKind::Number,
        ("percent-slider", "none") => ScalarWriteValueKind::Percent,
        ("dropdown", "none") => ScalarWriteValueKind::StringLike,
        (_, "none") => ScalarWriteValueKind::Unknown,
        _ => ScalarWriteValueKind::ComplexRaw,
    }
}
