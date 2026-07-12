# Structured-Family Active Config Write Pilot

The first active real config write pilot is implemented, heavily gated, rehearsed — and currently blocked by one environmental gate.

## What exists

`src/structured_family_active_config_pilot.rs` is the only path in the app that could ever write the user's active Hyprland config, and it cannot be triggered accidentally:

- It is unreachable from the UI, `main`, and the scalar write flow (test-enforced).
- The normal test suite never executes it; the live pilot lives behind an ignored, environment-gated test.
- It requires an explicit approval object with the exact typed confirmation phrase.
- Its preflight evaluates fifteen hard gates immediately before the write, and any failure aborts before a single byte is touched: approval, typed confirmation, target identity (active config, regular file, not a symlink), target existence, minimal reversible change (every original record preserved plus exactly one appended single-line record), backup path outside the active config area, rehearsal proof, rehearsal freshness (content-hash drift detection), rollback plan, post-write and post-restore verification plans, autoreload-disabled evidence, no reload, no runtime mutation, no automatic apply path.

The pilot change is the safest available: one unused named bezier curve (`hl.curve`) that references nothing and has zero behavioral effect. The pilot writes once atomically, verifies through the parser/projection path, then restores the original bytes and verifies the restoration byte-exactly — the config ends exactly as it began, with FNV-1a content hashes recorded in the receipt.

## Rehearsal: proven

The copied active-config rehearsal ran for real: the actual `~/.config/hypr/hyprland.conf` content was copied into a temp rehearsal root and the controlled executor round-tripped it — byte-exact backup, appended inert curve record, reread verification, restore, restore verification — while the real file was proven untouched by content hash and byte comparison.

## Live pilot: blocked, and by what

Read-only evidence: `hyprctl getoption misc:disable_autoreload` returns `bool: false`. Autoreload is enabled (the Hyprland default), so the running compositor reloads the config the moment the file changes. That makes any active-config write an immediate live runtime mutation — and runtime mutation is not approved. The `AutoreloadDisabledConfirmed` and `NoRuntimeMutationPlanned` gates therefore fail, and the pilot refuses to write. The active config was never written; its SHA-256 remains `efbb732063fccf35c99ad142a656de9041262d87558ee7bb2044f484a04acb7c`.

## How to unblock

Either:

1. Set `misc:disable_autoreload = true` in the Hyprland config (a user decision, itself a config change), or
2. Explicitly approve the one-shot compositor reload that autoreload would cause when the pilot writes and restores.

Then re-verify the option read-only and run:

```sh
HYPRLAND_SETTINGS_RUN_ACTIVE_PILOT=1 \
HYPRLAND_SETTINGS_AUTORELOAD_DISABLED_CONFIRMED=true \
cargo test --test structured_family_active_config_pilot -- --ignored
```

Even then, every gate re-evaluates immediately before the write and the pilot restores the original bytes after verification.
