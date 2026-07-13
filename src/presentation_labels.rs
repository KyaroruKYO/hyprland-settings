//! Friendly display labels for scalar rows, resolved through the
//! reference-matched presentation adoption (see
//! data/reports/hyprmod-full-presentation-adoption.v0.55.2.json for the
//! full matched/unmatched accounting). Labels are short factual setting
//! names; rows without a matched label keep the official metadata label.
//! The raw Hyprland setting key and the internal row id are unchanged —
//! this table is presentation-only and can never affect classification,
//! validation, or any save path.

/// One adopted display label. `reference_key` records the matched schema
/// key for provenance/auditing (never shown in the UI).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct RowDisplayLabel {
    pub row_id: &'static str,
    pub reference_key: &'static str,
    pub label: &'static str,
}

pub const ROW_DISPLAY_LABELS: &[RowDisplayLabel] = &[
    RowDisplayLabel {
        row_id: "animations.enabled",
        reference_key: "animations:enabled",
        label: "Enable animations",
    },
    RowDisplayLabel {
        row_id: "appearance.active_opacity",
        reference_key: "decoration:active_opacity",
        label: "Active opacity",
    },
    RowDisplayLabel {
        row_id: "appearance.blur.brightness",
        reference_key: "decoration:blur:brightness",
        label: "Blur brightness",
    },
    RowDisplayLabel {
        row_id: "appearance.blur.contrast",
        reference_key: "decoration:blur:contrast",
        label: "Blur contrast",
    },
    RowDisplayLabel {
        row_id: "appearance.blur.enabled",
        reference_key: "decoration:blur:enabled",
        label: "Enable blur",
    },
    RowDisplayLabel {
        row_id: "appearance.blur.ignore_opacity",
        reference_key: "decoration:blur:ignore_opacity",
        label: "Ignore opacity",
    },
    RowDisplayLabel {
        row_id: "appearance.blur.new_optimizations",
        reference_key: "decoration:blur:new_optimizations",
        label: "New optimizations",
    },
    RowDisplayLabel {
        row_id: "appearance.blur.noise",
        reference_key: "decoration:blur:noise",
        label: "Blur noise",
    },
    RowDisplayLabel {
        row_id: "appearance.blur.passes",
        reference_key: "decoration:blur:passes",
        label: "Blur passes",
    },
    RowDisplayLabel {
        row_id: "appearance.blur.popups",
        reference_key: "decoration:blur:popups",
        label: "Blur popups",
    },
    RowDisplayLabel {
        row_id: "appearance.blur.size",
        reference_key: "decoration:blur:size",
        label: "Blur size",
    },
    RowDisplayLabel {
        row_id: "appearance.blur.special",
        reference_key: "decoration:blur:special",
        label: "Blur special workspaces",
    },
    RowDisplayLabel {
        row_id: "appearance.blur.vibrancy",
        reference_key: "decoration:blur:vibrancy",
        label: "Blur vibrancy",
    },
    RowDisplayLabel {
        row_id: "appearance.blur.vibrancy_darkness",
        reference_key: "decoration:blur:vibrancy_darkness",
        label: "Blur vibrancy darkness",
    },
    RowDisplayLabel {
        row_id: "appearance.blur.xray",
        reference_key: "decoration:blur:xray",
        label: "X-Ray",
    },
    RowDisplayLabel {
        row_id: "appearance.border_size",
        reference_key: "general:border_size",
        label: "Border size",
    },
    RowDisplayLabel {
        row_id: "appearance.dim.around",
        reference_key: "decoration:dim_around",
        label: "Dim around",
    },
    RowDisplayLabel {
        row_id: "appearance.dim.inactive",
        reference_key: "decoration:dim_inactive",
        label: "Dim inactive",
    },
    RowDisplayLabel {
        row_id: "appearance.dim.special",
        reference_key: "decoration:dim_special",
        label: "Dim special",
    },
    RowDisplayLabel {
        row_id: "appearance.dim.strength",
        reference_key: "decoration:dim_strength",
        label: "Dim strength",
    },
    RowDisplayLabel {
        row_id: "appearance.fullscreen_opacity",
        reference_key: "decoration:fullscreen_opacity",
        label: "Fullscreen opacity",
    },
    RowDisplayLabel {
        row_id: "appearance.gaps_in",
        reference_key: "general:gaps_in",
        label: "Inner gaps",
    },
    RowDisplayLabel {
        row_id: "appearance.gaps_out",
        reference_key: "general:gaps_out",
        label: "Outer gaps",
    },
    RowDisplayLabel {
        row_id: "appearance.inactive_opacity",
        reference_key: "decoration:inactive_opacity",
        label: "Inactive opacity",
    },
    RowDisplayLabel {
        row_id: "appearance.rounding",
        reference_key: "decoration:rounding",
        label: "Corner rounding",
    },
    RowDisplayLabel {
        row_id: "appearance.rounding_power",
        reference_key: "decoration:rounding_power",
        label: "Rounding power",
    },
    RowDisplayLabel {
        row_id: "appearance.shadow.enabled",
        reference_key: "decoration:shadow:enabled",
        label: "Enable shadow",
    },
    RowDisplayLabel {
        row_id: "appearance.shadow.range",
        reference_key: "decoration:shadow:range",
        label: "Shadow range",
    },
    RowDisplayLabel {
        row_id: "appearance.shadow.render_power",
        reference_key: "decoration:shadow:render_power",
        label: "Shadow render power",
    },
    RowDisplayLabel {
        row_id: "appearance.shadow.scale",
        reference_key: "decoration:shadow:scale",
        label: "Shadow scale",
    },
    RowDisplayLabel {
        row_id: "cursor.enable_hyprcursor",
        reference_key: "cursor:enable_hyprcursor",
        label: "Enable Hyprcursor",
    },
    RowDisplayLabel {
        row_id: "cursor.hide_on_key_press",
        reference_key: "cursor:hide_on_key_press",
        label: "Hide on key press",
    },
    RowDisplayLabel {
        row_id: "cursor.hide_on_tablet",
        reference_key: "cursor:hide_on_tablet",
        label: "Hide on tablet",
    },
    RowDisplayLabel {
        row_id: "cursor.hide_on_touch",
        reference_key: "cursor:hide_on_touch",
        label: "Hide on touch",
    },
    RowDisplayLabel {
        row_id: "cursor.inactive_timeout",
        reference_key: "cursor:inactive_timeout",
        label: "Inactive timeout",
    },
    RowDisplayLabel {
        row_id: "cursor.no_hardware_cursors",
        reference_key: "cursor:no_hardware_cursors",
        label: "Hardware cursors",
    },
    RowDisplayLabel {
        row_id: "cursor.no_warps",
        reference_key: "cursor:no_warps",
        label: "Disable cursor warps",
    },
    RowDisplayLabel {
        row_id: "cursor.persistent_warps",
        reference_key: "cursor:persistent_warps",
        label: "Persistent warps",
    },
    RowDisplayLabel {
        row_id: "cursor.warp_on_change_workspace",
        reference_key: "cursor:warp_on_change_workspace",
        label: "Warp on workspace change",
    },
    RowDisplayLabel {
        row_id: "cursor.zoom_factor",
        reference_key: "cursor:zoom_factor",
        label: "Zoom factor",
    },
    RowDisplayLabel {
        row_id: "decoration.shadow.color",
        reference_key: "decoration:shadow:color",
        label: "Shadow color",
    },
    RowDisplayLabel {
        row_id: "decoration.shadow.color_inactive",
        reference_key: "decoration:shadow:color_inactive",
        label: "Inactive shadow color",
    },
    RowDisplayLabel {
        row_id: "decoration.shadow.offset",
        reference_key: "decoration:shadow:offset",
        label: "Shadow offset",
    },
    RowDisplayLabel {
        row_id: "dwindle.default_split_ratio",
        reference_key: "dwindle:default_split_ratio",
        label: "Default split ratio",
    },
    RowDisplayLabel {
        row_id: "dwindle.force_split",
        reference_key: "dwindle:force_split",
        label: "Force split direction",
    },
    RowDisplayLabel {
        row_id: "dwindle.permanent_direction_override",
        reference_key: "dwindle:permanent_direction_override",
        label: "Permanent direction override",
    },
    RowDisplayLabel {
        row_id: "dwindle.preserve_split",
        reference_key: "dwindle:preserve_split",
        label: "Preserve split",
    },
    RowDisplayLabel {
        row_id: "dwindle.smart_resizing",
        reference_key: "dwindle:smart_resizing",
        label: "Smart resizing",
    },
    RowDisplayLabel {
        row_id: "dwindle.smart_split",
        reference_key: "dwindle:smart_split",
        label: "Smart split",
    },
    RowDisplayLabel {
        row_id: "dwindle.special_scale_factor",
        reference_key: "dwindle:special_scale_factor",
        label: "Special workspace scale",
    },
    RowDisplayLabel {
        row_id: "dwindle.split_width_multiplier",
        reference_key: "dwindle:split_width_multiplier",
        label: "Split width multiplier",
    },
    RowDisplayLabel {
        row_id: "dwindle.use_active_for_splits",
        reference_key: "dwindle:use_active_for_splits",
        label: "Use active for splits",
    },
    RowDisplayLabel {
        row_id: "ecosystem.enforce_permissions",
        reference_key: "ecosystem:enforce_permissions",
        label: "Enforce permissions",
    },
    RowDisplayLabel {
        row_id: "ecosystem.no_donation_nag",
        reference_key: "ecosystem:no_donation_nag",
        label: "Disable donation nag",
    },
    RowDisplayLabel {
        row_id: "ecosystem.no_update_news",
        reference_key: "ecosystem:no_update_news",
        label: "Disable update news",
    },
    RowDisplayLabel {
        row_id: "general.allow_tearing",
        reference_key: "general:allow_tearing",
        label: "Allow tearing",
    },
    RowDisplayLabel {
        row_id: "general.col.active_border",
        reference_key: "general:col.active_border",
        label: "Active border color",
    },
    RowDisplayLabel {
        row_id: "general.col.inactive_border",
        reference_key: "general:col.inactive_border",
        label: "Inactive border color",
    },
    RowDisplayLabel {
        row_id: "general.extend_border_grab_area",
        reference_key: "general:extend_border_grab_area",
        label: "Extend border grab area",
    },
    RowDisplayLabel {
        row_id: "general.hover_icon_on_border",
        reference_key: "general:hover_icon_on_border",
        label: "Hover icon on border",
    },
    RowDisplayLabel {
        row_id: "general.resize_on_border",
        reference_key: "general:resize_on_border",
        label: "Resize on border",
    },
    RowDisplayLabel {
        row_id: "general.snap.border_overlap",
        reference_key: "general:snap:border_overlap",
        label: "Border overlap",
    },
    RowDisplayLabel {
        row_id: "general.snap.respect_gaps",
        reference_key: "general:snap:respect_gaps",
        label: "Respect gaps",
    },
    RowDisplayLabel {
        row_id: "gestures.workspace_swipe_cancel_ratio",
        reference_key: "gestures:workspace_swipe_cancel_ratio",
        label: "Cancel ratio",
    },
    RowDisplayLabel {
        row_id: "gestures.workspace_swipe_create_new",
        reference_key: "gestures:workspace_swipe_create_new",
        label: "Create new workspace",
    },
    RowDisplayLabel {
        row_id: "gestures.workspace_swipe_direction_lock",
        reference_key: "gestures:workspace_swipe_direction_lock",
        label: "Direction lock",
    },
    RowDisplayLabel {
        row_id: "gestures.workspace_swipe_distance",
        reference_key: "gestures:workspace_swipe_distance",
        label: "Swipe distance",
    },
    RowDisplayLabel {
        row_id: "gestures.workspace_swipe_forever",
        reference_key: "gestures:workspace_swipe_forever",
        label: "Swipe forever",
    },
    RowDisplayLabel {
        row_id: "gestures.workspace_swipe_invert",
        reference_key: "gestures:workspace_swipe_invert",
        label: "Invert direction",
    },
    RowDisplayLabel {
        row_id: "gestures.workspace_swipe_min_speed_to_force",
        reference_key: "gestures:workspace_swipe_min_speed_to_force",
        label: "Min speed to force",
    },
    RowDisplayLabel {
        row_id: "gestures.workspace_swipe_touch",
        reference_key: "gestures:workspace_swipe_touch",
        label: "Touch swipe",
    },
    RowDisplayLabel {
        row_id: "gestures.workspace_swipe_touch_invert",
        reference_key: "gestures:workspace_swipe_touch_invert",
        label: "Invert touch direction",
    },
    RowDisplayLabel {
        row_id: "gestures.workspace_swipe_use_r",
        reference_key: "gestures:workspace_swipe_use_r",
        label: "Use relative workspaces",
    },
    RowDisplayLabel {
        row_id: "input.accel_profile",
        reference_key: "input:accel_profile",
        label: "Acceleration profile",
    },
    RowDisplayLabel {
        row_id: "input.float_switch_override_focus",
        reference_key: "input:float_switch_override_focus",
        label: "Float switch override focus",
    },
    RowDisplayLabel {
        row_id: "input.follow_mouse",
        reference_key: "input:follow_mouse",
        label: "Follow mouse",
    },
    RowDisplayLabel {
        row_id: "input.kb_layout",
        reference_key: "input:kb_layout",
        label: "Keyboard layout",
    },
    RowDisplayLabel {
        row_id: "input.kb_options",
        reference_key: "input:kb_options",
        label: "Keyboard options",
    },
    RowDisplayLabel {
        row_id: "input.kb_variant",
        reference_key: "input:kb_variant",
        label: "Keyboard variant",
    },
    RowDisplayLabel {
        row_id: "input.left_handed",
        reference_key: "input:left_handed",
        label: "Left-handed mode",
    },
    RowDisplayLabel {
        row_id: "input.mouse_refocus",
        reference_key: "input:mouse_refocus",
        label: "Mouse refocus",
    },
    RowDisplayLabel {
        row_id: "input.natural_scroll",
        reference_key: "input:natural_scroll",
        label: "Natural scroll",
    },
    RowDisplayLabel {
        row_id: "input.numlock_by_default",
        reference_key: "input:numlock_by_default",
        label: "Numlock by default",
    },
    RowDisplayLabel {
        row_id: "input.pointer_sensitivity",
        reference_key: "input:sensitivity",
        label: "Mouse sensitivity",
    },
    RowDisplayLabel {
        row_id: "input.repeat_delay",
        reference_key: "input:repeat_delay",
        label: "Repeat delay",
    },
    RowDisplayLabel {
        row_id: "input.repeat_rate",
        reference_key: "input:repeat_rate",
        label: "Repeat rate",
    },
    RowDisplayLabel {
        row_id: "input.scroll_factor",
        reference_key: "input:scroll_factor",
        label: "Scroll factor",
    },
    RowDisplayLabel {
        row_id: "input.touchpad.clickfinger_behavior",
        reference_key: "input:touchpad:clickfinger_behavior",
        label: "Clickfinger behavior",
    },
    RowDisplayLabel {
        row_id: "input.touchpad.disable_while_typing",
        reference_key: "input:touchpad:disable_while_typing",
        label: "Disable while typing",
    },
    RowDisplayLabel {
        row_id: "input.touchpad.drag_lock",
        reference_key: "input:touchpad:drag_lock",
        label: "Drag lock",
    },
    RowDisplayLabel {
        row_id: "input.touchpad.middle_button_emulation",
        reference_key: "input:touchpad:middle_button_emulation",
        label: "Middle button emulation",
    },
    RowDisplayLabel {
        row_id: "input.touchpad.natural_scroll",
        reference_key: "input:touchpad:natural_scroll",
        label: "Natural scroll",
    },
    RowDisplayLabel {
        row_id: "input.touchpad.scroll_factor",
        reference_key: "input:touchpad:scroll_factor",
        label: "Touchpad scroll factor",
    },
    RowDisplayLabel {
        row_id: "input.touchpad.tap-to-click",
        reference_key: "input:touchpad:tap-to-click",
        label: "Tap to click",
    },
    RowDisplayLabel {
        row_id: "input.touchpad.tap_button_map",
        reference_key: "input:touchpad:tap_button_map",
        label: "Tap button map",
    },
    RowDisplayLabel {
        row_id: "layout.selection",
        reference_key: "general:layout",
        label: "Layout",
    },
    RowDisplayLabel {
        row_id: "master.allow_small_split",
        reference_key: "master:allow_small_split",
        label: "Allow small split",
    },
    RowDisplayLabel {
        row_id: "master.mfact",
        reference_key: "master:mfact",
        label: "Master factor",
    },
    RowDisplayLabel {
        row_id: "master.new_on_active",
        reference_key: "master:new_on_active",
        label: "New window placement",
    },
    RowDisplayLabel {
        row_id: "master.new_on_top",
        reference_key: "master:new_on_top",
        label: "New on top",
    },
    RowDisplayLabel {
        row_id: "master.new_status",
        reference_key: "master:new_status",
        label: "New window status",
    },
    RowDisplayLabel {
        row_id: "master.orientation",
        reference_key: "master:orientation",
        label: "Orientation",
    },
    RowDisplayLabel {
        row_id: "master.smart_resizing",
        reference_key: "master:smart_resizing",
        label: "Smart resizing",
    },
    RowDisplayLabel {
        row_id: "master.special_scale_factor",
        reference_key: "master:special_scale_factor",
        label: "Special workspace scale",
    },
    RowDisplayLabel {
        row_id: "misc.animate_manual_resizes",
        reference_key: "misc:animate_manual_resizes",
        label: "Animate manual resizes",
    },
    RowDisplayLabel {
        row_id: "misc.animate_mouse_windowdragging",
        reference_key: "misc:animate_mouse_windowdragging",
        label: "Animate mouse window dragging",
    },
    RowDisplayLabel {
        row_id: "misc.disable_autoreload",
        reference_key: "misc:disable_autoreload",
        label: "Disable autoreload",
    },
    RowDisplayLabel {
        row_id: "misc.disable_hyprland_logo",
        reference_key: "misc:disable_hyprland_logo",
        label: "Disable Hyprland logo",
    },
    RowDisplayLabel {
        row_id: "misc.disable_splash_rendering",
        reference_key: "misc:disable_splash_rendering",
        label: "Disable splash rendering",
    },
    RowDisplayLabel {
        row_id: "misc.focus_on_activate",
        reference_key: "misc:focus_on_activate",
        label: "Focus on activate",
    },
    RowDisplayLabel {
        row_id: "misc.force_default_wallpaper",
        reference_key: "misc:force_default_wallpaper",
        label: "Force default wallpaper",
    },
    RowDisplayLabel {
        row_id: "misc.key_press_enables_dpms",
        reference_key: "misc:key_press_enables_dpms",
        label: "Key press enables DPMS",
    },
    RowDisplayLabel {
        row_id: "misc.mouse_move_enables_dpms",
        reference_key: "misc:mouse_move_enables_dpms",
        label: "Mouse move enables DPMS",
    },
    RowDisplayLabel {
        row_id: "misc.vrr",
        reference_key: "misc:vrr",
        label: "Variable refresh rate",
    },
    RowDisplayLabel {
        row_id: "scrolling.column_width",
        reference_key: "scrolling:column_width",
        label: "Default column width",
    },
    RowDisplayLabel {
        row_id: "scrolling.direction",
        reference_key: "scrolling:direction",
        label: "Scroll direction",
    },
    RowDisplayLabel {
        row_id: "scrolling.explicit_column_widths",
        reference_key: "scrolling:explicit_column_widths",
        label: "Preset column widths",
    },
    RowDisplayLabel {
        row_id: "scrolling.focus_fit_method",
        reference_key: "scrolling:focus_fit_method",
        label: "Focus fit method",
    },
    RowDisplayLabel {
        row_id: "scrolling.follow_focus",
        reference_key: "scrolling:follow_focus",
        label: "Follow focus",
    },
    RowDisplayLabel {
        row_id: "scrolling.follow_min_visible",
        reference_key: "scrolling:follow_min_visible",
        label: "Minimum visible fraction",
    },
    RowDisplayLabel {
        row_id: "scrolling.fullscreen_on_one_column",
        reference_key: "scrolling:fullscreen_on_one_column",
        label: "Single column fullscreen",
    },
    RowDisplayLabel {
        row_id: "windows.snap.enabled",
        reference_key: "general:snap:enabled",
        label: "Enable snap",
    },
    RowDisplayLabel {
        row_id: "windows.snap.monitor_gap",
        reference_key: "general:snap:monitor_gap",
        label: "Monitor snap gap",
    },
    RowDisplayLabel {
        row_id: "windows.snap.window_gap",
        reference_key: "general:snap:window_gap",
        label: "Window snap gap",
    },
    RowDisplayLabel {
        row_id: "xwayland.enabled",
        reference_key: "xwayland:enabled",
        label: "Enable XWayland",
    },
    RowDisplayLabel {
        row_id: "xwayland.force_zero_scaling",
        reference_key: "xwayland:force_zero_scaling",
        label: "Force zero scaling",
    },
    RowDisplayLabel {
        row_id: "xwayland.use_nearest_neighbor",
        reference_key: "xwayland:use_nearest_neighbor",
        label: "Use nearest neighbor",
    },
];

/// The adopted display label for a row, if one was matched.
pub fn display_label_for_row(row_id: &str) -> Option<&'static str> {
    ROW_DISPLAY_LABELS
        .iter()
        .find(|entry| entry.row_id == row_id)
        .map(|entry| entry.label)
}
