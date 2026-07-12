//! Structured-family runtime preview capability model.
//!
//! Classifies every one of the 7 structured families for live runtime
//! preview based on evidence probed read-safely against the running
//! compositor (invalid-argument errors reveal API existence and schemas
//! without mutating anything):
//!
//! - Every family has a runtime record API in the Lua config manager
//!   (`hl.animation`, `hl.curve`, `hl.gesture`, `hl.bind`, `hl.device`,
//!   `hl.permission`, `hl.monitor`).
//! - `hyprctl animations` provides full read-only readback for animation
//!   leaves (name/overriden/bezier/enabled/speed/style) and bezier curves
//!   (name/X0/Y0/X1/Y1) — a real verification mechanism.
//! - There is NO removal mechanism for records and NO revert-to-inherit for
//!   animation overrides (`enabled` is mandatory), so record *creation* can
//!   never be exactly reverted. Only *modifying an existing record* has an
//!   exact, readback-verified revert: reapply the captured original values.
//!
//! Families are therefore promoted only for the modify-existing scope, only
//! after an env-gated live proof round trip passed with zero residue, and
//! only with the receipt recorded here.

use serde::Serialize;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
pub enum StructuredFamilyRuntimePreviewCapability {
    LivePreviewSupported,
    LivePreviewSupportedWithThrottle,
    LivePreviewSupportedWithDeadMan,
    LivePreviewModelOnly,
    RequiresConfigWrite,
    RequiresReload,
    RequiresRelog,
    RequiresRestart,
    BlockedHighRisk,
    BlockedUnsupportedRuntimeRecordSyntax,
    BlockedNoRevertMechanism,
    BlockedNoVerificationMechanism,
    NotProvenYet,
}

impl StructuredFamilyRuntimePreviewCapability {
    pub fn as_str(self) -> &'static str {
        match self {
            Self::LivePreviewSupported => "LivePreviewSupported",
            Self::LivePreviewSupportedWithThrottle => "LivePreviewSupportedWithThrottle",
            Self::LivePreviewSupportedWithDeadMan => "LivePreviewSupportedWithDeadMan",
            Self::LivePreviewModelOnly => "LivePreviewModelOnly",
            Self::RequiresConfigWrite => "RequiresConfigWrite",
            Self::RequiresReload => "RequiresReload",
            Self::RequiresRelog => "RequiresRelog",
            Self::RequiresRestart => "RequiresRestart",
            Self::BlockedHighRisk => "BlockedHighRisk",
            Self::BlockedUnsupportedRuntimeRecordSyntax => "BlockedUnsupportedRuntimeRecordSyntax",
            Self::BlockedNoRevertMechanism => "BlockedNoRevertMechanism",
            Self::BlockedNoVerificationMechanism => "BlockedNoVerificationMechanism",
            Self::NotProvenYet => "NotProvenYet",
        }
    }

    pub fn live_previewable(self) -> bool {
        matches!(
            self,
            Self::LivePreviewSupported
                | Self::LivePreviewSupportedWithThrottle
                | Self::LivePreviewSupportedWithDeadMan
        )
    }
}

/// A passed family-level live proof: a modify-existing round trip verified
/// through readback with zero residue. Recorded only after the env-gated
/// harness actually ran against the compositor.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
pub struct ProvenFamilyRecordProof {
    pub family_id: &'static str,
    pub record: &'static str,
    pub original: &'static str,
    pub preview: &'static str,
    pub verification: &'static str,
    pub proof_date: &'static str,
    pub proof_env: &'static str,
}

/// Entries are added only after a real env-gated proof run against the
/// running compositor (tests/structured_family_runtime_preview_live.rs).
pub const PROVEN_FAMILY_RECORD_PROOFS: &[ProvenFamilyRecordProof] = &[
    ProvenFamilyRecordProof {
        family_id: "hl.animation",
        record: "global",
        original: "enabled=1 speed=8.00 bezier=default",
        preview: "speed=8.5",
        verification: "read-only animations listing verified apply and exact restore; zero residue",
        proof_date: "2026-07-12",
        proof_env: "HYPRLAND_SETTINGS_RUN_STRUCTURED_RUNTIME_PREVIEW_LIVE=1 HYPRLAND_SETTINGS_STRUCTURED_FAMILY=hl.animation",
    },
    ProvenFamilyRecordProof {
        family_id: "hl.curve",
        record: "default",
        original: "(0, 0.75, 0.15, 1)",
        preview: "y0=0.76",
        verification: "read-only animations bezier listing verified apply and exact restore; zero residue",
        proof_date: "2026-07-12",
        proof_env: "HYPRLAND_SETTINGS_RUN_STRUCTURED_RUNTIME_PREVIEW_LIVE=1 HYPRLAND_SETTINGS_STRUCTURED_FAMILY=hl.curve",
    },
];

