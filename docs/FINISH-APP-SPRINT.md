# Finish-App Sprint

> Historical sprint record. It describes the repository at that sprint's
> checkpoint, not the current unreleased post-v0.2.0 implementation. See
> `CURRENT-PROJECT-HANDOFF.md` and `PROJECT-STATUS.md` for current truth.

This sprint pushed the structured-family write track as far as the safety gates allow, cleaned commit attribution, hardened the executor, added the active-config pilot, and integrated a review-only GUI status surface.

## Completed

- **Attribution cleanup**: the two commits carrying `Co-Authored-By` trailers were rewritten in place (`78c5bbc` → `bc07719`, `ef64b4f` → `9a1ee7c`) preserving author/committer identity and dates. No project text references existed. Local branches and tags verified clean; backup refs removed after verification.
- **Executor hardening**: fixed a symlink-through-target-file escape (a target file symlinked to a foreign path outside the controlled root now resolves to `SymlinkEscapeRejected`), and converted all target writes and restores to atomic temp-file-plus-rename so a crash mid-write cannot truncate a config. Regression tests added.
- **Active-config pilot** (`src/structured_family_active_config_pilot.rs`): fifteen-gate preflight, typed confirmation phrase, rehearsal-freshness drift detection, single atomic write, mandatory restoration, receipts and audit records with content hashes. Unreachable from UI/main/write_flow; live execution only via an ignored env-gated test.
- **Copied active-config rehearsal**: proven against the real config content in a temp copy; the real file was never modified.
- **Live pilot**: blocked by the `AutoreloadDisabledConfirmed` / `NoRuntimeMutationPlanned` gates — `misc:disable_autoreload` is `false` on this system, so a config write would live-reload the compositor. See `docs/STRUCTURED-FAMILY-ACTIVE-CONFIG-WRITE-PILOT.md` for the exact unblock path.
- **GUI**: a review-only, report-backed "Structured-family write status" card on the Config page with a permanently insensitive Apply control labeled as blocked pending pilot approval. Proven in the GTK evidence matrix via AT-SPI text (evidence root `/tmp/hyprland-settings-gtk-automation/20260711_224219`); all safety flags false.

## Unchanged (still gated)

- Active real config writes: not performed; SHA-256 of the active config unchanged.
- Hyprland reload, mutating hyprctl, runtime mutation: never run.
- GUI live Apply controls: disabled; no one-click live write exists.
- Hyprland 0.55.4 migration: advisory only, blocked by missing trusted official export data.
- Release: `v0.1.0` tag and `dist/v0.1.0` artifacts preserved; `main` untouched.

## The single remaining boundary

Everything up to the active-config write is implemented and proven. The one remaining step — the first live write — is blocked only by the compositor's autoreload behavior and requires an explicit user decision: disable autoreload or approve the reload the write would trigger.
