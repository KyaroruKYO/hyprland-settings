use anyhow::{anyhow, Result};

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RegexValue {
    raw: String,
}

impl RegexValue {
    pub fn parse(raw: &str) -> Result<Self> {
        let trimmed = raw.trim();
        if trimmed.is_empty() {
            return Err(anyhow!("regex value cannot be empty"));
        }
        if trimmed.contains('\n') || trimmed.contains('\r') {
            return Err(anyhow!("regex value cannot span multiple lines"));
        }
        if trimmed.contains('#') {
            return Err(anyhow!("regex value cannot contain config comment marker"));
        }
        if trimmed.contains("`") || trimmed.contains("$(") {
            return Err(anyhow!(
                "regex value cannot contain command-substitution syntax"
            ));
        }
        Ok(Self {
            raw: trimmed.to_string(),
        })
    }

    pub fn serialize(&self) -> &str {
        &self.raw
    }
}
