use std::path::Path;

use hyprland_settings::config_parser::parse_hyprland_config_text;
use hyprland_settings::current_config::{CurrentConfigSnapshot, CurrentValueProjection};
use hyprland_settings::pending_change::{
    stage_pending_change, PendingChangeValidation, ACTIVE_PENDING_CHANGE_SETTING,
};

fn current_value_for(setting_id: &str, config: &str) -> CurrentValueProjection {
    let parsed = parse_hyprland_config_text("/tmp/pending-change.conf", config);
    CurrentConfigSnapshot::from_parsed(parsed).value_for(setting_id)
}

#[test]
fn valid_pending_change_for_windows_snap_enabled_can_be_staged() {
    let current = current_value_for("general.snap.enabled", "general:snap:enabled = false\n");

    let change = stage_pending_change(ACTIVE_PENDING_CHANGE_SETTING, &current, "true");

    assert_eq!(change.validation, PendingChangeValidation::Valid);
    assert!(change.can_be_applied());
    assert_eq!(change.old_parsed_value.as_deref(), Some("false"));
    assert_eq!(change.proposed_value, "true");
    assert_eq!(
        change.source.as_ref().map(|source| source.line_number),
        Some(1)
    );
}

#[test]
fn invalid_pending_change_for_windows_snap_enabled_is_rejected() {
    let current = current_value_for("general.snap.enabled", "general:snap:enabled = false\n");

    let change = stage_pending_change(ACTIVE_PENDING_CHANGE_SETTING, &current, "maybe");

    assert_eq!(
        change.validation,
        PendingChangeValidation::Invalid {
            reason: "safe scalar toggle writes require a boolean value".to_string(),
        }
    );
    assert!(!change.can_be_applied());
}

#[test]
fn only_safe_writable_rows_can_be_staged() {
    let current = current_value_for("decoration.glow.range", "decoration:glow:range = 8\n");

    let change = stage_pending_change("appearance.glow.range", &current, "10");

    assert_eq!(
        change.validation,
        PendingChangeValidation::NotAllowed {
            reason: "setting is not pending-change allowlisted".to_string(),
        }
    );
    assert_eq!(
        change.non_editable_reason.as_deref(),
        Some("setting is not in the safe scalar write allowlist")
    );
}

#[test]
fn validator_needed_rows_can_be_staged_with_valid_values() {
    let cases = [
        ("appearance.blur.size", "decoration.blur.size", "10"),
        (
            "appearance.blur.brightness",
            "decoration.blur.brightness",
            "0.75",
        ),
        (
            "appearance.blur.contrast",
            "decoration.blur.contrast",
            "0.75",
        ),
        ("appearance.shadow.range", "decoration.shadow.range", "10"),
        (
            "appearance.shadow.render_power",
            "decoration.shadow.render_power",
            "3",
        ),
        ("appearance.gaps_in", "general.gaps_in", "8"),
        ("appearance.gaps_out", "general.gaps_out", "12"),
        ("appearance.border_size", "general.border_size", "2"),
        ("appearance.rounding", "decoration.rounding", "8"),
        (
            "appearance.active_opacity",
            "decoration.active_opacity",
            "0.9",
        ),
        (
            "appearance.inactive_opacity",
            "decoration.inactive_opacity",
            "0.8",
        ),
        ("windows.snap.window_gap", "general.snap.window_gap", "5"),
        ("windows.snap.monitor_gap", "general.snap.monitor_gap", "5"),
        ("input.pointer_sensitivity", "input.sensitivity", "-0.25"),
    ];

    for (row_id, official_setting, proposed) in cases {
        let config_key = official_setting.replace('.', ":");
        let current = current_value_for(official_setting, &format!("{config_key} = 0\n"));

        let change = stage_pending_change(row_id, &current, proposed);

        assert_eq!(
            change.validation,
            PendingChangeValidation::Valid,
            "{row_id} should accept {proposed}"
        );
        assert!(change.can_be_applied(), "{row_id} should be applicable");
    }
}

#[test]
fn validator_needed_rows_reject_invalid_values() {
    let cases = [
        ("appearance.blur.size", "-1"),
        ("appearance.blur.brightness", "1.5"),
        ("appearance.blur.contrast", "not-a-number"),
        ("appearance.shadow.range", "-2"),
        ("appearance.shadow.render_power", "2.5"),
        ("appearance.gaps_in", "-8"),
        ("appearance.gaps_out", "8 px"),
        ("appearance.border_size", "wide"),
        ("appearance.rounding", "-1"),
        ("appearance.active_opacity", "-0.1"),
        ("appearance.inactive_opacity", "2"),
        ("windows.snap.window_gap", "-5"),
        ("windows.snap.monitor_gap", "5 px"),
        ("input.pointer_sensitivity", "2"),
    ];

    let current = CurrentValueProjection::not_configured();
    for (row_id, proposed) in cases {
        let change = stage_pending_change(row_id, &current, proposed);
        assert!(
            matches!(change.validation, PendingChangeValidation::Invalid { .. }),
            "{row_id} should reject {proposed}, got {:?}",
            change.validation
        );
        assert!(!change.can_be_applied(), "{row_id} should not apply");
    }
}

