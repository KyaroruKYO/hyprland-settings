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

On the `future-capability-marathon` branch, missing/default insertion is now production-enabled only for reviewed single-root normal-scalar safe-batch targets. Source/include insertion target selection has disabled UI, fixture target-selection proof, selected-target dry-run planner, disabled preview UI, and a temp-fixture guarded executor that restores original bytes. Duplicate, structured `hl.bind`, and profile/mode paths now have guarded temp-fixture executors that prove reversible fixture mutation and restoration. Runtime/reload has a guarded dry-run executor that records restore commands without executing real commands. High-risk/display recovery has a guarded no-op readiness executor. Hyprland 0.55.4 migration has a local evidence collector that remains advisory only. Source/include insertion expansion, duplicate production writes, high-risk/display writes, structured-family writes, real profile/mode switching, runtime/reload mutation, and Hyprland 0.55.4 migration remain blocked or disabled pending copied-config/live proof and explicit approval.

Current future-capability tracker:

- Core app shell / UI / navigation: 93-96%
- Config discovery / source-aware model: 92-95%
- 341-row read/write model: 90-95%
- Safe normal-scalar writes: 94-97%
- Release packaging/tag/artifacts: 85-95%
- Missing/default insertion: 91-94%
- Duplicate resolution: 77-84%
- High-risk/display recovery: 54-64%
- Structured-family editors/writes: 53-63%
- Profile/mode switching: 55-65%
- Runtime/reload integration: 50-60%
- Hyprland 0.55.4 migration: 39-49%

Next exact work item: run controlled copied-config-tree proof for source/include, duplicate, structured, profile, and runtime paths before considering any real config/runtime mutation.
