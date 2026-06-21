# Future Capability Marathon Handoff

## Current state
- Branch: future-capability-marathon
- Starting commit: 895b67281f7551789e5b4a07c0ea849db1eab622
- Latest commit: see branch HEAD after final handoff commit
- Release artifacts preserved: `dist/v0.1.0`
- v0.1.0 tag modified: no
- Real config touched: no
- Runtime touched: yes; low-risk `general:gaps_in` was restored to its prior value

## Completed phases
- All seven future capability tracks were reviewed and recorded.
- Missing/default insertion received safe-env-only planner/executor proof, disabled review scaffolding for unsupported layouts, and a production-enabled single-root normal-scalar safe-batch insertion path.
- Missing/default insertion now has a source/include target-selection readiness model; source/include production insertion remains blocked.
- Missing/default insertion now has a disabled source/include target-selection review surface in the setting detail pane; it shows root config, candidate targets, selected target state, readiness state, and planned disabled actions.
- Missing/default insertion now has source/include fixture target-selection proof; fixture plans require an explicit selected target and block no-selection, generated/script-managed, symlink/profile, unknown, duplicate, and ambiguous targets.
- Missing/default insertion now has a source/include selected-target dry-run planner that previews the exact selected target and inserted line for root/source temp fixtures while keeping production source/include insertion disabled.
- Missing/default insertion now wires selected-target dry-run preview into the disabled detail UI and shows root path, selected target, source depth, planned inserted line, proposed value, and blocked/allowed dry-run state.
- Missing/default insertion now has a guarded temp-fixture executor that consumes source/include proof plus dry-run plan, inserts the exact planned line, verifies it, restores original bytes, and verifies restored content fingerprint.
- Copied-config-tree proof now copies root/source config files to temp, preserves relative source/include layout, records source depth and management hints, runs the source/include guarded executor against the copied target, restores the copied file, and proves the original file unchanged.
- Source/include insertion now has a default-disabled production gate review that requires explicit selected target, source-graph membership, copied-config-tree proof, matching dry-run plan, and default-disabled production flag.
- Source/include insertion now connects to the explicit approval-flow model; copied proof plus exact target/line evidence can reach approved-but-default-disabled, never production enabled by default.
- Duplicate resolution received a read-only occurrence model and safe-env exact-line replacement proof.
- Duplicate detail UI now has a disabled occurrence selector showing file, line, raw line, value, and source depth.
- Duplicate resolution now has a disabled review workflow for no selection, invalid selection, and selected-but-production-disabled states.
- Duplicate resolution now has a confirmation token/fingerprint model for missing, pending, confirmed, rejected, and expired states; production duplicate writes remain disabled.
- Duplicate resolution now has a production approval gate scaffold with occurrence preconditions and a confirmation-gated safe-env replacement wrapper; production duplicate writes remain disabled.
- Duplicate detail UI now has a disabled pre-Apply approval review showing approval state, precondition fingerprint, block reason, and a disabled planned confirmation action.
- Duplicate resolution now has a guarded temp-fixture executor that requires confirmed occurrence, matching fingerprint, matching raw line, matching old value, backup, replacement verification, and byte-exact restore.
- Duplicate resolution now has copied-config-tree proof for confirmed replacement against a copied duplicate target with byte-exact restoration and original-file unchanged verification.
- Duplicate resolution now has a copied-proof-aware default-disabled production gate review that requires confirmed occurrence, matching fingerprint, path, line number, raw line, old value, source depth, and copied-config-tree proof.
- Duplicate resolution now connects to the explicit approval-flow model; copied proof plus confirmed occurrence can reach approved-but-default-disabled, never production enabled by default.
- High-risk/display recovery received a mock watchdog state machine.
- High-risk/display recovery received a disabled review model backed by mock watchdog state.
- High-risk/display recovery now has a rollback proof workflow model for backup, reread, timeout restore, and restore reread requirements.
- High-risk/display recovery now has a no-op live-readiness protocol that refuses real config paths and runtime mutation.
- High-risk/display recovery now has a guarded no-op live-readiness executor that requires recovery channel evidence, dead-man timeout evidence, and restore command evidence before reporting readiness.
- High-risk/display recovery now records that read-only runtime socket checks were attempted but unavailable, so no real high-risk/display mutation was attempted.
- High-risk/display recovery now has a default-disabled production gate review requiring out-of-band recovery, dead-man timeout, restore command, config backup, runtime snapshot, explicit approval, and readiness proof.
- High-risk/display recovery now connects to the explicit approval-flow model but remains blocked until out-of-band recovery and live restore proof exist.
- Structured families received read-only disabled editor scaffolding for raw structured entries.
- Structured family scaffolding now rejects invalid prefixes and multiline candidate input.
- Structured families now have a disabled review workflow for `hl.bind` edit candidates with raw-line and comments/order preservation requirements.
- Structured families now have `hl.bind` lossless render and safe-env exact-line edit proof for temp fixtures; production structured writes remain disabled.
- Structured families now have a guarded `hl.bind` temp-fixture executor that validates candidates, edits one selected line, verifies the new raw line, preserves comments/order, and restores original bytes.
- Structured families now have copied-config-tree proof for `hl.bind` exact-line edit against a copied target with byte-exact restoration and original-file unchanged verification.
- Structured families now have a default-disabled `hl.bind` production gate review requiring exact old/new line, candidate validation, comments/order preservation, and copied-config-tree proof.
- Structured families now connect `hl.bind` production review to the explicit approval-flow model; copied proof can reach approved-but-default-disabled.
- Profile/mode switching received safe-env-only temp symlink switch/restore proof.
- Profile/mode switching received disabled review scaffolding and forced restore-failure coverage.
- Profile/mode switching now has a disabled selection review model for current profile, resolved target, selected target, and symlink path.
- Profile/mode switching now has a target approval review model that refuses real-session paths and keeps safe-env targets review-only.
- Profile/mode switching now has a guarded temp-fixture executor that switches a temp symlink, verifies the new target, restores the original target, and verifies restoration.
- Profile/mode switching now has copied-config-tree proof that preserves a copied profile symlink, switches it to a copied target, restores the original copied target, and proves the real symlink untouched.
- Profile/mode switching now has a default-disabled production gate review requiring selected target, current symlink, original symlink snapshot, copied symlink restore proof, and future real-session live proof.
- Profile/mode switching now connects to the explicit approval-flow model; copied symlink proof can reach approved-but-default-disabled, but real switching remains disabled.
- Runtime/reload received a dry-run action boundary and mock executor.
- Runtime/reload received a runtime action policy scaffold.
- Runtime/reload now has a disabled action review workflow combining policy, dry-run result, and execution log.
- Runtime/reload now classifies status as read-only and reload/keyword/dispatch as mutating risk classes without executing commands.
- Runtime/reload now has a guarded executor model that allows read-only status proof, records mutation dry-run intent, requires prior snapshots and restore commands, and keeps `real_command_executed` false.
- Runtime/reload now has a default-disabled production gate review requiring read-only evidence, prior value snapshot, restore command, command-specific recovery plan for dispatch, and explicit approval.
- Runtime/reload socket diagnosis found the prior timeout was caused by sandbox Unix-socket permission/process isolation; outside-sandbox read-only `hyprctl version`, `monitors -j`, and `getoption` queries now succeed.
- Runtime/reload now has a runtime live-restore proof model. The controlled `general:gaps_in` proof prepared prior value `5`, temporary value `6`, and restore commands before mutation; `keyword` and assignment `eval` syntax failed before value change, then `hyprctl eval 'hl.config({ general = { gaps_in = 6 } })'` changed the runtime value and the prepared `hl.config` restore command returned it to `5`.
- Runtime/reload now has a runtime approval live-restore gate. The proven `hl.config` eval command pair, exact readbacks, restore verification, and explicit approval token evidence can reach `approved_but_default_disabled`; production runtime/reload remains disabled.
- Runtime/reload now has a disabled setting-detail approval UI surface. It displays the proven `hl.config` eval mutation/restore command pair, prior/temp values, post-mutation and post-restore readbacks, approved-but-default-disabled status, and an insensitive planned enable control; it has no runtime handler and does not enable production runtime/reload.
- The Config page now has disabled approval review cards for source/include insertion, duplicate replacement, structured `hl.bind` writes, profile/mode switching, high-risk/display writes, and Hyprland 0.55.4 migration. Each card shows proof or blocker state, production-disabled status, and an insensitive planned enable action with no mutation handler.
- Source/include and duplicate activation forms now render real disabled GTK form fields. Scope/reason/token/category use disabled read-only entries, acknowledgements use disabled check buttons, and safety plans use disabled read-only text views; they still validate through the final controls as `ValidatedButExecutorUnwired` with executors `Unwired`.
- The disabled approval review cards are now report-backed. A typed adapter loads serialized card records from `data/reports/disabled-approval-ui-cards.v0.55.2.json`, preserves the existing stable widget names, and renders explicit `Missing from report` copy if serialized fields are absent.
- GTK safe-env screenshot-level assertions now cover all six disabled approval cards by checking each heading, production-disabled line, and planned disabled action through screenshot capture plus AT-SPI accessibility-tree text.
- Hyprland 0.55.4 migration received disabled assessment scaffolding.
- Hyprland 0.55.4 migration now has a side-by-side comparison review that keeps v0.55.2 active and records missing proof.
- Hyprland 0.55.4 migration now has a trusted-export requirement model; 0.55.2 remains the active default bundle.
- Hyprland 0.55.4 migration now has a local evidence collector that records package/runtime version evidence as advisory and blocks activation until official exports, row-count diff, write-safety review, GTK evidence, local version evidence, and explicit user approval exist.
- Hyprland 0.55.4 migration now records local package metadata evidence: `hyprland 0.55.4-1`; this does not activate migration.
- Hyprland 0.55.4 migration now records runtime version evidence from `hyprctl version`: commit `a0136d8c04687bb36eb8a28eb9d1ff92aea99704`; this remains advisory only.
- Hyprland 0.55.4 migration now has a default-disabled activation gate requiring official exports, row-count diff, write-safety review, safe-env evidence, and explicit user approval before the v0.55.2 default can change.
- Hyprland 0.55.4 migration now connects to the explicit approval-flow model. Approval alone cannot activate 0.55.4 without official exports, row diff, write-safety review, safe-env evidence, and explicit default-disabled activation review.
- Controlled live-test guard model now records backup, SHA256, symlink, runtime snapshot, restore, post-restore verification, out-of-band recovery, trusted-data, and explicit live flag prerequisites before any live/system mutation can be considered.
- No production write expansion was enabled.
- Deterministic tests were added for each concrete safe-env/mock model and disabled production status.

