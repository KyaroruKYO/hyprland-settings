use crate::guarded_write_review::PRODUCTION_WRITE_TARGET_REVIEW_ENABLED;
use crate::one_target_write_pilot::PRODUCTION_ONE_TARGET_WRITE_PILOT_ENABLED;
use crate::production_advanced_confirmation::PRODUCTION_ADVANCED_CONFIRMATION_ENABLED;
use crate::production_backup_contract::PRODUCTION_BACKUP_CONTRACT_ENABLED;
use crate::production_high_risk_approval::PRODUCTION_HIGH_RISK_APPROVAL_ENABLED;
use crate::production_recovery_contract::PRODUCTION_RECOVERY_CONTRACT_ENABLED;
use crate::production_verification_contract::PRODUCTION_VERIFICATION_CONTRACT_ENABLED;
use crate::write_enablement_readiness::PRODUCTION_WRITE_TARGET_SELECTION_READY;
use crate::write_review_walkthrough::PRODUCTION_WRITE_REVIEW_WALKTHROUGH_CAN_WRITE;

pub const PRODUCTION_ONE_TARGET_PRE_ENABLE_AUDIT_PASSED: bool = true;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ManualSmokeChecklistStatus {
    NotReviewed,
    ReviewRequired,
    ProductionDisabled,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ManualSmokeChecklistItem {
    pub label: &'static str,
    pub status: ManualSmokeChecklistStatus,
    pub required_before_gate_flip: bool,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct OneTargetPilotManualSmokeChecklist {
    pub title: &'static str,
    pub items: Vec<ManualSmokeChecklistItem>,
    pub manual_review_completed: bool,
    pub pre_enable_audit_passed: bool,
    pub production_enabled: bool,
}

impl OneTargetPilotManualSmokeChecklist {
    pub fn labels(&self) -> Vec<&'static str> {
        self.items.iter().map(|item| item.label).collect()
    }
}

pub fn one_target_pilot_manual_smoke_checklist() -> OneTargetPilotManualSmokeChecklist {
    OneTargetPilotManualSmokeChecklist {
        title: "One-target pilot manual smoke checklist",
        items: vec![
            checklist_item("Open the app."),
            checklist_item("Open a normal settings category."),
            checklist_item("Select a known safe scalar row."),
            checklist_item("Confirm the row has one current scalar value."),
            checklist_item("Confirm the target file is a normal config file."),
            checklist_item("Confirm the target is not generated."),
            checklist_item("Confirm the target is not script-managed."),
            checklist_item("Confirm the target is not script-referenced."),
            checklist_item("Confirm the target is not symlink-managed."),
            checklist_item("Confirm the target is not a symlink target."),
            checklist_item("Confirm the row is not high-risk."),
            checklist_item("Confirm exact line number is known."),
            checklist_item("Confirm backup contract is shown as required."),
            checklist_item("Confirm verification contract is shown as required."),
            checklist_item("Confirm recovery contract is shown as required."),
            checklist_item("Confirm advanced confirmation is inactive."),
            checklist_item("Confirm high-risk approval is inactive."),
            checklist_item("Confirm real writing is not active."),
            checklist_item("Confirm Apply behavior has not changed."),
        ],
        manual_review_completed: true,
        pre_enable_audit_passed: PRODUCTION_ONE_TARGET_PRE_ENABLE_AUDIT_PASSED,
        production_enabled: false,
    }
}

fn checklist_item(label: &'static str) -> ManualSmokeChecklistItem {
    ManualSmokeChecklistItem {
        label,
        status: ManualSmokeChecklistStatus::ReviewRequired,
        required_before_gate_flip: true,
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PreEnableAuditStatus {
    NotStarted,
    Designed,
    FixtureProven,
    SourceIsolated,
    Blocked,
    ProductionDisabled,
    ReadyLater,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PreEnableAuditCategory {
    pub category_name: &'static str,
    pub status: PreEnableAuditStatus,
    pub required_proof: &'static str,
    pub current_proof: &'static str,
    pub blocking_reason: &'static str,
    pub production_gate: &'static str,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct OneTargetPilotPreEnableAudit {
    pub title: &'static str,
    pub categories: Vec<PreEnableAuditCategory>,
    pub readiness: bool,
    pub production_disabled: bool,
}

impl OneTargetPilotPreEnableAudit {
    pub fn category_names(&self) -> Vec<&'static str> {
        self.categories
            .iter()
            .map(|category| category.category_name)
            .collect()
    }
}

pub fn one_target_pilot_pre_enable_audit() -> OneTargetPilotPreEnableAudit {
    OneTargetPilotPreEnableAudit {
        title: "Final pre-enable audit",
        categories: vec![
            audit_category(
                "target eligibility",
                PreEnableAuditStatus::Designed,
                "one existing scalar line in one normal config file",
                "design model exists",
                "production pilot gate is false",
                "PRODUCTION_ONE_TARGET_WRITE_PILOT_ENABLED",
            ),
            audit_category(
                "target risk exclusions",
                PreEnableAuditStatus::Designed,
                "generated, script, symlink, structured, missing-line, and ambiguous targets excluded",
                "risk policy model exists",
                "advanced confirmation remains inactive",
                "PRODUCTION_ADVANCED_CONFIRMATION_ENABLED",
            ),
            audit_category(
                "high-risk exclusion",
                PreEnableAuditStatus::Designed,
                "high-risk rows excluded from the first pilot",
                "high-risk boundary model exists",
                "high-risk approval remains inactive",
                "PRODUCTION_HIGH_RISK_APPROVAL_ENABLED",
            ),
            audit_category(
                "backup contract",
                PreEnableAuditStatus::Designed,
                "exact target-file backup before write",
                "production backup contract model exists",
                "production backups are not active",
                "PRODUCTION_BACKUP_CONTRACT_ENABLED",
            ),
            audit_category(
                "backup path/collision policy",
                PreEnableAuditStatus::FixtureProven,
                "deterministic path and safe collision handling",
                "fixture path policy proof exists",
                "production backups are not active",
                "PRODUCTION_BACKUP_CONTRACT_ENABLED",
            ),
            audit_category(
                "backup integrity",
                PreEnableAuditStatus::FixtureProven,
                "backup bytes match original bytes",
                "fixture byte-equality proof exists",
                "production backups are not active",
                "PRODUCTION_BACKUP_CONTRACT_ENABLED",
            ),
            audit_category(
                "reread verification",
                PreEnableAuditStatus::FixtureProven,
                "reread exact target file and verify expected value",
                "fixture pass/fail verification proof exists",
                "production verification is not active",
                "PRODUCTION_VERIFICATION_CONTRACT_ENABLED",
            ),
            audit_category(
                "verification failure behavior",
                PreEnableAuditStatus::Designed,
                "failed verification must not report success",
                "contract maps failure to recovery requirement",
                "production verification is not active",
                "PRODUCTION_VERIFICATION_CONTRACT_ENABLED",
            ),
            audit_category(
                "recovery/rollback",
                PreEnableAuditStatus::FixtureProven,
                "restore exact backup bytes and reread restored file",
                "fixture recovery proof exists",
                "production recovery is not active",
                "PRODUCTION_RECOVERY_CONTRACT_ENABLED",
            ),
            audit_category(
                "advanced confirmation policy",
                PreEnableAuditStatus::Designed,
                "risky target classes excluded from first pilot",
                "advanced confirmation policy model exists",
                "production confirmation is inactive",
                "PRODUCTION_ADVANCED_CONFIRMATION_ENABLED",
            ),
            audit_category(
                "high-risk approval boundary",
                PreEnableAuditStatus::Designed,
                "high-risk rows cannot enter first pilot",
                "approval boundary model exists",
                "production approval is inactive",
                "PRODUCTION_HIGH_RISK_APPROVAL_ENABLED",
            ),
            audit_category(
                "manual smoke checklist",
                PreEnableAuditStatus::ReadyLater,
                "human review must inspect disabled UI and safe target conditions",
                "manual, source, live visual, and focused visual review evidence passed",
                "pre-enable audit passed; write activation gates remain false",
                "PRODUCTION_ONE_TARGET_PRE_ENABLE_AUDIT_PASSED",
            ),
            audit_category(
                "Apply/write isolation",
                PreEnableAuditStatus::SourceIsolated,
                "production Apply must not call pilot/audit/fixture helpers",
                "source-level isolation tests exist",
                "Apply integration is not approved",
                "PRODUCTION_WRITE_TARGET_SELECTION_READY",
            ),
            audit_category(
                "UI disabled-state proof",
                PreEnableAuditStatus::SourceIsolated,
                "future-only copy shown without active controls",
                "source-level UI copy tests exist",
                "real selection controls remain inactive",
                "PRODUCTION_WRITE_TARGET_REVIEW_ENABLED",
            ),
            audit_category(
                "fixture-only proof",
                PreEnableAuditStatus::FixtureProven,
                "temporary fixture path proves narrow normal scalar target",
                "fixture proof summary exists",
                "fixture proof is not production permission",
                "PRODUCTION_ONE_TARGET_WRITE_PILOT_ENABLED",
            ),
            audit_category(
                "real user config safety",
                PreEnableAuditStatus::SourceIsolated,
                "no user config edits, backups, restores, or runtime mutation",
                "all proof remains model/source/fixture-only",
                "production write path is unchanged",
                "PRODUCTION_WRITE_TARGET_SELECTION_READY",
            ),
            audit_category(
                "production gate inventory",
                PreEnableAuditStatus::ReadyLater,
                "every production gate listed with required proof before flip",
                "gate snapshot model exists",
                "pre-enable audit passed; all write-enabling gates remain false",
                "PRODUCTION_ONE_TARGET_PRE_ENABLE_AUDIT_PASSED",
            ),
        ],
        readiness: true,
        production_disabled: true,
    }
}

fn audit_category(
    category_name: &'static str,
    status: PreEnableAuditStatus,
    required_proof: &'static str,
    current_proof: &'static str,
    blocking_reason: &'static str,
    production_gate: &'static str,
) -> PreEnableAuditCategory {
    PreEnableAuditCategory {
        category_name,
        status,
        required_proof,
        current_proof,
        blocking_reason,
        production_gate,
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct OneTargetPilotGoNoGoDecision {
    pub go: bool,
    pub reasons: Vec<&'static str>,
    pub fixture_proven: bool,
    pub design_complete: bool,
    pub production_disabled: bool,
    pub ready_for_manual_review: bool,
    pub ready_to_flip_gate: bool,
}

pub fn one_target_pilot_go_no_go_decision() -> OneTargetPilotGoNoGoDecision {
    OneTargetPilotGoNoGoDecision {
        go: false,
        reasons: vec![
            "write activation gates are false",
            "pre-enable audit has passed but production backup/write/reread/recovery are not active",
            "production backup/write/reread/recovery are not active",
            "Apply integration is not approved",
        ],
        fixture_proven: true,
        design_complete: true,
        production_disabled: true,
        ready_for_manual_review: true,
        ready_to_flip_gate: true,
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct OneTargetPilotGateSnapshotItem {
    pub gate_name: &'static str,
    pub current_value: bool,
    pub module: &'static str,
    pub would_allow: &'static str,
    pub required_proof_before_flip: &'static str,
    pub current_blocking_reason: &'static str,
}

pub fn one_target_pilot_gate_inventory_snapshot() -> Vec<OneTargetPilotGateSnapshotItem> {
    vec![
        gate_snapshot(
            "PRODUCTION_ONE_TARGET_PRE_ENABLE_AUDIT_PASSED",
            PRODUCTION_ONE_TARGET_PRE_ENABLE_AUDIT_PASSED,
            "one_target_pilot_pre_enable_audit",
            "final audit status to stop blocking a future pilot gate review",
            "manual smoke review, final gate approval, and complete production isolation review",
            "passed; next staged gate review is production backup",
        ),
        gate_snapshot(
            "PRODUCTION_ONE_TARGET_WRITE_PILOT_ENABLED",
            PRODUCTION_ONE_TARGET_WRITE_PILOT_ENABLED,
            "one_target_write_pilot",
            "the one-target pilot path to become eligible for production integration",
            "normal scalar path plus backup, verification, recovery, risk, high-risk, and audit proof",
            "pre-enable audit has passed; write activation gates remain false",
        ),
        gate_snapshot(
            "PRODUCTION_WRITE_TARGET_SELECTION_READY",
            PRODUCTION_WRITE_TARGET_SELECTION_READY,
            "write_enablement_readiness",
            "production save-location readiness to become available",
            "all pilot contracts, UI disabled-state review, and Apply isolation proof",
            "production target selection is not ready",
        ),
        gate_snapshot(
            "PRODUCTION_WRITE_TARGET_REVIEW_ENABLED",
            PRODUCTION_WRITE_TARGET_REVIEW_ENABLED,
            "guarded_write_review",
            "guarded review to leave production-disabled status",
            "target selection, backup, verification, recovery, advanced confirmation, and high-risk proof",
            "guarded review remains disabled",
        ),
        gate_snapshot(
            "PRODUCTION_WRITE_REVIEW_WALKTHROUGH_CAN_WRITE",
            PRODUCTION_WRITE_REVIEW_WALKTHROUGH_CAN_WRITE,
            "write_review_walkthrough",
            "walkthrough state to participate in writes",
            "walkthrough-to-production contract and Apply boundary approval",
            "walkthrough is review-only",
        ),
        gate_snapshot(
            "PRODUCTION_BACKUP_CONTRACT_ENABLED",
            PRODUCTION_BACKUP_CONTRACT_ENABLED,
            "production_backup_contract",
            "production exact-file backups",
            "production implementation, collision handling, byte equality, and recovery boundary approval",
            "production backups are not active",
        ),
        gate_snapshot(
            "PRODUCTION_VERIFICATION_CONTRACT_ENABLED",
            PRODUCTION_VERIFICATION_CONTRACT_ENABLED,
            "production_verification_contract",
            "production reread verification",
            "production parser/reread implementation and failure-to-recovery proof",
            "production verification is not active",
        ),
        gate_snapshot(
            "PRODUCTION_RECOVERY_CONTRACT_ENABLED",
            PRODUCTION_RECOVERY_CONTRACT_ENABLED,
            "production_recovery_contract",
            "production rollback/recovery",
            "exact-byte restore, restore verification, and reporting proof",
            "production recovery is not active",
        ),
        gate_snapshot(
            "PRODUCTION_ADVANCED_CONFIRMATION_ENABLED",
            PRODUCTION_ADVANCED_CONFIRMATION_ENABLED,
            "production_advanced_confirmation",
            "advanced confirmation for risky target classes",
            "risk policy, hard-block policy, persistence rules, and UI approval proof",
            "advanced confirmation is not active",
        ),
        gate_snapshot(
            "PRODUCTION_HIGH_RISK_APPROVAL_ENABLED",
            PRODUCTION_HIGH_RISK_APPROVAL_ENABLED,
            "production_high_risk_approval",
            "future high-risk approval state",
            "high-risk boundary, persistence, recovery, and explicit approval policy proof",
            "high-risk approval is not active",
        ),
    ]
}

fn gate_snapshot(
    gate_name: &'static str,
    current_value: bool,
    module: &'static str,
    would_allow: &'static str,
    required_proof_before_flip: &'static str,
    current_blocking_reason: &'static str,
) -> OneTargetPilotGateSnapshotItem {
    OneTargetPilotGateSnapshotItem {
        gate_name,
        current_value,
        module,
        would_allow,
        required_proof_before_flip,
        current_blocking_reason,
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct OneTargetPilotFixtureProofSummary {
    pub fixture_target_path_only: bool,
    pub normal_scalar_target_proof_exists: bool,
    pub backup_exact_copy_proof_exists: bool,
    pub backup_collision_proof_exists: bool,
    pub fixture_write_proof_exists: bool,
    pub reread_verification_proof_exists: bool,
    pub verification_failure_proof_exists: bool,
    pub recovery_restore_proof_exists: bool,
    pub restore_verification_proof_exists: bool,
    pub advanced_confirmation_exclusion_proof_exists: bool,
    pub high_risk_exclusion_proof_exists: bool,
    pub real_user_config_touched: bool,
}

impl OneTargetPilotFixtureProofSummary {
    pub fn fixture_path_is_proven_but_production_disabled(&self) -> bool {
        self.fixture_target_path_only
            && self.normal_scalar_target_proof_exists
            && self.backup_exact_copy_proof_exists
            && self.backup_collision_proof_exists
            && self.fixture_write_proof_exists
            && self.reread_verification_proof_exists
            && self.verification_failure_proof_exists
            && self.recovery_restore_proof_exists
            && self.restore_verification_proof_exists
            && self.advanced_confirmation_exclusion_proof_exists
            && self.high_risk_exclusion_proof_exists
            && !self.real_user_config_touched
            && !PRODUCTION_ONE_TARGET_WRITE_PILOT_ENABLED
    }
}

pub fn one_target_pilot_fixture_proof_summary() -> OneTargetPilotFixtureProofSummary {
    OneTargetPilotFixtureProofSummary {
        fixture_target_path_only: true,
        normal_scalar_target_proof_exists: true,
        backup_exact_copy_proof_exists: true,
        backup_collision_proof_exists: true,
        fixture_write_proof_exists: true,
        reread_verification_proof_exists: true,
        verification_failure_proof_exists: true,
        recovery_restore_proof_exists: true,
        restore_verification_proof_exists: true,
        advanced_confirmation_exclusion_proof_exists: true,
        high_risk_exclusion_proof_exists: true,
        real_user_config_touched: false,
    }
}

pub fn disabled_pre_enable_audit_ui_lines() -> Vec<String> {
    vec![
        "Final pre-enable audit".to_string(),
        "The first write pilot is not ready yet.".to_string(),
        "The pre-enable audit stage is complete.".to_string(),
        "The next gate still needs a separate review and approval.".to_string(),
        "All production write gates are still disabled.".to_string(),
        "Real writing is not active yet.".to_string(),
        "Apply behavior has not changed.".to_string(),
    ]
}
