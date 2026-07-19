use std::fs;
use std::os::unix::fs::{symlink, PermissionsExt};
use std::path::Path;

use anyhow::Result;
use hyprland_settings::config_backup::BackupManager;
use hyprland_settings::durable_fs::{
    capture_file_precondition, hardened_atomic_replace, hardened_atomic_replace_with_fault,
    restore_original_after_failed_verification, verify_file_precondition, DurableFsError,
    DurableWriteTestFault,
};
use tempfile::TempDir;

fn write_target(root: &Path, bytes: &[u8], mode: u32) -> Result<std::path::PathBuf> {
    let target = root.join("hyprland.conf");
    fs::write(&target, bytes)?;
    fs::set_permissions(&target, fs::Permissions::from_mode(mode))?;
    Ok(target)
}

fn temporary_write_artifacts(root: &Path) -> Result<Vec<String>> {
    Ok(fs::read_dir(root)?
        .filter_map(Result::ok)
        .filter_map(|entry| entry.file_name().into_string().ok())
        .filter(|name| name.starts_with(".hyprland-settings-write-"))
        .collect())
}

#[test]
fn hardened_replace_preserves_mode_syncs_and_verifies_final_state() -> Result<()> {
    let root = TempDir::new()?;
    let target = write_target(root.path(), b"general:gaps_in = 5\n", 0o640)?;
    let original = capture_file_precondition(&target)?;

    let receipt = hardened_atomic_replace(&original, b"general:gaps_in = 8\n")?;

    assert_eq!(fs::read(&target)?, b"general:gaps_in = 8\n");
    assert_eq!(fs::metadata(&target)?.permissions().mode() & 0o7777, 0o640);
    assert!(receipt.file_synced);
    assert!(receipt.parent_directory_synced);
    assert!(receipt.final_bytes_verified);
    assert!(receipt.final_metadata_verified);
    assert_eq!(receipt.target_mode, 0o640);
    assert!(temporary_write_artifacts(root.path())?.is_empty());
    Ok(())
}

#[test]
fn hardened_preconditions_reject_non_files_symlinks_and_parent_escape() -> Result<()> {
    let root = TempDir::new()?;
    let directory_target = root.path().join("directory-target");
    fs::create_dir(&directory_target)?;
    assert!(matches!(
        capture_file_precondition(&directory_target),
        Err(DurableFsError::TargetNotRegularFile(_))
    ));

    let real_target = write_target(root.path(), b"general:gaps_in = 5\n", 0o600)?;
    let target_link = root.path().join("target-link.conf");
    symlink(&real_target, &target_link)?;
    assert!(matches!(
        capture_file_precondition(&target_link),
        Err(DurableFsError::TargetSymlinkRejected(_))
    ));

    let real_parent = root.path().join("real-parent");
    fs::create_dir(&real_parent)?;
    fs::write(real_parent.join("nested.conf"), b"misc:vfr = true\n")?;
    let parent_link = root.path().join("parent-link");
    symlink(&real_parent, &parent_link)?;
    assert!(matches!(
        capture_file_precondition(parent_link.join("nested.conf")),
        Err(DurableFsError::ParentPathSymlinkRejected(_))
    ));

    let traversal = root.path().join("real-parent/../hyprland.conf");
    assert!(matches!(
        capture_file_precondition(traversal),
        Err(DurableFsError::ParentPathSymlinkRejected(_))
    ));
    Ok(())
}

#[test]
fn exact_bytes_and_inode_are_both_drift_preconditions() -> Result<()> {
    let root = TempDir::new()?;
    let target = write_target(root.path(), b"general:gaps_in = 5\n", 0o600)?;
    let original = capture_file_precondition(&target)?;

    fs::write(&target, b"general:gaps_in = 5\n# unrelated edit\n")?;
    assert!(matches!(
        verify_file_precondition(&original),
        Err(DurableFsError::OnDiskDriftDetected(_))
    ));
    assert!(matches!(
        hardened_atomic_replace(&original, b"general:gaps_in = 8\n"),
        Err(DurableFsError::OnDiskDriftDetected(_))
    ));
    assert_eq!(
        fs::read(&target)?,
        b"general:gaps_in = 5\n# unrelated edit\n"
    );

    fs::write(&target, &original.bytes)?;
    let inode_precondition = capture_file_precondition(&target)?;
    let displaced = root.path().join("displaced.conf");
    fs::rename(&target, &displaced)?;
    fs::write(&target, &inode_precondition.bytes)?;
    assert!(matches!(
        verify_file_precondition(&inode_precondition),
        Err(DurableFsError::TargetIdentityChanged(_))
    ));
    Ok(())
}

