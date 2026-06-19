# Project Status

## Current Counts

- Official scalar settings modeled: 341
- Readable rows: 341
- Writable rows: 341
- Blocked rows: 0

This is the current proven config-write model for Hyprland `0.55.2`.

## Proof Model

The current proof model is based on Rust source, generated reports, fixture tests, parser tests, and high-risk gate tests. It proves that all 341 official scalar settings are represented by the unified row-driven pipeline and are writable through either the normal config-write path or the gated high-risk config-write path.

Live runtime mutation and Hyprland reload proof are not claimed.

## High-risk Gate Summary

High-risk rows remain writable only through recovery and confirmation gates. The gate model requires persisted recovery plan validation, backup proof, rollback proof, parser reread proof, confirmation token proof, timeout/no-confirmation rollback behavior, and UI warning or advanced placement.

High-risk rows must not bypass the production gate or become ordinary low-risk writes.

## cursor.default_monitor Oracle Summary

`cursor.default_monitor` is writable as a gated high-risk cursor/input row. It uses `src/monitor_name_oracle.rs` and `ScalarWriteValueKind::MonitorName`.

The monitor-name oracle accepts names only from a current non-mutating snapshot and rejects empty, missing, stale, unsafe, path-like, command-like, malformed, and duplicate-problematic input. Tests use fixture and mock snapshots; the optional `hyprctl monitors` adapter is read-only and tested through fixture output only.

## Complete

- All 341 official scalar rows are modeled.
- All 341 rows are readable.
- All 341 rows are writable through the app's config-write or gated high-risk write model.
- Current aggregate reports agree on 341 readable / 341 writable / 0 blocked.
- `SAFE_WRITABLE_ROWS.len()` is 341.
- `cursor.default_monitor` is included and uses monitor-name oracle validation.
- High-risk rows require production gate proof.
- Screen-shader remains behind its production gate.

## Not Claimed

- Live runtime mutation proof for every setting.
- Hyprland reload/eval proof for every setting.
- Crash/debug proof against the active compositor.
- Stable packaged release status.
- Hyprland upstream endorsement.

## Next Recommended Work

On `main`, the v0.1.0 safe release scope remains complete for guarded normal-scalar safe-batch use on the v0.55.2 model.

On the `future-capability-marathon` branch, missing/default insertion is now production-enabled only for reviewed single-root normal-scalar safe-batch targets. Source/include insertion target selection, duplicate production writes, high-risk/display writes, structured-family writes, real profile/mode switching, runtime/reload mutation, and Hyprland 0.55.4 migration remain blocked or disabled pending explicit proof and approval.

Current future-capability tracker:

- Core app shell / UI / navigation: 90-95%
- Config discovery / source-aware model: 87-91%
- 341-row read/write model: 90-95%
- Safe normal-scalar writes: 91-96%
- Release packaging/tag/artifacts: 85-95%
- Missing/default insertion: 84-90%
- Duplicate resolution: 66-74%
- High-risk/display recovery: 50-60%
- Structured-family editors/writes: 48-58%
- Profile/mode switching: 50-60%
- Runtime/reload integration: 45-55%
- Hyprland 0.55.4 migration: 35-45%

Next exact work item: connect source/include insertion target-selection review to disabled UI, or build the explicit duplicate production approval gate while keeping duplicate writes disabled by default.
