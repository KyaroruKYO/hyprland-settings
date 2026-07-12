//! Runtime preview capability model.
//!
//! Classifies every one of the 341 scalar rows and all 7 structured families
//! for live runtime preview: whether the value can be applied to the running
//! compositor immediately (with revert), needs throttling, needs a dead-man
//! countdown, must go through a config write, or is blocked / not proven.
//!
//! Classification is evidence-based, not aspirational: the runtime mutation
//! mechanism (`hyprctl eval 'hl.config({ section = { option = value } })'`)
//! was proven live for `general.gaps_in` in this project's runtime/reload
//! proof, and rows are marked supported only where that same scalar mechanism,
//! a runtime-safe value grammar, and a low-risk classification all hold.
//! Rows the app's own high-risk model flags are blocked or dead-man gated.
//! Anything unproven is `NotProvenYet` — never guessed.

use serde::Serialize;

use crate::high_risk_family::{high_risk_family_for_row, HighRiskFamily};
use crate::write_classification::{SafeWritableRow, ScalarWriteValueKind, SAFE_WRITABLE_ROWS};

pub const RUNTIME_PREVIEW_MECHANISM_EVIDENCE: &str =
    "hl.config scalar runtime set proven live for general.gaps_in in the project's runtime/reload proof; per-row live verification pending";

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
pub enum RuntimePreviewCapability {
    LivePreviewSupported,
    LivePreviewSupportedWithThrottle,
    LivePreviewSupportedWithDeadMan,
    LivePreviewReadOnlyOnly,
    RequiresConfigWrite,
    RequiresReload,
    RequiresRelog,
    RequiresRestart,
    BlockedHighRisk,
    BlockedUnsupportedGrammar,
    BlockedStructuredFamilySemantics,
    NotProvenYet,
}

impl RuntimePreviewCapability {
    pub fn as_str(self) -> &'static str {
        match self {
            Self::LivePreviewSupported => "LivePreviewSupported",
            Self::LivePreviewSupportedWithThrottle => "LivePreviewSupportedWithThrottle",
            Self::LivePreviewSupportedWithDeadMan => "LivePreviewSupportedWithDeadMan",
            Self::LivePreviewReadOnlyOnly => "LivePreviewReadOnlyOnly",
            Self::RequiresConfigWrite => "RequiresConfigWrite",
            Self::RequiresReload => "RequiresReload",
            Self::RequiresRelog => "RequiresRelog",
            Self::RequiresRestart => "RequiresRestart",
            Self::BlockedHighRisk => "BlockedHighRisk",
            Self::BlockedUnsupportedGrammar => "BlockedUnsupportedGrammar",
            Self::BlockedStructuredFamilySemantics => "BlockedStructuredFamilySemantics",
            Self::NotProvenYet => "NotProvenYet",
        }
    }

    /// Rows that may mutate runtime without a dead-man countdown.
    pub fn live_previewable_by_default(self) -> bool {
        matches!(
            self,
            Self::LivePreviewSupported | Self::LivePreviewSupportedWithThrottle
        )
    }

    /// Rows that may mutate runtime only inside a confirmed dead-man session.
    pub fn live_previewable_with_dead_man(self) -> bool {
        self.live_previewable_by_default() || self == Self::LivePreviewSupportedWithDeadMan
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
pub enum RuntimePreviewRiskClass {
    LowRiskVisual,
    LowRiskLayout,
    MediumRiskBehavior,
    HighRiskInput,
    HighRiskDisplay,
    HighRiskSession,
    HighRiskSecurity,
    UnknownRisk,
}

impl RuntimePreviewRiskClass {
    pub fn as_str(self) -> &'static str {
        match self {
            Self::LowRiskVisual => "LowRiskVisual",
            Self::LowRiskLayout => "LowRiskLayout",
            Self::MediumRiskBehavior => "MediumRiskBehavior",
            Self::HighRiskInput => "HighRiskInput",
            Self::HighRiskDisplay => "HighRiskDisplay",
            Self::HighRiskSession => "HighRiskSession",
            Self::HighRiskSecurity => "HighRiskSecurity",
            Self::UnknownRisk => "UnknownRisk",
        }
    }
}

#[derive(Debug, Clone, Serialize)]
pub struct RuntimePreviewRowCapability {
    pub row_id: &'static str,
    pub official_setting: &'static str,
    pub value_kind: String,
    pub capability: RuntimePreviewCapability,
    pub risk: RuntimePreviewRiskClass,
    pub throttle_ms: Option<u64>,
    pub dead_man_required: bool,
    pub runtime_command_strategy: &'static str,
    pub revert_strategy: &'static str,
    pub reload_relog_restart_requirement: &'static str,
    pub reason: &'static str,
    pub evidence: &'static str,
}