## Partial phases
- Missing/default insertion is enabled only for reviewed single-file normal scalar safe-batch targets; source/include target selection and managed/duplicate/high-risk/structured/profile/runtime insertion remain blocked.
- Source/include target-selection UI is visible but disabled and does not write connected files.
- Duplicate resolution has no production write path despite safe-env exact-line proof, confirmation-token scaffolding, and a production approval gate model.
- Runtime/reload has read-only live evidence, a proven low-risk guarded mutation/restore attempt, and an approval review consuming that proof; production runtime/reload remains disabled.

## Blocked phases
- High-risk/display-render production writes require live recovery proof and explicit approval.
- Profile/mode switching requires explicit approval before real symlink/profile changes.
- Hyprland 0.55.4 migration requires trusted export/source proof before changing app data.

## Next exact work item
Keep activation draft persistence forbidden by default until explicit opt-in, private storage, redaction, retention, delete, encryption, and no-executor-wiring proof exists; otherwise continue only with production activation safety gates.

## Progress tracker
- Core app shell / UI / navigation: 98-99% -> 99-99%
- Config discovery / source-aware model: 90-93% -> 94-96%
- 341-row read/write model: 90-95% -> 90-95%
- Safe normal-scalar writes: 92-96% -> 95-97%
- Release packaging/tag/artifacts: 85-95% -> 85-95%
- Missing/default insertion: 99-99% -> 99-100%
- Duplicate resolution: 93-95% -> 94-95%
- High-risk/display recovery: 61-70% -> 62-70%
- Structured-family editors/writes: 63-73% -> 64-73%
- Profile/mode switching: 64-73% -> 65-73%
- Runtime/reload integration: 55-65% -> 66-76%
- Hyprland 0.55.4 migration: 49-60% -> 50-60%

