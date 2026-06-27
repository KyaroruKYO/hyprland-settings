# Structured Family Draft Rendered-Record Diff/Review

## Sprint Summary

This sprint adds a fixture-only diff/review summary layer for structured-family rendered-record plans.

Covered families:

- `hl.monitor`
- `hl.bind`
- `hl.animation`
- `hl.curve`
- `hl.gesture`
- `hl.device`
- `hl.permission`

## Proof Added

- Diff/review summaries are built from rendered-record plans and the fixture-only render/reread proof.
- Each summary links to render/reread proof status.
- Each summary preserves source draft count and source plan count.
- Each summary creates per-record review entries.
- Each entry compares the original raw line with the rendered-record preview.
- Field diffs are derived from rendered-record field maps.
- Entries are classified as changed or noop.
- Raw fallback and unsupported/not-proven counts are preserved.
- Unsupported/not-proven records remain marked as not safe for full synthesis.
- Review summary text is created in memory only.

## Safety Boundaries

- Draft written to disk: false.
- Diff summary written to disk: false.
- Rendered record written to temp fixture: inherited from render/reread proof.
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

## GTK Evidence

No visible GTK UI changed in this sprint.

GTK evidence root: `not-run-no-visible-ui-change`.

## Next Work

Add fixture-only structured-family draft rendered-record approval/confirmation model while keeping real writes blocked.
