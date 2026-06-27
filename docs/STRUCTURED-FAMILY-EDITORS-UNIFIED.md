# Structured Family Editors Unified

## Scope

This sprint adds a shared structured-family editor foundation for:

- `hl.monitor`
- `hl.bind`
- `hl.animation`
- `hl.curve`
- `hl.gesture`
- `hl.device`
- `hl.permission`

The work is review-only. Structured-family writes remain blocked by default.

## Implementation

- Shared model: `src/structured_family.rs`
- Config parser support: structured records continue to be preserved from parsed config lines, with `bind*` forms grouped under `hl.bind`.
- UI projection: all seven families are projected through a consistent card model.
- Config page UI: `hyprland-settings-structured-family-section` renders all seven review-only cards.
- Fixtures: `tests/fixtures/structured_families/`
- Tests: `tests/structured_family_unified.rs`
- Family-specific validators: conservative fixture-level validation for all seven families.
- Temp-fixture write plans: all seven families can validate, render to temp/test-owned output, reread, and preserve family identity without enabling real writes.
- Per-record editor forms: all seven families expose review-only form projections with family-specific fields, raw fallback status, validation status, source path, line number, and disabled action policy.
- Record draft model: all seven families expose review-only in-memory draft projections with clean/dirty state, reset proof, raw fallback status, disabled actions, blocked writes, and forbidden persistence.

## Safety

- Real config writes are not active.
- Runtime mutation is not active.
- `hyprctl reload` is not active.
- Fixture render/reread proof does not enable real writes.
- Unknown or unsupported record shapes are retained as raw and marked `not proven yet`.
- Real config render targets are rejected by the structured-family path guard.
- Temp-fixture write plans do not call `apply_setting_change`, real `write_flow`, `hyprctl`, or reload command paths.
- Record editor forms are review-only and do not add write, reload, runtime mutation, persistence, or production executor callbacks.
- Record draft models are in-memory only and do not write drafts to disk.

## Next Work

Add fixture-only structured-family draft rendered-record render/reread proof while keeping real writes blocked.
