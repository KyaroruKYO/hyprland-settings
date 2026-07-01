# Structured Family Production Activation Planning Scope

## Decision

Option B was approved: production activation planning scope only.

This is planning-only.

No real writes are approved.

No executor implementation is approved.

No executor wiring is approved.

No backup/restore execution is approved.

No reload or runtime mutation is approved.

No family ranking or activation subset is approved.

The project remains not production ready for structured-family real writes.

## Approved Planning Scope

- Production activation planning document.
- Executor architecture design requirements.
- Backup and restore design requirements.
- Rollback and recovery design requirements.
- Validation evidence design requirements.
- Manual approval checkpoint design.
- Future implementation stop-gate design.

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
- Family ranking.
- Activation subset selection.

## Future Stop Gates

- A later sprint must stop before implementing an executor unless the user explicitly approves executor implementation.
- A later sprint must stop before wiring an executor unless the user explicitly approves executor wiring.
- A later sprint must stop before real config writes unless the user explicitly approves the first real config write.
- A later sprint must stop before backup/restore execution unless the user explicitly approves backup/restore execution.
- A later sprint must stop before Hyprland reload unless the user explicitly approves reload policy.
- A later sprint must stop before runtime mutation unless the user explicitly approves runtime mutation policy.

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
- Family ranking excluded: true.
- Activation subset selected: false.
- Production readiness decision: not production ready.

## Boundary

The next sprint may create a design document only, unless the user explicitly expands scope.

GTK evidence root: `not-run-no-visible-ui-change`.

## Next Work

Create a planning-only structured-family production activation design document that does not implement or wire an executor.