const STRATEGY_HL_CONFIG: &str = "hyprctl eval hl.config scalar runtime set";
const STRATEGY_NONE: &str = "none";
const REVERT_REAPPLY: &str = "reapply captured original value via the same runtime set";
const REVERT_NONE: &str = "none";
const NO_RELOAD_REQUIREMENT: &str = "none";

fn value_kind_runtime_safe(kind: ScalarWriteValueKind) -> bool {
    matches!(
        kind,
        ScalarWriteValueKind::Boolean
            | ScalarWriteValueKind::FiniteChoice
            | ScalarWriteValueKind::Number
            | ScalarWriteValueKind::Percent
            | ScalarWriteValueKind::Color
            | ScalarWriteValueKind::Gradient
            | ScalarWriteValueKind::CssGap
            | ScalarWriteValueKind::Vector2
            | ScalarWriteValueKind::NumericList
            | ScalarWriteValueKind::CommaSeparatedFloatList
            | ScalarWriteValueKind::SourceBacked
    )
}

fn value_kind_needs_throttle(kind: ScalarWriteValueKind) -> bool {
    matches!(
        kind,
        ScalarWriteValueKind::Number
            | ScalarWriteValueKind::Percent
            | ScalarWriteValueKind::Color
            | ScalarWriteValueKind::Gradient
            | ScalarWriteValueKind::CssGap
            | ScalarWriteValueKind::Vector2
            | ScalarWriteValueKind::NumericList
            | ScalarWriteValueKind::CommaSeparatedFloatList
            | ScalarWriteValueKind::SourceBacked
    )
}

fn misc_visual_whitelisted(official_setting: &str) -> bool {
    matches!(
        official_setting,
        "misc.disable_hyprland_logo"
            | "misc.disable_splash_rendering"
            | "misc.background_color"
            | "misc.col.splash"
            | "misc.animate_manual_resizes"
            | "misc.animate_mouse_windowdragging"
    )
}

