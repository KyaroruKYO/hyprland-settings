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
- writable rows: 55 / 341
- blocked write rows: 286 / 341

These scalar rows are currently writable:

- `appearance.blur.enabled`
- `appearance.blur.size`
- `appearance.blur.brightness`
- `appearance.blur.contrast`
- `appearance.shadow.enabled`
- `appearance.shadow.range`
- `appearance.shadow.render_power`
- `decoration.shadow.color`
- `decoration.shadow.color_inactive`
- `decoration.shadow.offset`
- `decoration.screen_shader`
- `appearance.gaps_in`
- `appearance.gaps_out`
- `appearance.border_size`
- `appearance.rounding`
- `appearance.active_opacity`
- `appearance.inactive_opacity`
- `animations.enabled`
- `windows.snap.enabled`
- `windows.snap.window_gap`
- `windows.snap.monitor_gap`
- `general.col.inactive_border`
- `general.col.active_border`
- `general.col.nogroup_border`
- `general.col.nogroup_border_active`
- `input.pointer_sensitivity`
- `input.accel_profile`
- `input.scroll_points`
- `input.kb_file`
- `input.tablet.region_position`
- `input.tablet.region_size`
- `input.tablet.active_area_size`
- `input.tablet.active_area_position`
- `decoration.glow.color`
- `decoration.glow.color_inactive`
- `group.groupbar.text_color`
- `group.groupbar.text_color_inactive`
- `group.groupbar.text_color_locked_active`
- `group.groupbar.text_color_locked_inactive`
- `group.groupbar.font_family`
- `group.col.border_active`
- `group.col.border_inactive`
- `group.col.border_locked_inactive`
- `group.col.border_locked_active`
- `group.groupbar.col.active`
- `group.groupbar.col.inactive`
- `group.groupbar.col.locked_active`
- `group.groupbar.col.locked_inactive`
- `misc.col.splash`
- `misc.background_color`
- `misc.font_family`
- `misc.splash_font_family`
- `misc.swallow_regex`
- `misc.swallow_exception_regex`
- `layout.single_window_aspect_ratio`

All other scalar rows are blocked with a concrete blocker in the coverage report.
The validator/parser expansion target report records all 14 validator-needed rows and all 37 parser-needed rows enabled with parser, validator, fixture write, and UI/model projection tests. Remaining blocked rows are manual-review or high-risk rows.

## Manual Review Candidate Reports

The remaining blocked rows are classified for future review without changing write behavior:

```sh
jq '.counts' data/reports/manual-review-write-candidates.v0.55.2.json
jq '.counts' data/reports/high-risk-write-candidates.v0.55.2.json
jq '.counts' data/reports/live-validation-results.v0.55.2.json
jq '.currentBatchAResult' data/reports/future-live-validation-batches.v0.55.2.json
```

These reports are planning metadata only. They do not make any row writable. Batch A has a rollback-protected live-validation record: 39 rows were probed, Level 1 and Level 2 passed for all rows, revert verification passed for all rows, and runtime acceptance was not proven for any row. Batch A remains blocked until the live acceptance check is improved or reviewed. High-risk rows should remain blocked until a dedicated safety design exists.

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

For the next proof path, see [CONFIG_PERSISTENCE_VALIDATION.md](CONFIG_PERSISTENCE_VALIDATION.md).

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
