# Persist Safe Live Save Mode

Machine-readable result: `data/reports/persist-safe-live-save-mode.v0.55.2.json`.

## What this is

Safe Live Save Mode (runtime autoreload disable) previously lived only for
the session: after a restart, Hyprland's default autoreload returned. The
user can now persist `misc:disable_autoreload = true` to the config through
the **already-gated scalar Save**, so the mode is naturally active from
config after restarts.

Nothing is automatic. The Safe Live Save Mode card shows:

- **runtime status**: active / inactive / unknown (read-only readback),
- **persisted in config**: yes / no / unknown (from the same current-config
  projection the rest of the app reads — the flat `misc:disable_autoreload`
  syntax the gated Save writes),
- **Enable / Disable Safe Live Save Mode**: runtime-only transitions
  (no file write, no reload, verified readback),
- **Save as default**: persists the setting once through
  `production_save::gated_scalar_save_live`, enabled only while the runtime
  mode is active and not already persisted.

## Why the runtime gate comes first

The write itself must not be able to reload the compositor. The persist
path verifies live that `misc:disable_autoreload` is already `true` at
runtime before writing (`persist_safe_live_save_mode_gate`, failing closed
on unreadable state), and the gated scalar Save re-verifies the same gate
internally. So the ordering is always: enable the mode for the session
first, then choose to save it as the default.

## Narrow by construction

`src/persist_safe_live_save_mode.rs` can persist exactly one setting with
exactly one value — both module constants. No public function accepts a
setting id or a value. The module never touches files itself: the only
write path is the gated scalar Save (backup first, one write, reread
verification, no reload). Source guards pin all of this.

## Proof

Live flow proof (`tests/persist_safe_live_save_mode_live.rs`, env-gated
`HYPRLAND_SETTINGS_RUN_PERSIST_SAFE_LIVE_SAVE_MODE=1`, ignored) passed
2026-07-13 against the running compositor:

1. persist blocked while the runtime mode was inactive (gate proof),
2. after enabling the mode, the persist wrote once through the gated
   scalar Save with a byte-exact backup and reread-verified `true`,
3. the persisted-state projection flipped to PersistedTrue,
4. flow-proof cleanup restored the pre-test config bytes byte-exactly and
   the original runtime state (this test is a flow proof, not a user
   choice — production behavior does not restore).

Normal `cargo test` never writes the active config: the normal tests use
mock runners and temp files only.
