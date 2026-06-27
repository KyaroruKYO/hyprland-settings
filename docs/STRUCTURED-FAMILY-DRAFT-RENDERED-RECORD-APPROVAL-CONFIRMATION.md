# Structured Family Draft Rendered-Record Approval/Confirmation

## Sprint Summary

This sprint adds a fixture-only approval/confirmation model for structured-family rendered-record diff/review summaries.

Covered families:

- `hl.monitor`
- `hl.bind`
- `hl.animation`
- `hl.curve`
- `hl.gesture`
- `hl.device`
- `hl.permission`

## Proof Added

- Approval drafts are built from diff/review summaries.
- Approval drafts link to diff/review summary status and render/reread proof status.
- Approval drafts preserve source draft count, source plan count, review entry count, changed count, noop count, raw fallback count, unsupported/not-proven count, and field diff count.
- Raw fallback acknowledgement is required when raw fallback entries exist.
- Unsupported/not-proven acknowledgement is required when unsupported entries exist.
- Valid confirmations can be accepted in memory only.
- Valid confirmations can be rejected in memory only.
- Invalid confirmations return explicit invalidation reasons for mismatched counts, missing links, missing acknowledgements, or attempted write/runtime/reload/production policy changes.

## Approval Boundary

Approval means approved for the next fixture-only review stage only.

Approval does not authorize:

- Real config writes.
- Draft persistence.
- Confirmation persistence.
- Rendered-record writes to real config.
- Runtime mutation.
- `hyprctl reload`.
- Production executor wiring.

## Safety Boundaries

- Draft written to disk: false.
- Approval written to disk: false.
- Confirmation written to disk: false.
- Rendered record written to real config: false.
- Real config touched: false.
- Runtime mutated: false.
- `hyprctl reload` run: false.
- Production executor wired: false.
- Production behavior enabled: false.

## Policy

- Fixture-only status: `StructuredFamilyDraftRenderedRecordFixtureOnly`.
- Action policy: `StructuredFamilyDraftRenderedRecordActionsDisabled`.
- Write policy: `StructuredFamilyDraftRenderedRecordWritesBlockedByDefault`.
- Persistence policy: `StructuredFamilyDraftRenderedRecordPersistenceForbidden`.
- Real config target policy: `StructuredFamilyDraftRenderedRecordRealConfigTargetForbidden`.
- Production executor policy: `StructuredFamilyDraftRenderedRecordProductionExecutorForbidden`.

## GTK Evidence

No visible GTK UI changed in this sprint.

GTK evidence root: `not-run-no-visible-ui-change`.

## Next Work

Add fixture-only structured-family rendered-record staged apply plan while keeping real writes blocked.
