use hyprland_settings::production_recovery_contract::{
    all_recovery_trigger_decisions, recovery_trigger_decision, RecoveryTriggerAction,
    RecoveryTriggerCondition, PRODUCTION_RECOVERY_CONTRACT_ENABLED,
};
use hyprland_settings::write_classification::SAFE_WRITABLE_ROWS;

#[test]
fn recovery_trigger_model_classifies_restore_block_report_and_cancel_paths() {
    for condition in [
        RecoveryTriggerCondition::WriteFailedAfterBackup,
        RecoveryTriggerCondition::WriteSucceededVerificationFailed,
        RecoveryTriggerCondition::ExpectedSettingMissingAfterWrite,
        RecoveryTriggerCondition::ExpectedValueMismatchAfterWrite,
        RecoveryTriggerCondition::TargetUnreadableAfterWrite,
    ] {
        let decision = recovery_trigger_decision(condition);
        assert_eq!(decision.action, RecoveryTriggerAction::ShouldRestoreBackup);
        assert!(decision.should_restore_backup());
        assert!(decision.production_enabled);
        assert!(decision.fixture_only_recovery_allowed);
        assert!(!decision.hyprland_reload_allowed);
    }

    assert_eq!(
        recovery_trigger_decision(RecoveryTriggerCondition::BackupIntegrityMissingBeforeWrite)
            .action,
        RecoveryTriggerAction::ShouldBlockBeforeWriteBegins
    );
    assert_eq!(
        recovery_trigger_decision(RecoveryTriggerCondition::BackupRestoreFailed).action,
        RecoveryTriggerAction::ShouldReportFailureOnly
    );
    assert_eq!(
        recovery_trigger_decision(RecoveryTriggerCondition::UserCancellationBeforeWrite).action,
        RecoveryTriggerAction::ShouldNotRestoreBackup
    );
    assert_eq!(all_recovery_trigger_decisions().len(), 8);
    assert!(PRODUCTION_RECOVERY_CONTRACT_ENABLED);
    assert_eq!(SAFE_WRITABLE_ROWS.len(), 341);
}
