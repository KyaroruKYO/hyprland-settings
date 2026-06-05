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
    FiniteChoice,
    SourceBacked,
    MonitorName,
    Number,
    Percent,
    Color,
    Gradient,
    Vector2,
    NumericList,
    CommaSeparatedFloatList,
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

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct FiniteChoiceOption {
    pub raw_value: &'static str,
    pub label: &'static str,
}

pub const CONFLICT_FINITE_CHOICE_ROWS: &[&str] = &[
    "general.resize_corner",
    "input.focus_on_close",
    "input.float_switch_override_focus",
    "input.off_window_axis_events",
    "input.emulate_discrete_scroll",
    "input.touchpad.drag_lock",
    "input.touchpad.drag_3fg",
    "input.virtualkeyboard.share_states",
    "group.drag_into_group",
    "dwindle.force_split",
    "dwindle.split_bias",
    "scrolling.focus_fit_method",
];

pub const REMAINING_105_FINITE_CHOICE_ROWS: &[&str] = &[
    "layout.selection",
    "input.follow_mouse",
    "input.scroll_method",
    "input.touchpad.tap_button_map",
    "master.new_status",
    "master.new_on_active",
    "master.orientation",
    "scrolling.direction",
    "master.center_master_fallback",
];

pub const SOURCE_BACKED_INPUT_ROWS: &[&str] = &[
    "input.kb_model",
    "input.kb_layout",
    "input.kb_variant",
    "input.kb_options",
    "input.kb_rules",
];

pub const MONITOR_OUTPUT_ROWS: &[&str] = &["input.touchdevice.output", "input.tablet.output"];

pub const SESSION_RUNTIME_SENSITIVE_ROWS: &[&str] = &[
    "appearance.fullscreen_opacity",
    "appearance.blur.xray",
    "general.allow_tearing",
    "general.locale",
    "misc.vrr",
    "misc.mouse_move_enables_dpms",
    "misc.key_press_enables_dpms",
    "misc.disable_autoreload",
    "misc.focus_on_activate",
    "misc.allow_session_lock_restore",
    "misc.session_lock_xray",
    "misc.on_focus_under_fullscreen",
    "misc.exit_window_retains_fullscreen",
    "binds.movefocus_cycles_fullscreen",
    "binds.allow_pin_fullscreen",
    "scrolling.fullscreen_on_one_column",
];

pub const ECOSYSTEM_HIGH_RISK_WRITABLE_ROWS: &[&str] = &[
    "ecosystem.no_update_news",
    "ecosystem.no_donation_nag",
    "ecosystem.enforce_permissions",
];

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct SessionRuntimeWritePolicy {
    pub scope: &'static str,
    pub apply_path: &'static str,
    pub reread_oracle: &'static str,
    pub recovery_strategy: &'static str,
    pub approval_gate: &'static str,
    pub runtime_effect: &'static str,
    pub review_warning: &'static str,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct HighRiskWritePolicy {
    pub recovery_bucket: &'static str,
    pub approval_gate: &'static str,
    pub watchdog_requirement: &'static str,
    pub review_warning: &'static str,
}

const LAYOUT_SELECTION_CHOICES: &[FiniteChoiceOption] = &[
    FiniteChoiceOption {
        raw_value: "dwindle",
        label: "Dwindle",
    },
    FiniteChoiceOption {
        raw_value: "master",
        label: "Master",
    },
    FiniteChoiceOption {
        raw_value: "scrolling",
        label: "Scrolling",
    },
    FiniteChoiceOption {
        raw_value: "monocle",
        label: "Monocle",
    },
];

const INPUT_FOLLOW_MOUSE_CHOICES: &[FiniteChoiceOption] = &[
    FiniteChoiceOption {
        raw_value: "0",
        label: "Disabled",
    },
    FiniteChoiceOption {
        raw_value: "1",
        label: "Full",
    },
    FiniteChoiceOption {
        raw_value: "2",
        label: "Loose",
    },
    FiniteChoiceOption {
        raw_value: "3",
        label: "Loose (no mouse focus)",
    },
];

const INPUT_TOUCHPAD_TAP_BUTTON_MAP_CHOICES: &[FiniteChoiceOption] = &[
    FiniteChoiceOption {
        raw_value: "lrm",
        label: "Left / Right / Middle",
    },
    FiniteChoiceOption {
        raw_value: "lmr",
        label: "Left / Middle / Right",
    },
];

const MASTER_NEW_STATUS_CHOICES: &[FiniteChoiceOption] = &[
    FiniteChoiceOption {
        raw_value: "master",
        label: "Master",
    },
    FiniteChoiceOption {
        raw_value: "slave",
        label: "Slave",
    },
    FiniteChoiceOption {
        raw_value: "inherit",
        label: "Inherit",
    },
];

const MASTER_NEW_ON_ACTIVE_CHOICES: &[FiniteChoiceOption] = &[
    FiniteChoiceOption {
        raw_value: "none",
        label: "None",
    },
    FiniteChoiceOption {
        raw_value: "before",
        label: "Before active",
    },
    FiniteChoiceOption {
        raw_value: "after",
        label: "After active",
    },
];

const MASTER_ORIENTATION_CHOICES: &[FiniteChoiceOption] = &[
    FiniteChoiceOption {
        raw_value: "left",
        label: "Left",
    },
    FiniteChoiceOption {
        raw_value: "right",
        label: "Right",
    },
    FiniteChoiceOption {
        raw_value: "top",
        label: "Top",
    },
    FiniteChoiceOption {
        raw_value: "bottom",
        label: "Bottom",
    },
    FiniteChoiceOption {
        raw_value: "center",
        label: "Center",
    },
];

const SCROLLING_DIRECTION_CHOICES: &[FiniteChoiceOption] = &[
    FiniteChoiceOption {
        raw_value: "right",
        label: "Right",
    },
    FiniteChoiceOption {
        raw_value: "left",
        label: "Left",
    },
    FiniteChoiceOption {
        raw_value: "down",
        label: "Down",
    },
    FiniteChoiceOption {
        raw_value: "up",
        label: "Up",
    },
];

const MASTER_CENTER_MASTER_FALLBACK_CHOICES: &[FiniteChoiceOption] = &[
    FiniteChoiceOption {
        raw_value: "left",
        label: "Left",
    },
    FiniteChoiceOption {
        raw_value: "right",
        label: "Right",
    },
    FiniteChoiceOption {
        raw_value: "top",
        label: "Top",
    },
    FiniteChoiceOption {
        raw_value: "bottom",
        label: "Bottom",
    },
];

