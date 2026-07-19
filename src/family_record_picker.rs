//! Family record picker: extends gated structured-family persistence one
//! proven record shape at a time.
//!
//! The picker lists the records that actually exist in the runtime readback
//! (the read-only animations listing), classifies each one honestly against
//! the recorded live-proof receipts, and exposes supervised preview plus
//! gated Save only for records whose shape carries a passed proof:
//!
//! - `hl.animation`: the enabled flag, speed, and bezier reference of an
//!   animation leaf that already carries an explicit override — each field
//!   behind its own passed shape proof. The bezier may only name a curve
//!   that already exists in the readback. Inherited leaves are blocked
//!   (persisting one would create a new override — creation is blocked).
//!   Internal compositor records (`__`-prefixed) are blocked. Leaves with a
//!   non-empty style are save-only: the rendered config line preserves the
//!   style, but the runtime preview command's style handling is not
//!   live-proven. The style field itself is not editable anywhere: the set
//!   of valid style values is not known from trusted evidence.
//! - `hl.curve`: the four control points of a bezier curve that already
//!   exists in the readback. The proven runtime command always writes all
//!   four points, so editing any point is the same proven command shape.
//!
//! Live-proven compositor semantics the preview honors: while a record is
//! disabled, the compositor resets its speed/bezier readback (speed 1.00,
//! bezier default), so a preview that leaves a record disabled verifies the
//! enabled flag only; reverts always restore and verify the full record.
//!
//! Only these two families can be expressed here; there is no record
//! creation and no record removal — those operations do not exist in this
//! module. Every save routes through the gated persistence path
//! (`structured_family_gated_persistence`), which re-verifies Safe Live
//! Save Mode live before any file access. This module never writes files.

use serde::Serialize;

use crate::config_discovery::ConfigDiscovery;
use crate::runtime_preview_executor::{
    HyprctlRuntimePreviewRunner, RuntimePreviewDeadMan, RuntimePreviewDeadManVerdict,
    RuntimePreviewRunner,
};
use crate::structured_family_gated_persistence::{
    gated_family_record_save, gated_family_record_save_with_precondition, FamilyRecordSaveRequest,
    FamilySaveError, FamilySavePrecondition, FamilySaveReceipt,
};
use crate::structured_family_runtime_preview::{
    parse_animation_records, parse_bezier_records, proven_record_shape_proof,
    AnimationRuntimeRecord, BezierRuntimeRecord, ANIMATION_RECORD_BEZIER_SHAPE,
    ANIMATION_RECORD_ENABLED_SHAPE, ANIMATION_RECORD_SPEED_SHAPE, CURVE_RECORD_POINTS_SHAPE,
};

/// Why the animation style field is not editable anywhere in the picker.
/// Shown as a disabled row in the UI — support requires trusted evidence of
/// the valid per-record style values plus a passed live proof, and neither
/// exists.
pub const ANIMATION_STYLE_BLOCKED_REASON: &str = "Style is not editable: the set of valid style values is not known from trusted evidence, and style handling through the runtime command has no passed live proof. Saves preserve the current style unchanged.";

/// Why gesture-family records have no picker. The compositor exposes no
/// gesture readback listing (the gestures request is unknown to hyprctl on
/// 0.55.4), so modify-existing verification is impossible — and honest
/// proofs would additionally need touch hardware.
pub const GESTURE_FAMILY_BLOCKED_REASON: &str = "Gesture records have no runtime readback listing on Hyprland 0.55.4, so a modify-existing edit cannot be verified; gesture proofs also require touch hardware, which is deferred.";

pub const RECORD_PICKER_COUNTDOWN_MS: u64 = 10_000;

pub const ANIMATION_SPEED_MIN: f64 = 0.1;
pub const ANIMATION_SPEED_MAX: f64 = 20.0;
pub const CURVE_X_MIN: f64 = 0.0;
pub const CURVE_X_MAX: f64 = 1.0;
pub const CURVE_Y_MIN: f64 = -1.0;
pub const CURVE_Y_MAX: f64 = 2.0;

/// The two families the picker can express. No other family fits this type.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
pub enum PickedFamily {
    Animation,
    Curve,
}

impl PickedFamily {
    pub fn family_id(self) -> &'static str {
        match self {
            Self::Animation => "hl.animation",
            Self::Curve => "hl.curve",
        }
    }
}

/// How the picker supports one listed record.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
pub enum RecordPickerSupport {
    /// Live preview and gated Save are both available (shape proof passed).
    SupportedProven,
    /// Gated Save is available; live preview is blocked with a reason.
    SaveOnly,
    /// Neither preview nor Save; the reason says why.
    Blocked,
}

