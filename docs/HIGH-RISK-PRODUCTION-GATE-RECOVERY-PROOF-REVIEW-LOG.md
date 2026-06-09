# High-risk Production Gate and Recovery Proof Review Log

## Sprint summary

- Rows analyzed: 63
- Rows enabled: 0
- Counts before: 341 readable / 278 writable / 63 blocked
- Counts after: 341 readable / 278 writable / 63 blocked
- Why this sprint did or did not enable rows: This sprint defines bucket-level production gate and recovery requirements. It does not implement the production gate runner, live/runtime proof, or explicit high-risk enablement approval required for write allowlist changes.

## Bucket: display/render

### Failure modes
- blank or unreadable display
- render pipeline corruption
- XWayland session disruption
- HDR/color-management mismatch
- restart-required or output-dependent behavior

### Production gate requirement
- Production gate required: yes
- Purpose: Prevent applying display/render mutations unless a persisted recovery plan, backup, out-of-band confirmation, and rollback path are prepared before mutation.

### Recovery requirement
- Model: display-render-persisted-dead-man-watchdog-with-backup-rollback-and-output-independent-confirmation
- Must not depend on: visible compositor output, screen remains readable, app UI visibility, Hyprland keybinds inside the affected compositor session

### Non-live proof possible
- persisted plan schema validation
- temp config backup creation
- temp config rollback/reread verification
- out-of-band token generation and wrong-token rejection
- live-target refusal while live execution is disabled

### Live/runtime proof required
- Required for runtime display safety; non-live proof cannot prove visible output remains usable.

### Rows covered
- xwayland.enabled
- xwayland.create_abstract_socket
- opengl.nvidia_anti_flicker
- render.direct_scanout
- render.expand_undersized_textures
- render.xp_mode
- render.ctm_animation
- render.cm_enabled
- render.send_content_type
- render.cm_auto_hdr
- render.new_render_scheduling
- render.non_shader_cm
- render.cm_sdr_eotf
- render.commit_timing_enabled
- render.icc_vcgt_enabled
- render.use_shader_blur_blend
- render.use_fp16
- render.keep_unmodified_copy
- render.non_shader_cm_interop
- render.fp16_sdr_tf
- experimental.wp_cm_1_2
- quirks.prefer_hdr
- quirks.skip_non_kms_dmabuf_formats

### Rows needing special handling
- none

### Next concrete action
- Build display/render persisted recovery plan and separate confirmation/timeout runner in temp-only proof mode before any enablement sprint.

## Bucket: cursor/input

### Failure modes
- invisible cursor
- unusable cursor movement
- input focus disruption
- hardware cursor failure
- monitor-specific cursor placement mismatch

### Production gate requirement
- Production gate required: yes
- Purpose: Prevent applying cursor/input mutations unless recovery can be confirmed or reverted without relying on pointer visibility or mouse input.

### Recovery requirement
- Model: cursor-input-persisted-dead-man-watchdog-with-keyboard-token-confirmation-and-pointer-independent-rollback
- Must not depend on: pointer visibility, mouse input, app UI, Hyprland keybinds, normal pointer focus or cursor warping behavior

### Non-live proof possible
- persisted plan schema validation
- temp config backup creation
- temp config rollback/reread verification
- keyboard/token confirmation model
- live-target refusal while live execution is disabled

### Live/runtime proof required
- Required for runtime input safety; non-live proof cannot prove pointer/input usability remains intact.

### Rows covered
- cursor.invisible
- cursor.no_hardware_cursors
- cursor.no_break_fs_vrr
- cursor.min_refresh_rate
- cursor.hotspot_padding
- cursor.inactive_timeout
- cursor.no_warps
- cursor.persistent_warps
- cursor.warp_on_change_workspace
- cursor.warp_on_toggle_special
- cursor.default_monitor
- cursor.zoom_factor
- cursor.zoom_rigid
- cursor.zoom_disable_aa
- cursor.zoom_detached_camera
- cursor.enable_hyprcursor
- cursor.use_cpu_buffer
- cursor.warp_back_after_non_mouse_input

### Rows needing special handling
- cursor.default_monitor

### Next concrete action
- Build cursor/input persisted recovery plan and keyboard-token confirmation proof before any cursor/input enablement batch.

## Bucket: debug/crash

### Failure modes
- intentional compositor crash
- debug path disruption
- logging/error overlay disruption
- damage tracking or render-debug instability

### Production gate requirement
- Production gate required: yes
- Purpose: Prevent applying debug/crash mutations unless recovery survives disruption of the compositor or debug path being changed.

### Recovery requirement
- Model: debug-crash-persisted-dead-man-watchdog-with-external-process-rollback
- Must not depend on: process that may be disrupted, active compositor health, debug/logging subsystem being modified, app UI running inside the affected session, Hyprland keybinds

### Non-live proof possible
- persisted plan schema validation
- temp config backup creation
- temp config rollback/reread verification
- external-process ownership model
- live-target refusal while live execution is disabled

### Live/runtime proof required
- Required for runtime crash/debug safety; non-live proof cannot prove the active compositor survives or recovers.

### Rows covered
- debug.overlay
- debug.damage_blink
- debug.gl_debugging
- debug.disable_logs
- debug.disable_time
- debug.damage_tracking
- debug.enable_stdout_logs
- debug.manual_crash
- debug.suppress_errors
- debug.disable_scale_checks
- debug.error_limit
- debug.error_position
- debug.colored_stdout_logs
- debug.log_damage
- debug.pass
- debug.full_cm_proto
- debug.ds_handle_same_buffer
- debug.ds_handle_same_buffer_fifo
- debug.fifo_pending_workaround
- debug.render_solitary_wo_damage
- debug.vfr
- debug.invalidate_fp16

### Rows needing special handling
- none

### Next concrete action
- Build debug/crash external recovery proof and explicit no-crash temp-only tests before any debug/crash enablement batch.

## Row-by-row classification

- Row: xwayland.enabled
- Bucket: display/render
- Pre-enablement proof status: complete-pre-enablement-proof-only
- Production gate needed: yes
- Recovery model needed: display-render-persisted-dead-man-watchdog-with-backup-rollback-and-output-independent-confirmation
- Can be proven non-live: non-live gate scaffolding yes; full enablement without live/runtime proof no
- Requires live/runtime proof: yes
- Requires explicit approval: yes
- Recommended next action: Build display/render persisted recovery plan and separate confirmation/timeout runner in temp-only proof mode before any enablement sprint.
- Exact remaining blocker: missing production gate implementation and acceptance/rejection tests; missing persisted recovery plan implementation for this bucket; missing out-of-band confirmation runner proof; missing production rollback proof against the future gated write path; live/runtime safety proof remains required or must be explicitly waived by future approval; explicit high-risk enablement approval is missing

- Row: xwayland.create_abstract_socket
- Bucket: display/render
- Pre-enablement proof status: complete-pre-enablement-proof-only
- Production gate needed: yes
- Recovery model needed: display-render-persisted-dead-man-watchdog-with-backup-rollback-and-output-independent-confirmation
- Can be proven non-live: non-live gate scaffolding yes; full enablement without live/runtime proof no
- Requires live/runtime proof: yes
- Requires explicit approval: yes
- Recommended next action: Build display/render persisted recovery plan and separate confirmation/timeout runner in temp-only proof mode before any enablement sprint.
- Exact remaining blocker: missing production gate implementation and acceptance/rejection tests; missing persisted recovery plan implementation for this bucket; missing out-of-band confirmation runner proof; missing production rollback proof against the future gated write path; live/runtime safety proof remains required or must be explicitly waived by future approval; explicit high-risk enablement approval is missing