## Validation status
Passed: `cargo fmt`, `cargo fmt --check`, `cargo check`, `cargo test`, `cargo build --release`, `jq empty data/reports/*.json`, and `git diff --check`.

GTK matrix was run for the live activation draft-edit bridge; evidence root: `/tmp/hyprland-settings-gtk-automation/20260620_154855`. The run did not mutate runtime/config and screenshot plus AT-SPI accessibility-tree assertions passed for the legacy draft-edit and live memory-only draft-edit cards.

## Recommended next Codex prompt
Design explicit final-approval, production-flag, executor-wiring, and live production dry-run decisions before any source/include or duplicate production executor wiring can be reconsidered.

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

## 2026-06-20 - Activation control handoff

The branch now includes final review-only activation controls for source/include and duplicate. The controls validate complete request and safety-plan inputs and require executor wiring to remain `Unwired`; both remain `ValidatedButExecutorUnwired`, with production source/include insertion and duplicate writes disabled. Next exact work: design the explicit activation form/state machine that can collect real user request data while keeping executors unwired by default.

## 2026-06-20 - Activation form handoff

The branch now includes review-only activation form/state-machine projections for source/include and duplicate. Complete form states collect request fields and safety-plan acknowledgements, generate `ProductionActivationRequest` and `ProductionActivationSafetyPlan` values, validate through the final controls as `ValidatedButExecutorUnwired`, and keep production executors `Unwired`.

## 2026-06-20 - Disabled activation form fields handoff

The branch now renders source/include and duplicate activation form data through real disabled GTK fields. Entries, check buttons, and text views are read-only/insensitive, screenshot/accessibility assertions cover the field labels, and production executors remain `Unwired`.

## 2026-06-20 - Activation draft handoff

