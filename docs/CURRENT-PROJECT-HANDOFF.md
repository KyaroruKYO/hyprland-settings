# Current Project Handoff

## Stabilization Outcome

The active branch is `structured-family-editors-unified`. The stabilization sprint started at `d4d3489` after a read-only audit found Save-state ordering, stale-plan overwrite, write durability, partial batch commit, non-hermetic test, and stale documentation defects.

The write path is now fail-closed and transactionally explicit:

- Save state transitions follow durable write, directory synchronization, reread verification, and receipt creation.
- Injected failures retain pending rows and live-preview recovery.
- Every active scalar and proven structured-family write carries exact file and source-graph preconditions.
- The atomic primitive uses an exclusive same-directory candidate and Linux atomic exchange, verifies the displaced target, and swaps back on drift or verification failure.
- Backups are exclusive, restrictive, synchronized, and verified in the application XDG state directory.
- Restore refuses unverified backups and intervening target edits.
- One-file pending batches commit one staged document in one target exchange.
- Multi-file batches reject before writing because cross-file crash atomicity is not implemented.
- Normal tests are fixture/temp/mock based and cannot regenerate tracked reports without an explicit gate.

## Current Product Truth

- Published release: `v0.2.0` at `0ffaeb3`.
- Current branch: unreleased post-v0.2.0 work.
- Project model: Hyprland `0.55.2`.
- Scalar rows: 341 total; 290 editable; 51 high-risk blocked.
- Editable split: 135 live preview, 38 dead-man, 117 save-only.
- Structured families: seven classified; only `hl.animation` and `hl.curve` proven modify-existing shapes persist to active config.
- Blocked structured families: monitor, bind, gesture, device, permission.
- Runtime preview uses guarded reversible mutation. Explicit Hyprland reload is never used.
- All production Save paths require Safe Live Save Mode and their normal/high-risk/family gates.

## Transaction Contract

The pending action is labeled `Save all atomically`. It means atomic to one file only. The app preflights every row, stages one complete candidate, parser-rereads it, creates one backup, performs one target exchange, verifies the final file, and only then clears all affected pending state.

If rows resolve to multiple files, the operation is rejected before backup or write. If any preflight or commit step fails, every pending entry remains. The detail pane only stages a reviewed value into this batch.

## Drift And Filesystem Contract

The precondition binds the review to exact bytes, SHA-256, canonical target and parent, inode/device, uid/gid, mode, source graph, setting occurrence, raw line, and structured record identity. Any external edit, replacement inode, symlink substitution, occurrence change, missing-setting appearance, or source mapping change rejects the save.

Backups default to `$XDG_STATE_HOME/hyprland-settings/backups` (or the standard HOME fallback), with `0700` directory and `0600` files. Backup collision never overwrites. Restore verifies backup and target before committing.

Residual filesystem limits: ACLs, xattrs, and timestamps are not guaranteed; ownership mismatch fails closed; `RENAME_EXCHANGE` support is required.

## Test Boundary

`cargo test` does not inspect the real HOME config, does not require a Hyprland session, does not run `hyprctl`, does not write active config, and does not rewrite tracked reports. Read-only real-config audits require `HYPRLAND_SETTINGS_RUN_REAL_CONFIG_AUDIT=1`; tracked report regeneration requires `HYPRLAND_SETTINGS_REGENERATE_REPORTS=1`; live mutation/write proofs retain their separate ignored gates.

Two normal full suites and one isolated HOME/XDG suite each passed 1,154 tests
with zero failures and 25 ignored tests. Status, tracked diff, and report hashes
were unchanged after every run; the isolated `hyprctl` trap was not invoked.
The final safe-env GTK matrix produced 28 screenshots and 28 AT-SPI captures at
`/tmp/hyprland-settings-gtk-automation/20260718_write_stabilization`. It did
not click Apply, write config, create backups, reload Hyprland, or mutate
runtime. Read-only `hyprctl -j clients` was used only to crop app windows.

## Do Not Continue

- Do not add settings, structured families, runtime-preview coverage, profiles, source/include activation, display recovery, style editing, hardware proofs, or unrelated UX work before review.
- Do not run active-config write proofs without explicit user approval.
- Do not run `hyprctl reload`, monitor/display mutation, or hardware/input mutation.
- Do not merge, tag, publish, or modify release artifacts as part of stabilization.

## Next Exact Work

Run an independent review of the stabilized write, restore, and pending-state paths before considering feature or release work.

## Historical Record

The prior handoff was a chronological planning log. It remains available at `d4d3489`:

```sh
git show d4d3489:docs/CURRENT-PROJECT-HANDOFF.md
```

Versioned reports and dedicated design/proof documents under `data/reports/` and `docs/` retain the detailed history; they should not be read as current global truth unless explicitly labeled current.
