# Remaining 105 High-Risk And Runtime Policy

Generated: 2026-06-05T07:55:50.789962+00:00

This document summarizes rows that passed temp-config validation but remain blocked by runtime/session/high-risk policy. No active config or runtime state was modified.

- High-risk rows requiring recovery design: 72
- Session/runtime-sensitive rows requiring policy: 16
- Recommended default: keep blocked until a dedicated design defines warning level, recovery path, and confirmation UX.

## Session/runtime-sensitive rows

- `appearance.fullscreen_opacity`: Define session/runtime-sensitive write policy and user confirmation/recovery behavior before enabling this technically valid setting.
- `appearance.blur.xray`: Define session/runtime-sensitive write policy and user confirmation/recovery behavior before enabling this technically valid setting.
- `general.allow_tearing`: Define session/runtime-sensitive write policy and user confirmation/recovery behavior before enabling this technically valid setting.
- `general.locale`: Define session/runtime-sensitive write policy and user confirmation/recovery behavior before enabling this technically valid setting.
- `misc.vrr`: Define session/runtime-sensitive write policy and user confirmation/recovery behavior before enabling this technically valid setting.
- `misc.mouse_move_enables_dpms`: Define session/runtime-sensitive write policy and user confirmation/recovery behavior before enabling this technically valid setting.
- `misc.key_press_enables_dpms`: Define session/runtime-sensitive write policy and user confirmation/recovery behavior before enabling this technically valid setting.
- `misc.disable_autoreload`: Define session/runtime-sensitive write policy and user confirmation/recovery behavior before enabling this technically valid setting.
- `misc.focus_on_activate`: Define session/runtime-sensitive write policy and user confirmation/recovery behavior before enabling this technically valid setting.
- `misc.allow_session_lock_restore`: Define session/runtime-sensitive write policy and user confirmation/recovery behavior before enabling this technically valid setting.
- `misc.session_lock_xray`: Define session/runtime-sensitive write policy and user confirmation/recovery behavior before enabling this technically valid setting.
- `misc.on_focus_under_fullscreen`: Define session/runtime-sensitive write policy and user confirmation/recovery behavior before enabling this technically valid setting.
- `misc.exit_window_retains_fullscreen`: Define session/runtime-sensitive write policy and user confirmation/recovery behavior before enabling this technically valid setting.
- `binds.movefocus_cycles_fullscreen`: Define session/runtime-sensitive write policy and user confirmation/recovery behavior before enabling this technically valid setting.
- `binds.allow_pin_fullscreen`: Define session/runtime-sensitive write policy and user confirmation/recovery behavior before enabling this technically valid setting.
- `scrolling.fullscreen_on_one_column`: Define session/runtime-sensitive write policy and user confirmation/recovery behavior before enabling this technically valid setting.

## High-risk rows

