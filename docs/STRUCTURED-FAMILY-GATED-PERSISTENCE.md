# Structured-Family Gated Persistence

> Historical rollout detail. On the current unreleased branch, the proven
> `hl.animation` and `hl.curve` persistence paths use the shared drift-safe,
> metadata-aware atomic write and XDG-state backup primitives documented in
> `SAVE-WRITE-STABILIZATION.md`. The family scope described here remains
> unchanged.

The first real user-facing Save for structured families: previewed values
for the two live-proven records can now persist to the active config using
the proven pilot write shape — backup → one atomic write → reread
verification — **without** the pilot's restore step (this is the save, not
a drill). Production code never restores after a successful save;
`productionSaveRestoresAfterSuccess: false`.

## Scope: exactly the proven surface

| Target | Record written |
|---|---|
| `hl.animation` global speed | `animation = global, ONOFF, SPEED, BEZIER` |
| `hl.curve` default Y0 | `bezier = default, X0, Y0, X1, Y1` |

The target enum (`FamilySaveTarget`) can only express these two records.
The other five families (`hl.monitor`, `hl.bind`, `hl.gesture`,
`hl.device`, `hl.permission`) stay blocked; no record creation or deletion
exists anywhere in the module (source-guarded). Only the target record's
own line is replaced (or appended if absent) — every other line in the
config is preserved byte-for-byte.

The non-target fields of the rendered line come from live runtime readback
(`hyprctl animations`), so a save never clobbers a field the user changed
elsewhere at runtime.

## The gate sequence (`gated_family_save`)

1. **Value validation** — finite, in range (speed `0.1..=20`, y0 `-1..=2`)
   — before any gate or file access.
2. **Proven-family receipt** — the family must hold a passed live-proof
   receipt in `PROVEN_FAMILY_RECORD_PROOFS`.
3. **Safe Live Save Mode** — live-verified via
   `require_safe_live_save_mode`; fails closed (see
   [PRODUCTION-SAVE-INTEGRATION.md](PRODUCTION-SAVE-INTEGRATION.md)).
4. **Target identity** — the discovered config must be Found and pass
   `structured_family_path_is_active_real_config`.
5. **Backup** — byte-exact copy outside the config directory, verified
   readable, path recorded in the receipt.
6. **One atomic write** — `apply_rendered_family_records` +
   `atomic_controlled_write`.
7. **Reread verification** — exactly one record matching the target must
   exist, and its trimmed line must equal the rendered line exactly.
   On verification failure the backup is restored automatically; on
   success nothing is restored.

The receipt (`FamilySaveReceipt`) records the rendered line, backup path,
pre/post-save hashes, `restored_after_success: false`, `reload_run: false`.

## UI

Both family preview controls carry a **Save previewed value** button
routed through `gated_family_save_live` (the wrapper owns the runner; UI
never constructs commands). When Safe Live Save Mode is inactive the save
fails with the enable-first message, shown as the row status.

## Evidence

Live flow proof (env-gated, `HYPRLAND_SETTINGS_RUN_STRUCTURED_FAMILY_SAVE_LIVE=1`
plus `HYPRLAND_SETTINGS_STRUCTURED_FAMILY_SAVE_TARGET`), run 2026-07-12:

- Gate proof: the save was blocked while autoreload was active.
- Real saves: `animation = global, 1, 8, default` and
  `bezier = default, 0, 0.75, 0.15, 1` written to the real
  `hyprland.conf`, backups created, reread-verified, production code did
  not restore, config hash changed as expected.
- Flow-proof cleanup: the **test** (not production code) restored the
  pre-test bytes exactly (SHA verified) and the original autoreload state,
  so the machine ends unchanged. The pilot itself still restores
  (`pilotStillRestores: true`).
- No reload ran at any point; normal `cargo test` never touches the active
  config (the live test is ignored + env-gated).

Report: `data/reports/structured-family-gated-persistence.v0.55.2.json`.
