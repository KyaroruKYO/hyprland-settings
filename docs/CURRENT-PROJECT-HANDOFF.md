# Current Project Handoff

## Current Focus

Save persistence and migration marathon complete on `structured-family-editors-unified`: every Save path is gated on Safe Live Save Mode, structured-family gated persistence is live-proven for the two proven records, and the Hyprland 0.55.4 migration audit found zero drift.

## Completed This Sprint

- Added shared structured-family projections for `hl.monitor`, `hl.bind`, `hl.animation`, `hl.curve`, `hl.gesture`, `hl.device`, and `hl.permission`.
- Added review-only Config page cards for all seven families.
- Added fixture parse and fixture render/reread proof for all seven families.
- Added family-specific validators for all seven families.
- Added temp-fixture write plans with path guards for all seven families.
- Added temp-fixture render/reread proof through write plans for all seven families.
- Added review-only per-record editor form projections for all seven families.
- Added disabled per-record editor UI sections with stable family widgets.
- Surfaced raw fallback status for unsupported or not-proven records.
- Added review-only in-memory record draft models for all seven families.
- Added model-only dirty state tracking and reset proof for all seven families.
- Added draft persistence forbidden policy for all seven families.
- Added disabled record draft UI sections with stable family widgets.
- Added disabled live GTK draft-field binding projections for all seven families.
- Added memory-only draft-field binding update proof for all seven families.
- Kept GTK draft-field binding actions disabled and persistence forbidden.
- Added fixture-only draft-to-rendered-record planning for all seven families.
- Added in-memory rendered-record preview and field-map proof for all seven families.
- Added rendered-record real config target forbidden policy for all seven families.
- Added fixture-only draft rendered-record render/reread proof for all seven families.
- Reread rendered-record temp fixture text through the parser/projection path for all seven families.
- Added fixture-only rendered-record diff/review summaries for all seven families.
- Added in-memory changed/noop review entries and field-diff proof for all seven families.
- Added fixture-only rendered-record approval/confirmation models for all seven families.
- Added in-memory accepted, rejected, and invalidated confirmation proof for all seven families.
- Added fixture-only rendered-record staged apply plans for all seven families.
- Added in-memory ordered apply stages and operations for all seven families.
- Added blocked staged apply plan proof for rejected, invalid, and unsafe confirmations.
- Added fixture-only staged apply dry-run reports for all seven families.
- Added in-memory dry-run summaries for ready, rejected, invalid, and unsafe staged apply plans.
- Added fixture-only staged apply rollback/recovery reviews for all seven families.
- Added in-memory recovery-readiness summaries for ready and blocked dry-run reports.
- Added fixture-only final executor-readiness audits for all seven families.
- Added proof-chain completion, production activation required, executor-not-implemented, executor-not-wired, and not-production-ready findings.
- Added a requirements-only real-write activation audit listing universal activation requirements, missing backup/restore proof, and required user approval gates.
- Explicitly excluded family ranking, safest-family recommendations, family-block recommendations, and activation subset recommendations by user instruction.
- Recorded Option B as production activation planning scope only.
- Kept implementation scope approved false, real write scope approved false, activation subset selected false, and production readiness decision not production ready.
- Added the planning-only structured-family production activation design document.
- Defined future architecture, executor boundary, config target policy, backup/restore, rollback/recovery, validation sequence, manual confirmation, audit logging, and emergency stop requirements without implementing or wiring an executor.
- Classified structured-family editors/writes as blocked by design complete pending explicit executor architecture decision.
- Added the planning-only internal safe-write architecture plan before GUI real-write controls.
- Defined internal safe-write architecture boundaries, future pipeline stages, boundary objects, executor boundary, validation gates, backup/restore gates, rollback/recovery gates, audit log requirements, emergency stop conditions, and UI reachability boundaries without implementing an executor or designing GUI real-write controls.
- Kept GUI real-write controls approved false and GUI real-write controls enabled false.
- Classified structured-family editors/writes as blocked by safe-write architecture plan pending explicit executor implementation planning decision.
- Added the planning-only structured-family executor architecture implementation plan.
- Defined future module, type, function, interface, input, output, validation, backup/restore, rollback/recovery, audit, test, source guard, and UI reachability plans without creating an executable executor module.
- Kept actual executor implementation approved false, executor implementation approved false, executor wiring approved false, real write scope approved false, GUI real-write controls approved false, and production readiness not production ready.
- Classified structured-family editors/writes as blocked by executor architecture implementation plan pending explicit actual executor scaffold decision.
- Added the inert structured-family executor implementation scaffold.
- Added scaffold module, types, functions, default rejection reasons, non-mutating execution receipt, audit record, and emergency stop model.
- Proved the scaffold rejects by default, remains unreachable from current UI, and remains disconnected from `write_flow` and `apply_setting_change`.
- Kept executor wiring approved false, executor wired false, real write scope approved false, real write path enabled false, GUI real-write controls enabled false, backup creation false, restore execution false, rollback execution false, reload false, runtime mutation false, first real config write false, and production readiness not production ready.
- Classified structured-family editors/writes as blocked by executor scaffold pending explicit executor wiring planning decision.
- Recorded executor wiring planning approval as planning-only readiness work.
- Added the inert executor wiring-readiness module with boundary, candidate, preflight, approval-state, source-guard, and readiness-report models that compile and reject by default.
- Defined eight universal wiring boundaries covering the executor scaffold, `write_flow`, `apply_setting_change`, UI reachability, filesystem, backup/restore, rollback/recovery, and reload/runtime; none is family-specific.
- Recorded boundary-level wiring candidates as unwired, unapproved, and not family-specific.
- Added source-level regression guards proving the wiring-readiness layer has no executor call, no `write_flow` or `apply_setting_change` reachability, no UI reachability, no filesystem mutation, no command runner, and no approval flag flips.
- Re-proved the executor scaffold remains unreachable from `main`, `write_flow`, and UI sources.
- Kept executor wiring approved false, executor wired false, and production readiness not production ready.
- Classified structured-family editors/writes as blocked by executor wiring readiness plan pending explicit actual executor wiring scaffold decision.
- Implemented the controlled write-target policy (`src/structured_family_write_target.rs`) distinguishing test-owned fixture, copied config tree, temporary config, active real config, and unknown targets; only the first three are writable, and active-config paths are reclassified and rejected regardless of declared kind.
- Implemented and internally wired the controlled write executor (`src/structured_family_controlled_write.rs`): approval verification, target policy enforcement, staged-apply safety gating, byte-exact backup, family-record write that preserves non-family lines, parser/projection reread verification, automatic restoration on verification failure, rollback with byte-exact and reread verification, write receipts, and audit records.
- Proved full write/backup/reread/verify/restore/verify round trips for all seven families against copied temp targets, with real file writes confined to temporary test directories.
- Proved fail-closed behavior for missing approval, forbidden active-config approval, missing backup/restore/verification plans, out-of-root backup paths, unsafe staged apply plans, tampered linkage, empty rendered records, unknown targets, path escapes, symlink escapes, and disallowed roots.
- Proved the controlled executor is unreachable from live UI, `main`, and the scalar write flow, and contains no reload, command-runner, runtime-mutation, or GTK control paths.
- Kept active real config writes, GUI live Apply controls, hyprctl reload, runtime mutation, and first real config write unapproved and disabled.
- Classified structured-family editors/writes as blocked by active real config write approval.
- Added a project-area continuation scan.