impl RecordPickerSupport {
    pub fn as_str(self) -> &'static str {
        match self {
            Self::SupportedProven => "SupportedProven",
            Self::SaveOnly => "SaveOnly",
            Self::Blocked => "Blocked",
        }
    }
}

/// One animation record as the picker presents it.
#[derive(Debug, Clone, PartialEq, Serialize)]
pub struct AnimationRecordEntry {
    pub record: AnimationRuntimeRecord,
    pub support: RecordPickerSupport,
    pub preview_supported: bool,
    pub save_supported: bool,
    pub blocked_reason: Option<String>,
    pub current_value_text: String,
}

/// One bezier curve record as the picker presents it.
#[derive(Debug, Clone, PartialEq, Serialize)]
pub struct CurveRecordEntry {
    pub record: BezierRuntimeRecord,
    pub support: RecordPickerSupport,
    pub preview_supported: bool,
    pub save_supported: bool,
    pub blocked_reason: Option<String>,
    pub current_value_text: String,
}

/// Record names are interpolated into fixed-shape runtime expressions and
/// config lines, so only plainly safe names are accepted; anything else is
/// blocked (fail closed).
pub fn record_name_is_safe(name: &str) -> bool {
    !name.is_empty()
        && name
            .chars()
            .all(|character| character.is_ascii_alphanumeric() || character == '_')
}

/// Classify every animation leaf in the listing for the picker. Full
/// support needs a passed proof receipt for every editable field's shape
/// (speed, enabled, bezier) — a missing receipt fails closed to Blocked.
pub fn animation_record_entries(listing: &str) -> Vec<AnimationRecordEntry> {
    let shape_proven = [
        ANIMATION_RECORD_SPEED_SHAPE,
        ANIMATION_RECORD_ENABLED_SHAPE,
        ANIMATION_RECORD_BEZIER_SHAPE,
    ]
    .iter()
    .all(|shape| proven_record_shape_proof("hl.animation", shape).is_some());
    parse_animation_records(listing)
        .into_iter()
        .map(|record| {
            let current_value_text = format!(
                "speed {} (enabled {}, bezier {}{})",
                record.speed,
                record.enabled,
                if record.bezier.is_empty() {
                    "default"
                } else {
                    &record.bezier
                },
                if record.style.is_empty() {
                    String::new()
                } else {
                    format!(", style {}", record.style)
                }
            );
            let (support, blocked_reason) = if record.name.starts_with("__") {
                (
                    RecordPickerSupport::Blocked,
                    Some("internal compositor record; not part of user configuration".to_string()),
                )
            } else if !record_name_is_safe(&record.name) {
                (
                    RecordPickerSupport::Blocked,
                    Some("record name contains characters outside the safe set".to_string()),
                )
            } else if !record.overridden {
                (
                    RecordPickerSupport::Blocked,
                    Some(
                        "record inherits its values; saving would create a new override, and record creation is blocked (not modify-existing)"
                            .to_string(),
                    ),
                )
            } else if !shape_proven {
                (
                    RecordPickerSupport::Blocked,
                    Some("record shape not live-proven yet: a passed proof receipt is required for every editable animation field (speed, enabled, bezier)".to_string()),
                )
            } else if !record.style.is_empty() {
                (
                    RecordPickerSupport::SaveOnly,
                    Some(
                        "live preview blocked: style preservation through the runtime command is not proven; Save renders the config line with the style preserved"
                            .to_string(),
                    ),
                )
            } else {
                // Disabled records are preview-supported through the proven
                // enabled shape (0->1->0 round trip passed live); while a
                // record stays disabled, only its enabled flag is verifiable
                // (the compositor resets disabled speed/bezier readback).
                (RecordPickerSupport::SupportedProven, None)
            };
            AnimationRecordEntry {
                record,
                support,
                preview_supported: support == RecordPickerSupport::SupportedProven,
                save_supported: matches!(
                    support,
                    RecordPickerSupport::SupportedProven | RecordPickerSupport::SaveOnly
                ),
                blocked_reason,
                current_value_text,
            }
        })
        .collect()
}

/// Classify every bezier curve in the listing for the picker.
pub fn curve_record_entries(listing: &str) -> Vec<CurveRecordEntry> {
    let shape_proven = proven_record_shape_proof("hl.curve", CURVE_RECORD_POINTS_SHAPE).is_some();
    parse_bezier_records(listing)
        .into_iter()
        .map(|record| {
            let current_value_text = format!(
                "({}, {}, {}, {})",
                record.x0, record.y0, record.x1, record.y1
            );
            let (support, blocked_reason) = if !record_name_is_safe(&record.name) {
                (
                    RecordPickerSupport::Blocked,
                    Some("record name contains characters outside the safe set".to_string()),
                )
            } else if !shape_proven {
                (
                    RecordPickerSupport::Blocked,
                    Some("record shape not live-proven yet: no passed proof receipt exists for modifying an arbitrary existing curve".to_string()),
                )
            } else {
                (RecordPickerSupport::SupportedProven, None)
            };
            CurveRecordEntry {
                record,
                support,
                preview_supported: support == RecordPickerSupport::SupportedProven,
                save_supported: support == RecordPickerSupport::SupportedProven,
                blocked_reason,
                current_value_text,
            }
        })
        .collect()
}

