use anyhow::{anyhow, Result};

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PathValue {
    raw: String,
}

impl PathValue {
    pub fn parse(raw: &str) -> Result<Self> {
        let trimmed = raw.trim();
        if trimmed.is_empty() {
            return Err(anyhow!("path value cannot be empty"));
        }
        if trimmed.contains('\n') || trimmed.contains('\r') {
            return Err(anyhow!("path value cannot span multiple lines"));
        }
        if trimmed.contains('#') {
            return Err(anyhow!("path value cannot contain config comment marker"));
        }
        for forbidden in ["`", "$(", ";", "|", "&&", "||"] {
            if trimmed.contains(forbidden) {
                return Err(anyhow!(
                    "path value cannot contain command-like token {forbidden:?}"
                ));
            }
        }
        Ok(Self {
            raw: trimmed.to_string(),
        })
    }

    pub fn serialize(&self) -> &str {
        &self.raw
    }
}
