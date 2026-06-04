use std::collections::BTreeSet;
use std::fs;
use std::path::PathBuf;
use std::time::{SystemTime, UNIX_EPOCH};

use anyhow::Result;
use hyprland_settings::config_backup::BackupManager;
use hyprland_settings::config_discovery::{
    ConfigDiscovery, ConfigDiscoveryStatus, ConfigPathSource,
};
use hyprland_settings::config_parser::parse_hyprland_config_text;
use hyprland_settings::current_config::{CurrentConfigSnapshot, CurrentValueSourceStatus};
use hyprland_settings::pending_change::ACTIVE_PENDING_CHANGE_SETTING;
use hyprland_settings::write_classification::SAFE_WRITABLE_ROWS;
use hyprland_settings::write_flow::{
    apply_setting_change_with_backup_manager, edit_projection_for_setting,
    pending_projection_for_value,
};

fn temp_root(name: &str) -> Result<PathBuf> {
    let stamp = SystemTime::now().duration_since(UNIX_EPOCH)?.as_nanos();
    let root = std::env::temp_dir().join(format!(
        "hyprland-settings-write-flow-{name}-{}-{stamp}",
        std::process::id()
    ));
    fs::create_dir_all(&root)?;
    Ok(root)
}

fn known_ids() -> BTreeSet<String> {
    SAFE_WRITABLE_ROWS
        .iter()
        .map(|row| row.row_id.to_string())
        .collect()
}

fn discovery_for(path: PathBuf) -> ConfigDiscovery {
    ConfigDiscovery {
        status: ConfigDiscoveryStatus::Found {
            path: path.clone(),
            source: ConfigPathSource::HomeFallback,
        },
        attempted_paths: vec![path],
    }
}

fn snapshot_for(path: &PathBuf, contents: &str) -> CurrentConfigSnapshot {
    CurrentConfigSnapshot::from_parsed(parse_hyprland_config_text(path, contents))
}

#[test]
fn edit_projection_allows_only_safe_writable_rows() {
    let current =
        CurrentConfigSnapshot::read_unavailable("no config").value_for("general.snap.enabled");
    let blocked = edit_projection_for_setting("appearance.glow.range", &current);

    for row in SAFE_WRITABLE_ROWS {
        let editable = edit_projection_for_setting(row.row_id, &current);
        assert!(editable.editable, "{} should be editable", row.row_id);
        assert!(
            editable.proposed_value.as_deref().is_some(),
            "{} should provide a proposed value",
            row.row_id
        );
        assert!(!editable.pending.expect("pending projection").can_review);
    }
    assert!(!blocked.editable);
    assert_eq!(
        blocked.disabled_reason.as_deref(),
        Some("not write-allowlisted")
    );
}

#[test]
fn pending_projection_blocks_duplicate_conflict() {
    let parsed = parse_hyprland_config_text(
        "/tmp/hyprland.conf",
        "general:snap:enabled = false\ngeneral:snap:enabled = true\n",
    );
    let current = CurrentConfigSnapshot::from_parsed(parsed).value_for("general.snap.enabled");

    assert_eq!(current.status, CurrentValueSourceStatus::DuplicateConflict);
    let pending = pending_projection_for_value(ACTIVE_PENDING_CHANGE_SETTING, &current, "false");

    assert_eq!(pending.validation_label, "valid");
    assert!(!pending.can_review);
    assert!(pending
        .review_summary
        .iter()
        .any(|line| line.contains("duplicate config entries")));
}

#[test]
fn apply_flow_writes_fixture_and_reports_backup_and_rollback() -> Result<()> {
    let root = temp_root("apply")?;
    let source = root.join("hyprland.conf");
    fs::write(&source, "general:snap:enabled = false\n")?;
    let contents = fs::read_to_string(&source)?;
    let snapshot = snapshot_for(&source, &contents);
    let backup_manager = BackupManager::new(root.join("backups"));

    let outcome = apply_setting_change_with_backup_manager(
        known_ids(),
        &discovery_for(source.clone()),
        &snapshot,
        ACTIVE_PENDING_CHANGE_SETTING,
        "true",
        &backup_manager,
    )
    .map_err(|failure| anyhow::anyhow!("{failure:?}"))?;

    assert_eq!(outcome.setting_id, ACTIVE_PENDING_CHANGE_SETTING);
    assert_eq!(outcome.target_path, source);
    assert!(outcome.backup_path.exists());
    assert_eq!(outcome.rollback_source_path, outcome.target_path);
    assert_eq!(outcome.rollback_backup_path, outcome.backup_path);
    assert_eq!(outcome.verified_value.as_deref(), Some("true"));
    assert!(outcome.reload_note.contains("not performed"));
    assert_eq!(
        fs::read_to_string(&outcome.target_path)?,
        "general:snap:enabled = true\n"
    );

    fs::remove_dir_all(root)?;
    Ok(())
}