pub fn classify_runtime_preview_row(row: &SafeWritableRow) -> RuntimePreviewRowCapability {
    let value_kind = format!("{:?}", row.value_kind);
    let base = |capability: RuntimePreviewCapability,
                risk: RuntimePreviewRiskClass,
                throttle_ms: Option<u64>,
                dead_man_required: bool,
                reason: &'static str| {
        let supported = capability.live_previewable_with_dead_man();
        RuntimePreviewRowCapability {
            row_id: row.row_id,
            official_setting: row.official_setting,
            value_kind: value_kind.clone(),
            capability,
            risk,
            throttle_ms,
            dead_man_required,
            runtime_command_strategy: if supported {
                STRATEGY_HL_CONFIG
            } else {
                STRATEGY_NONE
            },
            revert_strategy: if supported {
                REVERT_REAPPLY
            } else {
                REVERT_NONE
            },
            reload_relog_restart_requirement: NO_RELOAD_REQUIREMENT,
            reason,
            evidence: RUNTIME_PREVIEW_MECHANISM_EVIDENCE,
        }
    };

    // The app's own high-risk model wins first.
    if let Some(family) = high_risk_family_for_row(row.row_id) {
        return match family {
            HighRiskFamily::InputDevice => base(
                RuntimePreviewCapability::LivePreviewSupportedWithDeadMan,
                RuntimePreviewRiskClass::HighRiskInput,
                Some(250),
                true,
                "input/cursor rows can lock the user out; preview only inside a confirmed dead-man session, disabled by default",
            ),
            HighRiskFamily::AnimationPerformance => base(
                RuntimePreviewCapability::LivePreviewSupportedWithDeadMan,
                RuntimePreviewRiskClass::MediumRiskBehavior,
                Some(250),
                true,
                "animation rows are flagged for performance risk by the app's high-risk model; dead-man gated, disabled by default",
            ),
            HighRiskFamily::MonitorOutput
            | HighRiskFamily::DisplayRenderPipeline
            | HighRiskFamily::ShaderScreenShader => base(
                RuntimePreviewCapability::BlockedHighRisk,
                RuntimePreviewRiskClass::HighRiskDisplay,
                None,
                true,
                "display/monitor/render/shader rows can black out the screen; blocked until display recovery proof exists",
            ),
            HighRiskFamily::WindowRuleWorkspaceBehavior => base(
                RuntimePreviewCapability::BlockedHighRisk,
                RuntimePreviewRiskClass::MediumRiskBehavior,
                None,
                true,
                "window/workspace behavior rows can rearrange the session unexpectedly; blocked pending behavior recovery proof",
            ),
            HighRiskFamily::ExecScriptPath => base(
                RuntimePreviewCapability::BlockedHighRisk,
                RuntimePreviewRiskClass::HighRiskSecurity,
                None,
                true,
                "exec/script/path rows execute or reference external code; never live previewed",
            ),
            HighRiskFamily::EnvironmentSession | HighRiskFamily::ProfileModeSwitch => base(
                RuntimePreviewCapability::BlockedHighRisk,
                RuntimePreviewRiskClass::HighRiskSession,
                None,
                true,
                "environment/session/profile rows affect the whole session; blocked",
            ),
            HighRiskFamily::UnknownHighRisk => base(
                RuntimePreviewCapability::BlockedHighRisk,
                RuntimePreviewRiskClass::UnknownRisk,
                None,
                true,
                "row is high-risk in the app's model without a mapped recovery family; blocked",
            ),
        };
    }

    // Grammar gates for non-high-risk rows.
    if matches!(
        row.value_kind,
        ScalarWriteValueKind::ComplexRaw | ScalarWriteValueKind::Unknown
    ) {
        return base(
            RuntimePreviewCapability::BlockedUnsupportedGrammar,
            RuntimePreviewRiskClass::UnknownRisk,
            None,
            false,
            "complex/unknown value grammar has no proven runtime representation",
        );
    }
    if !value_kind_runtime_safe(row.value_kind) {
        return base(
            RuntimePreviewCapability::RequiresConfigWrite,
            RuntimePreviewRiskClass::MediumRiskBehavior,
            None,
            false,
            "string/path/regex/monitor-name grammar is not proven runtime-safe; changes persist through the config write path",
        );
    }

    let section = row.official_setting.split('.').next().unwrap_or_default();
    let (previewable, risk): (bool, RuntimePreviewRiskClass) = match section {
        "general" | "dwindle" | "master" => (true, RuntimePreviewRiskClass::LowRiskLayout),
        "decoration" | "group" => (true, RuntimePreviewRiskClass::LowRiskVisual),
        "misc" => (
            misc_visual_whitelisted(row.official_setting),
            if misc_visual_whitelisted(row.official_setting) {
                RuntimePreviewRiskClass::LowRiskVisual
            } else {
                RuntimePreviewRiskClass::MediumRiskBehavior
            },
        ),
        "binds" | "gestures" | "grouping" => (false, RuntimePreviewRiskClass::MediumRiskBehavior),
        "debug" | "ecosystem" | "experimental" | "render" | "opengl" | "xwayland" => {
            (false, RuntimePreviewRiskClass::UnknownRisk)
        }
        _ => (false, RuntimePreviewRiskClass::UnknownRisk),
    };

    if previewable {
        if value_kind_needs_throttle(row.value_kind) {
            base(
                RuntimePreviewCapability::LivePreviewSupportedWithThrottle,
                risk,
                Some(150),
                false,
                "low-risk visual/layout scalar on the proven hl.config runtime path; continuous values are throttled",
            )
        } else {
            base(
                RuntimePreviewCapability::LivePreviewSupported,
                risk,
                None,
                false,
                "low-risk visual/layout toggle/choice on the proven hl.config runtime path",
            )
        }
    } else if risk == RuntimePreviewRiskClass::MediumRiskBehavior {
        base(
            RuntimePreviewCapability::RequiresConfigWrite,
            risk,
            None,
            false,
            "behavioral scalar; runtime application is plausible but unproven, so changes persist through the config write path",
        )
    } else {
        base(
            RuntimePreviewCapability::NotProvenYet,
            risk,
            None,
            false,
            "no runtime-safety evidence for this section yet; not proven",
        )
    }
}

pub fn runtime_preview_capability_matrix() -> Vec<RuntimePreviewRowCapability> {
    SAFE_WRITABLE_ROWS
        .iter()
        .map(classify_runtime_preview_row)
        .collect()
}

pub fn runtime_preview_row_capability(row_id: &str) -> Option<RuntimePreviewRowCapability> {
    SAFE_WRITABLE_ROWS
        .iter()
        .find(|row| row.row_id == row_id)
        .map(classify_runtime_preview_row)
}

#[derive(Debug, Clone, Serialize)]
pub struct RuntimePreviewFamilyCapability {
    pub family_id: &'static str,
    pub capability: RuntimePreviewCapability,
    pub risk: RuntimePreviewRiskClass,
    pub dead_man_required: bool,
    pub runtime_command_strategy: &'static str,
    pub revert_strategy: &'static str,
    pub reason: &'static str,
    pub evidence: &'static str,
}

