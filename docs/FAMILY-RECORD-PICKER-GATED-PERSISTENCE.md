# Family Record Picker — Gated Persistence

Machine-readable result: `data/reports/family-record-picker-gated-persistence.v0.55.2.json`.

## What this is

The record picker generalizes structured-family gated persistence from two
hardcoded records (hl.animation `global` speed, hl.curve `default` Y0) to
**proven record shapes**, one shape at a time:

- **hl.animation — record speed**: the speed of any animation record that
  already carries an explicit override. Other fields (enabled, bezier,
  style) are re-rendered exactly as the runtime readback reports them, so
  a styled record like `workspaces` keeps its style on save.
- **hl.curve — control points**: all four control points (X0, Y0, X1, Y1)
  of any bezier curve that exists in the readback. The proven runtime
  command always writes all four points, so editing any point is the same
  proven command shape.

## Proof before support

A shape receipt is recorded in `PROVEN_RECORD_SHAPE_PROOFS`
(`src/structured_family_runtime_preview.rs`) only after an env-gated live
round trip passed on a record that is NOT the original family-proof record:

- hl.animation: `fade` speed 3 → 3.25 → 3, full-record zero residue
  (2026-07-13).
- hl.curve: `quick` X1 0.1 → 0.11 → 0.1, zero residue on all four points
  (2026-07-13).

The picker classification consults these receipts; without a receipt the
shape is blocked. The save path (`gated_family_record_save`) requires the
receipt again independently.

## Honest per-record classification

The picker lists the records that exist in the runtime readback and
classifies each one:

| Record kind | Support | Why |
| --- | --- | --- |
| overridden, enabled, style-free | preview + Save | the proven shape |
| style-bearing (e.g. `workspaces`) | Save only | style preservation through the runtime command is not proven; Save renders the config line with the style preserved |
| disabled at runtime (e.g. `borderangle`) | Save only | **found by live proof**: the compositor does not apply speed changes to disabled records, so a preview cannot be readback-verified |
| inherited (no explicit override) | blocked | saving would create a new override — record creation is blocked |
| internal (`__`-prefixed) | blocked | compositor internals, not user configuration |
| unsafe record name | blocked | names are interpolated into fixed-shape expressions; fail closed |

Existing bezier curves are all supported (they can be redefined and
exactly restored — proven). Record creation and deletion do not exist
anywhere in the picker or the persistence path.

## Save path

`family_record_picker::save_picked_record` only builds a
`FamilyRecordSaveRequest`; every gate runs inside
`structured_family_gated_persistence::gated_family_record_save`:

1. shape proof receipt required (fail closed),
2. value validation (speed 0.1..=20; X 0..=1; Y -1..=2) and safe-name
   check,
3. Safe Live Save Mode verified live (the write cannot reload the
   compositor),
4. target identity: only the discovered active config,
5. render from the readback (modify-existing enforced: missing and
   inherited records refuse),
6. byte-exact backup outside the config directory, verified readable,
7. one atomic write replacing only the target record's own line
   (replace-or-append; every other line preserved),
8. reread verification through the parser; verification failure restores
   the backup automatically.

Live save flow proof passed for both families on 2026-07-13
(`tests/family_record_picker_live.rs`, env-gated) with byte-exact
flow-proof restores.

## UI

The Config page's structured-family card now shows, per family, a record
dropdown built from the live readback, the current value, value controls
(speed spin / four control-point spins), Preview with recovery (dead-man
countdown, Keep, Revert now, Cancel; session-drop revert), and Save
previewed value through the gated path. Records that cannot be picked are
listed under an expander with their reasons; save-only records show the
preview limitation inline.

## Tests

- `tests/family_record_picker.rs`: classification honesty, parsers,
  fixed-shape expressions, controller round trips with mocks, gate
  ordering, modify-existing enforcement, style preservation, source
  guards (no file writes, no process spawning, no other family, no
  creation/deletion, save routed through gated persistence only).
- `tests/family_record_picker_live.rs` (env-gated, ignored): the two
  shape proofs, the controller live round trips, and the real save flow
  proof.
