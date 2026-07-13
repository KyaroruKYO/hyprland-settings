//! Production Save for the two proven structured-family records.
//!
//! This is the pilot's proven write shape converted to production behavior:
//! Safe Live Save Mode is verified live (autoreload disabled at runtime, so
//! the write cannot reload the compositor), a byte-exact backup is created
//! outside the config directory, the final previewed value is written once
//! (replace-or-append of the target record's own line only, preserving every
//! other line), the config is reread and verified through the parser and the
//! projection path — and on success the config is NOT restored: that is the
//! save. On verification failure the backup is restored automatically.
//!
//! Scope is exactly the proven surface: the `global` animation record and
//! the `default` bezier record. The target enum cannot express any other
//! family or record, there is no deletion, and no other record is ever
//! touched. Blocked families remain blocked.

use std::fs;
use std::path::{Path, PathBuf};

use serde::Serialize;

use crate::config_discovery::{ConfigDiscovery, ConfigDiscoveryStatus};
use crate::runtime_preview_executor::RuntimePreviewRunner;
use crate::safe_live_save_mode::require_safe_live_save_mode;
use crate::structured_family::StructuredFamilyKind;
use crate::structured_family_active_config_pilot::active_config_pilot_content_hash;
use crate::structured_family_controlled_write::{
    apply_rendered_family_records, atomic_controlled_write, family_records_in_text,
};
use crate::structured_family_preview_controller::FamilyPreviewTarget;
use crate::structured_family_runtime_preview::{
    parse_animation_records, parse_bezier_records, proven_family_record_proof,
    proven_record_shape_proof, ANIMATION_RECORD_BEZIER_SHAPE, ANIMATION_RECORD_ENABLED_SHAPE,
    ANIMATION_RECORD_SPEED_SHAPE, CURVE_RECORD_POINTS_SHAPE,
};
use crate::structured_family_write_target::structured_family_path_is_active_real_config;

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum FamilySaveError {
    SafeLiveSaveModeRequired(String),
    FamilyNotProven(&'static str),
    InvalidValue(String),
    ConfigUnavailable(String),
    TargetIdentityFailed(String),
    BackupFailed(String),
    WriteFailed(String),
    VerificationFailedAndRestored(String),
    RestoreFailed(String),
}

impl FamilySaveError {
    pub fn user_text(&self) -> String {
        match self {
            Self::SafeLiveSaveModeRequired(reason) => reason.clone(),
            Self::FamilyNotProven(reason) => format!("Save unavailable: {reason}"),
            Self::InvalidValue(detail) => format!("Save rejected: {detail}"),
            Self::ConfigUnavailable(detail) => format!("Save failed: {detail}"),
            Self::TargetIdentityFailed(detail) => format!("Save refused: {detail}"),
            Self::BackupFailed(detail) => format!("Save aborted before writing: {detail}"),
            Self::WriteFailed(detail) => format!("Save write failed: {detail}"),
            Self::VerificationFailedAndRestored(detail) => format!(
                "Save verification failed; your original config was restored: {detail}"
            ),
            Self::RestoreFailed(detail) => format!(
                "Save verification failed AND restore failed - restore manually from the backup: {detail}"
            ),
        }
    }
}

#[derive(Debug, Clone, Serialize)]
pub struct FamilySaveReceipt {
    pub family_id: &'static str,
    pub record: String,
    pub saved_value: String,
    pub rendered_line: String,
    pub config_path: PathBuf,
    pub backup_path: PathBuf,
    pub pre_save_hash: u64,
    pub post_save_hash: u64,
    pub reread_verified: bool,
    pub restored_after_success: bool,
    pub reload_run: bool,
    pub status_text: String,
}

/// Validate the value range for a target before any gate or file access.
pub fn validate_save_value(target: FamilyPreviewTarget, value: f64) -> Result<(), FamilySaveError> {
    if !value.is_finite() {
        return Err(FamilySaveError::InvalidValue(
            "value must be finite".to_string(),
        ));
    }
    match target {
        FamilyPreviewTarget::AnimationGlobalSpeed if !(0.1..=20.0).contains(&value) => {
            Err(FamilySaveError::InvalidValue(
                "global animation speed saves are limited to 0.1..=20".to_string(),
            ))
        }
        FamilyPreviewTarget::CurveDefaultY0 if !(-1.0..=2.0).contains(&value) => {
            Err(FamilySaveError::InvalidValue(
                "curve control point saves are limited to -1..=2".to_string(),
            ))
        }
        _ => Ok(()),
    }
}

fn family_kind(target: FamilyPreviewTarget) -> StructuredFamilyKind {
    match target {
        FamilyPreviewTarget::AnimationGlobalSpeed => StructuredFamilyKind::Animation,
        FamilyPreviewTarget::CurveDefaultY0 => StructuredFamilyKind::Curve,
    }
}

/// Render the target record's config line from the validated value and the
/// current runtime record (so persisted fields match what the user sees).
pub fn render_target_line(
    target: FamilyPreviewTarget,
    value: f64,
    runner: &mut dyn RuntimePreviewRunner,
) -> Result<String, FamilySaveError> {
    let listing = runner
        .run("hyprctl", &["animations".to_string()])
        .map_err(FamilySaveError::ConfigUnavailable)?;
    match target {
        FamilyPreviewTarget::AnimationGlobalSpeed => {
            if !(0.1..=20.0).contains(&value) {
                return Err(FamilySaveError::InvalidValue(
                    "global animation speed saves are limited to 0.1..=20".to_string(),
                ));
            }
            let (enabled, _speed, bezier) =
                crate::structured_family_runtime_preview::parse_animation_leaf(&listing, "global")
                    .ok_or_else(|| {
                        FamilySaveError::ConfigUnavailable(
                            "global animation record not readable".to_string(),
                        )
                    })?;
            let onoff = if enabled == "1" { "1" } else { "0" };
            let bezier_name = if bezier.is_empty() {
                "default"
            } else {
                &bezier
            };
            Ok(format!(
                "animation = global, {onoff}, {value}, {bezier_name}"
            ))
        }
        FamilyPreviewTarget::CurveDefaultY0 => {
            if !(-1.0..=2.0).contains(&value) {
                return Err(FamilySaveError::InvalidValue(
                    "curve control point saves are limited to -1..=2".to_string(),
                ));
            }
            let (x0, _y0, x1, y1) =
                crate::structured_family_runtime_preview::parse_bezier_points(&listing, "default")
                    .ok_or_else(|| {
                        FamilySaveError::ConfigUnavailable(
                            "default bezier record not readable".to_string(),
                        )
                    })?;
            Ok(format!("bezier = default, {x0}, {value}, {x1}, {y1}"))
        }
    }
}

/// True when the raw family line's record name equals `record_name`.
pub fn record_line_matches_name(raw_line: &str, record_name: &str) -> bool {
    let value_part = raw_line.splitn(2, '=').nth(1).unwrap_or("").trim();
    let name = value_part.split(',').next().unwrap_or("").trim();
    name == record_name
}

pub fn record_matches_target(target: FamilyPreviewTarget, raw_line: &str) -> bool {
    record_line_matches_name(raw_line, target.record())
}

fn backup_root() -> PathBuf {
    std::env::temp_dir().join("hyprland-settings-family-save-backups")
}

/// The production Save. Verifies the Safe Live Save Mode gate live, writes
/// the final previewed value once, verifies, and does NOT restore on
/// success. Only the target record's own line is replaced or appended;
/// every other line is preserved.
pub fn gated_family_save(
    runner: &mut dyn RuntimePreviewRunner,
    discovery: &ConfigDiscovery,
    target: FamilyPreviewTarget,
    value: f64,
) -> Result<FamilySaveReceipt, FamilySaveError> {
    // Gate 1: only receipt-proven families can save.
    if proven_family_record_proof(target.family_id()).is_none() {
        return Err(FamilySaveError::FamilyNotProven(
            "no passed live proof receipt exists for this family",
        ));
    }
    validate_save_value(target, value)?;

    // Gate 2: Safe Live Save Mode must be active (live-verified) so this
    // write cannot reload the compositor.
    require_safe_live_save_mode(runner).map_err(FamilySaveError::SafeLiveSaveModeRequired)?;

    // Gate 3: the target must be the discovered active config file.
    let config_path = match &discovery.status {
        ConfigDiscoveryStatus::Found { path, .. } => path.clone(),
        other => {
            return Err(FamilySaveError::ConfigUnavailable(format!(
                "no config file discovered: {other:?}"
            )))
        }
    };
    if !structured_family_path_is_active_real_config(&config_path) {
        return Err(FamilySaveError::TargetIdentityFailed(format!(
            "{} is not the active Hyprland config",
            config_path.display()
        )));
    }

    // Render the record line from the validated value + current runtime.
    let rendered_line = render_target_line(target, value, runner)?;
    let kind = family_kind(target);

    persist_rendered_record_line(
        &config_path,
        kind,
        target.family_id(),
        target.record(),
        rendered_line,
        format!("{value}"),
    )
}

/// The shared persist tail every gated family save uses after its gates
/// passed: read original bytes, byte-exact backup outside the config
/// directory, replace-or-append the target record's own line only, one
/// atomic write, reread verification through the parser, and automatic
/// restore on verification failure. Never called before the Safe Live Save
/// Mode and target identity gates.
fn persist_rendered_record_line(
    config_path: &Path,
    kind: StructuredFamilyKind,
    family_id: &'static str,
    record_name: &str,
    rendered_line: String,
    saved_value: String,
) -> Result<FamilySaveReceipt, FamilySaveError> {
    // Read original bytes and hash.
    let original = fs::read(config_path)
        .map_err(|error| FamilySaveError::ConfigUnavailable(error.to_string()))?;
    let pre_save_hash = active_config_pilot_content_hash(&original);
    let original_text = String::from_utf8_lossy(&original).into_owned();

    // Build the family record list: replace the target record if present,
    // otherwise append it. Other records of the family are preserved as-is.
    let existing: Vec<String> = family_records_in_text(config_path, &original_text, kind)
        .into_iter()
        .map(|(_, raw_line)| raw_line)
        .collect();
    let mut replaced = false;
    let mut rendered_records: Vec<String> = existing
        .iter()
        .map(|line| {
            if record_line_matches_name(line, record_name) {
                replaced = true;
                rendered_line.clone()
            } else {
                line.clone()
            }
        })
        .collect();
    if !replaced {
        rendered_records.push(rendered_line.clone());
    }

    // Byte-exact backup outside the config directory, verified readable.
    let root = backup_root();
    fs::create_dir_all(&root).map_err(|error| FamilySaveError::BackupFailed(error.to_string()))?;
    let backup_path = root.join(format!(
        "hyprland.conf.family-save-{}-{}.bak",
        record_name,
        std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .map(|duration| duration.as_secs())
            .unwrap_or(0)
    ));
    fs::write(&backup_path, &original)
        .map_err(|error| FamilySaveError::BackupFailed(error.to_string()))?;
    let backup_bytes =
        fs::read(&backup_path).map_err(|error| FamilySaveError::BackupFailed(error.to_string()))?;
    if backup_bytes != original {
        return Err(FamilySaveError::BackupFailed(
            "backup is not byte-exact".to_string(),
        ));
    }

    // One atomic write of the updated family records.
    let new_text =
        apply_rendered_family_records(config_path, &original_text, kind, &rendered_records);
    atomic_controlled_write(config_path, new_text.as_bytes())
        .map_err(|error| FamilySaveError::WriteFailed(error.to_string()))?;

    // Reread and verify the intended record persisted through the parser.
    let verify = verify_saved_record_named(config_path, kind, record_name, &rendered_line);
    match verify {
        Ok(()) => {}
        Err(detail) => {
            // Fail closed: restore the backup before reporting.
            return match atomic_controlled_write(config_path, &original) {
                Ok(()) => Err(FamilySaveError::VerificationFailedAndRestored(detail)),
                Err(error) => Err(FamilySaveError::RestoreFailed(format!(
                    "{detail}; restore error: {error}; backup at {}",
                    backup_path.display()
                ))),
            };
        }
    }
    let post_save_hash = fs::read(config_path)
        .map(|bytes| active_config_pilot_content_hash(&bytes))
        .map_err(|error| FamilySaveError::WriteFailed(error.to_string()))?;

    Ok(FamilySaveReceipt {
        family_id,
        record: record_name.to_string(),
        saved_value,
        rendered_line: rendered_line.clone(),
        config_path: config_path.to_path_buf(),
        backup_path: backup_path.clone(),
        pre_save_hash,
        post_save_hash,
        reread_verified: true,
        restored_after_success: false,
        reload_run: false,
        status_text: format!(
            "Saved: `{rendered_line}` persisted to {} (backup: {}; reread-verified; no reload - Safe Live Save Mode active).",
            config_path.display(),
            backup_path.display()
        ),
    })
}

pub fn verify_saved_record(
    config_path: &Path,
    kind: StructuredFamilyKind,
    target: FamilyPreviewTarget,
    rendered_line: &str,
) -> Result<(), String> {
    verify_saved_record_named(config_path, kind, target.record(), rendered_line)
}

pub fn verify_saved_record_named(
    config_path: &Path,
    kind: StructuredFamilyKind,
    record_name: &str,
    rendered_line: &str,
) -> Result<(), String> {
    let contents =
        fs::read_to_string(config_path).map_err(|error| format!("reread failed: {error}"))?;
    let records = family_records_in_text(config_path, &contents, kind);
    let found = records
        .iter()
        .filter(|(_, line)| record_line_matches_name(line, record_name))
        .collect::<Vec<_>>();
    if found.len() != 1 {
        return Err(format!(
            "expected exactly one {record_name} record after save, found {}",
            found.len()
        ));
    }
    if found[0].1.trim() != rendered_line.trim() {
        return Err(format!(
            "persisted record mismatch: expected {rendered_line:?}, found {:?}",
            found[0].1
        ));
    }
    Ok(())
}

/// A picked-record save request: exactly the proven record shapes. The
/// enum cannot express any other family, field, or operation — there is no
/// record creation request and no removal request.
#[derive(Debug, Clone, PartialEq)]
pub enum FamilyRecordSaveRequest {
    /// Persist the proven fields (enabled, speed, bezier) of an animation
    /// record that already carries an explicit override. The style is
    /// re-rendered from the readback (preserved, never edited), and the
    /// bezier must name a curve that already exists in the readback. Each
    /// field is covered by its own passed shape-proof receipt.
    AnimationRecordFields {
        record: String,
        enabled: bool,
        speed: f64,
        bezier: String,
    },
    /// Persist the four control points of a bezier curve that already
    /// exists in the runtime readback.
    CurveRecordPoints {
        record: String,
        x0: f64,
        y0: f64,
        x1: f64,
        y1: f64,
    },
}

impl FamilyRecordSaveRequest {
    pub fn family_id(&self) -> &'static str {
        match self {
            Self::AnimationRecordFields { .. } => "hl.animation",
            Self::CurveRecordPoints { .. } => "hl.curve",
        }
    }

    pub fn record(&self) -> &str {
        match self {
            Self::AnimationRecordFields { record, .. } => record,
            Self::CurveRecordPoints { record, .. } => record,
        }
    }

    fn kind(&self) -> StructuredFamilyKind {
        match self {
            Self::AnimationRecordFields { .. } => StructuredFamilyKind::Animation,
            Self::CurveRecordPoints { .. } => StructuredFamilyKind::Curve,
        }
    }

    /// Every shape receipt the request depends on. A field may only be
    /// carried by a request whose shape passed a live proof, so the
    /// combined animation request requires all three animation shapes.
    fn required_shapes(&self) -> &'static [&'static str] {
        match self {
            Self::AnimationRecordFields { .. } => &[
                ANIMATION_RECORD_SPEED_SHAPE,
                ANIMATION_RECORD_ENABLED_SHAPE,
                ANIMATION_RECORD_BEZIER_SHAPE,
            ],
            Self::CurveRecordPoints { .. } => &[CURVE_RECORD_POINTS_SHAPE],
        }
    }

    fn validate(&self) -> Result<(), FamilySaveError> {
        let record_name_safe = |name: &str| {
            !name.is_empty()
                && name
                    .chars()
                    .all(|character| character.is_ascii_alphanumeric() || character == '_')
        };
        if !record_name_safe(self.record()) || self.record().starts_with("__") {
            return Err(FamilySaveError::InvalidValue(
                "record name is not in the safe user-record set".to_string(),
            ));
        }
        match self {
            Self::AnimationRecordFields { speed, bezier, .. } => {
                if !speed.is_finite() || !(0.1..=20.0).contains(speed) {
                    return Err(FamilySaveError::InvalidValue(
                        "animation speed saves are limited to 0.1..=20".to_string(),
                    ));
                }
                if !record_name_safe(bezier) || bezier.starts_with("__") {
                    return Err(FamilySaveError::InvalidValue(
                        "bezier name is not in the safe user-record set".to_string(),
                    ));
                }
            }
            Self::CurveRecordPoints { x0, y0, x1, y1, .. } => {
                for x in [x0, x1] {
                    if !x.is_finite() || !(0.0..=1.0).contains(x) {
                        return Err(FamilySaveError::InvalidValue(
                            "curve X control point saves are limited to 0..=1".to_string(),
                        ));
                    }
                }
                for y in [y0, y1] {
                    if !y.is_finite() || !(-1.0..=2.0).contains(y) {
                        return Err(FamilySaveError::InvalidValue(
                            "curve Y control point saves are limited to -1..=2".to_string(),
                        ));
                    }
                }
            }
        }
        Ok(())
    }

    fn saved_value_text(&self) -> String {
        match self {
            Self::AnimationRecordFields {
                enabled,
                speed,
                bezier,
                ..
            } => {
                format!(
                    "enabled {}, speed {speed}, bezier {bezier}",
                    if *enabled { "1" } else { "0" }
                )
            }
            Self::CurveRecordPoints { x0, y0, x1, y1, .. } => {
                format!("({x0}, {y0}, {x1}, {y1})")
            }
        }
    }
}