#[test]
fn apply_flow_writes_validator_backed_numeric_fixture() -> Result<()> {
    let root = temp_root("apply-numeric")?;
    let source = root.join("hyprland.conf");
    fs::write(&source, "decoration:blur:size = 5\n")?;
    let contents = fs::read_to_string(&source)?;
    let snapshot = snapshot_for(&source, &contents);
    let backup_manager = BackupManager::new(root.join("backups"));

    let outcome = apply_setting_change_with_backup_manager(
        known_ids(),
        &discovery_for(source.clone()),
        &snapshot,
        "appearance.blur.size",
        "10",
        &backup_manager,
    )
    .map_err(|failure| anyhow::anyhow!("{failure:?}"))?;

    assert_eq!(outcome.setting_id, "appearance.blur.size");
    assert_eq!(outcome.target_path, source);
    assert!(outcome.backup_path.exists());
    assert_eq!(outcome.verified_value.as_deref(), Some("10"));
    assert_eq!(
        fs::read_to_string(&outcome.target_path)?,
        "decoration:blur:size = 10\n"
    );

    fs::remove_dir_all(root)?;
    Ok(())
}

#[test]
fn apply_flow_writes_parser_backed_color_fixture() -> Result<()> {
    let root = temp_root("apply-color")?;
    let source = root.join("hyprland.conf");
    fs::write(&source, "misc:background_color = rgba(000000ff)\n")?;
    let contents = fs::read_to_string(&source)?;
    let snapshot = snapshot_for(&source, &contents);
    let backup_manager = BackupManager::new(root.join("backups"));

    let outcome = apply_setting_change_with_backup_manager(
        known_ids(),
        &discovery_for(source.clone()),
        &snapshot,
        "misc.background_color",
        "rgba(ffffffff)",
        &backup_manager,
    )
    .map_err(|failure| anyhow::anyhow!("{failure:?}"))?;

    assert_eq!(outcome.setting_id, "misc.background_color");
    assert_eq!(outcome.target_path, source);
    assert!(outcome.backup_path.exists());
    assert_eq!(outcome.verified_value.as_deref(), Some("rgba(ffffffff)"));
    assert_eq!(
        fs::read_to_string(&outcome.target_path)?,
        "misc:background_color = rgba(ffffffff)\n"
    );

    fs::remove_dir_all(root)?;
    Ok(())
}

#[test]
fn apply_flow_writes_gradient_color_list_fixture() -> Result<()> {
    let root = temp_root("apply-gradient")?;
    let source = root.join("hyprland.conf");
    fs::write(&source, "general:col:active_border = rgba(000000ff)\n")?;
    let contents = fs::read_to_string(&source)?;
    let snapshot = snapshot_for(&source, &contents);
    let backup_manager = BackupManager::new(root.join("backups"));

    let outcome = apply_setting_change_with_backup_manager(
        known_ids(),
        &discovery_for(source.clone()),
        &snapshot,
        "general.col.active_border",
        "rgba(ffffffff) rgba(000000ff) 45deg",
        &backup_manager,
    )
    .map_err(|failure| anyhow::anyhow!("{failure:?}"))?;

    assert_eq!(outcome.setting_id, "general.col.active_border");
    assert_eq!(outcome.target_path, source);
    assert!(outcome.backup_path.exists());
    assert_eq!(
        outcome.verified_value.as_deref(),
        Some("rgba(ffffffff) rgba(000000ff) 45deg")
    );
    assert_eq!(
        fs::read_to_string(&outcome.target_path)?,
        "general:col:active_border = rgba(ffffffff) rgba(000000ff) 45deg\n"
    );

    fs::remove_dir_all(root)?;
    Ok(())
}

