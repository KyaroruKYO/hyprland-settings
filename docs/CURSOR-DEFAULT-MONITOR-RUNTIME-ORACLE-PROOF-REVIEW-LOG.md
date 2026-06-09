# cursor.default_monitor Runtime Monitor-name Oracle Proof Review Log

## Sprint summary
- Starting commit: `3243fd31d51916b6390a2c4708ddedca10ddfe7c`
- Rows analyzed: 1
- Row targeted: `cursor.default_monitor`
- Enabled this sprint: yes
- Counts before: 341 readable / 340 writable / 1 blocked
- Counts after: 341 readable / 341 writable / 0 blocked
- Why the row was enabled: official Hyprland source proves the setting and runtime monitor-name comparison path; the new non-mutating monitor-name oracle proves current valid names from fixture/mock snapshots, rejects missing/stale/unsafe names, and integrates with the high-risk production gate and temp fixture write path.

## Official source evidence
- Files inspected: `/tmp/Hyprland-v0.55.2-full/src/config/values/ConfigValues.cpp`, `/tmp/Hyprland-v0.55.2-full/src/Compositor.cpp`
- What source proves: `cursor:default_monitor` is an official string setting and Hyprland compares the configured value with runtime monitor names before cursor placement.
- What source does not prove: it does not provide a static finite list of monitor names, so the app must prove the chosen name from a current runtime monitor-name snapshot.

## Runtime oracle design
- Oracle module: `src/monitor_name_oracle.rs`
- Snapshot source: `MonitorNameSnapshotSource`
- Fixture source: `FixtureMonitorNameOracle`
- Optional read-only adapter: `ReadOnlyHyprctlMonitorNameOracle`
- Why this is non-mutating: tests use fixture and mock snapshots only; the adapter models `hyprctl monitors` as read-only parsing input and does not mutate the compositor or config.

## Validation behavior
- Valid names: accepted only when present in the current oracle snapshot.
- Missing names: rejected.
- Stale names: rejected after snapshot refresh if no longer present.
- Unsafe names: rejected for newline, command-like, path-like, and other unsafe syntax.
- Empty names: rejected.
- Duplicate names: deduplicated deterministically.
- Malformed snapshots: rejected or produce no valid monitor names.

## Gate integration
- Writability mode: gated high-risk writable row.
- High-risk gate requirement: `cursor.default_monitor` requires high-risk production gate proof and cannot use the low-risk path.
- Recovery requirement: persisted recovery plan, backup, rollback, parser reread, and timeout/no-confirmation rollback behavior remain required.
- Confirmation requirement: valid confirmation token proof remains required.
- Rollback requirement: rollback proof remains required before gated temp writes.
- UI warning requirement: high-risk warning and advanced placement remain required.
- Low-risk path bypass prevention: `cursor.default_monitor` is listed as a high-risk cursor/input row and the pending-change validator requires monitor-name oracle proof instead of generic freeform string validation.

## Tests added
- Test file: `tests/cursor_default_monitor_runtime_oracle.rs`
- Test file: `tests/final_341_writable_coverage.rs`
- `fixture_monitor_oracle_accepts_current_monitor_names`: proves current fixture names validate.
- `fixture_monitor_oracle_rejects_missing_stale_and_unsafe_monitor_names`: proves empty, unknown, command-like, and path-like names reject.
- `hyprctl_monitors_fixture_parser_extracts_monitor_names_without_live_query`: proves parser extraction from fixture `hyprctl monitors` output.
- `stale_monitor_names_reject_after_snapshot_refresh`: proves names from an old snapshot reject after refresh.
- `production_gate_requires_monitor_name_oracle_proof`: proves missing oracle proof rejects and complete proof passes gate evaluation.
- `valid_oracle_and_gate_proof_accepts_temp_fixture_write`: proves valid oracle plus high-risk gate proof writes and rereads only a temp fixture.
- `final_341_writable_coverage` tests: prove final reports and Rust safe-writable table agree on 341 writable rows and 0 blocked rows.

## Remaining blocker
- None for non-live, gated persistent-config write proof. Live runtime mutation proof was not run and remains outside this sprint by boundary.

## Projected next 3 steps
1. If enabled, perform final all-341 consistency review across UI, reports, and write path.
2. If still blocked, complete the exact missing runtime-oracle proof and rerun this sprint.
3. Prepare final migration/handoff summary for the now-complete 341-row writable model.
