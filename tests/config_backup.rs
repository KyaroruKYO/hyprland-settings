use std::fs;
use std::path::PathBuf;
use std::time::{SystemTime, UNIX_EPOCH};

use anyhow::Result;
use hyprland_settings::config_backup::BackupManager;

fn temp_root(name: &str) -> Result<PathBuf> {
    let stamp = SystemTime::now().duration_since(UNIX_EPOCH)?.as_nanos();
    let root = std::env::temp_dir().join(format!(
        "hyprland-settings-backup-{name}-{}-{stamp}",
        std::process::id()
    ));
    fs::create_dir_all(&root)?;
    Ok(root)
}

#[test]
fn backup_is_created_and_verified() -> Result<()> {
    let root = temp_root("create")?;
    let source = root.join("hyprland.conf");
    let backup_root = root.join("backups");
    fs::write(&source, "animations:enabled = true\n")?;

    let backup = BackupManager::new(&backup_root).create_backup(&source)?;

    assert!(backup.backup_path.starts_with(&backup_root));
    assert_eq!(
        fs::read_to_string(&backup.backup_path)?,
        "animations:enabled = true\n"
    );
    assert_eq!(backup.byte_len, "animations:enabled = true\n".len());

    fs::remove_dir_all(root)?;
    Ok(())
}

#[test]
fn backup_never_overwrites_existing_backup() -> Result<()> {
    let root = temp_root("no-overwrite")?;
    let source = root.join("hyprland.conf");
    let backup_root = root.join("backups");
    fs::write(&source, "general:gaps_in = 5\n")?;

    let manager = BackupManager::new(&backup_root);
    let first = manager.create_backup(&source)?;
    let second = manager.create_backup(&source)?;

    assert_ne!(first.backup_path, second.backup_path);
    assert_eq!(
        fs::read_to_string(first.backup_path)?,
        "general:gaps_in = 5\n"
    );
    assert_eq!(
        fs::read_to_string(second.backup_path)?,
        "general:gaps_in = 5\n"
    );

    fs::remove_dir_all(root)?;
    Ok(())
}

#[test]
fn rollback_restores_fixture_content() -> Result<()> {
    let root = temp_root("rollback")?;
    let source = root.join("hyprland.conf");
    let backup_root = root.join("backups");
    fs::write(&source, "misc:disable_hyprland_logo = true\n")?;

    let manager = BackupManager::new(&backup_root);
    let backup = manager.create_backup(&source)?;
    fs::write(&source, "misc:disable_hyprland_logo = false\n")?;

    manager.rollback(&backup)?;

    assert_eq!(
        fs::read_to_string(&source)?,
        "misc:disable_hyprland_logo = true\n"
    );

    fs::remove_dir_all(root)?;
    Ok(())
}

#[test]
fn missing_source_returns_error() -> Result<()> {
    let root = temp_root("missing-source")?;
    let source = root.join("missing.conf");
    let backup_root = root.join("backups");

    let error = BackupManager::new(&backup_root)
        .create_backup(&source)
        .expect_err("missing source should fail");

    assert!(error.to_string().contains("failed to read source config"));

    fs::remove_dir_all(root)?;
    Ok(())
}

#[test]
fn default_backup_root_is_outside_repo_when_home_is_set() -> Result<()> {
    let backup_root = BackupManager::default_user_backup_root()?;
    let repo_root = PathBuf::from(env!("CARGO_MANIFEST_DIR"));

    assert!(!backup_root.starts_with(repo_root));
    assert!(backup_root.ends_with("hyprland-settings/backups"));

    Ok(())
}
