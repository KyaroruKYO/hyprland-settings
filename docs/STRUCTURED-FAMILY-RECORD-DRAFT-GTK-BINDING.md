# Structured Family Record Draft GTK Binding

## Scope

This sprint adds disabled live GTK draft-field binding projections for structured-family record drafts:

- `hl.monitor`
- `hl.bind`
- `hl.animation`
- `hl.curve`
- `hl.gesture`
- `hl.device`
- `hl.permission`

The binding layer is review-only. It does not enable real editing, persistence, config writes, runtime mutation, reload, or production executors.

## Binding Model

Each binding is created from an in-memory structured-family record draft and includes:

- family and record index,
- source path and line number,
- field name and kind,
- original, display, and draft values,
- insensitive GTK widget kind,
- dirty state,
- validation status,
- raw fallback status,
- disabled action policy,
- write-blocked policy,
- persistence-forbidden policy.

Model-only tests prove that a field update can recompute dirty state and validation state in memory. The GTK-visible widgets remain insensitive, and unsupported or not-proven records preserve raw fallback status instead of becoming fully editable.

## UI

The Config page includes `hyprland-settings-structured-family-record-draft-binding-section` and one family binding widget for each structured family. It shows record counts, draft counts, bound field counts, insensitive widget counts, dirty/validation/raw-fallback status, write policy, persistence policy, and action policy.

All update, reset, persist, and apply controls are insensitive.

## Safety

- Draft written to disk: false.
- Real config touched: false.
- Runtime mutated: false.
- `hyprctl reload` run: false.
- Production behavior enabled: false.
- Production executor wired: false.
- `apply_setting_change` is not integrated.
- `write_flow` is not integrated.
- `File::create`, `write_all`, and `serde_json::to_writer` are not used by the binding UI section.

## Next Work

Add fixture-only structured-family rendered-record final executor-readiness audit while keeping real writes blocked.
