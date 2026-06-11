# One-target Pilot Gate-flip Proposal Reviewed Draft

This is a reviewed draft only. It was not executed. No gate was flipped. It does not enable writes. It requires explicit user approval and a separate implementation/gate-flip sprint before any production write pilot can run.

## Review Result
- Original draft status: mostly correct, but clarification was required.
- Reviewed draft status: suitable to present for explicit user approval in a later sprint.
- Gate flip executed: no.
- Writes enabled: no.

## Proposed Future Scope
- Allow only one existing non-high-risk scalar line in one normal config file.
- Require exact line number, no duplicate ambiguity, backup, reread verification, and recovery.
- Keep session-selected config preview-only unless a later sprint explicitly approves production target selection behavior.

## Excluded Targets
- Generated targets.
- Script-managed targets.
- Script-referenced targets.
- Symlink-managed targets.
- Symlink targets.
- Unknown management state targets.
- Targets requiring script or Lua execution to understand.
- High-risk rows.
- Structured targets.
- Missing-line targets.
- Duplicate or ambiguous targets.

## Staged Gates For Future Review
These gates should not be flipped together without the required proof for each stage.

1. `PRODUCTION_ONE_TARGET_PRE_ENABLE_AUDIT_PASSED`
2. `PRODUCTION_BACKUP_CONTRACT_ENABLED`
3. `PRODUCTION_VERIFICATION_CONTRACT_ENABLED`
4. `PRODUCTION_RECOVERY_CONTRACT_ENABLED`
5. `PRODUCTION_WRITE_TARGET_REVIEW_ENABLED`
6. `PRODUCTION_WRITE_TARGET_SELECTION_READY`
7. `PRODUCTION_ONE_TARGET_WRITE_PILOT_ENABLED`
8. `PRODUCTION_WRITE_REVIEW_WALKTHROUGH_CAN_WRITE`

`PRODUCTION_ADVANCED_CONFIRMATION_ENABLED` and `PRODUCTION_HIGH_RISK_APPROVAL_ENABLED` should remain false for the first normal-only pilot.

## Required Proof
- Focused live read-only visual smoke review passed.
- Fixture-only backup, write, reread verification, and recovery proof exists.
- Production backup implementation must create an exact backup before write.
- Backup bytes must match original bytes before any write continues.
- Production write must affect exactly one expected scalar line.
- Reread verification must read the exact target file after write.
- Verification must confirm the expected value.
- Recovery must restore exact backup bytes on write or verification failure.
- Restore verification must reread the restored file.
- Restore verification must verify restored bytes and the restored scalar value.
- Recovery failure must report the failure without hiding the backup path.

## Apply Integration Scope
- Future Apply integration may only touch the approved one-target path.
- Existing `high_risk_write_policy` behavior must remain enforced.
- Blocked target classes remain blocked.
- Session-selected config does not automatically affect writes.
- Real target selection must prove a normal scalar file occurrence.
- Backup, verification, and recovery must wrap any write.
- Errors must block the write or report failure safely.

## Stop Conditions
- Target is generated, script-managed, script-referenced, symlink-managed, symlink target, unknown-management, script/Lua-required, high-risk, structured, missing-line, duplicate, or ambiguous.
- Backup cannot be created and verified before write.
- Write target is not one normal scalar line.
- Reread verification fails.
- Recovery restore or restore verification fails.
- Apply integration differs from the approved one-target scope.
- Any required gate remains false in the future sprint.

## Runtime Safety
- Do not reload Hyprland automatically.
- Do not run mutating `hyprctl`.
- Do not run scripts or Lua.
- Do not switch profiles or modes.