fn read_animations(runner: &mut dyn RuntimePreviewRunner) -> Result<String, String> {
    runner.run("hyprctl", &["animations".to_string()])
}

/// List animation records from the live readback.
pub fn list_animation_records(
    runner: &mut dyn RuntimePreviewRunner,
) -> Result<Vec<AnimationRecordEntry>, String> {
    Ok(animation_record_entries(&read_animations(runner)?))
}

/// List curve records from the live readback.
pub fn list_curve_records(
    runner: &mut dyn RuntimePreviewRunner,
) -> Result<Vec<CurveRecordEntry>, String> {
    Ok(curve_record_entries(&read_animations(runner)?))
}

pub fn list_animation_records_live() -> Result<Vec<AnimationRecordEntry>, String> {
    let mut runner = HyprctlRuntimePreviewRunner;
    list_animation_records(&mut runner)
}

pub fn list_curve_records_live() -> Result<Vec<CurveRecordEntry>, String> {
    let mut runner = HyprctlRuntimePreviewRunner;
    list_curve_records(&mut runner)
}

/// Validate a speed for an animation record.
pub fn validate_animation_speed(value: f64) -> Result<(), String> {
    if !value.is_finite() || !(ANIMATION_SPEED_MIN..=ANIMATION_SPEED_MAX).contains(&value) {
        return Err(format!(
            "animation speed is limited to {ANIMATION_SPEED_MIN}..={ANIMATION_SPEED_MAX}"
        ));
    }
    Ok(())
}

/// Validate a bezier reference for an animation record: a safe,
/// non-internal name that already exists in the readback curve list. Only
/// existing curves can ever be referenced (modify-existing, no creation).
pub fn validate_animation_bezier(bezier: &str, existing_curves: &[String]) -> Result<(), String> {
    if !record_name_is_safe(bezier) || bezier.starts_with("__") {
        return Err("bezier name is not in the safe user-record set".to_string());
    }
    if !existing_curves.iter().any(|curve| curve == bezier) {
        return Err(format!(
            "bezier {bezier} does not exist in the runtime readback; only existing curves can be referenced"
        ));
    }
    Ok(())
}

/// Validate the four control points for a curve record.
pub fn validate_curve_points(x0: f64, y0: f64, x1: f64, y1: f64) -> Result<(), String> {
    for x in [x0, x1] {
        if !x.is_finite() || !(CURVE_X_MIN..=CURVE_X_MAX).contains(&x) {
            return Err(format!(
                "curve X control points are limited to {CURVE_X_MIN}..={CURVE_X_MAX}"
            ));
        }
    }
    for y in [y0, y1] {
        if !y.is_finite() || !(CURVE_Y_MIN..=CURVE_Y_MAX).contains(&y) {
            return Err(format!(
                "curve Y control points are limited to {CURVE_Y_MIN}..={CURVE_Y_MAX}"
            ));
        }
    }
    Ok(())
}

/// The fixed-shape runtime expression writing an animation record's proven
/// fields (enabled, speed, bezier). Every proven shape runs through this
/// one expression form.
pub fn render_animation_record_expression(
    record_name: &str,
    enabled: bool,
    speed: f64,
    bezier: &str,
) -> Result<String, String> {
    if !record_name_is_safe(record_name) {
        return Err("unsafe record name".to_string());
    }
    validate_animation_speed(speed)?;
    if !record_name_is_safe(bezier) {
        return Err("unsafe bezier name".to_string());
    }
    let enabled_lua = if enabled { "true" } else { "false" };
    Ok(format!(
        "hl.animation({{ leaf = \"{record_name}\", enabled = {enabled_lua}, speed = {speed}, bezier = \"{bezier}\" }})"
    ))
}

/// The fixed-shape runtime expression for an animation record preview.
/// Fields other than speed come from the captured readback record.
pub fn render_animation_preview_expression(
    record: &AnimationRuntimeRecord,
    speed: f64,
) -> Result<String, String> {
    let bezier_name = if record.bezier.is_empty() {
        "default".to_string()
    } else {
        record.bezier.clone()
    };
    render_animation_record_expression(&record.name, record.enabled == "1", speed, &bezier_name)
}

