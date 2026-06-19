use std::fs;
use std::path::{Path, PathBuf};
use std::time::{SystemTime, UNIX_EPOCH};

use hyprland_settings::missing_default_insertion::{
    build_missing_default_insertion_plan, execute_missing_default_insertion_plan,
    MissingDefaultInsertionOptions, MissingDefaultInsertionRequest, MissingDefaultInsertionStatus,
};

fn temp_root(label: &str) -> PathBuf {
    let stamp = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .expect("time should work")
        .as_nanos();
    let root = std::env::temp_dir().join(format!(
        "hyprland-settings-missing-default-{label}-{}-{stamp}",
        std::process::id()
    ));
    fs::create_dir_all(&root).expect("temp root should be created");
    root
}

fn write_file(path: &Path, contents: &str) {
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent).expect("parent should exist");
    }
    fs::write(path, contents).expect("fixture should write");
}

fn request(path: &Path) -> MissingDefaultInsertionRequest {
    MissingDefaultInsertionRequest {
        setting_id: "misc.disable_splash_rendering".to_string(),
        proposed_value: "true".to_string(),
        target_path: path.to_path_buf(),
        backup_stamp: "fixture".to_string(),
    }
}

#[test]
fn safe_env_missing_default_insertion_succeeds_for_normal_scalar() {
    let root = temp_root("success");
    let config = root.join("hyprland.conf");
    write_file(&config, "decoration:blur:enabled = true\n");
    let plan = build_missing_default_insertion_plan(request(&config));

    assert!(plan.can_execute, "{:?}", plan.blocked_reasons);
    assert!(!plan.production_enabled);
    assert_eq!(plan.config_key, "misc:disable_splash_rendering");
    assert!(plan
        .insertion_line
        .contains("misc:disable_splash_rendering = true"));

    let report =
        execute_missing_default_insertion_plan(&plan, &MissingDefaultInsertionOptions::default());

    assert_eq!(report.status, MissingDefaultInsertionStatus::Succeeded);
    assert!(report.backup_created);
    assert!(report.backup_bytes_equal);
    assert!(report.inserted_line_verified);
    assert!(!report.real_config_touched);
    assert!(!report.runtime_touched);
    assert!(!report.production_behavior_enabled);
    let updated = fs::read_to_string(&config).expect("config should read");
    assert!(
        updated.contains("# Added by Hyprland Settings safe-env missing/default insertion proof")
    );
    assert!(updated.contains("misc:disable_splash_rendering = true"));
}

#[test]
fn missing_default_insertion_blocks_existing_duplicate_target() {
    let root = temp_root("already-configured");
    let config = root.join("hyprland.conf");
    write_file(&config, "misc:disable_splash_rendering = false\n");
    let plan = build_missing_default_insertion_plan(request(&config));

    assert!(!plan.can_execute);
    assert!(plan
        .blocked_reasons
        .iter()
        .any(|reason| reason.contains("already configured")));
    let report =
        execute_missing_default_insertion_plan(&plan, &MissingDefaultInsertionOptions::default());
    assert_eq!(report.status, MissingDefaultInsertionStatus::Blocked);
    assert!(!report.backup_created);
}

#[test]
fn missing_default_insertion_restores_after_forced_verification_failure() {
    let root = temp_root("restore");
    let config = root.join("hyprland.conf");
    let original = "decoration:blur:enabled = true\n";
    write_file(&config, original);
    let plan = build_missing_default_insertion_plan(request(&config));

    let report = execute_missing_default_insertion_plan(
        &plan,
        &MissingDefaultInsertionOptions {
            force_verification_failure: true,
            ..MissingDefaultInsertionOptions::default()
        },
    );

    assert_eq!(
        report.status,
        MissingDefaultInsertionStatus::RecoveredFailure
    );
    assert!(report.recovery_attempted);
    assert!(report.recovery_succeeded);
    assert_eq!(
        fs::read_to_string(&config).expect("config should read"),
        original
    );
}

#[test]
fn missing_default_insertion_reports_unrecovered_restore_failure() {
    let root = temp_root("restore-failure");
    let config = root.join("hyprland.conf");
    write_file(&config, "decoration:blur:enabled = true\n");
    let plan = build_missing_default_insertion_plan(request(&config));

    let report = execute_missing_default_insertion_plan(
        &plan,
        &MissingDefaultInsertionOptions {
            force_write_failure: true,
            force_restore_failure: true,
            ..MissingDefaultInsertionOptions::default()
        },
    );

    assert_eq!(
        report.status,
        MissingDefaultInsertionStatus::UnrecoveredFailure
    );
    assert!(report.recovery_attempted);
    assert!(!report.recovery_succeeded);
    assert!(report
        .failures
        .iter()
        .any(|failure| failure.contains("forced restore failure")));
}
