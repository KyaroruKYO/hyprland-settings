use std::fs;
use std::path::{Path, PathBuf};

use hyprland_settings::production_backup_contract::fixture_backup_exact_copy;
use hyprland_settings::production_recovery_contract::{
    fixture_restore_backup_bytes, fixture_verify_restored_file, planned_recovery_restore_operation,
    planned_restore_verification, RestoreVerificationStatus, PRODUCTION_RECOVERY_CONTRACT_ENABLED,
};
use hyprland_settings::write_classification::SAFE_WRITABLE_ROWS;
use hyprland_settings::write_target_candidate::WriteTargetCandidate;

fn temp_fixture(name: &str) -> PathBuf {
    let root = std::env::temp_dir().join(format!(
        "hyprland-settings-production-recovery-verification-{name}-{}",
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
fn restore_verification_rereads_exact_file_bytes_and_scalar_value() {
    let root = temp_fixture("pass");
    let config = root.join("hyprland.conf");
    write_file(&config, "# header\ngeneral:layout = dwindle\n");
    let backup = fixture_backup_exact_copy(&config, "20260610T100000Z")
        .expect("fixture backup should copy exact bytes");
    write_file(&config, "# header\ngeneral:layout = master\n");
    let operation = planned_recovery_restore_operation(
        &candidate(config.clone()),
        backup.backup_path.clone(),
        Some(backup.backup_metadata.byte_len),
    );
    fixture_restore_backup_bytes(&operation).expect("fixture restore should pass");

    let verification =
        planned_restore_verification(&config, &backup.backup_path, Some("dwindle".to_string()));
    assert_eq!(
        verification.restore_verification_status,
        RestoreVerificationStatus::ProductionDisabled
    );
    let verified = fixture_verify_restored_file(&verification, Some("general.layout"))
        .expect("restored fixture should verify");
    assert_eq!(
        verified.restore_verification_status,
        RestoreVerificationStatus::PassedInFixture
    );
    assert!(verified.bytes_match_backup);
    assert_eq!(verified.observed_restored_value.as_deref(), Some("dwindle"));
    assert!(!PRODUCTION_RECOVERY_CONTRACT_ENABLED);
    assert_eq!(SAFE_WRITABLE_ROWS.len(), 341);
}

#[test]
fn restore_verification_failure_can_be_represented() {
    let root = temp_fixture("fail");
    let config = root.join("hyprland.conf");
    write_file(&config, "general:layout = dwindle\n");
    let backup = fixture_backup_exact_copy(&config, "20260610T100000Z")
        .expect("fixture backup should copy exact bytes");
    write_file(&config, "general:layout = master\n");

    let verification =
        planned_restore_verification(&config, &backup.backup_path, Some("dwindle".to_string()));
    let verified = fixture_verify_restored_file(&verification, Some("general.layout"))
        .expect("fixture verification should report mismatch");
    assert_eq!(
        verified.restore_verification_status,
        RestoreVerificationStatus::FailedInFixture
    );
    assert!(!verified.bytes_match_backup);
    assert_eq!(SAFE_WRITABLE_ROWS.len(), 341);
}
