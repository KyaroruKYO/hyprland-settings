# Config Persistence Validation Design

This document defines a safe proof path for scalar settings that are valid Hyprland config entries but are not live-observable through `hyprctl getoption`.

## Current Problem

Batch A has 39 likely safe boolean rows. The existing live-validation harness proved parser/read and fixture write/reread safety for all 39 rows, and previous rollback-protected probes restored runtime state for every row tested.

Strict live-observable Level 3 proof still failed:

- `hyprctl keyword` did not produce a candidate-visible `getoption` change.
- Sampled diagnostics captured the non-legacy parser message: `keyword can't work with non-legacy parsers. Use eval.`
- `hyprctl configerrors` stayed clean.
- Revert verification passed.
- No Batch A rows were enabled.

That evidence is not enough to promote rows to writable status. A separate config-persistence proof is needed.

## Official Findings

Official Hyprland source shows that `hyprctl eval` is Lua-manager-only and executes Lua code. It is not a pure config snippet validator and must not be used by this app because the safety rules forbid Lua execution/evaluation.

Official Hyprland source also exposes `--config FILE` and `--verify-config`. The `--verify-config` help text says Hyprland should not run normally and should only print whether the config has errors. The explicit config path and `HYPRLAND_CONFIG` path selection make a temporary config file validation path viable.

## Rejected Approaches

- Active-session `hyprctl keyword` is not config-persistence proof for non-legacy parser sessions.
- `hyprctl eval` is rejected for this app because it executes Lua.
- `hyprctl reload` is rejected because it mutates the active session.
- Parser/writer fixture roundtrip alone is required but not sufficient for promotion.
- Nested/headless compositor validation is deferred until `--verify-config` is proven insufficient.

## Recommended Policy

Use strict config-persistence validation.

A future implementation may promote an accepted-unobservable scalar row only after all of these pass:

- Rust parser reads the setting.
- Typed validator accepts the candidate and rejects invalid values.
- Scalar writer can replace and append the setting in temporary fixtures.
- Generated temporary config contains only the intended candidate mutation.
- `Hyprland --verify-config` validates the temporary config with no config errors.
- Active `hyprland.conf` is not modified.
- Active Hyprland runtime state is not modified.
- Coverage report, UI/model projection, and tests are updated in the implementation sprint.

Parser/writer roundtrip alone is not enough for automatic enablement.

## Future Harness Shape

Recommended future command:

```sh
cargo run --bin hyprland-settings -- validate-config-persistence --batch batch-a-likely-safe-booleans --dry-run
```

Future optional command after command-shape verification:

```sh
cargo run --bin hyprland-settings -- validate-config-persistence --batch batch-a-likely-safe-booleans --verify-hyprland
```

The harness should:

- create a temporary directory;
- generate a temporary Hyprland config fixture;
- apply one candidate setting at a time through the scalar writer;
- reread and verify through the Rust parser;
- run `Hyprland --verify-config` against the temp config under a timeout;
- serialize results to `data/reports/config-persistence-validation-results.v0.55.2.json`;
- leave the active config and active compositor untouched.

## Batch A Impact

All 39 Batch A rows remain blocked in this sprint.

- 3 rows are accepted-unobservable from prior live semantics diagnostics.
- 36 rows are unproven by strict live-observable policy.
- 39 rows are candidates for a future temp-config validation implementation sprint.
- 0 rows are safe to enable now.

See:

```sh
jq '.counts' data/reports/batch-a-config-persistence-candidates.v0.55.2.json
jq '.recommendedApproach' data/reports/config-persistence-validation-design.v0.55.2.json
```

## Next Implementation Sprint

Implement the dry-run config-persistence harness first. Do not enable Batch A in the same step unless the harness, tests, `Hyprland --verify-config` command behavior, and generated results prove the full policy.