/// The fixed-shape runtime expression for a curve record preview. The proven
/// command shape always writes all four points.
pub fn render_curve_preview_expression(
    name: &str,
    x0: f64,
    y0: f64,
    x1: f64,
    y1: f64,
) -> Result<String, String> {
    if !record_name_is_safe(name) {
        return Err("unsafe record name".to_string());
    }
    validate_curve_points(x0, y0, x1, y1)?;
    Ok(format!(
        "hl.curve(\"{name}\", {{ type = \"bezier\", points = {{ {{{x0}, {y0}}}, {{{x1}, {y1}}} }} }})"
    ))
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
pub enum RecordPickerPhase {
    Disarmed,
    CountingDown,
    Kept,
    Saved,
    Reverted,
    TimedOutReverted,
    Cancelled,
}

impl RecordPickerPhase {
    pub fn as_str(self) -> &'static str {
        match self {
            Self::Disarmed => "Disarmed",
            Self::CountingDown => "Counting down",
            Self::Kept => "Kept",
            Self::Saved => "Saved",
            Self::Reverted => "Reverted",
            Self::TimedOutReverted => "Timed out and reverted",
            Self::Cancelled => "Cancelled",
        }
    }
}

#[derive(Debug, Clone, Serialize)]
pub struct RecordPickerReceipt {
    pub family_id: &'static str,
    pub record: String,
    pub action: &'static str,
    pub phase: RecordPickerPhase,
    pub original: Option<String>,
    pub applied: Option<String>,
    pub config_written: bool,
    pub reload_run: bool,
    pub status_text: String,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum RecordPickerError {
    RecordNotSupported(String),
    RecordMissing(String),
    InvalidValue(String),
    NotArmed,
    Runner(String),
    VerificationFailed(String),
}

impl RecordPickerError {
    pub fn user_text(&self) -> String {
        match self {
            Self::RecordNotSupported(reason) => format!("Record preview unavailable: {reason}"),
            Self::RecordMissing(record) => format!(
                "Record {record} was not found in the runtime readback; modify-existing preview refuses to run"
            ),
            Self::InvalidValue(detail) => format!("Value rejected: {detail}"),
            Self::NotArmed => "Start a supervised record preview first".to_string(),
            Self::Runner(detail) => format!("Preview failed: {detail}"),
            Self::VerificationFailed(detail) => format!("Verification failed: {detail}"),
        }
    }
}

/// The values a preview or save can carry — exactly the proven shapes. The
/// animation values bundle the three proven fields (each behind its own
/// receipt); the same fixed expression writes all of them every time.
#[derive(Debug, Clone, PartialEq, Serialize)]
pub enum PickedRecordValues {
    AnimationRecord {
        enabled: bool,
        speed: f64,
        bezier: String,
    },
    CurvePoints {
        x0: f64,
        y0: f64,
        x1: f64,
        y1: f64,
    },
}

#[derive(Debug, Clone)]
enum CapturedOriginal {
    Animation(AnimationRuntimeRecord),
    Curve(BezierRuntimeRecord),
}

/// Full-record equality with numeric tolerance on the speed (the readback
/// formats speeds with trailing zeros).
fn animation_records_equivalent(
    left: &AnimationRuntimeRecord,
    right: &AnimationRuntimeRecord,
) -> bool {
    let speeds_match = match (left.speed.parse::<f64>(), right.speed.parse::<f64>()) {
        (Ok(left_speed), Ok(right_speed)) => (left_speed - right_speed).abs() < 1e-3,
        _ => left.speed == right.speed,
    };
    left.name == right.name
        && left.overridden == right.overridden
        && left.bezier == right.bezier
        && left.enabled == right.enabled
        && left.style == right.style
        && speeds_match
}

/// Supervised preview controller for one picked record. Same recovery
/// semantics as the proven family preview controller: countdown, Keep,
/// Revert now, Cancel, session-drop revert; every step verified through the
/// read-only readback. No config writes happen here.
pub struct FamilyRecordPreviewController {
    runner: Box<dyn RuntimePreviewRunner>,
    family: PickedFamily,
    record_name: String,
    original: Option<CapturedOriginal>,
    phase: RecordPickerPhase,
    dead_man: Option<RuntimePreviewDeadMan>,
}

impl FamilyRecordPreviewController {
    /// Refuses records whose classification does not allow preview.
    pub fn new(
        family: PickedFamily,
        record_name: &str,
        runner: Box<dyn RuntimePreviewRunner>,
        listing: &str,
    ) -> Result<Self, RecordPickerError> {
        let preview_supported = match family {
            PickedFamily::Animation => animation_record_entries(listing)
                .into_iter()
                .find(|entry| entry.record.name == record_name)
                .map(|entry| (entry.preview_supported, entry.blocked_reason)),
            PickedFamily::Curve => curve_record_entries(listing)
                .into_iter()
                .find(|entry| entry.record.name == record_name)
                .map(|entry| (entry.preview_supported, entry.blocked_reason)),
        };
        match preview_supported {
            None => Err(RecordPickerError::RecordMissing(record_name.to_string())),
            Some((false, reason)) => Err(RecordPickerError::RecordNotSupported(
                reason.unwrap_or_else(|| "preview is not supported for this record".to_string()),
            )),
            Some((true, _)) => Ok(Self {
                runner,
                family,
                record_name: record_name.to_string(),
                original: None,
                phase: RecordPickerPhase::Disarmed,
                dead_man: None,
            }),
        }
    }

