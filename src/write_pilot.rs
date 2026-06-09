use std::fs::{self, OpenOptions};
use std::io::Write;
use std::path::Path;
use std::time::{SystemTime, UNIX_EPOCH};

use anyhow::{anyhow, Context, Result};

use crate::config_parser::{parse_hyprland_config_text, ParsedConfigLine};
use crate::pending_change::ACTIVE_PENDING_CHANGE_SETTING;
use crate::write_safety::{WritePlan, WritePlanAction, WriteResult};

const WINDOWS_SNAP_CONFIG_KEY: &str = "general:snap:enabled";
const WINDOWS_SNAP_SETTING_ID: &str = "general.snap.enabled";

pub fn apply_windows_snap_enabled_plan(plan: &WritePlan) -> Result<WriteResult> {
    if plan.setting_id != ACTIVE_PENDING_CHANGE_SETTING {
        return Err(anyhow!(
            "write pilot only supports {}",
            ACTIVE_PENDING_CHANGE_SETTING
        ));
    }

    let original = fs::read_to_string(&plan.target_path)
        .with_context(|| format!("failed to read {}", plan.target_path.display()))?;
    let updated = match plan.action {
        WritePlanAction::ReplaceLine { line_number } => {
            replace_line_value(&original, line_number, &plan.proposed_value)?
        }
        WritePlanAction::AppendSetting => {
            append_windows_snap_setting(&original, &plan.proposed_value)
        }
    };
    atomic_write(&plan.target_path, updated.as_bytes())?;

    let verified = fs::read_to_string(&plan.target_path)
        .with_context(|| format!("failed to reread {}", plan.target_path.display()))?;
    let parsed = parse_hyprland_config_text(&plan.target_path, &verified);
    let verified_value = parsed
        .scalar_records()
        .filter(|record| record.normalized_setting_id.as_deref() == Some(WINDOWS_SNAP_SETTING_ID))
        .last()
        .and_then(|record| record.raw_value.clone());

    if verified_value.as_deref() != Some(plan.proposed_value.as_str()) {
        return Err(anyhow!(
            "write verification failed for {}; expected {}, got {:?}",
            ACTIVE_PENDING_CHANGE_SETTING,
            plan.proposed_value,
            verified_value
        ));
    }

    Ok(WriteResult {
        plan: plan.clone(),
        verified_value,
    })
}

fn replace_line_value(contents: &str, line_number: usize, proposed_value: &str) -> Result<String> {
    let had_trailing_newline = contents.ends_with('\n');
    let mut lines: Vec<String> = contents.lines().map(ToOwned::to_owned).collect();
    let index = line_number
        .checked_sub(1)
        .ok_or_else(|| anyhow!("line numbers are 1-based"))?;
    let line = lines
        .get(index)
        .ok_or_else(|| anyhow!("source line {line_number} does not exist"))?;
    ensure_windows_snap_line(line)?;
    lines[index] = replace_value_preserving_key(line, proposed_value)?;
    let mut updated = lines.join("\n");
    if had_trailing_newline {
        updated.push('\n');
    }
    Ok(updated)
}

fn append_windows_snap_setting(contents: &str, proposed_value: &str) -> String {
    let mut updated = contents.to_string();
    if !updated.is_empty() && !updated.ends_with('\n') {
        updated.push('\n');
    }
    updated.push_str(&format!("{WINDOWS_SNAP_CONFIG_KEY} = {proposed_value}\n"));
    updated
}

fn ensure_windows_snap_line(line: &str) -> Result<()> {
    let parsed = parse_hyprland_config_text("/tmp/hyprland-settings-line-check.conf", line);
    let matches = parsed.records.iter().any(is_windows_snap_record);
    if matches {
        Ok(())
    } else {
        Err(anyhow!(
            "source line does not parse as {}",
            ACTIVE_PENDING_CHANGE_SETTING
        ))
    }
}

fn is_windows_snap_record(record: &ParsedConfigLine) -> bool {
    record.normalized_setting_id.as_deref() == Some(WINDOWS_SNAP_SETTING_ID)
}

fn replace_value_preserving_key(line: &str, proposed_value: &str) -> Result<String> {
    let (before_comment, comment) = line
        .split_once('#')
        .map(|(before, comment)| (before, Some(comment)))
        .unwrap_or((line, None));
    let (key, _) = before_comment
        .split_once('=')
        .ok_or_else(|| anyhow!("source line has no scalar assignment"))?;
    let mut replaced = format!("{key}= {proposed_value}");
    if let Some(comment) = comment {
        replaced.push_str(" #");
        replaced.push_str(comment);
    }
    Ok(replaced)
}

fn atomic_write(target: &Path, bytes: &[u8]) -> Result<()> {
    let parent = target
        .parent()
        .ok_or_else(|| anyhow!("target path has no parent"))?;
    let temp_path = parent.join(format!(
        ".{}.write-{}.tmp",
        target
            .file_name()
            .and_then(|name| name.to_str())
            .unwrap_or("hyprland.conf"),
        unique_stamp()?
    ));
    {
        let mut file = OpenOptions::new()
            .write(true)
            .create_new(true)
            .open(&temp_path)
            .with_context(|| format!("failed to create temp {}", temp_path.display()))?;
        file.write_all(bytes)
            .with_context(|| format!("failed to write temp {}", temp_path.display()))?;
        file.sync_all()
            .with_context(|| format!("failed to sync temp {}", temp_path.display()))?;
    }
    fs::rename(&temp_path, target).with_context(|| {
        format!(
            "failed to replace {} from temp {}",
            target.display(),
            temp_path.display()
        )
    })?;
    Ok(())
}

fn unique_stamp() -> Result<String> {
    let nanos = SystemTime::now().duration_since(UNIX_EPOCH)?.as_nanos();
    Ok(format!("{}-{nanos}", std::process::id()))
}
