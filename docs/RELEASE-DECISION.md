# Release Decision

Machine-readable result: `data/reports/release-decision.v0.55.2.json`.

## Decision

**Ready pending user approval.** No tag was created, nothing was merged to
main, no artifacts were built or published, and `v0.1.0` / `dist/v0.1.0`
are untouched. This document prepares the material a v0.2.0 release
candidate would need; the release itself is the user's call.

## Why ready

- The full validation suite passes (fmt, check, tests, report JSON
  validity, release build).
- Every production Save path is gated on Safe Live Save Mode, backed up,
  written once, and reread-verified — proven by env-gated live flow proofs
  against the running compositor, with byte-exact restores.
- The runtime preview system covers all 341 scalar rows with honest
  per-row classifications and live-proof receipts for every armed row.
- The Hyprland 0.55.4 migration audit found zero drift against a trusted
  `hyprctl -j descriptions` export, and the refresh workflow is repeatable
  (`tools/refresh_hyprland_descriptions_export.sh`).
- GTK evidence matrix runs prove the UI renders its safety states.

## Why pending approval (blockers)

1. User approval of this decision.
2. Merge decision: `structured-family-editors-unified` is 34 commits ahead
   of `main`.
3. `Cargo.toml` still says `0.1.0`; a release needs the version bump and a
   tag (suggested `v0.2.0`).
4. The release notes draft below needs a human read.
5. One manual pass of the test plan below on a real session.

## Release notes draft (v0.2.0)

Hyprland Settings v0.2.0 — runtime-first preview and gated persistence.

- **Live preview without config writes**: 135 settings preview instantly
  with revert/cancel; risky rows run under a dead-man countdown that
  auto-reverts unless kept. 36 input/cursor rows were promoted by live
  proofs on real hardware.
- **Safe Live Save Mode**: the app disables Hyprland's config autoreload
  at runtime (no file write, instantly reversible) so that saving cannot
  reload your compositor mid-session. Every Save requires the mode.
- **Gated Save**: saves are written once with a byte-exact backup and
  verified by rereading the config through the parser. Verification
  failure restores the backup automatically.
- **Save as default for Safe Live Save Mode**: persist
  `misc:disable_autoreload = true` through the same gated Save — your
  choice, never automatic.
- **Structured-family record picker**: modify existing animation records
  (speed) and bezier curves (all four control points) with supervised live
  preview and gated Save. Records that cannot be safely edited say why.
  Creating and deleting records stays blocked.
- **Hyprland 0.55.4 compatibility**: audited at zero drift against a
  trusted export from the official binary.

## Changelog draft (since v0.1.0)

- Runtime preview capability system, executor, and UI projection
  (313e6db, 1443bfa).
- Dead-man supervised preview UI; animation candidates armed (ea8c396).
- Input/cursor proof plans and receipt-gated promotion (e4c0fd7, ce61464,
  0eea596).
- Active config write pilot with fifteen gates, hardening, status UI
  (41644ba).
- Structured-family preview and pilot marathon; Safe Live Save Mode
  (2ef1c8f, b1b67e5).
- Evidence-based progress tracker audit (ba7aeb2).
- Hyprland 0.55.4 migration audit, zero drift (0af753f).
- Production Save integration and structured-family gated persistence
  (b3f373c).
- Family record picker, Safe Live Save Mode persistence, 0.55.4 export
  refresh workflow, release decision (this marathon).

## Artifact checklist (for the future release run)

- [ ] Merge `structured-family-editors-unified` → `main` (user approval).
- [ ] Bump `Cargo.toml` version to `0.2.0`.
- [ ] `cargo build --release` clean.
- [ ] Tag `v0.2.0` (annotated, like v0.1.0).
- [ ] Build `hyprland-settings-v0.2.0-linux-x86_64.tar.gz` and source
      tarball into `dist/v0.2.0/` (never touch `dist/v0.1.0/`).
- [ ] `SHA256SUMS` for the new artifacts.
- [ ] GitHub release with the reviewed notes.

## Test checklist

- [ ] `cargo fmt --check`
- [ ] `cargo check`
- [ ] `cargo test` (normal suite; never writes the active config)
- [ ] `jq empty data/reports/*.json`
- [ ] `cargo build --release`
- [ ] GTK evidence matrix (`tools/live_scenario_harness/run_gtk_evidence_matrix.sh`)
- [ ] Optional live flow proofs (env-gated, write + verify + restore):
      `HYPRLAND_SETTINGS_RUN_STRUCTURED_FAMILY_SAVE_LIVE=1`,
      `HYPRLAND_SETTINGS_RUN_FAMILY_RECORD_SAVE_LIVE=1`,
      `HYPRLAND_SETTINGS_RUN_PERSIST_SAFE_LIVE_SAVE_MODE=1`

## Manual test plan (one real session)

1. Launch the app in a live Hyprland session; confirm the dashboard and
   Config page render.
2. Preview a scalar (e.g. `general.gaps_in`) live; revert; confirm the
   runtime value restores.
3. Arm a dead-man preview (e.g. `animations.enabled`); let it time out;
   confirm auto-revert.
4. Enable Safe Live Save Mode; confirm the status card flips and saves
   unblock.
5. Save one scalar; confirm the config line, the backup path in the
   receipt, and that no reload happened.
6. In the record picker, preview an animation record speed; Keep; Revert
   now; confirm readback.
7. Save a curve control point; confirm the `bezier =` line in the config.
8. Press "Save as default" for Safe Live Save Mode; confirm
   `misc:disable_autoreload = true` appears in the config; remove it if
   not wanted.
9. Close the app during a counting-down preview; confirm the value
   reverts (session-drop recovery).

## Known limitations

- 18 touch-family rows and 3 secondary-device rows stay unproven pending
  hardware; they are disarmed, not broken.
- monitor/bind/gesture/device/permission families stay blocked for
  editing with honest reasons.
- The scalar model projects flat `section:key = value` syntax (what the
  app writes); block-syntax values in hand-written configs are not shown
  as current values.
- The data model tracks v0.55.2; 0.55.4 was audited at zero drift, so no
  migration is needed yet.
