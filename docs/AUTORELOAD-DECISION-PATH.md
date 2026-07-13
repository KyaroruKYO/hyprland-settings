# Autoreload Decision Path — Resolved

The long-standing blocker — "writing `hyprland.conf` would reload Hyprland because `misc:disable_autoreload = false`" — is resolved by evidence, not by an approval loop.

## The evidence chain

1. **Runtime control proven**: `misc:disable_autoreload` was set `true` and reverted `false` at runtime with getoption verification. No file was written; no reload command exists in this path.
2. **Genuine gate evidence**: with the runtime flag `true`, the pilot's live evidence collector reads `true` — honestly, because a config write genuinely cannot auto-reload while the runtime flag is set.
3. **The pilot passed** (2026-07-12): all fifteen gates, rehearsal freshness, typed confirmation, backup, one atomic write of an inert record, parser verification, byte-exact restoration (SHA-256 `efbb7320…` identical before and after), restore verification.
4. **No reload fired**: the runtime flag stayed `true` through the entire write+restore — a reload would have reset it to the config's `false`.
5. **State restored**: the runtime flag was returned to the user's original `false` afterward.

## What this means for the product

- Live preview handles real-time visual feedback (runtime APIs, reversible, no writes).
- Save is one gated config write performed while autoreload is runtime-disabled — no reload, no loop, no surprise.
- The user controls the mode explicitly through the Safe Live Save Mode card; the app never silently changes autoreload persistence in the config.

## What was rejected

Repeated write+reload approval cycles as a live-preview mechanism: clunky, sluggish, risky, and now unnecessary.
