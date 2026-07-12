# Structured-Family Runtime Preview

Structured families now have an evidence-based live preview capability model (`src/structured_family_runtime_preview.rs`), built from mutation-free probes and two passed live proofs — not assumptions.

## The evidence

Invalid-argument probes (which error without mutating) revealed that **all seven families have runtime record APIs** in Hyprland's Lua config manager: `hl.animation`, `hl.curve`, `hl.gesture`, `hl.bind`, `hl.device`, `hl.permission`, `hl.monitor`, each with a discoverable schema. `hyprctl animations` provides full read-only readback for animation leaves and bezier curves. Two hard limits shape everything:

1. **Records cannot be deleted**, and animation overrides cannot revert to inherited state (`enabled` is mandatory). So record *creation* is never exactly revertible — only *modifying an existing record* has an exact, readback-verified revert.
2. `hl.config({ animation = ... })`/`hl.config({ bezier = ... })` **silently ignore or reject** record keys while sometimes returning "ok" — readback verification is mandatory because success responses can be no-ops.

## Family classifications

| Family | Capability | Why |
| --- | --- | --- |
| hl.animation | **LivePreviewSupportedWithDeadMan** (modify-existing only) | live proof passed: the explicitly overridden `global` node round-tripped speed 8.00 → 8.5 → 8.00 with readback-verified apply and exact restore, zero residue. Inherited leaves are excluded (no revert-to-inherit exists). |
| hl.curve | **LivePreviewSupportedWithDeadMan** (modify-existing only) | live proof passed: the built-in `default` bezier was redefined via `hl.curve(name, { type = "bezier", points = ... })`, verified, and restored exactly, zero residue. Curve creation is excluded (no deletion). |
| hl.gesture | BlockedNoVerificationMechanism | the API exists, but there is no gesture record readback and no touch hardware to observe behavior |
| hl.monitor | BlockedHighRisk | display changes need a blind recovery system that reverts without user sight |
| hl.bind | BlockedHighRisk | a wrong bind can swallow the keys needed to confirm or revert; needs a proven unbind round trip plus a fallback model |
| hl.device | BlockedHighRisk | a wrong record can disable the confirming device |
| hl.permission | BlockedHighRisk | security policy is never live previewed |

Promotion is receipt-gated exactly like scalar rows: `PROVEN_FAMILY_RECORD_PROOFS` holds the two receipts, tests enforce that no family arms without one and that the scope stays modify-existing, and the four high-risk families can never be promoted this way.

## The proofs

`tests/structured_family_runtime_preview_live.rs` (ignored, env-gated behind `HYPRLAND_SETTINGS_RUN_STRUCTURED_RUNTIME_PREVIEW_LIVE=1` + `HYPRLAND_SETTINGS_STRUCTURED_FAMILY=<family|all>`) captures the record's original values via read-only readback, applies a minimal delta, verifies, restores the exact original, and verifies again. Both proofs ran this marathon and passed with zero residue. Normal `cargo test` never mutates the compositor.

## What a live family preview will look like

The mechanism is proven; the UI controls are the remaining step: a supervised family dead-man panel for existing animation/curve records (capture → throttled modify → countdown → Keep/Revert/Cancel), reusing the existing dead-man controller pattern. Persistence stays separate and goes through the active-config pilot, which remains blocked by the autoreload decision.