- `xwayland.enabled` (display_render_session_risk): Create a dedicated high-risk recovery/rollback design, decide warning/advanced-mode policy, then add validators and rerun temp-config proof before enablement.
- `xwayland.use_nearest_neighbor` (display_render_session_risk): Create a dedicated high-risk recovery/rollback design, decide warning/advanced-mode policy, then add validators and rerun temp-config proof before enablement.
- `xwayland.force_zero_scaling` (display_render_session_risk): Create a dedicated high-risk recovery/rollback design, decide warning/advanced-mode policy, then add validators and rerun temp-config proof before enablement.
- `xwayland.create_abstract_socket` (display_render_session_risk): Create a dedicated high-risk recovery/rollback design, decide warning/advanced-mode policy, then add validators and rerun temp-config proof before enablement.
- `opengl.nvidia_anti_flicker` (display_render_session_risk): Create a dedicated high-risk recovery/rollback design, decide warning/advanced-mode policy, then add validators and rerun temp-config proof before enablement.
- `render.direct_scanout` (display_render_session_risk): Create a dedicated high-risk recovery/rollback design, decide warning/advanced-mode policy, then add validators and rerun temp-config proof before enablement.
- `render.expand_undersized_textures` (display_render_session_risk): Create a dedicated high-risk recovery/rollback design, decide warning/advanced-mode policy, then add validators and rerun temp-config proof before enablement.
- `render.xp_mode` (display_render_session_risk): Create a dedicated high-risk recovery/rollback design, decide warning/advanced-mode policy, then add validators and rerun temp-config proof before enablement.
- `render.ctm_animation` (display_render_session_risk): Create a dedicated high-risk recovery/rollback design, decide warning/advanced-mode policy, then add validators and rerun temp-config proof before enablement.
- `render.cm_enabled` (display_render_session_risk): Create a dedicated high-risk recovery/rollback design, decide warning/advanced-mode policy, then add validators and rerun temp-config proof before enablement.
- `render.send_content_type` (display_render_session_risk): Create a dedicated high-risk recovery/rollback design, decide warning/advanced-mode policy, then add validators and rerun temp-config proof before enablement.
- `render.cm_auto_hdr` (display_render_session_risk): Create a dedicated high-risk recovery/rollback design, decide warning/advanced-mode policy, then add validators and rerun temp-config proof before enablement.
- `render.new_render_scheduling` (display_render_session_risk): Create a dedicated high-risk recovery/rollback design, decide warning/advanced-mode policy, then add validators and rerun temp-config proof before enablement.
- `render.non_shader_cm` (display_render_session_risk): Create a dedicated high-risk recovery/rollback design, decide warning/advanced-mode policy, then add validators and rerun temp-config proof before enablement.
- `render.cm_sdr_eotf` (display_render_session_risk): Create a dedicated high-risk recovery/rollback design, decide warning/advanced-mode policy, then add validators and rerun temp-config proof before enablement.
- `render.commit_timing_enabled` (display_render_session_risk): Create a dedicated high-risk recovery/rollback design, decide warning/advanced-mode policy, then add validators and rerun temp-config proof before enablement.
- `render.icc_vcgt_enabled` (display_render_session_risk): Create a dedicated high-risk recovery/rollback design, decide warning/advanced-mode policy, then add validators and rerun temp-config proof before enablement.
- `render.use_shader_blur_blend` (display_render_session_risk): Create a dedicated high-risk recovery/rollback design, decide warning/advanced-mode policy, then add validators and rerun temp-config proof before enablement.
- `render.use_fp16` (display_render_session_risk): Create a dedicated high-risk recovery/rollback design, decide warning/advanced-mode policy, then add validators and rerun temp-config proof before enablement.
- `render.keep_unmodified_copy` (display_render_session_risk): Create a dedicated high-risk recovery/rollback design, decide warning/advanced-mode policy, then add validators and rerun temp-config proof before enablement.
- `render.non_shader_cm_interop` (display_render_session_risk): Create a dedicated high-risk recovery/rollback design, decide warning/advanced-mode policy, then add validators and rerun temp-config proof before enablement.
- `render.fp16_sdr_tf` (display_render_session_risk): Create a dedicated high-risk recovery/rollback design, decide warning/advanced-mode policy, then add validators and rerun temp-config proof before enablement.
- `cursor.invisible` (cursor_input_risk): Create a dedicated high-risk recovery/rollback design, decide warning/advanced-mode policy, then add validators and rerun temp-config proof before enablement.
- `cursor.no_hardware_cursors` (cursor_input_risk): Create a dedicated high-risk recovery/rollback design, decide warning/advanced-mode policy, then add validators and rerun temp-config proof before enablement.
- `cursor.no_break_fs_vrr` (cursor_input_risk): Create a dedicated high-risk recovery/rollback design, decide warning/advanced-mode policy, then add validators and rerun temp-config proof before enablement.
- `cursor.min_refresh_rate` (cursor_input_risk): Create a dedicated high-risk recovery/rollback design, decide warning/advanced-mode policy, then add validators and rerun temp-config proof before enablement.
- `cursor.hotspot_padding` (cursor_input_risk): Create a dedicated high-risk recovery/rollback design, decide warning/advanced-mode policy, then add validators and rerun temp-config proof before enablement.
- `cursor.inactive_timeout` (cursor_input_risk): Create a dedicated high-risk recovery/rollback design, decide warning/advanced-mode policy, then add validators and rerun temp-config proof before enablement.
- `cursor.no_warps` (cursor_input_risk): Create a dedicated high-risk recovery/rollback design, decide warning/advanced-mode policy, then add validators and rerun temp-config proof before enablement.
- `cursor.persistent_warps` (cursor_input_risk): Create a dedicated high-risk recovery/rollback design, decide warning/advanced-mode policy, then add validators and rerun temp-config proof before enablement.
- `cursor.warp_on_change_workspace` (cursor_input_risk): Create a dedicated high-risk recovery/rollback design, decide warning/advanced-mode policy, then add validators and rerun temp-config proof before enablement.
- `cursor.warp_on_toggle_special` (cursor_input_risk): Create a dedicated high-risk recovery/rollback design, decide warning/advanced-mode policy, then add validators and rerun temp-config proof before enablement.
- `cursor.default_monitor` (cursor_input_risk): Create a dedicated high-risk recovery/rollback design, decide warning/advanced-mode policy, then add validators and rerun temp-config proof before enablement.
- `cursor.zoom_factor` (cursor_input_risk): Create a dedicated high-risk recovery/rollback design, decide warning/advanced-mode policy, then add validators and rerun temp-config proof before enablement.
- `cursor.zoom_rigid` (cursor_input_risk): Create a dedicated high-risk recovery/rollback design, decide warning/advanced-mode policy, then add validators and rerun temp-config proof before enablement.
- `cursor.zoom_disable_aa` (cursor_input_risk): Create a dedicated high-risk recovery/rollback design, decide warning/advanced-mode policy, then add validators and rerun temp-config proof before enablement.
- `cursor.zoom_detached_camera` (cursor_input_risk): Create a dedicated high-risk recovery/rollback design, decide warning/advanced-mode policy, then add validators and rerun temp-config proof before enablement.
- `cursor.enable_hyprcursor` (cursor_input_risk): Create a dedicated high-risk recovery/rollback design, decide warning/advanced-mode policy, then add validators and rerun temp-config proof before enablement.
- `cursor.hide_on_key_press` (cursor_input_risk): Create a dedicated high-risk recovery/rollback design, decide warning/advanced-mode policy, then add validators and rerun temp-config proof before enablement.
- `cursor.hide_on_touch` (cursor_input_risk): Create a dedicated high-risk recovery/rollback design, decide warning/advanced-mode policy, then add validators and rerun temp-config proof before enablement.
- `cursor.hide_on_tablet` (cursor_input_risk): Create a dedicated high-risk recovery/rollback design, decide warning/advanced-mode policy, then add validators and rerun temp-config proof before enablement.
- `cursor.use_cpu_buffer` (cursor_input_risk): Create a dedicated high-risk recovery/rollback design, decide warning/advanced-mode policy, then add validators and rerun temp-config proof before enablement.
- `cursor.sync_gsettings_theme` (cursor_input_risk): Create a dedicated high-risk recovery/rollback design, decide warning/advanced-mode policy, then add validators and rerun temp-config proof before enablement.
- `cursor.warp_back_after_non_mouse_input` (cursor_input_risk): Create a dedicated high-risk recovery/rollback design, decide warning/advanced-mode policy, then add validators and rerun temp-config proof before enablement.
- `ecosystem.no_update_news` (ecosystem_permission_risk): Create a dedicated high-risk recovery/rollback design, decide warning/advanced-mode policy, then add validators and rerun temp-config proof before enablement.
- `ecosystem.no_donation_nag` (ecosystem_permission_risk): Create a dedicated high-risk recovery/rollback design, decide warning/advanced-mode policy, then add validators and rerun temp-config proof before enablement.
- `ecosystem.enforce_permissions` (ecosystem_permission_risk): Create a dedicated high-risk recovery/rollback design, decide warning/advanced-mode policy, then add validators and rerun temp-config proof before enablement.
- `debug.overlay` (debug_crash_risk): Create a dedicated high-risk recovery/rollback design, decide warning/advanced-mode policy, then add validators and rerun temp-config proof before enablement.
- `debug.damage_blink` (debug_crash_risk): Create a dedicated high-risk recovery/rollback design, decide warning/advanced-mode policy, then add validators and rerun temp-config proof before enablement.
- `debug.gl_debugging` (debug_crash_risk): Create a dedicated high-risk recovery/rollback design, decide warning/advanced-mode policy, then add validators and rerun temp-config proof before enablement.
- `debug.disable_logs` (debug_crash_risk): Create a dedicated high-risk recovery/rollback design, decide warning/advanced-mode policy, then add validators and rerun temp-config proof before enablement.
- `debug.disable_time` (debug_crash_risk): Create a dedicated high-risk recovery/rollback design, decide warning/advanced-mode policy, then add validators and rerun temp-config proof before enablement.
- `debug.damage_tracking` (debug_crash_risk): Create a dedicated high-risk recovery/rollback design, decide warning/advanced-mode policy, then add validators and rerun temp-config proof before enablement.
- `debug.enable_stdout_logs` (debug_crash_risk): Create a dedicated high-risk recovery/rollback design, decide warning/advanced-mode policy, then add validators and rerun temp-config proof before enablement.
- `debug.manual_crash` (debug_crash_risk): Create a dedicated high-risk recovery/rollback design, decide warning/advanced-mode policy, then add validators and rerun temp-config proof before enablement.
- `debug.suppress_errors` (debug_crash_risk): Create a dedicated high-risk recovery/rollback design, decide warning/advanced-mode policy, then add validators and rerun temp-config proof before enablement.
- `debug.disable_scale_checks` (debug_crash_risk): Create a dedicated high-risk recovery/rollback design, decide warning/advanced-mode policy, then add validators and rerun temp-config proof before enablement.
- `debug.error_limit` (debug_crash_risk): Create a dedicated high-risk recovery/rollback design, decide warning/advanced-mode policy, then add validators and rerun temp-config proof before enablement.
- `debug.error_position` (debug_crash_risk): Create a dedicated high-risk recovery/rollback design, decide warning/advanced-mode policy, then add validators and rerun temp-config proof before enablement.
- `debug.colored_stdout_logs` (debug_crash_risk): Create a dedicated high-risk recovery/rollback design, decide warning/advanced-mode policy, then add validators and rerun temp-config proof before enablement.
- `debug.log_damage` (debug_crash_risk): Create a dedicated high-risk recovery/rollback design, decide warning/advanced-mode policy, then add validators and rerun temp-config proof before enablement.
- `debug.pass` (debug_crash_risk): Create a dedicated high-risk recovery/rollback design, decide warning/advanced-mode policy, then add validators and rerun temp-config proof before enablement.
- `debug.full_cm_proto` (debug_crash_risk): Create a dedicated high-risk recovery/rollback design, decide warning/advanced-mode policy, then add validators and rerun temp-config proof before enablement.
- `debug.ds_handle_same_buffer` (debug_crash_risk): Create a dedicated high-risk recovery/rollback design, decide warning/advanced-mode policy, then add validators and rerun temp-config proof before enablement.
- `debug.ds_handle_same_buffer_fifo` (debug_crash_risk): Create a dedicated high-risk recovery/rollback design, decide warning/advanced-mode policy, then add validators and rerun temp-config proof before enablement.
- `debug.fifo_pending_workaround` (debug_crash_risk): Create a dedicated high-risk recovery/rollback design, decide warning/advanced-mode policy, then add validators and rerun temp-config proof before enablement.
- `debug.render_solitary_wo_damage` (debug_crash_risk): Create a dedicated high-risk recovery/rollback design, decide warning/advanced-mode policy, then add validators and rerun temp-config proof before enablement.
- `debug.vfr` (debug_crash_risk): Create a dedicated high-risk recovery/rollback design, decide warning/advanced-mode policy, then add validators and rerun temp-config proof before enablement.
- `debug.invalidate_fp16` (debug_crash_risk): Create a dedicated high-risk recovery/rollback design, decide warning/advanced-mode policy, then add validators and rerun temp-config proof before enablement.
- `experimental.wp_cm_1_2` (display_render_session_risk): Create a dedicated high-risk recovery/rollback design, decide warning/advanced-mode policy, then add validators and rerun temp-config proof before enablement.
- `quirks.prefer_hdr` (display_render_session_risk): Create a dedicated high-risk recovery/rollback design, decide warning/advanced-mode policy, then add validators and rerun temp-config proof before enablement.
- `quirks.skip_non_kms_dmabuf_formats` (display_render_session_risk): Create a dedicated high-risk recovery/rollback design, decide warning/advanced-mode policy, then add validators and rerun temp-config proof before enablement.
