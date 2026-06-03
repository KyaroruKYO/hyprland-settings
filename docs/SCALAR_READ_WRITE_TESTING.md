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
```

## Current Safe Writable Rows

Only these scalar rows are currently writable:

- `appearance.blur.enabled`
- `appearance.shadow.enabled`
- `animations.enabled`
- `windows.snap.enabled`

All other scalar rows are blocked with a concrete blocker in the coverage report.

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