The branch now includes in-memory activation draft plumbing for source/include and duplicate. Draft state can be created empty or from current form state, updated/reset in memory, converted into `ProductionActivationFormState`, and validated through the existing form/control reviews as review-only. The Config page displays disabled draft cards with draft status, validation, dirty state, in-memory-only copy, executor wiring `Unwired`, and production-disabled status. Planned update/reset controls are insensitive and have no persistence, mutation, or executor handler. Next exact work: if interactive collection is desired, make the activation draft fields editable only behind a still-disabled UI state that updates memory without persistence or executor wiring.

## 2026-06-20 - Activation draft edit handoff

The branch now includes a still-disabled activation draft-edit layer for source/include and duplicate. Draft editing is disabled by default in the UI, but model tests can enter an in-memory-only edit mode, update draft request/safety-plan fields, recompute validation through the existing form/control reviews, reset to defaults, and prove persistence remains absent. The Config page displays disabled draft-edit cards with editing mode, dirty state, validation, in-memory-only copy, executor wiring `Unwired`, and production-disabled status. Planned update/reset controls are insensitive and have no persistence, mutation, or executor handler. Next exact work: connect live GTK field editing to the memory-only draft-edit model behind still-disabled controls, without persistence, production actions, or executor wiring.

## 2026-06-20 - Live activation draft edit bridge handoff

The branch now connects source/include and duplicate activation form field edits to a live memory-only draft-edit bridge. Entry, text-buffer, and check-button handlers update in-memory draft state, recompute draft/form/control validation, and reset memory state without persistence. The Config page displays live draft-edit cards with memory-only mode, dirty state, recomputed validation, `Not saved to disk`, executor wiring `Unwired`, and production-disabled status. Production activation controls remain disabled, no source/include or duplicate executor is wired, production flags remain false, and no real config/runtime/reload path was added. The follow-up persistence boundary now forbids saving activation drafts by default; remaining work is explicit opt-in/private-storage policy or production activation safety gates.

## 2026-06-20 - Remaining dependency scan handoff

The remaining dependency scan is recorded in `data/reports/future-capability-remaining-dependency-scan.v0.55.2.json` and `docs/FUTURE-CAPABILITY-REMAINING-DEPENDENCY-SCAN.md`. Core UI, config discovery, 341-row coverage, safe normal-scalar writes, and release packaging are effectively capped for this safe-release branch. Source/include, duplicate, structured, profile/mode, and runtime/reload are blocked by production activation. High-risk/display is blocked by high-risk recovery proof. Hyprland 0.55.4 migration is blocked by missing official export data.

## 2026-06-20 - Activation draft persistence boundary handoff

The branch now includes a default-disabled persistence boundary for source/include and duplicate activation drafts. Persistence is forbidden by default, no draft data is written to disk, storage path is `none`, no storage directory or serializer/write path exists, and source/include and duplicate executors remain `Unwired`. The Config page displays disabled persistence-boundary cards and not-available persistence/clear controls. Next work is either a full opt-in/private-storage/redaction/retention/delete/encryption policy review or production activation safety gates; no persistence should be added implicitly.

## 2026-06-20 - Production activation safety gates handoff

The branch now includes default-disabled production activation safety gates for source/include insertion and duplicate replacement. Both gates are `ProductionActivationProofPartiallySatisfiedButDefaultDisabled`; copied-fixture proof satisfies byte-exact backup, pre-write snapshot, target identity, write plan, diff preview, reread, restore, post-restore verification, and rollback checks without touching real config. Report-backed evidence satisfies no-auto-apply and persisted-draft auto-apply proof. Executors remain `Unwired`, production flags remain false, draft persistence remains forbidden by default, and no production write/reload/runtime/config path was added. Next exact work: design explicit final-approval, production-flag, executor-wiring, and live production dry-run decisions before any source/include or duplicate production executor wiring can be reconsidered.

## 2026-06-20 - Production activation safety proof handoff

The branch now includes `data/reports/default-disabled-production-activation-safety-gate-proof.v0.55.2.json` and disabled Config-page proof cards for source/include and duplicate production activation safety proof. The proof harness uses copied temp fixtures only, computes pre-write and backup hashes, applies a fixture-only write, rereads the target, restores from backup, verifies the restored hash equals the original hash, and records that no real config or runtime was touched. Explicit final approval, production flag decision, executor wiring decision, and live production dry-run remain required.
## 2026-06-20 - Production Activation Final Decisions

Source/include and duplicate now have final decision reviews after the copied-fixture safety proof layer. These reviews keep final approval, production flag opt-in, executor wiring opt-in, and live production dry-run policy missing/required. They explicitly prove copied-fixture proof, draft edit state, and persistence-boundary state cannot approve production, set production flags, wire executors, or authorize live dry-runs.

Next exact work item: define the future explicit user-approval UX and live production dry-run policy without wiring source/include or duplicate production executors.