- Row: opengl.nvidia_anti_flicker
- Bucket: display/render
- Pre-enablement proof status: complete-pre-enablement-proof-only
- Production gate needed: yes
- Recovery model needed: display-render-persisted-dead-man-watchdog-with-backup-rollback-and-output-independent-confirmation
- Can be proven non-live: non-live gate scaffolding yes; full enablement without live/runtime proof no
- Requires live/runtime proof: yes
- Requires explicit approval: yes
- Recommended next action: Build display/render persisted recovery plan and separate confirmation/timeout runner in temp-only proof mode before any enablement sprint.
- Exact remaining blocker: missing production gate implementation and acceptance/rejection tests; missing persisted recovery plan implementation for this bucket; missing out-of-band confirmation runner proof; missing production rollback proof against the future gated write path; live/runtime safety proof remains required or must be explicitly waived by future approval; explicit high-risk enablement approval is missing

- Row: render.direct_scanout
- Bucket: display/render
- Pre-enablement proof status: complete-pre-enablement-proof-only
- Production gate needed: yes
- Recovery model needed: display-render-persisted-dead-man-watchdog-with-backup-rollback-and-output-independent-confirmation
- Can be proven non-live: non-live gate scaffolding yes; full enablement without live/runtime proof no
- Requires live/runtime proof: yes
- Requires explicit approval: yes
- Recommended next action: Build display/render persisted recovery plan and separate confirmation/timeout runner in temp-only proof mode before any enablement sprint.
- Exact remaining blocker: missing production gate implementation and acceptance/rejection tests; missing persisted recovery plan implementation for this bucket; missing out-of-band confirmation runner proof; missing production rollback proof against the future gated write path; live/runtime safety proof remains required or must be explicitly waived by future approval; explicit high-risk enablement approval is missing

- Row: render.expand_undersized_textures
- Bucket: display/render
- Pre-enablement proof status: complete-pre-enablement-proof-only
- Production gate needed: yes
- Recovery model needed: display-render-persisted-dead-man-watchdog-with-backup-rollback-and-output-independent-confirmation
- Can be proven non-live: non-live gate scaffolding yes; full enablement without live/runtime proof no
- Requires live/runtime proof: yes
- Requires explicit approval: yes
- Recommended next action: Build display/render persisted recovery plan and separate confirmation/timeout runner in temp-only proof mode before any enablement sprint.
- Exact remaining blocker: missing production gate implementation and acceptance/rejection tests; missing persisted recovery plan implementation for this bucket; missing out-of-band confirmation runner proof; missing production rollback proof against the future gated write path; live/runtime safety proof remains required or must be explicitly waived by future approval; explicit high-risk enablement approval is missing

- Row: render.xp_mode
- Bucket: display/render
- Pre-enablement proof status: complete-pre-enablement-proof-only
- Production gate needed: yes
- Recovery model needed: display-render-persisted-dead-man-watchdog-with-backup-rollback-and-output-independent-confirmation
- Can be proven non-live: non-live gate scaffolding yes; full enablement without live/runtime proof no
- Requires live/runtime proof: yes
- Requires explicit approval: yes
- Recommended next action: Build display/render persisted recovery plan and separate confirmation/timeout runner in temp-only proof mode before any enablement sprint.
- Exact remaining blocker: missing production gate implementation and acceptance/rejection tests; missing persisted recovery plan implementation for this bucket; missing out-of-band confirmation runner proof; missing production rollback proof against the future gated write path; live/runtime safety proof remains required or must be explicitly waived by future approval; explicit high-risk enablement approval is missing

- Row: render.ctm_animation
- Bucket: display/render
- Pre-enablement proof status: complete-pre-enablement-proof-only
- Production gate needed: yes
- Recovery model needed: display-render-persisted-dead-man-watchdog-with-backup-rollback-and-output-independent-confirmation
- Can be proven non-live: non-live gate scaffolding yes; full enablement without live/runtime proof no
- Requires live/runtime proof: yes
- Requires explicit approval: yes
- Recommended next action: Build display/render persisted recovery plan and separate confirmation/timeout runner in temp-only proof mode before any enablement sprint.
- Exact remaining blocker: missing production gate implementation and acceptance/rejection tests; missing persisted recovery plan implementation for this bucket; missing out-of-band confirmation runner proof; missing production rollback proof against the future gated write path; live/runtime safety proof remains required or must be explicitly waived by future approval; explicit high-risk enablement approval is missing

- Row: render.cm_enabled
- Bucket: display/render
- Pre-enablement proof status: complete-pre-enablement-proof-only
- Production gate needed: yes
- Recovery model needed: display-render-persisted-dead-man-watchdog-with-backup-rollback-and-output-independent-confirmation
- Can be proven non-live: non-live gate scaffolding yes; full enablement without live/runtime proof no
- Requires live/runtime proof: yes
- Requires explicit approval: yes
- Recommended next action: Build display/render persisted recovery plan and separate confirmation/timeout runner in temp-only proof mode before any enablement sprint.
- Exact remaining blocker: missing production gate implementation and acceptance/rejection tests; missing persisted recovery plan implementation for this bucket; missing out-of-band confirmation runner proof; missing production rollback proof against the future gated write path; live/runtime safety proof remains required or must be explicitly waived by future approval; explicit high-risk enablement approval is missing

- Row: render.send_content_type
- Bucket: display/render
- Pre-enablement proof status: complete-pre-enablement-proof-only
- Production gate needed: yes
- Recovery model needed: display-render-persisted-dead-man-watchdog-with-backup-rollback-and-output-independent-confirmation
- Can be proven non-live: non-live gate scaffolding yes; full enablement without live/runtime proof no
- Requires live/runtime proof: yes
- Requires explicit approval: yes
- Recommended next action: Build display/render persisted recovery plan and separate confirmation/timeout runner in temp-only proof mode before any enablement sprint.
- Exact remaining blocker: missing production gate implementation and acceptance/rejection tests; missing persisted recovery plan implementation for this bucket; missing out-of-band confirmation runner proof; missing production rollback proof against the future gated write path; live/runtime safety proof remains required or must be explicitly waived by future approval; explicit high-risk enablement approval is missing

- Row: render.cm_auto_hdr
- Bucket: display/render
- Pre-enablement proof status: complete-pre-enablement-proof-only
- Production gate needed: yes
- Recovery model needed: display-render-persisted-dead-man-watchdog-with-backup-rollback-and-output-independent-confirmation
- Can be proven non-live: non-live gate scaffolding yes; full enablement without live/runtime proof no
- Requires live/runtime proof: yes
- Requires explicit approval: yes
- Recommended next action: Build display/render persisted recovery plan and separate confirmation/timeout runner in temp-only proof mode before any enablement sprint.
- Exact remaining blocker: missing production gate implementation and acceptance/rejection tests; missing persisted recovery plan implementation for this bucket; missing out-of-band confirmation runner proof; missing production rollback proof against the future gated write path; live/runtime safety proof remains required or must be explicitly waived by future approval; explicit high-risk enablement approval is missing

- Row: render.new_render_scheduling
- Bucket: display/render
- Pre-enablement proof status: complete-pre-enablement-proof-only
- Production gate needed: yes
- Recovery model needed: display-render-persisted-dead-man-watchdog-with-backup-rollback-and-output-independent-confirmation
- Can be proven non-live: non-live gate scaffolding yes; full enablement without live/runtime proof no
- Requires live/runtime proof: yes
- Requires explicit approval: yes
- Recommended next action: Build display/render persisted recovery plan and separate confirmation/timeout runner in temp-only proof mode before any enablement sprint.
- Exact remaining blocker: missing production gate implementation and acceptance/rejection tests; missing persisted recovery plan implementation for this bucket; missing out-of-band confirmation runner proof; missing production rollback proof against the future gated write path; live/runtime safety proof remains required or must be explicitly waived by future approval; explicit high-risk enablement approval is missing

