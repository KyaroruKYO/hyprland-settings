# Production Save Integration

Every path that can write the active Hyprland config is now gated on
**Safe Live Save Mode**: the save proceeds only after a live, read-only
verification that `misc:disable_autoreload` is `true` at runtime. If the
mode is inactive (or its state cannot be read), the save fails closed with
an actionable message — no file is touched.

## Why

Hyprland auto-reloads on config writes by default. Before this integration,
a scalar save while autoreload was active would immediately reload the
compositor — the exact write+reload loop the product decision rejected.
The app's flow is: preview live (runtime, reversible, no file) → enable
Safe Live Save Mode (runtime-only, no file, no reload) → save once.

## The gate

`safe_live_save_mode::require_safe_live_save_mode(runner)` is the single
gate. It reads `misc.disable_autoreload` from the runtime and:

- `ActiveViaRuntime` (`true`) → the save may proceed.
- `Inactive` → error: enabling the mode first is required.
- `Unknown` (unreadable) → error; fails closed.

There is deliberately no bypass parameter.

## Gated paths

| Save path | Route |
|---|---|
| Scalar preview "Save" button | `gated_scalar_save_live` |
| Scalar detail-pane "Apply reviewed change" | `gated_scalar_save_live` |
| Structured-family "Save previewed value" | `gated_family_save_live` |

`production_save::gated_scalar_save_live` verifies the mode live, then
delegates to the existing `apply_setting_change` backup/write/reread flow
exactly once. The gate itself lives in `safe_live_save_mode`, which stays
runtime-only and is source-guarded against ever writing a file; the gated
save lives in its own `production_save` module.
`gated_family_save` performs its own gate call as Gate 2 of its sequence
(see [STRUCTURED-FAMILY-GATED-PERSISTENCE.md](STRUCTURED-FAMILY-GATED-PERSISTENCE.md)).

UI code cannot call `apply_setting_change` directly anymore — a source
guard test (`persistence_sources_stay_guarded`) asserts `window.rs`
contains no direct call, and UI code never constructs runners (the `_live`
wrappers own the `HyprctlRuntimePreviewRunner`).

Non-UI write paths are unaffected in reach: `safe_batch_write` remains a
library/test path against temp configs; the active-config pilot remains an
env-gated ignored test with its own fifteen gates and restore.

## Evidence

- Unit tests: `save_is_blocked_without_safe_live_save_mode` proves the
  fail-closed behavior against a mock runner.
- Live flow proof (2026-07-12): with autoreload active, the gated family
  save was blocked with the enable-first message; after enabling the mode,
  real saves proceeded and no reload fired.
- Safety flags: `activeConfigWrittenDuringNormalTests: false`,
  `hyprctlReloadRan: false`.

Report: `data/reports/production-save-integration.v0.55.2.json`.