#[test]
fn target_and_parent_symlink_substitution_after_review_are_rejected() -> Result<()> {
    let root = TempDir::new()?;
    let target = write_target(root.path(), b"general:gaps_in = 5\n", 0o600)?;
    let target_precondition = capture_file_precondition(&target)?;
    let original_target = root.path().join("original-target.conf");
    let external_target = root.path().join("external-target.conf");
    fs::rename(&target, &original_target)?;
    fs::write(&external_target, b"external bytes\n")?;
    symlink(&external_target, &target)?;

    let error = hardened_atomic_replace(&target_precondition, b"general:gaps_in = 8\n")
        .expect_err("target symlink substitution must fail closed");
    assert!(matches!(
        error,
        DurableFsError::TargetSymlinkRejected(_) | DurableFsError::TargetIdentityChanged(_)
    ));
    assert_eq!(fs::read(&external_target)?, b"external bytes\n");

    let parent = root.path().join("reviewed-parent");
    fs::create_dir(&parent)?;
    let nested = parent.join("hyprland.conf");
    fs::write(&nested, b"misc:vfr = true\n")?;
    let parent_precondition = capture_file_precondition(&nested)?;
    let moved_parent = root.path().join("moved-reviewed-parent");
    let alternate_parent = root.path().join("alternate-parent");
    fs::rename(&parent, &moved_parent)?;
    fs::create_dir(&alternate_parent)?;
    fs::write(
        alternate_parent.join("hyprland.conf"),
        b"external parent bytes\n",
    )?;
    symlink(&alternate_parent, &parent)?;

    let error = hardened_atomic_replace(&parent_precondition, b"misc:vfr = false\n")
        .expect_err("parent symlink substitution must fail closed");
    assert!(matches!(
        error,
        DurableFsError::ParentPathSymlinkRejected(_)
    ));
    assert_eq!(
        fs::read(alternate_parent.join("hyprland.conf"))?,
        b"external parent bytes\n"
    );
    Ok(())
}

#[test]
fn injected_precommit_and_commit_failures_leave_original_and_clean_temps() -> Result<()> {
    for fault in [
        DurableWriteTestFault::FailBeforeCommit,
        DurableWriteTestFault::FailCommit,
    ] {
        let root = TempDir::new()?;
        let target = write_target(root.path(), b"misc:vfr = true\n", 0o600)?;
        let original = capture_file_precondition(&target)?;

        let error = hardened_atomic_replace_with_fault(&original, b"misc:vfr = false\n", fault)
            .expect_err("injected commit failure must fail closed");

        assert!(matches!(error, DurableFsError::CommitFailed(_)));
        assert_eq!(fs::read(&target)?, original.bytes);
        assert!(temporary_write_artifacts(root.path())?.is_empty());
    }
    Ok(())
}

#[test]
fn drift_at_the_atomic_commit_boundary_is_restored_without_overwrite() -> Result<()> {
    let root = TempDir::new()?;
    let target = write_target(root.path(), b"misc:vfr = true\n", 0o600)?;
    let original = capture_file_precondition(&target)?;
    let external = b"# injected concurrent external edit immediately before commit\n";

    let error = hardened_atomic_replace_with_fault(
        &original,
        b"misc:vfr = false\n",
        DurableWriteTestFault::ReplaceTargetImmediatelyBeforeCommit,
    )
    .expect_err("commit-boundary drift must reject the replacement");

    assert!(matches!(error, DurableFsError::OnDiskDriftDetected(_)));
    assert_eq!(fs::read(&target)?, external);
    assert!(temporary_write_artifacts(root.path())?.is_empty());
    Ok(())
}

#[test]
fn postcommit_verification_failure_restores_exact_bytes_and_metadata() -> Result<()> {
    let root = TempDir::new()?;
    let target = write_target(root.path(), b"misc:vrr = 0\n", 0o640)?;
    let original = capture_file_precondition(&target)?;

    let error = hardened_atomic_replace_with_fault(
        &original,
        b"misc:vrr = 1\n",
        DurableWriteTestFault::FailPostCommitVerification,
    )
    .expect_err("verification fault must be surfaced after recovery");

    assert!(matches!(
        error,
        DurableFsError::PostWriteVerificationFailed(_)
    ));
    assert_eq!(fs::read(&target)?, original.bytes);
    assert_eq!(fs::metadata(&target)?.permissions().mode() & 0o7777, 0o640);
    assert!(temporary_write_artifacts(root.path())?.is_empty());
    Ok(())
}