pub fn proven_family_record_proof(family_id: &str) -> Option<&'static ProvenFamilyRecordProof> {
    PROVEN_FAMILY_RECORD_PROOFS
        .iter()
        .find(|proof| proof.family_id == family_id)
}

#[derive(Debug, Clone, Serialize)]
pub struct StructuredFamilyRuntimePreviewProfile {
    pub family_id: &'static str,
    pub capability: StructuredFamilyRuntimePreviewCapability,
    pub scope: &'static str,
    pub runtime_command_strategy: &'static str,
    pub record_rendering_strategy: &'static str,
    pub original_capture_strategy: &'static str,
    pub preview_value_strategy: &'static str,
    pub revert_strategy: &'static str,
    pub verification_strategy: &'static str,
    pub dead_man_required: bool,
    pub ui_status: &'static str,
    pub blocked_reason: Option<&'static str>,
    pub next_proof_needed: &'static str,
    pub evidence: &'static str,
}

pub fn structured_family_runtime_preview_profiles() -> Vec<StructuredFamilyRuntimePreviewProfile> {
    let animation_proven = proven_family_record_proof("hl.animation").is_some();
    let curve_proven = proven_family_record_proof("hl.curve").is_some();
    vec![
        StructuredFamilyRuntimePreviewProfile {
            family_id: "hl.animation",
            capability: if animation_proven {
                StructuredFamilyRuntimePreviewCapability::LivePreviewSupportedWithDeadMan
            } else {
                StructuredFamilyRuntimePreviewCapability::NotProvenYet
            },
            scope: "modify-existing only: animation leaves that already carry an explicit override can be previewed and exactly reverted; leaves in inherited state cannot (no revert-to-inherit exists)",
            runtime_command_strategy: "hl.animation({ leaf = NAME, enabled = BOOL, speed = NUMBER, bezier = NAME }) via the eval runner",
            record_rendering_strategy: "one Lua table per animation leaf with the four schema fields",
            original_capture_strategy: "parse the leaf's overriden/enabled/speed/bezier/style from the read-only animations listing before any mutation",
            preview_value_strategy: "minimal delta on speed for an already-overridden leaf",
            revert_strategy: "reapply the captured explicit values and verify via readback; refuse leaves whose original state is inherited",
            verification_strategy: "read-only animations listing before and after apply and after revert",
            dead_man_required: true,
            ui_status: if animation_proven {
                "Supervised preview available for explicitly overridden animation records"
            } else {
                "Not proven yet: modify-existing round trip pending live proof"
            },
            blocked_reason: None,
            next_proof_needed: "env-gated modify-existing round trip on the explicitly overridden global node (zero residue)",
            evidence: "live proof: the explicitly overridden global node round-tripped (speed 8.00 -> 8.5 -> 8.00) with readback-verified apply and exact restore; enabled is mandatory and revert-to-inherit does not exist, so inherited leaves are excluded from scope",
        },
        StructuredFamilyRuntimePreviewProfile {
            family_id: "hl.curve",
            capability: if curve_proven {
                StructuredFamilyRuntimePreviewCapability::LivePreviewSupportedWithDeadMan
            } else {
                StructuredFamilyRuntimePreviewCapability::NotProvenYet
            },
            scope: "modify-existing only: bezier curves already present in the runtime can be redefined and exactly restored; creating new curves is excluded (no deletion mechanism)",
            runtime_command_strategy: "hl.curve(NAME, { type = \"bezier\", points = { {X0, Y0}, {X1, Y1} } }) via the eval runner",
            record_rendering_strategy: "curve table with type and control points matching the readback fields",
            original_capture_strategy: "parse the curve's X0/Y0/X1/Y1 from the read-only animations listing before any mutation",
            preview_value_strategy: "minimal control-point delta on an existing curve",
            revert_strategy: "redefine the curve with the captured original points and verify via readback; refuse curve creation",
            verification_strategy: "read-only animations bezier listing before and after apply and after revert",
            dead_man_required: true,
            ui_status: if curve_proven {
                "Supervised preview available for existing bezier curve records"
            } else {
                "Not proven yet: modify-existing round trip pending live proof"
            },
            blocked_reason: None,
            next_proof_needed: "env-gated modify-existing round trip on the built-in default curve (zero residue)",
            evidence: "live proof: the built-in default curve was redefined ({ type, points } table form), verified via bezier readback, and restored exactly with zero residue; hl.config bezier keys are silently ignored or rejected",
        },
        StructuredFamilyRuntimePreviewProfile {
            family_id: "hl.gesture",
            capability: StructuredFamilyRuntimePreviewCapability::BlockedNoVerificationMechanism,
            scope: "no live preview",
            runtime_command_strategy: "none approved",
            record_rendering_strategy: "none",
            original_capture_strategy: "none: no gesture record readback mechanism was identified",
            preview_value_strategy: "none",
            revert_strategy: "none",
            verification_strategy: "none identified: hyprctl exposes no gesture record listing",
            dead_man_required: true,
            ui_status: "Blocked: gesture records cannot be verified after a runtime change",
            blocked_reason: Some(
                "the runtime API exists (hl.gesture, schema requires a fingers field), but there is no readback mechanism to verify apply or revert, and no touch hardware is present to observe behavior",
            ),
            next_proof_needed: "a gesture record readback mechanism plus touch hardware",
            evidence: "live probe: hl.gesture exists (missing required field fingers); no readback found",
        },
        StructuredFamilyRuntimePreviewProfile {
            family_id: "hl.monitor",
            capability: StructuredFamilyRuntimePreviewCapability::BlockedHighRisk,
            scope: "no live preview",
            runtime_command_strategy: "none approved",
            record_rendering_strategy: "none",
            original_capture_strategy: "read-only monitor listing exists but is not sufficient recovery",
            preview_value_strategy: "none",
            revert_strategy: "none: a failed monitor change can blank the screen, and revert must not require the user to see it",
            verification_strategy: "hyprctl monitors readback exists but cannot confirm the user can still see the screen",
            dead_man_required: true,
            ui_status: "Blocked: display changes need a blind recovery system before any preview",
            blocked_reason: Some(
                "monitor records can disable or misconfigure displays; live preview requires a display dead-man recovery system that reverts without user sight, which does not exist",
            ),
            next_proof_needed: "a display recovery system with blind auto-revert and its own gates",
            evidence: "live probe: hl.monitor exists (output field required); risk model unchanged",
        },
        StructuredFamilyRuntimePreviewProfile {
            family_id: "hl.bind",
            capability: StructuredFamilyRuntimePreviewCapability::BlockedHighRisk,
            scope: "no live preview",
            runtime_command_strategy: "none approved",
            record_rendering_strategy: "none",
            original_capture_strategy: "hyprctl binds readback exists",
            preview_value_strategy: "none",
            revert_strategy: "unproven: no verified unbind round trip, and a wrong bind can capture input",
            verification_strategy: "hyprctl binds readback exists but does not remove the input risk",
            dead_man_required: true,
            ui_status: "Blocked: keybind records can capture or break input control",
            blocked_reason: Some(
                "bind records change input control; a wrong bind can swallow the keys needed to confirm or revert; requires a strong fallback model before any preview",
            ),
            next_proof_needed: "a proven unbind round trip plus a secondary input fallback model",
            evidence: "live probe: hl.bind exists (expects string arguments); binds readback exists via hyprctl binds",
        },
        StructuredFamilyRuntimePreviewProfile {
            family_id: "hl.device",
            capability: StructuredFamilyRuntimePreviewCapability::BlockedHighRisk,
            scope: "no live preview",
            runtime_command_strategy: "none approved",
            record_rendering_strategy: "none",
            original_capture_strategy: "hyprctl devices readback exists",
            preview_value_strategy: "none",
            revert_strategy: "unproven for per-device records",
            verification_strategy: "hyprctl devices readback exists but device reconfiguration risk stands",
            dead_man_required: true,
            ui_status: "Blocked: device records reconfigure input hardware",
            blocked_reason: Some(
                "device records target specific input hardware; a wrong record can disable the device used to confirm or revert",
            ),
            next_proof_needed: "per-device proof with a guaranteed unaffected secondary device",
            evidence: "live probe: hl.device exists (name field required)",
        },
        StructuredFamilyRuntimePreviewProfile {
            family_id: "hl.permission",
            capability: StructuredFamilyRuntimePreviewCapability::BlockedHighRisk,
            scope: "no live preview",
            runtime_command_strategy: "none approved",
            record_rendering_strategy: "none",
            original_capture_strategy: "none",
            preview_value_strategy: "none",
            revert_strategy: "none",
            verification_strategy: "none",
            dead_man_required: true,
            ui_status: "Blocked: permission records are security policy",
            blocked_reason: Some(
                "permission records grant or deny capabilities to binaries; security policy is never live previewed",
            ),
            next_proof_needed: "none planned: security policy stays config-write only",
            evidence: "live probe: hl.permission exists (expects { binary, type, mode }); policy unchanged",
        },
    ]
}