## Safety Boundaries

- Active real config touched: false during normal builds/tests; env-gated live proofs (pilot, save flow proof) wrote it under their gates and the flow-proof cleanup restored the pre-test bytes exactly.
- Runtime mutated: false during normal builds/tests; live proofs are env-gated and reverting.
- `hyprctl reload` run: false — never, anywhere.
- Production behavior enabled: false for source/include and duplicate phases.
- Structured-family controlled-target writes enabled: true (test-owned fixture, copied config tree, and temp targets only).
- Structured-family active-config writes enabled: true, ONLY via `gated_family_save` for the two proven records (hl.animation global speed, hl.curve default Y0) behind receipt + Safe Live Save Mode + identity gates with backup and reread verification; the controlled write executor still rejects the active config by construction; no record creation or deletion exists.
- Executor implemented: true (controlled write executor plus the earlier inert scaffold).
- Executor wired for controlled targets: true.
- Executor wired for active config: false.
- GUI live Apply controls enabled: false.
- GUI real-write controls enabled: false.
- Backup creation enabled: false.
- Restore execution enabled: false.
- Rollback execution enabled: false.
- First real config write approved: false.
- Source/include and duplicate production activation remain capped and separate-phase only.

## Finish-App Sprint Additions

- Removed tool `Co-Authored-By` trailers from the two affected commits via an in-place history rewrite preserving author/committer identity and dates.
- Hardened the controlled executor: a target file symlinked outside the controlled root is now rejected (`SymlinkEscapeRejected`), and every target write/restore is atomic via temp-file-plus-rename.
- Implemented the gated active-config pilot (`src/structured_family_active_config_pilot.rs`): fifteen-gate preflight, typed confirmation phrase, rehearsal-freshness drift detection, one atomic write, mandatory restoration, content-hash receipts and audit records; unreachable from UI/`main`/`write_flow` and executable only via an ignored env-gated test.
- Proved the copied active-config rehearsal against the real config content; the real file was never modified.
- Live pilot blocked by the `AutoreloadDisabledConfirmed`/`NoRuntimeMutationPlanned` gates: `misc:disable_autoreload` is `false`, so a config write would live-reload the compositor. See `docs/STRUCTURED-FAMILY-ACTIVE-CONFIG-WRITE-PILOT.md`.
- Added a review-only GUI "Structured-family write status" card with a permanently insensitive Apply control; proven via the GTK evidence matrix (evidence root `/tmp/hyprland-settings-gtk-automation/20260711_224219`).

