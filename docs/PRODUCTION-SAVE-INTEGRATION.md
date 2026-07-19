# Production Save Integration

> Current unreleased branch behavior after the write-stabilization work. Earlier
> reports linked below remain historical evidence for the state in which they
> were generated.

Every active-config Save is gated on **Safe Live Save Mode**. The gate reads
`misc:disable_autoreload` and fails closed unless it is active. The application
uses guarded, reversible runtime mutation for previews, but never invokes
`hyprctl reload`.

## Current Save Routes

| User action | Current route and semantics |
|---|---|
| Scalar detail action | **Stage reviewed change** adds an in-memory pending entry; it does not write. |
| Pending **Save all atomically** | Preflights every entry, requires one target file, stages all edits in memory, and performs one hardened atomic replacement. |
| Structured-family Save | Proven `hl.animation` and `hl.curve` record shapes use gated persistence; all other structured families remain blocked. |
| Safe Live Save Mode “Save as default” | Uses the hardened scalar write path and records success only after durable verification. |

The pending batch rejects multi-file operations before any backup or write. A
failed preflight, commit, verification, or restore retains every pending entry.
Runtime-preview sessions are marked saved only after a durable write receipt is
available.

## Write Guarantees

Active-config writes now share these requirements:

- an exact-byte and filesystem-identity precondition captured when planning;
- a fresh source-graph check immediately before execution where source-aware
  resolution applies;
- regular-file, owner, mode, canonical-parent, device, and inode validation;
- an exclusive, unpredictable temporary file in the target directory;
- file synchronization, Linux atomic exchange, parent-directory
  synchronization, and post-commit byte/metadata verification;
- an exclusive `0600` backup beneath an application-owned `0700` XDG state
  directory;
- exact-byte restore through the same hardened path if post-write semantic
  verification fails.

Any on-disk drift aborts without merging or overwriting external changes. The
user must reread the setting and prepare a fresh edit.

ACLs, extended attributes, and historical timestamps are not promised to be
preserved. Existing mode and ownership are verified and preserved; an ownership
assumption the process cannot satisfy fails closed.

## Boundaries

- UI code does not call `apply_setting_change` directly.
- UI code does not construct free-form `hyprctl` commands.
- High-risk scalar rows and unsupported structured-family records remain
  blocked.
- Normal tests use fixtures and temporary directories. Real-config audits are
  ignored and require `HYPRLAND_SETTINGS_RUN_REAL_CONFIG_AUDIT=1`.

The earlier integration evidence is retained in
`data/reports/production-save-integration.v0.55.2.json`. Current stabilization
truth is in `docs/SAVE-WRITE-STABILIZATION.md` and the completion report.
