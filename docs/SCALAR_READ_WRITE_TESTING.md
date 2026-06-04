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
```

These reports are planning metadata only. They do not make any row writable. Batch A is the recommended first review target; high-risk rows should remain blocked until a dedicated safety design exists.

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
