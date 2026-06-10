use std::fs;
use std::path::{Path, PathBuf};

use hyprland_settings::production_backup_contract::fixture_backup_exact_copy;
use hyprland_settings::production_recovery_contract::{
    fixture_restore_backup_bytes, planned_recovery_restore_operation, FixtureRestoreError,
    RestoreOperationStatus, PRODUCTION_RECOVERY_CONTRACT_ENABLED,
};
use hyprland_settings::write_classification::SAFE_WRITABLE_ROWS;
use hyprland_settings::write_target_candidate::WriteTargetCandidate;

fn temp_fixture(name: &str) -> PathBuf {
    let root = std::env::temp_dir().join(format!(
        "hyprland-settings-production-recovery-restore-{name}-{}",
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
fn restore_operation_contract_restores_exact_backup_bytes_in_fixture() {
    let root = temp_fixture("exact");
    let config = root.join("hyprland.conf");
    let unrelated = root.join("other.conf");
    write_file(&config, "# header\ngeneral:layout = dwindle\n");
    write_file(&unrelated, "misc:disable_hyprland_logo = true\n");
    let backup = fixture_backup_exact_copy(&config, "20260610T100000Z")
        .expect("fixture backup should copy exact bytes");
    write_file(&config, "# header\ngeneral:layout = master\n");
    let unrelated_before = fs::read_to_string(&unrelated).expect("unrelated should read");

    let operation = planned_recovery_restore_operation(
        &candidate(config.clone()),
        backup.backup_path.clone(),
        Some(backup.backup_metadata.byte_len),
    );
    assert_eq!(
        operation.restore_status,
        RestoreOperationStatus::ProductionDisabled
    );
    assert!(operation.writes_exact_backup_bytes);
    assert!(operation.modifies_only_target_file);
    assert!(!operation.follows_symlink_target);

    let restored = fixture_restore_backup_bytes(&operation).expect("fixture restore should pass");
    assert_eq!(
        restored.restore_status,
        RestoreOperationStatus::PassedInFixture
    );
    assert_eq!(
        fs::read(&config).expect("target should read"),
        fs::read(&backup.backup_path).expect("backup should read")
    );
    assert_eq!(
        fs::read_to_string(&unrelated).expect("unrelated should read"),
        unrelated_before
    );
    assert!(!PRODUCTION_RECOVERY_CONTRACT_ENABLED);
    assert_eq!(SAFE_WRITABLE_ROWS.len(), 341);
}

#[test]
fn restore_operation_blocks_user_config_and_managed_targets() {
    let root = temp_fixture("blocked");
    let config = root.join("hyprland.conf");
    write_file(&config, "general:layout = dwindle\n");
    let backup = fixture_backup_exact_copy(&config, "20260610T100000Z")
        .expect("fixture backup should copy exact bytes");

    let non_temp = planned_recovery_restore_operation(
        &candidate(PathBuf::from("/home/kyo/.config/hypr/hyprland.conf")),
        backup.backup_path.clone(),
        Some(backup.backup_metadata.byte_len),
    );
    assert_eq!(
        fixture_restore_backup_bytes(&non_temp).expect_err("user config should reject"),
        FixtureRestoreError::NonFixturePath
    );

    let mut managed = candidate(config);
    managed.generated_or_script_managed = true;
    let blocked = planned_recovery_restore_operation(
        &managed,
        backup.backup_path,
        Some(backup.backup_metadata.byte_len),
    );
    assert_eq!(
        blocked.restore_status,
        RestoreOperationStatus::BlockedBeforeWrite
    );
    assert!(blocked.generated_script_symlink_targets_blocked);
    assert_eq!(
        fixture_restore_backup_bytes(&blocked).expect_err("managed target should reject"),
        FixtureRestoreError::BlockedTarget
    );
    assert_eq!(SAFE_WRITABLE_ROWS.len(), 341);
}