/// Render the request's config line from the runtime readback, enforcing
/// modify-existing: the record must already exist in the readback, and an
/// animation record must already carry an explicit override (persisting an
/// inherited record would create a new override — creation is blocked).
/// The animation bezier must name a curve that already exists in the
/// readback (only existing curves can be referenced), and the style is
/// preserved exactly as the readback reports it (never edited here).
pub fn render_record_request_line(
    request: &FamilyRecordSaveRequest,
    runner: &mut dyn RuntimePreviewRunner,
) -> Result<String, FamilySaveError> {
    let listing = runner
        .run("hyprctl", &["animations".to_string()])
        .map_err(FamilySaveError::ConfigUnavailable)?;
    match request {
        FamilyRecordSaveRequest::AnimationRecordFields {
            record,
            enabled,
            speed,
            bezier,
        } => {
            let runtime_record = parse_animation_records(&listing)
                .into_iter()
                .find(|candidate| candidate.name == *record)
                .ok_or_else(|| {
                    FamilySaveError::ConfigUnavailable(format!(
                        "animation record {record} not present in the runtime readback"
                    ))
                })?;
            if !runtime_record.overridden {
                return Err(FamilySaveError::InvalidValue(format!(
                    "animation record {record} inherits its values; saving it would create a new override, and record creation is blocked"
                )));
            }
            if !parse_bezier_records(&listing)
                .into_iter()
                .any(|curve| curve.name == *bezier)
            {
                return Err(FamilySaveError::InvalidValue(format!(
                    "bezier {bezier} does not exist in the runtime readback; only existing curves can be referenced"
                )));
            }
            let onoff = if *enabled { "1" } else { "0" };
            if runtime_record.style.is_empty() {
                Ok(format!("animation = {record}, {onoff}, {speed}, {bezier}"))
            } else {
                Ok(format!(
                    "animation = {record}, {onoff}, {speed}, {bezier}, {}",
                    runtime_record.style
                ))
            }
        }
        FamilyRecordSaveRequest::CurveRecordPoints {
            record,
            x0,
            y0,
            x1,
            y1,
        } => {
            parse_bezier_records(&listing)
                .into_iter()
                .find(|candidate| candidate.name == *record)
                .ok_or_else(|| {
                    FamilySaveError::ConfigUnavailable(format!(
                        "curve record {record} not present in the runtime readback"
                    ))
                })?;
            Ok(format!("bezier = {record}, {x0}, {y0}, {x1}, {y1}"))
        }
    }
}

