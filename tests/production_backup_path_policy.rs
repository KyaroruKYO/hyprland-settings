use std::fs;
use std::path::{Path, PathBuf};

use hyprland_settings::production_backup_contract::{
    backup_path_policy_for_target, choose_unique_backup_path, fixture_backup_exact_copy,
    FixtureBackupContractError,
};
use hyprland_settings::write_classification::SAFE_WRITABLE_ROWS;

fn temp_fixture(name: &str) -> PathBuf {
    let root = std::env::temp_dir().join(format!(
        "hyprland-settings-production-backup-policy-{name}-{}",
        std::process::id()
    ));
    let _ = fs::remove_dir_all(&root);
    fs::create_dir_all(&root).expect("fixture root should be created");
    root
}

fn write_file(path: &Path, content: &str) {
    fs::write(path, content).expect("fixture file should be written");
}

#[test]
fn fixed_timestamp_produces_deterministic_backup_path_and_collision_path() {
    let root = temp_fixture("collision");
    let config = root.join("hyprland.conf");
    write_file(&config, "general:layout = dwindle\n");

    let policy = backup_path_policy_for_target(&config, "20260610T090000Z");
    assert_eq!(policy.backup_directory, root);
    assert_eq!(
        policy.first_candidate_path,
        config.with_file_name("hyprland.conf.20260610T090000Z.bak")
    );

    let first = choose_unique_backup_path(&config, "20260610T090000Z");
    write_file(&first, "existing backup");
    let second = choose_unique_backup_path(&config, "20260610T090000Z");
    assert_eq!(
        second,
        config.with_file_name("hyprland.conf.20260610T090000Z.1.bak")
    );
    assert_eq!(SAFE_WRITABLE_ROWS.len(), 341);
}

#[test]
fn fixture_backup_writes_exact_copy_and_rejects_non_fixture_paths() {
    let root = temp_fixture("copy");
    let config = root.join("hyprland.conf");
    write_file(
        &config,
        "# header\ngeneral:layout = dwindle\nmisc:disable_hyprland_logo = true\n",
    );

    let proof = fixture_backup_exact_copy(&config, "20260610T090000Z")
        .expect("fixture backup should copy bytes");
    assert!(proof.fixture_only);
    assert!(proof.backup_path.starts_with(std::env::temp_dir()));
    assert_eq!(
        proof.original_metadata.byte_len,
        proof.backup_metadata.byte_len
    );
    assert!(proof.bytes_equal);
    assert_eq!(
        fs::read(&config).expect("target should read"),
        fs::read(&proof.backup_path).expect("backup should read")
    );

    let rejected = fixture_backup_exact_copy("/home/kyo/.config/hypr/hyprland.conf", "x")
        .expect_err("user config path must be rejected by fixture proof");
    assert_eq!(rejected, FixtureBackupContractError::NonFixturePath);
    assert_eq!(SAFE_WRITABLE_ROWS.len(), 341);
}