pub fn structured_family_runtime_preview_profile(
    family_id: &str,
) -> Option<StructuredFamilyRuntimePreviewProfile> {
    structured_family_runtime_preview_profiles()
        .into_iter()
        .find(|profile| profile.family_id == family_id)
}

#[derive(Debug, Clone, Serialize)]
pub struct StructuredFamilyRuntimePreviewSummary {
    pub families_total: usize,
    pub families_classified: usize,
    pub live_preview_supported: usize,
    pub live_preview_supported_with_dead_man: usize,
    pub model_only: usize,
    pub blocked_high_risk: usize,
    pub blocked_unsupported_runtime_record_syntax: usize,
    pub blocked_no_revert_mechanism: usize,
    pub blocked_no_verification_mechanism: usize,
    pub not_proven_yet: usize,
}

pub fn structured_family_runtime_preview_summary() -> StructuredFamilyRuntimePreviewSummary {
    let profiles = structured_family_runtime_preview_profiles();
    let count = |capability: StructuredFamilyRuntimePreviewCapability| {
        profiles
            .iter()
            .filter(|profile| profile.capability == capability)
            .count()
    };
    StructuredFamilyRuntimePreviewSummary {
        families_total: 7,
        families_classified: profiles.len(),
        live_preview_supported: count(
            StructuredFamilyRuntimePreviewCapability::LivePreviewSupported,
        ),
        live_preview_supported_with_dead_man: count(
            StructuredFamilyRuntimePreviewCapability::LivePreviewSupportedWithDeadMan,
        ),
        model_only: count(StructuredFamilyRuntimePreviewCapability::LivePreviewModelOnly),
        blocked_high_risk: count(StructuredFamilyRuntimePreviewCapability::BlockedHighRisk),
        blocked_unsupported_runtime_record_syntax: count(
            StructuredFamilyRuntimePreviewCapability::BlockedUnsupportedRuntimeRecordSyntax,
        ),
        blocked_no_revert_mechanism: count(
            StructuredFamilyRuntimePreviewCapability::BlockedNoRevertMechanism,
        ),
        blocked_no_verification_mechanism: count(
            StructuredFamilyRuntimePreviewCapability::BlockedNoVerificationMechanism,
        ),
        not_proven_yet: count(StructuredFamilyRuntimePreviewCapability::NotProvenYet),
    }
}

