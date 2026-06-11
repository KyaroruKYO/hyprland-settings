# One-target Pilot Gate-flip Proposal Draft

This is a draft only. It was not executed. No gate was flipped. It requires explicit user approval and a separate sprint before any production write pilot can be enabled.

## Proposed Future Scope
- Allow only one existing non-high-risk scalar line in one normal config file.
- Require exact line number, no duplicate ambiguity, backup, reread verification, and recovery.
- Continue excluding generated, script-managed, script-referenced, symlink-managed, symlink target, high-risk, structured, missing-line, and ambiguous targets.

## Gates Proposed For Future Review
- `PRODUCTION_ONE_TARGET_PRE_ENABLE_AUDIT_PASSED`
- `PRODUCTION_WRITE_TARGET_SELECTION_READY`
- `PRODUCTION_WRITE_TARGET_REVIEW_ENABLED`
- `PRODUCTION_BACKUP_CONTRACT_ENABLED`
- `PRODUCTION_VERIFICATION_CONTRACT_ENABLED`
- `PRODUCTION_RECOVERY_CONTRACT_ENABLED`
- `PRODUCTION_ONE_TARGET_WRITE_PILOT_ENABLED`
- `PRODUCTION_WRITE_REVIEW_WALKTHROUGH_CAN_WRITE`

`PRODUCTION_ADVANCED_CONFIRMATION_ENABLED` and `PRODUCTION_HIGH_RISK_APPROVAL_ENABLED` should remain false for the first normal-only pilot.

## Required Proof Already Present
- Focused live read-only visual smoke review passed.
- Fixture-only backup, write, reread verification, and recovery proof exists.
- Advanced confirmation and high-risk target classes are excluded.
- Apply/write isolation tests prove the production write path is disconnected.
- All production gates remain false.

## Proof Still Missing
- Explicit user approval for a separate gate-flip sprint.
- Active production backup, write, reread verification, and recovery implementation.
- Approved Apply integration limited to the one-target pilot path.

## Stop Conditions
- Target is generated, script-managed, script-referenced, symlink-managed, a symlink target, high-risk, structured, missing-line, or ambiguous.
- Backup cannot be created and verified before write.
- Reread verification fails.
- Recovery restore or restore verification fails.
- Apply integration differs from the approved one-target scope.
- Any required gate remains false in the future sprint.

## Rollback Conditions
- Write fails after backup.
- Write succeeds but reread verification fails.
- Expected setting is missing after write.
- Expected value mismatches after write.
- Target file is unreadable after write.
