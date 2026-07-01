# Structured-Family Executor Architecture Implementation Plan

This is an executor architecture implementation-planning document for future structured-family safe writes.

It plans how a future executor architecture could be implemented later. No executor is implemented in this sprint. No executor module with executable behavior is created in this sprint. No executor is wired. No real writes are enabled. No user-facing real-write controls are designed or wired. No backup/restore execution is enabled. No reload/runtime mutation is enabled. No family ranking or activation subset is selected.

The project remains not production ready for structured-family real writes.

## Current Approval Boundary

Approved:

- Executor architecture implementation planning.

Not approved:

- Actual executor implementation.
- Executor wiring.
- Real config writes.
- GUI real-write controls.
- Real apply button wiring.
- Real backup creation.
- Real restore execution.
- Rollback execution.
- Hyprland reload.
- Runtime mutation.
- First real config write.
- Family ranking.
- Activation subset selection.
- Production readiness.

## Future Module Plan

The future module plan names boundaries only. It does not create executable modules in this sprint.

- Future `structured_family_safe_write_executor` module boundary.
- Future `structured_family_safe_write_plan` model boundary.
- Future `structured_family_safe_write_preflight` model boundary.
- Future `structured_family_safe_write_backup` model boundary.
- Future `structured_family_safe_write_rollback` model boundary.
- Future `structured_family_safe_write_audit` model boundary.

All future modules require separate implementation approval before creation.

## Future Type Plan

Future implementation planning would need types such as:

- `StructuredFamilySafeWritePlan`.
- `StructuredFamilySafeWritePreflight`.
- `StructuredFamilySafeWriteTargetPolicy`.
- `StructuredFamilySafeWriteBackupPlan`.
- `StructuredFamilySafeWriteRollbackPlan`.
- `StructuredFamilySafeWriteApprovalState`.
- `StructuredFamilySafeWriteExecutionReceipt`.
- `StructuredFamilySafeWriteAuditRecord`.
- `StructuredFamilySafeWriteEmergencyStop`.

These are future type names only. They are not implemented as executable write behavior in this sprint.

## Future Function Plan

Future implementation planning would need functions such as:

- `build_safe_write_plan`.
- `validate_safe_write_preflight`.
- `validate_safe_write_target_policy`.
- `prepare_safe_write_backup_plan`.
- `prepare_safe_write_rollback_plan`.
- `verify_manual_approval_state`.
- `execute_safe_write`.
- `verify_safe_write_result`.
- `emit_safe_write_audit_record`.
- `emergency_stop_reason`.

These function names are future design targets. They are not created as executable paths in this sprint.

## Interface Boundaries

The future executor architecture must keep these boundaries explicit:

- Review-only structured-family projections remain separate from execution.
- In-memory drafts remain separate from future persisted write plans.
- Rendered record candidates remain separate from real config targets.
- Approval state remains separate from execution receipts.
- Backup plans remain separate from backup execution.
- Rollback plans remain separate from rollback execution.
- Executor boundary remains unreachable from the current UI.
- Executor boundary remains disconnected from `apply_setting_change`.
- Executor boundary remains disconnected from `write_flow`.

## Future Inputs And Outputs

Future executor inputs would require approved safe-write plans, target policy, backup policy, rollback policy, manual confirmation state, rendered record candidates, version compatibility proof, and unsupported/not-proven preservation proof.

Future executor outputs would require an execution receipt, post-write reread report, post-write validation report, audit record, rollback availability report, and emergency stop report on failure.

## Validation Objects

Future validation objects must cover parser validation, semantic validation, target reread, candidate render/reread, manual diff review, Hyprland verify-config or equivalent validation, post-write reread, post-write verification, and rollback availability.

## Backup/Restore Objects

Future backup/restore objects must cover backup location policy, backup retention policy, backup integrity hash, backup reread proof, restore target validation, restore reread proof, post-restore validation, and restore approval.

## Rollback/Recovery Objects

Future rollback/recovery objects must cover rollback policy, failed-write recovery, interrupted-write recovery, partial-write recovery, restore-failure recovery, emergency stop, user-visible recovery instructions, and rollback approval.

## Audit Objects

Future audit objects must cover operation summary, source snapshot summary, candidate rendered record summary, target policy summary, backup policy summary, rollback policy summary, approval summary, validation summary, failure recovery summary, and sensitive data redaction.

## Test Plan

Future tests must prove:

- Executor remains unreachable from UI before wiring approval.
- Executor remains disconnected from `apply_setting_change` before approval.
- Executor remains disconnected from `write_flow` before approval.
- Blocked plans are rejected.
- Unsupported/not-proven records are rejected.
- Raw fallback behavior is preserved.
- Backup approval is required.
- Rollback approval is required.
- First-real-write approval is required.
- Hyprland reload cannot happen without reload approval.
- Runtime mutation cannot happen without runtime approval.
- GUI real-write controls cannot exist before GUI approval.

## Source Guards

The future implementation-planning section must retain guards against command runners, filesystem writes, file creation, raw file write calls, JSON persistence calls, real user Hyprland config paths, user Hyprland config directories, GTK button wiring, and click-triggered real-write wiring.

## UI Reachability Boundary

There are no user-facing real-write controls in this sprint. There is no Apply button wiring for structured-family real writes. There is no GTK control that can trigger real writes. The current UI remains review-only for structured-family records. Future GUI design requires separate explicit approval.

## Stop Gate

A later sprint must stop before actual executor implementation unless the user explicitly approves actual executor implementation scaffold.
