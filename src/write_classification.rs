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
    Color,
    Vector2,
    StringLike,
    ComplexRaw,
    Unknown,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct SafeWritableRow {
    pub row_id: &'static str,
    pub official_setting: &'static str,
    pub value_kind: ScalarWriteValueKind,
}

pub const SAFE_WRITABLE_TOGGLE_ROWS: &[(&str, &str)] = &[
    ("appearance.blur.enabled", "decoration.blur.enabled"),
    ("appearance.shadow.enabled", "decoration.shadow.enabled"),
    ("animations.enabled", "animations.enabled"),
    ("windows.snap.enabled", "general.snap.enabled"),
];

pub const SAFE_WRITABLE_ROWS: &[SafeWritableRow] = &[
    SafeWritableRow {
        row_id: "appearance.blur.enabled",
        official_setting: "decoration.blur.enabled",
        value_kind: ScalarWriteValueKind::Boolean,
    },
    SafeWritableRow {
        row_id: "appearance.blur.size",
        official_setting: "decoration.blur.size",
        value_kind: ScalarWriteValueKind::Number,
    },
    SafeWritableRow {
        row_id: "appearance.blur.brightness",
        official_setting: "decoration.blur.brightness",
        value_kind: ScalarWriteValueKind::Percent,
    },
    SafeWritableRow {
        row_id: "appearance.blur.contrast",
        official_setting: "decoration.blur.contrast",
        value_kind: ScalarWriteValueKind::Percent,
    },
    SafeWritableRow {
        row_id: "appearance.shadow.enabled",
        official_setting: "decoration.shadow.enabled",
        value_kind: ScalarWriteValueKind::Boolean,
    },
    SafeWritableRow {
        row_id: "appearance.shadow.range",
        official_setting: "decoration.shadow.range",
        value_kind: ScalarWriteValueKind::Number,
    },
    SafeWritableRow {
        row_id: "appearance.shadow.render_power",
        official_setting: "decoration.shadow.render_power",
        value_kind: ScalarWriteValueKind::Number,
    },
    SafeWritableRow {
        row_id: "decoration.shadow.color",
        official_setting: "decoration.shadow.color",
        value_kind: ScalarWriteValueKind::Color,
    },
    SafeWritableRow {
        row_id: "decoration.shadow.color_inactive",
        official_setting: "decoration.shadow.color_inactive",
        value_kind: ScalarWriteValueKind::Color,
    },
    SafeWritableRow {
        row_id: "decoration.shadow.offset",
        official_setting: "decoration.shadow.offset",
        value_kind: ScalarWriteValueKind::Vector2,
    },
    SafeWritableRow {
        row_id: "appearance.gaps_in",
        official_setting: "general.gaps_in",
        value_kind: ScalarWriteValueKind::Number,
    },
    SafeWritableRow {
        row_id: "appearance.gaps_out",
        official_setting: "general.gaps_out",
        value_kind: ScalarWriteValueKind::Number,
    },
    SafeWritableRow {
        row_id: "appearance.border_size",
        official_setting: "general.border_size",
        value_kind: ScalarWriteValueKind::Number,
    },
    SafeWritableRow {
        row_id: "appearance.rounding",
        official_setting: "decoration.rounding",
        value_kind: ScalarWriteValueKind::Number,
    },
    SafeWritableRow {
        row_id: "appearance.active_opacity",
        official_setting: "decoration.active_opacity",
        value_kind: ScalarWriteValueKind::Percent,
    },
    SafeWritableRow {
        row_id: "appearance.inactive_opacity",
        official_setting: "decoration.inactive_opacity",
        value_kind: ScalarWriteValueKind::Percent,
    },
    SafeWritableRow {
        row_id: "animations.enabled",
        official_setting: "animations.enabled",
        value_kind: ScalarWriteValueKind::Boolean,
    },
    SafeWritableRow {
        row_id: "windows.snap.enabled",
        official_setting: "general.snap.enabled",
        value_kind: ScalarWriteValueKind::Boolean,
    },
    SafeWritableRow {
        row_id: "windows.snap.window_gap",
        official_setting: "general.snap.window_gap",
        value_kind: ScalarWriteValueKind::Number,
    },
    SafeWritableRow {
        row_id: "windows.snap.monitor_gap",
        official_setting: "general.snap.monitor_gap",
        value_kind: ScalarWriteValueKind::Number,
    },
    SafeWritableRow {
        row_id: "input.pointer_sensitivity",
        official_setting: "input.sensitivity",
        value_kind: ScalarWriteValueKind::Percent,
    },
    SafeWritableRow {
        row_id: "input.tablet.region_position",
        official_setting: "input.tablet.region_position",
        value_kind: ScalarWriteValueKind::Vector2,
    },
    SafeWritableRow {
        row_id: "input.tablet.region_size",
        official_setting: "input.tablet.region_size",
        value_kind: ScalarWriteValueKind::Vector2,
    },
    SafeWritableRow {
        row_id: "input.tablet.active_area_size",
        official_setting: "input.tablet.active_area_size",
        value_kind: ScalarWriteValueKind::Vector2,
    },
    SafeWritableRow {
        row_id: "input.tablet.active_area_position",
        official_setting: "input.tablet.active_area_position",
        value_kind: ScalarWriteValueKind::Vector2,
    },
    SafeWritableRow {
        row_id: "decoration.glow.color",
        official_setting: "decoration.glow.color",
        value_kind: ScalarWriteValueKind::Color,
    },
    SafeWritableRow {
        row_id: "decoration.glow.color_inactive",
        official_setting: "decoration.glow.color_inactive",
        value_kind: ScalarWriteValueKind::Color,
    },
    SafeWritableRow {
        row_id: "group.groupbar.text_color",
        official_setting: "group.groupbar.text_color",
        value_kind: ScalarWriteValueKind::Color,
    },
    SafeWritableRow {
        row_id: "group.groupbar.text_color_inactive",
        official_setting: "group.groupbar.text_color_inactive",
        value_kind: ScalarWriteValueKind::Color,
    },
    SafeWritableRow {
        row_id: "group.groupbar.text_color_locked_active",
        official_setting: "group.groupbar.text_color_locked_active",
        value_kind: ScalarWriteValueKind::Color,
    },
    SafeWritableRow {
        row_id: "group.groupbar.text_color_locked_inactive",
        official_setting: "group.groupbar.text_color_locked_inactive",
        value_kind: ScalarWriteValueKind::Color,
    },
    SafeWritableRow {
        row_id: "misc.col.splash",
        official_setting: "misc.col.splash",
        value_kind: ScalarWriteValueKind::Color,
    },
    SafeWritableRow {
        row_id: "misc.background_color",
        official_setting: "misc.background_color",
        value_kind: ScalarWriteValueKind::Color,
    },
    SafeWritableRow {
        row_id: "layout.single_window_aspect_ratio",
        official_setting: "layout.single_window_aspect_ratio",
        value_kind: ScalarWriteValueKind::Vector2,
    },
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
    safe_writable_row(row_id).map(|row| row.official_setting)
}

pub fn safe_writable_value_kind(row_id: &str) -> Option<ScalarWriteValueKind> {
    safe_writable_row(row_id).map(|row| row.value_kind)
}

pub fn safe_writable_row(row_id: &str) -> Option<&'static SafeWritableRow> {
    SAFE_WRITABLE_ROWS
        .iter()
        .find(|candidate| candidate.row_id == row_id)
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
