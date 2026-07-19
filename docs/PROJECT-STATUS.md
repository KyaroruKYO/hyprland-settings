# Project Status

Status date: 2026-07-18. This file describes the unreleased `structured-family-editors-unified` branch after write stabilization. The published `v0.2.0` release remains at `0ffaeb3` and does not include all behavior described here.

## Current Counts

| Surface | Current branch |
| --- | ---: |
| Scalar rows modeled/readable | 341 |
| Scalar rows editable | 290 |
| Guarded live-preview rows | 135 |
| Supervised dead-man rows | 38 |
| Save-only rows | 117 |
| High-risk rows blocked | 51 |
| Structured families classified | 7 |
| Structured families with proven production persistence | 2 |
| Structured families blocked from production persistence | 5 |

The two persisted families are `hl.animation` and `hl.curve`, limited to proven modify-existing record shapes. `hl.monitor`, `hl.bind`, `hl.gesture`, `hl.device`, and `hl.permission` remain blocked.

## Stabilization Result

- Save-state transitions occur only after durable, reread-verified receipts.
- Failed scalar and structured preview saves keep pending state and recovery controls.
- Exact target bytes, SHA-256, inode/device, ownership, mode, parent identity, record shape, occurrence count, and source graph form the write precondition.
- Drift aborts without an automatic overwrite or merge.
- Active writes use the shared hardened Linux atomic-exchange primitive.
- Backups use the XDG state location, exclusive creation, directory mode `0700`, file mode `0600`, synchronization, and byte/hash verification.
- Verification failures restore exact bytes and verified mode/ownership; restore refuses intervening edits.
- One-file pending batches stage all rows and commit once. Multi-file batches reject before writing.
- The normal test suite is hermetic; real-machine audits and report regeneration require explicit gates.

## Runtime And Save Behavior

Runtime preview is real, guarded, and reversible. It uses fixed-shape mutating `hyprctl eval` operations for approved preview surfaces, with readback and Revert/Cancel recovery. The app never runs `hyprctl reload`.

Production scalar persistence exists behind Safe Live Save Mode and write gates. The active branch also persists proven Animation and Curve record shapes. No additional row or family was promoted during stabilization.

## Honest Limitations

- Cross-file save batches are not crash-atomic and are therefore rejected before writing.
- ACLs, extended attributes, and timestamps are not preserved as guarantees.
- The hardened commit requires Linux `renameat2(RENAME_EXCHANGE)` support and fails closed when unavailable.
- Ownership must already be reproducible by the current process; the writer does not elevate privileges.
- The 51 high-risk rows still require their dedicated production recovery gates.
- Five structured families, profiles/modes, broad source/include activation, duplicate auto-resolution, and style editing remain blocked.
- The data model remains pinned to Hyprland `0.55.2`; the live compositor may be `0.55.4`, but no migration is activated.

## Validation Boundary

Normal tests use only test-owned files and mock runtimes. Live runtime proofs and active-config write proofs remain ignored/env-gated and were not run during this stabilization sprint. The user's active config was not written.

Final evidence: two normal and one isolated HOME/XDG full-suite runs each
reported 1,154 passed, zero failed, and 25 ignored tests with no status, diff,
or report churn. The safe-env GTK matrix completed 28 screenshot and AT-SPI
runs without a config write, backup, reload, or runtime mutation.

## Next Review

Run an independent review of the stabilized write, restore, and pending-state paths before approving any new feature, release, or live-write proof work.

## Historical Status

Earlier planning and completion narratives are historical. They remain available in versioned reports under `data/reports/`, dedicated documents under `docs/`, and the pre-stabilization handoff at:

```sh
git show d4d3489:docs/CURRENT-PROJECT-HANDOFF.md
```
