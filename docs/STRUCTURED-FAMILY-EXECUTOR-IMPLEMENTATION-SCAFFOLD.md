# Structured-Family Executor Implementation Scaffold

This is an actual executor implementation scaffold for future structured-family real-write work.

The scaffold is inert and unwired. It compiles and can be tested. It rejects by default. No executor wiring is approved. No real writes are enabled. No user-facing real-write controls are designed or wired. No backup/restore execution is enabled. No rollback execution is enabled. No reload/runtime mutation is enabled. No family ranking or activation subset is selected.

The project remains not production ready for structured-family real writes.

## Scaffold Module

The scaffold lives in `src/structured_family_safe_write.rs` and is exported by `src/lib.rs`.

The module defines inert scaffold models only:

- `StructuredFamilySafeWritePlan`.
- `StructuredFamilySafeWritePreflight`.
- `StructuredFamilySafeWriteTargetPolicy`.
- `StructuredFamilySafeWriteBackupPlan`.
- `StructuredFamilySafeWriteRollbackPlan`.
- `StructuredFamilySafeWriteApprovalState`.
- `StructuredFamilySafeWriteExecutionReceipt`.
- `StructuredFamilySafeWriteAuditRecord`.
- `StructuredFamilySafeWriteEmergencyStop`.
- `StructuredFamilySafeWriteRejectionReason`.
- `StructuredFamilySafeWriteScaffoldStatus`.

## Scaffold Functions

The scaffold exposes functions that validate, describe, or reject future plans:

- `build_safe_write_plan_scaffold`.
- `validate_safe_write_preflight_scaffold`.
- `validate_safe_write_target_policy_scaffold`.
- `prepare_safe_write_backup_plan_scaffold`.
- `prepare_safe_write_rollback_plan_scaffold`.
- `verify_manual_approval_state_scaffold`.
- `execute_safe_write_scaffold`.
- `verify_safe_write_result_scaffold`.
- `emit_safe_write_audit_record_scaffold`.
- `emergency_stop_reason_scaffold`.

`execute_safe_write_scaffold` does not write anything. It always returns a rejected, inert receipt with execution false.

## Default Rejections

The scaffold rejects by default for these universal reasons:

- `ExecutorWiringNotApproved`.
- `RealWriteScopeNotApproved`.
- `RealConfigTargetNotApproved`.
- `BackupExecutionNotApproved`.
- `RestoreExecutionNotApproved`.
- `RollbackExecutionNotApproved`.
- `HyprlandReloadNotApproved`.
- `RuntimeMutationNotApproved`.
- `FirstRealConfigWriteNotApproved`.
- `GuiRealWriteControlsNotApproved`.
- `UnsupportedOrNotProvenRecord`.
- `BlockedPlan`.
- `ActivationSubsetNotSelected`.
- `ProductionReadinessNotApproved`.

These rejection reasons are not family-specific and do not rank families.

## Boundaries

The scaffold is not imported by `src/write_flow.rs`. It is not called by `apply_setting_change`. It is not imported by `src/ui/model.rs` or `src/ui/window.rs`. No GTK control can trigger it. No Apply button wiring was added.

The scaffold source does not call file-write APIs, does not create backups, does not restore files, does not create rollback files, does not run commands, does not call `hyprctl`, does not reload Hyprland, and does not mutate runtime.

## Remaining Approvals

Still required before any future production behavior:

- Executor wiring planning approval.
- Executor wiring approval.
- GUI real-write controls approval.
- Real write scope approval.
- Backup execution approval.
- Restore execution approval.
- Rollback execution approval.
- Reload policy approval.
- Runtime mutation policy approval.
- First real config write approval.

A later sprint must stop before executor wiring unless the user explicitly approves executor wiring planning.