    pub fn new_live(family: PickedFamily, record_name: &str) -> Result<Self, RecordPickerError> {
        let mut runner = HyprctlRuntimePreviewRunner;
        let listing = read_animations(&mut runner).map_err(RecordPickerError::Runner)?;
        Self::new(family, record_name, Box::new(runner), &listing)
    }

    pub fn family(&self) -> PickedFamily {
        self.family
    }

    pub fn record_name(&self) -> &str {
        &self.record_name
    }

    pub fn phase(&self) -> RecordPickerPhase {
        self.phase
    }

    pub fn remaining_seconds(&self) -> u64 {
        self.dead_man
            .as_ref()
            .map(|dead_man| {
                dead_man
                    .timeout_ms
                    .saturating_sub(dead_man.elapsed_ms)
                    .div_ceil(1000)
            })
            .unwrap_or(0)
    }

    fn read_animation_record(&mut self) -> Result<AnimationRuntimeRecord, RecordPickerError> {
        let listing = read_animations(self.runner.as_mut()).map_err(RecordPickerError::Runner)?;
        parse_animation_records(&listing)
            .into_iter()
            .find(|record| record.name == self.record_name)
            .ok_or_else(|| RecordPickerError::RecordMissing(self.record_name.clone()))
    }

    fn read_curve_record(&mut self) -> Result<BezierRuntimeRecord, RecordPickerError> {
        let listing = read_animations(self.runner.as_mut()).map_err(RecordPickerError::Runner)?;
        parse_bezier_records(&listing)
            .into_iter()
            .find(|record| record.name == self.record_name)
            .ok_or_else(|| RecordPickerError::RecordMissing(self.record_name.clone()))
    }

    /// Read the record's current value text (read-only).
    pub fn current_value(&mut self) -> Result<String, RecordPickerError> {
        match self.family {
            PickedFamily::Animation => {
                let record = self.read_animation_record()?;
                Ok(format!(
                    "enabled {}, speed {}, bezier {}",
                    record.enabled,
                    record.speed,
                    if record.bezier.is_empty() {
                        "default"
                    } else {
                        &record.bezier
                    }
                ))
            }
            PickedFamily::Curve => {
                let record = self.read_curve_record()?;
                Ok(format!(
                    "{}, {}, {}, {}",
                    record.x0, record.y0, record.x1, record.y1
                ))
            }
        }
    }

    /// Apply validated preview values under the dead-man countdown.
    /// Modify-existing is enforced: a missing record refuses before any
    /// command is issued.
    pub fn preview(
        &mut self,
        values: PickedRecordValues,
    ) -> Result<RecordPickerReceipt, RecordPickerError> {
        let (expression, applied_text) = match (self.family, &values) {
            (
                PickedFamily::Animation,
                PickedRecordValues::AnimationRecord {
                    enabled,
                    speed,
                    bezier,
                },
            ) => {
                let listing =
                    read_animations(self.runner.as_mut()).map_err(RecordPickerError::Runner)?;
                let record = parse_animation_records(&listing)
                    .into_iter()
                    .find(|record| record.name == self.record_name)
                    .ok_or_else(|| RecordPickerError::RecordMissing(self.record_name.clone()))?;
                let existing_curves: Vec<String> = parse_bezier_records(&listing)
                    .into_iter()
                    .map(|curve| curve.name)
                    .collect();
                validate_animation_bezier(bezier, &existing_curves)
                    .map_err(RecordPickerError::InvalidValue)?;
                if self.original.is_none() {
                    self.original = Some(CapturedOriginal::Animation(record.clone()));
                }
                (
                    render_animation_record_expression(&record.name, *enabled, *speed, bezier)
                        .map_err(RecordPickerError::InvalidValue)?,
                    format!(
                        "enabled = {enabled}, speed = {speed}, bezier = {bezier}{}",
                        if *enabled {
                            ""
                        } else {
                            " (record disabled: only the enabled flag is verifiable while disabled)"
                        }
                    ),
                )
            }
            (PickedFamily::Curve, &PickedRecordValues::CurvePoints { x0, y0, x1, y1 }) => {
                let record = self.read_curve_record()?;
                if self.original.is_none() {
                    self.original = Some(CapturedOriginal::Curve(record.clone()));
                }
                (
                    render_curve_preview_expression(&self.record_name.clone(), x0, y0, x1, y1)
                        .map_err(RecordPickerError::InvalidValue)?,
                    format!("points = ({x0}, {y0}, {x1}, {y1})"),
                )
            }
            _ => {
                return Err(RecordPickerError::InvalidValue(
                    "values do not match the picked family".to_string(),
                ))
            }
        };
        self.runner
            .run("hyprctl", &["eval".to_string(), expression])
            .map_err(RecordPickerError::Runner)?;

        // Verify the apply through readback.
        self.verify_values(&values)?;

        self.phase = RecordPickerPhase::CountingDown;
        let mut dead_man = RuntimePreviewDeadMan::new(RECORD_PICKER_COUNTDOWN_MS);
        dead_man.recovery_instruction =
            "if the app exits during an unconfirmed record preview, the change was runtime-only; reapply the recorded original values or restart Hyprland";
        self.dead_man = Some(dead_man);
        let record = self.record_name.clone();
        Ok(self.receipt(
            "preview",
            Some(applied_text.clone()),
            format!(
                "Previewing live: {} record {record}: {applied_text} (auto-revert in {} seconds unless you Keep changes)",
                self.family.family_id(),
                RECORD_PICKER_COUNTDOWN_MS / 1000
            ),
        ))
    }