/// Parse one animation leaf's fields out of the read-only animations listing.
pub fn parse_animation_leaf(listing: &str, leaf: &str) -> Option<(String, String, String)> {
    let mut lines = listing.lines().peekable();
    while let Some(line) = lines.next() {
        if line.trim() == format!("name: {leaf}") {
            let mut bezier = String::new();
            let mut enabled = String::new();
            let mut speed = String::new();
            for detail in lines.by_ref() {
                let detail = detail.trim();
                if detail.is_empty() {
                    break;
                }
                if let Some(value) = detail.strip_prefix("bezier:") {
                    bezier = value.trim().to_string();
                } else if let Some(value) = detail.strip_prefix("enabled:") {
                    enabled = value.trim().to_string();
                } else if let Some(value) = detail.strip_prefix("speed:") {
                    speed = value.trim().to_string();
                }
            }
            return Some((enabled, speed, bezier));
        }
    }
    None
}

/// Parse one bezier curve's control points out of the animations listing.
pub fn parse_bezier_points(listing: &str, name: &str) -> Option<(f64, f64, f64, f64)> {
    let mut lines = listing.lines().peekable();
    while let Some(line) = lines.next() {
        if line.trim() == format!("name: {name}") {
            let mut points = [None::<f64>; 4];
            for detail in lines.by_ref() {
                let detail = detail.trim();
                if detail.starts_with("name:") || detail.is_empty() {
                    break;
                }
                for (index, key) in ["X0:", "Y0:", "X1:", "Y1:"].iter().enumerate() {
                    if let Some(value) = detail.strip_prefix(key) {
                        points[index] = value.trim().parse::<f64>().ok();
                    }
                }
                if points.iter().all(Option::is_some) {
                    break;
                }
            }
            if let [Some(x0), Some(y0), Some(x1), Some(y1)] = points {
                return Some((x0, y0, x1, y1));
            }
            return None;
        }
    }
    None
}
