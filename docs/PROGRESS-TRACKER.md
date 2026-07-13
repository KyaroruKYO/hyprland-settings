# Progress Tracker (Evidence-Based Audit, 2026-07-12)

Independently audited at commit `6f4bdd2` from source, tests, and test-regenerated reports. Previous progress percentages were **not** used. Validation this run: **1,015 tests / 251 binaries, 0 failures**; fmt, check, jq, diff-check, and release build all green; main / v0.1.0 / dist untouched.

Verified anchor numbers: 341 scalar rows in source · 135 preview-enabled UI controls · 78 dead-man rows with 38 armed by receipt · 36 input/cursor proof receipts in source · 7 families classified with 2 promoted by receipt · ~53.7k lines across 82 source files.

## Foundation (mature)

| Category | Progress | Status |
| --- | --- | --- |
| Core app shell / UI / navigation | 95–98% | near-complete |
| Config discovery / source-aware model | 90–95% | near-complete (production source/include activation deliberately capped) |
| 341-row read/write model | 100% | complete |
| Safe normal-scalar config writes | 98% | complete — every UI save now gated on Safe Live Save Mode |
| Structured-family editors (read/review) | 95% | near-complete |
| Structured-family controlled writes (temp/copied) | 100% | complete — 13 executor tests green |
| GTK evidence / automation harness | 85–90% | near-complete |

## The live-preview era (new categories; the old "Runtime/reload integration" hid four tracks)

| Category | Progress | Status |
| --- | --- | --- |
| Scalar runtime live preview | ~95% of what this machine can prove | near-complete — 135 real controls, live-proven, defects found+fixed by proofs |
| Dead-man preview / recovery UI | 38/78 armed (all provable rows) | hardware-gated remainder (18 touch rows, 3 secondary-device rows) |
| Input/cursor proof receipt system | architecture 100%; 36 receipts | complete architecture |
| Structured-family runtime preview | 2/7 families (evidence-capped) | active — hl.animation + hl.curve proven and controllable in UI |
| Safe Live Save Mode / autoreload control | 90–95% | Save coupling done (every Save path gated); persisting the mode into the config remains optional |
| Active-config pilot / real-write proof | **PASSED** (write + byte-exact restore, no reload) | milestone reached; generalized into gated persistence (see below) |

## The gaps this audit surfaced (updated after the 2026-07-12 save/persistence/migration marathon)

| Category | Progress | Status |
| --- | --- | --- |
| **Production Save integration** | 95% | **CLOSED this marathon**: both scalar UI save paths route through `gated_scalar_save_live`; structured-family saves route through `gated_family_save`; direct `apply_setting_change` is eliminated from UI (guard-tested) |
| Structured-family gated persistence | 90% for the proven surface | **shipped this marathon**: `gated_family_save` (backup → write once → reread verify → no restore on success) live-flow-proven for both proven records; Save buttons in the family controls; breadth (record picker) remains |
| Hyprland 0.55.4 migration | audit complete | **risk dissolved this marathon**: trusted `hyprctl -j descriptions` capture shows zero option drift (341 = 341) and zero bounds drift (78 compared); compatibility pinned by a regression test |
| High-risk/display recovery | 15–25% | blocked — no blind display recovery executor exists |
| Profile/mode switching | 30–40% | blocked by production activation decision |
| Missing/default insertion + duplicate resolution | capped by design | deferred to a separate approved phase (percentage would mislead) |
| Release packaging | v0.1.0 shipped; next release not started | huge unreleased delta on the work branch |
| Project reports / handoff accuracy | regenerated core is live-accurate | 344 report files; the old future-capability tail is frozen history, not current state |

## Stale claims corrected by this audit

1. **Save ≠ safe yet**: scalar Save works but is not coupled to Safe Live Save Mode — a save with autoreload active still reloads Hyprland. Prior summaries did not surface this.
2. **Version skew is real**: docs discuss 0.55.4 "readiness," but no migration is possible without trusted upstream export data, and the skew grows.
3. Only test-regenerated reports should be read as current state; the long report tail is history.

## 2026-07-12 save/persistence/migration marathon result

All three recommended items were completed the same day: (1) every Save path is gated on Safe Live Save Mode; (2) structured-family gated persistence is live-flow-proven for the two proven records with no restore after success; (3) the 0.55.4 skew was measured at zero drift against a trusted export and is pinned by a regression test. Details: `docs/PRODUCTION-SAVE-INTEGRATION.md`, `docs/STRUCTURED-FAMILY-GATED-PERSISTENCE.md`, `docs/HYPRLAND-0.55.4-MIGRATION-AUDIT.md`.

## 2026-07-13 completion marathon result (record picker, safe mode persistence, refresh, release decision)

| Category | Progress | Status |
| --- | --- | --- |
| Family record picker | complete for proven shapes | `gated_family_record_save` persists any explicitly overridden animation record's speed (style preserved) and all four control points of any existing curve; shape proofs passed live on `fade` and `quick` (zero residue); live save flow proofs passed; disabled records save-only (found by live proof); inherited/internal blocked; no creation/deletion |
| Safe Live Save Mode persistence | complete | `misc:disable_autoreload = true` saved as default through the gated scalar Save, user-chosen only; live flow proof passed with byte-exact restore |
| Hyprland 0.55.4 export refresh | complete, reproducible | read-only refresh workflow reran live with zero drift in every category; pinned capture preserved for other live versions |
| Release decision | ready pending user approval | RC materials drafted (`docs/RELEASE-DECISION.md`); no tag, no merge, no artifacts |
| Hardware-gated proofs | deferred | 18 touch rows + 3 secondary-device rows; devices unavailable; no simulated/virtual proof path exists — any future one is proposal-only |

## 2026-07-13 RC + record-shape expansion marathon result (user-approved)

| Category | Progress | Status |
| --- | --- | --- |
| Record-shape expansion | complete for the provable non-hardware surface | animation **enabled** (proven `border` 1→0→1 and `borderangle` 0→1→0) and **bezier** (proven `windows`, existing curves only) join speed; combined `AnimationRecordFields` save gated on all three receipts; disabled records promoted to preview-supported; live finding: disabled records reset speed/bezier readback; style blocked (no trusted value evidence, disabled UI row); gestures blocked (no readback listing) |
| v0.2.0-rc.1 release candidate | prepared locally | version bump + validated metadata + changelog + release notes + guarded build-only artifact script + `dist/v0.2.0-rc.1/` artifacts + RC docs + pinning tests; **no tag** (no prerelease-tag convention; one-command remaining step), no merge, no publishing; `v0.1.0`/`dist/v0.1.0` checksum-verified untouched |
| Hardware-gated proofs | deferred | unchanged: 18 touch rows + 3 secondary-device rows; no fake or virtual proofs |

## Recommended next work

1. User decisions on v0.2.0-rc.1: optional annotated RC tag, one manual RC test pass, then final v0.2.0 approval (merge, bump, tag, publish).
2. Hardware-gated proofs when hardware is available (18 touch rows, 3 secondary-device rows).
3. Style editing only if trusted value evidence appears; gesture shapes need a readback mechanism that 0.55.4 does not provide.

Hard blockers outside code: touch hardware, secondary input devices, a display recovery design, and user decisions on the release.
