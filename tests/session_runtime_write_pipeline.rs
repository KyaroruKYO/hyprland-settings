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
use hyprland_settings::current_config::CurrentConfigSnapshot;
use hyprland_settings::pending_change::{stage_pending_change, PendingChangeValidation};
use hyprland_settings::write_classification::{
    finite_choice_options, is_high_risk_gated_writable_setting, is_safe_writable_setting,
    session_runtime_write_policy, ScalarWriteValueKind, SAFE_WRITABLE_ROWS,
    SESSION_RUNTIME_SENSITIVE_ROWS,
};
use hyprland_settings::write_flow::{
    apply_setting_change_with_backup_manager, edit_projection_for_setting, write_flow_value_kind,
};
use serde_json::Value;

const TARGETS: &[(&str, &str, &[&str], &[&str], &str)] = &[
    (
        "appearance.fullscreen_opacity",
        "decoration.fullscreen_opacity",
        &["1.0", "0", "1", "0.5"],
        &["not-a-number", "-1.0", "2.0"],
        "number",
    ),
    (
        "appearance.blur.xray",
        "decoration.blur.xray",
        &["true", "false"],
        &["maybe"],
        "toggle",
    ),
    (
        "general.allow_tearing",
        "general.allow_tearing",
        &["true", "false"],
        &["maybe"],
        "toggle",
    ),
    (
        "general.locale",
        "general.locale",
        &["", "example"],
        &["line\nbreak", "#comment", "$(`bad`)"],
        "text",
    ),
    (
        "misc.vrr",
        "misc.vrr",
        &["0", "1", "2", "3"],
        &["4", "not-a-valid-choice"],
        "dropdown",
    ),
    (
        "misc.mouse_move_enables_dpms",
        "misc.mouse_move_enables_dpms",
        &["true", "false"],
        &["maybe"],
        "toggle",
    ),
    (
        "misc.key_press_enables_dpms",
        "misc.key_press_enables_dpms",
        &["true", "false"],
        &["maybe"],
        "toggle",
    ),
    (
        "misc.disable_autoreload",
        "misc.disable_autoreload",
        &["true", "false"],
        &["maybe"],
        "toggle",
    ),
    (
        "misc.focus_on_activate",
        "misc.focus_on_activate",
        &["true", "false"],
        &["maybe"],
        "toggle",
    ),
    (
        "misc.allow_session_lock_restore",
        "misc.allow_session_lock_restore",
        &["true", "false"],
        &["maybe"],
        "toggle",
    ),
    (
        "misc.session_lock_xray",
        "misc.session_lock_xray",
        &["true", "false"],
        &["maybe"],
        "toggle",
    ),
    (
        "misc.on_focus_under_fullscreen",
        "misc.on_focus_under_fullscreen",
        &["0", "1", "2"],
        &["3", "not-a-valid-choice"],
        "dropdown",
    ),
    (
        "misc.exit_window_retains_fullscreen",
        "misc.exit_window_retains_fullscreen",
        &["true", "false"],
        &["maybe"],
        "toggle",
    ),
    (
        "binds.movefocus_cycles_fullscreen",
        "binds.movefocus_cycles_fullscreen",
        &["true", "false"],
        &["maybe"],
        "toggle",
    ),
    (
        "binds.allow_pin_fullscreen",
        "binds.allow_pin_fullscreen",
        &["true", "false"],
        &["maybe"],
        "toggle",
    ),
    (
        "scrolling.fullscreen_on_one_column",
        "scrolling.fullscreen_on_one_column",
        &["true", "false"],
        &["maybe"],
        "toggle",
    ),
];

fn report(path: &str) -> Result<Value> {
    Ok(serde_json::from_str(&fs::read_to_string(path)?)?)
}

fn current_value_for(
    official_setting: &str,
    config: &str,
) -> hyprland_settings::current_config::CurrentValueProjection {
    let parsed = parse_hyprland_config_text("/tmp/session-runtime.conf", config);
    CurrentConfigSnapshot::from_parsed(parsed).value_for(official_setting)
}

