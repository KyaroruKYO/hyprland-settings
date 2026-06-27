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
- Disabled GTK draft-field binding: all seven families expose insensitive draft-field binding projections with memory-only update proof.
- Draft rendered-record planning: all seven families map in-memory draft fields to fixture-only rendered-record previews.
- Draft rendered-record render/reread proof: all seven families render previews to temp fixture text and reread through the parser/projection path with family identity and record count preserved.
- Draft rendered-record diff/review summary: all seven families create in-memory changed/noop review entries with field diffs, raw fallback preservation, and unsupported/not-proven preservation.

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
- Draft rendered-record render/reread proof writes temp/test-owned fixture text only and does not write rendered records to real config.
- Draft rendered-record diff/review summaries are in-memory only and do not persist summaries or authorize writes.

## Next Work

Add fixture-only structured-family rendered-record staged apply plan while keeping real writes blocked.
