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
    if let Some(hex) = value
        .strip_prefix("rgb(")
        .and_then(|value| value.strip_suffix(')'))
    {
        return validate_hex_digits(hex, 6, "rgb color");
    }
    if let Some(hex) = value
        .strip_prefix("rgba(")
        .and_then(|value| value.strip_suffix(')'))
    {
        return validate_hex_digits(hex, 8, "rgba color");
    }
    if let Some(hex) = value.strip_prefix("0x") {
        return validate_hex_digits(hex, 8, "0xAARRGGBB color");
    }
    Err(anyhow!(
        "color value must be rgb(RRGGBB), rgba(RRGGBBAA), or 0xAARRGGBB"
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
