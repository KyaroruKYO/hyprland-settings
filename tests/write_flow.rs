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
use hyprland_settings::write_classification::{
    finite_choice_options, CONFLICT_FINITE_CHOICE_ROWS,
    CURSOR_HIDE_ON_KEY_PRESS_HIGH_RISK_WRITABLE_ROWS, CURSOR_THEME_SYNC_HIGH_RISK_WRITABLE_ROWS,
    CURSOR_VISIBILITY_CONDITIONAL_HIGH_RISK_WRITABLE_ROWS, ECOSYSTEM_HIGH_RISK_WRITABLE_ROWS,
    MONITOR_OUTPUT_ROWS, REMAINING_105_FINITE_CHOICE_ROWS, SAFE_WRITABLE_ROWS,
    SOURCE_BACKED_INPUT_ROWS, XWAYLAND_SCALING_HIGH_RISK_WRITABLE_ROWS,
};
use hyprland_settings::write_flow::{
    apply_setting_change_with_backup_manager, edit_projection_for_setting,
    edit_projection_for_setting_with_config, pending_projection_for_value,
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
    let blocked = edit_projection_for_setting("cursor.default_monitor", &current);

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
fn ecosystem_high_risk_rows_project_dead_man_review_warning() {
    for row_id in ECOSYSTEM_HIGH_RISK_WRITABLE_ROWS {
        let path = PathBuf::from(format!("/tmp/{row_id}.conf"));
        let snapshot = snapshot_for(&path, &format!("{} = false\n", row_id.replace('.', ":")));
        let current = snapshot.value_for(row_id);
        let projection = pending_projection_for_value(row_id, &current, "true");
        let summary = projection.review_summary.join("\n");
        assert!(projection.can_review);
        assert!(
            summary.contains("ecosystem-permission-policy"),
            "{row_id} should show the ecosystem recovery bucket"
        );
        assert!(
            summary.contains("dead-man"),
            "{row_id} should show the dead-man approval gate"
        );
        assert!(
            summary.contains("watchdog"),
            "{row_id} should show the watchdog requirement"
        );
    }
}

#[test]
fn xwayland_scaling_rows_project_display_render_watchdog_warning() {
    for row_id in XWAYLAND_SCALING_HIGH_RISK_WRITABLE_ROWS {
        let path = PathBuf::from(format!("/tmp/{row_id}.conf"));
        let snapshot = snapshot_for(&path, &format!("{} = false\n", row_id.replace('.', ":")));
        let current = snapshot.value_for(row_id);
        let projection = pending_projection_for_value(row_id, &current, "true");
        let summary = projection.review_summary.join("\n");
        assert!(projection.can_review);
        assert!(
            summary.contains("display-render-recovery:xwayland-scaling-policy-smoke-subset"),
            "{row_id} should show the display/render smoke subset recovery bucket"
        );
        assert!(
            summary.contains("dead-man"),
            "{row_id} should show the dead-man approval gate"
        );
        assert!(
            summary.contains("watchdog"),
            "{row_id} should show the watchdog requirement"
        );
    }
}

#[test]
fn cursor_theme_sync_row_projects_cursor_input_watchdog_warning() {
    for row_id in CURSOR_THEME_SYNC_HIGH_RISK_WRITABLE_ROWS {
        let path = PathBuf::from(format!("/tmp/{row_id}.conf"));
        let snapshot = snapshot_for(&path, &format!("{} = false\n", row_id.replace('.', ":")));
        let current = snapshot.value_for(row_id);
        let projection = pending_projection_for_value(row_id, &current, "true");
        let summary = projection.review_summary.join("\n");
        assert!(projection.can_review);
        assert!(
            summary.contains("cursor-input-recovery:cursor-theme-sync-policy-smoke-subset"),
            "{row_id} should show the cursor/input smoke subset recovery bucket"
        );
        assert!(
            summary.contains("dead-man"),
            "{row_id} should show the dead-man approval gate"
        );
        assert!(
            summary.contains("watchdog"),
            "{row_id} should show the watchdog requirement"
        );
        assert!(
            summary.contains("mouse input"),
            "{row_id} should show mouse-independent recovery requirements"
        );
    }
}

#[test]
fn cursor_visibility_conditional_rows_project_stronger_watchdog_warning() {
    for row_id in CURSOR_VISIBILITY_CONDITIONAL_HIGH_RISK_WRITABLE_ROWS {
        let path = PathBuf::from(format!("/tmp/{row_id}.conf"));
        let snapshot = snapshot_for(&path, &format!("{} = false\n", row_id.replace('.', ":")));
        let current = snapshot.value_for(row_id);
        let projection = pending_projection_for_value(row_id, &current, "true");
        let summary = projection.review_summary.join("\n");
        assert!(projection.can_review);
        assert!(
            summary.contains(
                "cursor-input-recovery:cursor-visibility-conditional-touch-tablet-subset"
            ),
            "{row_id} should show the cursor visibility recovery bucket"
        );
        assert!(
            summary.contains("dead-man"),
            "{row_id} should show the dead-man approval gate"
        );
        assert!(
            summary.contains("watchdog"),
            "{row_id} should show the watchdog requirement"
        );
        assert!(
            summary.contains("Cursor may disappear"),
            "{row_id} should show the stronger cursor visibility warning"
        );
        assert!(
            summary.contains("visible cursor"),
            "{row_id} should show visible-cursor-independent recovery requirements"
        );
        assert!(
            summary.contains("mouse input"),
            "{row_id} should show mouse-independent recovery requirements"
        );
    }
}

#[test]
fn cursor_hide_on_key_press_projects_keyboard_token_watchdog_warning() {
    for row_id in CURSOR_HIDE_ON_KEY_PRESS_HIGH_RISK_WRITABLE_ROWS {
        let path = PathBuf::from(format!("/tmp/{row_id}.conf"));
        let snapshot = snapshot_for(&path, &format!("{} = false\n", row_id.replace('.', ":")));
        let current = snapshot.value_for(row_id);
        let projection = pending_projection_for_value(row_id, &current, "true");
        let summary = projection.review_summary.join("\n");
        assert!(projection.can_review);
        assert!(
            summary
                .contains("cursor-input-recovery:cursor-hide-on-key-press-keyboard-token-subset"),
            "{row_id} should show the keyboard-token recovery bucket"
        );
        assert!(
            summary.contains("dead-man"),
            "{row_id} should show the dead-man approval gate"
        );
        assert!(
            summary.contains("watchdog"),
            "{row_id} should show the watchdog requirement"
        );
        assert!(
            summary.contains("Cursor may disappear while typing"),
            "{row_id} should show the keyboard-trigger cursor warning"
        );
        assert!(
            summary.contains("CLI token"),
            "{row_id} should show CLI-token confirmation"
        );
        for required in [
            "visible cursor",
            "mouse input",
            "Hyprland keybinds",
            "pointer focus",
            "workspace focus",
        ] {
            assert!(
                summary.contains(required),
                "{row_id} should show recovery independence from {required}"
            );
        }
    }
}

#[test]
fn source_backed_input_rows_project_as_dropdowns() {
    let current = CurrentConfigSnapshot::read_unavailable("no config").value_for("input.kb_layout");

    for row_id in SOURCE_BACKED_INPUT_ROWS {
        let projection = edit_projection_for_setting(row_id, &current);

        assert!(projection.editable, "{row_id} should be editable");
        assert_eq!(projection.editor_kind, "dropdown", "{row_id}");
        assert!(
            !projection.choices.is_empty(),
            "{row_id} should expose source-backed choices"
        );
        assert!(
            projection
                .proposed_value
                .as_deref()
                .is_some_and(|value| !value.is_empty()),
            "{row_id} should propose a source-backed value"
        );
        assert_eq!(
            projection
                .pending
                .as_ref()
                .expect("pending projection")
                .validation_label,
            "valid",
            "{row_id} should stage a known source-backed value"
        );
    }
}

#[test]
fn monitor_output_rows_project_configured_monitors_as_dropdowns() {
    let source = PathBuf::from("/tmp/monitor-output-ui.conf");
    let snapshot = snapshot_for(
        &source,
        "monitor = DP-1, preferred, auto, 1\nmonitor = HDMI-A-1, preferred, auto, 1\n",
    );
    let current = snapshot.value_for("input.touchdevice.output");

    for row_id in MONITOR_OUTPUT_ROWS {
        let projection = edit_projection_for_setting_with_config(row_id, &current, &snapshot);

        assert!(projection.editable, "{row_id} should be editable");
        assert_eq!(projection.editor_kind, "dropdown");
        assert_eq!(projection.choices[0].raw_value, "");
        assert_eq!(projection.choices[0].label, "Default / auto");
        assert!(projection
            .choices
            .iter()
            .any(|choice| choice.raw_value == "DP-1"));
        assert_eq!(projection.proposed_value.as_deref(), Some("DP-1"));
        assert_eq!(
            projection.pending.as_ref().unwrap().validation_label,
            "valid"
        );
    }
}

#[test]
fn conflict_rows_project_as_verified_finite_choice_dropdowns() {
    let current = CurrentConfigSnapshot::read_unavailable("no config").value_for("general.gaps_in");

    for row_id in CONFLICT_FINITE_CHOICE_ROWS
        .iter()
        .chain(REMAINING_105_FINITE_CHOICE_ROWS.iter())
    {
        let projection = edit_projection_for_setting(row_id, &current);
        let expected = finite_choice_options(row_id).expect("conflict row should have choices");

        assert!(projection.editable, "{row_id} should remain editable");
        assert_eq!(projection.editor_kind, "dropdown", "{row_id}");
        assert_eq!(projection.choices.len(), expected.len(), "{row_id}");
        assert_eq!(
            projection.proposed_value.as_deref(),
            expected.first().map(|option| option.raw_value),
            "{row_id} should propose a verified raw value when unset"
        );
        for (actual, expected) in projection.choices.iter().zip(expected.iter()) {
            assert_eq!(actual.raw_value, expected.raw_value, "{row_id}");
            assert_eq!(actual.label, expected.label, "{row_id}");
        }
        assert!(
            projection
                .pending
                .as_ref()
                .expect("pending projection")
                .validation_label
                == "valid",
            "{row_id} should stage a verified dropdown value"
        );
    }
}

#[test]
fn finite_choice_projection_advances_from_current_raw_value() {
    let parsed = parse_hyprland_config_text(
        "/tmp/hyprland.conf",
        "dwindle:split_bias = 0\nscrolling:focus_fit_method = 1\n",
    );
    let snapshot = CurrentConfigSnapshot::from_parsed(parsed);

    let split_bias = edit_projection_for_setting(
        "dwindle.split_bias",
        &snapshot.value_for("dwindle.split_bias"),
    );
    assert_eq!(split_bias.editor_kind, "dropdown");
    assert_eq!(split_bias.proposed_value.as_deref(), Some("1"));

    let focus_fit = edit_projection_for_setting(
        "scrolling.focus_fit_method",
        &snapshot.value_for("scrolling.focus_fit_method"),
    );
    assert_eq!(focus_fit.editor_kind, "dropdown");
    assert_eq!(focus_fit.proposed_value.as_deref(), Some("0"));
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
fn apply_flow_writes_monitor_output_from_declared_monitor_fixture() -> Result<()> {
    let root = temp_root("monitor-output")?;
    let source = root.join("hyprland.conf");
    fs::write(
        &source,
        "monitor = DP-1, preferred, auto, 1\ninput:touchdevice:output = \n",
    )?;
    let contents = fs::read_to_string(&source)?;
    let snapshot = snapshot_for(&source, &contents);
    let backup_manager = BackupManager::new(root.join("backups"));

    let outcome = apply_setting_change_with_backup_manager(
        known_ids(),
        &discovery_for(source.clone()),
        &snapshot,
        "input.touchdevice.output",
        "DP-1",
        &backup_manager,
    )
    .map_err(|failure| anyhow::anyhow!("{failure:?}"))?;

    assert_eq!(outcome.setting_id, "input.touchdevice.output");
    assert_eq!(outcome.verified_value.as_deref(), Some("DP-1"));
    assert_eq!(
        fs::read_to_string(&outcome.target_path)?,
        "monitor = DP-1, preferred, auto, 1\ninput:touchdevice:output = DP-1\n"
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
        "10 20",
        &backup_manager,
    )
    .map_err(|failure| anyhow::anyhow!("{failure:?}"))?;

    assert_eq!(outcome.setting_id, "decoration.shadow.offset");
    assert_eq!(outcome.target_path, source);
    assert!(outcome.backup_path.exists());
    assert_eq!(outcome.verified_value.as_deref(), Some("10 20"));
    assert_eq!(
        fs::read_to_string(&outcome.target_path)?,
        "decoration:shadow:offset = 10 20\n"
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
fn semantic_master_center_fallback_projects_and_writes_dropdown_fixture() -> Result<()> {
    let source = PathBuf::from("/tmp/master-center-ui.conf");
    let snapshot = snapshot_for(&source, "master:center_master_fallback = left\n");
    let projection = edit_projection_for_setting(
        "master.center_master_fallback",
        &snapshot.value_for("master.center_master_fallback"),
    );

    assert!(projection.editable);
    assert_eq!(projection.editor_kind, "dropdown");
    assert_eq!(projection.choices.len(), 4);
    assert!(projection
        .choices
        .iter()
        .any(|choice| { choice.raw_value == "bottom" && choice.label == "Bottom" }));
    assert_eq!(projection.proposed_value.as_deref(), Some("right"));

    let root = temp_root("apply-master-center-fallback")?;
    let source = root.join("hyprland.conf");
    fs::write(&source, "master:center_master_fallback = left\n")?;
    let contents = fs::read_to_string(&source)?;
    let snapshot = snapshot_for(&source, &contents);
    let backup_manager = BackupManager::new(root.join("backups"));

    let outcome = apply_setting_change_with_backup_manager(
        known_ids(),
        &discovery_for(source.clone()),
        &snapshot,
        "master.center_master_fallback",
        "bottom",
        &backup_manager,
    )
    .map_err(|failure| anyhow::anyhow!("{failure:?}"))?;

    assert_eq!(outcome.setting_id, "master.center_master_fallback");
    assert_eq!(outcome.target_path, source);
    assert!(outcome.backup_path.exists());
    assert_eq!(outcome.verified_value.as_deref(), Some("bottom"));
    assert_eq!(
        fs::read_to_string(&outcome.target_path)?,
        "master:center_master_fallback = bottom\n"
    );

    fs::remove_dir_all(root)?;
    Ok(())
}

#[test]
fn semantic_scrolling_explicit_column_widths_projects_and_writes_fixture() -> Result<()> {
    let source = PathBuf::from("/tmp/scrolling-widths-ui.conf");
    let snapshot = snapshot_for(
        &source,
        "scrolling:explicit_column_widths = 0.333, 0.5, 0.667, 1.0\n",
    );
    let projection = edit_projection_for_setting(
        "scrolling.explicit_column_widths",
        &snapshot.value_for("scrolling.explicit_column_widths"),
    );

    assert!(projection.editable);
    assert_eq!(projection.editor_kind, "comma-float-list-text");
    assert_eq!(
        projection.proposed_value.as_deref(),
        Some("0.333, 0.5, 0.667, 1.0")
    );

    let root = temp_root("apply-explicit-column-widths")?;
    let source = root.join("hyprland.conf");
    fs::write(
        &source,
        "scrolling:explicit_column_widths = 0.333, 0.5, 0.667, 1.0\n",
    )?;
    let contents = fs::read_to_string(&source)?;
    let snapshot = snapshot_for(&source, &contents);
    let backup_manager = BackupManager::new(root.join("backups"));

    let outcome = apply_setting_change_with_backup_manager(
        known_ids(),
        &discovery_for(source.clone()),
        &snapshot,
        "scrolling.explicit_column_widths",
        "0.25,0.75",
        &backup_manager,
    )
    .map_err(|failure| anyhow::anyhow!("{failure:?}"))?;

    assert_eq!(outcome.setting_id, "scrolling.explicit_column_widths");
    assert_eq!(outcome.target_path, source);
    assert!(outcome.backup_path.exists());
    assert_eq!(outcome.verified_value.as_deref(), Some("0.25,0.75"));
    assert_eq!(
        fs::read_to_string(&outcome.target_path)?,
        "scrolling:explicit_column_widths = 0.25,0.75\n"
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
    fs::write(&source, "input:kb_file = ./old.xkb\n")?;
    let contents = fs::read_to_string(&source)?;
    let snapshot = snapshot_for(&source, &contents);
    let backup_manager = BackupManager::new(root.join("backups"));

    let outcome = apply_setting_change_with_backup_manager(
        known_ids(),
        &discovery_for(source.clone()),
        &snapshot,
        "input.kb_file",
        "~/.config/hypr/example.xkb",
        &backup_manager,
    )
    .map_err(|failure| anyhow::anyhow!("{failure:?}"))?;

    assert_eq!(outcome.setting_id, "input.kb_file");
    assert_eq!(outcome.target_path, source);
    assert!(outcome.backup_path.exists());
    assert_eq!(
        outcome.verified_value.as_deref(),
        Some("~/.config/hypr/example.xkb")
    );
    assert_eq!(
        fs::read_to_string(&outcome.target_path)?,
        "input:kb_file = ~/.config/hypr/example.xkb\n"
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
        "cursor.default_monitor",
        "HDMI-A-1",
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