/// The generalized picked-record Save: the same gate chain as
/// `gated_family_save`, with the record shape proof receipt required in
/// place of the single-record family proof. Verifies Safe Live Save Mode
/// live, enforces target identity, renders the record line from the
/// readback (modify-existing enforced), then persists through the shared
/// backup/one-atomic-write/reread-verification tail.
pub fn gated_family_record_save(
    runner: &mut dyn RuntimePreviewRunner,
    discovery: &ConfigDiscovery,
    request: FamilyRecordSaveRequest,
) -> Result<FamilySaveReceipt, FamilySaveError> {
    // Gate 1: every record shape the request carries must have a passed
    // live-proof receipt.
    for shape in request.required_shapes() {
        if proven_record_shape_proof(request.family_id(), shape).is_none() {
            return Err(FamilySaveError::FamilyNotProven(
                "no passed live proof receipt exists for this record shape",
            ));
        }
    }
    request.validate()?;

    // Gate 2: Safe Live Save Mode must be active (live-verified) so this
    // write cannot reload the compositor.
    require_safe_live_save_mode(runner).map_err(FamilySaveError::SafeLiveSaveModeRequired)?;

    // Gate 3: the target must be the discovered active config file.
    let config_path = match &discovery.status {
        ConfigDiscoveryStatus::Found { path, .. } => path.clone(),
        other => {
            return Err(FamilySaveError::ConfigUnavailable(format!(
                "no config file discovered: {other:?}"
            )))
        }
    };
    if !structured_family_path_is_active_real_config(&config_path) {
        return Err(FamilySaveError::TargetIdentityFailed(format!(
            "{} is not the active Hyprland config",
            config_path.display()
        )));
    }

    // Render from the readback (modify-existing enforced inside).
    let rendered_line = render_record_request_line(&request, runner)?;

    persist_rendered_record_line(
        &config_path,
        request.kind(),
        request.family_id(),
        request.record(),
        rendered_line,
        request.saved_value_text(),
    )
}

/// Live wrapper owning the runner, so UI code never constructs one.
pub fn gated_family_record_save_live(
    discovery: &ConfigDiscovery,
    request: FamilyRecordSaveRequest,
) -> Result<FamilySaveReceipt, FamilySaveError> {
    let mut runner = crate::runtime_preview_executor::HyprctlRuntimePreviewRunner;
    gated_family_record_save(&mut runner, discovery, request)
}

/// Live wrapper owning the runner, so UI code never constructs one.
pub fn gated_family_save_live(
    discovery: &ConfigDiscovery,
    target: FamilyPreviewTarget,
    value: f64,
) -> Result<FamilySaveReceipt, FamilySaveError> {
    let mut runner = crate::runtime_preview_executor::HyprctlRuntimePreviewRunner;
    gated_family_save(&mut runner, discovery, target, value)
}
