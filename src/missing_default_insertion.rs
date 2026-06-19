use std::fs;
use std::path::{Path, PathBuf};

use anyhow::{anyhow, Context, Result};

use crate::config_parser::{parse_hyprland_config_file, ParseStatus};
use crate::write_classification::{
    config_key_from_official_setting, high_risk_write_policy, is_safe_writable_setting,
    safe_writable_official_setting,
};

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct MissingDefaultInsertionRequest {
    pub setting_id: String,
    pub proposed_value: String,
    pub target_path: PathBuf,
    pub backup_stamp: String,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct MissingDefaultInsertionPlan {
    pub setting_id: String,
    pub official_setting: String,
    pub config_key: String,
    pub proposed_value: String,
    pub target_path: PathBuf,
    pub backup_path: PathBuf,
    pub insertion_line: String,
    pub can_execute: bool,
    pub blocked_reasons: Vec<String>,
    pub production_enabled: bool,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct MissingDefaultInsertionOptions {
    pub force_write_failure: bool,
    pub force_verification_failure: bool,
    pub force_restore_failure: bool,
}

impl Default for MissingDefaultInsertionOptions {
    fn default() -> Self {
        Self {
            force_write_failure: false,
            force_verification_failure: false,
            force_restore_failure: false,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum MissingDefaultInsertionStatus {
    Blocked,
    Succeeded,
    RecoveredFailure,
    UnrecoveredFailure,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct MissingDefaultInsertionReport {
    pub status: MissingDefaultInsertionStatus,
    pub backup_created: bool,
    pub backup_bytes_equal: bool,
    pub inserted_line_verified: bool,
    pub recovery_attempted: bool,
    pub recovery_succeeded: bool,
    pub failures: Vec<String>,
    pub real_config_touched: bool,
    pub runtime_touched: bool,
    pub production_behavior_enabled: bool,
}

pub fn build_missing_default_insertion_plan(
    request: MissingDefaultInsertionRequest,
) -> MissingDefaultInsertionPlan {
    let mut blocked_reasons = Vec::new();
    let official_setting = safe_writable_official_setting(&request.setting_id)
        .unwrap_or(request.setting_id.as_str())
        .to_string();
    let config_key = config_key_from_official_setting(&official_setting);

    if !is_safe_writable_setting(&request.setting_id) {
        blocked_reasons.push("setting is not in the safe writable row table".to_string());
    }
    if high_risk_write_policy(&request.setting_id).is_some() {
        blocked_reasons.push("high-risk settings cannot use default insertion".to_string());
    }
    if display_render_risky(&request.setting_id) {
        blocked_reasons
            .push("display/render-risk settings cannot use default insertion".to_string());
    }
    if request.setting_id.starts_with("hl.") {
        blocked_reasons.push("structured-family settings cannot use scalar insertion".to_string());
    }
    if request.setting_id.contains("profile") || request.setting_id.contains("mode") {
        blocked_reasons.push("profile/mode settings cannot use default insertion".to_string());
    }
    if !request.target_path.is_file() {
        blocked_reasons.push("target file must already exist".to_string());
    }
    if is_symlink(&request.target_path) {
        blocked_reasons.push("target file must not be a symlink".to_string());
    }
    if setting_already_configured(&request.target_path, &official_setting).unwrap_or(false) {
        blocked_reasons
            .push("setting is already configured; duplicate resolution is required".to_string());
    }

    let backup_path = request
        .target_path
        .with_extension(format!("missing-default-backup-{}", request.backup_stamp));
    let insertion_line = format!("{config_key} = {}", request.proposed_value.trim());

    MissingDefaultInsertionPlan {
        setting_id: request.setting_id,
        official_setting,
        config_key,
        proposed_value: request.proposed_value,
        target_path: request.target_path,
        backup_path,
        insertion_line,
        can_execute: blocked_reasons.is_empty(),
        blocked_reasons,
        production_enabled: false,
    }
}

pub fn execute_missing_default_insertion_plan(
    plan: &MissingDefaultInsertionPlan,
    options: &MissingDefaultInsertionOptions,
) -> MissingDefaultInsertionReport {
    if !plan.can_execute {
        return MissingDefaultInsertionReport {
            status: MissingDefaultInsertionStatus::Blocked,
            backup_created: false,
            backup_bytes_equal: false,
            inserted_line_verified: false,
            recovery_attempted: false,
            recovery_succeeded: false,
            failures: plan.blocked_reasons.clone(),
            real_config_touched: false,
            runtime_touched: false,
            production_behavior_enabled: false,
        };
    }

    let original = match fs::read(&plan.target_path) {
        Ok(bytes) => bytes,
        Err(error) => return failed_without_recovery(format!("read target failed: {error}")),
    };
    if let Err(error) = fs::write(&plan.backup_path, &original) {
        return failed_without_recovery(format!("backup write failed: {error}"));
    }
    let backup = match fs::read(&plan.backup_path) {
        Ok(bytes) => bytes,
        Err(error) => return failed_without_recovery(format!("backup reread failed: {error}")),
    };
    if backup != original {
        return failed_without_recovery("backup byte equality failed".to_string());
    }

    if options.force_write_failure {
        return recover(plan, &original, "forced insertion write failure", options);
    }

    let mut updated = String::from_utf8_lossy(&original).into_owned();
    if !updated.ends_with('\n') {
        updated.push('\n');
    }
    updated.push_str("\n# Added by Hyprland Settings safe-env missing/default insertion proof\n");
    updated.push_str(&plan.insertion_line);
    updated.push('\n');

    if let Err(error) = fs::write(&plan.target_path, updated.as_bytes()) {
        return recover(
            plan,
            &original,
            &format!("insertion write failed: {error}"),
            options,
        );
    }

    if options.force_verification_failure {
        return recover(
            plan,
            &original,
            "forced insertion verification failure",
            options,
        );
    }

    match inserted_setting_matches(
        &plan.target_path,
        &plan.official_setting,
        &plan.proposed_value,
    ) {
        Ok(true) => MissingDefaultInsertionReport {
            status: MissingDefaultInsertionStatus::Succeeded,
            backup_created: true,
            backup_bytes_equal: true,
            inserted_line_verified: true,
            recovery_attempted: false,
            recovery_succeeded: false,
            failures: Vec::new(),
            real_config_touched: false,
            runtime_touched: false,
            production_behavior_enabled: false,
        },
        Ok(false) => recover(plan, &original, "inserted value did not verify", options),
        Err(error) => recover(
            plan,
            &original,
            &format!("verification read failed: {error}"),
            options,
        ),
    }
}

fn recover(
    plan: &MissingDefaultInsertionPlan,
    original: &[u8],
    failure: &str,
    options: &MissingDefaultInsertionOptions,
) -> MissingDefaultInsertionReport {
    if options.force_restore_failure {
        return MissingDefaultInsertionReport {
            status: MissingDefaultInsertionStatus::UnrecoveredFailure,
            backup_created: true,
            backup_bytes_equal: true,
            inserted_line_verified: false,
            recovery_attempted: true,
            recovery_succeeded: false,
            failures: vec![failure.to_string(), "forced restore failure".to_string()],
            real_config_touched: false,
            runtime_touched: false,
            production_behavior_enabled: false,
        };
    }

    let restore_result = fs::write(&plan.target_path, original);
    let restored = restore_result
        .and_then(|_| fs::read(&plan.target_path))
        .map(|bytes| bytes == original)
        .unwrap_or(false);
    MissingDefaultInsertionReport {
        status: if restored {
            MissingDefaultInsertionStatus::RecoveredFailure
        } else {
            MissingDefaultInsertionStatus::UnrecoveredFailure
        },
        backup_created: true,
        backup_bytes_equal: true,
        inserted_line_verified: false,
        recovery_attempted: true,
        recovery_succeeded: restored,
        failures: vec![failure.to_string()],
        real_config_touched: false,
        runtime_touched: false,
        production_behavior_enabled: false,
    }
}

fn failed_without_recovery(failure: String) -> MissingDefaultInsertionReport {
    MissingDefaultInsertionReport {
        status: MissingDefaultInsertionStatus::UnrecoveredFailure,
        backup_created: false,
        backup_bytes_equal: false,
        inserted_line_verified: false,
        recovery_attempted: false,
        recovery_succeeded: false,
        failures: vec![failure],
        real_config_touched: false,
        runtime_touched: false,
        production_behavior_enabled: false,
    }
}

fn setting_already_configured(path: &Path, official_setting: &str) -> Result<bool> {
    if !path.is_file() {
        return Ok(false);
    }
    let parsed = parse_hyprland_config_file(path)
        .with_context(|| format!("failed to parse {}", path.display()))?;
    Ok(parsed.records.iter().any(|record| {
        record.status == ParseStatus::Scalar
            && record.normalized_setting_id.as_deref() == Some(official_setting)
    }))
}

fn inserted_setting_matches(
    path: &Path,
    official_setting: &str,
    expected_value: &str,
) -> Result<bool> {
    let parsed = parse_hyprland_config_file(path)
        .with_context(|| format!("failed to parse {}", path.display()))?;
    let matches = parsed
        .records
        .iter()
        .filter(|record| {
            record.status == ParseStatus::Scalar
                && record.normalized_setting_id.as_deref() == Some(official_setting)
        })
        .collect::<Vec<_>>();
    if matches.len() != 1 {
        return Err(anyhow!(
            "expected one inserted occurrence for {official_setting}, found {}",
            matches.len()
        ));
    }
    Ok(matches[0].raw_value.as_deref() == Some(expected_value.trim()))
}

fn is_symlink(path: &Path) -> bool {
    fs::symlink_metadata(path)
        .map(|metadata| metadata.file_type().is_symlink())
        .unwrap_or(false)
}

fn display_render_risky(setting_id: &str) -> bool {
    setting_id == "decoration.screen_shader"
        || matches!(
            setting_id.split('.').next(),
            Some("render" | "xwayland" | "opengl" | "experimental" | "quirks")
        )
}
