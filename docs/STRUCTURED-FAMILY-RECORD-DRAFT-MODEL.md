# Structured Family Record Draft Model

## Scope

This sprint adds review-only in-memory draft state for structured-family records:

- `hl.monitor`
- `hl.bind`
- `hl.animation`
- `hl.curve`
- `hl.gesture`
- `hl.device`
- `hl.permission`

The draft model does not enable real editing, persistence, config writes, runtime mutation, reload, or production executors.

## Draft Model

Each draft is created from a record-editor form and includes:

- family and record index,
- source path and line number,
- raw original line and parsed key,
- original fields and draft fields,
- dirty state,
- validation status,
- unsupported or not-proven reason where present,
- raw fallback status,
- reset status,
- disabled action policy,
- write-blocked policy,
- persistence-forbidden policy,
- temp-fixture plan status.

Drafts start clean, are created in memory only, can become dirty through model-only tests, and reset back to the original projected fields. Unsupported or not-proven records keep raw fallback status and do not become fully editable.

## UI

The Config page includes `hyprland-settings-structured-family-record-draft-section` and one family draft widget for each structured family. It shows draft counts, dirty draft counts, raw fallback draft counts, write policy, persistence policy, and action policy.

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
- `File::create`, `write_all`, and `serde_json::to_writer` are not used by the draft UI section.

## Next Work

Add disabled live GTK draft-field binding for structured-family record drafts while keeping persistence and real writes blocked.
