# Hyprland 0.55.4 Migration Audit

**Result: zero option drift.** The app's v0.55.2 model is name- and
bounds-compatible with the live Hyprland 0.55.4 compositor. No migration
is required at this time, and the previously flagged "largest structural
risk" (version skew) is dissolved by trusted evidence rather than
assumption.

## The trusted source

The migration was blocked on trusted upstream data — the app must not
fake 0.55.4 support from guesses. The solution: the official installed
0.55.4 binary itself exports its full option model via

```
hyprctl -j descriptions
```

which returns every option's name, description, type, default, and
numeric min/max — straight from the running compositor
(`Hyprland 0.55.4, commit a0136d8c, clean build`). This is captured to
`data/exports/hyprland-0.55.4/hyprctl-descriptions.v0.55.4.json`
(341 options) alongside `hyprland-version.txt`.

## Measured drift

| Measure | Result |
|---|---|
| Options in the v0.55.2 model | 341 |
| Options exported by live 0.55.4 | 341 |
| Added in 0.55.4 | **0** |
| Removed in 0.55.4 | **0** |
| Modeled numeric bounds compared | 78 |
| Bounds mismatches | **0** |
| Description texts differing | 47 (cosmetic wording only) |

## Regression guard

`tests/hyprland_0554_migration_audit.rs` pins the compatibility against
the captured export on every `cargo test` run: exact option-set equality
and bounds agreement (with a floor of ≥70 comparable bounds so the check
cannot silently degrade to comparing nothing).

## Refresh procedure (on future Hyprland updates)

```sh
hyprctl -j descriptions > data/exports/hyprland-0.55.4/hyprctl-descriptions.v0.55.4.json
hyprctl version > data/exports/hyprland-0.55.4/hyprland-version.txt
cargo test --test hyprland_0554_migration_audit
```

If a future version adds/removes options or changes bounds, the pinned
test fails with the precise lists — that failure is the actionable
migration worklist.

Report: `data/reports/hyprland-0.55.4-migration-audit.v0.55.2.json`.
