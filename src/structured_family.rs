use std::fs;
use std::path::{Path, PathBuf};

use crate::config_parser::parse_hyprland_config_file;
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

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum StructuredFamilyDraftRenderedRecordRenderRereadStatus {
    Unavailable,
    ReviewOnly,
    Ready,
    RenderedToTempFixture,
    RereadFromTempFixture,
    FamilyPreserved,
    RecordCountPreserved,
    FieldMapPreserved,
    RawFallbackPreserved,
    UnsupportedNotProvenYet,
    FixtureOnly,
    ActionsDisabled,
    WritesBlockedByDefault,
    PersistenceForbidden,
    RealConfigTargetForbidden,
}

impl StructuredFamilyDraftRenderedRecordRenderRereadStatus {
    pub fn as_str(self) -> &'static str {
        match self {
            Self::Unavailable => "StructuredFamilyDraftRenderedRecordRenderRereadUnavailable",
            Self::ReviewOnly => "StructuredFamilyDraftRenderedRecordRenderRereadReviewOnly",
            Self::Ready => "StructuredFamilyDraftRenderedRecordRenderRereadReady",
            Self::RenderedToTempFixture => {
                "StructuredFamilyDraftRenderedRecordRenderedToTempFixture"
            }
            Self::RereadFromTempFixture => {
                "StructuredFamilyDraftRenderedRecordRereadFromTempFixture"
            }
            Self::FamilyPreserved => "StructuredFamilyDraftRenderedRecordFamilyPreserved",
            Self::RecordCountPreserved => "StructuredFamilyDraftRenderedRecordRecordCountPreserved",
            Self::FieldMapPreserved => "StructuredFamilyDraftRenderedRecordFieldMapPreserved",
            Self::RawFallbackPreserved => "StructuredFamilyDraftRenderedRecordRawFallbackPreserved",
            Self::UnsupportedNotProvenYet => {
                "StructuredFamilyDraftRenderedRecordUnsupportedNotProvenYet"
            }
            Self::FixtureOnly => "StructuredFamilyDraftRenderedRecordFixtureOnly",
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

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum StructuredFamilyDraftRenderedRecordDiffReviewStatus {
    Unavailable,
    ReviewOnly,
    Ready,
    DiffReady,
    ReviewSummaryReady,
    Noop,
    Changed,
    FieldDiffReady,
    RenderedPreviewCompared,
    OriginalRawPreserved,
    RawFallbackPreserved,
    UnsupportedNotProvenYet,
    RenderRereadProofLinked,
    FixtureOnly,
    ActionsDisabled,
    WritesBlockedByDefault,
    PersistenceForbidden,
    RealConfigTargetForbidden,
}

impl StructuredFamilyDraftRenderedRecordDiffReviewStatus {
    pub fn as_str(self) -> &'static str {
        match self {
            Self::Unavailable => "StructuredFamilyDraftRenderedRecordDiffReviewUnavailable",
            Self::ReviewOnly => "StructuredFamilyDraftRenderedRecordDiffReviewReviewOnly",
            Self::Ready => "StructuredFamilyDraftRenderedRecordDiffReviewReady",
            Self::DiffReady => "StructuredFamilyDraftRenderedRecordDiffReady",
            Self::ReviewSummaryReady => "StructuredFamilyDraftRenderedRecordReviewSummaryReady",
            Self::Noop => "StructuredFamilyDraftRenderedRecordNoop",
            Self::Changed => "StructuredFamilyDraftRenderedRecordChanged",
            Self::FieldDiffReady => "StructuredFamilyDraftRenderedRecordFieldDiffReady",
            Self::RenderedPreviewCompared => {
                "StructuredFamilyDraftRenderedRecordRenderedPreviewCompared"
            }
            Self::OriginalRawPreserved => "StructuredFamilyDraftRenderedRecordOriginalRawPreserved",
            Self::RawFallbackPreserved => "StructuredFamilyDraftRenderedRecordRawFallbackPreserved",
            Self::UnsupportedNotProvenYet => {
                "StructuredFamilyDraftRenderedRecordUnsupportedNotProvenYet"
            }
            Self::RenderRereadProofLinked => {
                "StructuredFamilyDraftRenderedRecordRenderRereadProofLinked"
            }
            Self::FixtureOnly => "StructuredFamilyDraftRenderedRecordFixtureOnly",
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

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum StructuredFamilyDraftRenderedRecordApprovalStatus {
    Unavailable,
    ReviewOnly,
    Ready,
    ConfirmationDraftReady,
    ConfirmationAcceptedInMemory,
    ConfirmationRejectedInMemory,
    ConfirmationInvalidated,
    DiffReviewLinked,
    RenderRereadProofLinked,
    ChangedEntriesAcknowledged,
    NoopEntriesAcknowledged,
    RawFallbackAcknowledged,
    UnsupportedNotProvenAcknowledged,
    FixtureOnly,
    ActionsDisabled,
    WritesBlockedByDefault,
    PersistenceForbidden,
    RealConfigTargetForbidden,
    ProductionExecutorForbidden,
}

impl StructuredFamilyDraftRenderedRecordApprovalStatus {
    pub fn as_str(self) -> &'static str {
        match self {
            Self::Unavailable => "StructuredFamilyDraftRenderedRecordApprovalUnavailable",
            Self::ReviewOnly => "StructuredFamilyDraftRenderedRecordApprovalReviewOnly",
            Self::Ready => "StructuredFamilyDraftRenderedRecordApprovalReady",
            Self::ConfirmationDraftReady => {
                "StructuredFamilyDraftRenderedRecordConfirmationDraftReady"
            }
            Self::ConfirmationAcceptedInMemory => {
                "StructuredFamilyDraftRenderedRecordConfirmationAcceptedInMemory"
            }
            Self::ConfirmationRejectedInMemory => {
                "StructuredFamilyDraftRenderedRecordConfirmationRejectedInMemory"
            }
            Self::ConfirmationInvalidated => {
                "StructuredFamilyDraftRenderedRecordConfirmationInvalidated"
            }
            Self::DiffReviewLinked => "StructuredFamilyDraftRenderedRecordDiffReviewLinked",
            Self::RenderRereadProofLinked => {
                "StructuredFamilyDraftRenderedRecordRenderRereadProofLinked"
            }
            Self::ChangedEntriesAcknowledged => {
                "StructuredFamilyDraftRenderedRecordChangedEntriesAcknowledged"
            }
            Self::NoopEntriesAcknowledged => {
                "StructuredFamilyDraftRenderedRecordNoopEntriesAcknowledged"
            }
            Self::RawFallbackAcknowledged => {
                "StructuredFamilyDraftRenderedRecordRawFallbackAcknowledged"
            }
            Self::UnsupportedNotProvenAcknowledged => {
                "StructuredFamilyDraftRenderedRecordUnsupportedNotProvenAcknowledged"
            }
            Self::FixtureOnly => "StructuredFamilyDraftRenderedRecordFixtureOnly",
            Self::ActionsDisabled => "StructuredFamilyDraftRenderedRecordActionsDisabled",
            Self::WritesBlockedByDefault => {
                "StructuredFamilyDraftRenderedRecordWritesBlockedByDefault"
            }
            Self::PersistenceForbidden => "StructuredFamilyDraftRenderedRecordPersistenceForbidden",
            Self::RealConfigTargetForbidden => {
                "StructuredFamilyDraftRenderedRecordRealConfigTargetForbidden"
            }
            Self::ProductionExecutorForbidden => {
                "StructuredFamilyDraftRenderedRecordProductionExecutorForbidden"
            }
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum StructuredFamilyDraftRenderedRecordConfirmationInvalidationReason {
    MismatchedFamily,
    MismatchedSourcePlanCount,
    MismatchedReviewEntryCount,
    MismatchedChangedEntryCount,
    MismatchedUnsupportedNotProvenCount,
    MismatchedRawFallbackCount,
    MissingDiffReviewSummary,
    MissingRenderRereadProofLink,
    RealConfigTargetNotAllowed,
    PersistenceNotAllowed,
    RuntimeMutationNotAllowed,
    HyprlandReloadNotAllowed,
    ProductionExecutorNotAllowed,
    UnsupportedNotProvenRequiresAcknowledgement,
    RawFallbackRequiresAcknowledgement,
}

impl StructuredFamilyDraftRenderedRecordConfirmationInvalidationReason {
    pub fn as_str(self) -> &'static str {
        match self {
            Self::MismatchedFamily => "MismatchedFamily",
            Self::MismatchedSourcePlanCount => "MismatchedSourcePlanCount",
            Self::MismatchedReviewEntryCount => "MismatchedReviewEntryCount",
            Self::MismatchedChangedEntryCount => "MismatchedChangedEntryCount",
            Self::MismatchedUnsupportedNotProvenCount => "MismatchedUnsupportedNotProvenCount",
            Self::MismatchedRawFallbackCount => "MismatchedRawFallbackCount",
            Self::MissingDiffReviewSummary => "MissingDiffReviewSummary",
            Self::MissingRenderRereadProofLink => "MissingRenderRereadProofLink",
            Self::RealConfigTargetNotAllowed => "RealConfigTargetNotAllowed",
            Self::PersistenceNotAllowed => "PersistenceNotAllowed",
            Self::RuntimeMutationNotAllowed => "RuntimeMutationNotAllowed",
            Self::HyprlandReloadNotAllowed => "HyprlandReloadNotAllowed",
            Self::ProductionExecutorNotAllowed => "ProductionExecutorNotAllowed",
            Self::UnsupportedNotProvenRequiresAcknowledgement => {
                "UnsupportedNotProvenRequiresAcknowledgement"
            }
            Self::RawFallbackRequiresAcknowledgement => "RawFallbackRequiresAcknowledgement",
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum StructuredFamilyDraftRenderedRecordStagedApplyStatus {
    Unavailable,
    ReviewOnly,
    Ready,
    PlanReady,
    OperationsReady,
    PreflightReady,
    ConfirmationLinked,
    DiffReviewLinked,
    RenderRereadLinked,
    AcceptedConfirmationRequired,
    RejectedConfirmationBlocked,
    InvalidConfirmationBlocked,
    RawFallbackPreserved,
    UnsupportedNotProvenPreserved,
    RollbackPlanReady,
    DryRunOnly,
    FixtureOnly,
    ActionsDisabled,
    WritesBlockedByDefault,
    PersistenceForbidden,
    RealConfigTargetForbidden,
    ProductionExecutorForbidden,
}

impl StructuredFamilyDraftRenderedRecordStagedApplyStatus {
    pub fn as_str(self) -> &'static str {
        match self {
            Self::Unavailable => "StructuredFamilyDraftRenderedRecordStagedApplyUnavailable",
            Self::ReviewOnly => "StructuredFamilyDraftRenderedRecordStagedApplyReviewOnly",
            Self::Ready => "StructuredFamilyDraftRenderedRecordStagedApplyReady",
            Self::PlanReady => "StructuredFamilyDraftRenderedRecordStagedApplyPlanReady",
            Self::OperationsReady => {
                "StructuredFamilyDraftRenderedRecordStagedApplyOperationsReady"
            }
            Self::PreflightReady => "StructuredFamilyDraftRenderedRecordStagedApplyPreflightReady",
            Self::ConfirmationLinked => {
                "StructuredFamilyDraftRenderedRecordStagedApplyConfirmationLinked"
            }
            Self::DiffReviewLinked => {
                "StructuredFamilyDraftRenderedRecordStagedApplyDiffReviewLinked"
            }
            Self::RenderRereadLinked => {
                "StructuredFamilyDraftRenderedRecordStagedApplyRenderRereadLinked"
            }
            Self::AcceptedConfirmationRequired => {
                "StructuredFamilyDraftRenderedRecordStagedApplyAcceptedConfirmationRequired"
            }
            Self::RejectedConfirmationBlocked => {
                "StructuredFamilyDraftRenderedRecordStagedApplyRejectedConfirmationBlocked"
            }
            Self::InvalidConfirmationBlocked => {
                "StructuredFamilyDraftRenderedRecordStagedApplyInvalidConfirmationBlocked"
            }
            Self::RawFallbackPreserved => {
                "StructuredFamilyDraftRenderedRecordStagedApplyRawFallbackPreserved"
            }
            Self::UnsupportedNotProvenPreserved => {
                "StructuredFamilyDraftRenderedRecordStagedApplyUnsupportedNotProvenPreserved"
            }
            Self::RollbackPlanReady => {
                "StructuredFamilyDraftRenderedRecordStagedApplyRollbackPlanReady"
            }
            Self::DryRunOnly => "StructuredFamilyDraftRenderedRecordStagedApplyDryRunOnly",
            Self::FixtureOnly => "StructuredFamilyDraftRenderedRecordFixtureOnly",
            Self::ActionsDisabled => "StructuredFamilyDraftRenderedRecordActionsDisabled",
            Self::WritesBlockedByDefault => {
                "StructuredFamilyDraftRenderedRecordWritesBlockedByDefault"
            }
            Self::PersistenceForbidden => "StructuredFamilyDraftRenderedRecordPersistenceForbidden",
            Self::RealConfigTargetForbidden => {
                "StructuredFamilyDraftRenderedRecordRealConfigTargetForbidden"
            }
            Self::ProductionExecutorForbidden => {
                "StructuredFamilyDraftRenderedRecordProductionExecutorForbidden"
            }
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum StructuredFamilyDraftRenderedRecordStagedApplyBlocker {
    MissingAcceptedConfirmation,
    RejectedConfirmation,
    InvalidConfirmation,
    MissingDiffReviewSummary,
    MissingRenderRereadProof,
    MissingApprovalAcknowledgement,
    RawFallbackRequiresPreservation,
    UnsupportedNotProvenRequiresPreservation,
    RealConfigTargetNotAllowed,
    PersistenceNotAllowed,
    RuntimeMutationNotAllowed,
    HyprlandReloadNotAllowed,
    ProductionExecutorNotAllowed,
    ExecutorNotAvailableByDesign,
}

impl StructuredFamilyDraftRenderedRecordStagedApplyBlocker {
    pub fn as_str(self) -> &'static str {
        match self {
            Self::MissingAcceptedConfirmation => "MissingAcceptedConfirmation",
            Self::RejectedConfirmation => "RejectedConfirmation",
            Self::InvalidConfirmation => "InvalidConfirmation",
            Self::MissingDiffReviewSummary => "MissingDiffReviewSummary",
            Self::MissingRenderRereadProof => "MissingRenderRereadProof",
            Self::MissingApprovalAcknowledgement => "MissingApprovalAcknowledgement",
            Self::RawFallbackRequiresPreservation => "RawFallbackRequiresPreservation",
            Self::UnsupportedNotProvenRequiresPreservation => {
                "UnsupportedNotProvenRequiresPreservation"
            }
            Self::RealConfigTargetNotAllowed => "RealConfigTargetNotAllowed",
            Self::PersistenceNotAllowed => "PersistenceNotAllowed",
            Self::RuntimeMutationNotAllowed => "RuntimeMutationNotAllowed",
            Self::HyprlandReloadNotAllowed => "HyprlandReloadNotAllowed",
            Self::ProductionExecutorNotAllowed => "ProductionExecutorNotAllowed",
            Self::ExecutorNotAvailableByDesign => "ExecutorNotAvailableByDesign",
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum StructuredFamilyDraftRenderedRecordStagedApplyDryRunStatus {
    Unavailable,
    ReviewOnly,
    Ready,
    ReportReady,
    PlanLinked,
    StagesSummarized,
    OperationsSummarized,
    ChangedOperationsSummarized,
    NoopOperationsSummarized,
    RawFallbackPreserved,
    UnsupportedNotProvenPreserved,
    BlockedPlanSummarized,
    ExecutorUnavailable,
    NotExecuted,
    FixtureOnly,
    ActionsDisabled,
    WritesBlockedByDefault,
    PersistenceForbidden,
    RealConfigTargetForbidden,
    ProductionExecutorForbidden,
}

impl StructuredFamilyDraftRenderedRecordStagedApplyDryRunStatus {
    pub fn as_str(self) -> &'static str {
        match self {
            Self::Unavailable => "StructuredFamilyDraftRenderedRecordStagedApplyDryRunUnavailable",
            Self::ReviewOnly => "StructuredFamilyDraftRenderedRecordStagedApplyDryRunReviewOnly",
            Self::Ready => "StructuredFamilyDraftRenderedRecordStagedApplyDryRunReady",
            Self::ReportReady => "StructuredFamilyDraftRenderedRecordStagedApplyDryRunReportReady",
            Self::PlanLinked => "StructuredFamilyDraftRenderedRecordStagedApplyDryRunPlanLinked",
            Self::StagesSummarized => {
                "StructuredFamilyDraftRenderedRecordStagedApplyDryRunStagesSummarized"
            }
            Self::OperationsSummarized => {
                "StructuredFamilyDraftRenderedRecordStagedApplyDryRunOperationsSummarized"
            }
            Self::ChangedOperationsSummarized => {
                "StructuredFamilyDraftRenderedRecordStagedApplyDryRunChangedOperationsSummarized"
            }
            Self::NoopOperationsSummarized => {
                "StructuredFamilyDraftRenderedRecordStagedApplyDryRunNoopOperationsSummarized"
            }
            Self::RawFallbackPreserved => {
                "StructuredFamilyDraftRenderedRecordStagedApplyDryRunRawFallbackPreserved"
            }
            Self::UnsupportedNotProvenPreserved => {
                "StructuredFamilyDraftRenderedRecordStagedApplyDryRunUnsupportedNotProvenPreserved"
            }
            Self::BlockedPlanSummarized => {
                "StructuredFamilyDraftRenderedRecordStagedApplyDryRunBlockedPlanSummarized"
            }
            Self::ExecutorUnavailable => {
                "StructuredFamilyDraftRenderedRecordStagedApplyDryRunExecutorUnavailable"
            }
            Self::NotExecuted => "StructuredFamilyDraftRenderedRecordStagedApplyDryRunNotExecuted",
            Self::FixtureOnly => "StructuredFamilyDraftRenderedRecordFixtureOnly",
            Self::ActionsDisabled => "StructuredFamilyDraftRenderedRecordActionsDisabled",
            Self::WritesBlockedByDefault => {
                "StructuredFamilyDraftRenderedRecordWritesBlockedByDefault"
            }
            Self::PersistenceForbidden => "StructuredFamilyDraftRenderedRecordPersistenceForbidden",
            Self::RealConfigTargetForbidden => {
                "StructuredFamilyDraftRenderedRecordRealConfigTargetForbidden"
            }
            Self::ProductionExecutorForbidden => {
                "StructuredFamilyDraftRenderedRecordProductionExecutorForbidden"
            }
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum StructuredFamilyDraftRenderedRecordStagedApplyRollbackRecoveryStatus {
    Unavailable,
    ReviewOnly,
    Ready,
    ReviewReady,
    DryRunLinked,
    PlanLinked,
    RollbackPlanSummarized,
    RecoveryPathSummarized,
    BackupRequirementReady,
    RestoreRequirementReady,
    BlockedPlanPreserved,
    ExecutorUnavailable,
    NotExecuted,
    FixtureOnly,
    ActionsDisabled,
    WritesBlockedByDefault,
    PersistenceForbidden,
    RealConfigTargetForbidden,
    ProductionExecutorForbidden,
}

impl StructuredFamilyDraftRenderedRecordStagedApplyRollbackRecoveryStatus {
    pub fn as_str(self) -> &'static str {
        match self {
            Self::Unavailable => {
                "StructuredFamilyDraftRenderedRecordStagedApplyRollbackRecoveryUnavailable"
            }
            Self::ReviewOnly => {
                "StructuredFamilyDraftRenderedRecordStagedApplyRollbackRecoveryReviewOnly"
            }
            Self::Ready => "StructuredFamilyDraftRenderedRecordStagedApplyRollbackRecoveryReady",
            Self::ReviewReady => {
                "StructuredFamilyDraftRenderedRecordStagedApplyRollbackRecoveryReviewReady"
            }
            Self::DryRunLinked => {
                "StructuredFamilyDraftRenderedRecordStagedApplyRollbackRecoveryDryRunLinked"
            }
            Self::PlanLinked => {
                "StructuredFamilyDraftRenderedRecordStagedApplyRollbackRecoveryPlanLinked"
            }
            Self::RollbackPlanSummarized => {
                "StructuredFamilyDraftRenderedRecordStagedApplyRollbackRecoveryRollbackPlanSummarized"
            }
            Self::RecoveryPathSummarized => {
                "StructuredFamilyDraftRenderedRecordStagedApplyRollbackRecoveryRecoveryPathSummarized"
            }
            Self::BackupRequirementReady => {
                "StructuredFamilyDraftRenderedRecordStagedApplyRollbackRecoveryBackupRequirementReady"
            }
            Self::RestoreRequirementReady => {
                "StructuredFamilyDraftRenderedRecordStagedApplyRollbackRecoveryRestoreRequirementReady"
            }
            Self::BlockedPlanPreserved => {
                "StructuredFamilyDraftRenderedRecordStagedApplyRollbackRecoveryBlockedPlanPreserved"
            }
            Self::ExecutorUnavailable => {
                "StructuredFamilyDraftRenderedRecordStagedApplyRollbackRecoveryExecutorUnavailable"
            }
            Self::NotExecuted => {
                "StructuredFamilyDraftRenderedRecordStagedApplyRollbackRecoveryNotExecuted"
            }
            Self::FixtureOnly => "StructuredFamilyDraftRenderedRecordFixtureOnly",
            Self::ActionsDisabled => "StructuredFamilyDraftRenderedRecordActionsDisabled",
            Self::WritesBlockedByDefault => {
                "StructuredFamilyDraftRenderedRecordWritesBlockedByDefault"
            }
            Self::PersistenceForbidden => "StructuredFamilyDraftRenderedRecordPersistenceForbidden",
            Self::RealConfigTargetForbidden => {
                "StructuredFamilyDraftRenderedRecordRealConfigTargetForbidden"
            }
            Self::ProductionExecutorForbidden => {
                "StructuredFamilyDraftRenderedRecordProductionExecutorForbidden"
            }
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum StructuredFamilyDraftRenderedRecordRollbackRecoveryRequirement {
    BackupRequiredBeforeFutureApply,
    RestoreRequiredBeforeFutureRecovery,
    ReloadForbiddenInCurrentSprint,
    RuntimeMutationForbiddenInCurrentSprint,
    RealConfigTargetForbiddenInCurrentSprint,
    ProductionExecutorForbiddenInCurrentSprint,
    FixtureOnlyReviewRequired,
    UnsupportedNotProvenRequiresPreservation,
    RawFallbackRequiresPreservation,
    DryRunMustRemainNotExecuted,
    StagedApplyMustRemainNotExecuted,
}

impl StructuredFamilyDraftRenderedRecordRollbackRecoveryRequirement {
    pub fn as_str(self) -> &'static str {
        match self {
            Self::BackupRequiredBeforeFutureApply => "BackupRequiredBeforeFutureApply",
            Self::RestoreRequiredBeforeFutureRecovery => "RestoreRequiredBeforeFutureRecovery",
            Self::ReloadForbiddenInCurrentSprint => "ReloadForbiddenInCurrentSprint",
            Self::RuntimeMutationForbiddenInCurrentSprint => {
                "RuntimeMutationForbiddenInCurrentSprint"
            }
            Self::RealConfigTargetForbiddenInCurrentSprint => {
                "RealConfigTargetForbiddenInCurrentSprint"
            }
            Self::ProductionExecutorForbiddenInCurrentSprint => {
                "ProductionExecutorForbiddenInCurrentSprint"
            }
            Self::FixtureOnlyReviewRequired => "FixtureOnlyReviewRequired",
            Self::UnsupportedNotProvenRequiresPreservation => {
                "UnsupportedNotProvenRequiresPreservation"
            }
            Self::RawFallbackRequiresPreservation => "RawFallbackRequiresPreservation",
            Self::DryRunMustRemainNotExecuted => "DryRunMustRemainNotExecuted",
            Self::StagedApplyMustRemainNotExecuted => "StagedApplyMustRemainNotExecuted",
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum StructuredFamilyDraftRenderedRecordRollbackRecoveryBlocker {
    MissingDryRunReportLink,
    MissingStagedApplyPlanLink,
    BlockedStagedApplyPlan,
    RealConfigTargetNotAllowed,
    PersistenceNotAllowed,
    RuntimeMutationNotAllowed,
    HyprlandReloadNotAllowed,
    ProductionExecutorNotAllowed,
    StagedApplyAlreadyExecuted,
    DryRunAlreadyExecuted,
    BackupAlreadyCreated,
    RestoreAlreadyExecuted,
}

impl StructuredFamilyDraftRenderedRecordRollbackRecoveryBlocker {
    pub fn as_str(self) -> &'static str {
        match self {
            Self::MissingDryRunReportLink => "MissingDryRunReportLink",
            Self::MissingStagedApplyPlanLink => "MissingStagedApplyPlanLink",
            Self::BlockedStagedApplyPlan => "BlockedStagedApplyPlan",
            Self::RealConfigTargetNotAllowed => "RealConfigTargetNotAllowed",
            Self::PersistenceNotAllowed => "PersistenceNotAllowed",
            Self::RuntimeMutationNotAllowed => "RuntimeMutationNotAllowed",
            Self::HyprlandReloadNotAllowed => "HyprlandReloadNotAllowed",
            Self::ProductionExecutorNotAllowed => "ProductionExecutorNotAllowed",
            Self::StagedApplyAlreadyExecuted => "StagedApplyAlreadyExecuted",
            Self::DryRunAlreadyExecuted => "DryRunAlreadyExecuted",
            Self::BackupAlreadyCreated => "BackupAlreadyCreated",
            Self::RestoreAlreadyExecuted => "RestoreAlreadyExecuted",
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum StructuredFamilyDraftRenderedRecordFinalExecutorReadinessStatus {
    Unavailable,
    ReviewOnly,
    Ready,
    AuditReady,
    RollbackRecoveryLinked,
    DryRunLinked,
    StagedApplyLinked,
    ProofChainComplete,
    FixtureOnlyComplete,
    ProductionActivationRequired,
    ExecutorNotImplemented,
    ExecutorNotWired,
    RealWritesBlocked,
    RuntimeMutationBlocked,
    ReloadBlocked,
    BackupRestoreNotImplemented,
    BlockedPlanPreserved,
    NotProductionReady,
    FixtureOnly,
    ActionsDisabled,
    WritesBlockedByDefault,
    PersistenceForbidden,
    RealConfigTargetForbidden,
    ProductionExecutorForbidden,
}

impl StructuredFamilyDraftRenderedRecordFinalExecutorReadinessStatus {
    pub fn as_str(self) -> &'static str {
        match self {
            Self::Unavailable => "StructuredFamilyDraftRenderedRecordFinalExecutorReadinessUnavailable",
            Self::ReviewOnly => "StructuredFamilyDraftRenderedRecordFinalExecutorReadinessReviewOnly",
            Self::Ready => "StructuredFamilyDraftRenderedRecordFinalExecutorReadinessReady",
            Self::AuditReady => "StructuredFamilyDraftRenderedRecordFinalExecutorReadinessAuditReady",
            Self::RollbackRecoveryLinked => {
                "StructuredFamilyDraftRenderedRecordFinalExecutorReadinessRollbackRecoveryLinked"
            }
            Self::DryRunLinked => {
                "StructuredFamilyDraftRenderedRecordFinalExecutorReadinessDryRunLinked"
            }
            Self::StagedApplyLinked => {
                "StructuredFamilyDraftRenderedRecordFinalExecutorReadinessStagedApplyLinked"
            }
            Self::ProofChainComplete => {
                "StructuredFamilyDraftRenderedRecordFinalExecutorReadinessProofChainComplete"
            }
            Self::FixtureOnlyComplete => {
                "StructuredFamilyDraftRenderedRecordFinalExecutorReadinessFixtureOnlyComplete"
            }
            Self::ProductionActivationRequired => {
                "StructuredFamilyDraftRenderedRecordFinalExecutorReadinessProductionActivationRequired"
            }
            Self::ExecutorNotImplemented => {
                "StructuredFamilyDraftRenderedRecordFinalExecutorReadinessExecutorNotImplemented"
            }
            Self::ExecutorNotWired => {
                "StructuredFamilyDraftRenderedRecordFinalExecutorReadinessExecutorNotWired"
            }
            Self::RealWritesBlocked => {
                "StructuredFamilyDraftRenderedRecordFinalExecutorReadinessRealWritesBlocked"
            }
            Self::RuntimeMutationBlocked => {
                "StructuredFamilyDraftRenderedRecordFinalExecutorReadinessRuntimeMutationBlocked"
            }
            Self::ReloadBlocked => {
                "StructuredFamilyDraftRenderedRecordFinalExecutorReadinessReloadBlocked"
            }
            Self::BackupRestoreNotImplemented => {
                "StructuredFamilyDraftRenderedRecordFinalExecutorReadinessBackupRestoreNotImplemented"
            }
            Self::BlockedPlanPreserved => {
                "StructuredFamilyDraftRenderedRecordFinalExecutorReadinessBlockedPlanPreserved"
            }
            Self::NotProductionReady => {
                "StructuredFamilyDraftRenderedRecordFinalExecutorReadinessNotProductionReady"
            }
            Self::FixtureOnly => "StructuredFamilyDraftRenderedRecordFixtureOnly",
            Self::ActionsDisabled => "StructuredFamilyDraftRenderedRecordActionsDisabled",
            Self::WritesBlockedByDefault => {
                "StructuredFamilyDraftRenderedRecordWritesBlockedByDefault"
            }
            Self::PersistenceForbidden => "StructuredFamilyDraftRenderedRecordPersistenceForbidden",
            Self::RealConfigTargetForbidden => {
                "StructuredFamilyDraftRenderedRecordRealConfigTargetForbidden"
            }
            Self::ProductionExecutorForbidden => {
                "StructuredFamilyDraftRenderedRecordProductionExecutorForbidden"
            }
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum StructuredFamilyDraftRenderedRecordFinalExecutorReadinessFinding {
    FixturePipelineComplete,
    ProductionActivationRequired,
    ExecutorNotImplemented,
    ExecutorNotWired,
    RealWritesBlocked,
    PersistenceBlocked,
    RuntimeMutationBlocked,
    HyprlandReloadBlocked,
    BackupImplementationMissing,
    RestoreImplementationMissing,
    RollbackExecutionMissing,
    RecoveryExecutionMissing,
    SourceTargetPolicyStillForbidden,
    UnsupportedNotProvenPreservationRequired,
    RawFallbackPreservationRequired,
    UserDecisionRequiredBeforeProduction,
    Hyprland0554MigrationNotActive,
    BlockedPlanPreserved,
}

impl StructuredFamilyDraftRenderedRecordFinalExecutorReadinessFinding {
    pub fn as_str(self) -> &'static str {
        match self {
            Self::FixturePipelineComplete => "FixturePipelineComplete",
            Self::ProductionActivationRequired => "ProductionActivationRequired",
            Self::ExecutorNotImplemented => "ExecutorNotImplemented",
            Self::ExecutorNotWired => "ExecutorNotWired",
            Self::RealWritesBlocked => "RealWritesBlocked",
            Self::PersistenceBlocked => "PersistenceBlocked",
            Self::RuntimeMutationBlocked => "RuntimeMutationBlocked",
            Self::HyprlandReloadBlocked => "HyprlandReloadBlocked",
            Self::BackupImplementationMissing => "BackupImplementationMissing",
            Self::RestoreImplementationMissing => "RestoreImplementationMissing",
            Self::RollbackExecutionMissing => "RollbackExecutionMissing",
            Self::RecoveryExecutionMissing => "RecoveryExecutionMissing",
            Self::SourceTargetPolicyStillForbidden => "SourceTargetPolicyStillForbidden",
            Self::UnsupportedNotProvenPreservationRequired => {
                "UnsupportedNotProvenPreservationRequired"
            }
            Self::RawFallbackPreservationRequired => "RawFallbackPreservationRequired",
            Self::UserDecisionRequiredBeforeProduction => "UserDecisionRequiredBeforeProduction",
            Self::Hyprland0554MigrationNotActive => "Hyprland0554MigrationNotActive",
            Self::BlockedPlanPreserved => "BlockedPlanPreserved",
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum StructuredFamilyDraftRenderedRecordFinalExecutorReadinessBlocker {
    MissingRollbackRecoveryLink,
    MissingDryRunLink,
    MissingStagedApplyPlanLink,
    BlockedStagedApplyPlan,
    RealConfigTargetNotAllowed,
    PersistenceNotAllowed,
    RuntimeMutationNotAllowed,
    HyprlandReloadNotAllowed,
    ProductionExecutorNotAllowed,
    ProductionActivationNotAllowed,
    ExecutorImplementationNotAllowed,
    ExecutorWiringNotAllowed,
}

impl StructuredFamilyDraftRenderedRecordFinalExecutorReadinessBlocker {
    pub fn as_str(self) -> &'static str {
        match self {
            Self::MissingRollbackRecoveryLink => "MissingRollbackRecoveryLink",
            Self::MissingDryRunLink => "MissingDryRunLink",
            Self::MissingStagedApplyPlanLink => "MissingStagedApplyPlanLink",
            Self::BlockedStagedApplyPlan => "BlockedStagedApplyPlan",
            Self::RealConfigTargetNotAllowed => "RealConfigTargetNotAllowed",
            Self::PersistenceNotAllowed => "PersistenceNotAllowed",
            Self::RuntimeMutationNotAllowed => "RuntimeMutationNotAllowed",
            Self::HyprlandReloadNotAllowed => "HyprlandReloadNotAllowed",
            Self::ProductionExecutorNotAllowed => "ProductionExecutorNotAllowed",
            Self::ProductionActivationNotAllowed => "ProductionActivationNotAllowed",
            Self::ExecutorImplementationNotAllowed => "ExecutorImplementationNotAllowed",
            Self::ExecutorWiringNotAllowed => "ExecutorWiringNotAllowed",
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum StructuredFamilyDraftRenderedRecordFinalExecutorReadinessDecision {
    NotProductionReady,
}

impl StructuredFamilyDraftRenderedRecordFinalExecutorReadinessDecision {
    pub fn as_str(self) -> &'static str {
        match self {
            Self::NotProductionReady => {
                "StructuredFamilyDraftRenderedRecordFinalExecutorReadinessNotProductionReady"
            }
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum StructuredFamilyRealWriteActivationAuditStatus {
    RequirementsAuditReady,
    RequirementsListed,
    BackupRestoreProofMissing,
    UserApprovalGatesRequired,
    ActivationNotApproved,
    ExecutorNotImplemented,
    ExecutorNotWired,
    BlockedByDefault,
    RealConfigTargetForbidden,
    RuntimeMutationForbidden,
    HyprlandReloadForbidden,
    ProductionActivationDecisionRequired,
    FamilyRankingExcludedByUser,
}

impl StructuredFamilyRealWriteActivationAuditStatus {
    pub fn as_str(self) -> &'static str {
        match self {
            Self::RequirementsAuditReady => {
                "StructuredFamilyRealWriteActivationRequirementsAuditReady"
            }
            Self::RequirementsListed => "StructuredFamilyRealWriteActivationRequirementsListed",
            Self::BackupRestoreProofMissing => "StructuredFamilyBackupRestoreProofMissing",
            Self::UserApprovalGatesRequired => "StructuredFamilyUserApprovalGatesRequired",
            Self::ActivationNotApproved => "StructuredFamilyRealWriteActivationNotApproved",
            Self::ExecutorNotImplemented => "StructuredFamilyRealWriteExecutorNotImplemented",
            Self::ExecutorNotWired => "StructuredFamilyRealWriteExecutorNotWired",
            Self::BlockedByDefault => "StructuredFamilyRealWriteBlockedByDefault",
            Self::RealConfigTargetForbidden => "StructuredFamilyRealConfigTargetForbidden",
            Self::RuntimeMutationForbidden => "StructuredFamilyRuntimeMutationForbidden",
            Self::HyprlandReloadForbidden => "StructuredFamilyHyprlandReloadForbidden",
            Self::ProductionActivationDecisionRequired => {
                "StructuredFamilyProductionActivationDecisionRequired"
            }
            Self::FamilyRankingExcludedByUser => "StructuredFamilyFamilyRankingExcludedByUser",
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct StructuredFamilyRealWriteActivationRequirementsAudit {
    pub user_instruction: &'static str,
    pub excluded_by_user: Vec<&'static str>,
    pub real_write_activation_requirements: Vec<&'static str>,
    pub missing_backup_restore_proof: Vec<&'static str>,
    pub required_user_approval_gates: Vec<&'static str>,
    pub activation_status: StructuredFamilyRealWriteActivationAuditStatus,
    pub executor_status: StructuredFamilyRealWriteActivationAuditStatus,
    pub real_write_boundary_status: StructuredFamilyRealWriteActivationAuditStatus,
    pub backup_restore_boundary_status: StructuredFamilyRealWriteActivationAuditStatus,
    pub reload_boundary_status: StructuredFamilyRealWriteActivationAuditStatus,
    pub runtime_mutation_boundary_status: StructuredFamilyRealWriteActivationAuditStatus,
    pub family_ranking_excluded: StructuredFamilyRealWriteActivationAuditStatus,
    pub production_activation_approved: bool,
    pub executor_implemented: bool,
    pub executor_wired: bool,
    pub real_write_path_enabled: bool,
    pub real_config_target_enabled: bool,
    pub backup_creation_enabled: bool,
    pub restore_execution_enabled: bool,
    pub rollback_execution_enabled: bool,
    pub hyprctl_reload_enabled: bool,
    pub runtime_mutation_enabled: bool,
    pub first_real_config_write_approved: bool,
    pub next_recommended_work: &'static str,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct StructuredFamilyProductionActivationPlanningScope {
    pub user_decision: &'static str,
    pub planning_scope_approved: bool,
    pub implementation_scope_approved: bool,
    pub real_write_scope_approved: bool,
    pub excluded_by_user: Vec<&'static str>,
    pub approved_planning_scope: Vec<&'static str>,
    pub not_approved_scope: Vec<&'static str>,
    pub executor_architecture_planning_requirements: Vec<&'static str>,
    pub backup_restore_planning_requirements: Vec<&'static str>,
    pub rollback_recovery_planning_requirements: Vec<&'static str>,
    pub validation_planning_requirements: Vec<&'static str>,
    pub manual_approval_checkpoints: Vec<&'static str>,
    pub future_implementation_stop_gates: Vec<&'static str>,
    pub production_activation_approved: bool,
    pub executor_implemented: bool,
    pub executor_wired: bool,
    pub real_write_path_enabled: bool,
    pub real_config_target_enabled: bool,
    pub backup_creation_enabled: bool,
    pub restore_execution_enabled: bool,
    pub rollback_execution_enabled: bool,
    pub hyprctl_reload_enabled: bool,
    pub runtime_mutation_enabled: bool,
    pub first_real_config_write_approved: bool,
    pub family_ranking_excluded: bool,
    pub activation_subset_selected: bool,
    pub production_readiness_decision: &'static str,
    pub next_recommended_work: &'static str,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum StructuredFamilyDraftRenderedRecordApplyStageKind {
    Preflight,
    Review,
    RenderPreview,
    RawFallbackPreservation,
    UnsupportedNotProvenPreservation,
    DryRunOnlyApply,
    RollbackPlan,
}

impl StructuredFamilyDraftRenderedRecordApplyStageKind {
    pub fn as_str(self) -> &'static str {
        match self {
            Self::Preflight => "preflight",
            Self::Review => "review",
            Self::RenderPreview => "render-preview",
            Self::RawFallbackPreservation => "raw-fallback-preservation",
            Self::UnsupportedNotProvenPreservation => "unsupported-not-proven-preservation",
            Self::DryRunOnlyApply => "dry-run-only-apply",
            Self::RollbackPlan => "rollback-plan",
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

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct StructuredFamilyDraftRenderedRecordRenderRereadProof {
    pub family: StructuredFamilyKind,
    pub source_draft_count: usize,
    pub source_plan_count: usize,
    pub rendered_fixture_path: PathBuf,
    pub rendered_fixture_text: String,
    pub reread_projection_family: StructuredFamilyKind,
    pub reread_record_count: usize,
    pub field_map_count: usize,
    pub raw_fallback_plan_count: usize,
    pub unsupported_not_proven_plan_count: usize,
    pub family_preserved: bool,
    pub record_count_preserved: bool,
    pub field_map_preserved: bool,
    pub raw_fallback_preserved: bool,
    pub unsupported_not_proven_preserved: bool,
    pub render_reread_status: StructuredFamilyDraftRenderedRecordRenderRereadStatus,
    pub rendered_temp_fixture_status: StructuredFamilyDraftRenderedRecordRenderRereadStatus,
    pub reread_status: StructuredFamilyDraftRenderedRecordRenderRereadStatus,
    pub family_preservation_status: StructuredFamilyDraftRenderedRecordRenderRereadStatus,
    pub record_count_preservation_status: StructuredFamilyDraftRenderedRecordRenderRereadStatus,
    pub field_map_preservation_status: StructuredFamilyDraftRenderedRecordRenderRereadStatus,
    pub raw_fallback_preservation_status: StructuredFamilyDraftRenderedRecordRenderRereadStatus,
    pub unsupported_not_proven_preservation_status:
        StructuredFamilyDraftRenderedRecordRenderRereadStatus,
    pub fixture_only_status: StructuredFamilyDraftRenderedRecordRenderRereadStatus,
    pub action_policy: StructuredFamilyDraftRenderedRecordRenderRereadStatus,
    pub write_policy: StructuredFamilyDraftRenderedRecordRenderRereadStatus,
    pub persistence_policy: StructuredFamilyDraftRenderedRecordRenderRereadStatus,
    pub real_config_target_policy: StructuredFamilyDraftRenderedRecordRenderRereadStatus,
    pub draft_written_to_disk: bool,
    pub rendered_record_written_to_temp_fixture: bool,
    pub rendered_record_written_to_real_config: bool,
    pub real_config_touched: bool,
    pub runtime_mutated: bool,
    pub hyprctl_reload_run: bool,
    pub production_executor_wired: bool,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct StructuredFamilyDraftRenderedRecordFieldDiff {
    pub field_name: String,
    pub original_value: String,
    pub draft_value: String,
    pub rendered_part: String,
    pub changed: bool,
    pub status: StructuredFamilyDraftRenderedRecordDiffReviewStatus,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct StructuredFamilyDraftRenderedRecordDiffReviewEntry {
    pub family: StructuredFamilyKind,
    pub record_index: usize,
    pub source_path: PathBuf,
    pub line_number: usize,
    pub original_raw_line: String,
    pub rendered_record_preview: String,
    pub field_diffs: Vec<StructuredFamilyDraftRenderedRecordFieldDiff>,
    pub changed: bool,
    pub diff_status: StructuredFamilyDraftRenderedRecordDiffReviewStatus,
    pub field_diff_status: StructuredFamilyDraftRenderedRecordDiffReviewStatus,
    pub rendered_preview_compared_status: StructuredFamilyDraftRenderedRecordDiffReviewStatus,
    pub original_raw_preserved_status: StructuredFamilyDraftRenderedRecordDiffReviewStatus,
    pub raw_fallback_status: StructuredFamilyDraftRenderedRecordDiffReviewStatus,
    pub unsupported_not_proven_status: StructuredFamilyDraftRenderedRecordDiffReviewStatus,
    pub not_safe_for_full_synthesis: bool,
    pub summary_text: String,
    pub risk_summary: String,
    pub review_decision_status: StructuredFamilyDraftRenderedRecordDiffReviewStatus,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct StructuredFamilyDraftRenderedRecordDiffReviewSummary {
    pub family: StructuredFamilyKind,
    pub source_draft_count: usize,
    pub source_plan_count: usize,
    pub render_reread_proof_status: StructuredFamilyDraftRenderedRecordDiffReviewStatus,
    pub review_entry_count: usize,
    pub changed_entry_count: usize,
    pub noop_entry_count: usize,
    pub raw_fallback_entry_count: usize,
    pub unsupported_not_proven_entry_count: usize,
    pub field_diff_count: usize,
    pub changed_field_diff_count: usize,
    pub entries: Vec<StructuredFamilyDraftRenderedRecordDiffReviewEntry>,
    pub summary_text: String,
    pub risk_summary: String,
    pub review_decision_status: StructuredFamilyDraftRenderedRecordDiffReviewStatus,
    pub diff_review_status: StructuredFamilyDraftRenderedRecordDiffReviewStatus,
    pub review_summary_status: StructuredFamilyDraftRenderedRecordDiffReviewStatus,
    pub field_diff_status: StructuredFamilyDraftRenderedRecordDiffReviewStatus,
    pub changed_entry_status: StructuredFamilyDraftRenderedRecordDiffReviewStatus,
    pub noop_entry_status: StructuredFamilyDraftRenderedRecordDiffReviewStatus,
    pub raw_fallback_review_status: StructuredFamilyDraftRenderedRecordDiffReviewStatus,
    pub unsupported_not_proven_review_status: StructuredFamilyDraftRenderedRecordDiffReviewStatus,
    pub fixture_only_status: StructuredFamilyDraftRenderedRecordDiffReviewStatus,
    pub action_policy: StructuredFamilyDraftRenderedRecordDiffReviewStatus,
    pub write_policy: StructuredFamilyDraftRenderedRecordDiffReviewStatus,
    pub persistence_policy: StructuredFamilyDraftRenderedRecordDiffReviewStatus,
    pub real_config_target_policy: StructuredFamilyDraftRenderedRecordDiffReviewStatus,
    pub draft_written_to_disk: bool,
    pub diff_summary_written_to_disk: bool,
    pub rendered_record_written_to_temp_fixture: bool,
    pub rendered_record_written_to_real_config: bool,
    pub real_config_touched: bool,
    pub runtime_mutated: bool,
    pub hyprctl_reload_run: bool,
    pub production_executor_wired: bool,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct StructuredFamilyDraftRenderedRecordApprovalDraft {
    pub family: StructuredFamilyKind,
    pub source_draft_count: usize,
    pub source_plan_count: usize,
    pub review_entry_count: usize,
    pub changed_entry_count: usize,
    pub noop_entry_count: usize,
    pub raw_fallback_entry_count: usize,
    pub unsupported_not_proven_entry_count: usize,
    pub field_diff_count: usize,
    pub summary_text: String,
    pub risk_summary: String,
    pub diff_review_summary_linked: bool,
    pub render_reread_proof_linked: bool,
    pub changed_entries_acknowledged: bool,
    pub noop_entries_acknowledged: bool,
    pub raw_fallback_acknowledged: bool,
    pub unsupported_not_proven_acknowledged: bool,
    pub approval_status: StructuredFamilyDraftRenderedRecordApprovalStatus,
    pub confirmation_status: StructuredFamilyDraftRenderedRecordApprovalStatus,
    pub fixture_only_status: StructuredFamilyDraftRenderedRecordApprovalStatus,
    pub action_policy: StructuredFamilyDraftRenderedRecordApprovalStatus,
    pub write_policy: StructuredFamilyDraftRenderedRecordApprovalStatus,
    pub persistence_policy: StructuredFamilyDraftRenderedRecordApprovalStatus,
    pub real_config_target_policy: StructuredFamilyDraftRenderedRecordApprovalStatus,
    pub production_executor_policy: StructuredFamilyDraftRenderedRecordApprovalStatus,
    pub draft_written_to_disk: bool,
    pub approval_written_to_disk: bool,
    pub confirmation_written_to_disk: bool,
    pub rendered_record_written_to_real_config: bool,
    pub real_config_touched: bool,
    pub runtime_mutated: bool,
    pub hyprctl_reload_run: bool,
    pub production_executor_wired: bool,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct StructuredFamilyDraftRenderedRecordConfirmationRequest {
    pub family: StructuredFamilyKind,
    pub source_plan_count: usize,
    pub review_entry_count: usize,
    pub changed_entry_count: usize,
    pub raw_fallback_entry_count: usize,
    pub unsupported_not_proven_entry_count: usize,
    pub diff_review_summary_linked: bool,
    pub render_reread_proof_linked: bool,
    pub changed_entries_acknowledged: bool,
    pub noop_entries_acknowledged: bool,
    pub raw_fallback_acknowledged: bool,
    pub unsupported_not_proven_acknowledged: bool,
    pub real_config_target_forbidden: bool,
    pub persistence_forbidden: bool,
    pub runtime_mutation_forbidden: bool,
    pub hyprland_reload_forbidden: bool,
    pub production_executor_forbidden: bool,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct StructuredFamilyDraftRenderedRecordConfirmation {
    pub family: StructuredFamilyKind,
    pub source_draft_count: usize,
    pub source_plan_count: usize,
    pub review_entry_count: usize,
    pub changed_entry_count: usize,
    pub noop_entry_count: usize,
    pub raw_fallback_entry_count: usize,
    pub unsupported_not_proven_entry_count: usize,
    pub field_diff_count: usize,
    pub summary_text: String,
    pub risk_summary: String,
    pub diff_review_summary_linked: bool,
    pub render_reread_proof_linked: bool,
    pub changed_entries_acknowledged: bool,
    pub noop_entries_acknowledged: bool,
    pub raw_fallback_acknowledged: bool,
    pub unsupported_not_proven_acknowledged: bool,
    pub approval_status: StructuredFamilyDraftRenderedRecordApprovalStatus,
    pub confirmation_status: StructuredFamilyDraftRenderedRecordApprovalStatus,
    pub confirmation_accepted_in_memory: bool,
    pub confirmation_rejected_in_memory: bool,
    pub confirmation_invalidation_reasons:
        Vec<StructuredFamilyDraftRenderedRecordConfirmationInvalidationReason>,
    pub fixture_only_status: StructuredFamilyDraftRenderedRecordApprovalStatus,
    pub action_policy: StructuredFamilyDraftRenderedRecordApprovalStatus,
    pub write_policy: StructuredFamilyDraftRenderedRecordApprovalStatus,
    pub persistence_policy: StructuredFamilyDraftRenderedRecordApprovalStatus,
    pub real_config_target_policy: StructuredFamilyDraftRenderedRecordApprovalStatus,
    pub production_executor_policy: StructuredFamilyDraftRenderedRecordApprovalStatus,
    pub draft_written_to_disk: bool,
    pub approval_written_to_disk: bool,
    pub confirmation_written_to_disk: bool,
    pub rendered_record_written_to_real_config: bool,
    pub real_config_touched: bool,
    pub runtime_mutated: bool,
    pub hyprctl_reload_run: bool,
    pub production_executor_wired: bool,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct StructuredFamilyDraftRenderedRecordStagedApplyOperation {
    pub family: StructuredFamilyKind,
    pub record_index: usize,
    pub stage_kind: StructuredFamilyDraftRenderedRecordApplyStageKind,
    pub operation_kind: String,
    pub original_raw_line: String,
    pub rendered_record_preview: String,
    pub field_diff_count: usize,
    pub status: StructuredFamilyDraftRenderedRecordStagedApplyStatus,
    pub action_policy: StructuredFamilyDraftRenderedRecordStagedApplyStatus,
    pub write_policy: StructuredFamilyDraftRenderedRecordStagedApplyStatus,
    pub persistence_policy: StructuredFamilyDraftRenderedRecordStagedApplyStatus,
    pub real_config_target_policy: StructuredFamilyDraftRenderedRecordStagedApplyStatus,
    pub production_executor_policy: StructuredFamilyDraftRenderedRecordStagedApplyStatus,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct StructuredFamilyDraftRenderedRecordStagedApplyStage {
    pub family: StructuredFamilyKind,
    pub stage_kind: StructuredFamilyDraftRenderedRecordApplyStageKind,
    pub status: StructuredFamilyDraftRenderedRecordStagedApplyStatus,
    pub operation_count: usize,
    pub summary_text: String,
    pub risk_summary: String,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct StructuredFamilyDraftRenderedRecordStagedApplyPlan {
    pub family: StructuredFamilyKind,
    pub source_draft_count: usize,
    pub source_plan_count: usize,
    pub review_entry_count: usize,
    pub changed_entry_count: usize,
    pub noop_entry_count: usize,
    pub raw_fallback_entry_count: usize,
    pub unsupported_not_proven_entry_count: usize,
    pub field_diff_count: usize,
    pub accepted_confirmation_linked: bool,
    pub diff_review_summary_linked: bool,
    pub render_reread_proof_linked: bool,
    pub stage_count: usize,
    pub operation_count: usize,
    pub changed_operation_count: usize,
    pub noop_operation_count: usize,
    pub raw_fallback_preservation_operation_count: usize,
    pub unsupported_not_proven_preservation_operation_count: usize,
    pub preflight_stage: StructuredFamilyDraftRenderedRecordStagedApplyStatus,
    pub review_stage: StructuredFamilyDraftRenderedRecordStagedApplyStatus,
    pub render_preview_stage: StructuredFamilyDraftRenderedRecordStagedApplyStatus,
    pub raw_fallback_preservation_stage: StructuredFamilyDraftRenderedRecordStagedApplyStatus,
    pub unsupported_not_proven_preservation_stage:
        StructuredFamilyDraftRenderedRecordStagedApplyStatus,
    pub dry_run_only_apply_stage: StructuredFamilyDraftRenderedRecordStagedApplyStatus,
    pub rollback_plan_stage: StructuredFamilyDraftRenderedRecordStagedApplyStatus,
    pub stages: Vec<StructuredFamilyDraftRenderedRecordStagedApplyStage>,
    pub operations: Vec<StructuredFamilyDraftRenderedRecordStagedApplyOperation>,
    pub summary_text: String,
    pub risk_summary: String,
    pub blockers: Vec<StructuredFamilyDraftRenderedRecordStagedApplyBlocker>,
    pub staged_apply_status: StructuredFamilyDraftRenderedRecordStagedApplyStatus,
    pub fixture_only_status: StructuredFamilyDraftRenderedRecordStagedApplyStatus,
    pub action_policy: StructuredFamilyDraftRenderedRecordStagedApplyStatus,
    pub write_policy: StructuredFamilyDraftRenderedRecordStagedApplyStatus,
    pub persistence_policy: StructuredFamilyDraftRenderedRecordStagedApplyStatus,
    pub real_config_target_policy: StructuredFamilyDraftRenderedRecordStagedApplyStatus,
    pub production_executor_policy: StructuredFamilyDraftRenderedRecordStagedApplyStatus,
    pub executor_availability_status: StructuredFamilyDraftRenderedRecordStagedApplyStatus,
    pub draft_written_to_disk: bool,
    pub staged_apply_plan_written_to_disk: bool,
    pub staged_apply_executed: bool,
    pub rendered_record_written_to_real_config: bool,
    pub real_config_touched: bool,
    pub runtime_mutated: bool,
    pub hyprctl_reload_run: bool,
    pub production_executor_wired: bool,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct StructuredFamilyDraftRenderedRecordStagedApplyDryRunEntry {
    pub family: StructuredFamilyKind,
    pub record_index: usize,
    pub stage_kind: StructuredFamilyDraftRenderedRecordApplyStageKind,
    pub operation_kind: String,
    pub original_raw_line: String,
    pub rendered_record_preview: String,
    pub status: StructuredFamilyDraftRenderedRecordStagedApplyDryRunStatus,
    pub action_policy: StructuredFamilyDraftRenderedRecordStagedApplyDryRunStatus,
    pub write_policy: StructuredFamilyDraftRenderedRecordStagedApplyDryRunStatus,
    pub persistence_policy: StructuredFamilyDraftRenderedRecordStagedApplyDryRunStatus,
    pub real_config_target_policy: StructuredFamilyDraftRenderedRecordStagedApplyDryRunStatus,
    pub production_executor_policy: StructuredFamilyDraftRenderedRecordStagedApplyDryRunStatus,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct StructuredFamilyDraftRenderedRecordStagedApplyDryRunReport {
    pub family: StructuredFamilyKind,
    pub source_draft_count: usize,
    pub source_plan_count: usize,
    pub review_entry_count: usize,
    pub changed_entry_count: usize,
    pub noop_entry_count: usize,
    pub raw_fallback_entry_count: usize,
    pub unsupported_not_proven_entry_count: usize,
    pub field_diff_count: usize,
    pub staged_apply_plan_linked: bool,
    pub stage_count: usize,
    pub operation_count: usize,
    pub changed_operation_count: usize,
    pub noop_operation_count: usize,
    pub raw_fallback_preservation_operation_count: usize,
    pub unsupported_not_proven_preservation_operation_count: usize,
    pub blocked_plan_count: usize,
    pub executor_unavailable_by_design: bool,
    pub dry_run_report_status: StructuredFamilyDraftRenderedRecordStagedApplyDryRunStatus,
    pub plan_status: StructuredFamilyDraftRenderedRecordStagedApplyDryRunStatus,
    pub stage_summary_status: StructuredFamilyDraftRenderedRecordStagedApplyDryRunStatus,
    pub operation_summary_status: StructuredFamilyDraftRenderedRecordStagedApplyDryRunStatus,
    pub changed_operation_summary_status:
        StructuredFamilyDraftRenderedRecordStagedApplyDryRunStatus,
    pub noop_operation_summary_status: StructuredFamilyDraftRenderedRecordStagedApplyDryRunStatus,
    pub raw_fallback_preservation_summary_status:
        StructuredFamilyDraftRenderedRecordStagedApplyDryRunStatus,
    pub unsupported_not_proven_preservation_summary_status:
        StructuredFamilyDraftRenderedRecordStagedApplyDryRunStatus,
    pub blocked_plan_summary_status: StructuredFamilyDraftRenderedRecordStagedApplyDryRunStatus,
    pub executor_availability_status: StructuredFamilyDraftRenderedRecordStagedApplyDryRunStatus,
    pub dry_run_execution_status: StructuredFamilyDraftRenderedRecordStagedApplyDryRunStatus,
    pub entries: Vec<StructuredFamilyDraftRenderedRecordStagedApplyDryRunEntry>,
    pub summary_text: String,
    pub risk_summary: String,
    pub fixture_only_status: StructuredFamilyDraftRenderedRecordStagedApplyDryRunStatus,
    pub action_policy: StructuredFamilyDraftRenderedRecordStagedApplyDryRunStatus,
    pub write_policy: StructuredFamilyDraftRenderedRecordStagedApplyDryRunStatus,
    pub persistence_policy: StructuredFamilyDraftRenderedRecordStagedApplyDryRunStatus,
    pub real_config_target_policy: StructuredFamilyDraftRenderedRecordStagedApplyDryRunStatus,
    pub production_executor_policy: StructuredFamilyDraftRenderedRecordStagedApplyDryRunStatus,
    pub blockers: Vec<StructuredFamilyDraftRenderedRecordStagedApplyBlocker>,
    pub draft_written_to_disk: bool,
    pub dry_run_report_written_to_disk: bool,
    pub staged_apply_plan_written_to_disk: bool,
    pub staged_apply_executed: bool,
    pub dry_run_executed: bool,
    pub rendered_record_written_to_real_config: bool,
    pub real_config_touched: bool,
    pub runtime_mutated: bool,
    pub hyprctl_reload_run: bool,
    pub production_executor_wired: bool,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct StructuredFamilyDraftRenderedRecordStagedApplyRollbackRecoveryEntry {
    pub family: StructuredFamilyKind,
    pub record_index: usize,
    pub stage_kind: StructuredFamilyDraftRenderedRecordApplyStageKind,
    pub operation_kind: String,
    pub original_raw_line: String,
    pub rendered_record_preview: String,
    pub status: StructuredFamilyDraftRenderedRecordStagedApplyRollbackRecoveryStatus,
    pub action_policy: StructuredFamilyDraftRenderedRecordStagedApplyRollbackRecoveryStatus,
    pub write_policy: StructuredFamilyDraftRenderedRecordStagedApplyRollbackRecoveryStatus,
    pub persistence_policy: StructuredFamilyDraftRenderedRecordStagedApplyRollbackRecoveryStatus,
    pub real_config_target_policy:
        StructuredFamilyDraftRenderedRecordStagedApplyRollbackRecoveryStatus,
    pub production_executor_policy:
        StructuredFamilyDraftRenderedRecordStagedApplyRollbackRecoveryStatus,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct StructuredFamilyDraftRenderedRecordStagedApplyRollbackRecoveryReview {
    pub family: StructuredFamilyKind,
    pub source_draft_count: usize,
    pub source_plan_count: usize,
    pub review_entry_count: usize,
    pub changed_entry_count: usize,
    pub noop_entry_count: usize,
    pub raw_fallback_entry_count: usize,
    pub unsupported_not_proven_entry_count: usize,
    pub field_diff_count: usize,
    pub dry_run_report_linked: bool,
    pub staged_apply_plan_linked: bool,
    pub stage_count: usize,
    pub operation_count: usize,
    pub changed_operation_count: usize,
    pub noop_operation_count: usize,
    pub raw_fallback_preservation_operation_count: usize,
    pub unsupported_not_proven_preservation_operation_count: usize,
    pub blocked_plan_count: usize,
    pub rollback_review_entry_count: usize,
    pub recovery_requirement_count: usize,
    pub backup_requirement_count: usize,
    pub restore_requirement_count: usize,
    pub blocked_recovery_reason_count: usize,
    pub executor_unavailable_by_design: bool,
    pub rollback_recovery_review_status:
        StructuredFamilyDraftRenderedRecordStagedApplyRollbackRecoveryStatus,
    pub dry_run_link_status: StructuredFamilyDraftRenderedRecordStagedApplyRollbackRecoveryStatus,
    pub staged_apply_plan_link_status:
        StructuredFamilyDraftRenderedRecordStagedApplyRollbackRecoveryStatus,
    pub rollback_plan_summary_status:
        StructuredFamilyDraftRenderedRecordStagedApplyRollbackRecoveryStatus,
    pub recovery_path_summary_status:
        StructuredFamilyDraftRenderedRecordStagedApplyRollbackRecoveryStatus,
    pub backup_requirement_status:
        StructuredFamilyDraftRenderedRecordStagedApplyRollbackRecoveryStatus,
    pub restore_requirement_status:
        StructuredFamilyDraftRenderedRecordStagedApplyRollbackRecoveryStatus,
    pub blocked_plan_preservation_status:
        StructuredFamilyDraftRenderedRecordStagedApplyRollbackRecoveryStatus,
    pub executor_availability_status:
        StructuredFamilyDraftRenderedRecordStagedApplyRollbackRecoveryStatus,
    pub execution_status: StructuredFamilyDraftRenderedRecordStagedApplyRollbackRecoveryStatus,
    pub summary_text: String,
    pub risk_summary: String,
    pub recovery_requirements: Vec<StructuredFamilyDraftRenderedRecordRollbackRecoveryRequirement>,
    pub entries: Vec<StructuredFamilyDraftRenderedRecordStagedApplyRollbackRecoveryEntry>,
    pub fixture_only_status: StructuredFamilyDraftRenderedRecordStagedApplyRollbackRecoveryStatus,
    pub action_policy: StructuredFamilyDraftRenderedRecordStagedApplyRollbackRecoveryStatus,
    pub write_policy: StructuredFamilyDraftRenderedRecordStagedApplyRollbackRecoveryStatus,
    pub persistence_policy: StructuredFamilyDraftRenderedRecordStagedApplyRollbackRecoveryStatus,
    pub real_config_target_policy:
        StructuredFamilyDraftRenderedRecordStagedApplyRollbackRecoveryStatus,
    pub production_executor_policy:
        StructuredFamilyDraftRenderedRecordStagedApplyRollbackRecoveryStatus,
    pub staged_apply_blockers: Vec<StructuredFamilyDraftRenderedRecordStagedApplyBlocker>,
    pub recovery_blockers: Vec<StructuredFamilyDraftRenderedRecordRollbackRecoveryBlocker>,
    pub draft_written_to_disk: bool,
    pub rollback_recovery_review_written_to_disk: bool,
    pub dry_run_report_written_to_disk: bool,
    pub staged_apply_plan_written_to_disk: bool,
    pub staged_apply_executed: bool,
    pub dry_run_executed: bool,
    pub rollback_executed: bool,
    pub recovery_executed: bool,
    pub backup_created: bool,
    pub restore_executed: bool,
    pub rendered_record_written_to_real_config: bool,
    pub real_config_touched: bool,
    pub runtime_mutated: bool,
    pub hyprctl_reload_run: bool,
    pub production_executor_wired: bool,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct StructuredFamilyDraftRenderedRecordFinalExecutorReadinessEntry {
    pub family: StructuredFamilyKind,
    pub record_index: usize,
    pub original_raw_line: String,
    pub rendered_record_preview: String,
    pub findings: Vec<StructuredFamilyDraftRenderedRecordFinalExecutorReadinessFinding>,
    pub status: StructuredFamilyDraftRenderedRecordFinalExecutorReadinessStatus,
    pub action_policy: StructuredFamilyDraftRenderedRecordFinalExecutorReadinessStatus,
    pub write_policy: StructuredFamilyDraftRenderedRecordFinalExecutorReadinessStatus,
    pub persistence_policy: StructuredFamilyDraftRenderedRecordFinalExecutorReadinessStatus,
    pub real_config_target_policy: StructuredFamilyDraftRenderedRecordFinalExecutorReadinessStatus,
    pub production_executor_policy: StructuredFamilyDraftRenderedRecordFinalExecutorReadinessStatus,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct StructuredFamilyDraftRenderedRecordFinalExecutorReadinessAudit {
    pub family: StructuredFamilyKind,
    pub source_draft_count: usize,
    pub source_plan_count: usize,
    pub review_entry_count: usize,
    pub changed_entry_count: usize,
    pub noop_entry_count: usize,
    pub raw_fallback_entry_count: usize,
    pub unsupported_not_proven_entry_count: usize,
    pub field_diff_count: usize,
    pub rollback_recovery_review_linked: bool,
    pub dry_run_report_linked: bool,
    pub staged_apply_plan_linked: bool,
    pub stage_count: usize,
    pub operation_count: usize,
    pub changed_operation_count: usize,
    pub noop_operation_count: usize,
    pub raw_fallback_preservation_operation_count: usize,
    pub unsupported_not_proven_preservation_operation_count: usize,
    pub blocked_plan_count: usize,
    pub recovery_requirement_count: usize,
    pub backup_requirement_count: usize,
    pub restore_requirement_count: usize,
    pub executor_unavailable_by_design: bool,
    pub final_audit_finding_count: usize,
    pub production_blocker_count: usize,
    pub fixture_only_proof_count: usize,
    pub final_audit_status: StructuredFamilyDraftRenderedRecordFinalExecutorReadinessStatus,
    pub rollback_recovery_link_status:
        StructuredFamilyDraftRenderedRecordFinalExecutorReadinessStatus,
    pub dry_run_link_status: StructuredFamilyDraftRenderedRecordFinalExecutorReadinessStatus,
    pub staged_apply_plan_link_status:
        StructuredFamilyDraftRenderedRecordFinalExecutorReadinessStatus,
    pub proof_chain_status: StructuredFamilyDraftRenderedRecordFinalExecutorReadinessStatus,
    pub fixture_only_completion_status:
        StructuredFamilyDraftRenderedRecordFinalExecutorReadinessStatus,
    pub production_activation_requirement_status:
        StructuredFamilyDraftRenderedRecordFinalExecutorReadinessStatus,
    pub executor_implementation_status:
        StructuredFamilyDraftRenderedRecordFinalExecutorReadinessStatus,
    pub executor_wiring_status: StructuredFamilyDraftRenderedRecordFinalExecutorReadinessStatus,
    pub real_write_boundary_status: StructuredFamilyDraftRenderedRecordFinalExecutorReadinessStatus,
    pub runtime_mutation_boundary_status:
        StructuredFamilyDraftRenderedRecordFinalExecutorReadinessStatus,
    pub reload_boundary_status: StructuredFamilyDraftRenderedRecordFinalExecutorReadinessStatus,
    pub backup_restore_implementation_status:
        StructuredFamilyDraftRenderedRecordFinalExecutorReadinessStatus,
    pub blocked_plan_preservation_status:
        StructuredFamilyDraftRenderedRecordFinalExecutorReadinessStatus,
    pub production_readiness_decision:
        StructuredFamilyDraftRenderedRecordFinalExecutorReadinessDecision,
    pub summary_text: String,
    pub risk_summary: String,
    pub findings: Vec<StructuredFamilyDraftRenderedRecordFinalExecutorReadinessFinding>,
    pub entries: Vec<StructuredFamilyDraftRenderedRecordFinalExecutorReadinessEntry>,
    pub blockers: Vec<StructuredFamilyDraftRenderedRecordFinalExecutorReadinessBlocker>,
    pub fixture_only_status: StructuredFamilyDraftRenderedRecordFinalExecutorReadinessStatus,
    pub action_policy: StructuredFamilyDraftRenderedRecordFinalExecutorReadinessStatus,
    pub write_policy: StructuredFamilyDraftRenderedRecordFinalExecutorReadinessStatus,
    pub persistence_policy: StructuredFamilyDraftRenderedRecordFinalExecutorReadinessStatus,
    pub real_config_target_policy: StructuredFamilyDraftRenderedRecordFinalExecutorReadinessStatus,
    pub production_executor_policy: StructuredFamilyDraftRenderedRecordFinalExecutorReadinessStatus,
    pub draft_written_to_disk: bool,
    pub final_audit_written_to_disk: bool,
    pub rollback_recovery_review_written_to_disk: bool,
    pub dry_run_report_written_to_disk: bool,
    pub staged_apply_plan_written_to_disk: bool,
    pub staged_apply_executed: bool,
    pub dry_run_executed: bool,
    pub rollback_executed: bool,
    pub recovery_executed: bool,
    pub backup_created: bool,
    pub restore_executed: bool,
    pub executor_implemented: bool,
    pub executor_wired: bool,
    pub production_activation_approved: bool,
    pub rendered_record_written_to_real_config: bool,
    pub real_config_touched: bool,
    pub runtime_mutated: bool,
    pub hyprctl_reload_run: bool,
    pub production_behavior_enabled: bool,
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

pub fn prove_structured_family_draft_rendered_record_render_reread(
    plans: &[StructuredFamilyDraftRenderedRecordPlan],
    rendered_fixture_path: impl AsRef<Path>,
) -> StructuredFamilyDraftRenderedRecordRenderRereadProof {
    let family = plans
        .first()
        .map(|plan| plan.family)
        .unwrap_or(StructuredFamilyKind::Monitor);
    let rendered_fixture_path = rendered_fixture_path.as_ref().to_path_buf();
    let rendered_fixture_text = render_draft_rendered_record_fixture_text(plans);
    let source_plan_count = plans.len();
    let field_map_count = plans.iter().map(|plan| plan.field_map.len()).sum::<usize>();
    let raw_fallback_plan_count = plans
        .iter()
        .filter(|plan| {
            plan.raw_fallback_status
                == StructuredFamilyDraftRenderedRecordStatus::RawFallbackPreserved
        })
        .count();
    let unsupported_not_proven_plan_count = plans
        .iter()
        .filter(|plan| {
            plan.unsupported_not_proven_status
                == StructuredFamilyDraftRenderedRecordStatus::UnsupportedNotProvenYet
        })
        .count();

    let mut proof = StructuredFamilyDraftRenderedRecordRenderRereadProof {
        family,
        source_draft_count: source_plan_count,
        source_plan_count,
        rendered_fixture_path,
        rendered_fixture_text,
        reread_projection_family: family,
        reread_record_count: 0,
        field_map_count,
        raw_fallback_plan_count,
        unsupported_not_proven_plan_count,
        family_preserved: false,
        record_count_preserved: false,
        field_map_preserved: field_map_count > 0,
        raw_fallback_preserved: raw_fallback_plan_count > 0,
        unsupported_not_proven_preserved: unsupported_not_proven_plan_count > 0,
        render_reread_status: StructuredFamilyDraftRenderedRecordRenderRereadStatus::Ready,
        rendered_temp_fixture_status:
            StructuredFamilyDraftRenderedRecordRenderRereadStatus::Unavailable,
        reread_status: StructuredFamilyDraftRenderedRecordRenderRereadStatus::Unavailable,
        family_preservation_status:
            StructuredFamilyDraftRenderedRecordRenderRereadStatus::Unavailable,
        record_count_preservation_status:
            StructuredFamilyDraftRenderedRecordRenderRereadStatus::Unavailable,
        field_map_preservation_status:
            StructuredFamilyDraftRenderedRecordRenderRereadStatus::FieldMapPreserved,
        raw_fallback_preservation_status:
            StructuredFamilyDraftRenderedRecordRenderRereadStatus::RawFallbackPreserved,
        unsupported_not_proven_preservation_status:
            StructuredFamilyDraftRenderedRecordRenderRereadStatus::UnsupportedNotProvenYet,
        fixture_only_status: StructuredFamilyDraftRenderedRecordRenderRereadStatus::FixtureOnly,
        action_policy: StructuredFamilyDraftRenderedRecordRenderRereadStatus::ActionsDisabled,
        write_policy: StructuredFamilyDraftRenderedRecordRenderRereadStatus::WritesBlockedByDefault,
        persistence_policy:
            StructuredFamilyDraftRenderedRecordRenderRereadStatus::PersistenceForbidden,
        real_config_target_policy:
            StructuredFamilyDraftRenderedRecordRenderRereadStatus::RealConfigTargetForbidden,
        draft_written_to_disk: false,
        rendered_record_written_to_temp_fixture: false,
        rendered_record_written_to_real_config: false,
        real_config_touched: false,
        runtime_mutated: false,
        hyprctl_reload_run: false,
        production_executor_wired: false,
    };

    if !structured_family_render_target_allowed(&proof.rendered_fixture_path) {
        proof.render_reread_status =
            StructuredFamilyDraftRenderedRecordRenderRereadStatus::RealConfigTargetForbidden;
        return proof;
    }

    let render_result = proof
        .rendered_fixture_path
        .parent()
        .ok_or_else(|| {
            std::io::Error::new(
                std::io::ErrorKind::InvalidInput,
                "render target has no parent",
            )
        })
        .and_then(|parent| fs::create_dir_all(parent))
        .and_then(|_| {
            fs::write(
                &proof.rendered_fixture_path,
                proof.rendered_fixture_text.as_bytes(),
            )
        });

    if render_result.is_err() {
        proof.render_reread_status =
            StructuredFamilyDraftRenderedRecordRenderRereadStatus::Unavailable;
        return proof;
    }

    proof.rendered_record_written_to_temp_fixture = true;
    proof.rendered_temp_fixture_status =
        StructuredFamilyDraftRenderedRecordRenderRereadStatus::RenderedToTempFixture;

    let reread_projection = parse_hyprland_config_file(&proof.rendered_fixture_path)
        .map(CurrentConfigSnapshot::from_parsed)
        .map(|snapshot| {
            snapshot
                .structured_family_projections()
                .into_iter()
                .find(|projection| projection.family == family)
        })
        .ok()
        .flatten();

    let Some(reread_projection) = reread_projection else {
        return proof;
    };

    proof.reread_projection_family = reread_projection.family;
    proof.reread_record_count = reread_projection.record_count();
    proof.family_preserved = reread_projection.family == family;
    proof.record_count_preserved = proof.reread_record_count == source_plan_count;
    proof.reread_status =
        StructuredFamilyDraftRenderedRecordRenderRereadStatus::RereadFromTempFixture;
    proof.family_preservation_status = if proof.family_preserved {
        StructuredFamilyDraftRenderedRecordRenderRereadStatus::FamilyPreserved
    } else {
        StructuredFamilyDraftRenderedRecordRenderRereadStatus::Unavailable
    };
    proof.record_count_preservation_status = if proof.record_count_preserved {
        StructuredFamilyDraftRenderedRecordRenderRereadStatus::RecordCountPreserved
    } else {
        StructuredFamilyDraftRenderedRecordRenderRereadStatus::Unavailable
    };
    proof.render_reread_status = StructuredFamilyDraftRenderedRecordRenderRereadStatus::ReviewOnly;
    proof
}

pub fn render_draft_rendered_record_fixture_text(
    plans: &[StructuredFamilyDraftRenderedRecordPlan],
) -> String {
    let mut output = plans
        .iter()
        .map(|plan| plan.rendered_record_preview.as_str())
        .collect::<Vec<_>>()
        .join("\n");
    if !output.is_empty() {
        output.push('\n');
    }
    output
}

pub fn structured_family_draft_rendered_record_diff_review_summary(
    plans: &[StructuredFamilyDraftRenderedRecordPlan],
    render_reread_proof: &StructuredFamilyDraftRenderedRecordRenderRereadProof,
) -> StructuredFamilyDraftRenderedRecordDiffReviewSummary {
    let family = plans
        .first()
        .map(|plan| plan.family)
        .unwrap_or(render_reread_proof.family);
    let entries = plans
        .iter()
        .map(structured_family_draft_rendered_record_diff_review_entry)
        .collect::<Vec<_>>();
    let changed_entry_count = entries.iter().filter(|entry| entry.changed).count();
    let review_entry_count = entries.len();
    let noop_entry_count = review_entry_count.saturating_sub(changed_entry_count);
    let raw_fallback_entry_count = entries
        .iter()
        .filter(|entry| {
            entry.raw_fallback_status
                == StructuredFamilyDraftRenderedRecordDiffReviewStatus::RawFallbackPreserved
        })
        .count();
    let unsupported_not_proven_entry_count = entries
        .iter()
        .filter(|entry| {
            entry.unsupported_not_proven_status
                == StructuredFamilyDraftRenderedRecordDiffReviewStatus::UnsupportedNotProvenYet
        })
        .count();
    let field_diff_count = entries
        .iter()
        .map(|entry| entry.field_diffs.len())
        .sum::<usize>();
    let changed_field_diff_count = entries
        .iter()
        .flat_map(|entry| entry.field_diffs.iter())
        .filter(|field_diff| field_diff.changed)
        .count();
    let summary_text = format!(
        "{} fixture-only diff/review summary: {} entries, {} changed, {} noop, {} raw fallback, {} not proven.",
        family.family_id(),
        review_entry_count,
        changed_entry_count,
        noop_entry_count,
        raw_fallback_entry_count,
        unsupported_not_proven_entry_count
    );
    let risk_summary =
        "review-only fixture diff; real writes, persistence, reload, runtime mutation, and production executors remain blocked"
            .to_string();

    StructuredFamilyDraftRenderedRecordDiffReviewSummary {
        family,
        source_draft_count: render_reread_proof.source_draft_count,
        source_plan_count: render_reread_proof.source_plan_count,
        render_reread_proof_status:
            StructuredFamilyDraftRenderedRecordDiffReviewStatus::RenderRereadProofLinked,
        review_entry_count,
        changed_entry_count,
        noop_entry_count,
        raw_fallback_entry_count,
        unsupported_not_proven_entry_count,
        field_diff_count,
        changed_field_diff_count,
        entries,
        summary_text,
        risk_summary,
        review_decision_status:
            StructuredFamilyDraftRenderedRecordDiffReviewStatus::ReviewSummaryReady,
        diff_review_status: StructuredFamilyDraftRenderedRecordDiffReviewStatus::ReviewOnly,
        review_summary_status:
            StructuredFamilyDraftRenderedRecordDiffReviewStatus::ReviewSummaryReady,
        field_diff_status: StructuredFamilyDraftRenderedRecordDiffReviewStatus::FieldDiffReady,
        changed_entry_status: if changed_entry_count > 0 {
            StructuredFamilyDraftRenderedRecordDiffReviewStatus::Changed
        } else {
            StructuredFamilyDraftRenderedRecordDiffReviewStatus::Noop
        },
        noop_entry_status: StructuredFamilyDraftRenderedRecordDiffReviewStatus::Noop,
        raw_fallback_review_status:
            StructuredFamilyDraftRenderedRecordDiffReviewStatus::RawFallbackPreserved,
        unsupported_not_proven_review_status:
            StructuredFamilyDraftRenderedRecordDiffReviewStatus::UnsupportedNotProvenYet,
        fixture_only_status: StructuredFamilyDraftRenderedRecordDiffReviewStatus::FixtureOnly,
        action_policy: StructuredFamilyDraftRenderedRecordDiffReviewStatus::ActionsDisabled,
        write_policy: StructuredFamilyDraftRenderedRecordDiffReviewStatus::WritesBlockedByDefault,
        persistence_policy:
            StructuredFamilyDraftRenderedRecordDiffReviewStatus::PersistenceForbidden,
        real_config_target_policy:
            StructuredFamilyDraftRenderedRecordDiffReviewStatus::RealConfigTargetForbidden,
        draft_written_to_disk: false,
        diff_summary_written_to_disk: false,
        rendered_record_written_to_temp_fixture: render_reread_proof
            .rendered_record_written_to_temp_fixture,
        rendered_record_written_to_real_config: false,
        real_config_touched: false,
        runtime_mutated: false,
        hyprctl_reload_run: false,
        production_executor_wired: false,
    }
}

fn structured_family_draft_rendered_record_diff_review_entry(
    plan: &StructuredFamilyDraftRenderedRecordPlan,
) -> StructuredFamilyDraftRenderedRecordDiffReviewEntry {
    let field_diffs = plan
        .field_map
        .iter()
        .map(|field_map| {
            let draft_field = plan
                .draft_fields
                .iter()
                .find(|field| field.name == field_map.field_name);
            let original_value = draft_field
                .map(|field| field.original_value.clone())
                .unwrap_or_default();
            let draft_value = draft_field
                .map(|field| field.draft_value.clone())
                .unwrap_or_else(|| field_map.draft_value.clone());
            StructuredFamilyDraftRenderedRecordFieldDiff {
                field_name: field_map.field_name.clone(),
                original_value: original_value.clone(),
                draft_value: draft_value.clone(),
                rendered_part: field_map.rendered_part.clone(),
                changed: original_value != draft_value,
                status: StructuredFamilyDraftRenderedRecordDiffReviewStatus::FieldDiffReady,
            }
        })
        .collect::<Vec<_>>();
    let field_changed = field_diffs.iter().any(|field_diff| field_diff.changed);
    let preview_changed = plan.raw_original_line.trim() != plan.rendered_record_preview.trim();
    let changed = preview_changed || field_changed;
    let unsupported = plan.unsupported_reason.is_some()
        || plan.unsupported_not_proven_status
            == StructuredFamilyDraftRenderedRecordStatus::UnsupportedNotProvenYet;
    let raw_fallback =
        plan.raw_fallback_status == StructuredFamilyDraftRenderedRecordStatus::RawFallbackPreserved;

    StructuredFamilyDraftRenderedRecordDiffReviewEntry {
        family: plan.family,
        record_index: plan.record_index,
        source_path: plan.source_path.clone(),
        line_number: plan.line_number,
        original_raw_line: plan.raw_original_line.clone(),
        rendered_record_preview: plan.rendered_record_preview.clone(),
        field_diffs,
        changed,
        diff_status: if changed {
            StructuredFamilyDraftRenderedRecordDiffReviewStatus::Changed
        } else {
            StructuredFamilyDraftRenderedRecordDiffReviewStatus::Noop
        },
        field_diff_status: StructuredFamilyDraftRenderedRecordDiffReviewStatus::FieldDiffReady,
        rendered_preview_compared_status:
            StructuredFamilyDraftRenderedRecordDiffReviewStatus::RenderedPreviewCompared,
        original_raw_preserved_status:
            StructuredFamilyDraftRenderedRecordDiffReviewStatus::OriginalRawPreserved,
        raw_fallback_status: if raw_fallback {
            StructuredFamilyDraftRenderedRecordDiffReviewStatus::RawFallbackPreserved
        } else {
            StructuredFamilyDraftRenderedRecordDiffReviewStatus::FieldDiffReady
        },
        unsupported_not_proven_status: if unsupported {
            StructuredFamilyDraftRenderedRecordDiffReviewStatus::UnsupportedNotProvenYet
        } else {
            StructuredFamilyDraftRenderedRecordDiffReviewStatus::FieldDiffReady
        },
        not_safe_for_full_synthesis: unsupported,
        summary_text: if changed {
            "Rendered preview differs from original raw line or in-memory draft field values"
                .to_string()
        } else {
            "Rendered preview matches original raw line and draft fields remain clean".to_string()
        },
        risk_summary: if unsupported {
            "not proven yet; raw fallback preserved and full synthesis is not approved".to_string()
        } else {
            "fixture-only review entry; no real write is authorized".to_string()
        },
        review_decision_status:
            StructuredFamilyDraftRenderedRecordDiffReviewStatus::ReviewSummaryReady,
    }
}

pub fn structured_family_draft_rendered_record_approval_draft(
    summary: &StructuredFamilyDraftRenderedRecordDiffReviewSummary,
) -> StructuredFamilyDraftRenderedRecordApprovalDraft {
    StructuredFamilyDraftRenderedRecordApprovalDraft {
        family: summary.family,
        source_draft_count: summary.source_draft_count,
        source_plan_count: summary.source_plan_count,
        review_entry_count: summary.review_entry_count,
        changed_entry_count: summary.changed_entry_count,
        noop_entry_count: summary.noop_entry_count,
        raw_fallback_entry_count: summary.raw_fallback_entry_count,
        unsupported_not_proven_entry_count: summary.unsupported_not_proven_entry_count,
        field_diff_count: summary.field_diff_count,
        summary_text: format!(
            "{} Approval is for fixture-only next-stage review only. {}",
            summary.family.family_id(),
            summary.summary_text
        ),
        risk_summary: format!(
            "{}; approval does not authorize real config writes, persistence, reload, runtime mutation, or production executor wiring",
            summary.risk_summary
        ),
        diff_review_summary_linked: true,
        render_reread_proof_linked: summary.render_reread_proof_status
            == StructuredFamilyDraftRenderedRecordDiffReviewStatus::RenderRereadProofLinked,
        changed_entries_acknowledged: summary.changed_entry_count == 0,
        noop_entries_acknowledged: summary.noop_entry_count == 0,
        raw_fallback_acknowledged: summary.raw_fallback_entry_count == 0,
        unsupported_not_proven_acknowledged: summary.unsupported_not_proven_entry_count == 0,
        approval_status: StructuredFamilyDraftRenderedRecordApprovalStatus::ReviewOnly,
        confirmation_status: StructuredFamilyDraftRenderedRecordApprovalStatus::ConfirmationDraftReady,
        fixture_only_status: StructuredFamilyDraftRenderedRecordApprovalStatus::FixtureOnly,
        action_policy: StructuredFamilyDraftRenderedRecordApprovalStatus::ActionsDisabled,
        write_policy: StructuredFamilyDraftRenderedRecordApprovalStatus::WritesBlockedByDefault,
        persistence_policy: StructuredFamilyDraftRenderedRecordApprovalStatus::PersistenceForbidden,
        real_config_target_policy:
            StructuredFamilyDraftRenderedRecordApprovalStatus::RealConfigTargetForbidden,
        production_executor_policy:
            StructuredFamilyDraftRenderedRecordApprovalStatus::ProductionExecutorForbidden,
        draft_written_to_disk: false,
        approval_written_to_disk: false,
        confirmation_written_to_disk: false,
        rendered_record_written_to_real_config: false,
        real_config_touched: false,
        runtime_mutated: false,
        hyprctl_reload_run: false,
        production_executor_wired: false,
    }
}

pub fn structured_family_draft_rendered_record_confirmation_request(
    draft: &StructuredFamilyDraftRenderedRecordApprovalDraft,
) -> StructuredFamilyDraftRenderedRecordConfirmationRequest {
    StructuredFamilyDraftRenderedRecordConfirmationRequest {
        family: draft.family,
        source_plan_count: draft.source_plan_count,
        review_entry_count: draft.review_entry_count,
        changed_entry_count: draft.changed_entry_count,
        raw_fallback_entry_count: draft.raw_fallback_entry_count,
        unsupported_not_proven_entry_count: draft.unsupported_not_proven_entry_count,
        diff_review_summary_linked: draft.diff_review_summary_linked,
        render_reread_proof_linked: draft.render_reread_proof_linked,
        changed_entries_acknowledged: true,
        noop_entries_acknowledged: true,
        raw_fallback_acknowledged: true,
        unsupported_not_proven_acknowledged: true,
        real_config_target_forbidden: true,
        persistence_forbidden: true,
        runtime_mutation_forbidden: true,
        hyprland_reload_forbidden: true,
        production_executor_forbidden: true,
    }
}

pub fn accept_structured_family_draft_rendered_record_confirmation(
    draft: &StructuredFamilyDraftRenderedRecordApprovalDraft,
    request: &StructuredFamilyDraftRenderedRecordConfirmationRequest,
) -> StructuredFamilyDraftRenderedRecordConfirmation {
    let invalidation_reasons =
        structured_family_draft_rendered_record_confirmation_invalidation_reasons(draft, request);
    structured_family_draft_rendered_record_confirmation(
        draft,
        request,
        invalidation_reasons.is_empty(),
        false,
        invalidation_reasons,
    )
}

pub fn reject_structured_family_draft_rendered_record_confirmation(
    draft: &StructuredFamilyDraftRenderedRecordApprovalDraft,
    request: &StructuredFamilyDraftRenderedRecordConfirmationRequest,
) -> StructuredFamilyDraftRenderedRecordConfirmation {
    structured_family_draft_rendered_record_confirmation(draft, request, false, true, Vec::new())
}

pub fn structured_family_draft_rendered_record_confirmation_invalidation_reasons(
    draft: &StructuredFamilyDraftRenderedRecordApprovalDraft,
    request: &StructuredFamilyDraftRenderedRecordConfirmationRequest,
) -> Vec<StructuredFamilyDraftRenderedRecordConfirmationInvalidationReason> {
    let mut reasons = Vec::new();
    if request.family != draft.family {
        reasons.push(
            StructuredFamilyDraftRenderedRecordConfirmationInvalidationReason::MismatchedFamily,
        );
    }
    if request.source_plan_count != draft.source_plan_count {
        reasons.push(
            StructuredFamilyDraftRenderedRecordConfirmationInvalidationReason::MismatchedSourcePlanCount,
        );
    }
    if request.review_entry_count != draft.review_entry_count {
        reasons.push(
            StructuredFamilyDraftRenderedRecordConfirmationInvalidationReason::MismatchedReviewEntryCount,
        );
    }
    if request.changed_entry_count != draft.changed_entry_count {
        reasons.push(
            StructuredFamilyDraftRenderedRecordConfirmationInvalidationReason::MismatchedChangedEntryCount,
        );
    }
    if request.raw_fallback_entry_count != draft.raw_fallback_entry_count {
        reasons.push(
            StructuredFamilyDraftRenderedRecordConfirmationInvalidationReason::MismatchedRawFallbackCount,
        );
    }
    if request.unsupported_not_proven_entry_count != draft.unsupported_not_proven_entry_count {
        reasons.push(
            StructuredFamilyDraftRenderedRecordConfirmationInvalidationReason::MismatchedUnsupportedNotProvenCount,
        );
    }
    if !request.diff_review_summary_linked {
        reasons.push(
            StructuredFamilyDraftRenderedRecordConfirmationInvalidationReason::MissingDiffReviewSummary,
        );
    }
    if !request.render_reread_proof_linked {
        reasons.push(
            StructuredFamilyDraftRenderedRecordConfirmationInvalidationReason::MissingRenderRereadProofLink,
        );
    }
    if draft.raw_fallback_entry_count > 0 && !request.raw_fallback_acknowledged {
        reasons.push(
            StructuredFamilyDraftRenderedRecordConfirmationInvalidationReason::RawFallbackRequiresAcknowledgement,
        );
    }
    if draft.unsupported_not_proven_entry_count > 0 && !request.unsupported_not_proven_acknowledged
    {
        reasons.push(
            StructuredFamilyDraftRenderedRecordConfirmationInvalidationReason::UnsupportedNotProvenRequiresAcknowledgement,
        );
    }
    if !request.real_config_target_forbidden {
        reasons.push(
            StructuredFamilyDraftRenderedRecordConfirmationInvalidationReason::RealConfigTargetNotAllowed,
        );
    }
    if !request.persistence_forbidden {
        reasons.push(
            StructuredFamilyDraftRenderedRecordConfirmationInvalidationReason::PersistenceNotAllowed,
        );
    }
    if !request.runtime_mutation_forbidden {
        reasons.push(
            StructuredFamilyDraftRenderedRecordConfirmationInvalidationReason::RuntimeMutationNotAllowed,
        );
    }
    if !request.hyprland_reload_forbidden {
        reasons.push(
            StructuredFamilyDraftRenderedRecordConfirmationInvalidationReason::HyprlandReloadNotAllowed,
        );
    }
    if !request.production_executor_forbidden {
        reasons.push(
            StructuredFamilyDraftRenderedRecordConfirmationInvalidationReason::ProductionExecutorNotAllowed,
        );
    }
    reasons
}

fn structured_family_draft_rendered_record_confirmation(
    draft: &StructuredFamilyDraftRenderedRecordApprovalDraft,
    request: &StructuredFamilyDraftRenderedRecordConfirmationRequest,
    accepted: bool,
    rejected: bool,
    invalidation_reasons: Vec<StructuredFamilyDraftRenderedRecordConfirmationInvalidationReason>,
) -> StructuredFamilyDraftRenderedRecordConfirmation {
    let confirmation_status = if !invalidation_reasons.is_empty() {
        StructuredFamilyDraftRenderedRecordApprovalStatus::ConfirmationInvalidated
    } else if accepted {
        StructuredFamilyDraftRenderedRecordApprovalStatus::ConfirmationAcceptedInMemory
    } else if rejected {
        StructuredFamilyDraftRenderedRecordApprovalStatus::ConfirmationRejectedInMemory
    } else {
        StructuredFamilyDraftRenderedRecordApprovalStatus::ConfirmationDraftReady
    };

    StructuredFamilyDraftRenderedRecordConfirmation {
        family: draft.family,
        source_draft_count: draft.source_draft_count,
        source_plan_count: draft.source_plan_count,
        review_entry_count: draft.review_entry_count,
        changed_entry_count: draft.changed_entry_count,
        noop_entry_count: draft.noop_entry_count,
        raw_fallback_entry_count: draft.raw_fallback_entry_count,
        unsupported_not_proven_entry_count: draft.unsupported_not_proven_entry_count,
        field_diff_count: draft.field_diff_count,
        summary_text: draft.summary_text.clone(),
        risk_summary: draft.risk_summary.clone(),
        diff_review_summary_linked: request.diff_review_summary_linked,
        render_reread_proof_linked: request.render_reread_proof_linked,
        changed_entries_acknowledged: request.changed_entries_acknowledged,
        noop_entries_acknowledged: request.noop_entries_acknowledged,
        raw_fallback_acknowledged: request.raw_fallback_acknowledged,
        unsupported_not_proven_acknowledged: request.unsupported_not_proven_acknowledged,
        approval_status: StructuredFamilyDraftRenderedRecordApprovalStatus::Ready,
        confirmation_status,
        confirmation_accepted_in_memory: accepted && invalidation_reasons.is_empty(),
        confirmation_rejected_in_memory: rejected,
        confirmation_invalidation_reasons: invalidation_reasons,
        fixture_only_status: StructuredFamilyDraftRenderedRecordApprovalStatus::FixtureOnly,
        action_policy: StructuredFamilyDraftRenderedRecordApprovalStatus::ActionsDisabled,
        write_policy: StructuredFamilyDraftRenderedRecordApprovalStatus::WritesBlockedByDefault,
        persistence_policy: StructuredFamilyDraftRenderedRecordApprovalStatus::PersistenceForbidden,
        real_config_target_policy:
            StructuredFamilyDraftRenderedRecordApprovalStatus::RealConfigTargetForbidden,
        production_executor_policy:
            StructuredFamilyDraftRenderedRecordApprovalStatus::ProductionExecutorForbidden,
        draft_written_to_disk: false,
        approval_written_to_disk: false,
        confirmation_written_to_disk: false,
        rendered_record_written_to_real_config: false,
        real_config_touched: false,
        runtime_mutated: false,
        hyprctl_reload_run: false,
        production_executor_wired: false,
    }
}

pub fn structured_family_draft_rendered_record_staged_apply_plan(
    confirmation: &StructuredFamilyDraftRenderedRecordConfirmation,
    summary: &StructuredFamilyDraftRenderedRecordDiffReviewSummary,
) -> StructuredFamilyDraftRenderedRecordStagedApplyPlan {
    let blockers =
        structured_family_draft_rendered_record_staged_apply_blockers(confirmation, summary);
    let ready = blockers.is_empty();
    let stages = structured_family_draft_rendered_record_staged_apply_stages(
        confirmation.family,
        summary,
        ready,
    );
    let operations = structured_family_draft_rendered_record_staged_apply_operations(summary);
    let changed_operation_count = operations
        .iter()
        .filter(|operation| operation.operation_kind == "changed review operation")
        .count();
    let noop_operation_count = operations
        .iter()
        .filter(|operation| operation.operation_kind == "noop preservation operation")
        .count();
    let raw_fallback_preservation_operation_count = operations
        .iter()
        .filter(|operation| operation.operation_kind == "raw fallback preservation operation")
        .count();
    let unsupported_not_proven_preservation_operation_count = operations
        .iter()
        .filter(|operation| {
            operation.operation_kind == "unsupported/not-proven preservation operation"
        })
        .count();
    let staged_apply_status = if ready {
        StructuredFamilyDraftRenderedRecordStagedApplyStatus::PlanReady
    } else if confirmation.confirmation_status
        == StructuredFamilyDraftRenderedRecordApprovalStatus::ConfirmationRejectedInMemory
    {
        StructuredFamilyDraftRenderedRecordStagedApplyStatus::RejectedConfirmationBlocked
    } else if confirmation.confirmation_status
        == StructuredFamilyDraftRenderedRecordApprovalStatus::ConfirmationInvalidated
    {
        StructuredFamilyDraftRenderedRecordStagedApplyStatus::InvalidConfirmationBlocked
    } else {
        StructuredFamilyDraftRenderedRecordStagedApplyStatus::AcceptedConfirmationRequired
    };

    StructuredFamilyDraftRenderedRecordStagedApplyPlan {
        family: confirmation.family,
        source_draft_count: confirmation.source_draft_count,
        source_plan_count: confirmation.source_plan_count,
        review_entry_count: confirmation.review_entry_count,
        changed_entry_count: confirmation.changed_entry_count,
        noop_entry_count: confirmation.noop_entry_count,
        raw_fallback_entry_count: confirmation.raw_fallback_entry_count,
        unsupported_not_proven_entry_count: confirmation.unsupported_not_proven_entry_count,
        field_diff_count: confirmation.field_diff_count,
        accepted_confirmation_linked: confirmation.confirmation_accepted_in_memory && ready,
        diff_review_summary_linked: confirmation.diff_review_summary_linked,
        render_reread_proof_linked: confirmation.render_reread_proof_linked,
        stage_count: stages.len(),
        operation_count: operations.len(),
        changed_operation_count,
        noop_operation_count,
        raw_fallback_preservation_operation_count,
        unsupported_not_proven_preservation_operation_count,
        preflight_stage: StructuredFamilyDraftRenderedRecordStagedApplyStatus::PreflightReady,
        review_stage: StructuredFamilyDraftRenderedRecordStagedApplyStatus::DiffReviewLinked,
        render_preview_stage: StructuredFamilyDraftRenderedRecordStagedApplyStatus::RenderRereadLinked,
        raw_fallback_preservation_stage:
            StructuredFamilyDraftRenderedRecordStagedApplyStatus::RawFallbackPreserved,
        unsupported_not_proven_preservation_stage:
            StructuredFamilyDraftRenderedRecordStagedApplyStatus::UnsupportedNotProvenPreserved,
        dry_run_only_apply_stage: StructuredFamilyDraftRenderedRecordStagedApplyStatus::DryRunOnly,
        rollback_plan_stage: StructuredFamilyDraftRenderedRecordStagedApplyStatus::RollbackPlanReady,
        stages,
        operations,
        summary_text: format!(
            "{} staged apply plan generated for fixture-only review: {} stages, {} operations, execution disabled.",
            confirmation.family.family_id(),
            7,
            changed_operation_count
                + noop_operation_count
                + raw_fallback_preservation_operation_count
                + unsupported_not_proven_preservation_operation_count
        ),
        risk_summary:
            "staged apply plan is review-only and in memory; real config writes, persistence, runtime mutation, reload, and production executors remain blocked"
                .to_string(),
        blockers,
        staged_apply_status,
        fixture_only_status: StructuredFamilyDraftRenderedRecordStagedApplyStatus::FixtureOnly,
        action_policy: StructuredFamilyDraftRenderedRecordStagedApplyStatus::ActionsDisabled,
        write_policy: StructuredFamilyDraftRenderedRecordStagedApplyStatus::WritesBlockedByDefault,
        persistence_policy: StructuredFamilyDraftRenderedRecordStagedApplyStatus::PersistenceForbidden,
        real_config_target_policy:
            StructuredFamilyDraftRenderedRecordStagedApplyStatus::RealConfigTargetForbidden,
        production_executor_policy:
            StructuredFamilyDraftRenderedRecordStagedApplyStatus::ProductionExecutorForbidden,
        executor_availability_status:
            StructuredFamilyDraftRenderedRecordStagedApplyStatus::ProductionExecutorForbidden,
        draft_written_to_disk: false,
        staged_apply_plan_written_to_disk: false,
        staged_apply_executed: false,
        rendered_record_written_to_real_config: false,
        real_config_touched: false,
        runtime_mutated: false,
        hyprctl_reload_run: false,
        production_executor_wired: false,
    }
}

pub fn structured_family_draft_rendered_record_staged_apply_blockers(
    confirmation: &StructuredFamilyDraftRenderedRecordConfirmation,
    summary: &StructuredFamilyDraftRenderedRecordDiffReviewSummary,
) -> Vec<StructuredFamilyDraftRenderedRecordStagedApplyBlocker> {
    let mut blockers = Vec::new();
    if confirmation.family != summary.family
        || confirmation.source_plan_count != summary.source_plan_count
        || confirmation.review_entry_count != summary.review_entry_count
        || confirmation.changed_entry_count != summary.changed_entry_count
        || confirmation.raw_fallback_entry_count != summary.raw_fallback_entry_count
        || confirmation.unsupported_not_proven_entry_count
            != summary.unsupported_not_proven_entry_count
    {
        blockers.push(
            StructuredFamilyDraftRenderedRecordStagedApplyBlocker::MissingAcceptedConfirmation,
        );
    }
    match confirmation.confirmation_status {
        StructuredFamilyDraftRenderedRecordApprovalStatus::ConfirmationAcceptedInMemory => {
            if !confirmation.confirmation_accepted_in_memory {
                blockers.push(
                    StructuredFamilyDraftRenderedRecordStagedApplyBlocker::MissingAcceptedConfirmation,
                );
            }
        }
        StructuredFamilyDraftRenderedRecordApprovalStatus::ConfirmationRejectedInMemory => {
            blockers
                .push(StructuredFamilyDraftRenderedRecordStagedApplyBlocker::RejectedConfirmation);
        }
        StructuredFamilyDraftRenderedRecordApprovalStatus::ConfirmationInvalidated => {
            blockers
                .push(StructuredFamilyDraftRenderedRecordStagedApplyBlocker::InvalidConfirmation);
        }
        _ => blockers.push(
            StructuredFamilyDraftRenderedRecordStagedApplyBlocker::MissingAcceptedConfirmation,
        ),
    }
    if !confirmation.confirmation_invalidation_reasons.is_empty() {
        blockers.push(StructuredFamilyDraftRenderedRecordStagedApplyBlocker::InvalidConfirmation);
    }
    if !confirmation.diff_review_summary_linked {
        blockers
            .push(StructuredFamilyDraftRenderedRecordStagedApplyBlocker::MissingDiffReviewSummary);
    }
    if !confirmation.render_reread_proof_linked {
        blockers
            .push(StructuredFamilyDraftRenderedRecordStagedApplyBlocker::MissingRenderRereadProof);
    }
    if confirmation.raw_fallback_entry_count > 0 && !confirmation.raw_fallback_acknowledged {
        blockers.push(
            StructuredFamilyDraftRenderedRecordStagedApplyBlocker::MissingApprovalAcknowledgement,
        );
    }
    if confirmation.unsupported_not_proven_entry_count > 0
        && !confirmation.unsupported_not_proven_acknowledged
    {
        blockers.push(
            StructuredFamilyDraftRenderedRecordStagedApplyBlocker::MissingApprovalAcknowledgement,
        );
    }
    if summary.raw_fallback_entry_count > 0
        && !summary.entries.iter().any(|entry| {
            entry.raw_fallback_status
                == StructuredFamilyDraftRenderedRecordDiffReviewStatus::RawFallbackPreserved
        })
    {
        blockers.push(
            StructuredFamilyDraftRenderedRecordStagedApplyBlocker::RawFallbackRequiresPreservation,
        );
    }
    if summary.unsupported_not_proven_entry_count > 0
        && !summary.entries.iter().any(|entry| {
            entry.unsupported_not_proven_status
                == StructuredFamilyDraftRenderedRecordDiffReviewStatus::UnsupportedNotProvenYet
        })
    {
        blockers.push(
            StructuredFamilyDraftRenderedRecordStagedApplyBlocker::UnsupportedNotProvenRequiresPreservation,
        );
    }
    if confirmation.real_config_target_policy
        != StructuredFamilyDraftRenderedRecordApprovalStatus::RealConfigTargetForbidden
        || confirmation.rendered_record_written_to_real_config
        || confirmation.real_config_touched
    {
        blockers.push(
            StructuredFamilyDraftRenderedRecordStagedApplyBlocker::RealConfigTargetNotAllowed,
        );
    }
    if confirmation.persistence_policy
        != StructuredFamilyDraftRenderedRecordApprovalStatus::PersistenceForbidden
        || confirmation.draft_written_to_disk
        || confirmation.approval_written_to_disk
        || confirmation.confirmation_written_to_disk
    {
        blockers.push(StructuredFamilyDraftRenderedRecordStagedApplyBlocker::PersistenceNotAllowed);
    }
    if confirmation.runtime_mutated {
        blockers
            .push(StructuredFamilyDraftRenderedRecordStagedApplyBlocker::RuntimeMutationNotAllowed);
    }
    if confirmation.hyprctl_reload_run {
        blockers
            .push(StructuredFamilyDraftRenderedRecordStagedApplyBlocker::HyprlandReloadNotAllowed);
    }
    if confirmation.production_executor_policy
        != StructuredFamilyDraftRenderedRecordApprovalStatus::ProductionExecutorForbidden
        || confirmation.production_executor_wired
    {
        blockers.push(
            StructuredFamilyDraftRenderedRecordStagedApplyBlocker::ProductionExecutorNotAllowed,
        );
    }
    blockers
}

fn structured_family_draft_rendered_record_staged_apply_stages(
    family: StructuredFamilyKind,
    summary: &StructuredFamilyDraftRenderedRecordDiffReviewSummary,
    ready: bool,
) -> Vec<StructuredFamilyDraftRenderedRecordStagedApplyStage> {
    [
        (
            StructuredFamilyDraftRenderedRecordApplyStageKind::Preflight,
            StructuredFamilyDraftRenderedRecordStagedApplyStatus::PreflightReady,
            "confirm fixture-only inputs and no-write policies",
            0,
        ),
        (
            StructuredFamilyDraftRenderedRecordApplyStageKind::Review,
            StructuredFamilyDraftRenderedRecordStagedApplyStatus::DiffReviewLinked,
            "review diff entries and acknowledgements",
            summary.review_entry_count,
        ),
        (
            StructuredFamilyDraftRenderedRecordApplyStageKind::RenderPreview,
            StructuredFamilyDraftRenderedRecordStagedApplyStatus::RenderRereadLinked,
            "link rendered-record previews to render/reread proof",
            summary.review_entry_count,
        ),
        (
            StructuredFamilyDraftRenderedRecordApplyStageKind::RawFallbackPreservation,
            StructuredFamilyDraftRenderedRecordStagedApplyStatus::RawFallbackPreserved,
            "preserve raw fallback entries without synthesis",
            summary.raw_fallback_entry_count,
        ),
        (
            StructuredFamilyDraftRenderedRecordApplyStageKind::UnsupportedNotProvenPreservation,
            StructuredFamilyDraftRenderedRecordStagedApplyStatus::UnsupportedNotProvenPreserved,
            "preserve unsupported or not-proven entries",
            summary.unsupported_not_proven_entry_count,
        ),
        (
            StructuredFamilyDraftRenderedRecordApplyStageKind::DryRunOnlyApply,
            StructuredFamilyDraftRenderedRecordStagedApplyStatus::DryRunOnly,
            "represent dry-run-only apply stage without execution",
            summary.review_entry_count,
        ),
        (
            StructuredFamilyDraftRenderedRecordApplyStageKind::RollbackPlan,
            StructuredFamilyDraftRenderedRecordStagedApplyStatus::RollbackPlanReady,
            "represent in-memory rollback plan for future fixture-only review",
            0,
        ),
    ]
    .into_iter()
    .map(|(stage_kind, status, summary_text, operation_count)| {
        StructuredFamilyDraftRenderedRecordStagedApplyStage {
            family,
            stage_kind,
            status: if ready {
                status
            } else {
                StructuredFamilyDraftRenderedRecordStagedApplyStatus::AcceptedConfirmationRequired
            },
            operation_count,
            summary_text: format!("{}: {summary_text}", stage_kind.as_str()),
            risk_summary:
                "stage is review-only; it does not execute, persist, reload, or mutate runtime"
                    .to_string(),
        }
    })
    .collect()
}

fn structured_family_draft_rendered_record_staged_apply_operations(
    summary: &StructuredFamilyDraftRenderedRecordDiffReviewSummary,
) -> Vec<StructuredFamilyDraftRenderedRecordStagedApplyOperation> {
    let mut operations = Vec::new();
    for entry in &summary.entries {
        operations.push(
            structured_family_draft_rendered_record_staged_apply_operation(
                entry,
                if entry.changed {
                    "changed review operation"
                } else {
                    "noop preservation operation"
                },
                if entry.changed {
                    StructuredFamilyDraftRenderedRecordApplyStageKind::Review
                } else {
                    StructuredFamilyDraftRenderedRecordApplyStageKind::Review
                },
                if entry.changed {
                    StructuredFamilyDraftRenderedRecordStagedApplyStatus::OperationsReady
                } else {
                    StructuredFamilyDraftRenderedRecordStagedApplyStatus::OperationsReady
                },
            ),
        );
        if entry.raw_fallback_status
            == StructuredFamilyDraftRenderedRecordDiffReviewStatus::RawFallbackPreserved
        {
            operations.push(
                structured_family_draft_rendered_record_staged_apply_operation(
                    entry,
                    "raw fallback preservation operation",
                    StructuredFamilyDraftRenderedRecordApplyStageKind::RawFallbackPreservation,
                    StructuredFamilyDraftRenderedRecordStagedApplyStatus::RawFallbackPreserved,
                ),
            );
        }
        if entry.unsupported_not_proven_status
            == StructuredFamilyDraftRenderedRecordDiffReviewStatus::UnsupportedNotProvenYet
        {
            operations.push(structured_family_draft_rendered_record_staged_apply_operation(
                entry,
                "unsupported/not-proven preservation operation",
                StructuredFamilyDraftRenderedRecordApplyStageKind::UnsupportedNotProvenPreservation,
                StructuredFamilyDraftRenderedRecordStagedApplyStatus::UnsupportedNotProvenPreserved,
            ));
        }
    }
    operations
}

fn structured_family_draft_rendered_record_staged_apply_operation(
    entry: &StructuredFamilyDraftRenderedRecordDiffReviewEntry,
    operation_kind: &str,
    stage_kind: StructuredFamilyDraftRenderedRecordApplyStageKind,
    status: StructuredFamilyDraftRenderedRecordStagedApplyStatus,
) -> StructuredFamilyDraftRenderedRecordStagedApplyOperation {
    StructuredFamilyDraftRenderedRecordStagedApplyOperation {
        family: entry.family,
        record_index: entry.record_index,
        stage_kind,
        operation_kind: operation_kind.to_string(),
        original_raw_line: entry.original_raw_line.clone(),
        rendered_record_preview: entry.rendered_record_preview.clone(),
        field_diff_count: entry.field_diffs.len(),
        status,
        action_policy: StructuredFamilyDraftRenderedRecordStagedApplyStatus::ActionsDisabled,
        write_policy: StructuredFamilyDraftRenderedRecordStagedApplyStatus::WritesBlockedByDefault,
        persistence_policy:
            StructuredFamilyDraftRenderedRecordStagedApplyStatus::PersistenceForbidden,
        real_config_target_policy:
            StructuredFamilyDraftRenderedRecordStagedApplyStatus::RealConfigTargetForbidden,
        production_executor_policy:
            StructuredFamilyDraftRenderedRecordStagedApplyStatus::ProductionExecutorForbidden,
    }
}

pub fn structured_family_draft_rendered_record_staged_apply_dry_run_report(
    plan: &StructuredFamilyDraftRenderedRecordStagedApplyPlan,
) -> StructuredFamilyDraftRenderedRecordStagedApplyDryRunReport {
    let ready = plan.blockers.is_empty()
        && plan.staged_apply_status
            == StructuredFamilyDraftRenderedRecordStagedApplyStatus::PlanReady;
    let blocked_plan_count = usize::from(!ready);
    let entries = plan
        .operations
        .iter()
        .map(structured_family_draft_rendered_record_staged_apply_dry_run_entry)
        .collect::<Vec<_>>();
    let dry_run_report_status = if ready {
        StructuredFamilyDraftRenderedRecordStagedApplyDryRunStatus::ReportReady
    } else {
        StructuredFamilyDraftRenderedRecordStagedApplyDryRunStatus::BlockedPlanSummarized
    };

    StructuredFamilyDraftRenderedRecordStagedApplyDryRunReport {
        family: plan.family,
        source_draft_count: plan.source_draft_count,
        source_plan_count: plan.source_plan_count,
        review_entry_count: plan.review_entry_count,
        changed_entry_count: plan.changed_entry_count,
        noop_entry_count: plan.noop_entry_count,
        raw_fallback_entry_count: plan.raw_fallback_entry_count,
        unsupported_not_proven_entry_count: plan.unsupported_not_proven_entry_count,
        field_diff_count: plan.field_diff_count,
        staged_apply_plan_linked: true,
        stage_count: plan.stage_count,
        operation_count: plan.operation_count,
        changed_operation_count: plan.changed_operation_count,
        noop_operation_count: plan.noop_operation_count,
        raw_fallback_preservation_operation_count: plan
            .raw_fallback_preservation_operation_count,
        unsupported_not_proven_preservation_operation_count: plan
            .unsupported_not_proven_preservation_operation_count,
        blocked_plan_count,
        executor_unavailable_by_design: true,
        dry_run_report_status,
        plan_status: StructuredFamilyDraftRenderedRecordStagedApplyDryRunStatus::PlanLinked,
        stage_summary_status:
            StructuredFamilyDraftRenderedRecordStagedApplyDryRunStatus::StagesSummarized,
        operation_summary_status:
            StructuredFamilyDraftRenderedRecordStagedApplyDryRunStatus::OperationsSummarized,
        changed_operation_summary_status:
            StructuredFamilyDraftRenderedRecordStagedApplyDryRunStatus::ChangedOperationsSummarized,
        noop_operation_summary_status:
            StructuredFamilyDraftRenderedRecordStagedApplyDryRunStatus::NoopOperationsSummarized,
        raw_fallback_preservation_summary_status:
            StructuredFamilyDraftRenderedRecordStagedApplyDryRunStatus::RawFallbackPreserved,
        unsupported_not_proven_preservation_summary_status:
            StructuredFamilyDraftRenderedRecordStagedApplyDryRunStatus::UnsupportedNotProvenPreserved,
        blocked_plan_summary_status:
            StructuredFamilyDraftRenderedRecordStagedApplyDryRunStatus::BlockedPlanSummarized,
        executor_availability_status:
            StructuredFamilyDraftRenderedRecordStagedApplyDryRunStatus::ExecutorUnavailable,
        dry_run_execution_status:
            StructuredFamilyDraftRenderedRecordStagedApplyDryRunStatus::NotExecuted,
        entries,
        summary_text: format!(
            "{} dry-run report generated for fixture-only report review only: {} stages, {} operations, {} blockers; dry-run executed false and staged apply executed false.",
            plan.family.family_id(),
            plan.stage_count,
            plan.operation_count,
            plan.blockers.len()
        ),
        risk_summary:
            "dry-run report is review-only and in memory; executor unavailable by design; real config writes, persistence, staged apply execution, dry-run execution, runtime mutation, reload, and production executors remain blocked"
                .to_string(),
        fixture_only_status:
            StructuredFamilyDraftRenderedRecordStagedApplyDryRunStatus::FixtureOnly,
        action_policy: StructuredFamilyDraftRenderedRecordStagedApplyDryRunStatus::ActionsDisabled,
        write_policy:
            StructuredFamilyDraftRenderedRecordStagedApplyDryRunStatus::WritesBlockedByDefault,
        persistence_policy:
            StructuredFamilyDraftRenderedRecordStagedApplyDryRunStatus::PersistenceForbidden,
        real_config_target_policy:
            StructuredFamilyDraftRenderedRecordStagedApplyDryRunStatus::RealConfigTargetForbidden,
        production_executor_policy:
            StructuredFamilyDraftRenderedRecordStagedApplyDryRunStatus::ProductionExecutorForbidden,
        blockers: plan.blockers.clone(),
        draft_written_to_disk: false,
        dry_run_report_written_to_disk: false,
        staged_apply_plan_written_to_disk: false,
        staged_apply_executed: false,
        dry_run_executed: false,
        rendered_record_written_to_real_config: false,
        real_config_touched: false,
        runtime_mutated: false,
        hyprctl_reload_run: false,
        production_executor_wired: false,
    }
}

fn structured_family_draft_rendered_record_staged_apply_dry_run_entry(
    operation: &StructuredFamilyDraftRenderedRecordStagedApplyOperation,
) -> StructuredFamilyDraftRenderedRecordStagedApplyDryRunEntry {
    StructuredFamilyDraftRenderedRecordStagedApplyDryRunEntry {
        family: operation.family,
        record_index: operation.record_index,
        stage_kind: operation.stage_kind,
        operation_kind: operation.operation_kind.clone(),
        original_raw_line: operation.original_raw_line.clone(),
        rendered_record_preview: operation.rendered_record_preview.clone(),
        status: match operation.status {
            StructuredFamilyDraftRenderedRecordStagedApplyStatus::RawFallbackPreserved => {
                StructuredFamilyDraftRenderedRecordStagedApplyDryRunStatus::RawFallbackPreserved
            }
            StructuredFamilyDraftRenderedRecordStagedApplyStatus::UnsupportedNotProvenPreserved => {
                StructuredFamilyDraftRenderedRecordStagedApplyDryRunStatus::UnsupportedNotProvenPreserved
            }
            _ if operation.operation_kind == "changed review operation" => {
                StructuredFamilyDraftRenderedRecordStagedApplyDryRunStatus::ChangedOperationsSummarized
            }
            _ if operation.operation_kind == "noop preservation operation" => {
                StructuredFamilyDraftRenderedRecordStagedApplyDryRunStatus::NoopOperationsSummarized
            }
            _ => StructuredFamilyDraftRenderedRecordStagedApplyDryRunStatus::OperationsSummarized,
        },
        action_policy: StructuredFamilyDraftRenderedRecordStagedApplyDryRunStatus::ActionsDisabled,
        write_policy:
            StructuredFamilyDraftRenderedRecordStagedApplyDryRunStatus::WritesBlockedByDefault,
        persistence_policy:
            StructuredFamilyDraftRenderedRecordStagedApplyDryRunStatus::PersistenceForbidden,
        real_config_target_policy:
            StructuredFamilyDraftRenderedRecordStagedApplyDryRunStatus::RealConfigTargetForbidden,
        production_executor_policy:
            StructuredFamilyDraftRenderedRecordStagedApplyDryRunStatus::ProductionExecutorForbidden,
    }
}

pub fn structured_family_draft_rendered_record_staged_apply_rollback_recovery_review(
    dry_run: &StructuredFamilyDraftRenderedRecordStagedApplyDryRunReport,
) -> StructuredFamilyDraftRenderedRecordStagedApplyRollbackRecoveryReview {
    let recovery_requirements =
        structured_family_draft_rendered_record_rollback_recovery_requirements(dry_run);
    let recovery_blockers =
        structured_family_draft_rendered_record_rollback_recovery_blockers(dry_run);
    let ready = recovery_blockers.is_empty()
        && dry_run.blockers.is_empty()
        && dry_run.dry_run_report_status
            == StructuredFamilyDraftRenderedRecordStagedApplyDryRunStatus::ReportReady;
    let review_status = if ready {
        StructuredFamilyDraftRenderedRecordStagedApplyRollbackRecoveryStatus::ReviewReady
    } else {
        StructuredFamilyDraftRenderedRecordStagedApplyRollbackRecoveryStatus::BlockedPlanPreserved
    };
    let entries = dry_run
        .entries
        .iter()
        .map(structured_family_draft_rendered_record_staged_apply_rollback_recovery_entry)
        .collect::<Vec<_>>();
    let backup_requirement_count = recovery_requirements
        .iter()
        .filter(|requirement| {
            **requirement
                == StructuredFamilyDraftRenderedRecordRollbackRecoveryRequirement::BackupRequiredBeforeFutureApply
        })
        .count();
    let restore_requirement_count = recovery_requirements
        .iter()
        .filter(|requirement| {
            **requirement
                == StructuredFamilyDraftRenderedRecordRollbackRecoveryRequirement::RestoreRequiredBeforeFutureRecovery
        })
        .count();

    StructuredFamilyDraftRenderedRecordStagedApplyRollbackRecoveryReview {
        family: dry_run.family,
        source_draft_count: dry_run.source_draft_count,
        source_plan_count: dry_run.source_plan_count,
        review_entry_count: dry_run.review_entry_count,
        changed_entry_count: dry_run.changed_entry_count,
        noop_entry_count: dry_run.noop_entry_count,
        raw_fallback_entry_count: dry_run.raw_fallback_entry_count,
        unsupported_not_proven_entry_count: dry_run.unsupported_not_proven_entry_count,
        field_diff_count: dry_run.field_diff_count,
        dry_run_report_linked: true,
        staged_apply_plan_linked: dry_run.staged_apply_plan_linked,
        stage_count: dry_run.stage_count,
        operation_count: dry_run.operation_count,
        changed_operation_count: dry_run.changed_operation_count,
        noop_operation_count: dry_run.noop_operation_count,
        raw_fallback_preservation_operation_count: dry_run
            .raw_fallback_preservation_operation_count,
        unsupported_not_proven_preservation_operation_count: dry_run
            .unsupported_not_proven_preservation_operation_count,
        blocked_plan_count: dry_run.blocked_plan_count,
        rollback_review_entry_count: entries.len(),
        recovery_requirement_count: recovery_requirements.len(),
        backup_requirement_count,
        restore_requirement_count,
        blocked_recovery_reason_count: recovery_blockers.len(),
        executor_unavailable_by_design: true,
        rollback_recovery_review_status: review_status,
        dry_run_link_status:
            StructuredFamilyDraftRenderedRecordStagedApplyRollbackRecoveryStatus::DryRunLinked,
        staged_apply_plan_link_status:
            StructuredFamilyDraftRenderedRecordStagedApplyRollbackRecoveryStatus::PlanLinked,
        rollback_plan_summary_status:
            StructuredFamilyDraftRenderedRecordStagedApplyRollbackRecoveryStatus::RollbackPlanSummarized,
        recovery_path_summary_status:
            StructuredFamilyDraftRenderedRecordStagedApplyRollbackRecoveryStatus::RecoveryPathSummarized,
        backup_requirement_status:
            StructuredFamilyDraftRenderedRecordStagedApplyRollbackRecoveryStatus::BackupRequirementReady,
        restore_requirement_status:
            StructuredFamilyDraftRenderedRecordStagedApplyRollbackRecoveryStatus::RestoreRequirementReady,
        blocked_plan_preservation_status:
            StructuredFamilyDraftRenderedRecordStagedApplyRollbackRecoveryStatus::BlockedPlanPreserved,
        executor_availability_status:
            StructuredFamilyDraftRenderedRecordStagedApplyRollbackRecoveryStatus::ExecutorUnavailable,
        execution_status:
            StructuredFamilyDraftRenderedRecordStagedApplyRollbackRecoveryStatus::NotExecuted,
        summary_text: format!(
            "{} rollback/recovery review generated for fixture-only recovery review only: {} operations, {} recovery requirements, {} blockers; rollback executed false, recovery executed false, backup created false, restore executed false, staged apply executed false, and dry-run executed false.",
            dry_run.family.family_id(),
            dry_run.operation_count,
            recovery_requirements.len(),
            recovery_blockers.len()
        ),
        risk_summary:
            "rollback/recovery review is review-only and in memory; backup and restore requirements are represented for future review only; executor unavailable by design; real config writes, persistence, staged apply execution, dry-run execution, rollback execution, recovery execution, runtime mutation, reload, and production executors remain blocked"
                .to_string(),
        recovery_requirements,
        entries,
        fixture_only_status:
            StructuredFamilyDraftRenderedRecordStagedApplyRollbackRecoveryStatus::FixtureOnly,
        action_policy:
            StructuredFamilyDraftRenderedRecordStagedApplyRollbackRecoveryStatus::ActionsDisabled,
        write_policy:
            StructuredFamilyDraftRenderedRecordStagedApplyRollbackRecoveryStatus::WritesBlockedByDefault,
        persistence_policy:
            StructuredFamilyDraftRenderedRecordStagedApplyRollbackRecoveryStatus::PersistenceForbidden,
        real_config_target_policy:
            StructuredFamilyDraftRenderedRecordStagedApplyRollbackRecoveryStatus::RealConfigTargetForbidden,
        production_executor_policy:
            StructuredFamilyDraftRenderedRecordStagedApplyRollbackRecoveryStatus::ProductionExecutorForbidden,
        staged_apply_blockers: dry_run.blockers.clone(),
        recovery_blockers,
        draft_written_to_disk: false,
        rollback_recovery_review_written_to_disk: false,
        dry_run_report_written_to_disk: false,
        staged_apply_plan_written_to_disk: false,
        staged_apply_executed: false,
        dry_run_executed: false,
        rollback_executed: false,
        recovery_executed: false,
        backup_created: false,
        restore_executed: false,
        rendered_record_written_to_real_config: false,
        real_config_touched: false,
        runtime_mutated: false,
        hyprctl_reload_run: false,
        production_executor_wired: false,
    }
}

fn structured_family_draft_rendered_record_rollback_recovery_requirements(
    dry_run: &StructuredFamilyDraftRenderedRecordStagedApplyDryRunReport,
) -> Vec<StructuredFamilyDraftRenderedRecordRollbackRecoveryRequirement> {
    let mut requirements = vec![
        StructuredFamilyDraftRenderedRecordRollbackRecoveryRequirement::BackupRequiredBeforeFutureApply,
        StructuredFamilyDraftRenderedRecordRollbackRecoveryRequirement::RestoreRequiredBeforeFutureRecovery,
        StructuredFamilyDraftRenderedRecordRollbackRecoveryRequirement::ReloadForbiddenInCurrentSprint,
        StructuredFamilyDraftRenderedRecordRollbackRecoveryRequirement::RuntimeMutationForbiddenInCurrentSprint,
        StructuredFamilyDraftRenderedRecordRollbackRecoveryRequirement::RealConfigTargetForbiddenInCurrentSprint,
        StructuredFamilyDraftRenderedRecordRollbackRecoveryRequirement::ProductionExecutorForbiddenInCurrentSprint,
        StructuredFamilyDraftRenderedRecordRollbackRecoveryRequirement::FixtureOnlyReviewRequired,
        StructuredFamilyDraftRenderedRecordRollbackRecoveryRequirement::DryRunMustRemainNotExecuted,
        StructuredFamilyDraftRenderedRecordRollbackRecoveryRequirement::StagedApplyMustRemainNotExecuted,
    ];
    if dry_run.raw_fallback_entry_count > 0 {
        requirements.push(
            StructuredFamilyDraftRenderedRecordRollbackRecoveryRequirement::RawFallbackRequiresPreservation,
        );
    }
    if dry_run.unsupported_not_proven_entry_count > 0 {
        requirements.push(
            StructuredFamilyDraftRenderedRecordRollbackRecoveryRequirement::UnsupportedNotProvenRequiresPreservation,
        );
    }
    requirements
}

fn structured_family_draft_rendered_record_rollback_recovery_blockers(
    dry_run: &StructuredFamilyDraftRenderedRecordStagedApplyDryRunReport,
) -> Vec<StructuredFamilyDraftRenderedRecordRollbackRecoveryBlocker> {
    let mut blockers = Vec::new();
    if !dry_run.staged_apply_plan_linked {
        blockers.push(
            StructuredFamilyDraftRenderedRecordRollbackRecoveryBlocker::MissingStagedApplyPlanLink,
        );
    }
    if dry_run.blocked_plan_count > 0
        || dry_run.dry_run_report_status
            == StructuredFamilyDraftRenderedRecordStagedApplyDryRunStatus::BlockedPlanSummarized
        || !dry_run.blockers.is_empty()
    {
        blockers.push(
            StructuredFamilyDraftRenderedRecordRollbackRecoveryBlocker::BlockedStagedApplyPlan,
        );
    }
    if dry_run.real_config_target_policy
        != StructuredFamilyDraftRenderedRecordStagedApplyDryRunStatus::RealConfigTargetForbidden
    {
        blockers.push(
            StructuredFamilyDraftRenderedRecordRollbackRecoveryBlocker::RealConfigTargetNotAllowed,
        );
    }
    if dry_run.persistence_policy
        != StructuredFamilyDraftRenderedRecordStagedApplyDryRunStatus::PersistenceForbidden
    {
        blockers.push(
            StructuredFamilyDraftRenderedRecordRollbackRecoveryBlocker::PersistenceNotAllowed,
        );
    }
    if dry_run.runtime_mutated {
        blockers.push(
            StructuredFamilyDraftRenderedRecordRollbackRecoveryBlocker::RuntimeMutationNotAllowed,
        );
    }
    if dry_run.hyprctl_reload_run {
        blockers.push(
            StructuredFamilyDraftRenderedRecordRollbackRecoveryBlocker::HyprlandReloadNotAllowed,
        );
    }
    if dry_run.production_executor_policy
        != StructuredFamilyDraftRenderedRecordStagedApplyDryRunStatus::ProductionExecutorForbidden
        || dry_run.production_executor_wired
    {
        blockers.push(
            StructuredFamilyDraftRenderedRecordRollbackRecoveryBlocker::ProductionExecutorNotAllowed,
        );
    }
    if dry_run.staged_apply_executed {
        blockers.push(
            StructuredFamilyDraftRenderedRecordRollbackRecoveryBlocker::StagedApplyAlreadyExecuted,
        );
    }
    if dry_run.dry_run_executed {
        blockers.push(
            StructuredFamilyDraftRenderedRecordRollbackRecoveryBlocker::DryRunAlreadyExecuted,
        );
    }
    blockers
}

fn structured_family_draft_rendered_record_staged_apply_rollback_recovery_entry(
    entry: &StructuredFamilyDraftRenderedRecordStagedApplyDryRunEntry,
) -> StructuredFamilyDraftRenderedRecordStagedApplyRollbackRecoveryEntry {
    StructuredFamilyDraftRenderedRecordStagedApplyRollbackRecoveryEntry {
        family: entry.family,
        record_index: entry.record_index,
        stage_kind: entry.stage_kind,
        operation_kind: entry.operation_kind.clone(),
        original_raw_line: entry.original_raw_line.clone(),
        rendered_record_preview: entry.rendered_record_preview.clone(),
        status: match entry.status {
            StructuredFamilyDraftRenderedRecordStagedApplyDryRunStatus::RawFallbackPreserved => {
                StructuredFamilyDraftRenderedRecordStagedApplyRollbackRecoveryStatus::BlockedPlanPreserved
            }
            StructuredFamilyDraftRenderedRecordStagedApplyDryRunStatus::UnsupportedNotProvenPreserved => {
                StructuredFamilyDraftRenderedRecordStagedApplyRollbackRecoveryStatus::BlockedPlanPreserved
            }
            _ => StructuredFamilyDraftRenderedRecordStagedApplyRollbackRecoveryStatus::RecoveryPathSummarized,
        },
        action_policy:
            StructuredFamilyDraftRenderedRecordStagedApplyRollbackRecoveryStatus::ActionsDisabled,
        write_policy:
            StructuredFamilyDraftRenderedRecordStagedApplyRollbackRecoveryStatus::WritesBlockedByDefault,
        persistence_policy:
            StructuredFamilyDraftRenderedRecordStagedApplyRollbackRecoveryStatus::PersistenceForbidden,
        real_config_target_policy:
            StructuredFamilyDraftRenderedRecordStagedApplyRollbackRecoveryStatus::RealConfigTargetForbidden,
        production_executor_policy:
            StructuredFamilyDraftRenderedRecordStagedApplyRollbackRecoveryStatus::ProductionExecutorForbidden,
    }
}

pub fn structured_family_draft_rendered_record_final_executor_readiness_audit(
    review: &StructuredFamilyDraftRenderedRecordStagedApplyRollbackRecoveryReview,
) -> StructuredFamilyDraftRenderedRecordFinalExecutorReadinessAudit {
    structured_family_draft_rendered_record_final_executor_readiness_audit_with_links(
        review,
        true,
        review.dry_run_report_linked,
        review.staged_apply_plan_linked,
    )
}

pub fn structured_family_draft_rendered_record_final_executor_readiness_audit_with_links(
    review: &StructuredFamilyDraftRenderedRecordStagedApplyRollbackRecoveryReview,
    rollback_recovery_review_linked: bool,
    dry_run_report_linked: bool,
    staged_apply_plan_linked: bool,
) -> StructuredFamilyDraftRenderedRecordFinalExecutorReadinessAudit {
    let blockers =
        structured_family_draft_rendered_record_final_executor_readiness_blockers(review);
    let mut blockers = blockers;
    if !rollback_recovery_review_linked {
        blockers.push(
            StructuredFamilyDraftRenderedRecordFinalExecutorReadinessBlocker::MissingRollbackRecoveryLink,
        );
    }
    if !dry_run_report_linked {
        blockers.push(
            StructuredFamilyDraftRenderedRecordFinalExecutorReadinessBlocker::MissingDryRunLink,
        );
    }
    if !staged_apply_plan_linked {
        blockers.push(
            StructuredFamilyDraftRenderedRecordFinalExecutorReadinessBlocker::MissingStagedApplyPlanLink,
        );
    }
    let ready = blockers.is_empty()
        && review.recovery_blockers.is_empty()
        && review.rollback_recovery_review_status
            == StructuredFamilyDraftRenderedRecordStagedApplyRollbackRecoveryStatus::ReviewReady;
    let mut findings =
        structured_family_draft_rendered_record_final_executor_readiness_findings(review);
    if !ready || review.blocked_plan_count > 0 {
        findings.push(
            StructuredFamilyDraftRenderedRecordFinalExecutorReadinessFinding::BlockedPlanPreserved,
        );
    }
    let entries = review
        .entries
        .iter()
        .map(structured_family_draft_rendered_record_final_executor_readiness_entry)
        .collect::<Vec<_>>();
    let production_blocker_count = findings
        .iter()
        .filter(|finding| {
            matches!(
                finding,
                StructuredFamilyDraftRenderedRecordFinalExecutorReadinessFinding::ProductionActivationRequired
                    | StructuredFamilyDraftRenderedRecordFinalExecutorReadinessFinding::ExecutorNotImplemented
                    | StructuredFamilyDraftRenderedRecordFinalExecutorReadinessFinding::ExecutorNotWired
                    | StructuredFamilyDraftRenderedRecordFinalExecutorReadinessFinding::RealWritesBlocked
                    | StructuredFamilyDraftRenderedRecordFinalExecutorReadinessFinding::PersistenceBlocked
                    | StructuredFamilyDraftRenderedRecordFinalExecutorReadinessFinding::RuntimeMutationBlocked
                    | StructuredFamilyDraftRenderedRecordFinalExecutorReadinessFinding::HyprlandReloadBlocked
                    | StructuredFamilyDraftRenderedRecordFinalExecutorReadinessFinding::BackupImplementationMissing
                    | StructuredFamilyDraftRenderedRecordFinalExecutorReadinessFinding::RestoreImplementationMissing
                    | StructuredFamilyDraftRenderedRecordFinalExecutorReadinessFinding::RollbackExecutionMissing
                    | StructuredFamilyDraftRenderedRecordFinalExecutorReadinessFinding::RecoveryExecutionMissing
                    | StructuredFamilyDraftRenderedRecordFinalExecutorReadinessFinding::UserDecisionRequiredBeforeProduction
            )
        })
        .count();

    StructuredFamilyDraftRenderedRecordFinalExecutorReadinessAudit {
        family: review.family,
        source_draft_count: review.source_draft_count,
        source_plan_count: review.source_plan_count,
        review_entry_count: review.review_entry_count,
        changed_entry_count: review.changed_entry_count,
        noop_entry_count: review.noop_entry_count,
        raw_fallback_entry_count: review.raw_fallback_entry_count,
        unsupported_not_proven_entry_count: review.unsupported_not_proven_entry_count,
        field_diff_count: review.field_diff_count,
        rollback_recovery_review_linked,
        dry_run_report_linked,
        staged_apply_plan_linked,
        stage_count: review.stage_count,
        operation_count: review.operation_count,
        changed_operation_count: review.changed_operation_count,
        noop_operation_count: review.noop_operation_count,
        raw_fallback_preservation_operation_count: review
            .raw_fallback_preservation_operation_count,
        unsupported_not_proven_preservation_operation_count: review
            .unsupported_not_proven_preservation_operation_count,
        blocked_plan_count: review.blocked_plan_count,
        recovery_requirement_count: review.recovery_requirement_count,
        backup_requirement_count: review.backup_requirement_count,
        restore_requirement_count: review.restore_requirement_count,
        executor_unavailable_by_design: true,
        final_audit_finding_count: findings.len(),
        production_blocker_count,
        fixture_only_proof_count: 13,
        final_audit_status: if ready {
            StructuredFamilyDraftRenderedRecordFinalExecutorReadinessStatus::AuditReady
        } else {
            StructuredFamilyDraftRenderedRecordFinalExecutorReadinessStatus::BlockedPlanPreserved
        },
        rollback_recovery_link_status:
            if rollback_recovery_review_linked {
                StructuredFamilyDraftRenderedRecordFinalExecutorReadinessStatus::RollbackRecoveryLinked
            } else {
                StructuredFamilyDraftRenderedRecordFinalExecutorReadinessStatus::BlockedPlanPreserved
            },
        dry_run_link_status: if dry_run_report_linked {
            StructuredFamilyDraftRenderedRecordFinalExecutorReadinessStatus::DryRunLinked
        } else {
            StructuredFamilyDraftRenderedRecordFinalExecutorReadinessStatus::BlockedPlanPreserved
        },
        staged_apply_plan_link_status: if staged_apply_plan_linked {
            StructuredFamilyDraftRenderedRecordFinalExecutorReadinessStatus::StagedApplyLinked
        } else {
            StructuredFamilyDraftRenderedRecordFinalExecutorReadinessStatus::BlockedPlanPreserved
        },
        proof_chain_status: if ready {
            StructuredFamilyDraftRenderedRecordFinalExecutorReadinessStatus::ProofChainComplete
        } else {
            StructuredFamilyDraftRenderedRecordFinalExecutorReadinessStatus::BlockedPlanPreserved
        },
        fixture_only_completion_status:
            StructuredFamilyDraftRenderedRecordFinalExecutorReadinessStatus::FixtureOnlyComplete,
        production_activation_requirement_status:
            StructuredFamilyDraftRenderedRecordFinalExecutorReadinessStatus::ProductionActivationRequired,
        executor_implementation_status:
            StructuredFamilyDraftRenderedRecordFinalExecutorReadinessStatus::ExecutorNotImplemented,
        executor_wiring_status:
            StructuredFamilyDraftRenderedRecordFinalExecutorReadinessStatus::ExecutorNotWired,
        real_write_boundary_status:
            StructuredFamilyDraftRenderedRecordFinalExecutorReadinessStatus::RealWritesBlocked,
        runtime_mutation_boundary_status:
            StructuredFamilyDraftRenderedRecordFinalExecutorReadinessStatus::RuntimeMutationBlocked,
        reload_boundary_status:
            StructuredFamilyDraftRenderedRecordFinalExecutorReadinessStatus::ReloadBlocked,
        backup_restore_implementation_status:
            StructuredFamilyDraftRenderedRecordFinalExecutorReadinessStatus::BackupRestoreNotImplemented,
        blocked_plan_preservation_status:
            StructuredFamilyDraftRenderedRecordFinalExecutorReadinessStatus::BlockedPlanPreserved,
        production_readiness_decision:
            StructuredFamilyDraftRenderedRecordFinalExecutorReadinessDecision::NotProductionReady,
        summary_text: format!(
            "{} final executor-readiness audit generated: true; fixture-only pipeline complete: {}; production activation required: true; production activation approved: false; executor implemented: false; executor wired: false; real config writes remain blocked.",
            review.family.family_id(),
            ready
        ),
        risk_summary:
            "fixture-only readiness does not imply production readiness; explicit user decision, production activation review, executor implementation, executor wiring, backup/restore implementation, and live recovery policy remain required before any real structured-family write path"
                .to_string(),
        findings,
        entries,
        blockers,
        fixture_only_status:
            StructuredFamilyDraftRenderedRecordFinalExecutorReadinessStatus::FixtureOnly,
        action_policy:
            StructuredFamilyDraftRenderedRecordFinalExecutorReadinessStatus::ActionsDisabled,
        write_policy:
            StructuredFamilyDraftRenderedRecordFinalExecutorReadinessStatus::WritesBlockedByDefault,
        persistence_policy:
            StructuredFamilyDraftRenderedRecordFinalExecutorReadinessStatus::PersistenceForbidden,
        real_config_target_policy:
            StructuredFamilyDraftRenderedRecordFinalExecutorReadinessStatus::RealConfigTargetForbidden,
        production_executor_policy:
            StructuredFamilyDraftRenderedRecordFinalExecutorReadinessStatus::ProductionExecutorForbidden,
        draft_written_to_disk: false,
        final_audit_written_to_disk: false,
        rollback_recovery_review_written_to_disk: false,
        dry_run_report_written_to_disk: false,
        staged_apply_plan_written_to_disk: false,
        staged_apply_executed: false,
        dry_run_executed: false,
        rollback_executed: false,
        recovery_executed: false,
        backup_created: false,
        restore_executed: false,
        executor_implemented: false,
        executor_wired: false,
        production_activation_approved: false,
        rendered_record_written_to_real_config: false,
        real_config_touched: false,
        runtime_mutated: false,
        hyprctl_reload_run: false,
        production_behavior_enabled: false,
        production_executor_wired: false,
    }
}

pub fn structured_family_draft_rendered_record_final_executor_readiness_audit_blockers(
    audit: &StructuredFamilyDraftRenderedRecordFinalExecutorReadinessAudit,
) -> Vec<StructuredFamilyDraftRenderedRecordFinalExecutorReadinessBlocker> {
    let mut blockers = Vec::new();
    if !audit.rollback_recovery_review_linked {
        blockers.push(
            StructuredFamilyDraftRenderedRecordFinalExecutorReadinessBlocker::MissingRollbackRecoveryLink,
        );
    }
    if !audit.dry_run_report_linked {
        blockers.push(
            StructuredFamilyDraftRenderedRecordFinalExecutorReadinessBlocker::MissingDryRunLink,
        );
    }
    if !audit.staged_apply_plan_linked {
        blockers.push(
            StructuredFamilyDraftRenderedRecordFinalExecutorReadinessBlocker::MissingStagedApplyPlanLink,
        );
    }
    if audit.blocked_plan_count > 0
        || audit.final_audit_status
            == StructuredFamilyDraftRenderedRecordFinalExecutorReadinessStatus::BlockedPlanPreserved
    {
        blockers.push(
            StructuredFamilyDraftRenderedRecordFinalExecutorReadinessBlocker::BlockedStagedApplyPlan,
        );
    }
    if audit.real_config_target_policy
        != StructuredFamilyDraftRenderedRecordFinalExecutorReadinessStatus::RealConfigTargetForbidden
    {
        blockers.push(
            StructuredFamilyDraftRenderedRecordFinalExecutorReadinessBlocker::RealConfigTargetNotAllowed,
        );
    }
    if audit.persistence_policy
        != StructuredFamilyDraftRenderedRecordFinalExecutorReadinessStatus::PersistenceForbidden
    {
        blockers.push(
            StructuredFamilyDraftRenderedRecordFinalExecutorReadinessBlocker::PersistenceNotAllowed,
        );
    }
    if audit.runtime_mutated {
        blockers.push(
            StructuredFamilyDraftRenderedRecordFinalExecutorReadinessBlocker::RuntimeMutationNotAllowed,
        );
    }
    if audit.hyprctl_reload_run {
        blockers.push(
            StructuredFamilyDraftRenderedRecordFinalExecutorReadinessBlocker::HyprlandReloadNotAllowed,
        );
    }
    if audit.production_executor_policy
        != StructuredFamilyDraftRenderedRecordFinalExecutorReadinessStatus::ProductionExecutorForbidden
        || audit.production_executor_wired
    {
        blockers.push(
            StructuredFamilyDraftRenderedRecordFinalExecutorReadinessBlocker::ProductionExecutorNotAllowed,
        );
    }
    if audit.production_activation_approved {
        blockers.push(
            StructuredFamilyDraftRenderedRecordFinalExecutorReadinessBlocker::ProductionActivationNotAllowed,
        );
    }
    if audit.executor_implemented {
        blockers.push(
            StructuredFamilyDraftRenderedRecordFinalExecutorReadinessBlocker::ExecutorImplementationNotAllowed,
        );
    }
    if audit.executor_wired {
        blockers.push(
            StructuredFamilyDraftRenderedRecordFinalExecutorReadinessBlocker::ExecutorWiringNotAllowed,
        );
    }
    blockers
}

fn structured_family_draft_rendered_record_final_executor_readiness_findings(
    review: &StructuredFamilyDraftRenderedRecordStagedApplyRollbackRecoveryReview,
) -> Vec<StructuredFamilyDraftRenderedRecordFinalExecutorReadinessFinding> {
    let mut findings = vec![
        StructuredFamilyDraftRenderedRecordFinalExecutorReadinessFinding::FixturePipelineComplete,
        StructuredFamilyDraftRenderedRecordFinalExecutorReadinessFinding::ProductionActivationRequired,
        StructuredFamilyDraftRenderedRecordFinalExecutorReadinessFinding::ExecutorNotImplemented,
        StructuredFamilyDraftRenderedRecordFinalExecutorReadinessFinding::ExecutorNotWired,
        StructuredFamilyDraftRenderedRecordFinalExecutorReadinessFinding::RealWritesBlocked,
        StructuredFamilyDraftRenderedRecordFinalExecutorReadinessFinding::PersistenceBlocked,
        StructuredFamilyDraftRenderedRecordFinalExecutorReadinessFinding::RuntimeMutationBlocked,
        StructuredFamilyDraftRenderedRecordFinalExecutorReadinessFinding::HyprlandReloadBlocked,
        StructuredFamilyDraftRenderedRecordFinalExecutorReadinessFinding::BackupImplementationMissing,
        StructuredFamilyDraftRenderedRecordFinalExecutorReadinessFinding::RestoreImplementationMissing,
        StructuredFamilyDraftRenderedRecordFinalExecutorReadinessFinding::RollbackExecutionMissing,
        StructuredFamilyDraftRenderedRecordFinalExecutorReadinessFinding::RecoveryExecutionMissing,
        StructuredFamilyDraftRenderedRecordFinalExecutorReadinessFinding::SourceTargetPolicyStillForbidden,
        StructuredFamilyDraftRenderedRecordFinalExecutorReadinessFinding::UserDecisionRequiredBeforeProduction,
        StructuredFamilyDraftRenderedRecordFinalExecutorReadinessFinding::Hyprland0554MigrationNotActive,
    ];
    if review.raw_fallback_entry_count > 0 {
        findings.push(
            StructuredFamilyDraftRenderedRecordFinalExecutorReadinessFinding::RawFallbackPreservationRequired,
        );
    }
    if review.unsupported_not_proven_entry_count > 0 {
        findings.push(
            StructuredFamilyDraftRenderedRecordFinalExecutorReadinessFinding::UnsupportedNotProvenPreservationRequired,
        );
    }
    findings
}

fn structured_family_draft_rendered_record_final_executor_readiness_blockers(
    review: &StructuredFamilyDraftRenderedRecordStagedApplyRollbackRecoveryReview,
) -> Vec<StructuredFamilyDraftRenderedRecordFinalExecutorReadinessBlocker> {
    let mut blockers = Vec::new();
    if !review.dry_run_report_linked {
        blockers.push(
            StructuredFamilyDraftRenderedRecordFinalExecutorReadinessBlocker::MissingDryRunLink,
        );
    }
    if !review.staged_apply_plan_linked {
        blockers.push(
            StructuredFamilyDraftRenderedRecordFinalExecutorReadinessBlocker::MissingStagedApplyPlanLink,
        );
    }
    if review.blocked_plan_count > 0
        || !review.recovery_blockers.is_empty()
        || review.rollback_recovery_review_status
            == StructuredFamilyDraftRenderedRecordStagedApplyRollbackRecoveryStatus::BlockedPlanPreserved
    {
        blockers.push(
            StructuredFamilyDraftRenderedRecordFinalExecutorReadinessBlocker::BlockedStagedApplyPlan,
        );
    }
    if review.real_config_target_policy
        != StructuredFamilyDraftRenderedRecordStagedApplyRollbackRecoveryStatus::RealConfigTargetForbidden
    {
        blockers.push(
            StructuredFamilyDraftRenderedRecordFinalExecutorReadinessBlocker::RealConfigTargetNotAllowed,
        );
    }
    if review.persistence_policy
        != StructuredFamilyDraftRenderedRecordStagedApplyRollbackRecoveryStatus::PersistenceForbidden
    {
        blockers.push(
            StructuredFamilyDraftRenderedRecordFinalExecutorReadinessBlocker::PersistenceNotAllowed,
        );
    }
    if review.runtime_mutated {
        blockers.push(
            StructuredFamilyDraftRenderedRecordFinalExecutorReadinessBlocker::RuntimeMutationNotAllowed,
        );
    }
    if review.hyprctl_reload_run {
        blockers.push(
            StructuredFamilyDraftRenderedRecordFinalExecutorReadinessBlocker::HyprlandReloadNotAllowed,
        );
    }
    if review.production_executor_policy
        != StructuredFamilyDraftRenderedRecordStagedApplyRollbackRecoveryStatus::ProductionExecutorForbidden
        || review.production_executor_wired
    {
        blockers.push(
            StructuredFamilyDraftRenderedRecordFinalExecutorReadinessBlocker::ProductionExecutorNotAllowed,
        );
    }
    blockers
}

fn structured_family_draft_rendered_record_final_executor_readiness_entry(
    entry: &StructuredFamilyDraftRenderedRecordStagedApplyRollbackRecoveryEntry,
) -> StructuredFamilyDraftRenderedRecordFinalExecutorReadinessEntry {
    StructuredFamilyDraftRenderedRecordFinalExecutorReadinessEntry {
        family: entry.family,
        record_index: entry.record_index,
        original_raw_line: entry.original_raw_line.clone(),
        rendered_record_preview: entry.rendered_record_preview.clone(),
        findings: vec![
            StructuredFamilyDraftRenderedRecordFinalExecutorReadinessFinding::RealWritesBlocked,
            StructuredFamilyDraftRenderedRecordFinalExecutorReadinessFinding::ExecutorNotImplemented,
            StructuredFamilyDraftRenderedRecordFinalExecutorReadinessFinding::ExecutorNotWired,
        ],
        status: StructuredFamilyDraftRenderedRecordFinalExecutorReadinessStatus::NotProductionReady,
        action_policy: StructuredFamilyDraftRenderedRecordFinalExecutorReadinessStatus::ActionsDisabled,
        write_policy:
            StructuredFamilyDraftRenderedRecordFinalExecutorReadinessStatus::WritesBlockedByDefault,
        persistence_policy:
            StructuredFamilyDraftRenderedRecordFinalExecutorReadinessStatus::PersistenceForbidden,
        real_config_target_policy:
            StructuredFamilyDraftRenderedRecordFinalExecutorReadinessStatus::RealConfigTargetForbidden,
        production_executor_policy:
            StructuredFamilyDraftRenderedRecordFinalExecutorReadinessStatus::ProductionExecutorForbidden,
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

pub fn structured_family_real_write_activation_requirements_audit(
) -> StructuredFamilyRealWriteActivationRequirementsAudit {
    StructuredFamilyRealWriteActivationRequirementsAudit {
        user_instruction:
            "Force Codex to list exactly what real-write activation would require, what backup/restore proof is missing, and what user approval gates must exist before a single real config write is allowed.",
        excluded_by_user: vec![
            "which families are safest",
            "which families should stay blocked",
            "family-by-family activation ranking",
            "activation subset recommendation",
        ],
        real_write_activation_requirements: vec![
            "explicit user production activation decision",
            "explicit activation scope document",
            "real config target policy",
            "source/include target policy",
            "atomic write strategy",
            "pre-write validation",
            "post-write reread validation",
            "Hyprland verify-config or equivalent validation strategy",
            "reload policy",
            "runtime mutation policy",
            "backup creation policy",
            "restore policy",
            "rollback policy",
            "failure recovery policy",
            "audit logging policy",
            "manual confirmation policy",
            "production executor implementation",
            "production executor wiring",
            "production executor tests",
            "UI confirmation gate",
            "dry-run to real-write transition gate",
            "blocked-plan rejection gate",
            "unsupported/not-proven preservation gate",
            "raw fallback preservation gate",
            "version compatibility gate for Hyprland 0.55.2 vs live 0.55.4",
        ],
        missing_backup_restore_proof: vec![
            "no real backup file creation proof",
            "no real backup location policy proof",
            "no backup integrity hash proof",
            "no backup reread proof",
            "no real restore execution proof",
            "no restore target validation proof",
            "no restore reread proof",
            "no rollback file creation proof",
            "no rollback execution proof",
            "no failed-write recovery proof",
            "no interrupted-write recovery proof",
            "no partial-write recovery proof",
            "no post-restore Hyprland validation proof",
            "no post-restore reload/restart policy proof",
            "no user-visible recovery instructions proof",
        ],
        required_user_approval_gates: vec![
            "approve entering production activation planning",
            "approve exact real-write activation scope",
            "approve config target path",
            "approve backup location and retention policy",
            "approve restore policy",
            "approve reload policy",
            "approve runtime mutation policy",
            "approve executor implementation",
            "approve executor wiring",
            "approve first real config write",
            "approve fallback behavior for unsupported/not-proven records",
            "approve raw fallback preservation behavior",
            "approve rollback procedure",
            "approve recovery procedure",
            "approve emergency stop procedure",
            "approve whether Hyprland 0.55.4 migration must happen before production activation",
        ],
        activation_status:
            StructuredFamilyRealWriteActivationAuditStatus::ProductionActivationDecisionRequired,
        executor_status: StructuredFamilyRealWriteActivationAuditStatus::ExecutorNotImplemented,
        real_write_boundary_status: StructuredFamilyRealWriteActivationAuditStatus::BlockedByDefault,
        backup_restore_boundary_status:
            StructuredFamilyRealWriteActivationAuditStatus::BackupRestoreProofMissing,
        reload_boundary_status:
            StructuredFamilyRealWriteActivationAuditStatus::HyprlandReloadForbidden,
        runtime_mutation_boundary_status:
            StructuredFamilyRealWriteActivationAuditStatus::RuntimeMutationForbidden,
        family_ranking_excluded:
            StructuredFamilyRealWriteActivationAuditStatus::FamilyRankingExcludedByUser,
        production_activation_approved: false,
        executor_implemented: false,
        executor_wired: false,
        real_write_path_enabled: false,
        real_config_target_enabled: false,
        backup_creation_enabled: false,
        restore_execution_enabled: false,
        rollback_execution_enabled: false,
        hyprctl_reload_enabled: false,
        runtime_mutation_enabled: false,
        first_real_config_write_approved: false,
        next_recommended_work:
            "Wait for explicit user approval of production activation planning scope before designing any real-write executor.",
    }
}

pub fn structured_family_production_activation_planning_scope(
) -> StructuredFamilyProductionActivationPlanningScope {
    StructuredFamilyProductionActivationPlanningScope {
        user_decision: "Option B: production activation planning scope only",
        planning_scope_approved: true,
        implementation_scope_approved: false,
        real_write_scope_approved: false,
        excluded_by_user: vec![
            "family safety ranking",
            "safest-family recommendation",
            "families that should stay blocked",
            "limited activation subset selection",
            "broad activation selection",
            "first family selection",
            "first record selection",
        ],
        approved_planning_scope: vec![
            "production activation planning document",
            "executor architecture design requirements",
            "backup and restore design requirements",
            "rollback and recovery design requirements",
            "validation evidence design requirements",
            "manual approval checkpoint design",
            "future implementation stop-gate design",
        ],
        not_approved_scope: vec![
            "executor implementation",
            "executor wiring",
            "real config writes",
            "real backup creation",
            "real restore execution",
            "rollback execution",
            "Hyprland reload",
            "runtime mutation",
            "first real config write",
            "family ranking",
            "activation subset selection",
        ],
        executor_architecture_planning_requirements: vec![
            "define executor boundaries without implementing an executor",
            "define explicit executor implementation approval gate",
            "define explicit executor wiring approval gate",
            "define no-auto-apply and no-implicit-activation invariants",
            "define source/include and duplicate production boundaries as separate scopes",
        ],
        backup_restore_planning_requirements: vec![
            "design backup location and retention policy",
            "design backup integrity hash proof",
            "design backup reread proof",
            "design restore target validation proof",
            "design restore reread proof",
            "design post-restore Hyprland validation policy",
        ],
        rollback_recovery_planning_requirements: vec![
            "design rollback file policy without creating rollback files",
            "design failed-write recovery procedure",
            "design interrupted-write recovery procedure",
            "design partial-write recovery procedure",
            "design emergency stop procedure",
            "design user-visible recovery instructions",
        ],
        validation_planning_requirements: vec![
            "define pre-write parser and validator evidence",
            "define fixture write/reread evidence",
            "define temporary config render and reread evidence",
            "define Hyprland verify-config or equivalent evidence",
            "define post-write reread evidence",
            "define Hyprland 0.55.2 model versus live 0.55.4 compatibility evidence",
        ],
        manual_approval_checkpoints: vec![
            "approve production activation planning document",
            "approve exact real-write activation scope",
            "approve executor implementation before implementation starts",
            "approve executor wiring before wiring starts",
            "approve backup and restore execution before execution starts",
            "approve reload policy before reload can be considered",
            "approve runtime mutation policy before runtime mutation can be considered",
            "approve first real config write before any real config write starts",
        ],
        future_implementation_stop_gates: vec![
            "A later sprint must stop before implementing an executor unless the user explicitly approves executor implementation.",
            "A later sprint must stop before wiring an executor unless the user explicitly approves executor wiring.",
            "A later sprint must stop before real config writes unless the user explicitly approves the first real config write.",
            "A later sprint must stop before backup/restore execution unless the user explicitly approves backup/restore execution.",
            "A later sprint must stop before Hyprland reload unless the user explicitly approves reload policy.",
            "A later sprint must stop before runtime mutation unless the user explicitly approves runtime mutation policy.",
        ],
        production_activation_approved: false,
        executor_implemented: false,
        executor_wired: false,
        real_write_path_enabled: false,
        real_config_target_enabled: false,
        backup_creation_enabled: false,
        restore_execution_enabled: false,
        rollback_execution_enabled: false,
        hyprctl_reload_enabled: false,
        runtime_mutation_enabled: false,
        first_real_config_write_approved: false,
        family_ranking_excluded: true,
        activation_subset_selected: false,
        production_readiness_decision: "not production ready",
        next_recommended_work:
            "Create a planning-only structured-family production activation design document that does not implement or wire an executor.",
    }
}

pub fn structured_family_kind_from_id(family_id: &str) -> Option<StructuredFamilyKind> {
    StructuredFamilyKind::from_family_id(family_id)
}