- Row: render.non_shader_cm
- Bucket: display/render
- Pre-enablement proof status: complete-pre-enablement-proof-only
- Production gate needed: yes
- Recovery model needed: display-render-persisted-dead-man-watchdog-with-backup-rollback-and-output-independent-confirmation
- Can be proven non-live: non-live gate scaffolding yes; full enablement without live/runtime proof no
- Requires live/runtime proof: yes
- Requires explicit approval: yes
- Recommended next action: Build display/render persisted recovery plan and separate confirmation/timeout runner in temp-only proof mode before any enablement sprint.
- Exact remaining blocker: missing production gate implementation and acceptance/rejection tests; missing persisted recovery plan implementation for this bucket; missing out-of-band confirmation runner proof; missing production rollback proof against the future gated write path; live/runtime safety proof remains required or must be explicitly waived by future approval; explicit high-risk enablement approval is missing

- Row: render.cm_sdr_eotf
- Bucket: display/render
- Pre-enablement proof status: complete-pre-enablement-proof-only
- Production gate needed: yes
- Recovery model needed: display-render-persisted-dead-man-watchdog-with-backup-rollback-and-output-independent-confirmation
- Can be proven non-live: non-live gate scaffolding yes; full enablement without live/runtime proof no
- Requires live/runtime proof: yes
- Requires explicit approval: yes
- Recommended next action: Build display/render persisted recovery plan and separate confirmation/timeout runner in temp-only proof mode before any enablement sprint.
- Exact remaining blocker: missing production gate implementation and acceptance/rejection tests; missing persisted recovery plan implementation for this bucket; missing out-of-band confirmation runner proof; missing production rollback proof against the future gated write path; live/runtime safety proof remains required or must be explicitly waived by future approval; explicit high-risk enablement approval is missing

- Row: render.commit_timing_enabled
- Bucket: display/render
- Pre-enablement proof status: complete-pre-enablement-proof-only
- Production gate needed: yes
- Recovery model needed: display-render-persisted-dead-man-watchdog-with-backup-rollback-and-output-independent-confirmation
- Can be proven non-live: non-live gate scaffolding yes; full enablement without live/runtime proof no
- Requires live/runtime proof: yes
- Requires explicit approval: yes
- Recommended next action: Build display/render persisted recovery plan and separate confirmation/timeout runner in temp-only proof mode before any enablement sprint.
- Exact remaining blocker: missing production gate implementation and acceptance/rejection tests; missing persisted recovery plan implementation for this bucket; missing out-of-band confirmation runner proof; missing production rollback proof against the future gated write path; live/runtime safety proof remains required or must be explicitly waived by future approval; explicit high-risk enablement approval is missing

- Row: render.icc_vcgt_enabled
- Bucket: display/render
- Pre-enablement proof status: complete-pre-enablement-proof-only
- Production gate needed: yes
- Recovery model needed: display-render-persisted-dead-man-watchdog-with-backup-rollback-and-output-independent-confirmation
- Can be proven non-live: non-live gate scaffolding yes; full enablement without live/runtime proof no
- Requires live/runtime proof: yes
- Requires explicit approval: yes
- Recommended next action: Build display/render persisted recovery plan and separate confirmation/timeout runner in temp-only proof mode before any enablement sprint.
- Exact remaining blocker: missing production gate implementation and acceptance/rejection tests; missing persisted recovery plan implementation for this bucket; missing out-of-band confirmation runner proof; missing production rollback proof against the future gated write path; live/runtime safety proof remains required or must be explicitly waived by future approval; explicit high-risk enablement approval is missing

- Row: render.use_shader_blur_blend
- Bucket: display/render
- Pre-enablement proof status: complete-pre-enablement-proof-only
- Production gate needed: yes
- Recovery model needed: display-render-persisted-dead-man-watchdog-with-backup-rollback-and-output-independent-confirmation
- Can be proven non-live: non-live gate scaffolding yes; full enablement without live/runtime proof no
- Requires live/runtime proof: yes
- Requires explicit approval: yes
- Recommended next action: Build display/render persisted recovery plan and separate confirmation/timeout runner in temp-only proof mode before any enablement sprint.
- Exact remaining blocker: missing production gate implementation and acceptance/rejection tests; missing persisted recovery plan implementation for this bucket; missing out-of-band confirmation runner proof; missing production rollback proof against the future gated write path; live/runtime safety proof remains required or must be explicitly waived by future approval; explicit high-risk enablement approval is missing

- Row: render.use_fp16
- Bucket: display/render
- Pre-enablement proof status: complete-pre-enablement-proof-only
- Production gate needed: yes
- Recovery model needed: display-render-persisted-dead-man-watchdog-with-backup-rollback-and-output-independent-confirmation
- Can be proven non-live: non-live gate scaffolding yes; full enablement without live/runtime proof no
- Requires live/runtime proof: yes
- Requires explicit approval: yes
- Recommended next action: Build display/render persisted recovery plan and separate confirmation/timeout runner in temp-only proof mode before any enablement sprint.
- Exact remaining blocker: missing production gate implementation and acceptance/rejection tests; missing persisted recovery plan implementation for this bucket; missing out-of-band confirmation runner proof; missing production rollback proof against the future gated write path; live/runtime safety proof remains required or must be explicitly waived by future approval; explicit high-risk enablement approval is missing

- Row: render.keep_unmodified_copy
- Bucket: display/render
- Pre-enablement proof status: complete-pre-enablement-proof-only
- Production gate needed: yes
- Recovery model needed: display-render-persisted-dead-man-watchdog-with-backup-rollback-and-output-independent-confirmation
- Can be proven non-live: non-live gate scaffolding yes; full enablement without live/runtime proof no
- Requires live/runtime proof: yes
- Requires explicit approval: yes
- Recommended next action: Build display/render persisted recovery plan and separate confirmation/timeout runner in temp-only proof mode before any enablement sprint.
- Exact remaining blocker: missing production gate implementation and acceptance/rejection tests; missing persisted recovery plan implementation for this bucket; missing out-of-band confirmation runner proof; missing production rollback proof against the future gated write path; live/runtime safety proof remains required or must be explicitly waived by future approval; explicit high-risk enablement approval is missing

- Row: render.non_shader_cm_interop
- Bucket: display/render
- Pre-enablement proof status: complete-pre-enablement-proof-only
- Production gate needed: yes
- Recovery model needed: display-render-persisted-dead-man-watchdog-with-backup-rollback-and-output-independent-confirmation
- Can be proven non-live: non-live gate scaffolding yes; full enablement without live/runtime proof no
- Requires live/runtime proof: yes
- Requires explicit approval: yes
- Recommended next action: Build display/render persisted recovery plan and separate confirmation/timeout runner in temp-only proof mode before any enablement sprint.
- Exact remaining blocker: missing production gate implementation and acceptance/rejection tests; missing persisted recovery plan implementation for this bucket; missing out-of-band confirmation runner proof; missing production rollback proof against the future gated write path; live/runtime safety proof remains required or must be explicitly waived by future approval; explicit high-risk enablement approval is missing