## Runtime Preview Sprint Additions

- Implemented the broad runtime preview capability model (`src/runtime_preview.rs`): all 341 scalar rows and 7 structured families classified — 135 live-previewable by default (62 direct, 73 throttled), 78 dead-man gated, 43 config-write-only, 74 blocked high-risk, 11 not proven.
- Implemented the reversible runtime preview executor (`src/runtime_preview_executor.rs`): capability-gated command construction over the proven `hl.config` runtime path, pluggable runner, read-only original-value capture, throttled applies, revert, Save-defers-to-config-write, and a dead-man countdown model.
- Proved a live `general.gaps_in` apply-and-revert round trip against the running compositor (env-gated test, run once); the live proof caught and drove fixes for the `css_gap` table grammar and hyprctl's exit-0 error reporting.
- Added the Config page "Live runtime preview readiness" card with real matrix counts (GTK evidence root `/tmp/hyprland-settings-gtk-automation/20260711_233413`).
- Preview never writes config files, never reloads Hyprland, and rejects unsupported/blocked/not-proven rows before building any command.

## Runtime Preview UI Controls Additions

- Wired per-setting live preview controls into the settings detail pane for all 135 default-previewable rows: 53 switches, 45 sliders, 3 spin rows, 22 color entries, 3 value entries, 9 dropdowns; every other row shows its capability badge and reason with no enabled control.
- Added the UI projection/controller layer (`src/runtime_preview_ui_projection.rs`): the GTK layer renders projections and calls controller actions only — no `hyprctl` strings, executor calls, or file-write APIs in UI code (test-enforced).
- Previewing Live status, gated Save (persists once through the existing safe apply flow when its review gates are open), Revert, and Cancel per row; throttled slider applies with a single trailing drain timer; session-drop and app-close recovery revert any active preview.
- Live UI controller smoke proven against the running compositor (throttled drag on `general.gaps_in`, cancel, restoration verified); GTK evidence at `/tmp/hyprland-settings-gtk-automation/20260712_001609` proves supported and blocked surfaces with all safety flags false.

## Dead-Man Preview UI Additions

