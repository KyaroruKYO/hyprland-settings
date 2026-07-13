# Final Release v0.2.0 (2026-07-13)

Machine-readable results: `data/reports/final-release-v0.2.0.v0.55.2.json`
and `data/reports/v0.2.0-release-artifact-manifest.json`.
Release notes: `docs/releases/v0.2.0.md`.

## What happened, in order

1. **Gates verified**: worktree clean on the reviewed RC commit `82cf18c`;
   `v0.1.0` and `dist/v0.1.0` checksum-verified untouched; no `v0.2.0*`
   tags existed; `gh` authenticated; main fast-forwardable (41 behind, 0
   ahead).
2. **v0.2.0-rc.1 annotated tag** created on `82cf18c` and pushed (message
   says release candidate, not final).
3. **Manual RC test passed** using the packaged RC binary against the real
   session — see `docs/MANUAL-RC-TEST-v0.2.0-rc.1.md` (10/10 driven steps;
   zero residue; active config untouched).
4. **Final version bump** to `0.2.0`: `Cargo.toml`, `Cargo.lock`,
   `CHANGELOG.md`, AppStream stable release entry (validated), release
   notes, reports, and the version-parameterized guarded artifact builder.
5. **Full validation** (fmt, check, tests, jq, diff-check, release build,
   appstream + desktop-file validation).
6. **Release commit** on `structured-family-editors-unified`;
   `dist/v0.2.0/` artifacts built from it (binary tarball, source tarball,
   SHA256SUMS); `dist/v0.1.0` and `dist/v0.2.0-rc.1` verified untouched.
7. **main fast-forwarded** to the release commit (no merge commit, no
   force-push).
8. **v0.2.0 annotated tag** on the release commit; branch, main, and tag
   pushed.
9. **GitHub release published** with the release notes and the three
   artifacts attached.

## Boundaries respected

- No force-push; no existing tag moved; `v0.1.0`, `dist/v0.1.0`, and
  `dist/v0.2.0-rc.1` untouched (checksum-verified).
- No `hyprctl reload`; no monitor/display mutation; normal tests never
  wrote the active config.
- Hardware-gated proofs stayed deferred (no fake proofs, none promoted).

## Rollback

- Binary: run the v0.1.0 or v0.2.0-rc.1 artifacts from `dist/`.
- The release changes no user config; the app's gated Save discipline
  (byte-exact backups, reread verification) is unchanged.
