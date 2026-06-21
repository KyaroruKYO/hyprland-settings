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

## Safety

- Real config writes are not active.
- Runtime mutation is not active.
- `hyprctl reload` is not active.
- Fixture render/reread proof does not enable real writes.
- Unknown or unsupported record shapes are retained as raw and marked `not proven yet`.

## Next Work

Add family-specific fixture write plans for structured-family editors using temp fixtures only. Production structured-family writes must remain blocked by default.
