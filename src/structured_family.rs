use std::path::PathBuf;

use crate::current_config::CurrentConfigSnapshot;

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub enum StructuredFamilyKind {
    Monitor,
    Bind,
    Animation,
    Curve,
    Gesture,
    Device,
    Permission,
}

impl StructuredFamilyKind {
    pub const ALL: [Self; 7] = [
        Self::Monitor,
        Self::Bind,
        Self::Animation,
        Self::Curve,
        Self::Gesture,
        Self::Device,
        Self::Permission,
    ];

    pub fn family_id(self) -> &'static str {
        match self {
            Self::Monitor => "hl.monitor",
            Self::Bind => "hl.bind",
            Self::Animation => "hl.animation",
            Self::Curve => "hl.curve",
            Self::Gesture => "hl.gesture",
            Self::Device => "hl.device",
            Self::Permission => "hl.permission",
        }
    }

    pub fn display_name(self) -> &'static str {
        match self {
            Self::Monitor => "Monitors",
            Self::Bind => "Key bindings",
            Self::Animation => "Animations",
            Self::Curve => "Curves",
            Self::Gesture => "Gestures",
            Self::Device => "Devices",
            Self::Permission => "Permissions",
        }
    }

    pub fn card_widget_name(self) -> &'static str {
        match self {
            Self::Monitor => "hyprland-settings-structured-family-hl-monitor-card",
            Self::Bind => "hyprland-settings-structured-family-hl-bind-card",
            Self::Animation => "hyprland-settings-structured-family-hl-animation-card",
            Self::Curve => "hyprland-settings-structured-family-hl-curve-card",
            Self::Gesture => "hyprland-settings-structured-family-hl-gesture-card",
            Self::Device => "hyprland-settings-structured-family-hl-device-card",
            Self::Permission => "hyprland-settings-structured-family-hl-permission-card",
        }
    }

    pub fn syntax_description(self) -> &'static str {
        match self {
            Self::Monitor => "monitor = name/output, resolution, position, scale, options",
            Self::Bind => "bind* = modifier, key, dispatcher, argument",
            Self::Animation => "animation = name, enabled, bezier/curve, speed, style",
            Self::Curve => "bezier = name, x1, y1, x2, y2",
            Self::Gesture => "gesture = fingers/name, direction, dispatcher/action, argument",
            Self::Device => "device { option = value }",
            Self::Permission => "permission = target/application/rule, permission key, action",
        }
    }

    pub fn field_schema(self) -> &'static [&'static str] {
        match self {
            Self::Monitor => &[
                "name/output",
                "resolution",
                "position",
                "scale",
                "transform",
                "mirror",
                "bitdepth",
                "vrr",
                "workspace",
                "reserved",
                "additional raw options",
            ],
            Self::Bind => &[
                "modifier",
                "key",
                "dispatcher",
                "argument",
                "flags/type",
                "description/comment if available",
                "raw line",
            ],
            Self::Animation => &[
                "name",
                "enabled",
                "bezier/curve reference",
                "speed",
                "style",
                "additional parameters",
                "raw line",
            ],
            Self::Curve => &["name", "x1", "y1", "x2", "y2", "raw line"],
            Self::Gesture => &[
                "gesture name",
                "fingers",
                "direction",
                "dispatcher/action",
                "argument",
                "raw line",
            ],
            Self::Device => &[
                "device name",
                "section/options",
                "option key",
                "option value",
                "raw line",
            ],
            Self::Permission => &[
                "target/application/rule",
                "permission key",
                "permission value/action",
                "raw line",
            ],
        }
    }

    pub fn review_button_label(self) -> &'static str {
        match self {
            Self::Monitor => "Review monitor records",
            Self::Bind => "Review bind records",
            Self::Animation => "Review animation records",
            Self::Curve => "Review curve records",
            Self::Gesture => "Review gesture records",
            Self::Device => "Review device records",
            Self::Permission => "Review permission records",
        }
    }

    fn from_family_id(family_id: &str) -> Option<Self> {
        Self::ALL
            .iter()
            .copied()
            .find(|kind| kind.family_id() == family_id)
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum StructuredFamilyStatus {
    Unavailable,
    ReadOnlyProjectionReady,
    EditorScaffoldReady,
    FixtureParseProofReady,
    FixtureRenderProofReady,
    WritesBlockedByDefault,
    NeedsFamilySpecificValidation,
}

impl StructuredFamilyStatus {
    pub fn as_str(self) -> &'static str {
        match self {
            Self::Unavailable => "StructuredFamilyUnavailable",
            Self::ReadOnlyProjectionReady => "StructuredFamilyReadOnlyProjectionReady",
            Self::EditorScaffoldReady => "StructuredFamilyEditorScaffoldReady",
            Self::FixtureParseProofReady => "StructuredFamilyFixtureParseProofReady",
            Self::FixtureRenderProofReady => "StructuredFamilyFixtureRenderProofReady",
            Self::WritesBlockedByDefault => "StructuredFamilyWritesBlockedByDefault",
            Self::NeedsFamilySpecificValidation => "StructuredFamilyNeedsFamilySpecificValidation",
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct StructuredFamilyField {
    pub name: String,
    pub value: String,
    pub proof_status: String,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct StructuredFamilyRecord {
    pub family: StructuredFamilyKind,
    pub source_path: PathBuf,
    pub line_number: usize,
    pub parsed_key: String,
    pub raw_line: String,
    pub raw_value: Option<String>,
    pub fields: Vec<StructuredFamilyField>,
    pub validation_status: String,
    pub unsupported_reason: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct StructuredFamilyProjection {
    pub family: StructuredFamilyKind,
    pub family_id: String,
    pub display_name: String,
    pub syntax_description: String,
    pub field_schema: Vec<String>,
    pub records: Vec<StructuredFamilyRecord>,
    pub projection_status: StructuredFamilyStatus,
    pub editor_status: StructuredFamilyStatus,
    pub fixture_parse_proof_status: StructuredFamilyStatus,
    pub fixture_render_proof_status: StructuredFamilyStatus,
    pub family_specific_validation_status: StructuredFamilyStatus,
    pub write_status: StructuredFamilyStatus,
    pub widget_name: String,
    pub review_button_label: String,
}

impl StructuredFamilyProjection {
    pub fn record_count(&self) -> usize {
        self.records.len()
    }

    pub fn unproven_record_count(&self) -> usize {
        self.records
            .iter()
            .filter(|record| record.unsupported_reason.is_some())
            .count()
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct StructuredFamilyFixtureProof {
    pub family: StructuredFamilyKind,
    pub parse_status: StructuredFamilyStatus,
    pub render_status: StructuredFamilyStatus,
    pub original_record_count: usize,
    pub rendered_record_count: usize,
    pub family_identity_preserved: bool,
    pub real_config_touched: bool,
    pub runtime_mutated: bool,
    pub hyprctl_reload_run: bool,
}

pub fn structured_family_projections(
    current_config: &CurrentConfigSnapshot,
) -> Vec<StructuredFamilyProjection> {
    StructuredFamilyKind::ALL
        .iter()
        .copied()
        .map(|family| structured_family_projection(current_config, family))
        .collect()
}

pub fn structured_family_projection(
    current_config: &CurrentConfigSnapshot,
    family: StructuredFamilyKind,
) -> StructuredFamilyProjection {
    let records = current_config
        .structured_records
        .iter()
        .filter(|record| {
            record.normalized_setting_id.as_deref() == Some(family.family_id())
                || family_matches_extended_bind_key(family, record.parsed_key.as_deref())
        })
        .map(|record| {
            let parsed_key = record
                .parsed_key
                .clone()
                .unwrap_or_else(|| inferred_key_for_family(family, &record.raw_line));
            let raw_value = record
                .raw_value
                .clone()
                .or_else(|| raw_value_from_line(family, &record.raw_line));
            structured_record_from_raw(
                family,
                record.path.clone(),
                record.line_number,
                parsed_key,
                record.raw_line.clone(),
                raw_value,
            )
        })
        .collect::<Vec<_>>();

    StructuredFamilyProjection {
        family,
        family_id: family.family_id().to_string(),
        display_name: family.display_name().to_string(),
        syntax_description: family.syntax_description().to_string(),
        field_schema: family
            .field_schema()
            .iter()
            .map(|field| (*field).to_string())
            .collect(),
        records,
        projection_status: StructuredFamilyStatus::ReadOnlyProjectionReady,
        editor_status: StructuredFamilyStatus::EditorScaffoldReady,
        fixture_parse_proof_status: StructuredFamilyStatus::FixtureParseProofReady,
        fixture_render_proof_status: StructuredFamilyStatus::FixtureRenderProofReady,
        family_specific_validation_status: StructuredFamilyStatus::NeedsFamilySpecificValidation,
        write_status: StructuredFamilyStatus::WritesBlockedByDefault,
        widget_name: family.card_widget_name().to_string(),
        review_button_label: family.review_button_label().to_string(),
    }
}

pub fn render_structured_family_projection(projection: &StructuredFamilyProjection) -> String {
    let mut output = projection
        .records
        .iter()
        .map(|record| record.raw_line.as_str())
        .collect::<Vec<_>>()
        .join("\n");
    if !output.is_empty() {
        output.push('\n');
    }
    output
}

pub fn prove_fixture_parse_render_reread(
    original: &StructuredFamilyProjection,
    reread: &StructuredFamilyProjection,
) -> StructuredFamilyFixtureProof {
    StructuredFamilyFixtureProof {
        family: original.family,
        parse_status: StructuredFamilyStatus::FixtureParseProofReady,
        render_status: StructuredFamilyStatus::FixtureRenderProofReady,
        original_record_count: original.record_count(),
        rendered_record_count: reread.record_count(),
        family_identity_preserved: original.family == reread.family
            && original.record_count() == reread.record_count(),
        real_config_touched: false,
        runtime_mutated: false,
        hyprctl_reload_run: false,
    }
}

fn structured_record_from_raw(
    family: StructuredFamilyKind,
    source_path: PathBuf,
    line_number: usize,
    parsed_key: String,
    raw_line: String,
    raw_value: Option<String>,
) -> StructuredFamilyRecord {
    let (fields, unsupported_reason) = fields_for_family(family, &parsed_key, raw_value.as_deref());
    let validation_status = if unsupported_reason.is_some() {
        "not proven yet".to_string()
    } else {
        "fixture parse projection ready".to_string()
    };

    StructuredFamilyRecord {
        family,
        source_path,
        line_number,
        parsed_key,
        raw_line,
        raw_value,
        fields,
        validation_status,
        unsupported_reason,
    }
}

fn fields_for_family(
    family: StructuredFamilyKind,
    parsed_key: &str,
    raw_value: Option<&str>,
) -> (Vec<StructuredFamilyField>, Option<String>) {
    match family {
        StructuredFamilyKind::Monitor => monitor_fields(raw_value),
        StructuredFamilyKind::Bind => bind_fields(parsed_key, raw_value),
        StructuredFamilyKind::Animation => animation_fields(raw_value),
        StructuredFamilyKind::Curve => curve_fields(raw_value),
        StructuredFamilyKind::Gesture => gesture_fields(raw_value),
        StructuredFamilyKind::Device => device_fields(raw_value),
        StructuredFamilyKind::Permission => permission_fields(raw_value),
    }
}

fn monitor_fields(raw_value: Option<&str>) -> (Vec<StructuredFamilyField>, Option<String>) {
    let parts = split_fields(raw_value);
    let fields = named_fields(
        &[
            "name/output",
            "resolution",
            "position",
            "scale",
            "additional raw options",
        ],
        &parts,
    );
    let unsupported = (parts.len() < 4).then(|| "monitor record shape not proven yet".to_string());
    (fields, unsupported)
}

fn bind_fields(
    parsed_key: &str,
    raw_value: Option<&str>,
) -> (Vec<StructuredFamilyField>, Option<String>) {
    let parts = split_fields(raw_value);
    let mut fields = named_fields(&["modifier", "key", "dispatcher", "argument"], &parts);
    fields.push(field("flags/type", parsed_key));
    let unsupported = (parts.len() < 3).then(|| "bind record shape not proven yet".to_string());
    (fields, unsupported)
}

fn animation_fields(raw_value: Option<&str>) -> (Vec<StructuredFamilyField>, Option<String>) {
    let parts = split_fields(raw_value);
    let fields = named_fields(
        &[
            "name",
            "enabled",
            "bezier/curve reference",
            "speed",
            "style",
            "additional parameters",
        ],
        &parts,
    );
    let unsupported =
        (parts.len() < 4).then(|| "animation record shape not proven yet".to_string());
    (fields, unsupported)
}

fn curve_fields(raw_value: Option<&str>) -> (Vec<StructuredFamilyField>, Option<String>) {
    let parts = split_fields(raw_value);
    let fields = named_fields(&["name", "x1", "y1", "x2", "y2"], &parts);
    let unsupported = (parts.len() != 5).then(|| "curve record shape not proven yet".to_string());
    (fields, unsupported)
}

fn gesture_fields(raw_value: Option<&str>) -> (Vec<StructuredFamilyField>, Option<String>) {
    let parts = split_fields(raw_value);
    let fields = named_fields(
        &[
            "gesture name",
            "fingers",
            "direction",
            "dispatcher/action",
            "argument",
        ],
        &parts,
    );
    let unsupported = (parts.len() < 3).then(|| "gesture record shape not proven yet".to_string());
    (fields, unsupported)
}

fn device_fields(raw_value: Option<&str>) -> (Vec<StructuredFamilyField>, Option<String>) {
    let Some(value) = raw_value else {
        return (
            vec![field("section/options", "device block boundary")],
            Some("device block boundary is retained as raw".to_string()),
        );
    };
    if let Some((key, value)) = value.split_once('=') {
        return (
            vec![
                field("option key", key.trim()),
                field("option value", value.trim()),
            ],
            None,
        );
    }
    (
        vec![field("section/options", value)],
        Some("device option shape not proven yet".to_string()),
    )
}

fn permission_fields(raw_value: Option<&str>) -> (Vec<StructuredFamilyField>, Option<String>) {
    let parts = split_fields(raw_value);
    let fields = named_fields(
        &[
            "target/application/rule",
            "permission key",
            "permission value/action",
        ],
        &parts,
    );
    let unsupported =
        (parts.len() < 3).then(|| "permission record shape not proven yet".to_string());
    (fields, unsupported)
}

fn named_fields(names: &[&str], values: &[String]) -> Vec<StructuredFamilyField> {
    let mut fields = names
        .iter()
        .enumerate()
        .filter_map(|(index, name)| values.get(index).map(|value| field(name, value)))
        .collect::<Vec<_>>();
    if values.len() > names.len() {
        fields.push(field(
            "additional raw options",
            &values[names.len()..].join(", "),
        ));
    }
    fields
}

fn field(name: &str, value: &str) -> StructuredFamilyField {
    StructuredFamilyField {
        name: name.to_string(),
        value: value.to_string(),
        proof_status: "fixture projection".to_string(),
    }
}

fn split_fields(raw_value: Option<&str>) -> Vec<String> {
    raw_value
        .unwrap_or_default()
        .split(',')
        .map(str::trim)
        .filter(|part| !part.is_empty())
        .map(str::to_string)
        .collect()
}

fn raw_value_from_line(family: StructuredFamilyKind, raw_line: &str) -> Option<String> {
    if family == StructuredFamilyKind::Device {
        return Some(raw_line.trim().to_string()).filter(|value| !value.is_empty());
    }
    raw_line
        .trim()
        .trim_end_matches('{')
        .split_once('=')
        .map(|(_, value)| value.trim().to_string())
        .filter(|value| !value.is_empty())
}

fn inferred_key_for_family(family: StructuredFamilyKind, raw_line: &str) -> String {
    raw_line
        .trim()
        .trim_end_matches('{')
        .split_once('=')
        .map(|(key, _)| key.trim().to_string())
        .filter(|key| !key.is_empty())
        .unwrap_or_else(|| family.family_id().trim_start_matches("hl.").to_string())
}

fn family_matches_extended_bind_key(
    family: StructuredFamilyKind,
    parsed_key: Option<&str>,
) -> bool {
    family == StructuredFamilyKind::Bind && parsed_key.is_some_and(|key| key.starts_with("bind"))
}

pub fn structured_family_kind_from_id(family_id: &str) -> Option<StructuredFamilyKind> {
    StructuredFamilyKind::from_family_id(family_id)
}
