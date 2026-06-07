use anyhow::{anyhow, Result};

#[derive(Debug, Clone, PartialEq)]
pub struct NumericListValue {
    raw: String,
    numbers: Vec<f64>,
}

impl NumericListValue {
    pub fn parse(raw: &str) -> Result<Self> {
        let trimmed = raw.trim();
        if trimmed.is_empty() {
            return Err(anyhow!("numeric list cannot be empty"));
        }
        if trimmed.contains('\n') || trimmed.contains('\r') {
            return Err(anyhow!("numeric list cannot span multiple lines"));
        }
        if trimmed.contains(',') {
            return Err(anyhow!("numeric list must use spaces, not commas"));
        }
        if trimmed.contains('#') {
            return Err(anyhow!("numeric list cannot contain config comment marker"));
        }
        if trimmed.contains("`") || trimmed.contains("$(") {
            return Err(anyhow!(
                "numeric list cannot contain command-substitution syntax"
            ));
        }

        let mut numbers = Vec::new();
        for token in trimmed.split_whitespace() {
            let number = token
                .parse::<f64>()
                .map_err(|_| anyhow!("numeric list token must be numeric: {token}"))?;
            if !number.is_finite() {
                return Err(anyhow!("numeric list values must be finite"));
            }
            numbers.push(number);
        }
        if numbers.len() < 2 {
            return Err(anyhow!("scroll points need a step and at least one point"));
        }
        if numbers[0] <= 0.0 {
            return Err(anyhow!("scroll points step must be positive"));
        }

        Ok(Self {
            raw: trimmed.to_string(),
            numbers,
        })
    }

    pub fn serialize(&self) -> &str {
        &self.raw
    }

    pub fn numbers(&self) -> &[f64] {
        &self.numbers
    }
}
