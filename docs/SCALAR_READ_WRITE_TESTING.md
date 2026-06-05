# Scalar Read/Write Testing

This app can be validated without a display. The scalar read/write path is covered by Rust tests, fixture configs, and the generated coverage report.

## No-Display Validation

```sh
cd /home/kyo/Projects/hyprland-settings
cargo fmt --check
cargo check
cargo test
cargo build --release
```

Optional desktop/AppStream metadata checks:

```sh
desktop-file-validate data/applications/io.github.kyarorukyo.hyprlandsettings.desktop
appstreamcli validate --pedantic data/metainfo/io.github.kyarorukyo.hyprlandsettings.metainfo.xml || true
```

The AppStream release metadata warning is expected until real release tags exist.

## Inspect Coverage

```sh
jq '.counts' data/reports/scalar-read-write-coverage.v0.55.2.json
jq '[.rows[] | .writeStatus] | group_by(.) | map({status: .[0], count: length})' data/reports/scalar-read-write-coverage.v0.55.2.json
jq -r '.rows[] | select(.writeStatus != "writable") | [.rowId, .writeStatus, .writeBlocker] | @tsv' data/reports/scalar-read-write-coverage.v0.55.2.json
jq '.counts' data/reports/scalar-write-expansion-targets.v0.55.2.json
```

## Current Safe Writable Rows

Current scalar coverage:

- readable rows: 341 / 341
- writable rows: 236 / 341
- blocked write rows: 105 / 341

The complete writable list is generated from `SAFE_WRITABLE_ROWS` and mirrored in the coverage report:

```sh
jq -r '.rows[] | select(.writeStatus == "writable") | .rowId' data/reports/scalar-read-write-coverage.v0.55.2.json
```

All remaining non-writable scalar rows have a concrete blocker in the coverage report:

- 17 dropdown/enum-like rows need exact allowed-value proof or a validated line-safe policy.
- 16 session/runtime-sensitive rows need a dedicated approval and recovery policy.
- 72 high-risk rows remain blocked by policy.

The remaining scalar completion report records the latest proof pass:

```sh
jq '.counts' data/reports/remaining-scalar-completion.v0.55.2.json
```

## Manual Review Candidate Reports

The remaining blocked rows are classified for future review without changing write behavior:

```sh
jq '.counts' data/reports/manual-review-write-candidates.v0.55.2.json
jq '.counts' data/reports/high-risk-write-candidates.v0.55.2.json
jq '.counts' data/reports/live-validation-results.v0.55.2.json
jq '.currentBatchAResult' data/reports/future-live-validation-batches.v0.55.2.json
```

The manual/high-risk reports are planning metadata. Batch A has a rollback-protected live-validation record: 39 rows were probed, Level 1 and Level 2 passed for all rows, revert verification passed for all rows, and runtime acceptance was not proven for any row. Batch A was later enabled through config-persistence proof instead: parser/writer roundtrip and `Hyprland --verify-config` passed for all 39 temporary configs with no active config/runtime mutation. High-risk rows should remain blocked until a dedicated safety design exists.

## Companion Schema Metadata Reports

HyprMod and `hyprland-schema` companion evidence is tracked separately from write enablement:

```sh
jq '.counts' data/reports/hyprmod-companion-full-scalar-comparison.v0.55.2.json
jq '.counts' data/reports/companion-schema-metadata-integration.v0.55.2.json
jq '.counts' data/reports/companion-schema-conflict-review.v0.55.2.json
```

These reports are advisory proof inputs. They can improve defaults, bounds, enum values, editor hints, and future test values, but they do not change the write allowlist by themselves. Conflict rows are held for manual review before any production behavior changes.

## Live Validation Harness

The live-validation harness is for controlled runtime probing only. It does not persistently edit `hyprland.conf`.

The Level 3 debug report captures raw `hyprctl` output for a three-row Batch A subset:

```sh
jq '.counts' data/reports/live-validation-level3-diagnostics.v0.55.2.json
jq -r '.items[] | [.rowId, .runtimeSetting, .keywordExitSuccess, .postApplyParsedValue, .valuesEquivalent, .revertVerified, .diagnosis] | @tsv' data/reports/live-validation-level3-diagnostics.v0.55.2.json
```