- Row: render.fp16_sdr_tf
- Bucket: display/render
- Pre-enablement proof status: complete-pre-enablement-proof-only
- Production gate needed: yes
- Recovery model needed: display-render-persisted-dead-man-watchdog-with-backup-rollback-and-output-independent-confirmation
- Can be proven non-live: non-live gate scaffolding yes; full enablement without live/runtime proof no
- Requires live/runtime proof: yes
- Requires explicit approval: yes
- Recommended next action: Build display/render persisted recovery plan and separate confirmation/timeout runner in temp-only proof mode before any enablement sprint.
- Exact remaining blocker: missing production gate implementation and acceptance/rejection tests; missing persisted recovery plan implementation for this bucket; missing out-of-band confirmation runner proof; missing production rollback proof against the future gated write path; live/runtime safety proof remains required or must be explicitly waived by future approval; explicit high-risk enablement approval is missing

- Row: experimental.wp_cm_1_2
- Bucket: display/render
- Pre-enablement proof status: complete-pre-enablement-proof-only
- Production gate needed: yes
- Recovery model needed: display-render-persisted-dead-man-watchdog-with-backup-rollback-and-output-independent-confirmation
- Can be proven non-live: non-live gate scaffolding yes; full enablement without live/runtime proof no
- Requires live/runtime proof: yes
- Requires explicit approval: yes
- Recommended next action: Build display/render persisted recovery plan and separate confirmation/timeout runner in temp-only proof mode before any enablement sprint.
- Exact remaining blocker: missing production gate implementation and acceptance/rejection tests; missing persisted recovery plan implementation for this bucket; missing out-of-band confirmation runner proof; missing production rollback proof against the future gated write path; live/runtime safety proof remains required or must be explicitly waived by future approval; explicit high-risk enablement approval is missing

- Row: quirks.prefer_hdr
- Bucket: display/render
- Pre-enablement proof status: complete-pre-enablement-proof-only
- Production gate needed: yes
- Recovery model needed: display-render-persisted-dead-man-watchdog-with-backup-rollback-and-output-independent-confirmation
- Can be proven non-live: non-live gate scaffolding yes; full enablement without live/runtime proof no
- Requires live/runtime proof: yes
- Requires explicit approval: yes
- Recommended next action: Build display/render persisted recovery plan and separate confirmation/timeout runner in temp-only proof mode before any enablement sprint.
- Exact remaining blocker: missing production gate implementation and acceptance/rejection tests; missing persisted recovery plan implementation for this bucket; missing out-of-band confirmation runner proof; missing production rollback proof against the future gated write path; live/runtime safety proof remains required or must be explicitly waived by future approval; explicit high-risk enablement approval is missing

- Row: quirks.skip_non_kms_dmabuf_formats
- Bucket: display/render
- Pre-enablement proof status: complete-pre-enablement-proof-only
- Production gate needed: yes
- Recovery model needed: display-render-persisted-dead-man-watchdog-with-backup-rollback-and-output-independent-confirmation
- Can be proven non-live: non-live gate scaffolding yes; full enablement without live/runtime proof no
- Requires live/runtime proof: yes
- Requires explicit approval: yes
- Recommended next action: Build display/render persisted recovery plan and separate confirmation/timeout runner in temp-only proof mode before any enablement sprint.
- Exact remaining blocker: missing production gate implementation and acceptance/rejection tests; missing persisted recovery plan implementation for this bucket; missing out-of-band confirmation runner proof; missing production rollback proof against the future gated write path; live/runtime safety proof remains required or must be explicitly waived by future approval; explicit high-risk enablement approval is missing

- Row: cursor.invisible
- Bucket: cursor/input
- Pre-enablement proof status: complete-pre-enablement-proof-only
- Production gate needed: yes
- Recovery model needed: cursor-input-persisted-dead-man-watchdog-with-keyboard-token-confirmation-and-pointer-independent-rollback
- Can be proven non-live: non-live gate scaffolding yes; full enablement without live/runtime proof no
- Requires live/runtime proof: yes
- Requires explicit approval: yes
- Recommended next action: Build cursor/input persisted recovery plan and keyboard-token confirmation proof before any cursor/input enablement batch.
- Exact remaining blocker: missing production gate implementation and acceptance/rejection tests; missing persisted recovery plan implementation for this bucket; missing out-of-band confirmation runner proof; missing production rollback proof against the future gated write path; live/runtime safety proof remains required or must be explicitly waived by future approval; explicit high-risk enablement approval is missing

- Row: cursor.no_hardware_cursors
- Bucket: cursor/input
- Pre-enablement proof status: complete-pre-enablement-proof-only
- Production gate needed: yes
- Recovery model needed: cursor-input-persisted-dead-man-watchdog-with-keyboard-token-confirmation-and-pointer-independent-rollback
- Can be proven non-live: non-live gate scaffolding yes; full enablement without live/runtime proof no
- Requires live/runtime proof: yes
- Requires explicit approval: yes
- Recommended next action: Build cursor/input persisted recovery plan and keyboard-token confirmation proof before any cursor/input enablement batch.
- Exact remaining blocker: missing production gate implementation and acceptance/rejection tests; missing persisted recovery plan implementation for this bucket; missing out-of-band confirmation runner proof; missing production rollback proof against the future gated write path; live/runtime safety proof remains required or must be explicitly waived by future approval; explicit high-risk enablement approval is missing

- Row: cursor.no_break_fs_vrr
- Bucket: cursor/input
- Pre-enablement proof status: complete-pre-enablement-proof-only
- Production gate needed: yes
- Recovery model needed: cursor-input-persisted-dead-man-watchdog-with-keyboard-token-confirmation-and-pointer-independent-rollback
- Can be proven non-live: non-live gate scaffolding yes; full enablement without live/runtime proof no
- Requires live/runtime proof: yes
- Requires explicit approval: yes
- Recommended next action: Build cursor/input persisted recovery plan and keyboard-token confirmation proof before any cursor/input enablement batch.
- Exact remaining blocker: missing production gate implementation and acceptance/rejection tests; missing persisted recovery plan implementation for this bucket; missing out-of-band confirmation runner proof; missing production rollback proof against the future gated write path; live/runtime safety proof remains required or must be explicitly waived by future approval; explicit high-risk enablement approval is missing

- Row: cursor.min_refresh_rate
- Bucket: cursor/input
- Pre-enablement proof status: complete-pre-enablement-proof-only
- Production gate needed: yes
- Recovery model needed: cursor-input-persisted-dead-man-watchdog-with-keyboard-token-confirmation-and-pointer-independent-rollback
- Can be proven non-live: non-live gate scaffolding yes; full enablement without live/runtime proof no
- Requires live/runtime proof: yes
- Requires explicit approval: yes
- Recommended next action: Build cursor/input persisted recovery plan and keyboard-token confirmation proof before any cursor/input enablement batch.
- Exact remaining blocker: missing production gate implementation and acceptance/rejection tests; missing persisted recovery plan implementation for this bucket; missing out-of-band confirmation runner proof; missing production rollback proof against the future gated write path; live/runtime safety proof remains required or must be explicitly waived by future approval; explicit high-risk enablement approval is missing

- Row: cursor.hotspot_padding
- Bucket: cursor/input
- Pre-enablement proof status: complete-pre-enablement-proof-only
- Production gate needed: yes
- Recovery model needed: cursor-input-persisted-dead-man-watchdog-with-keyboard-token-confirmation-and-pointer-independent-rollback
- Can be proven non-live: non-live gate scaffolding yes; full enablement without live/runtime proof no
- Requires live/runtime proof: yes
- Requires explicit approval: yes
- Recommended next action: Build cursor/input persisted recovery plan and keyboard-token confirmation proof before any cursor/input enablement batch.
- Exact remaining blocker: missing production gate implementation and acceptance/rejection tests; missing persisted recovery plan implementation for this bucket; missing out-of-band confirmation runner proof; missing production rollback proof against the future gated write path; live/runtime safety proof remains required or must be explicitly waived by future approval; explicit high-risk enablement approval is missing

