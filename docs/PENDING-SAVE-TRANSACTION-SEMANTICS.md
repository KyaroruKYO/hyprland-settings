# Pending Save Transaction Semantics

## Chosen policy

The supported pending-save transaction is one target file per operation.

For one-file batches the app:

1. snapshots every pending source without clearing it;
2. validates Safe Live Save Mode, row eligibility, values, target/source mapping, duplicate and overlap rules, exact drift preconditions, and preview-controller Saved transitions;
3. applies every change to one in-memory candidate;
4. parser-rereads and verifies that complete candidate;
5. creates one verified backup;
6. performs one synchronized atomic target exchange;
7. rereads and verifies all intended values;
8. marks all runtime sessions saved and clears all ledgers only after the durable batch receipt succeeds.

No supported one-file batch can commit only a prefix.

## Multi-file policy

Rows resolving to more than one target are rejected before backup or write. Cross-file renames are not crash-atomic, and this project does not currently implement a persistent transaction journal. The UI does not call a multi-file operation atomic.

## Failure behavior

Any preflight failure writes nothing. Commit or post-write verification failure leaves the original file intact or restores it exactly. Every pending row remains, runtime previews remain represented, and Revert/Cancel remains available.

Restore failure is reported as unrecovered and includes the verified backup path; it is never presented as a successful save.

## UI wording

- Pending action: `Save all atomically`.
- Detail action: `Stage reviewed change`.
- Success: all reviewed changes saved atomically to one file.
- Failure: batch not saved; all changes remain pending.

The detail pane has no alternate direct-write path.
