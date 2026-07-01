# Structured-Family Internal Safe-Write Architecture Plan

This is an internal safe-write architecture planning document for future structured-family real writes. It comes before GUI real-write controls.

This sprint is planning-only. No executor is implemented. No executor is wired. No real writes are enabled. No user-facing real-write controls are designed or wired. No backup or restore execution is enabled. No rollback execution is enabled. No reload or runtime mutation is enabled. No family ranking or activation subset is selected. The project remains not production ready for structured-family real writes.

## Architecture Boundary

The future architecture must separate the review-only model from any future execution model. Draft editing remains separate from future write approval. Rendered-record generation remains separate from future real config writes. Future executor implementation remains separate from future executor wiring. Future executor wiring remains separate from the first real config write. Source/include production activation and duplicate production activation remain separate scopes.

## Future Pipeline Stages

- Read current config snapshot.
- Project structured-family records.
- Edit in-memory draft.
- Render candidate record.
- Reread candidate from fixture or temporary config.
- Review diff.
- Collect manual approvals.
- Validate target policy.
- Validate backup policy.
- Validate rollback policy.
- Validate version compatibility.
- Stop at the future executor boundary.
- Stop at the future first-write gate.
- Require post-write reread and verification only after separately approved future execution.

## Future Boundary Objects

- `StructuredFamilyDraft` remains in-memory until separate persistence approval.
- `RenderedRecordCandidate` represents candidate text only.
- `SafeWritePlan` is future-only and not executable in this sprint.
- `SafeWritePreflightReport` is future-only evidence summary.
- `SafeWriteBackupPlan` is future-only and does not create backups in this sprint.
- `SafeWriteRollbackPlan` is future-only and does not create rollback files in this sprint.
- `SafeWriteApprovalState` is future-only and requires explicit user approval.
- `SafeWriteExecutionReceipt` is future-only and cannot exist until real writes are approved.

## Future Executor Boundary

The executor module is future-only. It is not reachable from current UI, not wired to structured-family records, not called by `apply_setting_change`, and not called by `write_flow`. It cannot run without explicit implementation approval, cannot be wired without explicit wiring approval, and cannot write without first-real-write approval. It must reject blocked plans, reject unsupported or not-proven records, and preserve raw fallback behavior.

## Validation Gates

Future execution must require parser validation, semantic validation, target reread, candidate render/reread, manual diff review, Hyprland verify-config or equivalent validation, post-write reread, post-write verification, and rollback availability checks. This sprint performs none of those as real-write execution.

## Backup/Restore Gates

Future execution must require backup location approval, backup retention approval, backup integrity hash proof, backup reread proof, restore target validation, restore reread, post-restore validation, and separate restore execution approval. This sprint creates no backups and executes no restore.

## Rollback/Recovery Gates

Future execution must require rollback policy approval, failed-write recovery path, interrupted-write recovery path, partial-write recovery path, restore-failure recovery path, emergency stop procedure, user-visible recovery instructions, and separate rollback execution approval. This sprint creates no rollback files and executes no recovery.

## Audit Log Requirements

Future audit logs must cover planned operation summary, source snapshot summary, candidate rendered record summary, target policy summary, backup policy summary, rollback policy summary, approval summary, validation summary, failure/recovery summary, and sensitive data redaction rules.

## Emergency Stop Conditions

The future pipeline must stop before executor implementation, executor wiring, GUI controls, backup execution, restore execution, rollback execution, real config write, reload, and runtime mutation unless the matching explicit approval exists. It must also stop on dirty tracked preflight, target ambiguity, unsupported or not-proven records, and failed validation.

## UI Reachability Boundary

There are no user-facing real-write controls in this sprint. There is no Apply button wiring for structured-family real writes. There is no GTK control that can trigger real writes. The current UI remains review-only for structured-family records. Future UI design requires separate explicit approval.

## Next Stop Gate

A later sprint must stop before executor implementation planning unless the user explicitly approves that scope.

Next exact work item:

```text
Stop for explicit user decision: approve or reject future executor architecture implementation planning.
```