- Row: cursor.inactive_timeout
- Bucket: cursor/input
- Pre-enablement proof status: complete-pre-enablement-proof-only
- Production gate needed: yes
- Recovery model needed: cursor-input-persisted-dead-man-watchdog-with-keyboard-token-confirmation-and-pointer-independent-rollback
- Can be proven non-live: non-live gate scaffolding yes; full enablement without live/runtime proof no
- Requires live/runtime proof: yes
- Requires explicit approval: yes
- Recommended next action: Build cursor/input persisted recovery plan and keyboard-token confirmation proof before any cursor/input enablement batch.
- Exact remaining blocker: missing production gate implementation and acceptance/rejection tests; missing persisted recovery plan implementation for this bucket; missing out-of-band confirmation runner proof; missing production rollback proof against the future gated write path; live/runtime safety proof remains required or must be explicitly waived by future approval; explicit high-risk enablement approval is missing

- Row: cursor.no_warps
- Bucket: cursor/input
- Pre-enablement proof status: complete-pre-enablement-proof-only
- Production gate needed: yes
- Recovery model needed: cursor-input-persisted-dead-man-watchdog-with-keyboard-token-confirmation-and-pointer-independent-rollback
- Can be proven non-live: non-live gate scaffolding yes; full enablement without live/runtime proof no
- Requires live/runtime proof: yes
- Requires explicit approval: yes
- Recommended next action: Build cursor/input persisted recovery plan and keyboard-token confirmation proof before any cursor/input enablement batch.
- Exact remaining blocker: missing production gate implementation and acceptance/rejection tests; missing persisted recovery plan implementation for this bucket; missing out-of-band confirmation runner proof; missing production rollback proof against the future gated write path; live/runtime safety proof remains required or must be explicitly waived by future approval; explicit high-risk enablement approval is missing

- Row: cursor.persistent_warps
- Bucket: cursor/input
- Pre-enablement proof status: complete-pre-enablement-proof-only
- Production gate needed: yes
- Recovery model needed: cursor-input-persisted-dead-man-watchdog-with-keyboard-token-confirmation-and-pointer-independent-rollback
- Can be proven non-live: non-live gate scaffolding yes; full enablement without live/runtime proof no
- Requires live/runtime proof: yes
- Requires explicit approval: yes
- Recommended next action: Build cursor/input persisted recovery plan and keyboard-token confirmation proof before any cursor/input enablement batch.
- Exact remaining blocker: missing production gate implementation and acceptance/rejection tests; missing persisted recovery plan implementation for this bucket; missing out-of-band confirmation runner proof; missing production rollback proof against the future gated write path; live/runtime safety proof remains required or must be explicitly waived by future approval; explicit high-risk enablement approval is missing

- Row: cursor.warp_on_change_workspace
- Bucket: cursor/input
- Pre-enablement proof status: complete-pre-enablement-proof-only
- Production gate needed: yes
- Recovery model needed: cursor-input-persisted-dead-man-watchdog-with-keyboard-token-confirmation-and-pointer-independent-rollback
- Can be proven non-live: non-live gate scaffolding yes; full enablement without live/runtime proof no
- Requires live/runtime proof: yes
- Requires explicit approval: yes
- Recommended next action: Build cursor/input persisted recovery plan and keyboard-token confirmation proof before any cursor/input enablement batch.
- Exact remaining blocker: missing production gate implementation and acceptance/rejection tests; missing persisted recovery plan implementation for this bucket; missing out-of-band confirmation runner proof; missing production rollback proof against the future gated write path; live/runtime safety proof remains required or must be explicitly waived by future approval; explicit high-risk enablement approval is missing

- Row: cursor.warp_on_toggle_special
- Bucket: cursor/input
- Pre-enablement proof status: complete-pre-enablement-proof-only
- Production gate needed: yes
- Recovery model needed: cursor-input-persisted-dead-man-watchdog-with-keyboard-token-confirmation-and-pointer-independent-rollback
- Can be proven non-live: non-live gate scaffolding yes; full enablement without live/runtime proof no
- Requires live/runtime proof: yes
- Requires explicit approval: yes
- Recommended next action: Build cursor/input persisted recovery plan and keyboard-token confirmation proof before any cursor/input enablement batch.
- Exact remaining blocker: missing production gate implementation and acceptance/rejection tests; missing persisted recovery plan implementation for this bucket; missing out-of-band confirmation runner proof; missing production rollback proof against the future gated write path; live/runtime safety proof remains required or must be explicitly waived by future approval; explicit high-risk enablement approval is missing

- Row: cursor.default_monitor
- Bucket: cursor/input
- Pre-enablement proof status: complete-pre-enablement-proof-only
- Production gate needed: yes
- Recovery model needed: cursor-input-persisted-dead-man-watchdog-with-keyboard-token-confirmation-and-pointer-independent-rollback
- Can be proven non-live: non-live gate scaffolding yes; full enablement without live/runtime proof no
- Requires live/runtime proof: yes
- Requires explicit approval: yes
- Recommended next action: First create a runtime monitor-name source oracle, then apply the cursor/input gate and recovery model.
- Exact remaining blocker: missing production gate implementation and acceptance/rejection tests; missing persisted recovery plan implementation for this bucket; missing out-of-band confirmation runner proof; missing production rollback proof against the future gated write path; live/runtime safety proof remains required or must be explicitly waived by future approval; explicit high-risk enablement approval is missing; runtime monitor-name allowlist/readback proof is missing

- Row: cursor.zoom_factor
- Bucket: cursor/input
- Pre-enablement proof status: complete-pre-enablement-proof-only
- Production gate needed: yes
- Recovery model needed: cursor-input-persisted-dead-man-watchdog-with-keyboard-token-confirmation-and-pointer-independent-rollback
- Can be proven non-live: non-live gate scaffolding yes; full enablement without live/runtime proof no
- Requires live/runtime proof: yes
- Requires explicit approval: yes
- Recommended next action: Build cursor/input persisted recovery plan and keyboard-token confirmation proof before any cursor/input enablement batch.
- Exact remaining blocker: missing production gate implementation and acceptance/rejection tests; missing persisted recovery plan implementation for this bucket; missing out-of-band confirmation runner proof; missing production rollback proof against the future gated write path; live/runtime safety proof remains required or must be explicitly waived by future approval; explicit high-risk enablement approval is missing

- Row: cursor.zoom_rigid
- Bucket: cursor/input
- Pre-enablement proof status: complete-pre-enablement-proof-only
- Production gate needed: yes
- Recovery model needed: cursor-input-persisted-dead-man-watchdog-with-keyboard-token-confirmation-and-pointer-independent-rollback
- Can be proven non-live: non-live gate scaffolding yes; full enablement without live/runtime proof no
- Requires live/runtime proof: yes
- Requires explicit approval: yes
- Recommended next action: Build cursor/input persisted recovery plan and keyboard-token confirmation proof before any cursor/input enablement batch.
- Exact remaining blocker: missing production gate implementation and acceptance/rejection tests; missing persisted recovery plan implementation for this bucket; missing out-of-band confirmation runner proof; missing production rollback proof against the future gated write path; live/runtime safety proof remains required or must be explicitly waived by future approval; explicit high-risk enablement approval is missing

- Row: cursor.zoom_disable_aa
- Bucket: cursor/input
- Pre-enablement proof status: complete-pre-enablement-proof-only
- Production gate needed: yes
- Recovery model needed: cursor-input-persisted-dead-man-watchdog-with-keyboard-token-confirmation-and-pointer-independent-rollback
- Can be proven non-live: non-live gate scaffolding yes; full enablement without live/runtime proof no
- Requires live/runtime proof: yes
- Requires explicit approval: yes
- Recommended next action: Build cursor/input persisted recovery plan and keyboard-token confirmation proof before any cursor/input enablement batch.
- Exact remaining blocker: missing production gate implementation and acceptance/rejection tests; missing persisted recovery plan implementation for this bucket; missing out-of-band confirmation runner proof; missing production rollback proof against the future gated write path; live/runtime safety proof remains required or must be explicitly waived by future approval; explicit high-risk enablement approval is missing

