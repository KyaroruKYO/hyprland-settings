# Structured-Family Completion Marathon

This marathon advanced structured-family live preview and active-config pilot readiness together, following live evidence end to end.

## What changed

- **Live preview**: all seven families are now classified from real probes (see `docs/STRUCTURED-FAMILY-RUNTIME-PREVIEW.md`). Two families were promoted by passed live proofs — `hl.animation` and `hl.curve`, both scoped to modifying existing records with exact readback-verified restore and zero residue. Four families remain blocked high-risk with precise reasons; gestures are blocked for lack of any verification mechanism.
- **Discoveries**: every family has a runtime record API (`hl.<family>` functions with discoverable schemas); `hyprctl animations` is a real verification mechanism; records can never be deleted, so only modify-existing operations are honest preview candidates; `hl.config` silently ignores record keys, so readback verification is mandatory.
- **Active-config pilot**: the gate is now evidence-driven twice over — the ignored pilot test requires both the operator env flag **and** live-collected read-only autoreload evidence (`collect_autoreload_evidence`, which fails closed on read failure, parse failure, or `false`). The readiness report (`structured-family-active-config-pilot-readiness.v0.55.2.json`) records the exact blocker, the two unblock options, and the exact pilot command.
- **UI**: the Config page gained a "Structured-family live preview & persistence" card showing each family's capability, status, and blocked reason, plus the autoreload blocker with the exact instructions. Proven via AT-SPI with all safety flags false.

## The user decision that remains

Your config has `misc:disable_autoreload = false`. Writing `hyprland.conf` would make Hyprland reload immediately, so the pilot will not write while that is true. Either set `misc:disable_autoreload = true` first, or explicitly approve the single reload the write-and-restore pilot would cause, then run:

```sh
HYPRLAND_SETTINGS_RUN_ACTIVE_PILOT=1 \
HYPRLAND_SETTINGS_AUTORELOAD_DISABLED_CONFIRMED=true \
cargo test --test structured_family_active_config_pilot -- --ignored
```

All fifteen preflight gates re-evaluate immediately before the write, live autoreload evidence is re-collected read-only, and the pilot restores your original bytes after verification.

## What remains after this marathon

- Supervised UI controls for the two proven families (modify-existing animation/curve records through a family dead-man panel).
- The active-config pilot decision (yours).
- Hardware-gated work: touch-family scalar proofs, gesture verification, and any monitor/display recovery system.
