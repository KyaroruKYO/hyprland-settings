use std::fs;
use std::path::{Path, PathBuf};

use hyprland_settings::production_backup_contract::fixture_backup_exact_copy;
use hyprland_settings::production_recovery_contract::{
    fixture_restore_backup_bytes, fixture_verify_restored_file, planned_restore_from_backup_proof,
    planned_restore_verification, recovery_trigger_decision, RecoveryTriggerCondition,
    RestoreVerificationStatus, PRODUCTION_RECOVERY_CONTRACT_ENABLED,
};
use hyprland_settings::production_verification_contract::{
    fixture_reread_verify_expected_value, production_verification_contract_for_candidate,
    ProductionVerificationStatus,
};
use hyprland_settings::write_classification::SAFE_WRITABLE_ROWS;
use hyprland_settings::write_target_candidate::WriteTargetCandidate;
use hyprland_settings::write_target_fixture_proof::{
    prove_fixture_target_write, FixtureTargetWriteProofRequest,
};

fn temp_fixture(name: &str) -> PathBuf {
    let root = std::env::temp_dir().join(format!(
        "hyprland-settings-production-recovery-fixture-{name}-{}",
        std::process::id()
    ));
    let _ = fs::remove_dir_all(&root);
    fs::create_dir_all(&root).expect("fixture root should be created");
    root
}

fn write_file(path: &Path, content: &str) {
    fs::write(path, content).expect("fixture file should be written");
}

fn candidate(path: PathBuf) -> WriteTargetCandidate {
    WriteTargetCandidate {
        label: "Main config".to_string(),
        file_path: path,
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
fn fixture_rollback_recovery_restores_backup_after_failed_verification() {
    let root = temp_fixture("roundtrip");
    let config = root.join("hyprland.conf");
    write_file(
        &config,
        "# header\ngeneral:layout = dwindle\nmisc:disable_hyprland_logo = true\n",
    );
    let original = fs::read(&config).expect("original should read");
    let candidate = candidate(config.clone());
    let backup = fixture_backup_exact_copy(&config, "20260610T100000Z")
        .expect("fixture backup should copy exact bytes");

    prove_fixture_target_write(&FixtureTargetWriteProofRequest {
        target: candidate.clone(),
        setting_id: "general.layout".to_string(),
        new_value: "master".to_string(),
        advanced_fixture_approval: false,
    })
    .expect("fixture write proof should change scalar value");

    let failed_verification =
        production_verification_contract_for_candidate(&candidate, "general.layout", "dwindle");
    let failed = fixture_reread_verify_expected_value(&failed_verification)
        .expect("fixture reread should produce mismatch");
    assert_eq!(failed.status, ProductionVerificationStatus::FailedInFixture);

    let trigger =
        recovery_trigger_decision(RecoveryTriggerCondition::WriteSucceededVerificationFailed);
    assert!(trigger.should_restore_backup());

    let restore = planned_restore_from_backup_proof(&candidate, &backup);
    let restored = fixture_restore_backup_bytes(&restore).expect("fixture restore should pass");
    assert!(restored.target_file_path.starts_with(std::env::temp_dir()));

    let restore_verification =
        planned_restore_verification(&config, &backup.backup_path, Some("dwindle".to_string()));
    let restore_verified =
        fixture_verify_restored_file(&restore_verification, Some("general.layout"))
            .expect("restored fixture should verify");
    assert_eq!(
        restore_verified.restore_verification_status,
        RestoreVerificationStatus::PassedInFixture
    );
    assert!(restore_verified.bytes_match_backup);
    assert_eq!(
        fs::read(&config).expect("restored target should read"),
        original
    );
    assert_eq!(
        restore_verified.observed_restored_value.as_deref(),
        Some("dwindle")
    );
    assert!(!PRODUCTION_RECOVERY_CONTRACT_ENABLED);
    assert_eq!(SAFE_WRITABLE_ROWS.len(), 341);
}
