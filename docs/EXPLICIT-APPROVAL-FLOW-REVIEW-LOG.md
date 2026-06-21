# Explicit Approval Flow Review Log

## Scope
The feature branch now has a reusable approval model for future production-gated capabilities:

- Source/include insertion
- Duplicate replacement
- Structured `hl.bind` write
- Profile/mode switch
- Runtime keyword
- Runtime reload
- High-risk/display write
- Hyprland 0.55.4 migration

## Requirements
Each approval must name the exact scope, target path or runtime command, old state, proposed new state, restore plan, and copied-config-tree or live-restore proof. Tokens are expiring/one-shot. Approval does not enable production by default.

## Test Evidence
The deterministic model tests cover missing approval, wrong scope, expired token, rejected approval, missing proof, copied-proof approval, and live-restore-proof approval while keeping production flags false.

## Production State
No future capability was enabled by this approval model. All production gates remain default-disabled.
