# Future Capability Marathon Review Log

## Sprint summary
- Starting commit: 895b67281f7551789e5b4a07c0ea849db1eab622
- Branch: future-capability-marathon
- Completed safe release scope preserved: yes
- Real config touched: no
- Runtime touched: no
- v0.1.0 tag modified: no
- dist/v0.1.0 modified: no
- v0.55.2 data/model preserved: yes
- Unsafe production behavior enabled: no
- Production missing/default insertion enabled: yes, only for reviewed single-root normal-scalar safe-batch targets

## Phase results
- Missing/default insertion: safe-env-only planner/executor proof plus production single-root normal-scalar safe-batch insertion added; unsafe insertion layouts remain blocked.
- Source/include insertion: target-selection readiness model, disabled setting-detail target review UI, fixture target-selection proof, and selected-target dry-run planner added; source/include production insertion remains blocked.
- Source/include insertion: selected-target dry-run preview UI and guarded temp-fixture executor added; executor inserts the planned line, verifies it, restores original bytes, and verifies restored content fingerprint; source/include production insertion remains blocked.
- Copied-config-tree proof: root/source configs are copied to temp, relative source/include layout is preserved, generated/script/symlink/profile hints are recorded, source/include/duplicate/structured/profile guarded executors run against copied targets, copied targets are restored, and original files are verified unchanged.
- Default-disabled production gates: source/include insertion, duplicate replacement, structured `hl.bind` writes, profile/mode switching, runtime/reload mutation, high-risk/display writes, and Hyprland 0.55.4 activation now have review models that keep production behavior disabled by default.
- Explicit approval flow: shared approval requests now cover source/include insertion, duplicate replacement, structured `hl.bind`, profile/mode switching, runtime keyword/reload, high-risk/display writes, and Hyprland 0.55.4 migration. Approval can reach approved/ready-but-default-disabled states only when copied-config-tree or live-restore proof is linked; production flags remain false by default.
- Duplicate setting resolution: read-only occurrence model, safe-env exact-line replacement proof, disabled occurrence selector UI, disabled pre-Apply approval review UI, disabled review workflow, confirmation token/fingerprint model, production approval gate scaffold, confirmation-gated safe-env replacement wrapper, and guarded temp-fixture executor added; production remains blocked.
- High-risk/display-render recovery: mock watchdog/recovery state machine, disabled review model, rollback proof workflow, no-op live-readiness protocol, and guarded no-op readiness executor added; real writes remain blocked.
- Structured-family editors/writes: read-only disabled editor scaffold, invalid-input candidate validation, disabled `hl.bind` review workflow, lossless render proof, safe-env exact-line edit proof, and guarded `hl.bind` temp-fixture executor added; production writes remain blocked.
- Profile/mode switching: safe-env temp symlink switch/restore proof, disabled review model, disabled selection review workflow, target approval review, forced restore-failure coverage, and guarded temp symlink executor added; real switching remains blocked.
- Runtime mutation/reload: dry-run action boundary, mock executor, runtime action policy scaffold, disabled action review workflow, command risk classification, controlled live-test guard prerequisites, and guarded runtime executor model added; real reload and mutating hyprctl remain blocked.
- Runtime socket diagnosis: sandboxed `hyprctl` and direct socket probes fail because the sandbox cannot connect to the Hyprland Unix socket; outside-sandbox read-only `hyprctl` evidence succeeds.
- Runtime read-only evidence: `hyprctl version`, `monitors -j`, `getoption general:gaps_in`, `general:gaps_out`, `decoration:blur:enabled`, and `misc:disable_hyprland_logo` succeeded outside the sandbox.
- Runtime live restore proof: `general:gaps_in` prior value `5` and temporary value `6` were prepared with restore commands before mutation. `keyword` was rejected for non-legacy parsers, assignment `eval` syntax failed before value change, and `hl.config` eval changed readback to `6` before the prepared restore returned readback to `5`.
- Runtime approval UI surface: the setting detail pane now displays the proven `hl.config` eval mutation/restore command pair, prior/temp values, post-mutation and post-restore readbacks, approved-but-default-disabled status, and disabled production runtime/reload state. The planned enable control is insensitive and has no runtime handler.
- Disabled approval UI cards: the Config page now displays review-only cards for source/include insertion, duplicate replacement, structured `hl.bind`, profile/mode switching, high-risk/display writes, and Hyprland 0.55.4 migration. Each card has stable widget names, proof or blocker copy, production-disabled status, and an insensitive planned enable action.
- Report-backed approval card data: the six Config-page disabled approval cards now load through a typed serialized report adapter from `data/reports/disabled-approval-ui-cards.v0.55.2.json`, with explicit `Missing from report` fallback copy for incomplete records.
- Screenshot-level approval card assertions: the GTK safe-env matrix now records each approval card heading, production-disabled line, and planned disabled action through screenshots plus AT-SPI accessibility-tree text; this is not OCR-based and does not click enable controls.
- Hyprland 0.55.4 migration: disabled assessment scaffold, versioned data bundle model, disabled migration review, side-by-side comparison review, trusted-export requirement model, and local evidence collector added; app still defaults to v0.55.2 data/model.
- Hyprland 0.55.4 package/runtime evidence: `pacman -Q hyprland` reported `hyprland 0.55.4-1`, and `hyprctl version` confirmed Hyprland 0.55.4 commit `a0136d8c04687bb36eb8a28eb9d1ff92aea99704`; this is advisory only and does not activate migration.
- Controlled live/system testing: a guarded low-risk runtime mutation was made for `general:gaps_in`, then restored immediately and verified with post-restore readback.

## Progress tracker
- Core app shell / UI / navigation: 96-98% -> 97-98%
- Config discovery / source-aware model: 90-93% -> 94-96%
- 341-row read/write model: 90-95% -> 90-95%
- Safe normal-scalar writes: 92-96% -> 95-97%
- Release packaging/tag/artifacts: 85-95% -> 85-95%
- Missing/default insertion: 95-97% -> 96-97%
- Duplicate resolution: 86-91% -> 87-91%
- High-risk/display recovery: 61-70% -> 62-70%
- Structured-family editors/writes: 63-73% -> 64-73%
- Profile/mode switching: 64-73% -> 65-73%
- Runtime/reload integration: 55-65% -> 66-76%
- Hyprland 0.55.4 migration: 49-60% -> 50-60%

## Safety
- Real user config edited: no
- AGS/Waybar touched: no
- Hyprland reloaded: no
- Mutating hyprctl used: yes, controlled `keyword`, assignment `eval`, and `hl.config` eval attempts for `general:gaps_in`; only `hl.config` changed the value and it was restored immediately.
- Scripts executed: no
- Lua executed: no
- Release/tag/package touched: no

## Validation
- cargo fmt: passed
- cargo fmt --check: passed
- cargo check: passed
- cargo test: passed
- cargo build --release: passed
- jq reports: passed
- git diff --check: passed
- GTK safe-env evidence matrix: passed for the deep approval card data surface; evidence root: `/tmp/hyprland-settings-gtk-automation/20260620_000757`.

## Next recommended sprint
Use the report-backed approval cards as the review source for future explicit production activation UX, starting with a still-default-disabled production activation decision model that can consume approval tokens without enabling production by default.