    fn verify_values(&mut self, values: &PickedRecordValues) -> Result<(), RecordPickerError> {
        match *values {
            PickedRecordValues::AnimationRecord {
                enabled,
                speed,
                ref bezier,
            } => {
                let observed = self.read_animation_record()?;
                let expected_enabled = if enabled { "1" } else { "0" };
                if observed.enabled != expected_enabled {
                    return Err(RecordPickerError::VerificationFailed(format!(
                        "expected enabled {expected_enabled}, observed {}",
                        observed.enabled
                    )));
                }
                // Live-proven compositor semantics: a disabled record's
                // speed/bezier readback is reset (speed 1.00, bezier
                // default), so those fields are only verifiable while the
                // record is enabled.
                if enabled {
                    let observed_speed: f64 = observed.speed.parse().map_err(|_| {
                        RecordPickerError::VerificationFailed("readback speed did not parse".into())
                    })?;
                    if (observed_speed - speed).abs() > 1e-3 {
                        return Err(RecordPickerError::VerificationFailed(format!(
                            "expected speed {speed}, observed {}",
                            observed.speed
                        )));
                    }
                    if observed.bezier != *bezier {
                        return Err(RecordPickerError::VerificationFailed(format!(
                            "expected bezier {bezier}, observed {}",
                            observed.bezier
                        )));
                    }
                }
            }
            PickedRecordValues::CurvePoints { x0, y0, x1, y1 } => {
                let observed = self.read_curve_record()?;
                for (expected, observed_point, label) in [
                    (x0, observed.x0, "X0"),
                    (y0, observed.y0, "Y0"),
                    (x1, observed.x1, "X1"),
                    (y1, observed.y1, "Y1"),
                ] {
                    if (observed_point - expected).abs() > 1e-3 {
                        return Err(RecordPickerError::VerificationFailed(format!(
                            "expected {label} {expected}, observed {observed_point}"
                        )));
                    }
                }
            }
        }
        Ok(())
    }

    /// Advance the countdown; auto-reverts on timeout.
    pub fn tick(
        &mut self,
        delta_ms: u64,
    ) -> Result<Option<RecordPickerReceipt>, RecordPickerError> {
        if self.phase != RecordPickerPhase::CountingDown {
            return Ok(None);
        }
        let verdict = match self.dead_man.as_mut() {
            Some(dead_man) => {
                dead_man.tick(delta_ms);
                dead_man.evaluate()
            }
            None => RuntimePreviewDeadManVerdict::KeepActive,
        };
        if verdict == RuntimePreviewDeadManVerdict::RevertRequired {
            let receipt = self.revert_internal("timeout")?;
            self.phase = RecordPickerPhase::TimedOutReverted;
            return Ok(Some(RecordPickerReceipt {
                phase: RecordPickerPhase::TimedOutReverted,
                ..receipt
            }));
        }
        Ok(None)
    }

    /// Keep the previewed values for this session (runtime-only; persistence
    /// is the separate gated Save).
    pub fn keep(&mut self) -> Result<RecordPickerReceipt, RecordPickerError> {
        if self.phase != RecordPickerPhase::CountingDown {
            return Err(RecordPickerError::NotArmed);
        }
        if let Some(dead_man) = self.dead_man.as_mut() {
            dead_man.confirm();
        }
        self.phase = RecordPickerPhase::Kept;
        let record = self.record_name.clone();
        Ok(self.receipt(
            "keep",
            None,
            format!(
                "Kept: record {record} stays at the previewed values for this session. Nothing was saved to your config."
            ),
        ))
    }

