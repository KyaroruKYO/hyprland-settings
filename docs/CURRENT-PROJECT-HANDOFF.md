# Current Project Handoff

## Current Focus

Structured-family controlled real-write implementation on `structured-family-editors-unified`.

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

- Active real config touched: false.
- Runtime mutated: false.
- `hyprctl reload` run: false.
- Production behavior enabled: false.
- Structured-family controlled-target writes enabled: true (test-owned fixture, copied config tree, and temp targets only).
- Structured-family active-config writes enabled: false.
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

## Next Exact Work

Stop for explicit user decision: approve or reject a first active real config write pilot for structured families.

For runtime preview: design per-row supervised live-proof runs for the 63 needs-live-proof input/cursor rows.

Unblocking the pilot requires either setting `misc:disable_autoreload = true` (a user decision) or explicitly approving the compositor reload the pilot write would trigger; then run the ignored pilot test with `HYPRLAND_SETTINGS_RUN_ACTIVE_PILOT=1` and confirmed autoreload evidence.