fn temp_root(name: &str) -> Result<PathBuf> {
    let stamp = SystemTime::now().duration_since(UNIX_EPOCH)?.as_nanos();
    let root = std::env::temp_dir().join(format!(
        "hyprland-settings-session-runtime-{name}-{}-{stamp}",
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

#[test]
fn session_runtime_reports_enable_only_the_16_target_rows() -> Result<()> {
    let pipeline = report("data/reports/session-runtime-write-pipeline.v0.55.2.json")?;
    let proof = report("data/reports/session-runtime-write-proof.v0.55.2.json")?;
    let policy = report("data/reports/session-runtime-write-policy-results.v0.55.2.json")?;
    let coverage = report("data/reports/scalar-read-write-coverage.v0.55.2.json")?;

    assert_eq!(pipeline["counts"]["rows"], 16);
    assert_eq!(pipeline["counts"]["enabledRows"], 16);
    assert_eq!(pipeline["counts"]["highRiskRowsIncluded"], 0);
    assert_eq!(pipeline["counts"]["finalWritableRows"], 269);
    assert_eq!(pipeline["counts"]["finalBlockedRows"], 72);

    assert_eq!(proof["counts"]["validatorPassed"], 16);
    assert_eq!(proof["counts"]["invalidRejectionPassed"], 16);
    assert_eq!(proof["counts"]["fixtureReplacePassed"], 16);
    assert_eq!(proof["counts"]["fixtureAppendPassed"], 16);
    assert_eq!(proof["counts"]["hyprlandVerifyConfigPassed"], 16);
    assert_eq!(proof["counts"]["activeConfigModified"], false);
    assert_eq!(proof["counts"]["activeRuntimeModified"], false);
    assert_eq!(proof["counts"]["reloadRun"], false);

    assert_eq!(policy["counts"]["persistentConfigOnlyRows"], 13);
    assert_eq!(policy["counts"]["persistentNeedsReloadRows"], 2);
    assert_eq!(policy["counts"]["startupOnlyRows"], 1);

    let coverage_rows = coverage["rows"].as_array().unwrap();
    for row_id in SESSION_RUNTIME_SENSITIVE_ROWS {
        let row = coverage_rows
            .iter()
            .find(|row| row["rowId"].as_str() == Some(row_id))
            .unwrap_or_else(|| panic!("missing coverage row {row_id}"));
        assert_eq!(row["writeStatus"].as_str(), Some("writable"), "{row_id}");
        assert_eq!(row["safeWriteSupported"].as_bool(), Some(true), "{row_id}");
        assert!(is_safe_writable_setting(row_id), "{row_id}");
    }

    assert_eq!(SAFE_WRITABLE_ROWS.len(), 340);
    assert!(is_safe_writable_setting("xwayland.enabled"));
    assert!(is_high_risk_gated_writable_setting("xwayland.enabled"));

    Ok(())
}

#[test]
fn session_runtime_validators_accept_valid_values_and_reject_invalid_values() {
    for (row_id, official_setting, valid_values, invalid_values, _) in TARGETS {
        let current = current_value_for(
            official_setting,
            &format!(
                "{} = {}\n",
                official_setting.replace('.', ":"),
                valid_values[0]
            ),
        );

        for value in *valid_values {
            let change = stage_pending_change(row_id, &current, *value);
            assert_eq!(
                change.validation,
                PendingChangeValidation::Valid,
                "{row_id} should accept {value:?}"
            );
        }

        for value in *invalid_values {
            let change = stage_pending_change(row_id, &current, *value);
            assert!(
                matches!(change.validation, PendingChangeValidation::Invalid { .. }),
                "{row_id} should reject {value:?}"
            );
        }
    }
}

#[test]
fn session_runtime_rows_project_editors_and_review_warnings() {
    for (row_id, official_setting, valid_values, _, expected_editor) in TARGETS {
        let current = current_value_for(
            official_setting,
            &format!(
                "{} = {}\n",
                official_setting.replace('.', ":"),
                valid_values[0]
            ),
        );
        let projection = edit_projection_for_setting(row_id, &current);
        let policy = session_runtime_write_policy(row_id).expect("policy should exist");

        assert!(projection.editable, "{row_id}");
        assert_eq!(projection.editor_kind, *expected_editor, "{row_id}");
        let pending = projection.pending.as_ref().expect("pending projection");
        assert_eq!(pending.validation_label, "valid", "{row_id}");
        assert!(
            pending
                .review_summary
                .iter()
                .any(|line| line.contains(policy.scope)),
            "{row_id}"
        );
        assert!(
            pending
                .review_summary
                .iter()
                .any(|line| line.contains(policy.runtime_effect)),
            "{row_id}"
        );
        assert!(
            pending
                .review_summary
                .iter()
                .any(|line| line.contains("Hyprland reload is not run")),
            "{row_id}"
        );

        if write_flow_value_kind(row_id) == Some(ScalarWriteValueKind::FiniteChoice) {
            let expected = finite_choice_options(row_id).expect("finite choices should exist");
            assert_eq!(projection.choices.len(), expected.len(), "{row_id}");
            for option in expected {
                assert!(
                    projection
                        .choices
                        .iter()
                        .any(|choice| choice.raw_value == option.raw_value
                            && choice.label == option.label),
                    "{row_id} should expose {}",
                    option.raw_value
                );
            }
        }
    }
}

#[test]
fn session_runtime_rows_write_fixture_with_existing_backup_reread_flow() -> Result<()> {
    for (row_id, official_setting, valid_values, _, _) in TARGETS {
        let root = temp_root(row_id.replace('.', "-").as_str())?;
        let source = root.join("hyprland.conf");
        let key = official_setting.replace('.', ":");
        fs::write(&source, format!("{key} = {}\n", valid_values[0]))?;
        let contents = fs::read_to_string(&source)?;
        let snapshot =
            CurrentConfigSnapshot::from_parsed(parse_hyprland_config_text(&source, &contents));
        let backup_manager = BackupManager::new(root.join("backups"));
        let proposed = valid_values.get(1).copied().unwrap_or(valid_values[0]);

        let outcome = apply_setting_change_with_backup_manager(
            known_ids(),
            &discovery_for(source.clone()),
            &snapshot,
            row_id,
            proposed,
            &backup_manager,
        )
        .map_err(|failure| anyhow::anyhow!("{failure:?}"))?;

        assert_eq!(outcome.setting_id, *row_id);
        assert_eq!(outcome.target_path, source);
        assert!(outcome.backup_path.exists());
        assert_eq!(outcome.verified_value.as_deref(), Some(proposed));
        assert!(outcome.reload_note.contains("Hyprland reload is not run"));
        assert_eq!(
            fs::read_to_string(&outcome.target_path)?,
            format!("{key} = {proposed}\n")
        );

        fs::remove_dir_all(root)?;
    }

    Ok(())
}