    /// Persist an active preview and consume its recovery registration only
    /// after the persistence layer returns a durable, reread-verified receipt.
    /// A persistence failure leaves the phase and original value untouched.
    pub fn persist_and_mark_saved<F>(
        &mut self,
        persist: F,
    ) -> Result<FamilySaveReceipt, FamilySaveError>
    where
        F: FnOnce() -> Result<FamilySaveReceipt, FamilySaveError>,
    {
        if !matches!(
            self.phase,
            RecordPickerPhase::CountingDown | RecordPickerPhase::Kept
        ) {
            return Err(FamilySaveError::PreviewStateInvalid(
                "the record preview is not active or kept".to_string(),
            ));
        }

        let receipt = persist()?;
        if !receipt.durable_receipt_created || !receipt.reread_verified {
            return Err(FamilySaveError::PreviewStateInvalid(
                "persistence did not return a durable reread-verified receipt".to_string(),
            ));
        }
        if receipt.family_id != self.family.family_id() || receipt.record != self.record_name {
            return Err(FamilySaveError::PreviewStateInvalid(
                "the durable receipt does not identify this previewed record".to_string(),
            ));
        }

        if let Some(dead_man) = self.dead_man.as_mut() {
            dead_man.confirm();
        }
        self.phase = RecordPickerPhase::Saved;
        self.original = None;
        self.dead_man = None;
        Ok(receipt)
    }

    fn revert_internal(
        &mut self,
        action: &'static str,
    ) -> Result<RecordPickerReceipt, RecordPickerError> {
        let original = self.original.clone().ok_or(RecordPickerError::NotArmed)?;
        let (expression, original_text, values) = match original {
            CapturedOriginal::Animation(record) => {
                let speed: f64 = record.speed.parse().map_err(|_| {
                    RecordPickerError::VerificationFailed("original speed did not parse".into())
                })?;
                (
                    render_animation_preview_expression(&record, speed)
                        .map_err(RecordPickerError::InvalidValue)?,
                    format!(
                        "enabled {}, speed {}, bezier {}",
                        record.enabled,
                        record.speed,
                        if record.bezier.is_empty() {
                            "default"
                        } else {
                            &record.bezier
                        }
                    ),
                    None,
                )
            }
            CapturedOriginal::Curve(record) => (
                render_curve_preview_expression(
                    &record.name,
                    record.x0,
                    record.y0,
                    record.x1,
                    record.y1,
                )
                .map_err(RecordPickerError::InvalidValue)?,
                format!(
                    "({}, {}, {}, {})",
                    record.x0, record.y0, record.x1, record.y1
                ),
                Some(PickedRecordValues::CurvePoints {
                    x0: record.x0,
                    y0: record.y0,
                    x1: record.x1,
                    y1: record.y1,
                }),
            ),
        };
        self.runner
            .run("hyprctl", &["eval".to_string(), expression])
            .map_err(RecordPickerError::Runner)?;
        // Verify the exact restore through readback. Animation reverts
        // compare the FULL record against the captured original (zero
        // residue — proven to hold even for disabled originals, whose reset
        // readback values are canonical).
        match values {
            Some(curve_values) => self.verify_values(&curve_values)?,
            None => {
                let observed = self.read_animation_record()?;
                if let Some(CapturedOriginal::Animation(original_record)) = &self.original {
                    if !animation_records_equivalent(&observed, original_record) {
                        return Err(RecordPickerError::VerificationFailed(format!(
                            "full-record restore mismatch: original {original_record:?}, observed {observed:?}"
                        )));
                    }
                }
            }
        }
        let record = self.record_name.clone();
        Ok(self.receipt(
            action,
            Some(original_text.clone()),
            format!(
                "Reverted: record {record} restored to {original_text} (verified via readback)."
            ),
        ))
    }

    /// Manual revert (works during countdown and after Keep).
    pub fn revert_now(&mut self) -> Result<RecordPickerReceipt, RecordPickerError> {
        if !matches!(
            self.phase,
            RecordPickerPhase::CountingDown | RecordPickerPhase::Kept
        ) {
            return Err(RecordPickerError::NotArmed);
        }
        let receipt = self.revert_internal("revert-now")?;
        self.phase = RecordPickerPhase::Reverted;
        Ok(receipt)
    }

