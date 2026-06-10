use crate::write_target_candidate::WriteTargetCandidate;

pub const PRODUCTION_ONE_TARGET_WRITE_PILOT_ENABLED: bool = false;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct OneTargetWritePilot {
    pub pilot_name: String,
    pub row_id: String,
    pub official_setting_id: String,
    pub target_category: String,
    pub target_constraints: Vec<String>,
    pub required_proof: Vec<String>,
    pub blocked_conditions: Vec<String>,
    pub production_enabled: bool,
    pub candidate_eligible: bool,
    pub why_minimum_path: String,
}

impl OneTargetWritePilot {
    pub fn user_facing_lines(&self) -> Vec<String> {
        vec![
            "First production write pilot".to_string(),
            "Status: Not enabled".to_string(),
            "Candidate: One existing scalar line in one normal config file".to_string(),
            "Before enabling this pilot, exact backup, reread verification, recovery, and advanced confirmation policy must be complete.".to_string(),
            "Real writing is not active yet.".to_string(),
            "Apply behavior has not changed.".to_string(),
        ]
    }
}

pub fn minimum_one_target_write_pilot_design() -> OneTargetWritePilot {
    OneTargetWritePilot {
        pilot_name: "One normal scalar target pilot".to_string(),
        row_id: "fixture.normal_scalar".to_string(),
        official_setting_id: "general.layout".to_string(),
        target_category: "one existing scalar line in one normal config file".to_string(),
        target_constraints: minimum_target_constraints(),
        required_proof: minimum_required_proof(),
        blocked_conditions: blocked_conditions(),
        production_enabled: PRODUCTION_ONE_TARGET_WRITE_PILOT_ENABLED,
        candidate_eligible: false,
        why_minimum_path:
            "It avoids generated, script-managed, symlink-managed, missing-line, structured, and high-risk targets."
                .to_string(),
    }
}

pub fn one_target_write_pilot_for_candidate(
    row_id: impl Into<String>,
    official_setting_id: impl Into<String>,
    candidate: &WriteTargetCandidate,
    high_risk_policy_present: bool,
    fixture_proof_passed: bool,
) -> OneTargetWritePilot {
    let row_id = row_id.into();
    let official_setting_id = official_setting_id.into();
    let candidate_eligible = candidate.safe
        && candidate.line_number.is_some()
        && !candidate.generated_or_script_managed
        && !candidate.symlink_managed
        && !candidate.requires_advanced_confirmation
        && !high_risk_policy_present
        && fixture_proof_passed;

    OneTargetWritePilot {
        pilot_name: "One normal scalar target pilot".to_string(),
        row_id,
        official_setting_id,
        target_category: "one existing scalar line in one normal config file".to_string(),
        target_constraints: minimum_target_constraints(),
        required_proof: minimum_required_proof(),
        blocked_conditions: blocked_conditions(),
        production_enabled: PRODUCTION_ONE_TARGET_WRITE_PILOT_ENABLED,
        candidate_eligible,
        why_minimum_path:
            "It uses one known scalar line, one normal file, exact backup, and reread verification."
                .to_string(),
    }
}

fn minimum_target_constraints() -> Vec<String> {
    vec![
        "one scalar setting".to_string(),
        "one existing scalar line".to_string(),
        "one target file".to_string(),
        "non-generated file".to_string(),
        "non-script-managed file".to_string(),
        "non-symlink-managed file".to_string(),
        "exact line number known".to_string(),
        "backup required".to_string(),
        "reread verification required".to_string(),
        "high-risk policy clear".to_string(),
        "fixture proof passed".to_string(),
        "production gate still false".to_string(),
    ]
}

fn minimum_required_proof() -> Vec<String> {
    vec![
        "fixture scalar file created".to_string(),
        "candidate generated for one existing scalar line".to_string(),
        "candidate is not generated, script-managed, or symlink-managed".to_string(),
        "backup plan exists".to_string(),
        "advanced confirmation is not required".to_string(),
        "verification plan exists".to_string(),
        "fixture write proof passes".to_string(),
        "unrelated lines are preserved".to_string(),
        "guarded review remains production disabled".to_string(),
        "no real user config files touched".to_string(),
    ]
}

fn blocked_conditions() -> Vec<String> {
    vec![
        "generated file".to_string(),
        "script-managed file".to_string(),
        "symlink-managed file".to_string(),
        "missing line number".to_string(),
        "structured block target".to_string(),
        "high-risk row".to_string(),
        "duplicate target requiring user choice".to_string(),
        "production gate false".to_string(),
    ]
}
