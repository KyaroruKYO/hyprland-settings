use std::path::Path;

use hyprland_settings::config_parser::parse_hyprland_config_text;
use hyprland_settings::current_config::{CurrentConfigSnapshot, CurrentValueProjection};
use hyprland_settings::pending_change::{
    stage_pending_change, stage_pending_change_with_sources, PendingChangeValidation,
    PendingChangeValueSources, ACTIVE_PENDING_CHANGE_SETTING,
};
use hyprland_settings::write_classification::{
    finite_choice_options, CONFLICT_FINITE_CHOICE_ROWS, MONITOR_OUTPUT_ROWS,
    REMAINING_105_FINITE_CHOICE_ROWS, SOURCE_BACKED_INPUT_ROWS,
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
    let current = current_value_for("xwayland.enabled", "xwayland:enabled = false\n");

    let change = stage_pending_change("xwayland.enabled", &current, "true");

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
fn semantic_master_center_fallback_accepts_only_official_values() {
    let current = current_value_for(
        "master.center_master_fallback",
        "master:center_master_fallback = left\n",
    );

    for value in ["left", "right", "top", "bottom"] {
        let change = stage_pending_change("master.center_master_fallback", &current, value);
        assert_eq!(
            change.validation,
            PendingChangeValidation::Valid,
            "center_master_fallback should accept {value}"
        );
        assert!(change.can_be_applied());
    }

    for value in ["", "center", "Left", "1", "example"] {
        let change = stage_pending_change("master.center_master_fallback", &current, value);
        assert!(
            matches!(change.validation, PendingChangeValidation::Invalid { .. }),
            "center_master_fallback should reject {value:?}"
        );
        assert!(!change.can_be_applied());
    }
}

#[test]
fn semantic_scrolling_explicit_column_widths_accepts_only_float_lists() {
    let current = current_value_for(
        "scrolling.explicit_column_widths",
        "scrolling:explicit_column_widths = 0.333, 0.5, 0.667, 1.0\n",
    );

    for value in ["1.0", "0.333, 0.5, 0.667, 1.0", "0.25,0.75"] {
        let change = stage_pending_change("scrolling.explicit_column_widths", &current, value);
        assert_eq!(
            change.validation,
            PendingChangeValidation::Valid,
            "explicit_column_widths should accept {value}"
        );
        assert!(change.can_be_applied());
    }

    for value in [
        "",
        "example",
        "0.5,",
        ",0.5",
        "0.5,,1.0",
        "NaN",
        "inf",
        "-inf",
        "0.5, bad",
        "0.5 0.75",
        "0.5\n1.0",
        "0.01",
        "1.5",
        "0.05,1.01",
    ] {
        let change = stage_pending_change("scrolling.explicit_column_widths", &current, value);
        assert!(
            matches!(change.validation, PendingChangeValidation::Invalid { .. }),
            "explicit_column_widths should reject {value:?}"
        );
        assert!(!change.can_be_applied());
    }
}

#[test]
fn source_backed_input_rows_can_be_staged_with_known_xkb_values() {
    let cases = [
        ("input.kb_model", "input.kb_model", "pc105"),
        ("input.kb_layout", "input.kb_layout", "us,de"),
        ("input.kb_variant", "input.kb_variant", "intl"),
        (
            "input.kb_options",
            "input.kb_options",
            "grp:alt_shift_toggle,ctrl:nocaps",
        ),
        ("input.kb_rules", "input.kb_rules", "evdev"),
    ];

    for (row_id, official_setting, proposed) in cases {
        let config_key = official_setting.replace('.', ":");
        let current = current_value_for(official_setting, &format!("{config_key} = {proposed}\n"));

        let change = stage_pending_change(row_id, &current, proposed);

        assert_eq!(
            change.validation,
            PendingChangeValidation::Valid,
            "{row_id} should accept source-backed value {proposed}"
        );
        assert!(change.can_be_applied(), "{row_id} should be applicable");
    }

    assert_eq!(SOURCE_BACKED_INPUT_ROWS.len(), 5);
}

#[test]
fn source_backed_input_rows_reject_unknown_or_multiline_values() {
    let current = CurrentValueProjection::not_configured();
    let cases = [
        ("input.kb_model", "__not_a_model__"),
        ("input.kb_layout", "__not_a_layout__"),
        ("input.kb_variant", "__not_a_variant__"),
        ("input.kb_options", "__not_an_option__"),
        ("input.kb_rules", "__not_rules__"),
        ("input.kb_layout", "us\nexec bad"),
    ];

    for (row_id, proposed) in cases {
        let change = stage_pending_change(row_id, &current, proposed);

        assert!(
            matches!(change.validation, PendingChangeValidation::Invalid { .. }),
            "{row_id} should reject {proposed:?}"
        );
        assert!(!change.can_be_applied(), "{row_id} should not apply");
    }
}

#[test]
fn monitor_output_rows_accept_empty_or_config_declared_monitor_names_only() {
    let current = CurrentValueProjection::not_configured();
    let sources = PendingChangeValueSources {
        monitor_names: vec!["DP-1".to_string(), "HDMI-A-1".to_string()],
    };

    for row_id in MONITOR_OUTPUT_ROWS {
        let empty = stage_pending_change(row_id, &current, "");
        assert_eq!(empty.validation, PendingChangeValidation::Valid);

        let known = stage_pending_change_with_sources(row_id, &current, "DP-1", &sources);
        assert_eq!(
            known.validation,
            PendingChangeValidation::Valid,
            "{row_id} should accept a monitor from current config"
        );

        let unknown =
            stage_pending_change_with_sources(row_id, &current, "__invalid_monitor__", &sources);
        assert!(
            matches!(unknown.validation, PendingChangeValidation::Invalid { .. }),
            "{row_id} should reject unknown monitor names"
        );

        let no_context = stage_pending_change(row_id, &current, "DP-1");
        assert!(
            matches!(
                no_context.validation,
                PendingChangeValidation::Invalid { .. }
            ),
            "{row_id} should not accept non-empty monitor names without source context"
        );

        let multiline = stage_pending_change_with_sources(row_id, &current, "DP-1\nexec", &sources);
        assert!(matches!(
            multiline.validation,
            PendingChangeValidation::Invalid { .. }
        ));
    }
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
        ("appearance.blur.brightness", "2.5"),
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
fn verified_finite_choice_rows_accept_only_verified_raw_values() {
    let current = CurrentValueProjection::not_configured();

    for row_id in CONFLICT_FINITE_CHOICE_ROWS
        .iter()
        .chain(REMAINING_105_FINITE_CHOICE_ROWS.iter())
    {
        let options = finite_choice_options(row_id).expect("conflict row should have choices");
        assert!(
            !options.is_empty(),
            "{row_id} should expose verified finite choices"
        );
        for option in options {
            let change = stage_pending_change(row_id, &current, option.raw_value);
            assert_eq!(
                change.validation,
                PendingChangeValidation::Valid,
                "{row_id} should accept verified raw value {} ({})",
                option.raw_value,
                option.label
            );
            assert!(change.can_be_applied(), "{row_id} should be applicable");
        }
    }
}

#[test]
fn verified_finite_choice_rows_reject_unverified_semantic_and_slider_values() {
    let current = CurrentValueProjection::not_configured();
    let cases = [
        ("general.resize_corner", "top_left"),
        ("input.focus_on_close", "cursor"),
        ("input.float_switch_override_focus", "Enabled"),
        ("input.off_window_axis_events", "warp"),
        ("input.emulate_discrete_scroll", "disable"),
        ("input.touchpad.drag_lock", "Sticky"),
        ("input.touchpad.drag_3fg", "3_finger"),
        ("input.virtualkeyboard.share_states", "enable"),
        ("group.drag_into_group", "enabled"),
        ("dwindle.force_split", "Left / Top"),
        ("dwindle.split_bias", "current"),
        ("scrolling.focus_fit_method", "fit"),
        ("scrolling.focus_fit_method", "2"),
        ("dwindle.split_bias", "99"),
        ("general.resize_corner", "-1"),
        ("layout.selection", "__invalid_choice__"),
        ("input.follow_mouse", "Full"),
        ("input.scroll_method", "__invalid_scroll__"),
        ("input.touchpad.tap_button_map", "Left / Right / Middle"),
        ("master.new_status", "__invalid_choice__"),
        ("master.new_on_active", "__invalid_choice__"),
        ("master.orientation", "__invalid_choice__"),
        ("scrolling.direction", "__invalid_choice__"),
    ];

    for (row_id, proposed) in cases {
        let change = stage_pending_change(row_id, &current, proposed);
        assert!(
            matches!(change.validation, PendingChangeValidation::Invalid { .. }),
            "{row_id} should reject unverified value {proposed}, got {:?}",
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
fn gradient_color_list_rows_can_be_staged_with_valid_values() {
    let cases = [
        ("general.col.inactive_border", "general.col.inactive_border"),
        ("general.col.active_border", "general.col.active_border"),
        ("general.col.nogroup_border", "general.col.nogroup_border"),
        (
            "general.col.nogroup_border_active",
            "general.col.nogroup_border_active",
        ),
        ("group.col.border_active", "group.col.border_active"),
        ("group.col.border_inactive", "group.col.border_inactive"),
        (
            "group.col.border_locked_inactive",
            "group.col.border_locked_inactive",
        ),
        (
            "group.col.border_locked_active",
            "group.col.border_locked_active",
        ),
        ("group.groupbar.col.active", "group.groupbar.col.active"),
        ("group.groupbar.col.inactive", "group.groupbar.col.inactive"),
        (
            "group.groupbar.col.locked_active",
            "group.groupbar.col.locked_active",
        ),
        (
            "group.groupbar.col.locked_inactive",
            "group.groupbar.col.locked_inactive",
        ),
    ];

    for (row_id, official_setting) in cases {
        let config_key = official_setting.replace('.', ":");
        let current = current_value_for(
            official_setting,
            &format!("{config_key} = rgba(000000ff)\n"),
        );

        let change = stage_pending_change(row_id, &current, "rgba(ffffffff) rgba(000000ff) 45deg");

        assert_eq!(
            change.validation,
            PendingChangeValidation::Valid,
            "{row_id} should accept a validated gradient/color-list value"
        );
        assert!(change.can_be_applied(), "{row_id} should be applicable");
    }
}

#[test]
fn gradient_color_list_rows_reject_invalid_values() {
    let current = CurrentValueProjection::not_configured();
    for proposed in [
        "",
        "45deg",
        "red",
        "rgba(ffffffff) red",
        "rgba(ffffffff) 45turn",
        "rgba(ffffffff) 45deg rgba(000000ff)",
        "rgba(ffffffff) # comment",
    ] {
        let change = stage_pending_change("general.col.active_border", &current, proposed);
        assert!(
            matches!(change.validation, PendingChangeValidation::Invalid { .. }),
            "general.col.active_border should reject {proposed:?}"
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

        let change = stage_pending_change(row_id, &current, "10 20");

        assert_eq!(
            change.validation,
            PendingChangeValidation::Valid,
            "{row_id} should accept a source-backed finite vec2 value"
        );
        assert!(change.can_be_applied(), "{row_id} should be applicable");
    }
}

#[test]
fn vector_tuple_rows_reject_invalid_values() {
    let current = CurrentValueProjection::not_configured();
    for proposed in ["10", "10 20 30", "10,20", "10,20,30", "nan 1"] {
        let change = stage_pending_change("decoration.shadow.offset", &current, proposed);
        assert!(
            matches!(change.validation, PendingChangeValidation::Invalid { .. }),
            "decoration.shadow.offset should reject {proposed}"
        );
        assert!(!change.can_be_applied());
    }
}

#[test]
fn enum_custom_string_rows_can_be_staged_with_line_safe_values() {
    let cases = [
        ("input.accel_profile", "input.accel_profile", "flat"),
        (
            "group.groupbar.font_family",
            "group.groupbar.font_family",
            "JetBrains Mono",
        ),
        ("misc.font_family", "misc.font_family", "Inter"),
        ("misc.splash_font_family", "misc.splash_font_family", "Sans"),
    ];

    for (row_id, official_setting, proposed) in cases {
        let config_key = official_setting.replace('.', ":");
        let current = current_value_for(official_setting, &format!("{config_key} = Sans\n"));

        let change = stage_pending_change(row_id, &current, proposed);

        assert_eq!(
            change.validation,
            PendingChangeValidation::Valid,
            "{row_id} should accept a line-safe string"
        );
        assert!(change.can_be_applied(), "{row_id} should be applicable");
    }
}

#[test]
fn enum_custom_string_rows_reject_config_breaking_values() {
    let current = CurrentValueProjection::not_configured();
    for proposed in ["", "value\nnext", "value # comment", "`cmd`", "$(cmd)"] {
        let change = stage_pending_change("misc.font_family", &current, proposed);
        assert!(
            matches!(change.validation, PendingChangeValidation::Invalid { .. }),
            "misc.font_family should reject {proposed:?}"
        );
        assert!(!change.can_be_applied());
    }
}

#[test]
fn sanitized_path_rows_can_be_staged_with_safe_paths() {
    let cases = [
        ("decoration.screen_shader", "decoration.screen_shader"),
        ("input.kb_file", "input.kb_file"),
    ];

    for (row_id, official_setting) in cases {
        let config_key = official_setting.replace('.', ":");
        let current = current_value_for(official_setting, &format!("{config_key} = ./old\n"));

        let change = stage_pending_change(row_id, &current, "~/.config/hypr/example.conf");

        assert_eq!(
            change.validation,
            PendingChangeValidation::Valid,
            "{row_id} should accept a sanitized path string"
        );
        assert!(change.can_be_applied(), "{row_id} should be applicable");
    }
}

#[test]
fn sanitized_path_rows_reject_command_like_values() {
    let current = CurrentValueProjection::not_configured();
    for proposed in ["", "a\nb", "a # b", "`cmd`", "$(cmd)", "a;b", "a|b"] {
        let change = stage_pending_change("decoration.screen_shader", &current, proposed);
        assert!(
            matches!(change.validation, PendingChangeValidation::Invalid { .. }),
            "decoration.screen_shader should reject {proposed:?}"
        );
        assert!(!change.can_be_applied());
    }
}

#[test]
fn regex_string_rows_can_be_staged_with_line_safe_patterns() {
    let cases = [
        ("misc.swallow_regex", "misc.swallow_regex"),
        (
            "misc.swallow_exception_regex",
            "misc.swallow_exception_regex",
        ),
    ];

    for (row_id, official_setting) in cases {
        let config_key = official_setting.replace('.', ":");
        let current = current_value_for(official_setting, &format!("{config_key} = firefox\n"));

        let change = stage_pending_change(row_id, &current, "^(Alacritty|kitty)$");

        assert_eq!(
            change.validation,
            PendingChangeValidation::Valid,
            "{row_id} should accept line-safe regex-like text"
        );
        assert!(change.can_be_applied(), "{row_id} should be applicable");
    }
}

#[test]
fn regex_string_rows_reject_config_breaking_values() {
    let current = CurrentValueProjection::not_configured();
    for proposed in ["", "a\nb", "a # b", "`cmd`", "$(cmd)"] {
        let change = stage_pending_change("misc.swallow_regex", &current, proposed);
        assert!(
            matches!(change.validation, PendingChangeValidation::Invalid { .. }),
            "misc.swallow_regex should reject {proposed:?}"
        );
        assert!(!change.can_be_applied());
    }
}

#[test]
fn numeric_list_row_can_be_staged_with_scroll_points() {
    let current = current_value_for("input.scroll_points", "input:scroll_points = 0.2 0.5 1\n");

    let change = stage_pending_change("input.scroll_points", &current, "0.2 0.0 0.5 1 1.2 1.5");

    assert_eq!(
        change.validation,
        PendingChangeValidation::Valid,
        "input.scroll_points should accept a step followed by finite points"
    );
    assert!(change.can_be_applied());
}

#[test]
fn numeric_list_row_rejects_invalid_scroll_points() {
    let current = CurrentValueProjection::not_configured();
    for proposed in ["", "0.2", "0 1", "0.2 nope", "0.2\n1", "0.2 # 1"] {
        let change = stage_pending_change("input.scroll_points", &current, proposed);
        assert!(
            matches!(change.validation, PendingChangeValidation::Invalid { .. }),
            "input.scroll_points should reject {proposed:?}"
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
