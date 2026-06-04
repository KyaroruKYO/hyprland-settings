# Config Persistence Validation

The live-validation harness can prove rollback safety for runtime probes, but the current Batch A semantics run did not produce strict Level 3 proof. `hyprctl keyword` succeeded for sampled rows and `configerrors` stayed empty, while `hyprctl getoption` stayed at the original value.

This means Batch A rows must remain blocked until a different proof path exists.

## Current Policy

Automatic write enablement requires strict live-observable Level 3 proof:

- `hyprctl keyword` succeeds
- `hyprctl configerrors` remains clean
- `hyprctl getoption` changes to the candidate value
- rollback restores the original value and is verified

`keyword` success with unchanged `getoption` is classified as `level3-accepted-unobservable`. It is not enough for automatic enablement.

## Future Config-Persistence Strategy

A future phase may validate config persistence instead of live runtime observability:

1. Write a candidate setting only to a temporary fixture config.
2. Keep real `~/.config/hypr/hyprland.conf` untouched.
3. Validate parser round-trip and scalar writer round-trip.
4. If a temporary isolated Hyprland instance is proven safe, test the temp config there.
5. Capture config errors from the isolated environment only.
6. Do not run `hyprctl reload` against the user's active session.
7. Do not treat active-session `keyword` success alone as proof.

## Deferred Work

- Decide whether nested/headless Hyprland validation is safe on this machine.
- Define how to pass an alternate config path to a test compositor without affecting the active session.
- Define cleanup and timeout behavior for any temporary compositor process.
- Keep high-risk rows out of this validation path until a dedicated high-risk design exists.

## Current Reports

```sh
jq '.counts' data/reports/live-validation-semantics.v0.55.2.json
jq '.counts' data/reports/live-validation-batch-a-semantics-classification.v0.55.2.json
```

Current Batch A decision: enable none.
