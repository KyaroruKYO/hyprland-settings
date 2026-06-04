use anyhow::{anyhow, Result};

use crate::value::color::ColorValue;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct GradientValue {
    raw: String,
}

impl GradientValue {
    pub fn parse(raw: &str) -> Result<Self> {
        let trimmed = raw.trim();
        if trimmed.is_empty() {
            return Err(anyhow!("gradient value cannot be empty"));
        }
        if trimmed.contains('\n') || trimmed.contains('\r') {
            return Err(anyhow!("gradient value cannot span multiple lines"));
        }
        if trimmed.contains('#') {
            return Err(anyhow!(
                "gradient value cannot contain config comment marker"
            ));
        }
        if trimmed.contains("`") || trimmed.contains("$(") {
            return Err(anyhow!(
                "gradient value cannot contain command-substitution syntax"
            ));
        }

        let tokens = trimmed.split_whitespace().collect::<Vec<_>>();
        let mut color_count = 0usize;
        let mut angle_seen = false;
        for (index, token) in tokens.iter().enumerate() {
            if let Some(angle) = token.strip_suffix("deg") {
                if angle_seen {
                    return Err(anyhow!("gradient value cannot contain multiple angles"));
                }
                if index + 1 != tokens.len() {
                    return Err(anyhow!("gradient angle must be the final token"));
                }
                validate_angle(angle)?;
                angle_seen = true;
            } else {
                ColorValue::parse(token)?;
                color_count += 1;
            }
        }
        if color_count == 0 {
            return Err(anyhow!("gradient value needs at least one color"));
        }

        Ok(Self {
            raw: trimmed.to_string(),
        })
    }

    pub fn serialize(&self) -> &str {
        &self.raw
    }
}

fn validate_angle(value: &str) -> Result<()> {
    if value.is_empty() {
        return Err(anyhow!("gradient angle cannot be empty"));
    }
    match value.parse::<f64>() {
        Ok(angle) if angle.is_finite() => Ok(()),
        Ok(_) => Err(anyhow!("gradient angle must be finite")),
        Err(_) => Err(anyhow!("gradient angle must be numeric")),
    }
}