const INPUT_SCROLL_METHOD_CHOICES: &[FiniteChoiceOption] = &[
    FiniteChoiceOption {
        raw_value: "2fg",
        label: "Two-finger",
    },
    FiniteChoiceOption {
        raw_value: "edge",
        label: "Edge",
    },
    FiniteChoiceOption {
        raw_value: "on_button_down",
        label: "On button down",
    },
    FiniteChoiceOption {
        raw_value: "no_scroll",
        label: "No scroll",
    },
];

const GENERAL_RESIZE_CORNER_CHOICES: &[FiniteChoiceOption] = &[
    FiniteChoiceOption {
        raw_value: "0",
        label: "disable",
    },
    FiniteChoiceOption {
        raw_value: "1",
        label: "top_left",
    },
    FiniteChoiceOption {
        raw_value: "2",
        label: "top_right",
    },
    FiniteChoiceOption {
        raw_value: "3",
        label: "bottom_right",
    },
    FiniteChoiceOption {
        raw_value: "4",
        label: "bottom_left",
    },
];

const INPUT_FOCUS_ON_CLOSE_CHOICES: &[FiniteChoiceOption] = &[
    FiniteChoiceOption {
        raw_value: "0",
        label: "next",
    },
    FiniteChoiceOption {
        raw_value: "1",
        label: "cursor",
    },
    FiniteChoiceOption {
        raw_value: "2",
        label: "mru",
    },
];

const INPUT_FLOAT_SWITCH_OVERRIDE_FOCUS_CHOICES: &[FiniteChoiceOption] = &[
    FiniteChoiceOption {
        raw_value: "0",
        label: "Disabled",
    },
    FiniteChoiceOption {
        raw_value: "1",
        label: "Enabled",
    },
    FiniteChoiceOption {
        raw_value: "2",
        label: "Enabled also unfocuses",
    },
];

const INPUT_OFF_WINDOW_AXIS_EVENTS_CHOICES: &[FiniteChoiceOption] = &[
    FiniteChoiceOption {
        raw_value: "0",
        label: "ignore",
    },
    FiniteChoiceOption {
        raw_value: "1",
        label: "send",
    },
    FiniteChoiceOption {
        raw_value: "2",
        label: "clamp",
    },
    FiniteChoiceOption {
        raw_value: "3",
        label: "warp",
    },
];

const INPUT_EMULATE_DISCRETE_SCROLL_CHOICES: &[FiniteChoiceOption] = &[
    FiniteChoiceOption {
        raw_value: "0",
        label: "disable",
    },
    FiniteChoiceOption {
        raw_value: "1",
        label: "non_standard",
    },
    FiniteChoiceOption {
        raw_value: "2",
        label: "force_all",
    },
];

const INPUT_TOUCHPAD_DRAG_LOCK_CHOICES: &[FiniteChoiceOption] = &[
    FiniteChoiceOption {
        raw_value: "0",
        label: "Disabled",
    },
    FiniteChoiceOption {
        raw_value: "1",
        label: "Enabled with timeout",
    },
    FiniteChoiceOption {
        raw_value: "2",
        label: "Sticky",
    },
];

const INPUT_TOUCHPAD_DRAG_3FG_CHOICES: &[FiniteChoiceOption] = &[
    FiniteChoiceOption {
        raw_value: "0",
        label: "disable",
    },
    FiniteChoiceOption {
        raw_value: "1",
        label: "3_finger",
    },
    FiniteChoiceOption {
        raw_value: "2",
        label: "4_finger",
    },
];

const INPUT_VIRTUALKEYBOARD_SHARE_STATES_CHOICES: &[FiniteChoiceOption] = &[
    FiniteChoiceOption {
        raw_value: "0",
        label: "disable",
    },
    FiniteChoiceOption {
        raw_value: "1",
        label: "enable",
    },
    FiniteChoiceOption {
        raw_value: "2",
        label: "only_non_ime",
    },
];

const GROUP_DRAG_INTO_GROUP_CHOICES: &[FiniteChoiceOption] = &[
    FiniteChoiceOption {
        raw_value: "0",
        label: "disabled",
    },
    FiniteChoiceOption {
        raw_value: "1",
        label: "enabled",
    },
    FiniteChoiceOption {
        raw_value: "2",
        label: "only when dragging into the groupbar",
    },
];

const DWINDLE_FORCE_SPLIT_CHOICES: &[FiniteChoiceOption] = &[
    FiniteChoiceOption {
        raw_value: "0",
        label: "Follow mouse",
    },
    FiniteChoiceOption {
        raw_value: "1",
        label: "Left / Top",
    },
    FiniteChoiceOption {
        raw_value: "2",
        label: "Right / Bottom",
    },
];

const DWINDLE_SPLIT_BIAS_CHOICES: &[FiniteChoiceOption] = &[
    FiniteChoiceOption {
        raw_value: "0",
        label: "directional",
    },
    FiniteChoiceOption {
        raw_value: "1",
        label: "current",
    },
];

const SCROLLING_FOCUS_FIT_METHOD_CHOICES: &[FiniteChoiceOption] = &[
    FiniteChoiceOption {
        raw_value: "0",
        label: "center",
    },
    FiniteChoiceOption {
        raw_value: "1",
        label: "fit",
    },
];

const MISC_VRR_CHOICES: &[FiniteChoiceOption] = &[
    FiniteChoiceOption {
        raw_value: "0",
        label: "Off",
    },
    FiniteChoiceOption {
        raw_value: "1",
        label: "On",
    },
    FiniteChoiceOption {
        raw_value: "2",
        label: "Fullscreen only",
    },
    FiniteChoiceOption {
        raw_value: "3",
        label: "Fullscreen game content",
    },
];

