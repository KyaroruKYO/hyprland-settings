# Live Scenario Harness

This harness supports live-desktop scenario review without making live mutation the default.

## Modes

- `safe-env`: preferred. Tests Hyprland Settings with temporary `HOME` and `XDG_CONFIG_HOME` scenario directories.
- `live-swap`: fallback only. Temporarily swaps the real Hyprland config after a full backup and restore plan exists.

The sprint test suite uses `safe-env` only. It does not click Apply in the real app, reload Hyprland, run mutating `hyprctl`, execute user scripts, disable AGS, disable Waybar, or edit the user's real config.

## Scripts

- `backup_desktop_state.sh <backup-root>` creates a timestamped backup under the supplied root.
- `verify_restore.sh <backup-dir>` compares the current config trees to a backup.
- `restore_desktop_state.sh <backup-dir>` restores backed-up config trees. Use from a TTY if a live-swap test fails.
- `run_safe_scenario.sh` runs the safe-env scenario integration test.

## Manual TTY Restore

If a live-swap test is ever run and the desktop becomes unusable:

```sh
cd /home/kyo/Projects/hyprland-settings
tools/live_scenario_harness/restore_desktop_state.sh /path/to/backup-dir
tools/live_scenario_harness/verify_restore.sh /path/to/backup-dir
```

Then restart shell components manually if needed:

```sh
systemctl --user restart hyprpaper-desktop.service 2>/dev/null || true
systemctl --user restart waybar-desktop.service 2>/dev/null || true
ags run ~/.config/ags/app.ts -g 4
```

The harness intentionally does not run profile or mode switch scripts.
