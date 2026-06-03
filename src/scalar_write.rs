use std::fs::{self, OpenOptions};
use std::io::Write;
use std::path::Path;
use std::time::{SystemTime, UNIX_EPOCH};

use anyhow::{anyhow, Context, Result};

use crate::config_parser::{parse_hyprland_config_text, ParsedConfigLine};
use crate::write_classification::{
    config_key_from_official_setting, safe_writable_official_setting,
};
use crate::write_safety::{WritePlan, WritePlanAction, WriteResult};

pub fn apply_scalar_write_plan(plan: &WritePlan) -> Result<WriteResult> {
    let official_setting = safe_writable_official_setting(&plan.setting_id)
        .ok_or_else(|| anyhow!("setting is not safe-writable: {}", plan.setting_id))?;
    let config_key = config_key_from_official_setting(official_setting);

    let original = fs::read_to_string(&plan.target_path)
        .with_context(|| format!("failed to read {}", plan.target_path.display()))?;
    let updated = match plan.action {
        WritePlanAction::ReplaceLine { line_number } => {
            replace_line_value(&original, line_number, &config_key, &plan.proposed_value)?
        }
        WritePlanAction::AppendSetting => {
            append_scalar_setting(&original, &config_key, &plan.proposed_value)
        }
    };
    atomic_write(&plan.target_path, updated.as_bytes())?;

    let verified = fs::read_to_string(&plan.target_path)
        .with_context(|| format!("failed to reread {}", plan.target_path.display()))?;
    let parsed = parse_hyprland_config_text(&plan.target_path, &verified);
    let verified_value = parsed
        .scalar_records()
        .filter(|record| record.normalized_setting_id.as_deref() == Some(official_setting))
        .last()
        .and_then(|record| record.raw_value.clone());

    if verified_value.as_deref() != Some(plan.proposed_value.as_str()) {
        return Err(anyhow!(
            "write verification failed for {}; expected {}, got {:?}",
            plan.setting_id,
            plan.proposed_value,
            verified_value
        ));
    }

    Ok(WriteResult {
        plan: plan.clone(),
        verified_value,
    })
}

fn replace_line_value(
    contents: &str,
    line_number: usize,
    config_key: &str,
    proposed_value: &str,
) -> Result<String> {
    let had_trailing_newline = contents.ends_with('\n');
    let mut lines: Vec<String> = contents.lines().map(ToOwned::to_owned).collect();
    let index = line_number
        .checked_sub(1)
        .ok_or_else(|| anyhow!("line numbers are 1-based"))?;
    let line = lines
        .get(index)
        .ok_or_else(|| anyhow!("source line {line_number} does not exist"))?;
    ensure_scalar_line(line, config_key)?;
    lines[index] = replace_value_preserving_key(line, proposed_value)?;
    let mut updated = lines.join("\n");
    if had_trailing_newline {
        updated.push('\n');
    }
    Ok(updated)
}

fn append_scalar_setting(contents: &str, config_key: &str, proposed_value: &str) -> String {
    let mut updated = contents.to_string();
    if !updated.is_empty() && !updated.ends_with('\n') {
        updated.push('\n');
    }
    updated.push_str(&format!("{config_key} = {proposed_value}\n"));
    updated
}

fn ensure_scalar_line(line: &str, config_key: &str) -> Result<()> {
    let parsed = parse_hyprland_config_text("/tmp/hyprland-settings-line-check.conf", line);
    let official_setting = config_key.replace(':', ".");
    let matches = parsed
        .records
        .iter()
        .any(|record| is_matching_scalar_record(record, &official_setting));
    if matches {
        Ok(())
    } else {
        Err(anyhow!("source line does not parse as {official_setting}"))
    }
}

fn is_matching_scalar_record(record: &ParsedConfigLine, official_setting: &str) -> bool {
    record.normalized_setting_id.as_deref() == Some(official_setting)
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
