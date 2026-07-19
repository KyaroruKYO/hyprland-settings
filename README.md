# Hyprland Settings

Hyprland Settings is a Rust + GTK4/libadwaita application for reviewing and changing Hyprland configuration with explicit validation, recovery, and runtime-safety gates. The bundled data model targets Hyprland `0.55.2`.

## Release And Branch Status

`v0.2.0` is published from commit `0ffaeb3`. The active `structured-family-editors-unified` branch contains substantial **unreleased post-v0.2.0 work**. The counts and behavior below describe that active branch, not the published binary.

## Current Active-Branch Surface

- 341 official scalar rows modeled and readable.
- 290 scalar rows exposed as editable: 135 guarded live-preview rows, 38 supervised dead-man rows, and 117 save-only rows.
- 51 high-risk scalar rows deliberately blocked by production gates.
- Seven structured families classified.
- Active-config persistence exists only for proven existing-record shapes in `hl.animation` and `hl.curve`.
- `hl.monitor`, `hl.bind`, `hl.gesture`, `hl.device`, and `hl.permission` remain blocked from production persistence.

The metadata pipeline classifies all 341 rows, but that does not mean every row is currently editable or safe to apply. Production behavior is limited by the counts and gates above.

## Save And Runtime Safety

Runtime preview uses guarded, reversible runtime mutation through fixed-shape Hyprland configuration expressions. Supervised rows retain Revert/Cancel recovery until a durable config-save receipt succeeds. Hyprland Settings never runs `hyprctl reload`.

Production scalar Save exists behind Safe Live Save Mode and the row's write gates. A save now:

1. rechecks the complete source graph and exact target-file precondition;
2. rejects byte, inode, metadata, target, occurrence, or source mapping drift;
3. creates an exclusive verified backup under the application XDG state directory;
4. stages and validates the complete candidate;
5. commits through one synchronized atomic exchange;
6. rereads and verifies the result;
7. restores exact original bytes on post-write verification failure;
8. marks UI/runtime state saved only after the durable receipt succeeds.

The user-facing drift result is fail-closed: nothing is written, pending state remains, and the user must reread before saving again.

Eligible missing/default insertions require a fresh proof that the setting is
still absent. Duplicate occurrence changes are detected as conflicts; the app
does not auto-resolve duplicate settings.

## Pending Batch Semantics

`Save all atomically` preflights all pending scalar rows before creating a backup or writing.

- Rows targeting one file are combined into one staged document and one atomic target replacement.
- Rows targeting multiple files are rejected before any write. The app does not claim cross-file crash atomicity.
- Any validation, drift, staging, backup, commit, or verification failure retains every pending row.
- Pending rows clear only after the whole supported one-file batch succeeds.

The detail pane stages reviewed changes into this batch; it does not perform an alternate immediate write.

## Backup And Filesystem Guarantees

Active-config backups use `$XDG_STATE_HOME/hyprland-settings/backups`, falling back to `~/.local/state/hyprland-settings/backups`. The backup directory is owned by the current user with mode `0700`; backup files are unpredictable, exclusive, mode `0600`, synchronized, and byte/hash verified.

Target replacement rejects symlinks and non-regular files, checks parent and target identity, preserves mode and same-owner/group assumptions, synchronizes file and parent directory state, and verifies final bytes and metadata. Linux `renameat2(RENAME_EXCHANGE)` is required for the commit-boundary race check.

ACLs, extended attributes, and timestamps are not preserved as an implementation guarantee. A target whose ownership cannot be reproduced safely is rejected.

## Structured Families

The app can save proven modify-existing record shapes for Animation and Bezier Curve records. Those paths use the same Safe Live Save Mode, target identity, backup, drift, atomic exchange, reread, and recovery guarantees. Record creation/deletion, style editing, and the other five structured families are not enabled.

## Hermetic Tests

The normal `cargo test` suite uses fixtures, temporary directories, mock runners, and deterministic report comparisons. It does not read the real Hyprland config, invoke the live compositor, mutate runtime, write active config, or regenerate tracked reports.

Real-machine audits and live proofs are ignored and require explicit environment gates. Tracked report regeneration separately requires `HYPRLAND_SETTINGS_REGENERATE_REPORTS=1`.

See:

- `docs/SAVE-WRITE-STABILIZATION.md`
- `docs/PENDING-SAVE-TRANSACTION-SEMANTICS.md`
- `docs/HERMETIC-TEST-STABILIZATION.md`
- `docs/CURRENT-PROJECT-HANDOFF.md`

## Run From Source

```sh
cargo run --bin hyprland-settings
```

With an explicit metadata export:

```sh
cargo run --bin hyprland-settings -- data/exports/hyprland-0.55.2
```

Build the current branch:

```sh
cargo build --release
./target/release/hyprland-settings
```

## Validation

```sh
cargo fmt --check
cargo check
cargo test
cargo clippy --all-targets
jq empty data/reports/*.json
git diff --check
cargo build --release
```

## Project Boundary

Hyprland Settings is not an official Hyprland project. It does not claim upstream endorsement, universal live-mutation proof, automatic duplicate resolution, profile switching, broad source/include activation, or production support for data newer than the pinned `0.55.2` model.
