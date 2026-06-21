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

On the `future-capability-marathon` branch, missing/default insertion is now production-enabled only for reviewed single-root normal-scalar safe-batch targets. Source/include insertion target selection has disabled UI, fixture target-selection proof, selected-target dry-run planner, disabled preview UI, temp-fixture guarded executor proof, copied-config-tree proof, default-disabled production gate review, explicit approval-flow integration, a report-backed disabled Config-page approval card, activation decision review, activation path review, activation control review, activation form/state-machine review, real disabled GTK activation form fields, in-memory activation draft state, still-disabled activation draft-edit review, a live GTK draft-edit bridge that updates memory-only draft state and recomputes review-only validation, a default-disabled draft persistence boundary that forbids saving drafts by default, and a default-disabled production activation safety gate. The source/include form can generate review-only request and safety-plan data, display that data through GTK entries/check buttons/text views, validate through the final control while the executor stays `Unwired`, and update draft edits in memory only without persistence. Duplicate production work has copied-config-tree executor proof, default-disabled production gate review, explicit approval integration, report-backed disabled Config-page approval card, activation decision/path/control reviews, activation form/state-machine review, real disabled GTK activation form fields, in-memory activation draft state, still-disabled activation draft-edit review, the same live GTK memory-only draft-edit bridge, the same default-disabled persistence boundary, and the same default-disabled production activation safety gate; it also validates through the final control while the executor stays `Unwired`. The source/include and duplicate safety gates remain blocked by missing/proof-required byte-exact backup, write/reread/restore, post-restore verification, no-auto-apply, persisted-draft auto-apply, final approval, production flag, executor wiring, rollback, and report-backed proof requirements. Structured `hl.bind` and profile/mode paths have copied-config-tree executor proof with byte/symlink restoration plus default-disabled production gate review, explicit approval-flow integration, and report-backed disabled Config-page approval cards. Runtime/reload has a guarded dry-run executor, default-disabled production gate, explicit approval-flow integration, runtime socket diagnosis, read-only live evidence, a proven low-risk `general:gaps_in` live-restore proof using `hyprctl eval 'hl.config({ general = { gaps_in = VALUE } })'`, an approved-but-default-disabled runtime approval review consuming that proof, and a disabled detail UI surface that displays the proof without any runtime handler; sandbox socket access is still blocked by `EPERM`, but all runtime proof ran outside the sandbox. High-risk/display recovery has a guarded no-op readiness executor, default-disabled production gate, explicit approval-flow integration, runtime read-only evidence, runtime approval review evidence, low-risk runtime restore proof as readiness input, and a report-backed disabled approval card that states that runtime proof is insufficient for high-risk activation while listing recovery/dead-man/restore/backup/snapshot blockers. Hyprland 0.55.4 migration has local package metadata evidence (`hyprland 0.55.4-1`), runtime version evidence, plus a default-disabled activation gate, explicit approval-flow integration, and a report-backed disabled migration approval card, but remains advisory only because no trusted local 0.55.4 export bundle was found. GTK safe-env screenshot-level assertions cover disabled approval, activation decision, activation path, activation control, activation form, activation draft, activation draft-edit, live draft-edit, persistence-boundary, and production activation safety-gate cards plus activation form field labels through screenshot capture plus AT-SPI accessibility-tree text. Source/include insertion expansion, duplicate production writes, high-risk/display writes, structured-family writes, real profile/mode switching, runtime/reload production mutation, draft persistence opt-in, and Hyprland 0.55.4 migration remain blocked or disabled pending explicit production activation gates and/or trusted data.

Current future-capability tracker:

- Core app shell / UI / navigation: 99-99%
- Config discovery / source-aware model: 94-96%
- 341-row read/write model: 90-95%
- Safe normal-scalar writes: 95-97%
- Release packaging/tag/artifacts: 85-95%
- Missing/default insertion: 99-100%
- Duplicate resolution: 94-95%
- High-risk/display recovery: 62-70%
- Structured-family editors/writes: 64-73%
- Profile/mode switching: 65-73%
- Runtime/reload integration: 66-76%
- Hyprland 0.55.4 migration: 50-60%