- Reclassified all 78 dead-man rows honestly: 2 animation candidates (armed), 63 input/cursor rows needing per-row live proof (panel disarmed with reason), 5 model-only, 8 blocked by grammar; zero rows forced into relog/restart/no-effect buckets without proof. Monitor/display rows remain outside the dead-man set entirely.
- Implemented the supervised controller (`src/runtime_preview_dead_man.rs`): arm captures the original read-only, apply starts a 10-second countdown, Keep stops it, Revert now/Cancel restore immediately, timeout auto-reverts, session-drop and app-close revert unconfirmed previews (Kept previews stay).
- Wired the dead-man panel into the detail pane with live countdown status, warnings, and the recovery instruction; added the Animations dashboard card.
- Live proof (run once): `animations.enabled` toggled under supervision, countdown expired unconfirmed, automatic revert restored the original.
- GTK evidence at `/tmp/hyprland-settings-gtk-automation/20260712_005942` proves the armed panel, blocked reasons, and unchanged supported-row controls with all safety flags false.

## Input/Cursor Proof Architecture Additions

- All 63 needs-live-proof rows now have specific proof plans (`src/runtime_preview_input_proof.rs`, exported to `runtime-preview-input-cursor-proof-plan.v0.55.2.json`): subsystem risk (15 cursor, 11 pointer, 18 touch-family, 7 focus, 6 keyboard), fallback requirements, minimal preview values, apply/revert/verification strategies, manual warnings, and recovery instructions. Four cursor-rendering rows are blocked (no runtime verification) and `cursor.invisible` is blocked as too dangerous.
- The env-gated harness (`tests/runtime_preview_input_live_proof.rs`) runs one row at a time via `HYPRLAND_SETTINGS_RUN_INPUT_LIVE_PROOF=1 HYPRLAND_SETTINGS_INPUT_PROOF_ROW=<row>` and fails closed on every missing precondition.
- One proof ran and passed: `cursor.inactive_timeout` (original 0.000000 → preview 1 verified live → reverted and verified). The receipt is recorded in `PROVEN_INPUT_ROWS`, which is the only promotion mechanism — the dead-man layer arms input/cursor rows solely on a recorded receipt (test-enforced). Armed candidates: 3 (two animation toggles + the proven cursor row); 62 rows remain disarmed.
- Disarmed rows show proof-aware status (risk, fallback, proof classification, env command); GTK evidence at `/tmp/hyprland-settings-gtk-automation/20260712_013141` with all safety flags false.

## Proof Marathon Additions

- Ran 71 env-gated live proof executions across two batches; 36 input/cursor rows are now proven and armed (receipts in `PROVEN_INPUT_ROWS`), covering every row provable on this machine.
- Live proofs exposed and fixed a real executor defect: int-typed FiniteChoice options rejected quoted strings; choice values are now validated against each row's allowed list and numeric choices render bare.
- `input.scroll_method` blocked with live evidence: unset str options read back as `[[EMPTY]]` and cannot be round-tripped.
- The harness gained batch mode (`HYPRLAND_SETTINGS_INPUT_PROOF_ROW=all`); the active-config pilot gained a read-only autoreload evidence collector that fails closed.
- GTK evidence proves an armed proof-passed input row with its provenance line; all safety flags false.

## Family Completion Marathon Additions

- Probed all seven family runtime record APIs mutation-free and classified every family from evidence (`src/structured_family_runtime_preview.rs`).
- Ran and passed two zero-residue live proofs: `hl.animation` (global node speed 8.00 → 8.5 → 8.00) and `hl.curve` (default bezier y0 0.75 → 0.76 → 0.75), both readback-verified exact restores; receipts recorded, promotion receipt-gated and scoped to modify-existing.
- Hardened the active-config pilot gate with live autoreload evidence collection; added the readiness report with exact unblock instructions.
- Added the Config page structured-family live preview & persistence card (AT-SPI-proven with all safety flags false).

## Safe Live Save Mode Sprint Additions

