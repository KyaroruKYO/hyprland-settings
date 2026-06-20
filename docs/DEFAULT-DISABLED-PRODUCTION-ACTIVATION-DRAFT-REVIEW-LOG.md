# Default-Disabled Production Activation Draft Review Log

## 2026-06-20

- Added in-memory activation draft plumbing for source/include insertion and duplicate replacement.
- Drafts can be created empty or from existing review-only activation form state.
- Draft updates can change text fields, acknowledgement fields, safety-plan fields, and touched-file lists in memory.
- Draft reset returns to the empty default state.
- Complete drafts validate through the existing activation form and final activation control pipeline as review-only.
- Source/include and duplicate draft controls still validate as `ValidatedButExecutorUnwired`.
- The Config page displays disabled draft status cards for source/include and duplicate.
- Draft cards show draft status, draft validation, dirty state, in-memory-only status, executor wiring, and production-disabled status.
- Draft update/reset controls are insensitive and have no mutation, persistence, or executor handler.
- Source/include and duplicate executors remain `Unwired`.
- Source/include and duplicate production flags remain false.
- No disk persistence, real config mutation, runtime mutation, reload, AGS/Waybar touch, release artifact change, or migration activation was added.

## 2026-06-20 - Draft edit review

- Added a still-disabled activation draft-edit layer for source/include insertion and duplicate replacement.
- Draft editing is disabled by default in the live UI.
- Model tests can enter in-memory-only edit mode, update request and safety-plan draft values, recompute validation through the existing form/control pipeline, and reset to default draft state.
- Draft-edit validation remains review-only: source/include and duplicate still validate through the final control as `ValidatedButExecutorUnwired`.
- The Config page displays disabled draft-edit status cards for source/include and duplicate.
- Draft-edit cards show editing mode, dirty state, draft validation, in-memory-only status, executor wiring, and production-disabled status.
- Draft-edit update/reset controls are insensitive and have no mutation, persistence, or executor handler.
- Source/include and duplicate executors remain `Unwired`.
- Source/include and duplicate production flags remain false.
- No disk persistence, real config mutation, runtime mutation, reload, AGS/Waybar touch, release artifact change, or migration activation was added.
