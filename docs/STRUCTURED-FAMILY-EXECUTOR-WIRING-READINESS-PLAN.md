# Structured-Family Executor Wiring Readiness Plan

This is executor wiring planning and readiness work. It defines wiring boundaries before actual executor wiring and adds inert wiring-readiness models only.

No actual executor wiring is approved. No executor call path is added. No `write_flow` integration is added. No `apply_setting_change` integration is added. No UI integration is added. No real writes are enabled. No user-facing real-write controls are designed or wired. No backup/restore execution is enabled. No rollback execution is enabled. No reload/runtime mutation is enabled. No family ranking or activation subset is selected.

The project remains not production ready for structured-family real writes.

A later sprint must stop before an actual executor wiring scaffold unless the user explicitly approves it.

## Readiness Module

The wiring-readiness layer lives in `src/structured_family_executor_wiring.rs` and is exported by `src/lib.rs` only. It is not imported by `src/main.rs`, `src/write_flow.rs`, `src/structured_family_safe_write.rs`, or any UI module, so exporting it does not make it reachable from runtime, UI, or write paths.

The module defines inert models only:

- `StructuredFamilyExecutorWiringReadiness`.
- `StructuredFamilyExecutorWiringBoundary`.
- `StructuredFamilyExecutorWiringCandidate`.
- `StructuredFamilyExecutorWiringPreflight`.
- `StructuredFamilyExecutorWiringApprovalState`.
- `StructuredFamilyExecutorWiringRejectionReason`.
- `StructuredFamilyExecutorWiringSourceGuard`.
- `StructuredFamilyExecutorWiringReadinessReport`.
- `StructuredFamilyExecutorWiringStatus`.

## Readiness Functions

The module exposes inert planning functions only:

- `build_executor_wiring_readiness`.
- `build_executor_wiring_boundary`.
- `build_executor_wiring_candidate`.
- `validate_executor_wiring_preflight`.
- `verify_executor_wiring_approval_state`.
- `executor_wiring_rejection_reasons`.
- `executor_wiring_source_guards`.
- `executor_wiring_readiness_report`.

Every preflight and readiness function reports executor wiring approved false and executor wired false. The preflight rejects by default with `passed` false and `wiring_may_proceed` false.

## Wiring Boundaries

Any future wiring must respect these boundaries, each recorded as wiring-forbidden and requiring separate explicit approval:

- Executor scaffold boundary: the inert safe-write scaffold stays unwired; nothing may make its execute entry point reachable.
- Write-flow boundary: `write_flow` must not import or call structured-family executor code.
- Apply-setting-change boundary: `apply_setting_change` remains scalar-only.
- UI reachability boundary: no UI model, window, or control may reach executor or wiring-readiness code.
- Filesystem boundary: no real config file, include file, or user config directory may be written.
- Backup/restore boundary: no real backup creation and no restore execution.
- Rollback/recovery boundary: no rollback file creation and no rollback or recovery execution.
- Reload/runtime boundary: no Hyprland reload, no mutating `hyprctl`, no runtime mutation, no command runner.

Boundaries are universal. They are not family-specific and do not rank families.

## Wiring Candidates

Wiring candidates are boundary-level future integration points recorded for planning only, each with wired false and approved false:

- A future dedicated structured-family write coordinator would be the only allowed caller of the executor scaffold.
- A future approval-gate adapter would translate explicit user approval state into executor preflight input and fail closed on any missing approval.
- A future write-flow isolation check would keep proving the scalar write flow stays disconnected.
- Any future UI surface stays review-only until GUI real-write controls are separately approved.

Candidates name no family, no record, and no activation subset.

## Default Wiring Rejections

Wiring readiness rejects by default for these universal reasons:

- `ExecutorWiringPlanningOnly`.
- `ExecutorWiringNotApproved`.
- `ExecutorWiredFalse`.
- `ExecutorReachabilityNotApproved`.
- `WriteFlowBoundaryNotApproved`.
- `ApplySettingChangeBoundaryNotApproved`.
- `UiReachabilityNotApproved`.
- `RealWriteScopeNotApproved`.
- `RealConfigTargetNotApproved`.
- `BackupExecutionNotApproved`.
- `RestoreExecutionNotApproved`.
- `RollbackExecutionNotApproved`.
- `HyprlandReloadNotApproved`.
- `RuntimeMutationNotApproved`.
- `FirstRealConfigWriteNotApproved`.
- `GuiRealWriteControlsNotApproved`.
- `ProductionReadinessNotApproved`.

## Source Guards

`tests/structured_family_executor_wiring_readiness.rs` enforces source-level regression guards proving the wiring-readiness module does not call the executor scaffold, `write_flow`, or `apply_setting_change`; does not use filesystem write APIs, `std::fs`, `std::process`, or command runners; does not reference the real Hyprland config path; does not create GTK widgets or click handlers; and does not flip any approval flag to true. Reachability guards also re-prove that `src/main.rs`, `src/write_flow.rs`, and all UI sources reference neither the wiring-readiness module nor the executor scaffold.

Safety-policy field names such as `hyprctlReloadEnabled` in reports and docs describe policy state; they are not execution paths and do not fail these guards.

## Remaining Approvals

Still required before any future production behavior:

- Actual executor wiring scaffold approval.
- Executor wiring approval.
- GUI real-write controls approval.
- Real write scope approval.
- Backup execution approval.
- Restore execution approval.
- Rollback execution approval.
- Reload policy approval.
- Runtime mutation policy approval.
- Activation subset approval.
- First real config write approval.
- Production readiness approval.

The next step is a stop: the user must explicitly approve or reject an actual executor wiring scaffold.
