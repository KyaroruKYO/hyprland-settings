# Structured-Family Production Activation Design

This is the planning-only production activation design document for structured-family rendered-record writes.

Option B remains planning-only. No executor is implemented. No executor is wired. No real writes are enabled. No backup or restore execution is enabled. No reload or runtime mutation is enabled. No family ranking or activation subset is selected. The project remains not production ready for structured-family real writes.

## Approved Scope

- Production activation planning design only.
- Future architecture requirements.
- Future executor boundary requirements.
- Future config target policy requirements.
- Future backup/restore design requirements.
- Future rollback/recovery design requirements.
- Future validation sequence requirements.
- Future manual confirmation requirements.
- Future audit logging requirements.
- Future emergency stop requirements.

## Not Approved

- Executor implementation.
- Executor wiring.
- Real config writes.
- Real backup creation.
- Real restore execution.
- Rollback execution.
- Hyprland reload.
- Runtime mutation.
- First real config write.
- Family safety ranking.
- Activation subset selection.
- Production readiness.

## Future Architecture

The design artifact is review-only. A future production activation path would need separate phases for design, executor implementation, executor wiring, and the first real write. There must be no automatic transition from planning to implementation. Source/include production activation and duplicate production activation remain separate scopes and cannot be bundled into structured-family activation by implication.

## Future Executor Boundary

The executor is future-only and must not be reachable from the current UI. Executor implementation requires separate explicit user approval. Executor wiring requires separate explicit user approval. Any future executor must reject blocked plans, reject unsupported or not-proven records, preserve raw fallback behavior, and require manual confirmation before any future real write.

## Future Config Target Policy

A real config target must be explicitly approved. Source/include target policy must also be explicitly approved. A future write target must be reread before write and validated after write. Ambiguous targets must block activation. Generated or script-managed targets must block activation or require explicit policy. The Hyprland 0.55.2 project model versus live Hyprland 0.55.4 compatibility boundary must be resolved before write activation.

## Future Backup/Restore Design

A future activation plan must define backup location policy, backup retention policy, backup integrity hash proof, backup reread proof, restore target validation proof, restore reread proof, post-restore Hyprland validation proof, and a post-restore reload or restart policy approval gate. This sprint does not create backups and does not execute restore.

## Future Rollback/Recovery Design

A future activation plan must define rollback file policy, failed-write recovery path, interrupted-write recovery path, partial-write recovery path, restore failure recovery path, emergency stop path, and user-visible recovery instructions. This sprint does not create rollback files and does not execute rollback or recovery.

## Future Validation Sequence

A future activation plan must define pre-write parser validation, pre-write semantic validation, fixture render/reread validation, temporary config validation, Hyprland verify-config or equivalent validation, manual diff review, post-write reread validation, post-write config verification, and post-write rollback availability checks.

## Future Manual Confirmation

A future activation plan must require confirmation of the exact activation scope, exact config target, backup location and retention policy, reload policy, runtime mutation policy, rollback/recovery policy, and first real config write.

## Future Audit Logging

A future activation plan must define audit logging for planned operations, pre-write evidence, backup artifacts, write targets, validation results, manual approval, failure and recovery summaries, and must avoid leaking secret or sensitive data.

## Future Emergency Stop

A future activation plan must stop before executor implementation, executor wiring, backup execution, restore execution, real config write, reload, and runtime mutation unless the matching user approval exists. It must also stop on failed preflight, failed validation, ambiguous target, and unsupported or not-proven records.

## Stop Gate

A later sprint must stop before implementation unless the user explicitly approves implementation-planning scope. The next step is not executor implementation.

Next exact work item:

```text
Stop for explicit user decision: approve or reject a future executor architecture implementation-planning sprint.
```
