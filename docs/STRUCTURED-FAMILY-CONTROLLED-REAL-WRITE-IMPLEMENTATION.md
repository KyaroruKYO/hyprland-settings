# Structured-Family Controlled Real-Write Implementation

Real write code now exists for structured families — for controlled targets only.

Executor wiring exists only for controlled targets: test-owned fixture files, copied config trees, and temporary config files. Active real config writes are still blocked. Backup/restore/rollback proof exists only for controlled targets. Hyprland reload remains disabled. Runtime mutation remains disabled. GUI live Apply controls remain disabled. The first real active-config write still requires explicit approval.

## Modules

- `src/structured_family_write_target.rs` — controlled target classification and policy.
- `src/structured_family_controlled_write.rs` — the controlled write executor.

Both are exported from `src/lib.rs` and internally wired to the structured-family parse/render/projection pipeline. Neither is imported by `src/main.rs`, `src/write_flow.rs`, or any UI module; no live UI control can reach them.

## Target Policy

`classify_structured_family_write_target` distinguishes:

- `TestOwnedFixtureTarget` — writable when the controlled root is a tests/fixtures-style or temp path.
- `CopiedConfigTreeTarget` — writable when the controlled root is under the system temp directory.
- `TemporaryConfigTarget` — writable when the controlled root is under the system temp directory.
- `ActiveRealConfigTarget` — always rejected. Any path that resolves under the active Hyprland config directory (textually or through symlinks) is reclassified to this kind and refused, regardless of the declared kind.
- `UnknownTarget` — always rejected. Path escapes (`..`), symlink escapes, targets outside the controlled root, disallowed roots, and inconsistent declared kinds resolve here.

The policy's `active_real_config_writable` field is a constant `false`, not a switch.

## Executor Behavior

`execute_structured_family_controlled_write` runs the real write pipeline:

1. Verify approval. The approval model cannot approve active real config writes — that flag being true is itself an error (`ActiveRealConfigApprovalForbidden`).
2. Classify the target; refuse any non-writable policy before reading a byte.
3. Require backup, restore, and verification plans; any missing or weakened plan fails closed. The backup path must stay inside the controlled root.
4. Take an accepted, unblocked, plan-ready staged apply plan as input. Blocked, rejected, invalid, already-executed, or tampered plans are refused as unsafe.
5. Create a byte-exact verified backup.
6. Write the rendered structured-family records, replacing only the family's records and preserving comments, unknown syntax, other families, and scalar lines.
7. Reread the target through the parser/projection path and verify the intended records are present. On verification failure, the original bytes are restored automatically before the error is reported.
8. Emit a write receipt; `structured_family_controlled_write_audit_record` emits the audit record.

`restore_structured_family_controlled_write` executes rollback for controlled targets: it restores the backup, verifies byte-exact restoration, and rereads the restored family records through the parser/projection path. `execute_structured_family_controlled_write_round_trip` runs write → verify → restore → verify in one call and carries the rollback proof in the receipt.

No code path calls `hyprctl`, reloads Hyprland, runs commands, or mutates runtime. The receipt flags `active_real_config_touched`, `hyprctl_reload_run`, and `runtime_mutated` are constant false.

## Proof

`tests/structured_family_controlled_write.rs` proves, with real file writes inside temporary test directories only:

- Full write/backup/reread/verify/restore/verify round trips for all seven families against copied temp targets.
- Non-family lines are preserved across writes.
- The active real Hyprland config is refused, even with a lying declared kind and even when nested under a claimed controlled root.
- Unknown targets, path escapes, symlink escapes, out-of-root targets, and disallowed roots are refused.
- Missing approval, forbidden active-config approval, missing backup/restore/verification plans, out-of-root backup paths, unsafe staged apply plans, tampered linkage, and empty rendered records all fail closed without touching the target.
- Post-write verification failure restores the original bytes.
- Source guards: no reload path, no command runner, no GTK controls, no active config literal, no `write_flow`/`apply_setting_change`/scaffold calls, and no live UI reachability.

## Remaining Boundary

The next step across the boundary is a first active real config write pilot. It requires a fresh explicit user approval and does not exist in code today: no target kind, approval flag, or executor path can reach `~/.config/hypr` as written.

## Runtime preview relationship

Structured-family live preview is a separate, runtime-only path (see `docs/STRUCTURED-FAMILY-RUNTIME-PREVIEW.md`): `hl.animation` and `hl.curve` records that already exist can be previewed and exactly reverted under supervision, proven live with zero residue. Persistence of any structured-family change still goes exclusively through this controlled write path and the gated active-config pilot.