#[test]
fn apply_flow_writes_vector_tuple_fixture() -> Result<()> {
    let root = temp_root("apply-vector")?;
    let source = root.join("hyprland.conf");
    fs::write(&source, "decoration:shadow:offset = 0 0\n")?;
    let contents = fs::read_to_string(&source)?;
    let snapshot = snapshot_for(&source, &contents);
    let backup_manager = BackupManager::new(root.join("backups"));

    let outcome = apply_setting_change_with_backup_manager(
        known_ids(),
        &discovery_for(source.clone()),
        &snapshot,
        "decoration.shadow.offset",
        "10,20",
        &backup_manager,
    )
    .map_err(|failure| anyhow::anyhow!("{failure:?}"))?;

    assert_eq!(outcome.setting_id, "decoration.shadow.offset");
    assert_eq!(outcome.target_path, source);
    assert!(outcome.backup_path.exists());
    assert_eq!(outcome.verified_value.as_deref(), Some("10,20"));
    assert_eq!(
        fs::read_to_string(&outcome.target_path)?,
        "decoration:shadow:offset = 10,20\n"
    );

    fs::remove_dir_all(root)?;
    Ok(())
}

#[test]
fn apply_flow_writes_numeric_list_fixture() -> Result<()> {
    let root = temp_root("apply-numeric-list")?;
    let source = root.join("hyprland.conf");
    fs::write(&source, "input:scroll_points = 0.2 0.5 1\n")?;
    let contents = fs::read_to_string(&source)?;
    let snapshot = snapshot_for(&source, &contents);
    let backup_manager = BackupManager::new(root.join("backups"));

    let outcome = apply_setting_change_with_backup_manager(
        known_ids(),
        &discovery_for(source.clone()),
        &snapshot,
        "input.scroll_points",
        "0.2 0.0 0.5 1 1.2 1.5",
        &backup_manager,
    )
    .map_err(|failure| anyhow::anyhow!("{failure:?}"))?;

    assert_eq!(outcome.setting_id, "input.scroll_points");
    assert_eq!(outcome.target_path, source);
    assert!(outcome.backup_path.exists());
    assert_eq!(
        outcome.verified_value.as_deref(),
        Some("0.2 0.0 0.5 1 1.2 1.5")
    );
    assert_eq!(
        fs::read_to_string(&outcome.target_path)?,
        "input:scroll_points = 0.2 0.0 0.5 1 1.2 1.5\n"
    );

    fs::remove_dir_all(root)?;
    Ok(())
}

#[test]
fn apply_flow_writes_enum_custom_string_fixture() -> Result<()> {
    let root = temp_root("apply-string")?;
    let source = root.join("hyprland.conf");
    fs::write(&source, "misc:font_family = Sans\n")?;
    let contents = fs::read_to_string(&source)?;
    let snapshot = snapshot_for(&source, &contents);
    let backup_manager = BackupManager::new(root.join("backups"));

    let outcome = apply_setting_change_with_backup_manager(
        known_ids(),
        &discovery_for(source.clone()),
        &snapshot,
        "misc.font_family",
        "JetBrains Mono",
        &backup_manager,
    )
    .map_err(|failure| anyhow::anyhow!("{failure:?}"))?;

    assert_eq!(outcome.setting_id, "misc.font_family");
    assert_eq!(outcome.target_path, source);
    assert!(outcome.backup_path.exists());
    assert_eq!(outcome.verified_value.as_deref(), Some("JetBrains Mono"));
    assert_eq!(
        fs::read_to_string(&outcome.target_path)?,
        "misc:font_family = JetBrains Mono\n"
    );

    fs::remove_dir_all(root)?;
    Ok(())
}

#[test]
fn apply_flow_writes_sanitized_path_fixture() -> Result<()> {
    let root = temp_root("apply-path")?;
    let source = root.join("hyprland.conf");
    fs::write(&source, "decoration:screen_shader = ./old.frag\n")?;
    let contents = fs::read_to_string(&source)?;
    let snapshot = snapshot_for(&source, &contents);
    let backup_manager = BackupManager::new(root.join("backups"));

    let outcome = apply_setting_change_with_backup_manager(
        known_ids(),
        &discovery_for(source.clone()),
        &snapshot,
        "decoration.screen_shader",
        "~/.config/hypr/example.frag",
        &backup_manager,
    )
    .map_err(|failure| anyhow::anyhow!("{failure:?}"))?;

    assert_eq!(outcome.setting_id, "decoration.screen_shader");
    assert_eq!(outcome.target_path, source);
    assert!(outcome.backup_path.exists());
    assert_eq!(
        outcome.verified_value.as_deref(),
        Some("~/.config/hypr/example.frag")
    );
    assert_eq!(
        fs::read_to_string(&outcome.target_path)?,
        "decoration:screen_shader = ~/.config/hypr/example.frag\n"
    );

    fs::remove_dir_all(root)?;
    Ok(())
}