- Row: cursor.zoom_detached_camera
- Bucket: cursor/input
- Pre-enablement proof status: complete-pre-enablement-proof-only
- Production gate needed: yes
- Recovery model needed: cursor-input-persisted-dead-man-watchdog-with-keyboard-token-confirmation-and-pointer-independent-rollback
- Can be proven non-live: non-live gate scaffolding yes; full enablement without live/runtime proof no
- Requires live/runtime proof: yes
- Requires explicit approval: yes
- Recommended next action: Build cursor/input persisted recovery plan and keyboard-token confirmation proof before any cursor/input enablement batch.
- Exact remaining blocker: missing production gate implementation and acceptance/rejection tests; missing persisted recovery plan implementation for this bucket; missing out-of-band confirmation runner proof; missing production rollback proof against the future gated write path; live/runtime safety proof remains required or must be explicitly waived by future approval; explicit high-risk enablement approval is missing

- Row: cursor.enable_hyprcursor
- Bucket: cursor/input
- Pre-enablement proof status: complete-pre-enablement-proof-only
- Production gate needed: yes
- Recovery model needed: cursor-input-persisted-dead-man-watchdog-with-keyboard-token-confirmation-and-pointer-independent-rollback
- Can be proven non-live: non-live gate scaffolding yes; full enablement without live/runtime proof no
- Requires live/runtime proof: yes
- Requires explicit approval: yes
- Recommended next action: Build cursor/input persisted recovery plan and keyboard-token confirmation proof before any cursor/input enablement batch.
- Exact remaining blocker: missing production gate implementation and acceptance/rejection tests; missing persisted recovery plan implementation for this bucket; missing out-of-band confirmation runner proof; missing production rollback proof against the future gated write path; live/runtime safety proof remains required or must be explicitly waived by future approval; explicit high-risk enablement approval is missing

- Row: cursor.use_cpu_buffer
- Bucket: cursor/input
- Pre-enablement proof status: complete-pre-enablement-proof-only
- Production gate needed: yes
- Recovery model needed: cursor-input-persisted-dead-man-watchdog-with-keyboard-token-confirmation-and-pointer-independent-rollback
- Can be proven non-live: non-live gate scaffolding yes; full enablement without live/runtime proof no
- Requires live/runtime proof: yes
- Requires explicit approval: yes
- Recommended next action: Build cursor/input persisted recovery plan and keyboard-token confirmation proof before any cursor/input enablement batch.
- Exact remaining blocker: missing production gate implementation and acceptance/rejection tests; missing persisted recovery plan implementation for this bucket; missing out-of-band confirmation runner proof; missing production rollback proof against the future gated write path; live/runtime safety proof remains required or must be explicitly waived by future approval; explicit high-risk enablement approval is missing

- Row: cursor.warp_back_after_non_mouse_input
- Bucket: cursor/input
- Pre-enablement proof status: complete-pre-enablement-proof-only
- Production gate needed: yes
- Recovery model needed: cursor-input-persisted-dead-man-watchdog-with-keyboard-token-confirmation-and-pointer-independent-rollback
- Can be proven non-live: non-live gate scaffolding yes; full enablement without live/runtime proof no
- Requires live/runtime proof: yes
- Requires explicit approval: yes
- Recommended next action: Build cursor/input persisted recovery plan and keyboard-token confirmation proof before any cursor/input enablement batch.
- Exact remaining blocker: missing production gate implementation and acceptance/rejection tests; missing persisted recovery plan implementation for this bucket; missing out-of-band confirmation runner proof; missing production rollback proof against the future gated write path; live/runtime safety proof remains required or must be explicitly waived by future approval; explicit high-risk enablement approval is missing

- Row: debug.overlay
- Bucket: debug/crash
- Pre-enablement proof status: complete-pre-enablement-proof-only
- Production gate needed: yes
- Recovery model needed: debug-crash-persisted-dead-man-watchdog-with-external-process-rollback
- Can be proven non-live: non-live gate scaffolding yes; full enablement without live/runtime proof no
- Requires live/runtime proof: yes
- Requires explicit approval: yes
- Recommended next action: Build debug/crash external recovery proof and explicit no-crash temp-only tests before any debug/crash enablement batch.
- Exact remaining blocker: missing production gate implementation and acceptance/rejection tests; missing persisted recovery plan implementation for this bucket; missing out-of-band confirmation runner proof; missing production rollback proof against the future gated write path; live/runtime safety proof remains required or must be explicitly waived by future approval; explicit high-risk enablement approval is missing

- Row: debug.damage_blink
- Bucket: debug/crash
- Pre-enablement proof status: complete-pre-enablement-proof-only
- Production gate needed: yes
- Recovery model needed: debug-crash-persisted-dead-man-watchdog-with-external-process-rollback
- Can be proven non-live: non-live gate scaffolding yes; full enablement without live/runtime proof no
- Requires live/runtime proof: yes
- Requires explicit approval: yes
- Recommended next action: Build debug/crash external recovery proof and explicit no-crash temp-only tests before any debug/crash enablement batch.
- Exact remaining blocker: missing production gate implementation and acceptance/rejection tests; missing persisted recovery plan implementation for this bucket; missing out-of-band confirmation runner proof; missing production rollback proof against the future gated write path; live/runtime safety proof remains required or must be explicitly waived by future approval; explicit high-risk enablement approval is missing

- Row: debug.gl_debugging
- Bucket: debug/crash
- Pre-enablement proof status: complete-pre-enablement-proof-only
- Production gate needed: yes
- Recovery model needed: debug-crash-persisted-dead-man-watchdog-with-external-process-rollback
- Can be proven non-live: non-live gate scaffolding yes; full enablement without live/runtime proof no
- Requires live/runtime proof: yes
- Requires explicit approval: yes
- Recommended next action: Build debug/crash external recovery proof and explicit no-crash temp-only tests before any debug/crash enablement batch.
- Exact remaining blocker: missing production gate implementation and acceptance/rejection tests; missing persisted recovery plan implementation for this bucket; missing out-of-band confirmation runner proof; missing production rollback proof against the future gated write path; live/runtime safety proof remains required or must be explicitly waived by future approval; explicit high-risk enablement approval is missing

- Row: debug.disable_logs
- Bucket: debug/crash
- Pre-enablement proof status: complete-pre-enablement-proof-only
- Production gate needed: yes
- Recovery model needed: debug-crash-persisted-dead-man-watchdog-with-external-process-rollback
- Can be proven non-live: non-live gate scaffolding yes; full enablement without live/runtime proof no
- Requires live/runtime proof: yes
- Requires explicit approval: yes
- Recommended next action: Build debug/crash external recovery proof and explicit no-crash temp-only tests before any debug/crash enablement batch.
- Exact remaining blocker: missing production gate implementation and acceptance/rejection tests; missing persisted recovery plan implementation for this bucket; missing out-of-band confirmation runner proof; missing production rollback proof against the future gated write path; live/runtime safety proof remains required or must be explicitly waived by future approval; explicit high-risk enablement approval is missing