const MISC_ON_FOCUS_UNDER_FULLSCREEN_CHOICES: &[FiniteChoiceOption] = &[
    FiniteChoiceOption {
        raw_value: "0",
        label: "ignore",
    },
    FiniteChoiceOption {
        raw_value: "1",
        label: "take_over",
    },
    FiniteChoiceOption {
        raw_value: "2",
        label: "exit_fullscreen",
    },
];

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
        row_id: "appearance.fullscreen_opacity",
        official_setting: "decoration.fullscreen_opacity",
        value_kind: ScalarWriteValueKind::Percent,
    },
    SafeWritableRow {
        row_id: "appearance.blur.xray",
        official_setting: "decoration.blur.xray",
        value_kind: ScalarWriteValueKind::Boolean,
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
        row_id: "misc.vrr",
        official_setting: "misc.vrr",
        value_kind: ScalarWriteValueKind::FiniteChoice,
    },
    SafeWritableRow {
        row_id: "misc.mouse_move_enables_dpms",
        official_setting: "misc.mouse_move_enables_dpms",
        value_kind: ScalarWriteValueKind::Boolean,
    },
    SafeWritableRow {
        row_id: "misc.key_press_enables_dpms",
        official_setting: "misc.key_press_enables_dpms",
        value_kind: ScalarWriteValueKind::Boolean,
    },
    SafeWritableRow {
        row_id: "misc.disable_autoreload",
        official_setting: "misc.disable_autoreload",
        value_kind: ScalarWriteValueKind::Boolean,
    },
    SafeWritableRow {
        row_id: "misc.focus_on_activate",
        official_setting: "misc.focus_on_activate",
        value_kind: ScalarWriteValueKind::Boolean,
    },
    SafeWritableRow {
        row_id: "misc.allow_session_lock_restore",
        official_setting: "misc.allow_session_lock_restore",
        value_kind: ScalarWriteValueKind::Boolean,
    },
    SafeWritableRow {
        row_id: "misc.session_lock_xray",
        official_setting: "misc.session_lock_xray",
        value_kind: ScalarWriteValueKind::Boolean,
    },
    SafeWritableRow {
        row_id: "misc.on_focus_under_fullscreen",
        official_setting: "misc.on_focus_under_fullscreen",
        value_kind: ScalarWriteValueKind::FiniteChoice,
    },
    SafeWritableRow {
        row_id: "misc.exit_window_retains_fullscreen",
        official_setting: "misc.exit_window_retains_fullscreen",
        value_kind: ScalarWriteValueKind::Boolean,
    },
    SafeWritableRow {
        row_id: "windows.snap.enabled",
        official_setting: "general.snap.enabled",
        value_kind: ScalarWriteValueKind::Boolean,
    },
    SafeWritableRow {
        row_id: "general.allow_tearing",
        official_setting: "general.allow_tearing",
        value_kind: ScalarWriteValueKind::Boolean,
    },
    SafeWritableRow {
        row_id: "general.locale",
        official_setting: "general.locale",
        value_kind: ScalarWriteValueKind::LineSafeString,
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
        row_id: "input.kb_model",
        official_setting: "input.kb_model",
        value_kind: ScalarWriteValueKind::SourceBacked,
    },
    SafeWritableRow {
        row_id: "input.kb_layout",
        official_setting: "input.kb_layout",
        value_kind: ScalarWriteValueKind::SourceBacked,
    },
    SafeWritableRow {
        row_id: "input.kb_variant",
        official_setting: "input.kb_variant",
        value_kind: ScalarWriteValueKind::SourceBacked,
    },
    SafeWritableRow {
        row_id: "input.kb_options",
        official_setting: "input.kb_options",
        value_kind: ScalarWriteValueKind::SourceBacked,
    },
    SafeWritableRow {
        row_id: "input.kb_rules",
        official_setting: "input.kb_rules",
        value_kind: ScalarWriteValueKind::SourceBacked,
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
    SafeWritableRow {
        row_id: "appearance.blur.passes",
        official_setting: "decoration.blur.passes",
        value_kind: ScalarWriteValueKind::Number,
    },
    SafeWritableRow {
        row_id: "appearance.blur.noise",
        official_setting: "decoration.blur.noise",
        value_kind: ScalarWriteValueKind::Number,
    },
    SafeWritableRow {
        row_id: "appearance.shadow.scale",
        official_setting: "decoration.shadow.scale",
        value_kind: ScalarWriteValueKind::Percent,
    },
    SafeWritableRow {
        row_id: "appearance.rounding_power",
        official_setting: "decoration.rounding_power",
        value_kind: ScalarWriteValueKind::Number,
    },
    SafeWritableRow {
        row_id: "appearance.dim.strength",
        official_setting: "decoration.dim_strength",
        value_kind: ScalarWriteValueKind::Percent,
    },
    SafeWritableRow {
        row_id: "appearance.dim.special",
        official_setting: "decoration.dim_special",
        value_kind: ScalarWriteValueKind::Percent,
    },
    SafeWritableRow {
        row_id: "appearance.dim.around",
        official_setting: "decoration.dim_around",
        value_kind: ScalarWriteValueKind::Percent,
    },
    SafeWritableRow {
        row_id: "appearance.blur.vibrancy",
        official_setting: "decoration.blur.vibrancy",
        value_kind: ScalarWriteValueKind::Percent,
    },
    SafeWritableRow {
        row_id: "appearance.blur.vibrancy_darkness",
        official_setting: "decoration.blur.vibrancy_darkness",
        value_kind: ScalarWriteValueKind::Percent,
    },
    SafeWritableRow {
        row_id: "appearance.blur.popups_ignorealpha",
        official_setting: "decoration.blur.popups_ignorealpha",
        value_kind: ScalarWriteValueKind::Percent,
    },
    SafeWritableRow {
        row_id: "appearance.blur.input_methods_ignorealpha",
        official_setting: "decoration.blur.input_methods_ignorealpha",
        value_kind: ScalarWriteValueKind::Percent,
    },
    SafeWritableRow {
        row_id: "appearance.glow.range",
        official_setting: "decoration.glow.range",
        value_kind: ScalarWriteValueKind::Number,
    },
    SafeWritableRow {
        row_id: "appearance.glow.render_power",
        official_setting: "decoration.glow.render_power",
        value_kind: ScalarWriteValueKind::Number,
    },
    SafeWritableRow {
        row_id: "general.float_gaps",
        official_setting: "general.float_gaps",
        value_kind: ScalarWriteValueKind::Number,
    },
    SafeWritableRow {
        row_id: "general.gaps_workspaces",
        official_setting: "general.gaps_workspaces",
        value_kind: ScalarWriteValueKind::Number,
    },
    SafeWritableRow {
        row_id: "layout.selection",
        official_setting: "general.layout",
        value_kind: ScalarWriteValueKind::FiniteChoice,
    },
    SafeWritableRow {
        row_id: "general.no_focus_fallback",
        official_setting: "general.no_focus_fallback",
        value_kind: ScalarWriteValueKind::Boolean,
    },
    SafeWritableRow {
        row_id: "general.resize_on_border",
        official_setting: "general.resize_on_border",
        value_kind: ScalarWriteValueKind::Boolean,
    },
    SafeWritableRow {
        row_id: "general.extend_border_grab_area",
        official_setting: "general.extend_border_grab_area",
        value_kind: ScalarWriteValueKind::Number,
    },
    SafeWritableRow {
        row_id: "general.hover_icon_on_border",
        official_setting: "general.hover_icon_on_border",
        value_kind: ScalarWriteValueKind::Boolean,
    },
    SafeWritableRow {
        row_id: "general.resize_corner",
        official_setting: "general.resize_corner",
        value_kind: ScalarWriteValueKind::FiniteChoice,
    },
    SafeWritableRow {
        row_id: "general.snap.border_overlap",
        official_setting: "general.snap.border_overlap",
        value_kind: ScalarWriteValueKind::Boolean,
    },
    SafeWritableRow {
        row_id: "general.snap.respect_gaps",
        official_setting: "general.snap.respect_gaps",
        value_kind: ScalarWriteValueKind::Boolean,
    },
    SafeWritableRow {
        row_id: "general.modal_parent_blocking",
        official_setting: "general.modal_parent_blocking",
        value_kind: ScalarWriteValueKind::Boolean,
    },
    SafeWritableRow {
        row_id: "input.numlock_by_default",
        official_setting: "input.numlock_by_default",
        value_kind: ScalarWriteValueKind::Boolean,
    },
    SafeWritableRow {
        row_id: "input.resolve_binds_by_sym",
        official_setting: "input.resolve_binds_by_sym",
        value_kind: ScalarWriteValueKind::Boolean,
    },
    SafeWritableRow {
        row_id: "input.repeat_rate",
        official_setting: "input.repeat_rate",
        value_kind: ScalarWriteValueKind::Number,
    },
    SafeWritableRow {
        row_id: "input.repeat_delay",
        official_setting: "input.repeat_delay",
        value_kind: ScalarWriteValueKind::Number,
    },
    SafeWritableRow {
        row_id: "input.force_no_accel",
        official_setting: "input.force_no_accel",
        value_kind: ScalarWriteValueKind::Boolean,
    },
    SafeWritableRow {
        row_id: "input.rotation",
        official_setting: "input.rotation",
        value_kind: ScalarWriteValueKind::Number,
    },
    SafeWritableRow {
        row_id: "input.left_handed",
        official_setting: "input.left_handed",
        value_kind: ScalarWriteValueKind::Boolean,
    },
    SafeWritableRow {
        row_id: "input.scroll_button",
        official_setting: "input.scroll_button",
        value_kind: ScalarWriteValueKind::Number,
    },
    SafeWritableRow {
        row_id: "input.scroll_button_lock",
        official_setting: "input.scroll_button_lock",
        value_kind: ScalarWriteValueKind::Boolean,
    },
    SafeWritableRow {
        row_id: "input.scroll_factor",
        official_setting: "input.scroll_factor",
        value_kind: ScalarWriteValueKind::Percent,
    },
    SafeWritableRow {
        row_id: "input.scroll_method",
        official_setting: "input.scroll_method",
        value_kind: ScalarWriteValueKind::FiniteChoice,
    },
    SafeWritableRow {
        row_id: "input.natural_scroll",
        official_setting: "input.natural_scroll",
        value_kind: ScalarWriteValueKind::Boolean,
    },
    SafeWritableRow {
        row_id: "input.follow_mouse_threshold",
        official_setting: "input.follow_mouse_threshold",
        value_kind: ScalarWriteValueKind::Number,
    },
    SafeWritableRow {
        row_id: "input.follow_mouse",
        official_setting: "input.follow_mouse",
        value_kind: ScalarWriteValueKind::FiniteChoice,
    },
    SafeWritableRow {
        row_id: "input.focus_on_close",
        official_setting: "input.focus_on_close",
        value_kind: ScalarWriteValueKind::FiniteChoice,
    },
    SafeWritableRow {
        row_id: "input.mouse_refocus",
        official_setting: "input.mouse_refocus",
        value_kind: ScalarWriteValueKind::Boolean,
    },
    SafeWritableRow {
        row_id: "input.float_switch_override_focus",
        official_setting: "input.float_switch_override_focus",
        value_kind: ScalarWriteValueKind::FiniteChoice,
    },
    SafeWritableRow {
        row_id: "input.special_fallthrough",
        official_setting: "input.special_fallthrough",
        value_kind: ScalarWriteValueKind::Boolean,
    },
    SafeWritableRow {
        row_id: "input.off_window_axis_events",
        official_setting: "input.off_window_axis_events",
        value_kind: ScalarWriteValueKind::FiniteChoice,
    },
    SafeWritableRow {
        row_id: "input.emulate_discrete_scroll",
        official_setting: "input.emulate_discrete_scroll",
        value_kind: ScalarWriteValueKind::FiniteChoice,
    },
    SafeWritableRow {
        row_id: "input.follow_mouse_shrink",
        official_setting: "input.follow_mouse_shrink",
        value_kind: ScalarWriteValueKind::Number,
    },
    SafeWritableRow {
        row_id: "input.touchpad.disable_while_typing",
        official_setting: "input.touchpad.disable_while_typing",
        value_kind: ScalarWriteValueKind::Boolean,
    },
    SafeWritableRow {
        row_id: "input.touchpad.natural_scroll",
        official_setting: "input.touchpad.natural_scroll",
        value_kind: ScalarWriteValueKind::Boolean,
    },
    SafeWritableRow {
        row_id: "input.touchpad.scroll_factor",
        official_setting: "input.touchpad.scroll_factor",
        value_kind: ScalarWriteValueKind::Percent,
    },
    SafeWritableRow {
        row_id: "input.touchpad.middle_button_emulation",
        official_setting: "input.touchpad.middle_button_emulation",
        value_kind: ScalarWriteValueKind::Boolean,
    },
    SafeWritableRow {
        row_id: "input.touchpad.clickfinger_behavior",
        official_setting: "input.touchpad.clickfinger_behavior",
        value_kind: ScalarWriteValueKind::Boolean,
    },
    SafeWritableRow {
        row_id: "input.touchpad.tap-to-click",
        official_setting: "input.touchpad.tap-to-click",
        value_kind: ScalarWriteValueKind::Boolean,
    },
    SafeWritableRow {
        row_id: "input.touchpad.drag_lock",
        official_setting: "input.touchpad.drag_lock",
        value_kind: ScalarWriteValueKind::FiniteChoice,
    },
    SafeWritableRow {
        row_id: "input.touchpad.tap_button_map",
        official_setting: "input.touchpad.tap_button_map",
        value_kind: ScalarWriteValueKind::FiniteChoice,
    },
    SafeWritableRow {
        row_id: "input.touchpad.tap-and-drag",
        official_setting: "input.touchpad.tap-and-drag",
        value_kind: ScalarWriteValueKind::Boolean,
    },
    SafeWritableRow {
        row_id: "input.touchpad.flip_x",
        official_setting: "input.touchpad.flip_x",
        value_kind: ScalarWriteValueKind::Boolean,
    },
    SafeWritableRow {
        row_id: "input.touchpad.flip_y",
        official_setting: "input.touchpad.flip_y",
        value_kind: ScalarWriteValueKind::Boolean,
    },
    SafeWritableRow {
        row_id: "input.touchpad.drag_3fg",
        official_setting: "input.touchpad.drag_3fg",
        value_kind: ScalarWriteValueKind::FiniteChoice,
    },
    SafeWritableRow {
        row_id: "input.touchdevice.transform",
        official_setting: "input.touchdevice.transform",
        value_kind: ScalarWriteValueKind::Number,
    },
    SafeWritableRow {
        row_id: "input.touchdevice.enabled",
        official_setting: "input.touchdevice.enabled",
        value_kind: ScalarWriteValueKind::Boolean,
    },
    SafeWritableRow {
        row_id: "input.touchdevice.output",
        official_setting: "input.touchdevice.output",
        value_kind: ScalarWriteValueKind::MonitorName,
    },
    SafeWritableRow {
        row_id: "input.virtualkeyboard.share_states",
        official_setting: "input.virtualkeyboard.share_states",
        value_kind: ScalarWriteValueKind::FiniteChoice,
    },
    SafeWritableRow {
        row_id: "input.virtualkeyboard.release_pressed_on_close",
        official_setting: "input.virtualkeyboard.release_pressed_on_close",
        value_kind: ScalarWriteValueKind::Boolean,
    },
    SafeWritableRow {
        row_id: "input.tablet.transform",
        official_setting: "input.tablet.transform",
        value_kind: ScalarWriteValueKind::Number,
    },
    SafeWritableRow {
        row_id: "input.tablet.absolute_region_position",
        official_setting: "input.tablet.absolute_region_position",
        value_kind: ScalarWriteValueKind::Boolean,
    },
    SafeWritableRow {
        row_id: "input.tablet.relative_input",
        official_setting: "input.tablet.relative_input",
        value_kind: ScalarWriteValueKind::Boolean,
    },
    SafeWritableRow {
        row_id: "input.tablet.output",
        official_setting: "input.tablet.output",
        value_kind: ScalarWriteValueKind::MonitorName,
    },
    SafeWritableRow {
        row_id: "input.tablet.left_handed",
        official_setting: "input.tablet.left_handed",
        value_kind: ScalarWriteValueKind::Boolean,
    },
    SafeWritableRow {
        row_id: "gestures.workspace_swipe_distance",
        official_setting: "gestures.workspace_swipe_distance",
        value_kind: ScalarWriteValueKind::Number,
    },
    SafeWritableRow {
        row_id: "gestures.workspace_swipe_touch",
        official_setting: "gestures.workspace_swipe_touch",
        value_kind: ScalarWriteValueKind::Boolean,
    },
    SafeWritableRow {
        row_id: "gestures.workspace_swipe_invert",
        official_setting: "gestures.workspace_swipe_invert",
        value_kind: ScalarWriteValueKind::Boolean,
    },
    SafeWritableRow {
        row_id: "gestures.workspace_swipe_touch_invert",
        official_setting: "gestures.workspace_swipe_touch_invert",
        value_kind: ScalarWriteValueKind::Boolean,
    },
    SafeWritableRow {
        row_id: "gestures.workspace_swipe_min_speed_to_force",
        official_setting: "gestures.workspace_swipe_min_speed_to_force",
        value_kind: ScalarWriteValueKind::Number,
    },
    SafeWritableRow {
        row_id: "gestures.workspace_swipe_cancel_ratio",
        official_setting: "gestures.workspace_swipe_cancel_ratio",
        value_kind: ScalarWriteValueKind::Percent,
    },
    SafeWritableRow {
        row_id: "gestures.workspace_swipe_create_new",
        official_setting: "gestures.workspace_swipe_create_new",
        value_kind: ScalarWriteValueKind::Boolean,
    },
    SafeWritableRow {
        row_id: "gestures.workspace_swipe_direction_lock",
        official_setting: "gestures.workspace_swipe_direction_lock",
        value_kind: ScalarWriteValueKind::Boolean,
    },
    SafeWritableRow {
        row_id: "gestures.workspace_swipe_direction_lock_threshold",
        official_setting: "gestures.workspace_swipe_direction_lock_threshold",
        value_kind: ScalarWriteValueKind::Number,
    },
    SafeWritableRow {
        row_id: "gestures.workspace_swipe_forever",
        official_setting: "gestures.workspace_swipe_forever",
        value_kind: ScalarWriteValueKind::Boolean,
    },
    SafeWritableRow {
        row_id: "gestures.workspace_swipe_use_r",
        official_setting: "gestures.workspace_swipe_use_r",
        value_kind: ScalarWriteValueKind::Boolean,
    },
    SafeWritableRow {
        row_id: "gestures.close_max_timeout",
        official_setting: "gestures.close_max_timeout",
        value_kind: ScalarWriteValueKind::Number,
    },
    SafeWritableRow {
        row_id: "gestures.scrolling.move_snap_to_grid",
        official_setting: "gestures.scrolling.move_snap_to_grid",
        value_kind: ScalarWriteValueKind::Boolean,
    },
    SafeWritableRow {
        row_id: "gestures.scrolling.move_snap_cursor",
        official_setting: "gestures.scrolling.move_snap_cursor",
        value_kind: ScalarWriteValueKind::Boolean,
    },
    SafeWritableRow {
        row_id: "group.insert_after_current",
        official_setting: "group.insert_after_current",
        value_kind: ScalarWriteValueKind::Boolean,
    },
    SafeWritableRow {
        row_id: "group.focus_removed_window",
        official_setting: "group.focus_removed_window",
        value_kind: ScalarWriteValueKind::Boolean,
    },
    SafeWritableRow {
        row_id: "group.merge_groups_on_drag",
        official_setting: "group.merge_groups_on_drag",
        value_kind: ScalarWriteValueKind::Boolean,
    },
    SafeWritableRow {
        row_id: "group.merge_groups_on_groupbar",
        official_setting: "group.merge_groups_on_groupbar",
        value_kind: ScalarWriteValueKind::Boolean,
    },
    SafeWritableRow {
        row_id: "group.auto_group",
        official_setting: "group.auto_group",
        value_kind: ScalarWriteValueKind::Boolean,
    },
    SafeWritableRow {
        row_id: "group.drag_into_group",
        official_setting: "group.drag_into_group",
        value_kind: ScalarWriteValueKind::FiniteChoice,
    },
    SafeWritableRow {
        row_id: "group.merge_floated_into_tiled_on_groupbar",
        official_setting: "group.merge_floated_into_tiled_on_groupbar",
        value_kind: ScalarWriteValueKind::Boolean,
    },
    SafeWritableRow {
        row_id: "group.group_on_movetoworkspace",
        official_setting: "group.group_on_movetoworkspace",
        value_kind: ScalarWriteValueKind::Boolean,
    },
    SafeWritableRow {
        row_id: "group.groupbar.font_weight_active",
        official_setting: "group.groupbar.font_weight_active",
        value_kind: ScalarWriteValueKind::Number,
    },
    SafeWritableRow {
        row_id: "group.groupbar.font_weight_inactive",
        official_setting: "group.groupbar.font_weight_inactive",
        value_kind: ScalarWriteValueKind::Number,
    },
    SafeWritableRow {
        row_id: "group.groupbar.font_size",
        official_setting: "group.groupbar.font_size",
        value_kind: ScalarWriteValueKind::Number,
    },
    SafeWritableRow {
        row_id: "group.groupbar.height",
        official_setting: "group.groupbar.height",
        value_kind: ScalarWriteValueKind::Number,
    },
    SafeWritableRow {
        row_id: "group.groupbar.indicator_gap",
        official_setting: "group.groupbar.indicator_gap",
        value_kind: ScalarWriteValueKind::Number,
    },
    SafeWritableRow {
        row_id: "group.groupbar.indicator_height",
        official_setting: "group.groupbar.indicator_height",
        value_kind: ScalarWriteValueKind::Number,
    },
    SafeWritableRow {
        row_id: "group.groupbar.priority",
        official_setting: "group.groupbar.priority",
        value_kind: ScalarWriteValueKind::Number,
    },
    SafeWritableRow {
        row_id: "group.groupbar.rounding",
        official_setting: "group.groupbar.rounding",
        value_kind: ScalarWriteValueKind::Number,
    },
    SafeWritableRow {
        row_id: "group.groupbar.rounding_power",
        official_setting: "group.groupbar.rounding_power",
        value_kind: ScalarWriteValueKind::Number,
    },
    SafeWritableRow {
        row_id: "group.groupbar.gradient_rounding",
        official_setting: "group.groupbar.gradient_rounding",
        value_kind: ScalarWriteValueKind::Number,
    },
    SafeWritableRow {
        row_id: "group.groupbar.gradient_rounding_power",
        official_setting: "group.groupbar.gradient_rounding_power",
        value_kind: ScalarWriteValueKind::Number,
    },
    SafeWritableRow {
        row_id: "group.groupbar.gaps_out",
        official_setting: "group.groupbar.gaps_out",
        value_kind: ScalarWriteValueKind::Number,
    },
    SafeWritableRow {
        row_id: "group.groupbar.gaps_in",
        official_setting: "group.groupbar.gaps_in",
        value_kind: ScalarWriteValueKind::Number,
    },
    SafeWritableRow {
        row_id: "group.groupbar.text_offset",
        official_setting: "group.groupbar.text_offset",
        value_kind: ScalarWriteValueKind::Number,
    },
    SafeWritableRow {
        row_id: "group.groupbar.text_padding",
        official_setting: "group.groupbar.text_padding",
        value_kind: ScalarWriteValueKind::Number,
    },
    SafeWritableRow {
        row_id: "misc.force_default_wallpaper",
        official_setting: "misc.force_default_wallpaper",
        value_kind: ScalarWriteValueKind::Number,
    },
    SafeWritableRow {
        row_id: "misc.initial_workspace_tracking",
        official_setting: "misc.initial_workspace_tracking",
        value_kind: ScalarWriteValueKind::Number,
    },
    SafeWritableRow {
        row_id: "misc.render_unfocused_fps",
        official_setting: "misc.render_unfocused_fps",
        value_kind: ScalarWriteValueKind::Number,
    },
    SafeWritableRow {
        row_id: "misc.lockdead_screen_delay",
        official_setting: "misc.lockdead_screen_delay",
        value_kind: ScalarWriteValueKind::Number,
    },
    SafeWritableRow {
        row_id: "misc.anr_missed_pings",
        official_setting: "misc.anr_missed_pings",
        value_kind: ScalarWriteValueKind::Number,
    },
    SafeWritableRow {
        row_id: "binds.pass_mouse_when_bound",
        official_setting: "binds.pass_mouse_when_bound",
        value_kind: ScalarWriteValueKind::Boolean,
    },
    SafeWritableRow {
        row_id: "binds.scroll_event_delay",
        official_setting: "binds.scroll_event_delay",
        value_kind: ScalarWriteValueKind::Number,
    },
    SafeWritableRow {
        row_id: "binds.workspace_back_and_forth",
        official_setting: "binds.workspace_back_and_forth",
        value_kind: ScalarWriteValueKind::Boolean,
    },
    SafeWritableRow {
        row_id: "binds.hide_special_on_workspace_change",
        official_setting: "binds.hide_special_on_workspace_change",
        value_kind: ScalarWriteValueKind::Boolean,
    },
    SafeWritableRow {
        row_id: "binds.allow_workspace_cycles",
        official_setting: "binds.allow_workspace_cycles",
        value_kind: ScalarWriteValueKind::Boolean,
    },
    SafeWritableRow {
        row_id: "binds.workspace_center_on",
        official_setting: "binds.workspace_center_on",
        value_kind: ScalarWriteValueKind::Number,
    },
    SafeWritableRow {
        row_id: "binds.focus_preferred_method",
        official_setting: "binds.focus_preferred_method",
        value_kind: ScalarWriteValueKind::Number,
    },
    SafeWritableRow {
        row_id: "binds.ignore_group_lock",
        official_setting: "binds.ignore_group_lock",
        value_kind: ScalarWriteValueKind::Boolean,
    },
    SafeWritableRow {
        row_id: "binds.movefocus_cycles_groupfirst",
        official_setting: "binds.movefocus_cycles_groupfirst",
        value_kind: ScalarWriteValueKind::Boolean,
    },
    SafeWritableRow {
        row_id: "binds.movefocus_cycles_fullscreen",
        official_setting: "binds.movefocus_cycles_fullscreen",
        value_kind: ScalarWriteValueKind::Boolean,
    },
    SafeWritableRow {
        row_id: "binds.disable_keybind_grabbing",
        official_setting: "binds.disable_keybind_grabbing",
        value_kind: ScalarWriteValueKind::Boolean,
    },
    SafeWritableRow {
        row_id: "binds.window_direction_monitor_fallback",
        official_setting: "binds.window_direction_monitor_fallback",
        value_kind: ScalarWriteValueKind::Boolean,
    },
    SafeWritableRow {
        row_id: "binds.drag_threshold",
        official_setting: "binds.drag_threshold",
        value_kind: ScalarWriteValueKind::Number,
    },
    SafeWritableRow {
        row_id: "binds.allow_pin_fullscreen",
        official_setting: "binds.allow_pin_fullscreen",
        value_kind: ScalarWriteValueKind::Boolean,
    },
    SafeWritableRow {
        row_id: "layout.single_window_aspect_ratio_tolerance",
        official_setting: "layout.single_window_aspect_ratio_tolerance",
        value_kind: ScalarWriteValueKind::Percent,
    },
    SafeWritableRow {
        row_id: "dwindle.force_split",
        official_setting: "dwindle.force_split",
        value_kind: ScalarWriteValueKind::FiniteChoice,
    },
    SafeWritableRow {
        row_id: "dwindle.preserve_split",
        official_setting: "dwindle.preserve_split",
        value_kind: ScalarWriteValueKind::Boolean,
    },
    SafeWritableRow {
        row_id: "dwindle.smart_split",
        official_setting: "dwindle.smart_split",
        value_kind: ScalarWriteValueKind::Boolean,
    },
    SafeWritableRow {
        row_id: "dwindle.smart_resizing",
        official_setting: "dwindle.smart_resizing",
        value_kind: ScalarWriteValueKind::Boolean,
    },
    SafeWritableRow {
        row_id: "dwindle.permanent_direction_override",
        official_setting: "dwindle.permanent_direction_override",
        value_kind: ScalarWriteValueKind::Boolean,
    },
    SafeWritableRow {
        row_id: "dwindle.special_scale_factor",
        official_setting: "dwindle.special_scale_factor",
        value_kind: ScalarWriteValueKind::Percent,
    },
    SafeWritableRow {
        row_id: "dwindle.split_width_multiplier",
        official_setting: "dwindle.split_width_multiplier",
        value_kind: ScalarWriteValueKind::Number,
    },
    SafeWritableRow {
        row_id: "dwindle.use_active_for_splits",
        official_setting: "dwindle.use_active_for_splits",
        value_kind: ScalarWriteValueKind::Boolean,
    },
    SafeWritableRow {
        row_id: "dwindle.default_split_ratio",
        official_setting: "dwindle.default_split_ratio",
        value_kind: ScalarWriteValueKind::Percent,
    },
    SafeWritableRow {
        row_id: "dwindle.split_bias",
        official_setting: "dwindle.split_bias",
        value_kind: ScalarWriteValueKind::FiniteChoice,
    },
    SafeWritableRow {
        row_id: "dwindle.precise_mouse_move",
        official_setting: "dwindle.precise_mouse_move",
        value_kind: ScalarWriteValueKind::Boolean,
    },
    SafeWritableRow {
        row_id: "master.allow_small_split",
        official_setting: "master.allow_small_split",
        value_kind: ScalarWriteValueKind::Boolean,
    },
    SafeWritableRow {
        row_id: "master.special_scale_factor",
        official_setting: "master.special_scale_factor",
        value_kind: ScalarWriteValueKind::Percent,
    },
    SafeWritableRow {
        row_id: "master.mfact",
        official_setting: "master.mfact",
        value_kind: ScalarWriteValueKind::Number,
    },
    SafeWritableRow {
        row_id: "master.new_on_top",
        official_setting: "master.new_on_top",
        value_kind: ScalarWriteValueKind::Boolean,
    },
    SafeWritableRow {
        row_id: "master.new_status",
        official_setting: "master.new_status",
        value_kind: ScalarWriteValueKind::FiniteChoice,
    },
    SafeWritableRow {
        row_id: "master.new_on_active",
        official_setting: "master.new_on_active",
        value_kind: ScalarWriteValueKind::FiniteChoice,
    },
    SafeWritableRow {
        row_id: "master.orientation",
        official_setting: "master.orientation",
        value_kind: ScalarWriteValueKind::FiniteChoice,
    },
    SafeWritableRow {
        row_id: "master.slave_count_for_center_master",
        official_setting: "master.slave_count_for_center_master",
        value_kind: ScalarWriteValueKind::Number,
    },
    SafeWritableRow {
        row_id: "master.center_ignores_reserved",
        official_setting: "master.center_ignores_reserved",
        value_kind: ScalarWriteValueKind::Boolean,
    },
    SafeWritableRow {
        row_id: "master.center_master_fallback",
        official_setting: "master.center_master_fallback",
        value_kind: ScalarWriteValueKind::FiniteChoice,
    },
    SafeWritableRow {
        row_id: "master.smart_resizing",
        official_setting: "master.smart_resizing",
        value_kind: ScalarWriteValueKind::Boolean,
    },
    SafeWritableRow {
        row_id: "master.drop_at_cursor",
        official_setting: "master.drop_at_cursor",
        value_kind: ScalarWriteValueKind::Boolean,
    },
    SafeWritableRow {
        row_id: "master.always_keep_position",
        official_setting: "master.always_keep_position",
        value_kind: ScalarWriteValueKind::Boolean,
    },
    SafeWritableRow {
        row_id: "scrolling.column_width",
        official_setting: "scrolling.column_width",
        value_kind: ScalarWriteValueKind::Number,
    },
    SafeWritableRow {
        row_id: "scrolling.focus_fit_method",
        official_setting: "scrolling.focus_fit_method",
        value_kind: ScalarWriteValueKind::FiniteChoice,
    },
    SafeWritableRow {
        row_id: "scrolling.follow_focus",
        official_setting: "scrolling.follow_focus",
        value_kind: ScalarWriteValueKind::Boolean,
    },
    SafeWritableRow {
        row_id: "scrolling.follow_min_visible",
        official_setting: "scrolling.follow_min_visible",
        value_kind: ScalarWriteValueKind::Number,
    },
    SafeWritableRow {
        row_id: "scrolling.explicit_column_widths",
        official_setting: "scrolling.explicit_column_widths",
        value_kind: ScalarWriteValueKind::CommaSeparatedFloatList,
    },
    SafeWritableRow {
        row_id: "scrolling.wrap_focus",
        official_setting: "scrolling.wrap_focus",
        value_kind: ScalarWriteValueKind::Boolean,
    },
    SafeWritableRow {
        row_id: "scrolling.wrap_swapcol",
        official_setting: "scrolling.wrap_swapcol",
        value_kind: ScalarWriteValueKind::Boolean,
    },
    SafeWritableRow {
        row_id: "scrolling.direction",
        official_setting: "scrolling.direction",
        value_kind: ScalarWriteValueKind::FiniteChoice,
    },
    SafeWritableRow {
        row_id: "scrolling.fullscreen_on_one_column",
        official_setting: "scrolling.fullscreen_on_one_column",
        value_kind: ScalarWriteValueKind::Boolean,
    },
    SafeWritableRow {
        row_id: "ecosystem.no_update_news",
        official_setting: "ecosystem.no_update_news",
        value_kind: ScalarWriteValueKind::Boolean,
    },
    SafeWritableRow {
        row_id: "ecosystem.no_donation_nag",
        official_setting: "ecosystem.no_donation_nag",
        value_kind: ScalarWriteValueKind::Boolean,
    },
    SafeWritableRow {
        row_id: "ecosystem.enforce_permissions",
        official_setting: "ecosystem.enforce_permissions",
        value_kind: ScalarWriteValueKind::Boolean,
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

pub fn finite_choice_options(row_id: &str) -> Option<&'static [FiniteChoiceOption]> {
    match row_id {
        "layout.selection" => Some(LAYOUT_SELECTION_CHOICES),
        "input.follow_mouse" => Some(INPUT_FOLLOW_MOUSE_CHOICES),
        "input.touchpad.tap_button_map" => Some(INPUT_TOUCHPAD_TAP_BUTTON_MAP_CHOICES),
        "master.new_status" => Some(MASTER_NEW_STATUS_CHOICES),
        "master.new_on_active" => Some(MASTER_NEW_ON_ACTIVE_CHOICES),
        "master.orientation" => Some(MASTER_ORIENTATION_CHOICES),
        "master.center_master_fallback" => Some(MASTER_CENTER_MASTER_FALLBACK_CHOICES),
        "scrolling.direction" => Some(SCROLLING_DIRECTION_CHOICES),
        "input.scroll_method" => Some(INPUT_SCROLL_METHOD_CHOICES),
        "general.resize_corner" => Some(GENERAL_RESIZE_CORNER_CHOICES),
        "input.focus_on_close" => Some(INPUT_FOCUS_ON_CLOSE_CHOICES),
        "input.float_switch_override_focus" => Some(INPUT_FLOAT_SWITCH_OVERRIDE_FOCUS_CHOICES),
        "input.off_window_axis_events" => Some(INPUT_OFF_WINDOW_AXIS_EVENTS_CHOICES),
        "input.emulate_discrete_scroll" => Some(INPUT_EMULATE_DISCRETE_SCROLL_CHOICES),
        "input.touchpad.drag_lock" => Some(INPUT_TOUCHPAD_DRAG_LOCK_CHOICES),
        "input.touchpad.drag_3fg" => Some(INPUT_TOUCHPAD_DRAG_3FG_CHOICES),
        "input.virtualkeyboard.share_states" => Some(INPUT_VIRTUALKEYBOARD_SHARE_STATES_CHOICES),
        "group.drag_into_group" => Some(GROUP_DRAG_INTO_GROUP_CHOICES),
        "dwindle.force_split" => Some(DWINDLE_FORCE_SPLIT_CHOICES),
        "dwindle.split_bias" => Some(DWINDLE_SPLIT_BIAS_CHOICES),
        "scrolling.focus_fit_method" => Some(SCROLLING_FOCUS_FIT_METHOD_CHOICES),
        "misc.vrr" => Some(MISC_VRR_CHOICES),
        "misc.on_focus_under_fullscreen" => Some(MISC_ON_FOCUS_UNDER_FULLSCREEN_CHOICES),
        _ => None,
    }
}

pub fn session_runtime_write_policy(row_id: &str) -> Option<SessionRuntimeWritePolicy> {
    match row_id {
        "misc.disable_autoreload" | "scrolling.fullscreen_on_one_column" => {
            Some(SessionRuntimeWritePolicy {
                scope: "persistent-needs-reload",
                apply_path: "persistent-config-write-with-backup-reread",
                reread_oracle: "file-reread",
                recovery_strategy: "backup-rollback",
                approval_gate: "explicit-warning-pending-user-reload",
                runtime_effect: "pending-user-reload",
                review_warning: "This setting is written to config only. Hyprland reload is not run, so the runtime effect is pending user reload.",
            })
        }
        "misc.allow_session_lock_restore" => Some(SessionRuntimeWritePolicy {
            scope: "startup-only",
            apply_path: "persistent-config-write-with-backup-reread",
            reread_oracle: "file-reread",
            recovery_strategy: "backup-rollback",
            approval_gate: "explicit-warning-pending-session-restart",
            runtime_effect: "pending-session-restart",
            review_warning: "This setting is written to config only. Hyprland reload is not run. It is startup/session sensitive, so the effect is pending a future Hyprland session start.",
        }),
        row_id if SESSION_RUNTIME_SENSITIVE_ROWS.contains(&row_id) => {
            Some(SessionRuntimeWritePolicy {
                scope: "persistent-config-only",
                apply_path: "persistent-config-write-with-backup-reread",
                reread_oracle: "file-reread",
                recovery_strategy: "backup-rollback",
                approval_gate: "normal-safe-write-review-with-session-runtime-note",
                runtime_effect: "file-reread-proof-only-runtime-not-mutated",
                review_warning: "This setting may affect session/runtime behavior, but this app only writes persistent config with backup and file reread verification. Hyprland reload is not run.",
            })
        }
        _ => None,
    }
}

pub fn high_risk_write_policy(row_id: &str) -> Option<HighRiskWritePolicy> {
    if !ECOSYSTEM_HIGH_RISK_WRITABLE_ROWS.contains(&row_id) {
        return None;
    }
    Some(HighRiskWritePolicy {
        recovery_bucket: "ecosystem-permission-policy",
        approval_gate: "advanced-opt-in-plus-policy-confirmation-plus-independent-dead-man-confirm-or-revert",
        watchdog_requirement: "production-capable watchdog plan must be armed before mutation; timeout restores the backup unless the user confirms the change",
        review_warning: "High-risk ecosystem/policy setting. Requires advanced opt-in and dead-man confirm-or-revert recovery; display/render, cursor/input, and debug/crash rows remain blocked.",
    })
}

pub fn is_verified_finite_choice_value(row_id: &str, value: &str) -> bool {
    let trimmed = value.trim();
    finite_choice_options(row_id)
        .map(|options| options.iter().any(|option| option.raw_value == trimmed))
        .unwrap_or(false)
}

pub fn finite_choice_label(row_id: &str, raw_value: &str) -> Option<&'static str> {
    let trimmed = raw_value.trim();
    finite_choice_options(row_id).and_then(|options| {
        options
            .iter()
            .find(|option| option.raw_value == trimmed)
            .map(|option| option.label)
    })
}

pub fn config_key_from_official_setting(setting: &str) -> String {
    setting.replace('.', ":")
}

pub fn value_kind_for_control(control_kind: &str, value_family: &str) -> ScalarWriteValueKind {
    match (control_kind, value_family) {
        ("toggle", "none") => ScalarWriteValueKind::Boolean,
        ("slider" | "number-input", "none") => ScalarWriteValueKind::Number,
        ("percent-slider", "none") => ScalarWriteValueKind::Percent,
        ("dropdown", "none") => ScalarWriteValueKind::FiniteChoice,
        (_, "none") => ScalarWriteValueKind::Unknown,
        _ => ScalarWriteValueKind::ComplexRaw,
    }
}
