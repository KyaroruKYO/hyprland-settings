use anyhow::{anyhow, Result};

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ColorValue {
    raw: String,
}

impl ColorValue {
    pub fn parse(raw: &str) -> Result<Self> {
        let trimmed = raw.trim();
        if trimmed.is_empty() {
            return Err(anyhow!("color value cannot be empty"));
        }
        if trimmed.contains(char::is_whitespace) {
            return Err(anyhow!("color value cannot contain whitespace"));
        }
        validate_color_literal(trimmed)?;
        Ok(Self {
            raw: trimmed.to_string(),
        })
    }

    pub fn serialize(&self) -> &str {
        &self.raw
    }
}

pub fn validate_color_literal(value: &str) -> Result<()> {
    if let Some(hex) = value.strip_prefix('#') {
        return match hex.len() {
            3 => validate_hex_digits(hex, 3, "#RGB color"),
            6 => validate_hex_digits(hex, 6, "#RRGGBB color"),
            8 => validate_hex_digits(hex, 8, "#RRGGBBAA color"),
            _ => Err(anyhow!("# color value must contain 3, 6, or 8 hex digits")),
        };
    }
    if let Some(hex) = value
        .strip_prefix("rgb(")
        .and_then(|value| value.strip_suffix(')'))
    {
        return validate_rgb_body(hex);
    }
    if let Some(hex) = value
        .strip_prefix("rgba(")
        .and_then(|value| value.strip_suffix(')'))
    {
        return validate_rgba_body(hex);
    }
    if let Some(hex) = value.strip_prefix("0x") {
        if hex.is_empty() || hex.len() > 8 {
            return Err(anyhow!("0x color value must contain 1 to 8 hex digits"));
        }
        return validate_hex_digits(hex, hex.len(), "0x color");
    }
    if !value.is_empty() && value.chars().all(|char| char.is_ascii_digit()) {
        return match value.parse::<u32>() {
            Ok(_) => Ok(()),
            Err(_) => Err(anyhow!("decimal color integer must fit in uint32")),
        };
    }
    Err(anyhow!(
        "color value must be an official Hyprland color literal"
    ))
}

fn validate_hex_digits(value: &str, expected_len: usize, label: &str) -> Result<()> {
    if value.len() != expected_len {
        return Err(anyhow!("{label} must contain {expected_len} hex digits"));
    }
    if value.chars().all(|char| char.is_ascii_hexdigit()) {
        Ok(())
    } else {
        Err(anyhow!("{label} contains non-hex characters"))
    }
}

fn validate_rgb_body(value: &str) -> Result<()> {
    if value.contains(',') {
        let parts = comma_parts(value, 3, "rgb color")?;
        for part in parts {
            validate_u8(part, "rgb color component")?;
        }
        Ok(())
    } else {
        validate_hex_digits(value, 6, "rgb color")
    }
}

fn validate_rgba_body(value: &str) -> Result<()> {
    if value.contains(',') {
        let parts = comma_parts(value, 4, "rgba color")?;
        for part in &parts[..3] {
            validate_u8(part, "rgba color component")?;
        }
        validate_alpha(parts[3])
    } else {
        validate_hex_digits(value, 8, "rgba color")
    }
}

fn comma_parts<'a>(value: &'a str, expected_len: usize, label: &str) -> Result<Vec<&'a str>> {
    let parts = value.split(',').map(str::trim).collect::<Vec<_>>();
    if parts.len() != expected_len || parts.iter().any(|part| part.is_empty()) {
        return Err(anyhow!("{label} has the wrong number of comma values"));
    }
    Ok(parts)
}

fn validate_u8(value: &str, label: &str) -> Result<()> {
    match value.parse::<u8>() {
        Ok(_) => Ok(()),
        Err(_) => Err(anyhow!("{label} must be an integer from 0 to 255")),
    }
}

fn validate_alpha(value: &str) -> Result<()> {
    match value.parse::<f32>() {
        Ok(alpha) if alpha.is_finite() && (0.0..=1.0).contains(&alpha) => Ok(()),
        Ok(_) => Err(anyhow!("rgba alpha must be between 0 and 1")),
        Err(_) => Err(anyhow!("rgba alpha must be numeric")),
    }
}