- Row: debug.disable_time
- Bucket: debug/crash
- Pre-enablement proof status: complete-pre-enablement-proof-only
- Production gate needed: yes
- Recovery model needed: debug-crash-persisted-dead-man-watchdog-with-external-process-rollback
- Can be proven non-live: non-live gate scaffolding yes; full enablement without live/runtime proof no
- Requires live/runtime proof: yes
- Requires explicit approval: yes
- Recommended next action: Build debug/crash external recovery proof and explicit no-crash temp-only tests before any debug/crash enablement batch.
- Exact remaining blocker: missing production gate implementation and acceptance/rejection tests; missing persisted recovery plan implementation for this bucket; missing out-of-band confirmation runner proof; missing production rollback proof against the future gated write path; live/runtime safety proof remains required or must be explicitly waived by future approval; explicit high-risk enablement approval is missing

- Row: debug.damage_tracking
- Bucket: debug/crash
- Pre-enablement proof status: complete-pre-enablement-proof-only
- Production gate needed: yes
- Recovery model needed: debug-crash-persisted-dead-man-watchdog-with-external-process-rollback
- Can be proven non-live: non-live gate scaffolding yes; full enablement without live/runtime proof no
- Requires live/runtime proof: yes
- Requires explicit approval: yes
- Recommended next action: Build debug/crash external recovery proof and explicit no-crash temp-only tests before any debug/crash enablement batch.
- Exact remaining blocker: missing production gate implementation and acceptance/rejection tests; missing persisted recovery plan implementation for this bucket; missing out-of-band confirmation runner proof; missing production rollback proof against the future gated write path; live/runtime safety proof remains required or must be explicitly waived by future approval; explicit high-risk enablement approval is missing

- Row: debug.enable_stdout_logs
- Bucket: debug/crash
- Pre-enablement proof status: complete-pre-enablement-proof-only
- Production gate needed: yes
- Recovery model needed: debug-crash-persisted-dead-man-watchdog-with-external-process-rollback
- Can be proven non-live: non-live gate scaffolding yes; full enablement without live/runtime proof no
- Requires live/runtime proof: yes
- Requires explicit approval: yes
- Recommended next action: Build debug/crash external recovery proof and explicit no-crash temp-only tests before any debug/crash enablement batch.
- Exact remaining blocker: missing production gate implementation and acceptance/rejection tests; missing persisted recovery plan implementation for this bucket; missing out-of-band confirmation runner proof; missing production rollback proof against the future gated write path; live/runtime safety proof remains required or must be explicitly waived by future approval; explicit high-risk enablement approval is missing

- Row: debug.manual_crash
- Bucket: debug/crash
- Pre-enablement proof status: complete-pre-enablement-proof-only
- Production gate needed: yes
- Recovery model needed: debug-crash-persisted-dead-man-watchdog-with-external-process-rollback
- Can be proven non-live: non-live gate scaffolding yes; full enablement without live/runtime proof no
- Requires live/runtime proof: yes
- Requires explicit approval: yes
- Recommended next action: Build debug/crash external recovery proof and explicit no-crash temp-only tests before any debug/crash enablement batch.
- Exact remaining blocker: missing production gate implementation and acceptance/rejection tests; missing persisted recovery plan implementation for this bucket; missing out-of-band confirmation runner proof; missing production rollback proof against the future gated write path; live/runtime safety proof remains required or must be explicitly waived by future approval; explicit high-risk enablement approval is missing

- Row: debug.suppress_errors
- Bucket: debug/crash
- Pre-enablement proof status: complete-pre-enablement-proof-only
- Production gate needed: yes
- Recovery model needed: debug-crash-persisted-dead-man-watchdog-with-external-process-rollback
- Can be proven non-live: non-live gate scaffolding yes; full enablement without live/runtime proof no
- Requires live/runtime proof: yes
- Requires explicit approval: yes
- Recommended next action: Build debug/crash external recovery proof and explicit no-crash temp-only tests before any debug/crash enablement batch.
- Exact remaining blocker: missing production gate implementation and acceptance/rejection tests; missing persisted recovery plan implementation for this bucket; missing out-of-band confirmation runner proof; missing production rollback proof against the future gated write path; live/runtime safety proof remains required or must be explicitly waived by future approval; explicit high-risk enablement approval is missing

- Row: debug.disable_scale_checks
- Bucket: debug/crash
- Pre-enablement proof status: complete-pre-enablement-proof-only
- Production gate needed: yes
- Recovery model needed: debug-crash-persisted-dead-man-watchdog-with-external-process-rollback
- Can be proven non-live: non-live gate scaffolding yes; full enablement without live/runtime proof no
- Requires live/runtime proof: yes
- Requires explicit approval: yes
- Recommended next action: Build debug/crash external recovery proof and explicit no-crash temp-only tests before any debug/crash enablement batch.
- Exact remaining blocker: missing production gate implementation and acceptance/rejection tests; missing persisted recovery plan implementation for this bucket; missing out-of-band confirmation runner proof; missing production rollback proof against the future gated write path; live/runtime safety proof remains required or must be explicitly waived by future approval; explicit high-risk enablement approval is missing

- Row: debug.error_limit
- Bucket: debug/crash
- Pre-enablement proof status: complete-pre-enablement-proof-only
- Production gate needed: yes
- Recovery model needed: debug-crash-persisted-dead-man-watchdog-with-external-process-rollback
- Can be proven non-live: non-live gate scaffolding yes; full enablement without live/runtime proof no
- Requires live/runtime proof: yes
- Requires explicit approval: yes
- Recommended next action: Build debug/crash external recovery proof and explicit no-crash temp-only tests before any debug/crash enablement batch.
- Exact remaining blocker: missing production gate implementation and acceptance/rejection tests; missing persisted recovery plan implementation for this bucket; missing out-of-band confirmation runner proof; missing production rollback proof against the future gated write path; live/runtime safety proof remains required or must be explicitly waived by future approval; explicit high-risk enablement approval is missing

- Row: debug.error_position
- Bucket: debug/crash
- Pre-enablement proof status: complete-pre-enablement-proof-only
- Production gate needed: yes
- Recovery model needed: debug-crash-persisted-dead-man-watchdog-with-external-process-rollback
- Can be proven non-live: non-live gate scaffolding yes; full enablement without live/runtime proof no
- Requires live/runtime proof: yes
- Requires explicit approval: yes
- Recommended next action: Build debug/crash external recovery proof and explicit no-crash temp-only tests before any debug/crash enablement batch.
- Exact remaining blocker: missing production gate implementation and acceptance/rejection tests; missing persisted recovery plan implementation for this bucket; missing out-of-band confirmation runner proof; missing production rollback proof against the future gated write path; live/runtime safety proof remains required or must be explicitly waived by future approval; explicit high-risk enablement approval is missing

- Row: debug.colored_stdout_logs
- Bucket: debug/crash
- Pre-enablement proof status: complete-pre-enablement-proof-only
- Production gate needed: yes
- Recovery model needed: debug-crash-persisted-dead-man-watchdog-with-external-process-rollback
- Can be proven non-live: non-live gate scaffolding yes; full enablement without live/runtime proof no
- Requires live/runtime proof: yes
- Requires explicit approval: yes
- Recommended next action: Build debug/crash external recovery proof and explicit no-crash temp-only tests before any debug/crash enablement batch.
- Exact remaining blocker: missing production gate implementation and acceptance/rejection tests; missing persisted recovery plan implementation for this bucket; missing out-of-band confirmation runner proof; missing production rollback proof against the future gated write path; live/runtime safety proof remains required or must be explicitly waived by future approval; explicit high-risk enablement approval is missing

