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
- Missing/default insertion received safe-env-only planner/executor proof plus disabled production review scaffolding.
- Duplicate resolution received a read-only occurrence model and safe-env exact-line replacement proof.
- Duplicate detail UI now has a disabled occurrence selector showing file, line, raw line, value, and source depth.
- Duplicate resolution now has a disabled review workflow for no selection, invalid selection, and selected-but-production-disabled states.
- High-risk/display recovery received a mock watchdog state machine.
- High-risk/display recovery received a disabled review model backed by mock watchdog state.
- High-risk/display recovery now has a rollback proof workflow model for backup, reread, timeout restore, and restore reread requirements.
- Structured families received read-only disabled editor scaffolding for raw structured entries.
- Structured family scaffolding now rejects invalid prefixes and multiline candidate input.
- Structured families now have a disabled review workflow for `hl.bind` edit candidates with raw-line and comments/order preservation requirements.
- Profile/mode switching received safe-env-only temp symlink switch/restore proof.
- Profile/mode switching received disabled review scaffolding and forced restore-failure coverage.
- Profile/mode switching now has a disabled selection review model for current profile, resolved target, selected target, and symlink path.
- Runtime/reload received a dry-run action boundary and mock executor.
- Runtime/reload received a runtime action policy scaffold.
- Runtime/reload now has a disabled action review workflow combining policy, dry-run result, and execution log.
- Hyprland 0.55.4 migration received disabled assessment scaffolding.
- Hyprland 0.55.4 migration now has a side-by-side comparison review that keeps v0.55.2 active and records missing proof.
- No production write expansion was enabled.
- Deterministic tests were added for each concrete safe-env/mock model and disabled production status.

## Partial phases
- Missing/default insertion has safe-env-only append-section insertion with backup, reread verification, and restore-on-failure.
- Duplicate resolution has no production write path despite safe-env exact-line proof.
- Runtime/reload and high-risk recovery remain mock/dry-run only.

## Blocked phases
- High-risk/display-render production writes require live recovery proof and explicit approval.
- Profile/mode switching requires explicit approval before real symlink/profile changes.
- Hyprland 0.55.4 migration requires trusted export/source proof before changing app data.

## Next exact work item
Define explicit approval gates and architecture for any production activation track. No remaining future-capability track has a safe production-enablement path without user approval, trusted external data, or new architecture.

## Validation status
Passed: `cargo fmt`, `cargo fmt --check`, `cargo check`, `cargo test`, `cargo build --release`, `jq empty data/reports/*.json`, `git diff --check`, and `tools/live_scenario_harness/run_gtk_evidence_matrix.sh`.
GTK evidence root: `/tmp/hyprland-settings-gtk-automation/20260619_101038`.

## Recommended next Codex prompt
Review the `future-capability-marathon` branch and choose which blocked production activation track, if any, should receive explicit approval and a dedicated architecture sprint.
