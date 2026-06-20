# Future Capability Marathon Handoff

## Current state
- Branch: future-capability-marathon
- Starting commit: 895b67281f7551789e5b4a07c0ea849db1eab622
- Latest commit: see branch HEAD after final handoff commit
- Release artifacts preserved: `dist/v0.1.0`
- v0.1.0 tag modified: no
- Real config touched: no
- Runtime touched: no

## Completed phases
- All seven future capability tracks were reviewed and recorded.
- Missing/default insertion received safe-env-only planner/executor proof, disabled review scaffolding for unsupported layouts, and a production-enabled single-root normal-scalar safe-batch insertion path.
- Missing/default insertion now has a source/include target-selection readiness model; source/include production insertion remains blocked.
- Missing/default insertion now has a disabled source/include target-selection review surface in the setting detail pane; it shows root config, candidate targets, selected target state, readiness state, and planned disabled actions.
- Missing/default insertion now has source/include fixture target-selection proof; fixture plans require an explicit selected target and block no-selection, generated/script-managed, symlink/profile, unknown, duplicate, and ambiguous targets.
- Missing/default insertion now has a source/include selected-target dry-run planner that previews the exact selected target and inserted line for root/source temp fixtures while keeping production source/include insertion disabled.
- Duplicate resolution received a read-only occurrence model and safe-env exact-line replacement proof.
- Duplicate detail UI now has a disabled occurrence selector showing file, line, raw line, value, and source depth.
- Duplicate resolution now has a disabled review workflow for no selection, invalid selection, and selected-but-production-disabled states.
- Duplicate resolution now has a confirmation token/fingerprint model for missing, pending, confirmed, rejected, and expired states; production duplicate writes remain disabled.
- Duplicate resolution now has a production approval gate scaffold with occurrence preconditions and a confirmation-gated safe-env replacement wrapper; production duplicate writes remain disabled.
- Duplicate detail UI now has a disabled pre-Apply approval review showing approval state, precondition fingerprint, block reason, and a disabled planned confirmation action.
- High-risk/display recovery received a mock watchdog state machine.
- High-risk/display recovery received a disabled review model backed by mock watchdog state.
- High-risk/display recovery now has a rollback proof workflow model for backup, reread, timeout restore, and restore reread requirements.
- High-risk/display recovery now has a no-op live-readiness protocol that refuses real config paths and runtime mutation.
- Structured families received read-only disabled editor scaffolding for raw structured entries.
- Structured family scaffolding now rejects invalid prefixes and multiline candidate input.
- Structured families now have a disabled review workflow for `hl.bind` edit candidates with raw-line and comments/order preservation requirements.
- Structured families now have `hl.bind` lossless render and safe-env exact-line edit proof for temp fixtures; production structured writes remain disabled.
- Profile/mode switching received safe-env-only temp symlink switch/restore proof.
- Profile/mode switching received disabled review scaffolding and forced restore-failure coverage.
- Profile/mode switching now has a disabled selection review model for current profile, resolved target, selected target, and symlink path.
- Profile/mode switching now has a target approval review model that refuses real-session paths and keeps safe-env targets review-only.
- Runtime/reload received a dry-run action boundary and mock executor.
- Runtime/reload received a runtime action policy scaffold.
- Runtime/reload now has a disabled action review workflow combining policy, dry-run result, and execution log.
- Runtime/reload now classifies status as read-only and reload/keyword/dispatch as mutating risk classes without executing commands.
- Hyprland 0.55.4 migration received disabled assessment scaffolding.
- Hyprland 0.55.4 migration now has a side-by-side comparison review that keeps v0.55.2 active and records missing proof.
- Hyprland 0.55.4 migration now has a trusted-export requirement model; 0.55.2 remains the active default bundle.
- Controlled live-test guard model now records backup, SHA256, symlink, runtime snapshot, restore, post-restore verification, out-of-band recovery, trusted-data, and explicit live flag prerequisites before any live/system mutation can be considered.
- No production write expansion was enabled.
- Deterministic tests were added for each concrete safe-env/mock model and disabled production status.

## Partial phases
- Missing/default insertion is enabled only for reviewed single-file normal scalar safe-batch targets; source/include target selection and managed/duplicate/high-risk/structured/profile/runtime insertion remain blocked.
- Source/include target-selection UI is visible but disabled and does not write connected files.
- Duplicate resolution has no production write path despite safe-env exact-line proof, confirmation-token scaffolding, and a production approval gate model.
- Runtime/reload and high-risk recovery remain mock/dry-run/no-op only.

## Blocked phases
- High-risk/display-render production writes require live recovery proof and explicit approval.
- Profile/mode switching requires explicit approval before real symlink/profile changes.
- Hyprland 0.55.4 migration requires trusted export/source proof before changing app data.

## Next exact work item
Wire source/include selected-target dry-run preview into the disabled detail UI and add a temp-fixture guarded live-test executor for non-real config paths.

## Progress tracker
- Core app shell / UI / navigation: 92-96% -> 92-96%
- Config discovery / source-aware model: 90-93% -> 91-94%
- 341-row read/write model: 90-95% -> 90-95%
- Safe normal-scalar writes: 92-96% -> 93-96%
- Release packaging/tag/artifacts: 85-95% -> 85-95%
- Missing/default insertion: 87-92% -> 89-93%
- Duplicate resolution: 73-81% -> 74-82%
- High-risk/display recovery: 50-60% -> 52-62%
- Structured-family editors/writes: 48-58% -> 50-60%
- Profile/mode switching: 50-60% -> 52-62%
- Runtime/reload integration: 45-55% -> 47-57%
- Hyprland 0.55.4 migration: 35-45% -> 37-47%

## Validation status
Passed: `cargo fmt`, `cargo fmt --check`, `cargo check`, `cargo test`, `cargo build --release`, `jq empty data/reports/*.json`, `git diff --check`, and `tools/live_scenario_harness/run_gtk_evidence_matrix.sh`.
GTK evidence root: `/tmp/hyprland-settings-gtk-automation/20260619_101038`.

## Recommended next Codex prompt
Wire source/include selected-target dry-run preview into the disabled detail UI and add a temp-fixture guarded live-test executor for non-real config paths.
