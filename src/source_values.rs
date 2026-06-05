use std::fs;
use std::path::{Path, PathBuf};

use anyhow::{Context, Result};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum XkbRulesSection {
    Model,
    Layout,
    Variant,
    Option,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct XkbSourceValue {
    pub raw_value: String,
    pub label: String,
    pub parent_layout: Option<String>,
}

#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct XkbRulesValues {
    pub source_path: Option<PathBuf>,
    pub models: Vec<XkbSourceValue>,
    pub layouts: Vec<XkbSourceValue>,
    pub variants: Vec<XkbSourceValue>,
    pub options: Vec<XkbSourceValue>,
}

impl XkbRulesValues {
    pub fn has_model(&self, value: &str) -> bool {
        self.models.iter().any(|item| item.raw_value == value)
    }

    pub fn has_layout(&self, value: &str) -> bool {
        self.layouts.iter().any(|item| item.raw_value == value)
    }

    pub fn has_variant(&self, value: &str) -> bool {
        self.variants.iter().any(|item| item.raw_value == value)
    }

    pub fn has_option(&self, value: &str) -> bool {
        self.options.iter().any(|item| item.raw_value == value)
    }
}

pub fn read_system_xkb_rules() -> Result<XkbRulesValues> {
    for path in system_xkb_rules_candidates() {
        if path.is_file() {
            let content = fs::read_to_string(&path)
                .with_context(|| format!("failed to read XKB rules file {}", path.display()))?;
            let mut values = parse_xkb_rules_lst(&content);
            values.source_path = Some(path);
            return Ok(values);
        }
    }

    anyhow::bail!("no supported XKB rules .lst file found")
}

pub fn system_xkb_rules_candidates() -> Vec<PathBuf> {
    vec![
        Path::new("/usr/share/X11/xkb/rules/evdev.lst").to_path_buf(),
        Path::new("/usr/share/X11/xkb/rules/base.lst").to_path_buf(),
    ]
}

pub fn parse_xkb_rules_lst(content: &str) -> XkbRulesValues {
    let mut values = XkbRulesValues::default();
    let mut section = None;

    for line in content.lines() {
        let trimmed = line.trim();
        if trimmed.is_empty() {
            continue;
        }

        if let Some(header) = trimmed.strip_prefix('!') {
            section = match header.trim() {
                "model" => Some(XkbRulesSection::Model),
                "layout" => Some(XkbRulesSection::Layout),
                "variant" => Some(XkbRulesSection::Variant),
                "option" => Some(XkbRulesSection::Option),
                _ => None,
            };
            continue;
        }

        let Some(section) = section else {
            continue;
        };
        let Some((raw_value, description)) = split_xkb_item(trimmed) else {
            continue;
        };

        let value = match section {
            XkbRulesSection::Variant => {
                let (parent_layout, label) = parse_variant_description(description);
                XkbSourceValue {
                    raw_value: raw_value.to_string(),
                    label,
                    parent_layout,
                }
            }
            _ => XkbSourceValue {
                raw_value: raw_value.to_string(),
                label: description.trim().to_string(),
                parent_layout: None,
            },
        };

        match section {
            XkbRulesSection::Model => values.models.push(value),
            XkbRulesSection::Layout => values.layouts.push(value),
            XkbRulesSection::Variant => values.variants.push(value),
            XkbRulesSection::Option => values.options.push(value),
        }
    }

    values
}

fn split_xkb_item(line: &str) -> Option<(&str, &str)> {
    let mut split_at = None;
    let mut previous_was_whitespace = false;

    for (index, ch) in line.char_indices() {
        let is_whitespace = ch.is_whitespace();
        if previous_was_whitespace && !is_whitespace {
            split_at = Some(index);
            break;
        }
        previous_was_whitespace = is_whitespace;
    }

    let split_at = split_at?;
    let raw_value = line[..split_at].trim();
    let description = line[split_at..].trim();
    if raw_value.is_empty() || description.is_empty() {
        return None;
    }
    Some((raw_value, description))
}

fn parse_variant_description(description: &str) -> (Option<String>, String) {
    if let Some((layout, label)) = description.split_once(':') {
        let layout = layout.trim();
        let label = label.trim();
        if !layout.is_empty() && !label.is_empty() {
            return (Some(layout.to_string()), label.to_string());
        }
    }

    (None, description.trim().to_string())
}
