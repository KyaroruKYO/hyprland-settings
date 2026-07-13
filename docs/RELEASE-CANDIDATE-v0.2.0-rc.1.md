# Release Candidate v0.2.0-rc.1

Machine-readable result:
`data/reports/release-candidate-v0.2.0-rc.1.v0.55.2.json`.
Release notes: `docs/releases/v0.2.0-rc.1.md`.

## Status

**Release-candidate prepared locally.** The version is bumped to
`0.2.0-rc.1`, the metadata carries the release entry, local artifacts build
reproducibly into `dist/v0.2.0-rc.1/`, and the full validation suite
passes. **No tag was created, nothing was merged to main, nothing was
published**, and `v0.1.0` / `dist/v0.1.0` are untouched (checksums
verified).

## Why no RC tag was created

The tag gates require a clear repo convention for prerelease tags. The repo
has exactly one release tag (`v0.1.0`, annotated) and no prerelease tag
precedent, so an RC tag convention does not clearly exist. Tagging remains
a separate final step; if wanted, it is one command on the reviewed commit:
an annotated `v0.2.0-rc.1` tag mirroring the `v0.1.0` tag style. The
candidate is complete without it.

## Completed RC checklist

- [x] `Cargo.toml` / `Cargo.lock` version `0.2.0-rc.1` (cargo semver
      prerelease verified by `cargo check`).
- [x] `CHANGELOG.md` section for 0.2.0-rc.1.
- [x] AppStream metainfo `<releases>` entry (`0.2.0~rc.1`,
      `type="development"`); `appstreamcli validate` passes.
- [x] `desktop-file-validate` passes.
- [x] Release notes draft: `docs/releases/v0.2.0-rc.1.md`.
- [x] Artifact builder: `tools/build_release_candidate_artifacts.sh`
      (build-only; refuses version mismatch; refuses to overwrite; verifies
      `dist/v0.1.0` checksums after building; never tags or publishes).
- [x] Local artifacts in `dist/v0.2.0-rc.1/` (binary tarball mirroring the
      v0.1.0 layout, source tarball from the RC commit, SHA256SUMS).
- [x] Full validation suite green (fmt, check, test, report JSON validity,
      release build, git diff check).
- [x] Pinning tests: `tests/release_candidate.rs`.

## Remaining before final v0.2.0 (user-approved steps)

1. Optional: annotated `v0.2.0-rc.1` tag on the reviewed commit (see
   above); push the tag.
2. One manual pass of the test plan below on a real session, using the RC
   binary.
3. User approval of the final release.
4. Merge `structured-family-editors-unified` → `main`.
5. Version bump `0.2.0-rc.1` → `0.2.0`, final changelog/metainfo entries.
6. Annotated `v0.2.0` tag, `dist/v0.2.0/` artifacts, GitHub release with
   the reviewed notes.

## Manual test plan (one real session, RC binary)

1. Unpack `dist/v0.2.0-rc.1/hyprland-settings-v0.2.0-rc.1-linux-x86_64.tar.gz`
   and launch the binary in a live Hyprland session; confirm the dashboard
   and Config page render.
2. Preview a scalar (e.g. `general.gaps_in`) live; revert; confirm the
   runtime value restores.
3. Arm a dead-man preview (e.g. `animations.enabled`); let it time out;
   confirm auto-revert.
4. Enable Safe Live Save Mode; confirm the status card flips and saves
   unblock.
5. Save one scalar; confirm the config line, the backup path in the
   receipt, and that no reload happened.
6. In the record picker, pick an animation record; change speed; toggle
   the enabled switch; pick a different existing curve; Preview; Keep;
   Revert now; confirm readback each time.
7. Save a curve control point; confirm the `bezier =` line in the config.
8. Press "Save as default" for Safe Live Save Mode; confirm
   `misc:disable_autoreload = true` appears in the config; remove it if
   not wanted.
9. Close the app during a counting-down preview; confirm the value reverts
   (session-drop recovery).

## Known limitations

- 18 touch-family rows and 3 secondary-device rows stay unproven pending
  hardware; they are disarmed, not broken.
- monitor/bind/gesture/device/permission families stay blocked for editing
  with honest reasons.
- The animation style field is not editable (no trusted evidence for valid
  values); it is preserved on save. Styled records are save-only for
  preview purposes.
- While an animation record is disabled, the compositor resets its
  speed/bezier readback (live-proof finding); the preview verifies the
  enabled flag only in that state, and reverts restore the full record.
- The scalar model projects flat `section:key = value` syntax (what the
  app writes); block-syntax values in hand-written configs are not shown
  as current values.
- The data model tracks v0.55.2; 0.55.4 was audited at zero drift, so no
  migration is needed.

## Upgrade notes (v0.1.0 → v0.2.0-rc.1)

- No config-format changes and no forced writes: the app never modifies
  your config except through the gated Save you invoke.
- New requirement for saving: Safe Live Save Mode must be active at
  runtime (the app offers to enable it; enabling is runtime-only and
  reversible). v0.1.0-style safe-batch writes now also route through this
  gate.
- Optional persisted mode: "Save as default" writes
  `misc:disable_autoreload = true` — only when you press it.

## Rollback notes

- Binary rollback: run the v0.1.0 binary from
  `dist/v0.1.0/hyprland-settings-v0.1.0-linux-x86_64.tar.gz` (checksums in
  `dist/v0.1.0/SHA256SUMS`).
- Config rollback: every gated Save leaves a byte-exact backup (path shown
  in the save receipt; scalar saves back up under
  `~/.local/state/hyprland-settings/backups/`). Restoring the backup file
  restores the pre-save config exactly.
- Runtime rollback: Safe Live Save Mode is runtime-only; disable it from
  the app card or restart Hyprland. If you persisted it, remove the
  `misc:disable_autoreload = true` line (or restore the save's backup).

## Artifact manifest

`data/reports/v0.2.0-rc.1-release-artifact-manifest.json`. Checksums are
authoritative in `dist/v0.2.0-rc.1/SHA256SUMS` (not embedded in the
committed manifest: the source archive is built from the commit that
contains the manifest, so embedded checksums could never match
themselves).
