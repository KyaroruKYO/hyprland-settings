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
- writable rows: 28 / 341
- blocked write rows: 313 / 341

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
- `input.pointer_sensitivity`
- `decoration.glow.color`
- `decoration.glow.color_inactive`
- `group.groupbar.text_color`
- `group.groupbar.text_color_inactive`
- `group.groupbar.text_color_locked_active`
- `group.groupbar.text_color_locked_inactive`
- `misc.col.splash`
- `misc.background_color`

All other scalar rows are blocked with a concrete blocker in the coverage report.
The validator/parser expansion target report records 14 validator-needed rows enabled, 10 parser-backed color rows enabled, and 27 parser-needed rows still blocked for manual review.

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
