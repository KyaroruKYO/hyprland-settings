# Save and Write Stabilization

Status: implementation contract for the post-v0.2.0 stabilization sprint.

This document defines the invariants that active-config Save paths must satisfy. It does not expand the writable setting or structured-family surface.

## Save-state commit ordering

UI and runtime-preview state may transition to `Saved` only after the write layer returns a durable success receipt. Durable success means the target replacement was committed, the temporary file and parent directory were synchronized, the final bytes were reread, setting/record verification passed, and the backup receipt exists. A failed write leaves every pending entry and runtime recovery controller intact. Revert and Cancel remain available.

## Write-plan preconditions and drift detection

The source-aware config snapshot captures each reviewed file's exact bytes, canonical path, Unix file identity, mode, owner, and SHA-256 digest, plus a fingerprint of the source/include graph. Every active write carries that immutable precondition. Immediately before backup creation and again immediately before rename, the writer rejects symlinks, non-regular files, parent-path substitutions, inode changes, byte changes, setting occurrence changes, insertion targets that became present, structured records that changed, and source-graph changes. Drift never triggers an automatic merge or overwrite.

## Hardened atomic replacement

The shared Unix replacement primitive:

1. validates the expected target and parent identity;
2. creates an unpredictable, exclusive temporary file in the target directory;
3. applies the target mode and safely preserves the expected owner/group;
4. writes all bytes, flushes, and calls `sync_all`;
5. revalidates the target precondition immediately before commit;
6. atomically renames the temporary file over the target;
7. synchronizes the parent directory;
8. rereads and verifies exact final bytes and required metadata;
9. removes the temporary file on every pre-commit failure.

ACLs, extended attributes, and non-Unix platforms are outside this implementation contract unless explicitly implemented and tested. Active config writes fail closed when ownership cannot be preserved safely.

## Secure backups

Active-config backups live under `$XDG_STATE_HOME/hyprland-settings/backups`, or `~/.local/state/hyprland-settings/backups` when XDG state is unset. The directory is a real directory owned by the current user and mode `0700`. Backup files are unpredictable, exclusively created, mode `0600`, synchronized, and byte/hash verified against the write precondition. A receipt records source identity, backup path, SHA-256 digest, byte length, timestamp/id, and source metadata. Existing files are never overwritten and symlinks are never followed.

## Restore and rollback

Post-write semantic verification failure triggers restoration from the verified backup through the same hardened atomic replacement path. Restore verifies the backup receipt before use, restores exact bytes and expected mode/ownership, synchronizes the target and parent directory, then rereads both bytes and metadata. Restore failure is surfaced as an unrecovered failure with the verified backup path retained; it is never reported as success.

## Pending-save transaction semantics

All pending scalar rows are preflighted together. If they resolve to one approved regular target, the app builds one combined candidate from the precondition bytes, validates every row, parser-rereads the staged text, creates one backup, and performs one atomic replacement. Controllers and ledgers clear only after the one durable receipt succeeds.

Pending rows resolving to more than one file are rejected before backup or write. The UI does not call this atomic because cross-file crash atomicity is not implemented. Users must save separately after reviewing each target. No supported Save-all operation can commit only a prefix.

## Normal-test hermeticity

Default tests use committed fixtures, temporary directories, mock runtime runners, and temporary HOME/XDG roots. Real-config and real-runtime audits require explicit environment gates and are ignored or fail closed otherwise. Normal tests never rewrite committed reports; report regeneration is a separate explicit command or environment-gated workflow.

## Documentation truth source

Current truth comes from executable classification summaries and deterministic tests. Public docs distinguish the published v0.2.0 release from unreleased branch behavior. Historical reports remain historical and are not rewritten to imply that later capabilities existed earlier.

## Non-goals

This stabilization does not promote settings, families, source/include activation, profiles, runtime previews, display recovery, or releases. It never runs `hyprctl reload`, mutates monitors, or writes the live config during validation.