#[test]
fn restore_refuses_to_overwrite_intervening_bytes() -> Result<()> {
    let root = TempDir::new()?;
    let target = write_target(root.path(), b"misc:vrr = 0\n", 0o600)?;
    let original = capture_file_precondition(&target)?;
    let committed = b"misc:vrr = 1\n";
    hardened_atomic_replace(&original, committed)?;
    fs::write(&target, b"# external edit after commit\nmisc:vrr = 2\n")?;

    let error = restore_original_after_failed_verification(&original, committed)
        .expect_err("restore must reject intervening bytes");

    assert!(matches!(error, DurableFsError::RestoreFailed(_)));
    assert_eq!(
        fs::read(&target)?,
        b"# external edit after commit\nmisc:vrr = 2\n"
    );
    Ok(())
}

#[test]
fn unrelated_sibling_temp_name_is_never_overwritten() -> Result<()> {
    let root = TempDir::new()?;
    let target = write_target(root.path(), b"misc:vfr = true\n", 0o600)?;
    let collision = root.path().join(".hyprland-settings-write-collision.tmp");
    fs::write(&collision, b"owned by another operation")?;
    let original = capture_file_precondition(&target)?;

    hardened_atomic_replace(&original, b"misc:vfr = false\n")?;

    assert_eq!(fs::read(&collision)?, b"owned by another operation");
    assert_eq!(
        temporary_write_artifacts(root.path())?,
        vec![".hyprland-settings-write-collision.tmp".to_string()]
    );
    Ok(())
}

#[test]
fn backups_are_exclusive_restrictive_verified_and_symlink_safe() -> Result<()> {
    let root = TempDir::new()?;
    let target = write_target(root.path(), b"general:gaps_in = 5\n", 0o640)?;
    let precondition = capture_file_precondition(&target)?;
    let backup_root = root.path().join("state/backups");
    let manager = BackupManager::new(&backup_root);

    let first = manager.create_backup_from_precondition(&precondition)?;
    let second = manager.create_backup_from_precondition(&precondition)?;
    assert_ne!(first.backup_path, second.backup_path);
    assert_eq!(fs::read(&first.backup_path)?, precondition.bytes);
    assert_eq!(first.source_sha256, first.backup_precondition.sha256);
    assert_eq!(
        fs::metadata(&backup_root)?.permissions().mode() & 0o7777,
        0o700
    );
    assert_eq!(
        fs::metadata(&first.backup_path)?.permissions().mode() & 0o7777,
        0o600
    );

    let fixed = backup_root.join("fixed-collision.bak");
    manager.create_backup_at_path_from_precondition(&precondition, &fixed)?;
    let original_fixed = fs::read(&fixed)?;
    assert!(manager
        .create_backup_at_path_from_precondition(&precondition, &fixed)
        .is_err());
    assert_eq!(fs::read(&fixed)?, original_fixed);

    let real_backup_root = root.path().join("real-backups");
    fs::create_dir(&real_backup_root)?;
    let linked_backup_root = root.path().join("linked-backups");
    symlink(&real_backup_root, &linked_backup_root)?;
    assert!(BackupManager::new(linked_backup_root)
        .create_backup_from_precondition(&precondition)
        .is_err());
    Ok(())
}

#[test]
fn backup_restore_requires_verified_backup_and_expected_current_bytes() -> Result<()> {
    let root = TempDir::new()?;
    let original_bytes = b"general:gaps_in = 5\n";
    let replacement = b"general:gaps_in = 8\n";
    let target = write_target(root.path(), original_bytes, 0o640)?;
    let original = capture_file_precondition(&target)?;
    let manager = BackupManager::new(root.path().join("backups"));
    let backup = manager.create_backup_from_precondition(&original)?;
    hardened_atomic_replace(&original, replacement)?;

    assert!(manager
        .rollback(&backup, b"different expected bytes\n")
        .is_err());
    assert_eq!(fs::read(&target)?, replacement);
    let receipt = manager.rollback(&backup, replacement)?;
    assert_eq!(fs::read(&target)?, original_bytes);
    assert_eq!(fs::metadata(&target)?.permissions().mode() & 0o7777, 0o640);
    assert!(receipt.parent_directory_synced);

    let fresh = capture_file_precondition(&target)?;
    let corrupted = manager.create_backup_from_precondition(&fresh)?;
    fs::write(&corrupted.backup_path, b"corrupt")?;
    hardened_atomic_replace(&fresh, replacement)?;
    assert!(manager.rollback(&corrupted, replacement).is_err());
    assert_eq!(fs::read(&target)?, replacement);
    Ok(())
}
