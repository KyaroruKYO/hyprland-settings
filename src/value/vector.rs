use anyhow::{anyhow, Result};

#[derive(Debug, Clone, PartialEq)]
pub struct Vec2Value {
    pub x: f64,
    pub y: f64,
    separator: Vec2Separator,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum Vec2Separator {
    Space,
    Comma,
}

impl Vec2Value {
    pub fn parse(raw: &str) -> Result<Self> {
        let trimmed = raw.trim();
        if trimmed.is_empty() {
            return Err(anyhow!("vector value cannot be empty"));
        }
        if trimmed.contains('\n') || trimmed.contains('\r') {
            return Err(anyhow!("vector value cannot span multiple lines"));
        }
        if trimmed.matches(',').count() == 1 {
            let parts = trimmed.split(',').map(str::trim).collect::<Vec<_>>();
            return parse_parts(&parts, Vec2Separator::Comma);
        }
        if trimmed.contains(',') {
            return Err(anyhow!("vector value must contain exactly one comma"));
        }
        let parts = trimmed.split_whitespace().collect::<Vec<_>>();
        parse_parts(&parts, Vec2Separator::Space)
    }

    pub fn serialize(&self) -> String {
        match self.separator {
            Vec2Separator::Space => format!("{} {}", format_number(self.x), format_number(self.y)),
            Vec2Separator::Comma => format!("{},{}", format_number(self.x), format_number(self.y)),
        }
    }
}

fn parse_parts(parts: &[&str], separator: Vec2Separator) -> Result<Vec2Value> {
    if parts.len() != 2 {
        return Err(anyhow!("vector value must contain exactly two numbers"));
    }
    let x = parse_finite(parts[0])?;
    let y = parse_finite(parts[1])?;
    Ok(Vec2Value { x, y, separator })
}

fn parse_finite(raw: &str) -> Result<f64> {
    if raw.is_empty() {
        return Err(anyhow!("vector component cannot be empty"));
    }
    let value = raw
        .parse::<f64>()
        .map_err(|_| anyhow!("vector component must be numeric"))?;
    if value.is_finite() {
        Ok(value)
    } else {
        Err(anyhow!("vector component must be finite"))
    }
}

fn format_number(value: f64) -> String {
    if value.fract() == 0.0 {
        format!("{value:.0}")
    } else {
        value.to_string()
    }
}
