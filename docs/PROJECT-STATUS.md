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

On the `future-capability-marathon` branch, missing/default insertion is now production-enabled only for reviewed single-root normal-scalar safe-batch targets. Source/include insertion target selection has disabled UI, fixture target-selection proof, selected-target dry-run planner, disabled preview UI, temp-fixture guarded executor proof, copied-config-tree proof, default-disabled production gate review, explicit approval-flow integration, and a report-backed disabled Config-page approval card showing copied proof, preconditions, restore status, and original-unchanged proof. Duplicate, structured `hl.bind`, and profile/mode paths now have copied-config-tree executor proof with byte/symlink restoration plus default-disabled production gate review, explicit approval-flow integration, and report-backed disabled Config-page approval cards. Runtime/reload has a guarded dry-run executor, default-disabled production gate, explicit approval-flow integration, runtime socket diagnosis, read-only live evidence, a proven low-risk `general:gaps_in` live-restore proof using `hyprctl eval 'hl.config({ general = { gaps_in = VALUE } })'`, an approved-but-default-disabled runtime approval review consuming that proof, and a disabled detail UI surface that displays the proof without any runtime handler; sandbox socket access is still blocked by `EPERM`, but all runtime proof for this sprint ran outside the sandbox. High-risk/display recovery has a guarded no-op readiness executor, default-disabled production gate, explicit approval-flow integration, runtime read-only evidence, runtime approval review evidence, low-risk runtime restore proof as readiness input, and a report-backed disabled approval card that states that runtime proof is insufficient for high-risk activation while listing recovery/dead-man/restore/backup/snapshot blockers. Hyprland 0.55.4 migration has local package metadata evidence (`hyprland 0.55.4-1`), runtime version evidence, plus a default-disabled activation gate, explicit approval-flow integration, and a report-backed disabled migration approval card, but remains advisory only because no trusted local 0.55.4 export bundle was found. GTK safe-env screenshot-level assertions cover all six Config-page approval cards through screenshot capture plus AT-SPI accessibility-tree text. Source/include insertion expansion, duplicate production writes, high-risk/display writes, structured-family writes, real profile/mode switching, runtime/reload production mutation, and Hyprland 0.55.4 migration remain blocked or disabled pending explicit production activation gates and/or trusted data.

Current future-capability tracker:

- Core app shell / UI / navigation: 97-98%
- Config discovery / source-aware model: 94-96%
- 341-row read/write model: 90-95%
- Safe normal-scalar writes: 95-97%
- Release packaging/tag/artifacts: 85-95%
- Missing/default insertion: 96-97%
- Duplicate resolution: 87-91%
- High-risk/display recovery: 62-70%
- Structured-family editors/writes: 64-73%
- Profile/mode switching: 65-73%
- Runtime/reload integration: 66-76%
- Hyprland 0.55.4 migration: 50-60%

Next exact work item: use report-backed approval card data as the input for a future default-disabled production activation decision review, beginning with source/include and duplicate paths while keeping production flags false.

## Default-Disabled Production Activation Decision Review - 2026-06-20

- Added source/include and duplicate production activation decision reviews that consume report-backed approval card data.
- Both decisions can reach ApprovedButDefaultDisabled only while production flags remain false.
- Added disabled Config-page decision cards and GTK screenshot plus AT-SPI assertions for both cards.
- No production source/include insertion, duplicate write, runtime mutation, reload, or real config mutation was enabled.

## Default-Disabled Production Activation Path Review - 2026-06-20

- Added source/include and duplicate production activation path reviews that consume ApprovedButDefaultDisabled decisions.
- Added explicit future request and safety-plan requirements: production activation request, user approval, production flag, backup, restore, reread, post-restore verification, dry-run summary, touched-file list, and final confirmation.
- Added disabled Config-page activation path cards and GTK screenshot plus AT-SPI assertions for both cards.
- Production source/include insertion and duplicate replacement remain disabled; no real config, runtime mutation, reload, or executor path was enabled.
