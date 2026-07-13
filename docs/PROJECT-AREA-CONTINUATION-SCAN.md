# Project Area Continuation Scan

## Result

Structured-family editors/writes have reached production Save for the proven surface: every Save path is gated on Safe Live Save Mode, and `gated_family_save` persists the two live-proven records (hl.animation global speed, hl.curve default Y0) with backup, one atomic write, and reread verification — no restore on success. The Hyprland 0.55.4 migration audit is complete with zero drift. The historical account below records how the boundary was crossed: controlled targets first, then the approved fifteen-gate pilot, then the gated production save.

Real structured-family write code now exists for controlled targets. The controlled write-target policy distinguishes test-owned fixture, copied config tree, temporary config, active real config, and unknown targets; only the first three are writable, and any path that resolves to the active Hyprland config — textually or through symlinks — is reclassified and rejected regardless of declared kind. The controlled write executor is implemented, internally wired to the parse/render/projection pipeline, and proven by tests that really write files inside temporary test directories: write, byte-exact backup, restore, rollback, post-write reread verification, and post-restore reread verification round-trip for all seven families against copied temp targets. Fail-closed proof covers missing approval, forbidden active-config approval, missing backup/restore/verification plans, out-of-root backup paths, unsafe staged apply plans, tampered linkage, empty rendered records, unknown targets, path escapes, symlink escapes, and disallowed roots; post-write verification failure automatically restores the original bytes. The executor emits write receipts and audit records with constant-false active-config/reload/runtime flags, is unreachable from live UI, main, and the scalar write flow, and never calls hyprctl, reloads Hyprland, runs commands, or mutates runtime. Active real config writes approved remains false, GUI live Apply controls enabled remains false, hyprctl reload enabled remains false, runtime mutation enabled remains false, and first real config write approved remains false. Source/include and duplicate runway work remains capped and must not continue on this branch.

## Classifications

- Core app shell / UI / navigation: capped.
- Config discovery / source-aware model: needs audit; non-mutating source graph tests can continue, production source/include activation cannot.
- 341-row read/write model: capped.
- Safe normal-scalar writes: capped.
- Release packaging/tag/artifacts: release_decision_ready_pending_user_approval — a full release decision was recorded (no tag, no merge, no artifacts); RC materials are drafted in docs/RELEASE-DECISION.md.
- Missing/default insertion: capped by source/include production activation closeout.
- Duplicate resolution: capped by duplicate production activation closeout.
- High-risk/display recovery: blocked by high-risk recovery proof.
- Structured-family editors/writes: production_save_complete_for_proven_surface — the pilot passed its fifteen-gate path, and gated persistence now saves the two proven records behind receipt + Safe Live Save Mode + identity gates; the other five families stay blocked; the controlled write executor still rejects the active config by construction; remaining growth is breadth (record picker), not architecture.
- Profile/mode switching: blocked by production activation and live proof.
- Runtime/reload integration: proof marathon complete; 135 default-previewable rows with real controls, 38 armed dead-man candidates (2 animation + 36 proof-passed input/cursor rows), 27 disarmed pending hardware or secondary-device proofs, two executor defects found and fixed by live proofs, monitor/display rows fully blocked, reload still disabled.
- Hyprland 0.55.4 migration: audit_complete_zero_drift — trusted `hyprctl -j descriptions` capture from the official 0.55.4 binary; 341 = 341 options, 0 added, 0 removed, 78 numeric bounds compared with 0 mismatches; pinned by a regression test; the refresh is now a repeatable workflow (`tools/refresh_hyprland_descriptions_export.sh`) that preserves the pinned capture for other live versions — executed live 2026-07-13 with zero drift in every category.

## Finish-App Sprint Update

The gated active-config pilot module now exists with a fifteen-gate preflight, and the copied active-config rehearsal is proven against the real config content with the source untouched. The live pilot is blocked by the `AutoreloadDisabledConfirmed` and `NoRuntimeMutationPlanned` gates: `misc:disable_autoreload` is `false` on this system, so an active-config write would live-reload the compositor, and runtime mutation is not approved. The controlled executor was hardened (symlink-through-target-file escape fixed; atomic temp-file-plus-rename writes), and a review-only GUI status card with a permanently insensitive Apply control is proven in the GTK evidence matrix.

## Family Completion Marathon Update

All seven structured families are classified for runtime preview from live evidence; hl.animation and hl.curve were promoted to supervised modify-existing preview by passed zero-residue proofs; four families stay blocked high-risk and gestures stay blocked without a verification mechanism. The active-config pilot gate now requires live-collected autoreload evidence in addition to the operator flag.

## Save Persistence Migration Marathon Update

The active-config write pilot passed (runtime-first strategy, no reload, byte-exact restore), and this marathon turned the proven mechanism into product behavior: all scalar UI saves route through `gated_scalar_save_live`, structured-family saves route through `gated_family_save` (live-flow-proven for both proven records, with no restore after success), and the 0.55.4 version skew was measured at zero drift against a trusted export.

## Completion Marathon Update (record picker, safe mode persistence, refresh, release decision)

The family record picker generalized gated persistence one proven record shape at a time: record-shape proofs passed live on non-family-proof records (hl.animation `fade` speed; hl.curve `quick` control points; both zero residue), and `gated_family_record_save` now persists the speed of any explicitly overridden animation record (style preserved) and all four control points of any existing curve through the same gate chain, with live save flow proofs and byte-exact flow-proof restores. A live proof also found that disabled-at-runtime records cannot be preview-verified (the compositor ignores speed changes on them) — they are save-only; inherited and internal records stay blocked because creation is blocked. Safe Live Save Mode can now be persisted (`misc:disable_autoreload = true`) through the gated scalar Save — user-chosen via a Save as default control, never automatic, live-flow-proven. The 0.55.4 capture refresh is a repeatable read-only workflow that reran with zero drift. A release decision was recorded: ready pending user approval, with RC materials drafted and no release action taken.

## RC + Record-Shape Expansion Marathon Update (2026-07-13)

The user approved release-candidate preparation and further record-shape expansion. Record shapes: the animation **enabled** flag was proven live in both directions (`border` 1→0→1, `borderangle` 0→1→0) and the animation **bezier** reference was proven on `windows` (existing curves only) — both zero residue; the picked-record save became the combined `AnimationRecordFields` request gated on all three animation shape receipts, disabled records were promoted to preview-supported, and a live finding was recorded (disabled records reset their speed/bezier readback, so disabled previews verify the enabled flag only). Style stays not-editable (no trusted value evidence; disabled UI row with the reason) and gestures stay blocked (no readback listing on 0.55.4). Release: v0.2.0-rc.1 was prepared locally — version bump, validated metadata, changelog, release notes, a guarded build-only artifact script, `dist/v0.2.0-rc.1/` artifacts with SHA256SUMS, RC checklist/test-plan/limitations/upgrade/rollback docs, and pinning tests; no RC tag was created (the repo has no prerelease-tag convention — a documented one-command remaining step), nothing was merged or published, and `v0.1.0`/`dist/v0.1.0` were checksum-verified untouched. The release-area classification is now `release_candidate_v020rc1_prepared_locally`.

## Recommended Next Work

The RC and record-shape expansion areas are done. Remaining: user decisions on v0.2.0-rc.1 (optional RC tag, manual test pass, final v0.2.0 approval: merge, bump, tag, publish); hardware-gated proofs (18 touch rows, 3 secondary-device rows) when devices exist; style editing only if trusted value evidence appears.
