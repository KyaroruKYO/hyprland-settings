# Structured Family Record Editor Forms

## Scope

This sprint adds review-only per-record editor form projections for:

- `hl.monitor`
- `hl.bind`
- `hl.animation`
- `hl.curve`
- `hl.gesture`
- `hl.device`
- `hl.permission`

The forms expose parsed structured-family records as review-only data. They do not enable real config editing.

## Form Model

Each record form includes:

- family and record index,
- source path and line number,
- raw line and parsed key,
- validation status,
- unsupported or not-proven reason where present,
- family-specific field list,
- raw fallback status,
- disabled action policy,
- write-blocked policy,
- temp-fixture plan status.

Unknown or unsupported record shapes stay raw and are marked as requiring raw fallback.

## UI

The Config page includes `hyprland-settings-structured-family-record-editor-section` and one family record-editor widget for each structured family.

The UI states:

- review-only per-record editor forms,
- record editor projection ready,
- family-specific fields projected,
- raw fallback required where not proven,
- editor actions disabled,
- real and production writes blocked by default,
- real config targets, runtime mutation, and Hyprland reload are not allowed.

All edit, apply, and real-config render controls are insensitive.

## Safety

- Real config touched: false.
- Runtime mutated: false.
- `hyprctl reload` run: false.
- Production behavior enabled: false.
- Production executor wired: false.
- `apply_setting_change` is not integrated.
- `write_flow` is not integrated.

## Follow-On Draft Model

Review-only in-memory draft models now consume the record-editor forms for all seven families. Drafts start clean, track dirty state in model tests, reset to original projected fields, preserve raw fallback status, and keep draft persistence forbidden.

## Next Work

Add fixture-only structured-family draft-to-rendered-record planning while keeping real writes blocked.
