# Config Persistence Validation Design

This document defines a safe proof path for scalar settings that are valid Hyprland config entries but are not live-observable through `hyprctl getoption`.

## Current Problem

Batch A has 39 likely safe boolean rows. The live-validation harness proved parser/read and fixture write/reread safety for all 39 rows, and previous rollback-protected probes restored runtime state for every row tested.

Strict live-observable Level 3 proof still failed:

- `hyprctl keyword` did not produce a candidate-visible `getoption` change.
- Sampled diagnostics captured the non-legacy parser message: `keyword can't work with non-legacy parsers. Use eval.`
- `hyprctl configerrors` stayed clean.
- Revert verification passed.
- No Batch A rows were enabled.

That evidence was not enough to promote rows to writable status. The config-persistence harness now provides the separate proof path.

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

## Implemented Harness Shape

Dry-run command:

```sh
cargo run --bin hyprland-settings -- validate-config-persistence --batch batch-a-likely-safe-booleans --dry-run
```

Hyprland temp-config verification command:

```sh
cargo run --bin hyprland-settings -- validate-config-persistence --batch batch-a-likely-safe-booleans --verify-hyprland --timeout-seconds 10
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

All 39 Batch A rows passed strict config-persistence validation and were enabled.

- 3 rows are accepted-unobservable from prior live semantics diagnostics.
- 36 rows are unproven by strict live-observable policy.
- 39 rows passed temp-config validation.
- 39 rows are safe to enable now.
- Writable scalar rows after enablement: 94 / 341.

## Remaining Scalar Completion Impact

The harness was generalized for primitive scalar manual-review rows after Batch A.

- Batch B numeric rows enabled: 33
- Batch D primitive input rows enabled: 64
- Batch E primitive window/layout rows enabled: 45
- Additional rows enabled: 142
- Writable scalar rows after remaining scalar completion: 236 / 341
- Active config modified: no
- Active runtime modified: no

The remaining blocked scalar rows are dropdown/enum value proof gaps, session/runtime-sensitive settings, and high-risk rows.

See:

```sh
jq '.counts' data/reports/batch-a-config-persistence-candidates.v0.55.2.json
jq '.recommendedApproach' data/reports/config-persistence-validation-design.v0.55.2.json
```

## Next Implementation Sprint

Use the same proof discipline for the remaining dropdown/enum-like rows, but first derive exact allowed values from official Hyprland docs/source or define a validated line-safe policy. Keep session/runtime-sensitive and high-risk rows blocked until dedicated approval and recovery policies exist.
