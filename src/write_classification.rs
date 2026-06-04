use crate::export::InventoryEntry;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ScalarWriteClassification {
    pub row_id: String,
    pub official_setting: String,
    pub config_key: String,
    pub status: ScalarWriteStatus,
    pub blocker: Option<String>,
    pub value_kind: ScalarWriteValueKind,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ScalarWriteStatus {
    SafeWritable,
    BlockedHighRisk,
    BlockedParserNeeded,
    BlockedValidatorNeeded,
    BlockedStructured,
    BlockedAmbiguousKey,
    BlockedSpecialSemantics,
    BlockedManualReview,
    Unsupported,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ScalarWriteValueKind {
    Boolean,
    Number,
    Percent,
    Color,
    Gradient,
    Vector2,
    NumericList,
    LineSafeString,
    Path,
    RegexString,
    StringLike,
    ComplexRaw,
    Unknown,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct SafeWritableRow {
    pub row_id: &'static str,
    pub official_setting: &'static str,
    pub value_kind: ScalarWriteValueKind,
}

pub const SAFE_WRITABLE_TOGGLE_ROWS: &[(&str, &str)] = &[
    ("appearance.dim.modal", "decoration.dim_modal"),
    ("appearance.dim.inactive", "decoration.dim_inactive"),
    (
        "appearance.border_part_of_window",
        "decoration.border_part_of_window",
    ),
    ("appearance.blur.enabled", "decoration.blur.enabled"),
    (
        "appearance.blur.ignore_opacity",
        "decoration.blur.ignore_opacity",
    ),
    (
        "appearance.blur.new_optimizations",
        "decoration.blur.new_optimizations",
    ),
    ("appearance.blur.special", "decoration.blur.special"),
    ("appearance.blur.popups", "decoration.blur.popups"),
    (
        "appearance.blur.input_methods",
        "decoration.blur.input_methods",
    ),
    ("appearance.shadow.enabled", "decoration.shadow.enabled"),
    ("appearance.shadow.sharp", "decoration.shadow.sharp"),
    ("appearance.glow.enabled", "decoration.glow.enabled"),
    ("animations.enabled", "animations.enabled"),
    (
        "animations.workspace_wraparound",
        "animations.workspace_wraparound",
    ),
    ("group.groupbar.enabled", "group.groupbar.enabled"),
    ("group.groupbar.gradients", "group.groupbar.gradients"),
    ("group.groupbar.stacked", "group.groupbar.stacked"),
    (
        "group.groupbar.render_titles",
        "group.groupbar.render_titles",
    ),
    ("group.groupbar.scrolling", "group.groupbar.scrolling"),
    (
        "group.groupbar.middle_click_close",
        "group.groupbar.middle_click_close",
    ),
    (
        "group.groupbar.round_only_edges",
        "group.groupbar.round_only_edges",
    ),
    (
        "group.groupbar.gradient_round_only_edges",
        "group.groupbar.gradient_round_only_edges",
    ),
    (
        "group.groupbar.keep_upper_gap",
        "group.groupbar.keep_upper_gap",
    ),
    ("group.groupbar.blur", "group.groupbar.blur"),
    ("misc.disable_hyprland_logo", "misc.disable_hyprland_logo"),
    (
        "misc.disable_splash_rendering",
        "misc.disable_splash_rendering",
    ),
    ("misc.name_vk_after_proc", "misc.name_vk_after_proc"),
    ("misc.always_follow_on_dnd", "misc.always_follow_on_dnd"),
    (
        "misc.layers_hog_keyboard_focus",
        "misc.layers_hog_keyboard_focus",
    ),
    ("misc.animate_manual_resizes", "misc.animate_manual_resizes"),
    (
        "misc.animate_mouse_windowdragging",
        "misc.animate_mouse_windowdragging",
    ),
    ("misc.enable_swallow", "misc.enable_swallow"),
    (
        "misc.mouse_move_focuses_monitor",
        "misc.mouse_move_focuses_monitor",
    ),
    ("misc.close_special_on_empty", "misc.close_special_on_empty"),
    ("misc.middle_click_paste", "misc.middle_click_paste"),
    ("misc.disable_xdg_env_checks", "misc.disable_xdg_env_checks"),
    (
        "misc.disable_hyprland_guiutils_check",
        "misc.disable_hyprland_guiutils_check",
    ),
    (
        "misc.disable_watchdog_warning",
        "misc.disable_watchdog_warning",
    ),
    ("misc.enable_anr_dialog", "misc.enable_anr_dialog"),
    ("misc.screencopy_force_8b", "misc.screencopy_force_8b"),
    (
        "misc.disable_scale_notification",
        "misc.disable_scale_notification",
    ),
    ("misc.size_limits_tiled", "misc.size_limits_tiled"),
    ("windows.snap.enabled", "general.snap.enabled"),
];

pub const SAFE_WRITABLE_ROWS: &[SafeWritableRow] = &[
    SafeWritableRow {
        row_id: "appearance.dim.modal",
        official_setting: "decoration.dim_modal",
        value_kind: ScalarWriteValueKind::Boolean,
    },
    SafeWritableRow {
        row_id: "appearance.dim.inactive",
        official_setting: "decoration.dim_inactive",
        value_kind: ScalarWriteValueKind::Boolean,
    },
    SafeWritableRow {
        row_id: "appearance.border_part_of_window",
        official_setting: "decoration.border_part_of_window",
        value_kind: ScalarWriteValueKind::Boolean,
    },
    SafeWritableRow {
        row_id: "appearance.blur.enabled",
        official_setting: "decoration.blur.enabled",
        value_kind: ScalarWriteValueKind::Boolean,
    },
    SafeWritableRow {
        row_id: "appearance.blur.ignore_opacity",
        official_setting: "decoration.blur.ignore_opacity",
        value_kind: ScalarWriteValueKind::Boolean,
    },
    SafeWritableRow {
        row_id: "appearance.blur.new_optimizations",
        official_setting: "decoration.blur.new_optimizations",
        value_kind: ScalarWriteValueKind::Boolean,
    },
    SafeWritableRow {
        row_id: "appearance.blur.special",
        official_setting: "decoration.blur.special",
        value_kind: ScalarWriteValueKind::Boolean,
    },
    SafeWritableRow {
        row_id: "appearance.blur.popups",
        official_setting: "decoration.blur.popups",
        value_kind: ScalarWriteValueKind::Boolean,
    },
    SafeWritableRow {
        row_id: "appearance.blur.input_methods",
        official_setting: "decoration.blur.input_methods",
        value_kind: ScalarWriteValueKind::Boolean,
    },
    SafeWritableRow {
        row_id: "appearance.blur.size",
        official_setting: "decoration.blur.size",
        value_kind: ScalarWriteValueKind::Number,
    },
    SafeWritableRow {
        row_id: "appearance.blur.brightness",
        official_setting: "decoration.blur.brightness",
        value_kind: ScalarWriteValueKind::Percent,
    },
    SafeWritableRow {
        row_id: "appearance.blur.contrast",
        official_setting: "decoration.blur.contrast",
        value_kind: ScalarWriteValueKind::Percent,
    },
    SafeWritableRow {
        row_id: "appearance.shadow.enabled",
        official_setting: "decoration.shadow.enabled",
        value_kind: ScalarWriteValueKind::Boolean,
    },
    SafeWritableRow {
        row_id: "appearance.shadow.sharp",
        official_setting: "decoration.shadow.sharp",
        value_kind: ScalarWriteValueKind::Boolean,
    },
    SafeWritableRow {
        row_id: "appearance.shadow.range",
        official_setting: "decoration.shadow.range",
        value_kind: ScalarWriteValueKind::Number,
    },
    SafeWritableRow {
        row_id: "appearance.shadow.render_power",
        official_setting: "decoration.shadow.render_power",
        value_kind: ScalarWriteValueKind::Number,
    },
    SafeWritableRow {
        row_id: "decoration.shadow.color",
        official_setting: "decoration.shadow.color",
        value_kind: ScalarWriteValueKind::Color,
    },
    SafeWritableRow {
        row_id: "decoration.shadow.color_inactive",
        official_setting: "decoration.shadow.color_inactive",
        value_kind: ScalarWriteValueKind::Color,
    },
    SafeWritableRow {
        row_id: "decoration.shadow.offset",
        official_setting: "decoration.shadow.offset",
        value_kind: ScalarWriteValueKind::Vector2,
    },
    SafeWritableRow {
        row_id: "decoration.screen_shader",
        official_setting: "decoration.screen_shader",
        value_kind: ScalarWriteValueKind::Path,
    },
    SafeWritableRow {
        row_id: "appearance.gaps_in",
        official_setting: "general.gaps_in",
        value_kind: ScalarWriteValueKind::Number,
    },
    SafeWritableRow {
        row_id: "appearance.gaps_out",
        official_setting: "general.gaps_out",
        value_kind: ScalarWriteValueKind::Number,
    },
    SafeWritableRow {
        row_id: "appearance.border_size",
        official_setting: "general.border_size",
        value_kind: ScalarWriteValueKind::Number,
    },
    SafeWritableRow {
        row_id: "appearance.rounding",
        official_setting: "decoration.rounding",
        value_kind: ScalarWriteValueKind::Number,
    },
    SafeWritableRow {
        row_id: "appearance.active_opacity",
        official_setting: "decoration.active_opacity",
        value_kind: ScalarWriteValueKind::Percent,
    },
    SafeWritableRow {
        row_id: "appearance.inactive_opacity",
        official_setting: "decoration.inactive_opacity",
        value_kind: ScalarWriteValueKind::Percent,
    },
    SafeWritableRow {
        row_id: "animations.enabled",
        official_setting: "animations.enabled",
        value_kind: ScalarWriteValueKind::Boolean,
    },
    SafeWritableRow {
        row_id: "animations.workspace_wraparound",
        official_setting: "animations.workspace_wraparound",
        value_kind: ScalarWriteValueKind::Boolean,
    },
    SafeWritableRow {
        row_id: "appearance.glow.enabled",
        official_setting: "decoration.glow.enabled",
        value_kind: ScalarWriteValueKind::Boolean,
    },
    SafeWritableRow {
        row_id: "group.groupbar.enabled",
        official_setting: "group.groupbar.enabled",
        value_kind: ScalarWriteValueKind::Boolean,
    },
    SafeWritableRow {
        row_id: "group.groupbar.gradients",
        official_setting: "group.groupbar.gradients",
        value_kind: ScalarWriteValueKind::Boolean,
    },
    SafeWritableRow {
        row_id: "group.groupbar.stacked",
        official_setting: "group.groupbar.stacked",
        value_kind: ScalarWriteValueKind::Boolean,
    },
    SafeWritableRow {
        row_id: "group.groupbar.render_titles",
        official_setting: "group.groupbar.render_titles",
        value_kind: ScalarWriteValueKind::Boolean,
    },
    SafeWritableRow {
        row_id: "group.groupbar.scrolling",
        official_setting: "group.groupbar.scrolling",
        value_kind: ScalarWriteValueKind::Boolean,
    },
    SafeWritableRow {
        row_id: "group.groupbar.middle_click_close",
        official_setting: "group.groupbar.middle_click_close",
        value_kind: ScalarWriteValueKind::Boolean,
    },
    SafeWritableRow {
        row_id: "group.groupbar.round_only_edges",
        official_setting: "group.groupbar.round_only_edges",
        value_kind: ScalarWriteValueKind::Boolean,
    },
    SafeWritableRow {
        row_id: "group.groupbar.gradient_round_only_edges",
        official_setting: "group.groupbar.gradient_round_only_edges",
        value_kind: ScalarWriteValueKind::Boolean,
    },
    SafeWritableRow {
        row_id: "group.groupbar.keep_upper_gap",
        official_setting: "group.groupbar.keep_upper_gap",
        value_kind: ScalarWriteValueKind::Boolean,
    },
    SafeWritableRow {
        row_id: "group.groupbar.blur",
        official_setting: "group.groupbar.blur",
        value_kind: ScalarWriteValueKind::Boolean,
    },
    SafeWritableRow {
        row_id: "misc.disable_hyprland_logo",
        official_setting: "misc.disable_hyprland_logo",
        value_kind: ScalarWriteValueKind::Boolean,
    },
    SafeWritableRow {
        row_id: "misc.disable_splash_rendering",
        official_setting: "misc.disable_splash_rendering",
        value_kind: ScalarWriteValueKind::Boolean,
    },
    SafeWritableRow {
        row_id: "misc.name_vk_after_proc",
        official_setting: "misc.name_vk_after_proc",
        value_kind: ScalarWriteValueKind::Boolean,
    },
    SafeWritableRow {
        row_id: "misc.always_follow_on_dnd",
        official_setting: "misc.always_follow_on_dnd",
        value_kind: ScalarWriteValueKind::Boolean,
    },
    SafeWritableRow {
        row_id: "misc.layers_hog_keyboard_focus",
        official_setting: "misc.layers_hog_keyboard_focus",
        value_kind: ScalarWriteValueKind::Boolean,
    },
    SafeWritableRow {
        row_id: "misc.animate_manual_resizes",
        official_setting: "misc.animate_manual_resizes",
        value_kind: ScalarWriteValueKind::Boolean,
    },
    SafeWritableRow {
        row_id: "misc.animate_mouse_windowdragging",
        official_setting: "misc.animate_mouse_windowdragging",
        value_kind: ScalarWriteValueKind::Boolean,
    },
    SafeWritableRow {
        row_id: "misc.enable_swallow",
        official_setting: "misc.enable_swallow",
        value_kind: ScalarWriteValueKind::Boolean,
    },
    SafeWritableRow {
        row_id: "misc.mouse_move_focuses_monitor",
        official_setting: "misc.mouse_move_focuses_monitor",
        value_kind: ScalarWriteValueKind::Boolean,
    },
    SafeWritableRow {
        row_id: "misc.close_special_on_empty",
        official_setting: "misc.close_special_on_empty",
        value_kind: ScalarWriteValueKind::Boolean,
    },
    SafeWritableRow {
        row_id: "misc.middle_click_paste",
        official_setting: "misc.middle_click_paste",
        value_kind: ScalarWriteValueKind::Boolean,
    },
    SafeWritableRow {
        row_id: "misc.disable_xdg_env_checks",
        official_setting: "misc.disable_xdg_env_checks",
        value_kind: ScalarWriteValueKind::Boolean,
    },
    SafeWritableRow {
        row_id: "misc.disable_hyprland_guiutils_check",
        official_setting: "misc.disable_hyprland_guiutils_check",
        value_kind: ScalarWriteValueKind::Boolean,
    },
    SafeWritableRow {
        row_id: "misc.disable_watchdog_warning",
        official_setting: "misc.disable_watchdog_warning",
        value_kind: ScalarWriteValueKind::Boolean,
    },
    SafeWritableRow {
        row_id: "misc.enable_anr_dialog",
        official_setting: "misc.enable_anr_dialog",
        value_kind: ScalarWriteValueKind::Boolean,
    },
    SafeWritableRow {
        row_id: "misc.screencopy_force_8b",
        official_setting: "misc.screencopy_force_8b",
        value_kind: ScalarWriteValueKind::Boolean,
    },
    SafeWritableRow {
        row_id: "misc.disable_scale_notification",
        official_setting: "misc.disable_scale_notification",
        value_kind: ScalarWriteValueKind::Boolean,
    },
    SafeWritableRow {
        row_id: "misc.size_limits_tiled",
        official_setting: "misc.size_limits_tiled",
        value_kind: ScalarWriteValueKind::Boolean,
    },
    SafeWritableRow {
        row_id: "windows.snap.enabled",
        official_setting: "general.snap.enabled",
        value_kind: ScalarWriteValueKind::Boolean,
    },
    SafeWritableRow {
        row_id: "windows.snap.window_gap",
        official_setting: "general.snap.window_gap",
        value_kind: ScalarWriteValueKind::Number,
    },
    SafeWritableRow {
        row_id: "windows.snap.monitor_gap",
        official_setting: "general.snap.monitor_gap",
        value_kind: ScalarWriteValueKind::Number,
    },
    SafeWritableRow {
        row_id: "general.col.inactive_border",
        official_setting: "general.col.inactive_border",
        value_kind: ScalarWriteValueKind::Gradient,
    },
    SafeWritableRow {
        row_id: "general.col.active_border",
        official_setting: "general.col.active_border",
        value_kind: ScalarWriteValueKind::Gradient,
    },
    SafeWritableRow {
        row_id: "general.col.nogroup_border",
        official_setting: "general.col.nogroup_border",
        value_kind: ScalarWriteValueKind::Gradient,
    },
    SafeWritableRow {
        row_id: "general.col.nogroup_border_active",
        official_setting: "general.col.nogroup_border_active",
        value_kind: ScalarWriteValueKind::Gradient,
    },
    SafeWritableRow {
        row_id: "input.pointer_sensitivity",
        official_setting: "input.sensitivity",
        value_kind: ScalarWriteValueKind::Percent,
    },
    SafeWritableRow {
        row_id: "input.accel_profile",
        official_setting: "input.accel_profile",
        value_kind: ScalarWriteValueKind::LineSafeString,
    },
    SafeWritableRow {
        row_id: "input.scroll_points",
        official_setting: "input.scroll_points",
        value_kind: ScalarWriteValueKind::NumericList,
    },
    SafeWritableRow {
        row_id: "input.kb_file",
        official_setting: "input.kb_file",
        value_kind: ScalarWriteValueKind::Path,
    },
    SafeWritableRow {
        row_id: "input.tablet.region_position",
        official_setting: "input.tablet.region_position",
        value_kind: ScalarWriteValueKind::Vector2,
    },
    SafeWritableRow {
        row_id: "input.tablet.region_size",
        official_setting: "input.tablet.region_size",
        value_kind: ScalarWriteValueKind::Vector2,
    },
    SafeWritableRow {
        row_id: "input.tablet.active_area_size",
        official_setting: "input.tablet.active_area_size",
        value_kind: ScalarWriteValueKind::Vector2,
    },
    SafeWritableRow {
        row_id: "input.tablet.active_area_position",
        official_setting: "input.tablet.active_area_position",
        value_kind: ScalarWriteValueKind::Vector2,
    },
    SafeWritableRow {
        row_id: "decoration.glow.color",
        official_setting: "decoration.glow.color",
        value_kind: ScalarWriteValueKind::Color,
    },
    SafeWritableRow {
        row_id: "decoration.glow.color_inactive",
        official_setting: "decoration.glow.color_inactive",
        value_kind: ScalarWriteValueKind::Color,
    },
    SafeWritableRow {
        row_id: "group.groupbar.text_color",
        official_setting: "group.groupbar.text_color",
        value_kind: ScalarWriteValueKind::Color,
    },
    SafeWritableRow {
        row_id: "group.groupbar.text_color_inactive",
        official_setting: "group.groupbar.text_color_inactive",
        value_kind: ScalarWriteValueKind::Color,
    },
    SafeWritableRow {
        row_id: "group.groupbar.text_color_locked_active",
        official_setting: "group.groupbar.text_color_locked_active",
        value_kind: ScalarWriteValueKind::Color,
    },
    SafeWritableRow {
        row_id: "group.groupbar.text_color_locked_inactive",
        official_setting: "group.groupbar.text_color_locked_inactive",
        value_kind: ScalarWriteValueKind::Color,
    },
    SafeWritableRow {
        row_id: "group.groupbar.font_family",
        official_setting: "group.groupbar.font_family",
        value_kind: ScalarWriteValueKind::LineSafeString,
    },
    SafeWritableRow {
        row_id: "group.col.border_active",
        official_setting: "group.col.border_active",
        value_kind: ScalarWriteValueKind::Gradient,
    },
    SafeWritableRow {
        row_id: "group.col.border_inactive",
        official_setting: "group.col.border_inactive",
        value_kind: ScalarWriteValueKind::Gradient,
    },
    SafeWritableRow {
        row_id: "group.col.border_locked_inactive",
        official_setting: "group.col.border_locked_inactive",
        value_kind: ScalarWriteValueKind::Gradient,
    },
    SafeWritableRow {
        row_id: "group.col.border_locked_active",
        official_setting: "group.col.border_locked_active",
        value_kind: ScalarWriteValueKind::Gradient,
    },
    SafeWritableRow {
        row_id: "group.groupbar.col.active",
        official_setting: "group.groupbar.col.active",
        value_kind: ScalarWriteValueKind::Gradient,
    },
    SafeWritableRow {
        row_id: "group.groupbar.col.inactive",
        official_setting: "group.groupbar.col.inactive",
        value_kind: ScalarWriteValueKind::Gradient,
    },
    SafeWritableRow {
        row_id: "group.groupbar.col.locked_active",
        official_setting: "group.groupbar.col.locked_active",
        value_kind: ScalarWriteValueKind::Gradient,
    },
    SafeWritableRow {
        row_id: "group.groupbar.col.locked_inactive",
        official_setting: "group.groupbar.col.locked_inactive",
        value_kind: ScalarWriteValueKind::Gradient,
    },
    SafeWritableRow {
        row_id: "misc.col.splash",
        official_setting: "misc.col.splash",
        value_kind: ScalarWriteValueKind::Color,
    },
    SafeWritableRow {
        row_id: "misc.background_color",
        official_setting: "misc.background_color",
        value_kind: ScalarWriteValueKind::Color,
    },
    SafeWritableRow {
        row_id: "misc.font_family",
        official_setting: "misc.font_family",
        value_kind: ScalarWriteValueKind::LineSafeString,
    },
    SafeWritableRow {
        row_id: "misc.splash_font_family",
        official_setting: "misc.splash_font_family",
        value_kind: ScalarWriteValueKind::LineSafeString,
    },
    SafeWritableRow {
        row_id: "misc.swallow_regex",
        official_setting: "misc.swallow_regex",
        value_kind: ScalarWriteValueKind::RegexString,
    },
    SafeWritableRow {
        row_id: "misc.swallow_exception_regex",
        official_setting: "misc.swallow_exception_regex",
        value_kind: ScalarWriteValueKind::RegexString,
    },
    SafeWritableRow {
        row_id: "layout.single_window_aspect_ratio",
        official_setting: "layout.single_window_aspect_ratio",
        value_kind: ScalarWriteValueKind::Vector2,
    },
];

pub fn classify_inventory_entry(entry: &InventoryEntry) -> ScalarWriteClassification {
    let value_kind = value_kind_for_control(&entry.control_kind, &entry.value_family);
    let (status, blocker) = if safe_writable_official_setting(&entry.row_id).is_some() {
        (ScalarWriteStatus::SafeWritable, None)
    } else if entry.structured_family.unwrap_or(false) {
        (
            ScalarWriteStatus::BlockedStructured,
            Some("structured-family metadata is not writable through scalar path".to_string()),
        )
    } else if entry.report_only {
        (
            ScalarWriteStatus::BlockedHighRisk,
            Some(format!("report-only/high-risk row: {}", entry.risk_class)),
        )
    } else if entry.parser_status != "scalar-current-value-parser" {
        (
            ScalarWriteStatus::BlockedParserNeeded,
            Some(format!(
                "semantic parser needed before safe write: {}",
                entry.parser_status
            )),
        )
    } else if entry.risk_class != "safe" {
        (
            ScalarWriteStatus::BlockedManualReview,
            Some(format!(
                "manual review required for risk class: {}",
                entry.risk_class
            )),
        )
    } else if value_kind != ScalarWriteValueKind::Boolean {
        (
            ScalarWriteStatus::BlockedValidatorNeeded,
            Some("safe numeric/string validation metadata is not available yet".to_string()),
        )
    } else {
        (
            ScalarWriteStatus::BlockedManualReview,
            Some("not selected for the initial safe toggle write allowlist".to_string()),
        )
    };

    ScalarWriteClassification {
        row_id: entry.row_id.clone(),
        official_setting: entry.official_setting.clone(),
        config_key: config_key_from_official_setting(&entry.official_setting),
        status,
        blocker,
        value_kind,
    }
}

pub fn is_safe_writable_setting(row_id: &str) -> bool {
    safe_writable_official_setting(row_id).is_some()
}

pub fn safe_writable_official_setting(row_id: &str) -> Option<&'static str> {
    safe_writable_row(row_id).map(|row| row.official_setting)
}

pub fn safe_writable_value_kind(row_id: &str) -> Option<ScalarWriteValueKind> {
    safe_writable_row(row_id).map(|row| row.value_kind)
}

pub fn safe_writable_row(row_id: &str) -> Option<&'static SafeWritableRow> {
    SAFE_WRITABLE_ROWS
        .iter()
        .find(|candidate| candidate.row_id == row_id)
}

pub fn config_key_from_official_setting(setting: &str) -> String {
    setting.replace('.', ":")
}

pub fn value_kind_for_control(control_kind: &str, value_family: &str) -> ScalarWriteValueKind {
    match (control_kind, value_family) {
        ("toggle", "none") => ScalarWriteValueKind::Boolean,
        ("slider" | "number-input", "none") => ScalarWriteValueKind::Number,
        ("percent-slider", "none") => ScalarWriteValueKind::Percent,
        ("dropdown", "none") => ScalarWriteValueKind::StringLike,
        (_, "none") => ScalarWriteValueKind::Unknown,
        _ => ScalarWriteValueKind::ComplexRaw,
    }
}