- Proved `misc:disable_autoreload` is runtime-controllable (no file write, no reload, getoption-verified, exactly reversible) and built Safe Live Save Mode on it: status/enable/disable with fixed constant commands, readback verification, and a Config page card with working Enable/Disable buttons.
- The first active-config write pilot PASSED through its fifteen-gate path using the runtime-first strategy: wrote one inert record to the real `hyprland.conf`, verified, restored byte-exactly (SHA recorded); the runtime-flag marker proved no reload fired.
- Shipped supervised structured-family preview UI controls for the two proven records (global animation speed, default curve Y0), modify-existing only.

## Save Persistence Migration Marathon Additions

- Gated every Save path on Safe Live Save Mode: both scalar UI save paths route through `gated_scalar_save_live` (live gate check, then the existing backup/write/reread apply flow); direct `apply_setting_change` is eliminated from UI code and guard-tested. See `docs/PRODUCTION-SAVE-INTEGRATION.md`.
- Implemented structured-family gated persistence (`src/structured_family_gated_persistence.rs`): value validation → proven-record receipt gate → live Safe Live Save Mode gate → active-config identity gate → byte-exact backup → one atomic write → reread verification; restore only on verification failure, never after success. Save buttons shipped in both family preview controls. See `docs/STRUCTURED-FAMILY-GATED-PERSISTENCE.md`.
- Ran the env-gated live save flow proof: the gate blocked correctly while autoreload was active; both targets then wrote real config records, reread-verified, with no restore by production code; the flow-proof cleanup restored the pre-test bytes exactly and the original autoreload state.
- Completed the Hyprland 0.55.4 migration audit with a trusted source (`hyprctl -j descriptions` from the official 0.55.4 binary, captured to `data/exports/hyprland-0.55.4/`): zero option drift (341 = 341, 0 added, 0 removed), zero numeric-bounds drift (78 compared), 47 cosmetic description diffs; a regression test pins compatibility every `cargo test` run. See `docs/HYPRLAND-0.55.4-MIGRATION-AUDIT.md`.

## Completion Marathon Additions (record picker, safe mode persistence, refresh, release decision)

- Family record picker (`src/family_record_picker.rs`): lists the records that exist in the runtime readback with honest per-record classification, previews picked records live under the recovery countdown, and persists them through `gated_family_record_save` (the same gate chain, now shape-receipt-gated). Record-shape proofs passed live on non-family-proof records (hl.animation `fade` speed; hl.curve `quick` control points; zero residue), and the live save flow proof persisted both shapes to the real config with byte-exact flow-proof restores. A live proof found disabled-at-runtime records cannot be preview-verified — they are save-only; styled records are save-only with the style preserved; inherited and internal records stay blocked (creation is blocked). See `docs/FAMILY-RECORD-PICKER-GATED-PERSISTENCE.md`.
- Safe Live Save Mode persistence (`src/persist_safe_live_save_mode.rs`): `misc:disable_autoreload = true` can be saved as the default through the gated scalar Save — user-chosen via the Save as default control (enabled only while the runtime mode is active), never automatic; the module can express no other setting or value. Live flow proof passed with byte-exact restore. See `docs/PERSIST-SAFE-LIVE-SAVE-MODE.md`.
- 0.55.4 export refresh workflow (`tools/refresh_hyprland_descriptions_export.sh`): read-only re-capture + diff + pinned-test rerun; executed live with zero drift in every category; a different live version gets its own capture without touching the pinned one. See `docs/HYPRLAND-0.55.4-EXPORT-REFRESH.md`.
- Release decision: ready pending user approval — RC materials drafted (release notes, changelog, checklists, manual test plan); no tag, no merge, no artifacts. See `docs/RELEASE-DECISION.md`.

## Next Exact Work

The completion marathon areas are done. Remaining:

- Hardware-gated proofs: 18 touch-family rows need touch/tablet hardware; 3 rows need secondary-device proofs (deferred; devices unavailable — no simulated/virtual-device path is proven, and any future one is proposal-only requiring explicit approval).
- Release: the decision is ready pending user approval (merge, version bump, tag v0.2.0, dist artifacts — all user-gated).
- Further record shapes (style/enabled fields, other families) require new live proofs before any support.