Next exact work item: design explicit final-approval, production-flag, executor-wiring, and live production dry-run decisions before any production executor wiring can be reconsidered.

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

## 2026-06-20 - Activation Control Status

Source/include and duplicate now have final default-disabled activation control reviews. Each validates complete request and safety-plan inputs for review only, displays executor wiring as `Unwired`, and keeps production behavior disabled. No real config, runtime, reload, AGS/Waybar, release tag, or dist artifact was modified.

## 2026-06-20 - Activation Form Status

Source/include and duplicate now have review-only activation form/state-machine projections. Each form can generate request and safety-plan data, validate through the final activation control as `ValidatedButExecutorUnwired`, display executor wiring as `Unwired`, and keep production behavior disabled. No real config, runtime, reload, AGS/Waybar, release tag, or dist artifact was modified.

## 2026-06-20 - Activation Form Field Status

Source/include and duplicate activation form data now renders through real disabled GTK form fields. Scope/reason/token/category use read-only insensitive entries, acknowledgement states use insensitive check buttons, and safety-plan data uses read-only insensitive text views. The form state machine and final activation control remain review-only; executors stay `Unwired`, production flags stay false, and no real config, runtime, reload, AGS/Waybar, release tag, or dist artifact was modified.

The GTK matrix was run for the form field UI at `/tmp/hyprland-settings-gtk-automation/20260620_134347`; the run preserved safety boundaries, but live AT-SPI field-label proof was blocked because the accessibility bus socket was unavailable in that run.

## 2026-06-20 - Activation Draft Status

Source/include and duplicate now have in-memory activation draft plumbing. Drafts can be created from existing form state, updated/reset in memory, converted back into form state, and validated through the existing form/control reviews as review-only. The Config page displays disabled draft cards for both categories with `In-memory only`, `Executor wiring: Unwired`, and production-disabled copy. Planned update/reset controls are insensitive and have no persistence, mutation, or executor handler. Production flags remain false, executors remain `Unwired`, no disk persistence was added, and no real config, runtime, reload, AGS/Waybar, release tag, dist artifact, or 0.55.4 migration state was modified.

## 2026-06-20 - Activation Draft Edit Status

Source/include and duplicate now have still-disabled activation draft-edit plumbing. Draft editing is disabled by default in the live UI, but model tests can enter in-memory-only edit mode, update draft request and safety-plan fields, recompute validation through the existing form/control pipeline, reset back to defaults, and prove disk persistence remains absent. The Config page displays disabled draft-edit cards for both categories with editing mode, dirty state, validation, `In-memory only`, `Executor wiring: Unwired`, and production-disabled copy. Planned update/reset controls are insensitive and have no persistence, mutation, or executor handler.

The GTK matrix was run for the draft-edit UI at `/tmp/hyprland-settings-gtk-automation/20260620_154855`; screenshot plus AT-SPI accessibility-tree assertions passed for the source/include and duplicate draft-edit cards.

## 2026-06-20 - Remaining Dependency Scan

The remaining dependency scan classifies core UI, config discovery, 341-row coverage, safe normal-scalar writes, and release packaging as effectively capped for this safe-release branch. Missing/default insertion, duplicate resolution, structured-family writes, profile/mode switching, and runtime/reload integration are blocked by production activation. High-risk/display recovery is blocked by high-risk recovery proof. Hyprland 0.55.4 migration is blocked by missing official export data.

## 2026-06-20 - Production Activation Safety Proof

Source/include and duplicate production activation safety proof now runs against copied fixtures only. The proof satisfies byte-exact backup, pre-write snapshot, target identity, dry-run write plan, diff preview, post-write reread, restore, post-restore verification, and rollback checks without touching real config. No-auto-apply and persisted-draft auto-apply proof are satisfied by default-disabled report-backed UI/control evidence and the persistence boundary. Final approval, production flag decision, executor wiring decision, and live production dry-run remain unresolved; source/include insertion and duplicate writes remain disabled, executors remain `Unwired`, and draft persistence remains forbidden by default.
