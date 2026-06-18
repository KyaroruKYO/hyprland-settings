use std::path::{Path, PathBuf};

use crate::config_parser::{parse_hyprland_config_file, ParseStatus};
use crate::write_target_candidate::WriteTargetCandidate;

pub const PRODUCTION_VERIFICATION_CONTRACT_ENABLED: bool = true;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ProductionVerificationContract {
    pub target_file_path: PathBuf,
    pub setting_id: String,
    pub expected_value: String,
    pub expected_line_number: Option<usize>,
    pub reread_parser_method: String,
    pub normalization_policy: String,
    pub verification_statuses: Vec<ProductionVerificationStatus>,
    pub failure_reasons: Vec<String>,
    pub fixture_only_proof_status: ProductionVerificationStatus,
    pub production_enabled: bool,
}

impl ProductionVerificationContract {
    pub fn user_facing_lines(&self) -> Vec<String> {
        vec![
            "The app will reread the file to confirm the value.".to_string(),
            "If verification fails, the app must not report the change as complete.".to_string(),
            "Verification approval is staged; real verification is still not active yet."
                .to_string(),
        ]
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ProductionVerificationStatus {
    NotRun,
    Planned,
    PassedInFixture,
    FailedInFixture,
    ProductionDisabled,
    WouldRequireRollback,
}

impl ProductionVerificationStatus {
    pub fn label(self) -> &'static str {
        match self {
            Self::NotRun => "not run",
            Self::Planned => "planned",
            Self::PassedInFixture => "passed in fixture",
            Self::FailedInFixture => "failed in fixture",
            Self::ProductionDisabled => "production disabled",
            Self::WouldRequireRollback => "would require rollback",
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct FixtureRereadVerificationProof {
    pub target_file_path: PathBuf,
    pub setting_id: String,
    pub expected_value: String,
    pub observed_value: Option<String>,
    pub expected_line_number: Option<usize>,
    pub observed_line_number: Option<usize>,
    pub status: ProductionVerificationStatus,
    pub fixture_only: bool,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum FixtureRereadVerificationError {
    NonFixturePath,
    ReadFailed(String),
}

pub fn production_verification_contract_for_candidate(
    candidate: &WriteTargetCandidate,
    setting_id: impl Into<String>,
    expected_value: impl Into<String>,
) -> ProductionVerificationContract {
    ProductionVerificationContract {
        target_file_path: candidate.file_path.clone(),
        setting_id: setting_id.into(),
        expected_value: expected_value.into(),
        expected_line_number: candidate.line_number,
        reread_parser_method:
            "reread exact target file, parse scalar records, and compare normalized setting id"
                .to_string(),
        normalization_policy:
            "Use existing Hyprland scalar key normalization and compare the raw parsed value."
                .to_string(),
        verification_statuses: all_production_verification_statuses(),
        failure_reasons: vec![
            "target file could not be reread".to_string(),
            "expected setting id was not found".to_string(),
            "expected line no longer matches the setting".to_string(),
            "observed value did not match expected value".to_string(),
        ],
        fixture_only_proof_status: ProductionVerificationStatus::NotRun,
        production_enabled: PRODUCTION_VERIFICATION_CONTRACT_ENABLED,
    }
}

pub fn all_production_verification_statuses() -> Vec<ProductionVerificationStatus> {
    vec![
        ProductionVerificationStatus::NotRun,
        ProductionVerificationStatus::Planned,
        ProductionVerificationStatus::PassedInFixture,
        ProductionVerificationStatus::FailedInFixture,
        ProductionVerificationStatus::ProductionDisabled,
        ProductionVerificationStatus::WouldRequireRollback,
    ]
}

pub fn fixture_reread_verify_expected_value(
    contract: &ProductionVerificationContract,
) -> Result<FixtureRereadVerificationProof, FixtureRereadVerificationError> {
    if !contract.target_file_path.starts_with(std::env::temp_dir()) {
        return Err(FixtureRereadVerificationError::NonFixturePath);
    }

    let parsed = parse_hyprland_config_file(&contract.target_file_path)
        .map_err(|error| FixtureRereadVerificationError::ReadFailed(error.to_string()))?;
    let record = parsed
        .scalar_records()
        .filter(|record| {
            record.status == ParseStatus::Scalar
                && record.normalized_setting_id.as_deref() == Some(contract.setting_id.as_str())
        })
        .find(|record| {
            contract
                .expected_line_number
                .map(|line| record.line_number == line)
                .unwrap_or(true)
        })
        .or_else(|| {
            parsed
                .scalar_records()
                .filter(|record| {
                    record.status == ParseStatus::Scalar
                        && record.normalized_setting_id.as_deref()
                            == Some(contract.setting_id.as_str())
                })
                .last()
        });

    let observed_value = record.and_then(|record| record.raw_value.clone());
    let observed_line_number = record.map(|record| record.line_number);
    let status = if observed_value.as_deref() == Some(contract.expected_value.as_str()) {
        ProductionVerificationStatus::PassedInFixture
    } else {
        ProductionVerificationStatus::FailedInFixture
    };

    Ok(FixtureRereadVerificationProof {
        target_file_path: contract.target_file_path.clone(),
        setting_id: contract.setting_id.clone(),
        expected_value: contract.expected_value.clone(),
        observed_value,
        expected_line_number: contract.expected_line_number,
        observed_line_number,
        status,
        fixture_only: true,
    })
}

#[allow(dead_code)]
fn _path_is_fixture(path: &Path) -> bool {
    path.starts_with(std::env::temp_dir())
}