pub fn runtime_preview_family_capabilities() -> Vec<RuntimePreviewFamilyCapability> {
    let family = |family_id: &'static str,
                  capability: RuntimePreviewCapability,
                  risk: RuntimePreviewRiskClass,
                  dead_man_required: bool,
                  reason: &'static str| {
        RuntimePreviewFamilyCapability {
        family_id,
        capability,
        risk,
        dead_man_required,
        runtime_command_strategy: STRATEGY_NONE,
        revert_strategy: REVERT_NONE,
        reason,
        evidence: "structured-family records are directive-like; no runtime record-set mechanism has been proven in this project",
    }
    };
    vec![
        family(
            "hl.monitor",
            RuntimePreviewCapability::BlockedHighRisk,
            RuntimePreviewRiskClass::HighRiskDisplay,
            true,
            "monitor records can disable or misconfigure displays; blocked until display recovery and dead-man proof exist",
        ),
        family(
            "hl.bind",
            RuntimePreviewCapability::BlockedHighRisk,
            RuntimePreviewRiskClass::HighRiskInput,
            true,
            "bind records change input behavior and can lock the user out; blocked pending dead-man proof",
        ),
        family(
            "hl.animation",
            RuntimePreviewCapability::NotProvenYet,
            RuntimePreviewRiskClass::LowRiskVisual,
            false,
            "animation records are visually low-risk, but no runtime record-application mechanism is proven yet",
        ),
        family(
            "hl.curve",
            RuntimePreviewCapability::NotProvenYet,
            RuntimePreviewRiskClass::LowRiskVisual,
            false,
            "an unused bezier curve is inert, but no runtime record-application mechanism is proven yet",
        ),
        family(
            "hl.gesture",
            RuntimePreviewCapability::BlockedStructuredFamilySemantics,
            RuntimePreviewRiskClass::MediumRiskBehavior,
            true,
            "gesture records change touch/pointer behavior mid-session; record semantics are not runtime-safe without proof",
        ),
        family(
            "hl.device",
            RuntimePreviewCapability::BlockedHighRisk,
            RuntimePreviewRiskClass::HighRiskInput,
            true,
            "device records reconfigure input hardware; blocked pending dead-man proof",
        ),
        family(
            "hl.permission",
            RuntimePreviewCapability::BlockedHighRisk,
            RuntimePreviewRiskClass::HighRiskSecurity,
            true,
            "permission records are security policy; never live previewed",
        ),
    ]
}

#[derive(Debug, Clone, Serialize)]
pub struct RuntimePreviewMatrixSummary {
    pub scalar_rows_total: usize,
    pub scalar_rows_classified: usize,
    pub structured_families_total: usize,
    pub structured_families_classified: usize,
    pub live_preview_supported: usize,
    pub live_preview_supported_with_throttle: usize,
    pub dead_man_required: usize,
    pub requires_config_write: usize,
    pub requires_reload: usize,
    pub requires_relog: usize,
    pub requires_restart: usize,
    pub blocked_high_risk: usize,
    pub blocked_unsupported_grammar: usize,
    pub not_proven_yet: usize,
}

pub fn runtime_preview_matrix_summary() -> RuntimePreviewMatrixSummary {
    let rows = runtime_preview_capability_matrix();
    let families = runtime_preview_family_capabilities();
    let count = |capability: RuntimePreviewCapability| {
        rows.iter()
            .filter(|row| row.capability == capability)
            .count()
    };
    RuntimePreviewMatrixSummary {
        scalar_rows_total: SAFE_WRITABLE_ROWS.len(),
        scalar_rows_classified: rows.len(),
        structured_families_total: 7,
        structured_families_classified: families.len(),
        live_preview_supported: count(RuntimePreviewCapability::LivePreviewSupported),
        live_preview_supported_with_throttle: count(
            RuntimePreviewCapability::LivePreviewSupportedWithThrottle,
        ),
        dead_man_required: count(RuntimePreviewCapability::LivePreviewSupportedWithDeadMan),
        requires_config_write: count(RuntimePreviewCapability::RequiresConfigWrite),
        requires_reload: count(RuntimePreviewCapability::RequiresReload),
        requires_relog: count(RuntimePreviewCapability::RequiresRelog),
        requires_restart: count(RuntimePreviewCapability::RequiresRestart),
        blocked_high_risk: count(RuntimePreviewCapability::BlockedHighRisk),
        blocked_unsupported_grammar: count(RuntimePreviewCapability::BlockedUnsupportedGrammar),
        not_proven_yet: count(RuntimePreviewCapability::NotProvenYet),
    }
}