- Row: debug.log_damage
- Bucket: debug/crash
- Pre-enablement proof status: complete-pre-enablement-proof-only
- Production gate needed: yes
- Recovery model needed: debug-crash-persisted-dead-man-watchdog-with-external-process-rollback
- Can be proven non-live: non-live gate scaffolding yes; full enablement without live/runtime proof no
- Requires live/runtime proof: yes
- Requires explicit approval: yes
- Recommended next action: Build debug/crash external recovery proof and explicit no-crash temp-only tests before any debug/crash enablement batch.
- Exact remaining blocker: missing production gate implementation and acceptance/rejection tests; missing persisted recovery plan implementation for this bucket; missing out-of-band confirmation runner proof; missing production rollback proof against the future gated write path; live/runtime safety proof remains required or must be explicitly waived by future approval; explicit high-risk enablement approval is missing

- Row: debug.pass
- Bucket: debug/crash
- Pre-enablement proof status: complete-pre-enablement-proof-only
- Production gate needed: yes
- Recovery model needed: debug-crash-persisted-dead-man-watchdog-with-external-process-rollback
- Can be proven non-live: non-live gate scaffolding yes; full enablement without live/runtime proof no
- Requires live/runtime proof: yes
- Requires explicit approval: yes
- Recommended next action: Build debug/crash external recovery proof and explicit no-crash temp-only tests before any debug/crash enablement batch.
- Exact remaining blocker: missing production gate implementation and acceptance/rejection tests; missing persisted recovery plan implementation for this bucket; missing out-of-band confirmation runner proof; missing production rollback proof against the future gated write path; live/runtime safety proof remains required or must be explicitly waived by future approval; explicit high-risk enablement approval is missing

- Row: debug.full_cm_proto
- Bucket: debug/crash
- Pre-enablement proof status: complete-pre-enablement-proof-only
- Production gate needed: yes
- Recovery model needed: debug-crash-persisted-dead-man-watchdog-with-external-process-rollback
- Can be proven non-live: non-live gate scaffolding yes; full enablement without live/runtime proof no
- Requires live/runtime proof: yes
- Requires explicit approval: yes
- Recommended next action: Build debug/crash external recovery proof and explicit no-crash temp-only tests before any debug/crash enablement batch.
- Exact remaining blocker: missing production gate implementation and acceptance/rejection tests; missing persisted recovery plan implementation for this bucket; missing out-of-band confirmation runner proof; missing production rollback proof against the future gated write path; live/runtime safety proof remains required or must be explicitly waived by future approval; explicit high-risk enablement approval is missing

- Row: debug.ds_handle_same_buffer
- Bucket: debug/crash
- Pre-enablement proof status: complete-pre-enablement-proof-only
- Production gate needed: yes
- Recovery model needed: debug-crash-persisted-dead-man-watchdog-with-external-process-rollback
- Can be proven non-live: non-live gate scaffolding yes; full enablement without live/runtime proof no
- Requires live/runtime proof: yes
- Requires explicit approval: yes
- Recommended next action: Build debug/crash external recovery proof and explicit no-crash temp-only tests before any debug/crash enablement batch.
- Exact remaining blocker: missing production gate implementation and acceptance/rejection tests; missing persisted recovery plan implementation for this bucket; missing out-of-band confirmation runner proof; missing production rollback proof against the future gated write path; live/runtime safety proof remains required or must be explicitly waived by future approval; explicit high-risk enablement approval is missing

- Row: debug.ds_handle_same_buffer_fifo
- Bucket: debug/crash
- Pre-enablement proof status: complete-pre-enablement-proof-only
- Production gate needed: yes
- Recovery model needed: debug-crash-persisted-dead-man-watchdog-with-external-process-rollback
- Can be proven non-live: non-live gate scaffolding yes; full enablement without live/runtime proof no
- Requires live/runtime proof: yes
- Requires explicit approval: yes
- Recommended next action: Build debug/crash external recovery proof and explicit no-crash temp-only tests before any debug/crash enablement batch.
- Exact remaining blocker: missing production gate implementation and acceptance/rejection tests; missing persisted recovery plan implementation for this bucket; missing out-of-band confirmation runner proof; missing production rollback proof against the future gated write path; live/runtime safety proof remains required or must be explicitly waived by future approval; explicit high-risk enablement approval is missing

- Row: debug.fifo_pending_workaround
- Bucket: debug/crash
- Pre-enablement proof status: complete-pre-enablement-proof-only
- Production gate needed: yes
- Recovery model needed: debug-crash-persisted-dead-man-watchdog-with-external-process-rollback
- Can be proven non-live: non-live gate scaffolding yes; full enablement without live/runtime proof no
- Requires live/runtime proof: yes
- Requires explicit approval: yes
- Recommended next action: Build debug/crash external recovery proof and explicit no-crash temp-only tests before any debug/crash enablement batch.
- Exact remaining blocker: missing production gate implementation and acceptance/rejection tests; missing persisted recovery plan implementation for this bucket; missing out-of-band confirmation runner proof; missing production rollback proof against the future gated write path; live/runtime safety proof remains required or must be explicitly waived by future approval; explicit high-risk enablement approval is missing

- Row: debug.render_solitary_wo_damage
- Bucket: debug/crash
- Pre-enablement proof status: complete-pre-enablement-proof-only
- Production gate needed: yes
- Recovery model needed: debug-crash-persisted-dead-man-watchdog-with-external-process-rollback
- Can be proven non-live: non-live gate scaffolding yes; full enablement without live/runtime proof no
- Requires live/runtime proof: yes
- Requires explicit approval: yes
- Recommended next action: Build debug/crash external recovery proof and explicit no-crash temp-only tests before any debug/crash enablement batch.
- Exact remaining blocker: missing production gate implementation and acceptance/rejection tests; missing persisted recovery plan implementation for this bucket; missing out-of-band confirmation runner proof; missing production rollback proof against the future gated write path; live/runtime safety proof remains required or must be explicitly waived by future approval; explicit high-risk enablement approval is missing

- Row: debug.vfr
- Bucket: debug/crash
- Pre-enablement proof status: complete-pre-enablement-proof-only
- Production gate needed: yes
- Recovery model needed: debug-crash-persisted-dead-man-watchdog-with-external-process-rollback
- Can be proven non-live: non-live gate scaffolding yes; full enablement without live/runtime proof no
- Requires live/runtime proof: yes
- Requires explicit approval: yes
- Recommended next action: Build debug/crash external recovery proof and explicit no-crash temp-only tests before any debug/crash enablement batch.
- Exact remaining blocker: missing production gate implementation and acceptance/rejection tests; missing persisted recovery plan implementation for this bucket; missing out-of-band confirmation runner proof; missing production rollback proof against the future gated write path; live/runtime safety proof remains required or must be explicitly waived by future approval; explicit high-risk enablement approval is missing

- Row: debug.invalidate_fp16
- Bucket: debug/crash
- Pre-enablement proof status: complete-pre-enablement-proof-only
- Production gate needed: yes
- Recovery model needed: debug-crash-persisted-dead-man-watchdog-with-external-process-rollback
- Can be proven non-live: non-live gate scaffolding yes; full enablement without live/runtime proof no
- Requires live/runtime proof: yes
- Requires explicit approval: yes
- Recommended next action: Build debug/crash external recovery proof and explicit no-crash temp-only tests before any debug/crash enablement batch.
- Exact remaining blocker: missing production gate implementation and acceptance/rejection tests; missing persisted recovery plan implementation for this bucket; missing out-of-band confirmation runner proof; missing production rollback proof against the future gated write path; live/runtime safety proof remains required or must be explicitly waived by future approval; explicit high-risk enablement approval is missing

## Projected next 3 steps
1. Enable fully proven rows in safe batches.
2. Create special recovery/live-proof plans for rows that still require runtime safety proof.
3. Repeat grouped proof + enablement cycles until all 341 rows are writable where safely possible.
