# Config Persistence Validation

The live-validation harness can prove rollback safety for runtime probes, but the current Batch A semantics run did not produce strict Level 3 proof. `hyprctl keyword` succeeded for sampled rows and `configerrors` stayed empty, while `hyprctl getoption` stayed at the original value.

The config-persistence harness now provides that different proof path for Batch A.

## Current Policy

Automatic write enablement requires strict live-observable Level 3 proof:

- `hyprctl keyword` succeeds
- `hyprctl configerrors` remains clean
- `hyprctl getoption` changes to the candidate value
- rollback restores the original value and is verified

`keyword` success with unchanged `getoption` is classified as `level3-accepted-unobservable`. It is not enough for automatic enablement.

## Config-Persistence Strategy

The implemented harness validates config persistence instead of live runtime observability:

1. Write a candidate setting only to a temporary fixture config.
2. Keep real `~/.config/hypr/hyprland.conf` untouched.
3. Validate parser round-trip and scalar writer round-trip.
4. Validate the generated temp config with Hyprland's official `--verify-config` path.
5. Capture config errors from the temp-config validation environment only.
6. Do not run `hyprctl reload` against the user's active session.
7. Do not treat active-session `keyword` success alone as proof.

Official source review found that `hyprctl eval` is Lua-manager-only and executes Lua. It is not a safe validation shortcut for this app.

The policy is strict config-persistence validation: parser/writer roundtrip is required, and promotion also requires a successful temporary config validation with no active config or active runtime mutation.

## Current Batch A Result

Batch A likely safe booleans passed config-persistence validation:

- Batch A rows processed: 39
- Parser roundtrip passed: 39
- Writer roundtrip passed: 39
- Typed validator passed: 39
- Single mutation verified: 39
- `Hyprland --verify-config` passed: 39
- Rows enabled by config-persistence proof: 39
- Writable rows after Batch A: 94 / 341

The active `hyprland.conf` was not modified, and the active Hyprland runtime was not mutated.

## Deferred Work

- Continue using exact local command shape `Hyprland --verify-config --config <temp-file>`.
- Improve temp directory cleanup/preservation policy if future reports should not retain temp paths.
- Keep nested/headless Hyprland validation deferred unless `--verify-config` is insufficient.
- Keep high-risk rows out of this validation path until a dedicated high-risk design exists.

Detailed design: [CONFIG_PERSISTENCE_VALIDATION_DESIGN.md](CONFIG_PERSISTENCE_VALIDATION_DESIGN.md).

## Current Reports

```sh
jq '.counts' data/reports/live-validation-semantics.v0.55.2.json
jq '.counts' data/reports/live-validation-batch-a-semantics-classification.v0.55.2.json
```

Current Batch A config-persistence decision: enabled after strict temp-config proof.
