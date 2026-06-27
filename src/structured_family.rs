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

    pub fn record_editor_widget_name(self) -> &'static str {
        match self {
            Self::Monitor => "hyprland-settings-structured-family-hl-monitor-record-editor",
            Self::Bind => "hyprland-settings-structured-family-hl-bind-record-editor",
            Self::Animation => "hyprland-settings-structured-family-hl-animation-record-editor",
            Self::Curve => "hyprland-settings-structured-family-hl-curve-record-editor",
            Self::Gesture => "hyprland-settings-structured-family-hl-gesture-record-editor",
            Self::Device => "hyprland-settings-structured-family-hl-device-record-editor",
            Self::Permission => "hyprland-settings-structured-family-hl-permission-record-editor",
        }
    }

    pub fn record_draft_widget_name(self) -> &'static str {
        match self {
            Self::Monitor => "hyprland-settings-structured-family-hl-monitor-record-draft",
            Self::Bind => "hyprland-settings-structured-family-hl-bind-record-draft",
            Self::Animation => "hyprland-settings-structured-family-hl-animation-record-draft",
            Self::Curve => "hyprland-settings-structured-family-hl-curve-record-draft",
            Self::Gesture => "hyprland-settings-structured-family-hl-gesture-record-draft",
            Self::Device => "hyprland-settings-structured-family-hl-device-record-draft",
            Self::Permission => "hyprland-settings-structured-family-hl-permission-record-draft",
        }
    }

    pub fn record_draft_binding_widget_name(self) -> &'static str {
        match self {
            Self::Monitor => "hyprland-settings-structured-family-hl-monitor-record-draft-binding",
            Self::Bind => "hyprland-settings-structured-family-hl-bind-record-draft-binding",
            Self::Animation => {
                "hyprland-settings-structured-family-hl-animation-record-draft-binding"
            }
            Self::Curve => "hyprland-settings-structured-family-hl-curve-record-draft-binding",
            Self::Gesture => "hyprland-settings-structured-family-hl-gesture-record-draft-binding",
            Self::Device => "hyprland-settings-structured-family-hl-device-record-draft-binding",
            Self::Permission => {
                "hyprland-settings-structured-family-hl-permission-record-draft-binding"
            }
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

    pub fn disabled_record_edit_label(self) -> &'static str {
        match self {
            Self::Monitor => "Edit monitor record (not available)",
            Self::Bind => "Edit bind record (not available)",
            Self::Animation => "Edit animation record (not available)",
            Self::Curve => "Edit curve record (not available)",
            Self::Gesture => "Edit gesture record (not available)",
            Self::Device => "Edit device record (not available)",
            Self::Permission => "Edit permission record (not available)",
        }
    }

    pub fn disabled_record_draft_update_label(self) -> &'static str {
        match self {
            Self::Monitor => "Update monitor draft (not available)",
            Self::Bind => "Update bind draft (not available)",
            Self::Animation => "Update animation draft (not available)",
            Self::Curve => "Update curve draft (not available)",
            Self::Gesture => "Update gesture draft (not available)",
            Self::Device => "Update device draft (not available)",
            Self::Permission => "Update permission draft (not available)",
        }
    }

    pub fn disabled_record_draft_binding_update_label(self) -> &'static str {
        match self {
            Self::Monitor => "Update monitor draft field (not available)",
            Self::Bind => "Update bind draft field (not available)",
            Self::Animation => "Update animation draft field (not available)",
            Self::Curve => "Update curve draft field (not available)",
            Self::Gesture => "Update gesture draft field (not available)",
            Self::Device => "Update device draft field (not available)",
            Self::Permission => "Update permission draft field (not available)",
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

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum StructuredFamilyRecordEditorStatus {
    Unavailable,
    ReviewOnly,
    ProjectionReady,
    ValidationReady,
    RawFallbackRequired,
    ActionsDisabled,
    WritesBlockedByDefault,
}

impl StructuredFamilyRecordEditorStatus {
    pub fn as_str(self) -> &'static str {
        match self {
            Self::Unavailable => "StructuredFamilyRecordEditorUnavailable",
            Self::ReviewOnly => "StructuredFamilyRecordEditorReviewOnly",
            Self::ProjectionReady => "StructuredFamilyRecordEditorProjectionReady",
            Self::ValidationReady => "StructuredFamilyRecordEditorValidationReady",
            Self::RawFallbackRequired => "StructuredFamilyRecordEditorRawFallbackRequired",
            Self::ActionsDisabled => "StructuredFamilyRecordEditorActionsDisabled",
            Self::WritesBlockedByDefault => "StructuredFamilyRecordEditorWritesBlockedByDefault",
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum StructuredFamilyRecordEditorFieldKind {
    FamilySpecific,
    ParsedKey,
    RawLine,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum StructuredFamilyRecordDraftStatus {
    Unavailable,
    ReviewOnly,
    ProjectionReady,
    CreatedInMemory,
    Dirty,
    Clean,
    ValidationReady,
    ValidationPassed,
    ValidationWarning,
    RawFallbackRequired,
    ActionsDisabled,
    WritesBlockedByDefault,
    PersistenceForbidden,
}

impl StructuredFamilyRecordDraftStatus {
    pub fn as_str(self) -> &'static str {
        match self {
            Self::Unavailable => "StructuredFamilyRecordDraftUnavailable",
            Self::ReviewOnly => "StructuredFamilyRecordDraftReviewOnly",
            Self::ProjectionReady => "StructuredFamilyRecordDraftProjectionReady",
            Self::CreatedInMemory => "StructuredFamilyRecordDraftCreatedInMemory",
            Self::Dirty => "StructuredFamilyRecordDraftDirty",
            Self::Clean => "StructuredFamilyRecordDraftClean",
            Self::ValidationReady => "StructuredFamilyRecordDraftValidationReady",
            Self::ValidationPassed => "StructuredFamilyRecordDraftValidationPassed",
            Self::ValidationWarning => "StructuredFamilyRecordDraftValidationWarning",
            Self::RawFallbackRequired => "StructuredFamilyRecordDraftRawFallbackRequired",
            Self::ActionsDisabled => "StructuredFamilyRecordDraftActionsDisabled",
            Self::WritesBlockedByDefault => "StructuredFamilyRecordDraftWritesBlockedByDefault",
            Self::PersistenceForbidden => "StructuredFamilyRecordDraftPersistenceForbidden",
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum StructuredFamilyRecordDraftGtkBindingStatus {
    Unavailable,
    ReviewOnly,
    ProjectionReady,
    CreatedInMemory,
    Disabled,
    CanUpdateMemoryOnly,
    DirtyStateRecomputed,
    ValidationRecomputed,
    RawFallbackPreserved,
    ActionsDisabled,
    WritesBlockedByDefault,
    PersistenceForbidden,
}

impl StructuredFamilyRecordDraftGtkBindingStatus {
    pub fn as_str(self) -> &'static str {
        match self {
            Self::Unavailable => "StructuredFamilyRecordDraftGtkBindingUnavailable",
            Self::ReviewOnly => "StructuredFamilyRecordDraftGtkBindingReviewOnly",
            Self::ProjectionReady => "StructuredFamilyRecordDraftGtkBindingProjectionReady",
            Self::CreatedInMemory => "StructuredFamilyRecordDraftGtkBindingCreatedInMemory",
            Self::Disabled => "StructuredFamilyRecordDraftGtkBindingDisabled",
            Self::CanUpdateMemoryOnly => "StructuredFamilyRecordDraftGtkBindingCanUpdateMemoryOnly",
            Self::DirtyStateRecomputed => {
                "StructuredFamilyRecordDraftGtkBindingDirtyStateRecomputed"
            }
            Self::ValidationRecomputed => {
                "StructuredFamilyRecordDraftGtkBindingValidationRecomputed"
            }
            Self::RawFallbackPreserved => {
                "StructuredFamilyRecordDraftGtkBindingRawFallbackPreserved"
            }
            Self::ActionsDisabled => "StructuredFamilyRecordDraftGtkBindingActionsDisabled",
            Self::WritesBlockedByDefault => {
                "StructuredFamilyRecordDraftGtkBindingWritesBlockedByDefault"
            }
            Self::PersistenceForbidden => {
                "StructuredFamilyRecordDraftGtkBindingPersistenceForbidden"
            }
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum StructuredFamilyDraftRenderedRecordStatus {
    Unavailable,
    ReviewOnly,
    PlanReady,
    CreatedInMemory,
    FixtureOnly,
    FieldMapReady,
    SyntaxProjected,
    RawFallbackPreserved,
    UnsupportedNotProvenYet,
    ActionsDisabled,
    WritesBlockedByDefault,
    PersistenceForbidden,
    RealConfigTargetForbidden,
}

impl StructuredFamilyDraftRenderedRecordStatus {
    pub fn as_str(self) -> &'static str {
        match self {
            Self::Unavailable => "StructuredFamilyDraftRenderedRecordUnavailable",
            Self::ReviewOnly => "StructuredFamilyDraftRenderedRecordReviewOnly",
            Self::PlanReady => "StructuredFamilyDraftRenderedRecordPlanReady",
            Self::CreatedInMemory => "StructuredFamilyDraftRenderedRecordCreatedInMemory",
            Self::FixtureOnly => "StructuredFamilyDraftRenderedRecordFixtureOnly",
            Self::FieldMapReady => "StructuredFamilyDraftRenderedRecordFieldMapReady",
            Self::SyntaxProjected => "StructuredFamilyDraftRenderedRecordSyntaxProjected",
            Self::RawFallbackPreserved => "StructuredFamilyDraftRenderedRecordRawFallbackPreserved",
            Self::UnsupportedNotProvenYet => {
                "StructuredFamilyDraftRenderedRecordUnsupportedNotProvenYet"
            }
            Self::ActionsDisabled => "StructuredFamilyDraftRenderedRecordActionsDisabled",
            Self::WritesBlockedByDefault => {
                "StructuredFamilyDraftRenderedRecordWritesBlockedByDefault"
            }
            Self::PersistenceForbidden => "StructuredFamilyDraftRenderedRecordPersistenceForbidden",
            Self::RealConfigTargetForbidden => {
                "StructuredFamilyDraftRenderedRecordRealConfigTargetForbidden"
            }
        }
    }
}

impl StructuredFamilyRecordEditorFieldKind {
    pub fn as_str(self) -> &'static str {
        match self {
            Self::FamilySpecific => "family-specific",
            Self::ParsedKey => "parsed-key",
            Self::RawLine => "raw-line",
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

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct StructuredFamilyRecordEditorField {
    pub name: String,
    pub value: String,
    pub kind: StructuredFamilyRecordEditorFieldKind,
    pub editable: bool,
    pub editability_status: String,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct StructuredFamilyRecordEditorForm {
    pub family: StructuredFamilyKind,
    pub record_index: usize,
    pub source_path: PathBuf,
    pub line_number: usize,
    pub raw_line: String,
    pub parsed_key: String,
    pub validation_status: String,
    pub unsupported_reason: Option<String>,
    pub fields: Vec<StructuredFamilyRecordEditorField>,
    pub field_editability_status: String,
    pub raw_fallback_status: String,
    pub action_policy: StructuredFamilyRecordEditorStatus,
    pub write_blocked_status: StructuredFamilyRecordEditorStatus,
    pub temp_fixture_plan_status: StructuredFamilyTempWritePlanStatus,
    pub projection_status: StructuredFamilyRecordEditorStatus,
    pub review_status: StructuredFamilyRecordEditorStatus,
    pub real_config_touched: bool,
    pub runtime_mutated: bool,
    pub hyprctl_reload_run: bool,
    pub production_executor_wired: bool,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct StructuredFamilyRecordDraftField {
    pub name: String,
    pub original_value: String,
    pub draft_value: String,
    pub kind: StructuredFamilyRecordEditorFieldKind,
    pub dirty: bool,
    pub editable: bool,
    pub editability_status: String,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct StructuredFamilyRecordDraft {
    pub family: StructuredFamilyKind,
    pub record_index: usize,
    pub source_path: PathBuf,
    pub line_number: usize,
    pub raw_original_line: String,
    pub parsed_key: String,
    pub original_fields: Vec<StructuredFamilyRecordDraftField>,
    pub draft_fields: Vec<StructuredFamilyRecordDraftField>,
    pub dirty_state: StructuredFamilyRecordDraftStatus,
    pub validation_status: StructuredFamilyRecordDraftStatus,
    pub unsupported_reason: Option<String>,
    pub raw_fallback_status: String,
    pub reset_status: String,
    pub action_policy: StructuredFamilyRecordDraftStatus,
    pub write_blocked_status: StructuredFamilyRecordDraftStatus,
    pub persistence_policy: StructuredFamilyRecordDraftStatus,
    pub temp_fixture_plan_status: StructuredFamilyTempWritePlanStatus,
    pub projection_status: StructuredFamilyRecordDraftStatus,
    pub review_status: StructuredFamilyRecordDraftStatus,
    pub created_status: StructuredFamilyRecordDraftStatus,
    pub real_config_touched: bool,
    pub runtime_mutated: bool,
    pub hyprctl_reload_run: bool,
    pub production_executor_wired: bool,
    pub draft_written_to_disk: bool,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct StructuredFamilyRecordDraftResetProof {
    pub family: StructuredFamilyKind,
    pub record_index: usize,
    pub original_fields_restored: bool,
    pub dirty_state_after_reset: StructuredFamilyRecordDraftStatus,
    pub real_config_touched: bool,
    pub runtime_mutated: bool,
    pub hyprctl_reload_run: bool,
    pub production_executor_wired: bool,
    pub draft_written_to_disk: bool,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct StructuredFamilyRecordDraftGtkField {
    pub family: StructuredFamilyKind,
    pub record_index: usize,
    pub source_path: PathBuf,
    pub line_number: usize,
    pub field_name: String,
    pub field_kind: StructuredFamilyRecordEditorFieldKind,
    pub original_value: String,
    pub display_value: String,
    pub draft_value: String,
    pub widget_kind: String,
    pub widget_sensitive: bool,
    pub binding_status: StructuredFamilyRecordDraftGtkBindingStatus,
    pub dirty_state: StructuredFamilyRecordDraftStatus,
    pub validation_status: StructuredFamilyRecordDraftStatus,
    pub raw_fallback_status: String,
    pub action_policy: StructuredFamilyRecordDraftGtkBindingStatus,
    pub write_policy: StructuredFamilyRecordDraftGtkBindingStatus,
    pub persistence_policy: StructuredFamilyRecordDraftGtkBindingStatus,
    pub real_config_touched: bool,
    pub runtime_mutated: bool,
    pub hyprctl_reload_run: bool,
    pub production_executor_wired: bool,
    pub draft_written_to_disk: bool,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct StructuredFamilyRecordDraftGtkBinding {
    pub family: StructuredFamilyKind,
    pub record_index: usize,
    pub source_path: PathBuf,
    pub line_number: usize,
    pub fields: Vec<StructuredFamilyRecordDraftGtkField>,
    pub binding_status: StructuredFamilyRecordDraftGtkBindingStatus,
    pub review_status: StructuredFamilyRecordDraftGtkBindingStatus,
    pub created_status: StructuredFamilyRecordDraftGtkBindingStatus,
    pub widget_sensitive: bool,
    pub dirty_state: StructuredFamilyRecordDraftStatus,
    pub validation_status: StructuredFamilyRecordDraftStatus,
    pub raw_fallback_status: String,
    pub action_policy: StructuredFamilyRecordDraftGtkBindingStatus,
    pub write_policy: StructuredFamilyRecordDraftGtkBindingStatus,
    pub persistence_policy: StructuredFamilyRecordDraftGtkBindingStatus,
    pub reset_status: String,
    pub real_config_touched: bool,
    pub runtime_mutated: bool,
    pub hyprctl_reload_run: bool,
    pub production_executor_wired: bool,
    pub draft_written_to_disk: bool,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct StructuredFamilyRecordDraftGtkUpdateResult {
    pub updated_draft: StructuredFamilyRecordDraft,
    pub binding: StructuredFamilyRecordDraftGtkBinding,
    pub update_status: StructuredFamilyRecordDraftGtkBindingStatus,
    pub dirty_state_recomputed: StructuredFamilyRecordDraftGtkBindingStatus,
    pub validation_recomputed: StructuredFamilyRecordDraftGtkBindingStatus,
    pub raw_fallback_preserved: StructuredFamilyRecordDraftGtkBindingStatus,
    pub reset_restores_original_fields: bool,
    pub real_config_touched: bool,
    pub runtime_mutated: bool,
    pub hyprctl_reload_run: bool,
    pub production_executor_wired: bool,
    pub draft_written_to_disk: bool,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct StructuredFamilyDraftRenderedRecordFieldMap {
    pub field_name: String,
    pub draft_value: String,
    pub rendered_part: String,
    pub status: StructuredFamilyDraftRenderedRecordStatus,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct StructuredFamilyDraftRenderedRecordPlan {
    pub family: StructuredFamilyKind,
    pub record_index: usize,
    pub source_path: PathBuf,
    pub line_number: usize,
    pub raw_original_line: String,
    pub parsed_key: String,
    pub draft_fields: Vec<StructuredFamilyRecordDraftField>,
    pub field_map: Vec<StructuredFamilyDraftRenderedRecordFieldMap>,
    pub rendered_record_preview: String,
    pub rendered_record_syntax_status: StructuredFamilyDraftRenderedRecordStatus,
    pub raw_fallback_status: StructuredFamilyDraftRenderedRecordStatus,
    pub unsupported_not_proven_status: StructuredFamilyDraftRenderedRecordStatus,
    pub unsupported_reason: Option<String>,
    pub fixture_only_status: StructuredFamilyDraftRenderedRecordStatus,
    pub action_policy: StructuredFamilyDraftRenderedRecordStatus,
    pub write_policy: StructuredFamilyDraftRenderedRecordStatus,
    pub persistence_policy: StructuredFamilyDraftRenderedRecordStatus,
    pub real_config_target_policy: StructuredFamilyDraftRenderedRecordStatus,
    pub temp_fixture_plan_status: StructuredFamilyTempWritePlanStatus,
    pub review_status: StructuredFamilyDraftRenderedRecordStatus,
    pub plan_status: StructuredFamilyDraftRenderedRecordStatus,
    pub created_status: StructuredFamilyDraftRenderedRecordStatus,
    pub draft_written_to_disk: bool,
    pub rendered_record_written_to_disk: bool,
    pub real_config_touched: bool,
    pub runtime_mutated: bool,
    pub hyprctl_reload_run: bool,
    pub production_executor_wired: bool,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct StructuredFamilyDraftRenderedRecordPlanProof {
    pub family: StructuredFamilyKind,
    pub plan_count: usize,
    pub draft_count: usize,
    pub field_map_count: usize,
    pub raw_fallback_plan_count: usize,
    pub unsupported_not_proven_plan_count: usize,
    pub fixture_only_status: StructuredFamilyDraftRenderedRecordStatus,
    pub write_policy: StructuredFamilyDraftRenderedRecordStatus,
    pub persistence_policy: StructuredFamilyDraftRenderedRecordStatus,
    pub real_config_target_policy: StructuredFamilyDraftRenderedRecordStatus,
    pub draft_written_to_disk: bool,
    pub rendered_record_written_to_disk: bool,
    pub real_config_touched: bool,
    pub runtime_mutated: bool,
    pub hyprctl_reload_run: bool,
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

pub fn structured_family_record_editor_forms(
    projection: &StructuredFamilyProjection,
) -> Vec<StructuredFamilyRecordEditorForm> {
    projection
        .records
        .iter()
        .enumerate()
        .map(|(index, record)| structured_family_record_editor_form(projection, index, record))
        .collect()
}

pub fn structured_family_record_editor_form(
    projection: &StructuredFamilyProjection,
    record_index: usize,
    record: &StructuredFamilyRecord,
) -> StructuredFamilyRecordEditorForm {
    let mut fields = record
        .fields
        .iter()
        .map(|field| StructuredFamilyRecordEditorField {
            name: field.name.clone(),
            value: field.value.clone(),
            kind: StructuredFamilyRecordEditorFieldKind::FamilySpecific,
            editable: false,
            editability_status: "review-only; editing disabled".to_string(),
        })
        .collect::<Vec<_>>();
    fields.push(StructuredFamilyRecordEditorField {
        name: "parsed key".to_string(),
        value: record.parsed_key.clone(),
        kind: StructuredFamilyRecordEditorFieldKind::ParsedKey,
        editable: false,
        editability_status: "review-only; editing disabled".to_string(),
    });
    fields.push(StructuredFamilyRecordEditorField {
        name: "raw line".to_string(),
        value: record.raw_line.clone(),
        kind: StructuredFamilyRecordEditorFieldKind::RawLine,
        editable: false,
        editability_status: "review-only; editing disabled".to_string(),
    });

    let raw_fallback_status = if record.unsupported_reason.is_some() {
        StructuredFamilyRecordEditorStatus::RawFallbackRequired.as_str()
    } else {
        "StructuredFamilyRecordEditorRawFallbackNotRequired"
    };

    StructuredFamilyRecordEditorForm {
        family: projection.family,
        record_index,
        source_path: record.source_path.clone(),
        line_number: record.line_number,
        raw_line: record.raw_line.clone(),
        parsed_key: record.parsed_key.clone(),
        validation_status: record.validation_status.clone(),
        unsupported_reason: record.unsupported_reason.clone(),
        fields,
        field_editability_status:
            "Family-specific fields projected; review-only field editability disabled".to_string(),
        raw_fallback_status: raw_fallback_status.to_string(),
        action_policy: StructuredFamilyRecordEditorStatus::ActionsDisabled,
        write_blocked_status: StructuredFamilyRecordEditorStatus::WritesBlockedByDefault,
        temp_fixture_plan_status: StructuredFamilyTempWritePlanStatus::Validated,
        projection_status: StructuredFamilyRecordEditorStatus::ProjectionReady,
        review_status: StructuredFamilyRecordEditorStatus::ReviewOnly,
        real_config_touched: false,
        runtime_mutated: false,
        hyprctl_reload_run: false,
        production_executor_wired: false,
    }
}

pub fn structured_family_record_drafts(
    forms: &[StructuredFamilyRecordEditorForm],
) -> Vec<StructuredFamilyRecordDraft> {
    forms
        .iter()
        .map(structured_family_record_draft_from_form)
        .collect()
}

pub fn structured_family_record_draft_from_form(
    form: &StructuredFamilyRecordEditorForm,
) -> StructuredFamilyRecordDraft {
    let draft_fields = form
        .fields
        .iter()
        .map(|field| StructuredFamilyRecordDraftField {
            name: field.name.clone(),
            original_value: field.value.clone(),
            draft_value: field.value.clone(),
            kind: field.kind,
            dirty: false,
            editable: false,
            editability_status: "review-only in-memory draft; editing disabled".to_string(),
        })
        .collect::<Vec<_>>();
    let raw_fallback_status = if form.unsupported_reason.is_some() {
        StructuredFamilyRecordDraftStatus::RawFallbackRequired.as_str()
    } else {
        "StructuredFamilyRecordDraftRawFallbackNotRequired"
    };

    StructuredFamilyRecordDraft {
        family: form.family,
        record_index: form.record_index,
        source_path: form.source_path.clone(),
        line_number: form.line_number,
        raw_original_line: form.raw_line.clone(),
        parsed_key: form.parsed_key.clone(),
        original_fields: draft_fields.clone(),
        draft_fields,
        dirty_state: StructuredFamilyRecordDraftStatus::Clean,
        validation_status: StructuredFamilyRecordDraftStatus::ValidationReady,
        unsupported_reason: form.unsupported_reason.clone(),
        raw_fallback_status: raw_fallback_status.to_string(),
        reset_status: "StructuredFamilyRecordDraftResetAvailableInMemoryOnly".to_string(),
        action_policy: StructuredFamilyRecordDraftStatus::ActionsDisabled,
        write_blocked_status: StructuredFamilyRecordDraftStatus::WritesBlockedByDefault,
        persistence_policy: StructuredFamilyRecordDraftStatus::PersistenceForbidden,
        temp_fixture_plan_status: form.temp_fixture_plan_status,
        projection_status: StructuredFamilyRecordDraftStatus::ProjectionReady,
        review_status: StructuredFamilyRecordDraftStatus::ReviewOnly,
        created_status: StructuredFamilyRecordDraftStatus::CreatedInMemory,
        real_config_touched: false,
        runtime_mutated: false,
        hyprctl_reload_run: false,
        production_executor_wired: false,
        draft_written_to_disk: false,
    }
}

pub fn update_structured_family_record_draft_field(
    draft: &StructuredFamilyRecordDraft,
    field_name: &str,
    new_value: impl Into<String>,
) -> StructuredFamilyRecordDraft {
    let new_value = new_value.into();
    let mut updated = draft.clone();
    for field in &mut updated.draft_fields {
        if field.name == field_name {
            field.draft_value = new_value.clone();
            field.dirty = field.draft_value != field.original_value;
        }
    }
    let has_dirty_field = updated.draft_fields.iter().any(|field| field.dirty);
    updated.dirty_state = if has_dirty_field {
        StructuredFamilyRecordDraftStatus::Dirty
    } else {
        StructuredFamilyRecordDraftStatus::Clean
    };
    updated.validation_status = if updated.unsupported_reason.is_some() {
        StructuredFamilyRecordDraftStatus::ValidationWarning
    } else {
        StructuredFamilyRecordDraftStatus::ValidationPassed
    };
    updated.real_config_touched = false;
    updated.runtime_mutated = false;
    updated.hyprctl_reload_run = false;
    updated.production_executor_wired = false;
    updated.draft_written_to_disk = false;
    updated
}

pub fn reset_structured_family_record_draft(
    draft: &StructuredFamilyRecordDraft,
) -> StructuredFamilyRecordDraft {
    let mut reset = draft.clone();
    reset.draft_fields = reset.original_fields.clone();
    reset.dirty_state = StructuredFamilyRecordDraftStatus::Clean;
    reset.validation_status = StructuredFamilyRecordDraftStatus::ValidationReady;
    reset.reset_status = "StructuredFamilyRecordDraftResetRestoredOriginalFields".to_string();
    reset.real_config_touched = false;
    reset.runtime_mutated = false;
    reset.hyprctl_reload_run = false;
    reset.production_executor_wired = false;
    reset.draft_written_to_disk = false;
    reset
}

pub fn prove_structured_family_record_draft_reset(
    draft: &StructuredFamilyRecordDraft,
) -> StructuredFamilyRecordDraftResetProof {
    let reset = reset_structured_family_record_draft(draft);
    StructuredFamilyRecordDraftResetProof {
        family: reset.family,
        record_index: reset.record_index,
        original_fields_restored: reset.draft_fields == reset.original_fields,
        dirty_state_after_reset: reset.dirty_state,
        real_config_touched: false,
        runtime_mutated: false,
        hyprctl_reload_run: false,
        production_executor_wired: false,
        draft_written_to_disk: false,
    }
}

pub fn structured_family_record_draft_gtk_bindings(
    drafts: &[StructuredFamilyRecordDraft],
) -> Vec<StructuredFamilyRecordDraftGtkBinding> {
    drafts
        .iter()
        .map(structured_family_record_draft_gtk_binding)
        .collect()
}

pub fn structured_family_record_draft_gtk_binding(
    draft: &StructuredFamilyRecordDraft,
) -> StructuredFamilyRecordDraftGtkBinding {
    let fields = draft
        .draft_fields
        .iter()
        .map(|field| StructuredFamilyRecordDraftGtkField {
            family: draft.family,
            record_index: draft.record_index,
            source_path: draft.source_path.clone(),
            line_number: draft.line_number,
            field_name: field.name.clone(),
            field_kind: field.kind,
            original_value: field.original_value.clone(),
            display_value: field.draft_value.clone(),
            draft_value: field.draft_value.clone(),
            widget_kind: "gtk::Entry (insensitive)".to_string(),
            widget_sensitive: false,
            binding_status: StructuredFamilyRecordDraftGtkBindingStatus::Disabled,
            dirty_state: draft.dirty_state,
            validation_status: draft.validation_status,
            raw_fallback_status: draft.raw_fallback_status.clone(),
            action_policy: StructuredFamilyRecordDraftGtkBindingStatus::ActionsDisabled,
            write_policy: StructuredFamilyRecordDraftGtkBindingStatus::WritesBlockedByDefault,
            persistence_policy: StructuredFamilyRecordDraftGtkBindingStatus::PersistenceForbidden,
            real_config_touched: false,
            runtime_mutated: false,
            hyprctl_reload_run: false,
            production_executor_wired: false,
            draft_written_to_disk: false,
        })
        .collect::<Vec<_>>();

    StructuredFamilyRecordDraftGtkBinding {
        family: draft.family,
        record_index: draft.record_index,
        source_path: draft.source_path.clone(),
        line_number: draft.line_number,
        fields,
        binding_status: StructuredFamilyRecordDraftGtkBindingStatus::ProjectionReady,
        review_status: StructuredFamilyRecordDraftGtkBindingStatus::ReviewOnly,
        created_status: StructuredFamilyRecordDraftGtkBindingStatus::CreatedInMemory,
        widget_sensitive: false,
        dirty_state: draft.dirty_state,
        validation_status: draft.validation_status,
        raw_fallback_status: draft.raw_fallback_status.clone(),
        action_policy: StructuredFamilyRecordDraftGtkBindingStatus::ActionsDisabled,
        write_policy: StructuredFamilyRecordDraftGtkBindingStatus::WritesBlockedByDefault,
        persistence_policy: StructuredFamilyRecordDraftGtkBindingStatus::PersistenceForbidden,
        reset_status: draft.reset_status.clone(),
        real_config_touched: false,
        runtime_mutated: false,
        hyprctl_reload_run: false,
        production_executor_wired: false,
        draft_written_to_disk: false,
    }
}

pub fn update_structured_family_record_draft_gtk_binding(
    draft: &StructuredFamilyRecordDraft,
    field_name: &str,
    new_value: impl Into<String>,
) -> StructuredFamilyRecordDraftGtkUpdateResult {
    let updated_draft = update_structured_family_record_draft_field(draft, field_name, new_value);
    let binding = structured_family_record_draft_gtk_binding(&updated_draft);
    let reset = reset_structured_family_record_draft(&updated_draft);

    StructuredFamilyRecordDraftGtkUpdateResult {
        updated_draft,
        binding,
        update_status: StructuredFamilyRecordDraftGtkBindingStatus::CanUpdateMemoryOnly,
        dirty_state_recomputed: StructuredFamilyRecordDraftGtkBindingStatus::DirtyStateRecomputed,
        validation_recomputed: StructuredFamilyRecordDraftGtkBindingStatus::ValidationRecomputed,
        raw_fallback_preserved: StructuredFamilyRecordDraftGtkBindingStatus::RawFallbackPreserved,
        reset_restores_original_fields: reset.draft_fields == reset.original_fields,
        real_config_touched: false,
        runtime_mutated: false,
        hyprctl_reload_run: false,
        production_executor_wired: false,
        draft_written_to_disk: false,
    }
}

pub fn structured_family_draft_rendered_record_plans(
    drafts: &[StructuredFamilyRecordDraft],
) -> Vec<StructuredFamilyDraftRenderedRecordPlan> {
    drafts
        .iter()
        .map(structured_family_draft_rendered_record_plan)
        .collect()
}

pub fn structured_family_draft_rendered_record_plan(
    draft: &StructuredFamilyRecordDraft,
) -> StructuredFamilyDraftRenderedRecordPlan {
    let field_map = draft
        .draft_fields
        .iter()
        .filter(|field| field.name != "parsed key")
        .map(|field| StructuredFamilyDraftRenderedRecordFieldMap {
            field_name: field.name.clone(),
            draft_value: field.draft_value.clone(),
            rendered_part: if field.name == "raw line" {
                "raw fallback source".to_string()
            } else {
                format!("{}={}", field.name, field.draft_value)
            },
            status: StructuredFamilyDraftRenderedRecordStatus::FieldMapReady,
        })
        .collect::<Vec<_>>();
    let unsupported = draft.unsupported_reason.is_some();
    let rendered_record_preview = if unsupported {
        draft.raw_original_line.clone()
    } else {
        rendered_record_preview_from_draft(draft).unwrap_or_else(|| draft.raw_original_line.clone())
    };

    StructuredFamilyDraftRenderedRecordPlan {
        family: draft.family,
        record_index: draft.record_index,
        source_path: draft.source_path.clone(),
        line_number: draft.line_number,
        raw_original_line: draft.raw_original_line.clone(),
        parsed_key: draft.parsed_key.clone(),
        draft_fields: draft.draft_fields.clone(),
        field_map,
        rendered_record_preview,
        rendered_record_syntax_status: if unsupported {
            StructuredFamilyDraftRenderedRecordStatus::UnsupportedNotProvenYet
        } else {
            StructuredFamilyDraftRenderedRecordStatus::SyntaxProjected
        },
        raw_fallback_status: if unsupported {
            StructuredFamilyDraftRenderedRecordStatus::RawFallbackPreserved
        } else {
            StructuredFamilyDraftRenderedRecordStatus::FieldMapReady
        },
        unsupported_not_proven_status: if unsupported {
            StructuredFamilyDraftRenderedRecordStatus::UnsupportedNotProvenYet
        } else {
            StructuredFamilyDraftRenderedRecordStatus::FieldMapReady
        },
        unsupported_reason: draft.unsupported_reason.clone(),
        fixture_only_status: StructuredFamilyDraftRenderedRecordStatus::FixtureOnly,
        action_policy: StructuredFamilyDraftRenderedRecordStatus::ActionsDisabled,
        write_policy: StructuredFamilyDraftRenderedRecordStatus::WritesBlockedByDefault,
        persistence_policy: StructuredFamilyDraftRenderedRecordStatus::PersistenceForbidden,
        real_config_target_policy:
            StructuredFamilyDraftRenderedRecordStatus::RealConfigTargetForbidden,
        temp_fixture_plan_status: draft.temp_fixture_plan_status,
        review_status: StructuredFamilyDraftRenderedRecordStatus::ReviewOnly,
        plan_status: StructuredFamilyDraftRenderedRecordStatus::PlanReady,
        created_status: StructuredFamilyDraftRenderedRecordStatus::CreatedInMemory,
        draft_written_to_disk: false,
        rendered_record_written_to_disk: false,
        real_config_touched: false,
        runtime_mutated: false,
        hyprctl_reload_run: false,
        production_executor_wired: false,
    }
}

pub fn prove_structured_family_draft_rendered_record_plans(
    drafts: &[StructuredFamilyRecordDraft],
) -> StructuredFamilyDraftRenderedRecordPlanProof {
    let family = drafts
        .first()
        .map(|draft| draft.family)
        .unwrap_or(StructuredFamilyKind::Monitor);
    let plans = structured_family_draft_rendered_record_plans(drafts);
    StructuredFamilyDraftRenderedRecordPlanProof {
        family,
        plan_count: plans.len(),
        draft_count: drafts.len(),
        field_map_count: plans.iter().map(|plan| plan.field_map.len()).sum(),
        raw_fallback_plan_count: plans
            .iter()
            .filter(|plan| {
                plan.raw_fallback_status
                    == StructuredFamilyDraftRenderedRecordStatus::RawFallbackPreserved
            })
            .count(),
        unsupported_not_proven_plan_count: plans
            .iter()
            .filter(|plan| {
                plan.unsupported_not_proven_status
                    == StructuredFamilyDraftRenderedRecordStatus::UnsupportedNotProvenYet
            })
            .count(),
        fixture_only_status: StructuredFamilyDraftRenderedRecordStatus::FixtureOnly,
        write_policy: StructuredFamilyDraftRenderedRecordStatus::WritesBlockedByDefault,
        persistence_policy: StructuredFamilyDraftRenderedRecordStatus::PersistenceForbidden,
        real_config_target_policy:
            StructuredFamilyDraftRenderedRecordStatus::RealConfigTargetForbidden,
        draft_written_to_disk: false,
        rendered_record_written_to_disk: false,
        real_config_touched: false,
        runtime_mutated: false,
        hyprctl_reload_run: false,
        production_executor_wired: false,
    }
}

fn rendered_record_preview_from_draft(draft: &StructuredFamilyRecordDraft) -> Option<String> {
    match draft.family {
        StructuredFamilyKind::Monitor => {
            let output = draft_field_value(draft, "name/output")?;
            let resolution = draft_field_value(draft, "resolution")?;
            let position = draft_field_value(draft, "position")?;
            let scale = draft_field_value(draft, "scale")?;
            let extra = draft_field_value(draft, "additional raw options");
            Some(match extra.filter(|value| !value.trim().is_empty()) {
                Some(extra) => {
                    format!("monitor = {output}, {resolution}, {position}, {scale}, {extra}")
                }
                None => format!("monitor = {output}, {resolution}, {position}, {scale}"),
            })
        }
        StructuredFamilyKind::Bind => {
            let modifier = draft_field_value(draft, "modifier")?;
            let key = draft_field_value(draft, "key")?;
            let dispatcher = draft_field_value(draft, "dispatcher")?;
            let argument = draft_field_value(draft, "argument");
            let variant = draft_field_value(draft, "flags/type")
                .unwrap_or(draft.parsed_key.as_str())
                .trim();
            Some(match argument.filter(|value| !value.trim().is_empty()) {
                Some(argument) => {
                    format!("{variant} = {modifier}, {key}, {dispatcher}, {argument}")
                }
                None => format!("{variant} = {modifier}, {key}, {dispatcher}"),
            })
        }
        StructuredFamilyKind::Animation => {
            let name = draft_field_value(draft, "name")?;
            let enabled = draft_field_value(draft, "enabled")?;
            let curve = draft_field_value(draft, "bezier/curve reference")?;
            let speed = draft_field_value(draft, "speed")?;
            let style = draft_field_value(draft, "style");
            let additional = draft_field_value(draft, "additional parameters");
            let mut parts = vec![name, enabled, speed, curve];
            if let Some(style) = style.filter(|value| !value.trim().is_empty()) {
                parts.push(style);
            }
            if let Some(additional) = additional.filter(|value| !value.trim().is_empty()) {
                parts.push(additional);
            }
            Some(format!("animation = {}", parts.join(", ")))
        }
        StructuredFamilyKind::Curve => {
            let name = draft_field_value(draft, "name")?;
            let x1 = draft_field_value(draft, "x1")?;
            let y1 = draft_field_value(draft, "y1")?;
            let x2 = draft_field_value(draft, "x2")?;
            let y2 = draft_field_value(draft, "y2")?;
            Some(format!("bezier = {name}, {x1}, {y1}, {x2}, {y2}"))
        }
        StructuredFamilyKind::Gesture => {
            if !draft.raw_original_line.trim().is_empty() {
                Some(draft.raw_original_line.clone())
            } else {
                None
            }
        }
        StructuredFamilyKind::Device => {
            let key = draft_field_value(draft, "option key")?;
            let value = draft_field_value(draft, "option value")?;
            Some(format!("{key} = {value}"))
        }
        StructuredFamilyKind::Permission => {
            if !draft.raw_original_line.trim().is_empty() {
                Some(draft.raw_original_line.clone())
            } else {
                None
            }
        }
    }
}

fn draft_field_value<'a>(draft: &'a StructuredFamilyRecordDraft, name: &str) -> Option<&'a str> {
    draft
        .draft_fields
        .iter()
        .find(|field| field.name == name)
        .map(|field| field.draft_value.as_str())
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
