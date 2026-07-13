# Manual RC Test — v0.2.0-rc.1 (2026-07-13)

Machine-readable result: `data/reports/manual-rc-test-v0.2.0-rc.1.v0.55.2.json`.
Evidence root: `/tmp/hyprland-settings-rc-manual-test/20260713_102653`.
Tooling: `tools/live_scenario_harness/run_manual_rc_test.sh` +
`tools/live_scenario_harness/manual_rc_test_driver.py`.

## Result: PASSED (10/10 driven steps)

The **packaged** v0.2.0-rc.1 binary artifact was checksum-verified,
unpacked, and launched in the **real** Hyprland 0.55.4 session with the
real HOME. An AT-SPI driver performed real interactions on the running
app; every mutating step was verified through read-only `hyprctl` readback
and finished with zero residue. The wrapper proved the active config bytes
(sha256), `gaps_in`, monitor geometry, and the autoreload flag were
identical before and after the whole run.

## What was driven (all passed, method: real interaction, automation-driven)

1. RC binary launch from `dist/v0.2.0-rc.1/` — app on the AT-SPI bus.
2. Dashboard renders.
3. Config page renders (sidebar navigation; record picker + Safe Live Save
   Mode card present).
4. Record picker shows **real-session** readback values (label matches the
   live `hyprctl animations` output).
5. Animation record preview — Enabled switch on + speed +0.25 → runtime
   changed (the new **enabled shape** through the real UI) → **Revert now**
   → full-record zero residue.
6. Dead-man countdown: second preview armed, runtime changed, countdown
   text visible, **timeout auto-reverted** the runtime with no further
   interaction.
7. Curve record preview (X1 +0.01) → runtime changed → Revert now → zero
   residue.
8. Safe Live Save Mode: Enable → `misc:disable_autoreload` readback `true`
   → Disable → back to `false` (pre-test state). Runtime-only.
9. Gated Save controls present with their gate descriptions.
10. Zero runtime residue overall (byte-identical animations listing,
    restored autoreload, unchanged config hash, unchanged monitors).

## What was NOT tested through the RC UI (recorded honestly)

- **Save previewed value** and **Save as default** clicks: the controls'
  own descriptions prohibit GTK automation from activating them (they
  write the active config). They remain a human step. Their exact code
  paths are live-flow-proven by env-gated tests that wrote the real config
  with byte-exact backups and restores (re-passed 2026-07-13).
- **Scalar `general:gaps_in` preview via the settings list**: row selection
  was not driven this pass; the scalar preview controller is live-proven by
  env-gated round trips, and the same supervised-preview flow was exercised
  through the RC UI on the record picker.
- **Session-drop (close during countdown)**: SIGTERM-based closing would
  bypass the in-app revert and could leave residue, so it was not driven;
  the countdown **timeout** auto-revert (the stronger unattended-recovery
  property) was proven through the RC UI instead, and session-drop revert
  is unit- and live-proven at the controller level.

## Verdict for the final release

**Pass.** The packaged RC binary works against the real session; no
failure was observed in any driven step; the untested UI clicks are
config-writing controls whose underlying gated paths are live-flow-proven.
