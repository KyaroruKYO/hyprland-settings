# Safe Live Save Mode

Safe Live Save Mode is the app's answer to the autoreload problem: real-time changes come from live runtime preview, persistence is one config write after preview, and that write must never trigger an accidental compositor reload loop.

## The proven strategy: runtime-first

`misc:disable_autoreload` can be controlled **at runtime** — no file write, no reload, getoption-verified, instantly reversible. Live-proven on 2026-07-12:

1. Runtime set to `true`, verified; runtime revert to `false`, verified.
2. With the runtime flag `true`, the first active-config write pilot ran through its designed fifteen-gate path: it wrote one inert record to the real `hyprland.conf`, verified via the parser, restored the original bytes (SHA-256 identical before and after), and verified the restoration.
3. **Reload avoidance proven by marker**: the runtime flag stayed `true` through the write and restore. If autoreload had fired, the reload would have re-applied the config's `false`. It didn't.

So the write+reload approval loop is unnecessary by design: enable the mode (runtime-only), preview live, save once, optionally restore the flag.

## The UI

The Config page's "Safe Live Save Mode (recommended)" card shows the live `misc:disable_autoreload` value, whether the mode is active, why saves are blocked when it isn't, and live **Enable/Disable** buttons. The buttons route through `src/safe_live_save_mode.rs`, which issues fixed constant commands (no user input reaches command construction) and verifies every transition through read-only readback, failing closed on any mismatch or unreadable state.

Enabling the mode never needs a reload and touches no file. Persisting `misc:disable_autoreload = true` into the config (so the mode survives restarts) is a separate, gated step — and can itself be done reload-free while the runtime flag is already `true`.

## Boundaries

- The mode module contains no file-write APIs, no reload path, and no free-form command construction (guard-tested).
- The status fails closed when the runtime value cannot be read.
- Normal `cargo test` never mutates the compositor; the round-trip proof is ignored and env-gated (`HYPRLAND_SETTINGS_RUN_SAFE_LIVE_SAVE_MODE_PROOF=1`).
