# Hyprland 0.55.4 Export Refresh

Machine-readable result: `data/reports/hyprland-0.55.4-export-refresh.v0.55.2.json`.

## What this is

The 0.55.4 migration audit trusts one artifact: the
`hyprctl -j descriptions` export captured from the official binary under
`data/exports/hyprland-0.55.4/`. This workflow makes refreshing that
capture repeatable whenever Hyprland updates:

```sh
tools/refresh_hyprland_descriptions_export.sh
```

## Behavior

- **Read-only against the compositor**: the only hyprctl commands are
  `hyprctl version` and `hyprctl -j descriptions`. No reload, no
  dispatch, no runtime mutation, no active-config access.
- **Live version matches the pinned 0.55.4**: the capture is refreshed in
  place, the version metadata is rewritten, the diff against the previous
  capture is computed (rows added / removed, numeric bounds drift,
  cosmetic description-only diffs, other data changes), and the pinned
  migration test (`cargo test --test hyprland_0554_migration_audit`)
  reruns.
- **Live version differs**: the pinned 0.55.4 capture is NOT overwritten.
  A new versioned capture is written under
  `data/exports/hyprland-<version>/` and the report states the migration
  step needed. No guessed rows are ever hand-edited, and migration is
  never claimed complete without a trusted capture.

## Last refresh (2026-07-13)

Live Hyprland 0.55.4 at commit `a0136d8c04687bb36eb8a28eb9d1ff92aea99704`
(the same commit as the original capture). Result: fully reproducible —
341 = 341 rows, 0 added, 0 removed, 0 bounds changes, 0 description
diffs, 0 other data changes; the pinned migration test passed.

## Pins

`tests/hyprland_0554_export_refresh.rs` pins: the script exists,
is executable, uses only the two read-only commands, preserves the pinned
capture for other live versions, and reruns the audit; the capture holds
341 options with matching version metadata; and the refresh report
records zero drift with `hyprctlReloadRan: false`,
`runtimeMutationRan: false`, `handEditedRows: false`.