    /// Cancel: revert and disarm.
    pub fn cancel(&mut self) -> Result<RecordPickerReceipt, RecordPickerError> {
        let mut receipt = self.revert_internal("cancel")?;
        self.phase = RecordPickerPhase::Cancelled;
        self.original = None;
        self.dead_man = None;
        receipt.phase = RecordPickerPhase::Cancelled;
        Ok(receipt)
    }

    /// Session-drop / app-close recovery: revert unconfirmed previews only.
    pub fn revert_if_unconfirmed(&mut self) -> Option<RecordPickerReceipt> {
        if self.phase == RecordPickerPhase::CountingDown {
            let receipt = self.revert_internal("session-drop").ok()?;
            self.phase = RecordPickerPhase::Reverted;
            return Some(receipt);
        }
        None
    }

    fn receipt(
        &self,
        action: &'static str,
        applied: Option<String>,
        status_text: String,
    ) -> RecordPickerReceipt {
        RecordPickerReceipt {
            family_id: self.family.family_id(),
            record: self.record_name.clone(),
            action,
            phase: self.phase,
            original: self.original.as_ref().map(|original| match original {
                CapturedOriginal::Animation(record) => record.speed.clone(),
                CapturedOriginal::Curve(record) => format!(
                    "({}, {}, {}, {})",
                    record.x0, record.y0, record.x1, record.y1
                ),
            }),
            applied,
            config_written: false,
            reload_run: false,
            status_text,
        }
    }
}

/// Save a picked record through the gated persistence path. This is the only
/// save entry the picker exposes, and it only builds the request — every
/// gate (proof receipt, Safe Live Save Mode, target identity, backup, one
/// atomic write, reread verification) runs inside the persistence module.
pub fn save_picked_record(
    runner: &mut dyn RuntimePreviewRunner,
    discovery: &ConfigDiscovery,
    family: PickedFamily,
    record_name: &str,
    values: PickedRecordValues,
) -> Result<FamilySaveReceipt, FamilySaveError> {
    let request = match (family, values) {
        (
            PickedFamily::Animation,
            PickedRecordValues::AnimationRecord {
                enabled,
                speed,
                bezier,
            },
        ) => FamilyRecordSaveRequest::AnimationRecordFields {
            record: record_name.to_string(),
            enabled,
            speed,
            bezier,
        },
        (PickedFamily::Curve, PickedRecordValues::CurvePoints { x0, y0, x1, y1 }) => {
            FamilyRecordSaveRequest::CurveRecordPoints {
                record: record_name.to_string(),
                x0,
                y0,
                x1,
                y1,
            }
        }
        _ => {
            return Err(FamilySaveError::InvalidValue(
                "values do not match the picked family".to_string(),
            ))
        }
    };
    gated_family_record_save(runner, discovery, request)
}

pub fn save_picked_record_with_precondition(
    runner: &mut dyn RuntimePreviewRunner,
    discovery: &ConfigDiscovery,
    family: PickedFamily,
    record_name: &str,
    values: PickedRecordValues,
    precondition: &FamilySavePrecondition,
) -> Result<FamilySaveReceipt, FamilySaveError> {
    let request = match (family, values) {
        (
            PickedFamily::Animation,
            PickedRecordValues::AnimationRecord {
                enabled,
                speed,
                bezier,
            },
        ) => FamilyRecordSaveRequest::AnimationRecordFields {
            record: record_name.to_string(),
            enabled,
            speed,
            bezier,
        },
        (PickedFamily::Curve, PickedRecordValues::CurvePoints { x0, y0, x1, y1 }) => {
            FamilyRecordSaveRequest::CurveRecordPoints {
                record: record_name.to_string(),
                x0,
                y0,
                x1,
                y1,
            }
        }
        _ => {
            return Err(FamilySaveError::InvalidValue(
                "values do not match the picked family".to_string(),
            ))
        }
    };
    gated_family_record_save_with_precondition(runner, discovery, request, precondition)
}

/// Live wrapper owning the runner, so UI code never constructs one.
pub fn save_picked_record_live(
    discovery: &ConfigDiscovery,
    family: PickedFamily,
    record_name: &str,
    values: PickedRecordValues,
) -> Result<FamilySaveReceipt, FamilySaveError> {
    let mut runner = HyprctlRuntimePreviewRunner;
    save_picked_record(&mut runner, discovery, family, record_name, values)
}

pub fn save_picked_record_with_precondition_live(
    discovery: &ConfigDiscovery,
    family: PickedFamily,
    record_name: &str,
    values: PickedRecordValues,
    precondition: &FamilySavePrecondition,
) -> Result<FamilySaveReceipt, FamilySaveError> {
    let mut runner = HyprctlRuntimePreviewRunner;
    save_picked_record_with_precondition(
        &mut runner,
        discovery,
        family,
        record_name,
        values,
        precondition,
    )
}