#[test]
fn apply_flow_writes_regex_string_fixture() -> Result<()> {
    let root = temp_root("apply-regex")?;
    let source = root.join("hyprland.conf");
    fs::write(&source, "misc:swallow_regex = firefox\n")?;
    let contents = fs::read_to_string(&source)?;
    let snapshot = snapshot_for(&source, &contents);
    let backup_manager = BackupManager::new(root.join("backups"));

    let outcome = apply_setting_change_with_backup_manager(
        known_ids(),
        &discovery_for(source.clone()),
        &snapshot,
        "misc.swallow_regex",
        "^(Alacritty|kitty)$",
        &backup_manager,
    )
    .map_err(|failure| anyhow::anyhow!("{failure:?}"))?;

    assert_eq!(outcome.setting_id, "misc.swallow_regex");
    assert_eq!(outcome.target_path, source);
    assert!(outcome.backup_path.exists());
    assert_eq!(
        outcome.verified_value.as_deref(),
        Some("^(Alacritty|kitty)$")
    );
    assert_eq!(
        fs::read_to_string(&outcome.target_path)?,
        "misc:swallow_regex = ^(Alacritty|kitty)$\n"
    );

    fs::remove_dir_all(root)?;
    Ok(())
}

#[test]
fn apply_flow_blocks_missing_config_target() {
    let discovery = ConfigDiscovery {
        status: ConfigDiscoveryStatus::Missing,
        attempted_paths: Vec::new(),
    };
    let snapshot = CurrentConfigSnapshot::read_unavailable("missing");
    let backup_manager = BackupManager::new(std::env::temp_dir().join("unused"));

    let error = apply_setting_change_with_backup_manager(
        known_ids(),
        &discovery,
        &snapshot,
        ACTIVE_PENDING_CHANGE_SETTING,
        "true",
        &backup_manager,
    )
    .expect_err("missing config should block apply");

    assert!(error.reason.contains("no Hyprland config file"));
    assert!(error.failures.contains(&"MissingCurrentSource".to_string()));
}

#[test]
fn apply_flow_blocks_non_allowlisted_setting() {
    let discovery = ConfigDiscovery {
        status: ConfigDiscoveryStatus::Missing,
        attempted_paths: Vec::new(),
    };
    let snapshot = CurrentConfigSnapshot::read_unavailable("missing");
    let backup_manager = BackupManager::new(std::env::temp_dir().join("unused"));

    let error = apply_setting_change_with_backup_manager(
        known_ids(),
        &discovery,
        &snapshot,
        "appearance.glow.range",
        "false",
        &backup_manager,
    )
    .expect_err("non-allowlisted setting should block apply");

    assert_eq!(error.reason, "setting is not write-allowlisted");
    assert!(error.failures.contains(&"NotAllowlisted".to_string()));
}

#[test]
fn apply_flow_blocks_duplicate_before_backup_side_effect() -> Result<()> {
    let root = temp_root("duplicate-before-backup")?;
    let source = root.join("hyprland.conf");
    let backup_root = root.join("backups");
    fs::write(
        &source,
        "general:snap:enabled = false\ngeneral:snap:enabled = true\n",
    )?;
    let snapshot = snapshot_for(&source, &fs::read_to_string(&source)?);
    let backup_manager = BackupManager::new(&backup_root);

    let error = apply_setting_change_with_backup_manager(
        known_ids(),
        &discovery_for(source),
        &snapshot,
        ACTIVE_PENDING_CHANGE_SETTING,
        "false",
        &backup_manager,
    )
    .expect_err("duplicate conflict should block apply");

    assert!(error.failures.contains(&"DuplicateConflict".to_string()));
    assert!(!backup_root.exists());

    fs::remove_dir_all(root)?;
    Ok(())
}

#[test]
fn apply_flow_blocks_invalid_value_before_backup_side_effect() -> Result<()> {
    let root = temp_root("invalid-before-backup")?;
    let source = root.join("hyprland.conf");
    let backup_root = root.join("backups");
    fs::write(&source, "general:snap:enabled = false\n")?;
    let snapshot = snapshot_for(&source, &fs::read_to_string(&source)?);
    let backup_manager = BackupManager::new(&backup_root);

    let error = apply_setting_change_with_backup_manager(
        known_ids(),
        &discovery_for(source),
        &snapshot,
        ACTIVE_PENDING_CHANGE_SETTING,
        "maybe",
        &backup_manager,
    )
    .expect_err("invalid value should block apply");

    assert!(error.failures.contains(&"InvalidProposedValue".to_string()));
    assert!(!backup_root.exists());

    fs::remove_dir_all(root)?;
    Ok(())
}
