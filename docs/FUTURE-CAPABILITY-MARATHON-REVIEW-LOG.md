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
- GTK safe-env evidence matrix: passed for the activation form/state-machine UI surface; evidence root: `/tmp/hyprland-settings-gtk-automation/20260620_131546`.

## Next recommended sprint
Use the report-backed approval cards as the review source for future explicit production activation UX, starting with a still-default-disabled production activation decision model that can consume approval tokens without enabling production by default.

## Default-Disabled Production Activation Decision Review - 2026-06-20

- Added source/include and duplicate production activation decision reviews that consume report-backed approval card data.
- Both decisions can reach ApprovedButDefaultDisabled only while production flags remain false.
- Added disabled Config-page decision cards and GTK screenshot plus AT-SPI assertions for both cards.
- No production source/include insertion, duplicate write, runtime mutation, reload, or real config mutation was enabled.

## Default-Disabled Production Activation Path Review - 2026-06-20

- Added source/include and duplicate production activation path reviews that consume ApprovedButDefaultDisabled decisions.
- Added explicit future request and safety-plan requirements: production activation request, user approval, production flag, backup, restore, reread, post-restore verification, dry-run summary, touched-file list, and final confirmation.
- Added disabled Config-page activation path cards and GTK screenshot plus AT-SPI assertions for both cards.
- Production source/include insertion and duplicate replacement remain disabled; no real config, runtime mutation, reload, or executor path was enabled.

## 2026-06-20 - Default-disabled activation controls

Implemented final review-only production activation controls for source/include insertion and duplicate replacement. Both controls consume activation path reviews, validate complete activation request inputs, validate complete safety-plan inputs, require executor wiring to remain `Unwired`, and can reach `ValidatedButExecutorUnwired` while all production flags remain false. No source/include or duplicate production executor was wired, no real config was touched, no runtime mutation was run, and v0.55.2 remains the active/default model. Evidence is recorded in `data/reports/default-disabled-production-activation-control.v0.55.2.json`.

## 2026-06-20 - Activation form state machine

Added review-only activation form/state-machine coverage for source/include insertion and duplicate replacement. The form state collects scope, reason, explicit activation token, decision category, safety acknowledgements, backup/restore/reread/post-restore plans, dry-run summary, and touched-file list data. Complete form states generate `ProductionActivationRequest` and `ProductionActivationSafetyPlan` values, validate through the final activation controls as `ValidatedButExecutorUnwired`, and keep executors `Unwired`. No production source/include insertion, duplicate write, real config mutation, runtime mutation, reload, or migration activation was enabled.

## 2026-06-20 - Disabled activation form fields

Replaced the source/include and duplicate activation form projection surfaces with real disabled GTK field widgets. Scope/category, reason, token, and decision category render as read-only insensitive entries; acknowledgement requirements render as insensitive check buttons; backup, restore, reread, post-restore verification, dry-run summary, and touched-file safety-plan data render as read-only insensitive text views. The state-machine logic remains unchanged and still validates through final controls as `ValidatedButExecutorUnwired` while production flags remain false and executors remain `Unwired`. Source-level tests now cover field helpers, stable widget-name suffixes, read-only/insensitive flags, and no mutation handlers. The GTK matrix was run at `/tmp/hyprland-settings-gtk-automation/20260620_134347`; live field-label proof was blocked because AT-SPI could not open the runtime bus socket in that run.

## 2026-06-20 - Activation draft state

Added still-disabled in-memory activation draft plumbing for source/include insertion and duplicate replacement. Drafts can be created empty or from existing form state, updated/reset in memory, converted back into `ProductionActivationFormState`, and validated through the existing form/control pipeline as `DraftValidatedForReviewOnly` / `ValidatedButExecutorUnwired`. The Config page now shows disabled draft cards with draft status, validation, dirty state, in-memory-only status, executor wiring, and production-disabled copy. Update/reset controls are insensitive and have no persistence, mutation, or executor handler. Production flags remain false, executors remain `Unwired`, and no disk persistence, real config mutation, runtime mutation, reload, or migration activation was added.

## 2026-06-20 - Activation draft edit review

Added a still-disabled activation draft-edit layer for source/include insertion and duplicate replacement. Draft-edit mode is disabled by default in the live UI, but model tests can enter an in-memory-only edit mode, update request and safety-plan draft values, recompute form/control validation, reset back to default draft state, and prove no persistence or executor wiring is introduced. The Config page now shows disabled draft-edit status cards with editing mode, dirty state, validation, in-memory-only status, executor wiring, and production-disabled copy. Planned update/reset controls remain insensitive.

## 2026-06-20 - Live activation draft edit bridge

Connected source/include and duplicate activation form field edits to a memory-only GTK draft-edit bridge. Entry, text-buffer, and check-button handlers update in-memory draft state, recompute draft/form/control validation, and can reset back to default memory state without disk persistence, executor wiring, production flag changes, real config writes, runtime mutation, or reload. The Config page now shows live draft-edit cards with memory-only editing mode, dirty state, recomputed validation, `Not saved to disk`, executor `Unwired`, and production-disabled copy. Production activation controls remain disabled and no source/include or duplicate executor was wired.

## 2026-06-20 - Remaining dependency scan

Added a remaining dependency scan across all tracker categories. Core UI, config discovery, 341-row coverage, safe normal-scalar writes, and release packaging are effectively capped for the current safe-release scope. Missing/default insertion, duplicate resolution, structured-family writes, profile/mode switching, and runtime/reload integration are blocked by explicit production activation. High-risk/display recovery is blocked by missing high-risk recovery proof. Hyprland 0.55.4 migration is blocked by missing official export data. No extra independent production-safe implementation was selected after the draft-edit layer because the remaining work is either capped or blocked by those dependencies.
