# Record-Shape Expansion (2026-07-13)

Machine-readable result: `data/reports/record-shape-expansion.v0.55.2.json`.
Live proofs: `tests/record_shape_expansion_live.rs` (env-gated, `#[ignore]`).

## What expanded

The structured-family record picker grew from one proven animation shape
(speed) to three, each behind its own passed live-proof receipt in
`PROVEN_RECORD_SHAPE_PROOFS`:

| Shape | Proven on | Round trip | Result |
| --- | --- | --- | --- |
| `modify-existing-animation-record-enabled` | `border` | enabled 1 → 0 → 1 | passed, zero residue |
| `modify-existing-animation-record-enabled` | `borderangle` | enabled 0 → 1 → 0 | passed, zero residue |
| `modify-existing-animation-record-bezier` | `windows` | easeOutQuint → quick → easeOutQuint | passed, zero residue |

`hl.curve` was re-confirmed complete: the runtime schema carries exactly
the four control points (plus the fixed `type = "bezier"`), so no further
curve shape exists to expand into.

## Live-proof findings

1. **Disabled records reset their readback.** While an animation record is
   disabled, the compositor reports reset speed/bezier values (speed 1.00,
   bezier default) regardless of what was applied. Consequences, encoded in
   the picker: a preview that leaves a record disabled verifies the enabled
   flag only; reverts always restore and verify the full record (proven to
   hold — the reset values are canonical for disabled records).
2. **Disabled records are now preview-supported.** The 0→1→0 proof on
   `borderangle` means the picker can supervise enabling a disabled record
   live; the previous save-only classification for disabled records is
   lifted.
3. **Proofs on shared runtime state must run serially.** The first proof
   run executed the three round trips concurrently and they interleaved on
   the same records; the documented command now pins `--test-threads=1`,
   and the proofs pick disjoint records (sorted-first vs sorted-last).
   Residue from the failed concurrent run was restored manually and
   verified byte-identically before the serial re-run.

## How the proven shapes are wired

- One combined save request, `FamilyRecordSaveRequest::AnimationRecordFields
  { record, enabled, speed, bezier }`, replaces the speed-only request. The
  gate requires **all three** animation shape receipts (fail-closed if any
  is removed). The rendered config line takes enabled/speed/bezier from the
  request and preserves the style from the readback.
- The bezier may only name a curve that already exists in the readback —
  validated in the picker (`validate_animation_bezier`), re-validated in
  the persistence render step, and safe-name-checked in request validation.
- The preview controller applies one fixed expression
  (`render_animation_record_expression`) carrying exactly the three proven
  fields, verifies through the read-only listing (enabled-aware per finding
  1), and reverts to the captured original with full-record verification.
- UI: the animation group gained an Enabled switch and a Curve dropdown
  (existing curves only) beside the speed control; one Preview-with-recovery
  and one gated Save cover the record's proven fields.

## What stays blocked, and why

- **`hl.animation.style`**: not editable anywhere. The set of valid style
  values is not known from trusted evidence (the 0.55.4 descriptions export
  carries none), and style handling through the runtime command has no
  live proof. Shown in the UI as a disabled row with the reason; saves
  preserve the current style unchanged.
- **Gesture family**: no runtime readback listing exists on 0.55.4 (the
  gestures request is unknown to hyprctl), so modify-existing verification
  is impossible; honest proofs would additionally need touch hardware
  (deferred). No picker, no save path, no fake proof.
- **monitor / bind / device / permission families**: unchanged — cannot be
  expressed by the picker or the save request types (guard-tested).
- **Record creation and deletion**: unchanged — the operations do not
  exist in the modules.
