# Structured Family Rendered-Record Staged Apply Plan

## Sprint Summary

This sprint adds a fixture-only staged apply plan model for structured-family rendered-record review.

Covered families:

- `hl.monitor`
- `hl.bind`
- `hl.animation`
- `hl.curve`
- `hl.gesture`
- `hl.device`
- `hl.permission`

## Proof Added

- Accepted in-memory confirmations can generate staged apply plans for all seven families.
- Staged apply plans link to the accepted confirmation, diff/review summary, and render/reread proof status.
- Plans preserve source draft count, source plan count, review entry count, changed entry count, noop entry count, raw fallback entry count, unsupported/not-proven entry count, and field diff count.
- Plans create ordered review-only stages: preflight, review, render preview, raw fallback preservation, unsupported/not-proven preservation, dry-run-only apply, and rollback plan.
- Changed entries produce changed review operations.
- Noop entries produce noop preservation operations.
- Raw fallback entries produce raw fallback preservation operations.
- Unsupported/not-proven entries produce unsupported preservation operations.
- Rejected confirmations produce blocked staged apply plans.
- Invalid confirmations produce blocked staged apply plans.
- Policy-relaxed confirmations produce explicit blockers.

## Apply Boundary

A staged apply plan is a review-only in-memory plan of what would be applied later.

It is not:

- An executor.
- A writer.
- A persistence mechanism.
- A reload path.
- A production activation path.

## Safety Boundaries

- Draft written to disk: false.
- Staged apply plan written to disk: false.
- Staged apply executed: false.
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
- Executor availability: unavailable by design.

## GTK Evidence

No visible GTK UI changed in this sprint.

GTK evidence root: `not-run-no-visible-ui-change`.

## Follow-Up Status

The fixture-only staged apply dry-run report layer is now complete. It summarizes staged apply plans, blocked plan cases, operation counts, and no-write/no-runtime guarantees without executing staged apply or dry-run behavior.

## Next Work

Add fixture-only structured-family rendered-record final executor-readiness audit while keeping real writes blocked.
