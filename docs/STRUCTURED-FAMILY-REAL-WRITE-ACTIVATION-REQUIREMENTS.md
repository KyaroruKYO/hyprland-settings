# Structured Family Real-Write Activation Requirements

## Scope

This is a requirements-only audit for future structured-family real-write activation planning.

It is not:

- A production executor.
- Executor wiring.
- A real config write path.
- A backup writer.
- A restore path.
- A rollback executor.
- A reload path.
- A runtime mutation path.
- A production activation approval.

Excluded by user instruction:

- Which families are safest.
- Which families should stay blocked.
- Family-by-family activation ranking.
- Activation subset recommendation.

## 1. Real-Write Activation Requirements

- Explicit user production activation decision.
- Explicit activation scope document.
- Real config target policy.
- Source/include target policy.
- Atomic write strategy.
- Pre-write validation.
- Post-write reread validation.
- Hyprland verify-config or equivalent validation strategy.
- Reload policy.
- Runtime mutation policy.
- Backup creation policy.
- Restore policy.
- Rollback policy.
- Failure recovery policy.
- Audit logging policy.
- Manual confirmation policy.
- Production executor implementation.
- Production executor wiring.
- Production executor tests.
- UI confirmation gate.
- Dry-run to real-write transition gate.
- Blocked-plan rejection gate.
- Unsupported/not-proven preservation gate.
- Raw fallback preservation gate.
- Version compatibility gate for Hyprland 0.55.2 vs live 0.55.4.

## 2. Missing Backup/Restore Proof

- No real backup file creation proof.
- No real backup location policy proof.
- No backup integrity hash proof.
- No backup reread proof.
- No real restore execution proof.
- No restore target validation proof.
- No restore reread proof.
- No rollback file creation proof.
- No rollback execution proof.
- No failed-write recovery proof.
- No interrupted-write recovery proof.
- No partial-write recovery proof.
- No post-restore Hyprland validation proof.
- No post-restore reload/restart policy proof.
- No user-visible recovery instructions proof.

## 3. Required User Approval Gates

- Approve entering production activation planning.
- Approve exact real-write activation scope.
- Approve config target path.
- Approve backup location and retention policy.
- Approve restore policy.
- Approve reload policy.
- Approve runtime mutation policy.
- Approve executor implementation.
- Approve executor wiring.
- Approve first real config write.
- Approve fallback behavior for unsupported/not-proven records.
- Approve raw fallback preservation behavior.
- Approve rollback procedure.
- Approve recovery procedure.
- Approve emergency stop procedure.
- Approve whether Hyprland 0.55.4 migration must happen before production activation.

## Non-Approval State

- Production activation approved: false.
- Executor implemented: false.
- Executor wired: false.
- Real write path enabled: false.
- Real config target enabled: false.
- Backup creation enabled: false.
- Restore execution enabled: false.
- Rollback execution enabled: false.
- `hyprctl reload` enabled: false.
- Runtime mutation enabled: false.
- First real config write approved: false.

## Boundary

The fixture-only proof chain remains complete, but that is not production readiness. Real-write activation still requires explicit user approval of the production activation planning scope before any executor design.

GTK evidence root: `not-run-no-visible-ui-change`.

## Next Work

Wait for explicit user approval of production activation planning scope before designing any real-write executor.
