use std::collections::BTreeMap;
use std::fs;
use std::path::{Path, PathBuf};

use anyhow::{Context, Result};

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ParsedConfig {
    pub path: PathBuf,
    pub records: Vec<ParsedConfigLine>,
    pub duplicate_scalar_keys: BTreeMap<String, Vec<usize>>,
}

impl ParsedConfig {
    pub fn scalar_records(&self) -> impl Iterator<Item = &ParsedConfigLine> {
        self.records
            .iter()
            .filter(|record| record.status == ParseStatus::Scalar)
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ParsedConfigLine {
    pub path: PathBuf,
    pub line_number: usize,
    pub raw_line: String,
    pub parsed_key: Option<String>,
    pub raw_value: Option<String>,
    pub normalized_setting_id: Option<String>,
    pub status: ParseStatus,
    pub warning: Option<String>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ParseStatus {
    Blank,
    Comment,
    Scalar,
    StructuredRaw,
    Unsupported,
}

pub fn parse_hyprland_config_file(path: impl AsRef<Path>) -> Result<ParsedConfig> {
    let path = path.as_ref();
    let contents = fs::read_to_string(path)
        .with_context(|| format!("failed to read Hyprland config fixture {}", path.display()))?;
    Ok(parse_hyprland_config_text(path, &contents))
}

pub fn parse_hyprland_config_text(path: impl AsRef<Path>, contents: &str) -> ParsedConfig {
    let path = path.as_ref().to_path_buf();
    let mut records = Vec::new();
    let mut in_device_block = false;

    for (index, raw_line) in contents.lines().enumerate() {
        let line_number = index + 1;
        let trimmed = raw_line.trim();
        if in_device_block {
            let ends_block = trimmed == "}";
            records.push(structured_record(
                &path,
                line_number,
                raw_line,
                Some("hl.device"),
                "device block preserved as raw structured metadata",
            ));
            if ends_block {
                in_device_block = false;
            }
            continue;
        }

        let record = parse_line(&path, line_number, raw_line);
        if record.normalized_setting_id.as_deref() == Some("hl.device") && trimmed.ends_with('{') {
            in_device_block = true;
        }
        records.push(record);
    }

    let duplicate_scalar_keys = duplicate_scalar_keys(&records);
    ParsedConfig {
        path,
        records,
        duplicate_scalar_keys,
    }
}

fn parse_line(path: &Path, line_number: usize, raw_line: &str) -> ParsedConfigLine {
    let trimmed = raw_line.trim();
    if trimmed.is_empty() {
        return base_record(path, line_number, raw_line, ParseStatus::Blank);
    }
    if trimmed.starts_with('#') {
        return base_record(path, line_number, raw_line, ParseStatus::Comment);
    }

    let without_comment = strip_inline_comment(trimmed).trim();
    if without_comment.is_empty() {
        return base_record(path, line_number, raw_line, ParseStatus::Comment);
    }

    if let Some(record) = parse_structured_line(path, line_number, raw_line, without_comment) {
        return record;
    }

    if let Some((key, value)) = without_comment.split_once('=') {
        let parsed_key = key.trim();
        let raw_value = value.trim();
        if parsed_key.is_empty() {
            return unsupported_record(path, line_number, raw_line, "missing setting key");
        }
        return ParsedConfigLine {
            path: path.to_path_buf(),
            line_number,
            raw_line: raw_line.to_string(),
            parsed_key: Some(parsed_key.to_string()),
            raw_value: Some(raw_value.to_string()),
            normalized_setting_id: normalize_scalar_key(parsed_key),
            status: ParseStatus::Scalar,
            warning: normalize_scalar_key(parsed_key)
                .is_none()
                .then(|| "scalar key could not be normalized".to_string()),
        };
    }

    unsupported_record(path, line_number, raw_line, "unsupported config syntax")
}

fn parse_structured_line(
    path: &Path,
    line_number: usize,
    raw_line: &str,
    line: &str,
) -> Option<ParsedConfigLine> {
    let (key, value) = line.split_once('=').unwrap_or((line, ""));
    let key = key.trim();
    let block_key = key.strip_suffix('{').map(str::trim);
    let value = value.trim();
    let family = match key {
        "monitor" => "hl.monitor",
        "bind" => "hl.bind",
        "windowrule" | "windowrulev2" => "hl.windowrule",
        "animation" => "hl.animation",
        "bezier" => "hl.curve",
        "gesture" => "hl.gesture",
        "permission" => "hl.permission",
        _ => match block_key {
            Some("device") => "hl.device",
            _ => return None,
        },
    };

    Some(ParsedConfigLine {
        path: path.to_path_buf(),
        line_number,
        raw_line: raw_line.to_string(),
        parsed_key: Some(block_key.unwrap_or(key).to_string()),
        raw_value: (!value.is_empty()).then(|| value.to_string()),
        normalized_setting_id: Some(family.to_string()),
        status: ParseStatus::StructuredRaw,
        warning: Some(format!("{family} preserved as raw structured metadata")),
    })
}

fn normalize_scalar_key(key: &str) -> Option<String> {
    let key = key.trim();
    let (section, setting) = key.split_once(':')?;
    let section = section.trim();
    let setting = setting.trim();
    if section.is_empty() || setting.is_empty() || setting.contains(':') {
        return None;
    }
    Some(format!("{section}.{setting}"))
}

fn duplicate_scalar_keys(records: &[ParsedConfigLine]) -> BTreeMap<String, Vec<usize>> {
    let mut occurrences: BTreeMap<String, Vec<usize>> = BTreeMap::new();
    for record in records {
        if record.status == ParseStatus::Scalar {
            if let Some(setting_id) = &record.normalized_setting_id {
                occurrences
                    .entry(setting_id.clone())
                    .or_default()
                    .push(record.line_number);
            }
        }
    }
    occurrences.retain(|_, lines| lines.len() > 1);
    occurrences
}

fn base_record(
    path: &Path,
    line_number: usize,
    raw_line: &str,
    status: ParseStatus,
) -> ParsedConfigLine {
    ParsedConfigLine {
        path: path.to_path_buf(),
        line_number,
        raw_line: raw_line.to_string(),
        parsed_key: None,
        raw_value: None,
        normalized_setting_id: None,
        status,
        warning: None,
    }
}

fn structured_record(
    path: &Path,
    line_number: usize,
    raw_line: &str,
    normalized_setting_id: Option<&str>,
    warning: &str,
) -> ParsedConfigLine {
    ParsedConfigLine {
        path: path.to_path_buf(),
        line_number,
        raw_line: raw_line.to_string(),
        parsed_key: None,
        raw_value: None,
        normalized_setting_id: normalized_setting_id.map(str::to_string),
        status: ParseStatus::StructuredRaw,
        warning: Some(warning.to_string()),
    }
}

fn unsupported_record(
    path: &Path,
    line_number: usize,
    raw_line: &str,
    warning: &str,
) -> ParsedConfigLine {
    ParsedConfigLine {
        path: path.to_path_buf(),
        line_number,
        raw_line: raw_line.to_string(),
        parsed_key: None,
        raw_value: None,
        normalized_setting_id: None,
        status: ParseStatus::Unsupported,
        warning: Some(warning.to_string()),
    }
}

fn strip_inline_comment(line: &str) -> &str {
    line.split_once('#')
        .map(|(before_comment, _)| before_comment)
        .unwrap_or(line)
}
