use std::fs;
use std::path::PathBuf;

use crate::config_parser::{parse_hyprland_config_file, ParseStatus};
use crate::write_target_candidate::WriteTargetCandidate;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct FixtureTargetWriteProofRequest {
    pub target: WriteTargetCandidate,
    pub setting_id: String,
    pub new_value: String,
    pub advanced_fixture_approval: bool,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct FixtureTargetWriteProof {
    pub backup_path: PathBuf,
    pub target_path: PathBuf,
    pub reread_value: Option<String>,
    pub unrelated_lines_preserved: bool,
    pub fixture_only: bool,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum FixtureTargetWriteProofError {
    UnsafeTargetRequiresAdvancedApproval,
    MissingLineNumber,
    ReadFailed(String),
    WriteFailed(String),
    RereadFailed(String),
    VerificationFailed,
}

pub fn prove_fixture_target_write(
    request: &FixtureTargetWriteProofRequest,
) -> Result<FixtureTargetWriteProof, FixtureTargetWriteProofError> {
    if request.target.generated_or_script_managed && !request.advanced_fixture_approval {
        return Err(FixtureTargetWriteProofError::UnsafeTargetRequiresAdvancedApproval);
    }

    let line_number = request
        .target
        .line_number
        .ok_or(FixtureTargetWriteProofError::MissingLineNumber)?;
    let target_path = request.target.file_path.clone();
    let original = fs::read_to_string(&target_path)
        .map_err(|error| FixtureTargetWriteProofError::ReadFailed(error.to_string()))?;
    let backup_path = target_path.with_extension("fixture-proof.bak");
    fs::write(&backup_path, &original)
        .map_err(|error| FixtureTargetWriteProofError::WriteFailed(error.to_string()))?;

    let mut lines = original.lines().map(str::to_string).collect::<Vec<_>>();
    let index = line_number
        .checked_sub(1)
        .filter(|index| *index < lines.len())
        .ok_or(FixtureTargetWriteProofError::MissingLineNumber)?;
    let before_lines = lines.clone();
    let config_key = request.setting_id.replace('.', ":");
    lines[index] = format!("{config_key} = {}", request.new_value);
    let had_trailing_newline = original.ends_with('\n');
    let mut updated = lines.join("\n");
    if had_trailing_newline {
        updated.push('\n');
    }
    fs::write(&target_path, updated)
        .map_err(|error| FixtureTargetWriteProofError::WriteFailed(error.to_string()))?;

    let reread = parse_hyprland_config_file(&target_path)
        .map_err(|error| FixtureTargetWriteProofError::RereadFailed(error.to_string()))?;
    let reread_value = reread
        .scalar_records()
        .filter(|record| {
            record.status == ParseStatus::Scalar
                && record.normalized_setting_id.as_deref() == Some(&request.setting_id)
        })
        .filter_map(|record| record.raw_value.clone())
        .last();
    if reread_value.as_deref() != Some(request.new_value.as_str()) {
        return Err(FixtureTargetWriteProofError::VerificationFailed);
    }

    let unrelated_lines_preserved = before_lines
        .iter()
        .enumerate()
        .all(|(line_index, before)| line_index == index || lines.get(line_index) == Some(before));

    Ok(FixtureTargetWriteProof {
        backup_path,
        target_path,
        reread_value,
        unrelated_lines_preserved,
        fixture_only: true,
    })
}
