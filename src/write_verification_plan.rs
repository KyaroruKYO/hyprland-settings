use std::path::PathBuf;

use crate::write_target_candidate::WriteTargetCandidate;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct WriteVerificationPlan {
    pub target_file_path: PathBuf,
    pub setting_id: String,
    pub expected_value: String,
    pub reread_method: String,
    pub verification_status: WriteVerificationStatus,
    pub fixture_only: bool,
    pub production_disabled: bool,
}

impl WriteVerificationPlan {
    pub fn user_facing_lines(&self) -> Vec<String> {
        vec![
            "The app will reread the file to confirm the value.".to_string(),
            format!("Expected value: {}", self.expected_value),
            format!("Verification: {}", self.verification_status.label()),
        ]
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum WriteVerificationStatus {
    NotRun,
    Planned,
    PassedInFixture,
    FailedInFixture,
    ProductionDisabled,
}

impl WriteVerificationStatus {
    pub fn label(self) -> &'static str {
        match self {
            Self::NotRun => "not run",
            Self::Planned => "planned",
            Self::PassedInFixture => "passed in fixture",
            Self::FailedInFixture => "failed in fixture",
            Self::ProductionDisabled => "production disabled",
        }
    }
}

pub fn planned_reread_verification(
    candidate: &WriteTargetCandidate,
    setting_id: impl Into<String>,
    expected_value: impl Into<String>,
) -> WriteVerificationPlan {
    WriteVerificationPlan {
        target_file_path: candidate.file_path.clone(),
        setting_id: setting_id.into(),
        expected_value: expected_value.into(),
        reread_method: "parse fixture/config file and compare scalar value".to_string(),
        verification_status: WriteVerificationStatus::ProductionDisabled,
        fixture_only: true,
        production_disabled: true,
    }
}

pub fn fixture_verification_passed(plan: &WriteVerificationPlan) -> WriteVerificationPlan {
    WriteVerificationPlan {
        verification_status: WriteVerificationStatus::PassedInFixture,
        ..plan.clone()
    }
}
