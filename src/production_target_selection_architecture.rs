use crate::guarded_write_review::PRODUCTION_WRITE_TARGET_REVIEW_ENABLED;
use crate::production_advanced_confirmation::PRODUCTION_ADVANCED_CONFIRMATION_ENABLED;
use crate::write_enablement_readiness::PRODUCTION_WRITE_TARGET_SELECTION_READY;
use crate::write_review_walkthrough::PRODUCTION_WRITE_REVIEW_WALKTHROUGH_CAN_WRITE;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ProductionTargetSelectionArchitecture {
    pub entry_point: String,
    pub required_input_models: Vec<String>,
    pub target_candidate_source: String,
    pub target_review_state: String,
    pub dependencies: Vec<ProductionTargetSelectionDependency>,
    pub apply_integration_boundary: ApplyIntegrationBoundary,
    pub production_gate_boundary: String,
    pub already_implemented_read_only_or_fixture_only: Vec<String>,
    pub needed_before_first_production_enablement: Vec<String>,
    pub must_remain_disabled_this_sprint: Vec<String>,
    pub production_enabled: bool,
}

impl ProductionTargetSelectionArchitecture {
    pub fn user_facing_lines(&self) -> Vec<String> {
        vec![
            "Minimum production target-selection path is not enabled yet.".to_string(),
            "The current app can preview the review flow but cannot write through it.".to_string(),
            "One fixture-proven target path may be used later as the first pilot.".to_string(),
            format!("Entry point: {}", self.entry_point),
            format!("Production gate: {}", self.production_gate_boundary),
        ]
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ProductionTargetSelectionDependency {
    pub name: String,
    pub already_available_as_read_only_or_fixture_only: bool,
    pub needed_before_enablement: bool,
    pub must_remain_disabled_now: bool,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ApplyIntegrationBoundary {
    pub production_apply_may_call_target_selection_after_all_gates: bool,
    pub production_apply_must_not_call_fixture_proof: bool,
    pub production_apply_must_not_call_walkthrough_directly: bool,
    pub production_apply_must_not_skip_backup: bool,
    pub production_apply_must_not_skip_reread_verification: bool,
    pub production_apply_must_not_bypass_high_risk_policy: bool,
}

impl ApplyIntegrationBoundary {
    pub fn report_lines(&self) -> Vec<String> {
        vec![
            "Production Apply may only call target selection after all gates pass.".to_string(),
            "Production Apply must not call fixture proof.".to_string(),
            "Production Apply must not call the walkthrough directly.".to_string(),
            "Production Apply must not skip backup.".to_string(),
            "Production Apply must not skip reread verification.".to_string(),
            "Production Apply must not bypass high-risk policy.".to_string(),
        ]
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ProductionWriteGateInventoryItem {
    pub gate_name: &'static str,
    pub current_value: bool,
    pub module: &'static str,
    pub would_allow: &'static str,
    pub required_proof_before_flip: &'static str,
    pub must_remain_false_now: bool,
}

pub fn minimum_production_target_selection_architecture() -> ProductionTargetSelectionArchitecture {
    ProductionTargetSelectionArchitecture {
        entry_point:
            "Disabled setting-detail write review for a layered scalar setting".to_string(),
        required_input_models: vec![
            "LayeredSettingValues".to_string(),
            "WriteTargetCandidate".to_string(),
            "WriteTargetRecommendation".to_string(),
            "GuardedWriteTargetReview".to_string(),
            "WriteBackupPlan".to_string(),
            "WriteAdvancedConfirmation".to_string(),
            "WriteVerificationPlan".to_string(),
        ],
        target_candidate_source:
            "One safe candidate generated from one existing scalar occurrence".to_string(),
        target_review_state:
            "Disabled review state until every production gate is explicitly true".to_string(),
        dependencies: vec![
            dependency("target candidate generation", true, false, true),
            dependency("exact backup plan", true, true, true),
            dependency("advanced confirmation design", true, true, true),
            dependency("reread verification plan", true, true, true),
            dependency("rollback/recovery implementation", false, true, true),
            dependency("Apply integration boundary", true, true, true),
            dependency("production gate approval", true, true, true),
        ],
        apply_integration_boundary: production_apply_integration_boundary(),
        production_gate_boundary:
            "All production target-selection and pilot gates must be true before Apply can use the path."
                .to_string(),
        already_implemented_read_only_or_fixture_only: vec![
            "read-only layered occurrence detection".to_string(),
            "disabled target recommendation".to_string(),
            "fixture-only backup/write/reread proof".to_string(),
            "guarded review model".to_string(),
            "non-writing walkthrough".to_string(),
            "production readiness model".to_string(),
        ],
        needed_before_first_production_enablement: vec![
            "production backup implementation".to_string(),
            "backup path policy finalization".to_string(),
            "production reread verification".to_string(),
            "rollback/recovery implementation".to_string(),
            "advanced confirmation implementation".to_string(),
            "high-risk policy integration".to_string(),
            "manual smoke review completion".to_string(),
            "explicit production Apply integration approval".to_string(),
        ],
        must_remain_disabled_this_sprint: vec![
            "production target selection".to_string(),
            "real layered writes".to_string(),
            "session-selected config as write target".to_string(),
            "production Apply integration".to_string(),
        ],
        production_enabled: false,
    }
}

pub fn production_apply_integration_boundary() -> ApplyIntegrationBoundary {
    ApplyIntegrationBoundary {
        production_apply_may_call_target_selection_after_all_gates: false,
        production_apply_must_not_call_fixture_proof: true,
        production_apply_must_not_call_walkthrough_directly: true,
        production_apply_must_not_skip_backup: true,
        production_apply_must_not_skip_reread_verification: true,
        production_apply_must_not_bypass_high_risk_policy: true,
    }
}

pub fn production_write_gate_inventory() -> Vec<ProductionWriteGateInventoryItem> {
    vec![
        ProductionWriteGateInventoryItem {
            gate_name: "PRODUCTION_WRITE_TARGET_SELECTION_READY",
            current_value: PRODUCTION_WRITE_TARGET_SELECTION_READY,
            module: "write_enablement_readiness",
            would_allow: "the target-selection UI readiness model to move toward production",
            required_proof_before_flip:
                "exact backup, reread verification, recovery, advanced confirmation, and Apply boundary proof",
            must_remain_false_now: true,
        },
        ProductionWriteGateInventoryItem {
            gate_name: "PRODUCTION_WRITE_TARGET_REVIEW_ENABLED",
            current_value: PRODUCTION_WRITE_TARGET_REVIEW_ENABLED,
            module: "guarded_write_review",
            would_allow: "guarded target review to leave production-disabled status",
            required_proof_before_flip:
                "all guarded review gates and high-risk policy integration proven",
            must_remain_false_now: true,
        },
        ProductionWriteGateInventoryItem {
            gate_name: "PRODUCTION_WRITE_REVIEW_WALKTHROUGH_CAN_WRITE",
            current_value: PRODUCTION_WRITE_REVIEW_WALKTHROUGH_CAN_WRITE,
            module: "write_review_walkthrough",
            would_allow: "walkthrough-derived state to participate in writes",
            required_proof_before_flip:
                "walkthrough state converted into a production-safe review contract",
            must_remain_false_now: true,
        },
        ProductionWriteGateInventoryItem {
            gate_name: "PRODUCTION_ADVANCED_CONFIRMATION_ENABLED",
            current_value: PRODUCTION_ADVANCED_CONFIRMATION_ENABLED,
            module: "production_advanced_confirmation",
            would_allow: "advanced confirmation policy to become available for risky target classes",
            required_proof_before_flip:
                "generated, script-managed, symlink-managed, hard-block, recovery, and high-risk policies proven",
            must_remain_false_now: true,
        },
        ProductionWriteGateInventoryItem {
            gate_name: "PRODUCTION_ONE_TARGET_WRITE_PILOT_ENABLED",
            current_value: crate::one_target_write_pilot::PRODUCTION_ONE_TARGET_WRITE_PILOT_ENABLED,
            module: "one_target_write_pilot",
            would_allow: "the one-target pilot path to become eligible for production integration",
            required_proof_before_flip:
                "one normal scalar target path proven with production backup, verification, recovery, and manual approval",
            must_remain_false_now: true,
        },
    ]
}

fn dependency(
    name: &str,
    already_available_as_read_only_or_fixture_only: bool,
    needed_before_enablement: bool,
    must_remain_disabled_now: bool,
) -> ProductionTargetSelectionDependency {
    ProductionTargetSelectionDependency {
        name: name.to_string(),
        already_available_as_read_only_or_fixture_only,
        needed_before_enablement,
        must_remain_disabled_now,
    }
}
