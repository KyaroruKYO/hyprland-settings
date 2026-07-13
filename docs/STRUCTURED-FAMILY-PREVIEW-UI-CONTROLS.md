# Structured-Family Preview UI Controls

The two live-proven structured families now have real supervised preview controls on the Config page.

## The controls

**Structured-family live preview (proven records)** card:

- **Global animation speed** (`hl.animation`, record `global`): spin control (0.1–20) with "Preview with recovery".
- **Default curve first control point Y0** (`hl.curve`, record `default`): spin control (−1–2) with the same flow.

Each control runs the supervised lifecycle: Preview with recovery applies the validated value live (readback-verified) and starts a 10-second countdown; **Keep changes** stops the countdown (session-only); **Revert now** and **Cancel** restore the exact original values with readback verification; timeout auto-reverts; navigating away or closing the app reverts unconfirmed previews. Save to config is shown disabled with its reason — persistence goes through the pilot path under Safe Live Save Mode.

## Scope enforcement (test-backed)

- **Modify-existing only**: the controller (`src/structured_family_preview_controller.rs`) refuses to run when the record is missing from the runtime readback; creation and deletion operations do not exist in the module.
- Only the two proven targets can be expressed; monitor/bind/gesture/device/permission cannot appear (guard-tested).
- All values are validated (finite, range-limited) before the fixed-shape command is built; the UI builds no commands.
- A family arms only with a `PROVEN_FAMILY_RECORD_PROOFS` receipt.

## Proof

- 4 model tests (simulated runner tracking live state through commands): preview/keep/revert with verification, timeout auto-revert, modify-existing + validation enforcement, source guards.
- Live UI-controller proofs (env-gated, run once each): `hl.animation` 8.00 → 8.5 → 8.00 exact; `hl.curve` 0.75 → 0.76 → 0.75 exact.
- GTK/AT-SPI evidence: both control labels, "Preview with recovery", "Keep changes", and the Save-disabled reason all present; safety flags false.
