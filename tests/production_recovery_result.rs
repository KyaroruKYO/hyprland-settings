use std::path::PathBuf;

use hyprland_settings::production_recovery_contract::{
    planned_recovery_restore_operation, planned_restore_verification, recovery_report_for,
    recovery_trigger_decision, RecoveryReportStatus, RecoveryTriggerCondition,
    RestoreOperationStatus, RestoreVerificationStatus, PRODUCTION_RECOVERY_CONTRACT_ENABLED,
};
use hyprland_settings::write_classification::SAFE_WRITABLE_ROWS;
use hyprland_settings::write_target_candidate::WriteTargetCandidate;

fn candidate() -> WriteTargetCandidate {
    WriteTargetCandidate {
        label: "Main config".to_string(),
        file_path: PathBuf::from("/tmp/hyprland.conf"),
        resolved_path: None,
        line_number: Some(2),
        safe: true,
        generated_or_script_managed: false,
        symlink_managed: false,
        requires_advanced_confirmation: false,
        backup_required: true,
        fixture_only: true,
    }
}

#[test]
fn recovery_report_represents_success_failure_skip_and_backup_availability() {
    let trigger =
        recovery_trigger_decision(RecoveryTriggerCondition::WriteSucceededVerificationFailed);
    let mut restore =
        planned_recovery_restore_operation(&candidate(), "/tmp/hyprland.conf.bak", Some(10));
    let mut verification =
        planned_restore_verification("/tmp/hyprland.conf", "/tmp/hyprland.conf.bak", None);

    restore.restore_status = RestoreOperationStatus::PassedInFixture;
    verification.restore_verification_status = RestoreVerificationStatus::PassedInFixture;
    let success = recovery_report_for(&trigger, &restore, &verification);
    assert_eq!(success.status, RecoveryReportStatus::RecoverySucceeded);
    assert!(success.recovery_attempted);
    assert!(success.user_facing_summary.contains("restored the backup"));

    verification.restore_verification_status = RestoreVerificationStatus::FailedInFixture;
    let failure = recovery_report_for(&trigger, &restore, &verification);
    assert_eq!(failure.status, RecoveryReportStatus::RecoveryFailed);
    assert!(failure.safe_next_action.contains("Keep the backup file"));
    assert_eq!(failure.backup_path, PathBuf::from("/tmp/hyprland.conf.bak"));

    let cancel = recovery_trigger_decision(RecoveryTriggerCondition::UserCancellationBeforeWrite);
    let skipped = recovery_report_for(&cancel, &restore, &verification);
    assert_eq!(skipped.status, RecoveryReportStatus::RecoverySkipped);
    assert!(skipped.recovery_skipped);
    assert!(!skipped.recovery_attempted);
    assert!(skipped.production_enabled);
    assert!(PRODUCTION_RECOVERY_CONTRACT_ENABLED);
    assert_eq!(SAFE_WRITABLE_ROWS.len(), 341);
}