Current diagnosis: `hyprctl keyword` exits successfully for the sampled rows and reverts verify, but `hyprctl getoption` remains at the original parsed value after the candidate apply. Batch A is not enabled from that signal.

The semantics policy and Batch A classification are available here:

```sh
jq '.counts' data/reports/live-validation-semantics.v0.55.2.json
jq '.counts' data/reports/live-validation-batch-a-semantics-classification.v0.55.2.json
```

The selected policy is strict for automatic enablement: `keyword` success, clean config errors, candidate-visible `getoption`, and verified revert. Rows that only show `keyword` success with unchanged `getoption` remain blocked as accepted-unobservable.

For the next proof path, see [CONFIG_PERSISTENCE_VALIDATION.md](CONFIG_PERSISTENCE_VALIDATION.md) and [CONFIG_PERSISTENCE_VALIDATION_DESIGN.md](CONFIG_PERSISTENCE_VALIDATION_DESIGN.md).

The config-persistence design rejects `hyprctl eval` for this app because official source shows it executes Lua. The recommended next proof path is temporary config validation through Hyprland's `--verify-config` support, with no mutation of active `hyprland.conf` and no active runtime mutation.

Inspect the design reports:

```sh
jq '.recommendedApproach' data/reports/config-persistence-validation-design.v0.55.2.json
jq '.counts' data/reports/batch-a-config-persistence-candidates.v0.55.2.json
jq '.counts' data/reports/config-persistence-validation-results.v0.55.2.json
```

Batch A passed that config-persistence path and is now included in the safe writable scalar allowlist. This does not mean `hyprctl keyword` live-observable proof passed; it means the temporary config was parser/writer verified and accepted by `Hyprland --verify-config` without active config or runtime mutation.

Dry-run the Batch A plan:

```sh
cargo run --bin hyprland-settings -- live-validate --dry-run --plan data/reports/live-validation-plan.v0.55.2.json --results /tmp/hyprland-settings-live-validation-dry-run.json
```

Run live validation only from an active Hyprland session, one setting at a time through the harness, with rollback armed:

```sh
cargo run --bin hyprland-settings -- live-validate --live --batch batch-a-likely-safe-booleans --timeout-seconds 5 --plan data/reports/live-validation-plan.v0.55.2.json --results data/reports/live-validation-results.v0.55.2.json
```

Do not run ad hoc `hyprctl keyword` commands for write approval. The harness prepares rollback first, applies the candidate, reverts immediately, and verifies the original value is restored. It does not run `hyprctl reload`.

Parser-backed writable families currently include:

- strict color literals: `rgb(RRGGBB)`, `rgba(RRGGBBAA)`, `0xAARRGGBB`
- gradient/color-list values: one or more supported colors plus an optional final finite `deg` angle
- vector/tuple values: `x y` or `x,y`
- line-safe strings
- line-safe regex strings, stored as text only and never compiled or executed
- sanitized path strings, stored as text only and never opened or executed
- scroll point numeric lists in `<step> <points...>` form with a positive finite step

## Run The App Later

When a display is available:

```sh
cd /home/kyo/Projects/hyprland-settings
cargo run --bin hyprland-settings
```

The app reads `hyprland.conf` as plain text and does not execute Lua or Hyprland commands.

## Safely Test One Writable Setting

Before testing writes manually, make a separate backup of your Hyprland config:

```sh
mkdir -p /home/kyo/hyprland-config-backups
cp -a /home/kyo/.config/hypr /home/kyo/hyprland-config-backups/manual-before-hyprland-settings-test-$(date +%Y%m%d_%H%M%S)
```

Then open the app, select one of the safe writable rows, review the proposed value, and apply from the row detail panel.

The app creates a backup before writing, writes only the detected `hyprland.conf`, rereads the value for verification, and shows rollback source/backup paths. It does not run `hyprctl reload`.

## Restore From App Backup

The app reports the backup path after a write. To restore manually:

```sh
cp <backup-path> /home/kyo/.config/hypr/hyprland.conf
```

Review the path before copying. Do not use this command if your detected config target is somewhere else.
