# Structured Family Draft Rendered Record Plan

## Scope

This sprint adds fixture-only draft-to-rendered-record planning for structured-family drafts:

- `hl.monitor`
- `hl.bind`
- `hl.animation`
- `hl.curve`
- `hl.gesture`
- `hl.device`
- `hl.permission`

The plan model is review-only and in memory only. It does not write rendered records to disk, write real config, persist drafts, reload Hyprland, mutate runtime, or wire production executors.

## Planning Model

Each plan is created from a structured-family record draft and includes:

- family and record index,
- source path and line number,
- raw original line and parsed key,
- draft fields,
- rendered-record field map,
- rendered-record preview,
- syntax projection status,
- raw fallback status,
- unsupported or not-proven reason where present,
- fixture-only policy,
- disabled action policy,
- write-blocked policy,
- persistence-forbidden policy,
- real-config-target-forbidden policy.

Proven simple fixture records receive a preview string using family-specific syntax. Unsupported or not-proven records preserve the raw original line and are marked as not proven instead of pretending to synthesize a full Hyprland record.

## Safety

- Draft written to disk: false.
- Rendered record written to disk: false.
- Real config touched: false.
- Runtime mutated: false.
- `hyprctl reload` run: false.
- Production behavior enabled: false.
- Production executor wired: false.
- `apply_setting_change` is not integrated.
- `write_flow` is not integrated.
- `File::create`, `write_all`, and `serde_json::to_writer` are not used by the draft-to-rendered-record planning layer.

## Next Work

Add fixture-only structured-family draft rendered-record render/reread proof while keeping real writes blocked.
