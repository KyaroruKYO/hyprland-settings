use std::fs;
use std::path::{Path, PathBuf};

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

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum StructuredFamilyValidationStatus {
    Ready,
    Passed,
    Warning,
    Failed,
    NotProvenYet,
}

impl StructuredFamilyValidationStatus {
    pub fn as_str(self) -> &'static str {
        match self {
            Self::Ready => "StructuredFamilyValidationReady",
            Self::Passed => "StructuredFamilyValidationPassed",
            Self::Warning => "StructuredFamilyValidationWarning",
            Self::Failed => "StructuredFamilyValidationFailed",
            Self::NotProvenYet => "StructuredFamilyValidationNotProvenYet",
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum StructuredFamilyTempWritePlanStatus {
    Ready,
    Validated,
    RenderedToTempFixture,
    RereadVerified,
    BlockedFromRealConfig,
    ProductionWritesBlockedByDefault,
}

impl StructuredFamilyTempWritePlanStatus {
    pub fn as_str(self) -> &'static str {
        match self {
            Self::Ready => "StructuredFamilyTempWritePlanReady",
            Self::Validated => "StructuredFamilyTempWritePlanValidated",
            Self::RenderedToTempFixture => "StructuredFamilyTempWritePlanRenderedToTempFixture",
            Self::RereadVerified => "StructuredFamilyTempWritePlanRereadVerified",
            Self::BlockedFromRealConfig => "StructuredFamilyTempWritePlanBlockedFromRealConfig",
            Self::ProductionWritesBlockedByDefault => {
                "StructuredFamilyProductionWritesBlockedByDefault"
            }
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

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct StructuredFamilyValidationIssue {
    pub line_number: usize,
    pub status: StructuredFamilyValidationStatus,
    pub message: String,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct StructuredFamilyValidationResult {
    pub family: StructuredFamilyKind,
    pub status: StructuredFamilyValidationStatus,
    pub passed_count: usize,
    pub warning_count: usize,
    pub not_proven_count: usize,
    pub failed_count: usize,
    pub issues: Vec<StructuredFamilyValidationIssue>,
    pub real_config_touched: bool,
    pub runtime_mutated: bool,
    pub hyprctl_reload_run: bool,
    pub production_write_enabled: bool,
    pub production_executor_wired: bool,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct StructuredFamilyTempWritePlan {
    pub family: StructuredFamilyKind,
    pub source_fixture_path: PathBuf,
    pub temp_render_target_path: PathBuf,
    pub records_planned: usize,
    pub validation_status: StructuredFamilyValidationStatus,
    pub plan_status: StructuredFamilyTempWritePlanStatus,
    pub render_status: StructuredFamilyTempWritePlanStatus,
    pub reread_status: StructuredFamilyTempWritePlanStatus,
    pub path_guard_status: StructuredFamilyTempWritePlanStatus,
    pub real_config_touched: bool,
    pub runtime_mutated: bool,
    pub hyprctl_reload_run: bool,
    pub production_write_enabled: bool,
    pub production_executor_wired: bool,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct StructuredFamilyTempWriteProof {
    pub family: StructuredFamilyKind,
    pub validation_status: StructuredFamilyValidationStatus,
    pub plan_status: StructuredFamilyTempWritePlanStatus,
    pub render_status: StructuredFamilyTempWritePlanStatus,
    pub reread_status: StructuredFamilyTempWritePlanStatus,
    pub path_guard_status: StructuredFamilyTempWritePlanStatus,
    pub original_record_count: usize,
    pub reread_record_count: usize,
    pub family_identity_preserved: bool,
    pub record_count_preserved: bool,
    pub record_count_explanation: String,
    pub real_config_touched: bool,
    pub runtime_mutated: bool,
    pub hyprctl_reload_run: bool,
    pub production_write_enabled: bool,
    pub production_executor_wired: bool,
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

pub fn validate_structured_family_projection(
    projection: &StructuredFamilyProjection,
) -> StructuredFamilyValidationResult {
    let mut issues = Vec::new();
    let mut passed_count = 0;
    let mut warning_count = 0;
    let mut not_proven_count = 0;
    let mut failed_count = 0;

    for record in &projection.records {
        let issue = validate_structured_family_record(record);
        match issue.status {
            StructuredFamilyValidationStatus::Passed => passed_count += 1,
            StructuredFamilyValidationStatus::Warning => warning_count += 1,
            StructuredFamilyValidationStatus::NotProvenYet => not_proven_count += 1,
            StructuredFamilyValidationStatus::Failed => failed_count += 1,
            StructuredFamilyValidationStatus::Ready => {}
        }
        issues.push(issue);
    }

    let status = if projection.records.is_empty() || failed_count > 0 {
        StructuredFamilyValidationStatus::Failed
    } else if not_proven_count > 0 {
        StructuredFamilyValidationStatus::NotProvenYet
    } else if warning_count > 0 {
        StructuredFamilyValidationStatus::Warning
    } else {
        StructuredFamilyValidationStatus::Passed
    };

    StructuredFamilyValidationResult {
        family: projection.family,
        status,
        passed_count,
        warning_count,
        not_proven_count,
        failed_count,
        issues,
        real_config_touched: false,
        runtime_mutated: false,
        hyprctl_reload_run: false,
        production_write_enabled: false,
        production_executor_wired: false,
    }
}

pub fn build_structured_family_temp_write_plan(
    projection: &StructuredFamilyProjection,
    source_fixture_path: impl AsRef<Path>,
    temp_render_target_path: impl AsRef<Path>,
) -> StructuredFamilyTempWritePlan {
    let validation = validate_structured_family_projection(projection);
    let temp_render_target_path = temp_render_target_path.as_ref().to_path_buf();
    let path_guard_status = if structured_family_render_target_allowed(&temp_render_target_path) {
        StructuredFamilyTempWritePlanStatus::Ready
    } else {
        StructuredFamilyTempWritePlanStatus::BlockedFromRealConfig
    };
    let plan_status = if path_guard_status == StructuredFamilyTempWritePlanStatus::Ready
        && validation.status != StructuredFamilyValidationStatus::Failed
    {
        StructuredFamilyTempWritePlanStatus::Validated
    } else {
        StructuredFamilyTempWritePlanStatus::BlockedFromRealConfig
    };

    StructuredFamilyTempWritePlan {
        family: projection.family,
        source_fixture_path: source_fixture_path.as_ref().to_path_buf(),
        temp_render_target_path,
        records_planned: projection.record_count(),
        validation_status: validation.status,
        plan_status,
        render_status: StructuredFamilyTempWritePlanStatus::Ready,
        reread_status: StructuredFamilyTempWritePlanStatus::Ready,
        path_guard_status,
        real_config_touched: false,
        runtime_mutated: false,
        hyprctl_reload_run: false,
        production_write_enabled: false,
        production_executor_wired: false,
    }
}

pub fn prove_structured_family_temp_write_plan(
    projection: &StructuredFamilyProjection,
    source_fixture_path: impl AsRef<Path>,
    temp_render_target_path: impl AsRef<Path>,
) -> StructuredFamilyTempWriteProof {
    let plan = build_structured_family_temp_write_plan(
        projection,
        source_fixture_path,
        temp_render_target_path,
    );
    if plan.path_guard_status == StructuredFamilyTempWritePlanStatus::BlockedFromRealConfig
        || plan.plan_status == StructuredFamilyTempWritePlanStatus::BlockedFromRealConfig
    {
        return StructuredFamilyTempWriteProof {
            family: projection.family,
            validation_status: plan.validation_status,
            plan_status: plan.plan_status,
            render_status: plan.render_status,
            reread_status: plan.reread_status,
            path_guard_status: plan.path_guard_status,
            original_record_count: projection.record_count(),
            reread_record_count: 0,
            family_identity_preserved: false,
            record_count_preserved: false,
            record_count_explanation: "render target rejected by structured-family path guard"
                .to_string(),
            real_config_touched: false,
            runtime_mutated: false,
            hyprctl_reload_run: false,
            production_write_enabled: false,
            production_executor_wired: false,
        };
    }

    let rendered = render_structured_family_projection(projection);
    let render_status = match plan.temp_render_target_path.parent() {
        Some(parent) => fs::create_dir_all(parent)
            .and_then(|_| fs::write(&plan.temp_render_target_path, rendered.as_bytes()))
            .map(|_| StructuredFamilyTempWritePlanStatus::RenderedToTempFixture)
            .unwrap_or(StructuredFamilyTempWritePlanStatus::BlockedFromRealConfig),
        None => StructuredFamilyTempWritePlanStatus::BlockedFromRealConfig,
    };

    if render_status != StructuredFamilyTempWritePlanStatus::RenderedToTempFixture {
        return StructuredFamilyTempWriteProof {
            family: projection.family,
            validation_status: plan.validation_status,
            plan_status: plan.plan_status,
            render_status,
            reread_status: StructuredFamilyTempWritePlanStatus::Ready,
            path_guard_status: plan.path_guard_status,
            original_record_count: projection.record_count(),
            reread_record_count: 0,
            family_identity_preserved: false,
            record_count_preserved: false,
            record_count_explanation: "temp fixture render did not complete".to_string(),
            real_config_touched: false,
            runtime_mutated: false,
            hyprctl_reload_run: false,
            production_write_enabled: false,
            production_executor_wired: false,
        };
    }

    let reread_count = fs::read_to_string(&plan.temp_render_target_path)
        .map(|contents| {
            contents
                .lines()
                .filter(|line| line_belongs_to_family(projection.family, line))
                .count()
        })
        .unwrap_or(0);
    let record_count_preserved = projection.record_count() == reread_count;

    StructuredFamilyTempWriteProof {
        family: projection.family,
        validation_status: plan.validation_status,
        plan_status: plan.plan_status,
        render_status,
        reread_status: if record_count_preserved {
            StructuredFamilyTempWritePlanStatus::RereadVerified
        } else {
            StructuredFamilyTempWritePlanStatus::Ready
        },
        path_guard_status: plan.path_guard_status,
        original_record_count: projection.record_count(),
        reread_record_count: reread_count,
        family_identity_preserved: record_count_preserved,
        record_count_preserved,
        record_count_explanation: if record_count_preserved {
            "record count and family identity preserved after temp fixture reread".to_string()
        } else {
            "record count changed after temp fixture reread".to_string()
        },
        real_config_touched: false,
        runtime_mutated: false,
        hyprctl_reload_run: false,
        production_write_enabled: false,
        production_executor_wired: false,
    }
}

pub fn structured_family_render_target_allowed(path: &Path) -> bool {
    if path.starts_with("/home/kyo/.config/hypr") || path.starts_with("/home/kyo/.config/hyprland")
    {
        return false;
    }
    if let Ok(home) = std::env::var("HOME") {
        let home = PathBuf::from(home);
        if path.starts_with(home.join(".config/hypr"))
            || path.starts_with(home.join(".config/hyprland"))
        {
            return false;
        }
    }

    let temp_dir = std::env::temp_dir();
    if path.starts_with(&temp_dir) {
        return true;
    }

    let mut saw_tests = false;
    for component in path.components() {
        let text = component.as_os_str().to_string_lossy();
        if text == "tests" {
            saw_tests = true;
        }
        if saw_tests && matches!(text.as_ref(), "fixtures" | "temp" | "tmp") {
            return true;
        }
    }
    false
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

fn validate_structured_family_record(
    record: &StructuredFamilyRecord,
) -> StructuredFamilyValidationIssue {
    let parts = split_fields(record.raw_value.as_deref());
    let message = match record.family {
        StructuredFamilyKind::Monitor => validate_monitor_record(&parts),
        StructuredFamilyKind::Bind => validate_bind_record(&record.parsed_key, &parts),
        StructuredFamilyKind::Animation => validate_animation_record(&parts),
        StructuredFamilyKind::Curve => validate_curve_record(&parts),
        StructuredFamilyKind::Gesture => validate_gesture_record(&parts),
        StructuredFamilyKind::Device => validate_device_record(record.raw_value.as_deref()),
        StructuredFamilyKind::Permission => validate_permission_record(&parts),
    };
    StructuredFamilyValidationIssue {
        line_number: record.line_number,
        status: message.0,
        message: message.1,
    }
}

fn validate_monitor_record(parts: &[String]) -> (StructuredFamilyValidationStatus, String) {
    if parts.len() < 4 {
        return not_proven("monitor record missing resolution, position, or scale");
    }
    if !monitor_resolution_shape_supported(&parts[1]) {
        return not_proven("monitor resolution form is not proven yet");
    }
    if !monitor_position_shape_supported(&parts[2]) {
        return not_proven("monitor position form is not proven yet");
    }
    if parts[3].parse::<f64>().is_err() {
        return not_proven("monitor scale does not parse as a number");
    }
    if parts.len() > 4 {
        return warning("monitor extra options preserved as raw fixture data");
    }
    passed("monitor fixture shape validated")
}

fn validate_bind_record(
    parsed_key: &str,
    parts: &[String],
) -> (StructuredFamilyValidationStatus, String) {
    if !matches!(parsed_key, "bind" | "bindm" | "bindl" | "bindr" | "binde") {
        return not_proven("bind variant is preserved but not proven yet");
    }
    if parts.len() < 3 {
        return not_proven("bind record missing key or dispatcher");
    }
    passed("bind fixture shape validated")
}

fn validate_animation_record(parts: &[String]) -> (StructuredFamilyValidationStatus, String) {
    if parts.len() < 4 {
        return not_proven("animation record missing curve/reference or speed");
    }
    if !is_boolish_or_integer(&parts[1]) {
        return not_proven("animation enabled value is not boolean-ish or integer-ish");
    }
    if parts[3].parse::<f64>().is_err() {
        return not_proven("animation speed does not parse as a number");
    }
    if parts.len() > 4 {
        return warning("animation style/additional parameters preserved as raw fixture data");
    }
    passed("animation fixture shape validated")
}

fn validate_curve_record(parts: &[String]) -> (StructuredFamilyValidationStatus, String) {
    if parts.len() != 5 {
        return not_proven("curve record does not have exactly four control-point values");
    }
    if parts[1..].iter().any(|value| value.parse::<f64>().is_err()) {
        return not_proven("curve control point does not parse as a number");
    }
    passed("curve fixture shape validated")
}

fn validate_gesture_record(parts: &[String]) -> (StructuredFamilyValidationStatus, String) {
    if parts.len() < 3 {
        return not_proven("gesture record missing direction/action or dispatcher");
    }
    if parts.len() > 4 {
        return warning("gesture additional parameters preserved as raw fixture data");
    }
    passed("gesture fixture shape validated")
}

fn validate_device_record(raw_value: Option<&str>) -> (StructuredFamilyValidationStatus, String) {
    let Some(value) = raw_value.map(str::trim).filter(|value| !value.is_empty()) else {
        return not_proven("device record is preserved as raw block metadata");
    };
    if matches!(value, "device {" | "}") {
        return not_proven("device block boundary preserved as raw fixture data");
    }
    if let Some((key, value)) = value.split_once('=') {
        if key.trim().is_empty() || value.trim().is_empty() {
            return not_proven("device option key or value is missing");
        }
        return passed("device option fixture shape validated");
    }
    not_proven("device option shape is not proven yet")
}

fn validate_permission_record(parts: &[String]) -> (StructuredFamilyValidationStatus, String) {
    if parts.len() < 3 {
        return not_proven("permission record missing permission key or action");
    }
    passed("permission fixture shape validated")
}

fn monitor_resolution_shape_supported(value: &str) -> bool {
    value == "preferred" || value.contains('x')
}

fn monitor_position_shape_supported(value: &str) -> bool {
    value == "auto" || value.contains('x')
}

fn is_boolish_or_integer(value: &str) -> bool {
    matches!(
        value.trim().to_ascii_lowercase().as_str(),
        "true" | "false" | "yes" | "no" | "on" | "off"
    ) || value.parse::<i64>().is_ok()
}

fn passed(message: &str) -> (StructuredFamilyValidationStatus, String) {
    (
        StructuredFamilyValidationStatus::Passed,
        message.to_string(),
    )
}

fn warning(message: &str) -> (StructuredFamilyValidationStatus, String) {
    (
        StructuredFamilyValidationStatus::Warning,
        message.to_string(),
    )
}

fn not_proven(message: &str) -> (StructuredFamilyValidationStatus, String) {
    (
        StructuredFamilyValidationStatus::NotProvenYet,
        message.to_string(),
    )
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

fn line_belongs_to_family(family: StructuredFamilyKind, line: &str) -> bool {
    let trimmed = line.trim();
    if trimmed.is_empty() || trimmed.starts_with('#') {
        return false;
    }
    match family {
        StructuredFamilyKind::Monitor => trimmed.starts_with("monitor"),
        StructuredFamilyKind::Bind => trimmed.starts_with("bind"),
        StructuredFamilyKind::Animation => trimmed.starts_with("animation"),
        StructuredFamilyKind::Curve => trimmed.starts_with("bezier"),
        StructuredFamilyKind::Gesture => trimmed.starts_with("gesture"),
        StructuredFamilyKind::Device => true,
        StructuredFamilyKind::Permission => trimmed.starts_with("permission"),
    }
}

pub fn structured_family_kind_from_id(family_id: &str) -> Option<StructuredFamilyKind> {
    StructuredFamilyKind::from_family_id(family_id)
}
