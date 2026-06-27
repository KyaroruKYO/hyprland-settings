# Structured Family Temp Write Plans

## Scope

This sprint adds family-specific validators and temp-fixture write plans for:

- `hl.monitor`
- `hl.bind`
- `hl.animation`
- `hl.curve`
- `hl.gesture`
- `hl.device`
- `hl.permission`

The plans are fixture-only. They do not enable real structured-family writes.

## Validator Status

Each family now has a conservative validator that checks only fixture-proven shape:

- Required fields must be present where the fixture proves them.
- Numeric fields are parsed only where the fixture proves numeric shape.
- Unknown, incomplete, or unsupported forms are preserved raw and marked `not proven yet`.
- No validator calls `hyprctl`, inspects live hardware, executes dispatchers, applies permissions, or validates against real applications.

## Temp-Fixture Plan Status

For every family, the model can:

- build a temp-fixture write plan,
- validate the plan,
- render to a temp/test-owned target,
- reread the rendered output,
- preserve family identity and record count,
- reject real config render targets.

The path guard rejects `/home/kyo/.config/hypr/hyprland.conf` and `~/.config/hypr` targets.

## Safety

- Production writes remain blocked by default.
- No production executor is wired.
- `apply_setting_change` is not integrated with structured-family temp plans.
- `write_flow` is not integrated with structured-family temp plans.
- Real config touched: false.
- Runtime mutated: false.
- `hyprctl reload` run: false.
- Source/include and duplicate production activation remain capped and separate-phase only.

## Follow-On Record Forms

Review-only per-record editor form projections now consume the validator and temp-fixture plan status for all seven families. The forms expose source path, line number, raw line, validation status, family-specific fields, raw fallback status, and disabled action/write policy without enabling real writes.

Review-only in-memory draft models now consume those forms and keep draft persistence forbidden while proving dirty-state and reset behavior in model tests.

## Next Work

Stop for explicit user decision: structured-family production activation remains blocked until the user approves real-write activation scope.