#[test]
fn parser_backed_color_rows_can_be_staged_with_valid_literals() {
    let cases = [
        ("decoration.shadow.color", "decoration.shadow.color"),
        (
            "decoration.shadow.color_inactive",
            "decoration.shadow.color_inactive",
        ),
        ("decoration.glow.color", "decoration.glow.color"),
        (
            "decoration.glow.color_inactive",
            "decoration.glow.color_inactive",
        ),
        ("group.groupbar.text_color", "group.groupbar.text_color"),
        (
            "group.groupbar.text_color_inactive",
            "group.groupbar.text_color_inactive",
        ),
        (
            "group.groupbar.text_color_locked_active",
            "group.groupbar.text_color_locked_active",
        ),
        (
            "group.groupbar.text_color_locked_inactive",
            "group.groupbar.text_color_locked_inactive",
        ),
        ("misc.col.splash", "misc.col.splash"),
        ("misc.background_color", "misc.background_color"),
    ];

    for (row_id, official_setting) in cases {
        let config_key = official_setting.replace('.', ":");
        let current = current_value_for(
            official_setting,
            &format!("{config_key} = rgba(000000ff)\n"),
        );

        let change = stage_pending_change(row_id, &current, "rgba(ffffffff)");

        assert_eq!(
            change.validation,
            PendingChangeValidation::Valid,
            "{row_id} should accept a strict rgba literal"
        );
        assert!(change.can_be_applied(), "{row_id} should be applicable");
    }
}

#[test]
fn parser_backed_color_rows_reject_invalid_literals() {
    let current = CurrentValueProjection::not_configured();
    for proposed in ["red", "rgb(ffff)", "rgba(fffffffff)", "0xzzzzzzzz"] {
        let change = stage_pending_change("misc.background_color", &current, proposed);
        assert!(
            matches!(change.validation, PendingChangeValidation::Invalid { .. }),
            "misc.background_color should reject {proposed}"
        );
        assert!(!change.can_be_applied());
    }
}

#[test]
fn vector_tuple_rows_can_be_staged_with_valid_values() {
    let cases = [
        ("decoration.shadow.offset", "decoration.shadow.offset"),
        (
            "input.tablet.region_position",
            "input.tablet.region_position",
        ),
        ("input.tablet.region_size", "input.tablet.region_size"),
        (
            "input.tablet.active_area_size",
            "input.tablet.active_area_size",
        ),
        (
            "input.tablet.active_area_position",
            "input.tablet.active_area_position",
        ),
        (
            "layout.single_window_aspect_ratio",
            "layout.single_window_aspect_ratio",
        ),
    ];

    for (row_id, official_setting) in cases {
        let config_key = official_setting.replace('.', ":");
        let current = current_value_for(official_setting, &format!("{config_key} = 0 0\n"));

        let change = stage_pending_change(row_id, &current, "10,20");

        assert_eq!(
            change.validation,
            PendingChangeValidation::Valid,
            "{row_id} should accept a finite vec2 value"
        );
        assert!(change.can_be_applied(), "{row_id} should be applicable");
    }
}

#[test]
fn vector_tuple_rows_reject_invalid_values() {
    let current = CurrentValueProjection::not_configured();
    for proposed in ["10", "10 20 30", "10,20,30", "nan 1"] {
        let change = stage_pending_change("decoration.shadow.offset", &current, proposed);
        assert!(
            matches!(change.validation, PendingChangeValidation::Invalid { .. }),
            "decoration.shadow.offset should reject {proposed}"
        );
        assert!(!change.can_be_applied());
    }
}

#[test]
fn pending_change_preserves_missing_old_value_for_absent_setting() {
    let current = CurrentValueProjection::not_configured();

    let change = stage_pending_change(ACTIVE_PENDING_CHANGE_SETTING, &current, "false");

    assert_eq!(change.validation, PendingChangeValidation::Valid);
    assert_eq!(change.old_parsed_value, None);
    assert_eq!(change.source, None);
}

#[test]
fn no_write_executor_exists_in_rust_project() {
    let root = Path::new(env!("CARGO_MANIFEST_DIR"));

    assert!(!root.join("src/write_executor.rs").exists());
    assert!(!root.join("src/write_runtime_executor.rs").exists());
    assert!(!root.join("writeRuntimeExecutor.ts").exists());
}
