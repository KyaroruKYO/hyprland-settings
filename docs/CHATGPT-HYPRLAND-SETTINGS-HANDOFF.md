# Hyprland Settings ChatGPT Handoff

This file is the starting context for a new ChatGPT conversation. It is written for ChatGPT and the user, not for Codex internals. Assume the new conversation has no other history.

State reviewed after the all blocked rows autonomous writability completion sprint on branch `completion-sprint`.

Latest reviewed implementation baseline before this sprint:

- `f304481 Audit all blocked rows official evidence`

Latest sprint commit:

- `Complete autonomous blocked row writability pass` (this commit; use `git log -1 --oneline` for the exact hash)

Latest restore point and backups:

- Restore tag: `pre-all-blocked-autonomous-writability-completion-20260608-160533`
- Project backup: `/home/kyo/Documents/hyprland-settings-pre-all-blocked-autonomous-writability-completion-backup_20260608_160533/`
- AGS backup: `/home/kyo/Documents/ags-pre-all-blocked-autonomous-writability-completion-backup_20260608_160533`
- Hypr config backup: `/home/kyo/hyprland-config-backups/hypr-pre-all-blocked-autonomous-writability-completion-20260608_160533`
- Handoff backup: `/home/kyo/Documents/system-audit/next-agent-handoff-pre-all-blocked-autonomous-writability-completion-backup_20260608_160533`

Latest autonomous blocked-row status:

- Rows processed: 63.
- Rows enabled this sprint: 0.
- Rows still blocked: 63.
- Writable rows before/after: 278 / 278.
- Blocked rows before/after: 63 / 63.
- Final counts: 341 readable / 278 writable / 63 blocked.
- HyprMod usage status: not used.
- Main blocker categories: missing invalid-value behavior fixtures, validator proof, fixture write/reread proof, high-risk safety gates, UI warning proof, row-specific tests, live-runtime-equivalent proof, and future explicit approval. `cursor.default_monitor` also still needs dynamic monitor-name allowed-value proof.
- Review log path: `docs/ALL-BLOCKED-ROWS-AUTONOMOUS-WRITABILITY-REVIEW-LOG.md` and `/home/kyo/.config/hypr/docs/ALL-BLOCKED-ROWS-AUTONOMOUS-WRITABILITY-REVIEW-LOG.md`.
- Error/future-research log path: `data/reports/all-blocked-rows-autonomous-error-and-future-research-log.v0.55.2.json`.
- Completion report: `data/reports/all-blocked-rows-autonomous-writability-completion.v0.55.2.json`.
- Summary report: `data/reports/all-blocked-rows-autonomous-writability-summary.v0.55.2.json`.
- Screen-shader closure status: closed for now.
- `SAFE_WRITABLE_ROWS` changed: no.
- Write allowlist changed: no.
- Real config/runtime touched: no.
- Reload/eval/Lua/live display/input/crash proof used: no.
- Validation results: `cargo fmt`, `cargo fmt --check`, `cargo check`, `cargo test`, `cargo build --release`, `desktop-file-validate`, export validator, UI design validator, and schema validator passed. `appstreamcli validate --pedantic` produced only the expected non-blocking GitHub URL and releases-info warnings.
- Next recommended sprint: `Blocked high-risk rows safety-gate and invalid-value proof design sprint`.
- Hard boundaries preserved: yes. No push.

## 1. Primary Project Goal

Hyprland Settings is a Rust + GTK4/libadwaita native settings app for Hyprland.

The app replaces the older AGS prototype as the final app direction. The AGS work remains useful as a source/spec/export path, but AGS is not the final app runtime.

The goal is to expose Hyprland configuration settings in a GUI while preserving correctness and safety. The app should stay focused on Hyprland configuration settings. It should not grow into unrelated desktop management.

Primary sources of truth:

- Official Hyprland docs and source.
- Official Hyprland website and GitHub when current source is needed.
- Exported project reports under `data/reports/`.
- Validated local proof artifacts.
- Existing tests that enforce report and safety invariants.

Safety and correctness are more important than speed. Do not guess missing behavior. If something is unknown, record exactly what is unknown and where to inspect next.

App identity:

- GitHub owner: `KyaroruKYO`
- Repository: `hyprland-settings`
- Binary: `hyprland-settings`
- App ID: `io.github.kyarorukyo.hyprlandsettings`

## 2. User/Workflow Expectations

The user is not a coder and relies on ChatGPT to create clear Codex prompts.

Roles:

- ChatGPT is the architect, reviewer, and prompt writer.
- Codex is the executor, verifier, and reporter.
- The user is the final approver.

ChatGPT should not guess. It should write prompts that tell Codex which files and reports to inspect, which counts to verify, what safety boundaries must remain intact, what outputs to create, and when to stop.

The preferred workflow is larger risk-class or pipeline sprints. Avoid tiny one-row crawls unless a row is uniquely dangerous or needs isolated proof, such as `decoration.screen_shader`.

Every sprint should preserve safety boundaries and end with clear final counts:

- readable rows
- writable rows
- blocked rows
- rows enabled
- write allowlist changed or unchanged
- real config/runtime touched or untouched
- validation results

## 3. Current Verified State

Current branch: `completion-sprint`

Latest reviewed implementation baseline before the display/render blocked rows readiness batching sprint:

- `761abf2 Plan next high-risk bucket readiness batch`

Latest sprint commit message:

- `Plan display render blocked row readiness`

Current scalar row counts:

- Readable scalar rows: 341 / 341
- Writable scalar rows: 278 / 341
- Blocked scalar rows: 63 / 341

Current high-risk state from `data/reports/high-risk-unified-pipeline-reconciliation.v0.55.2.json`:

- Enabled high-risk rows audited: 9
- Rows failing unified pipeline conformance: 0
- Rows missing proof metadata: 0
- Rows missing watchdog metadata: 0
- Enabled rows still incorrectly listed as high-risk candidates: 0
- Remaining display/render blocked rows: 23
- Remaining cursor/input blocked rows: 18
- Remaining debug/crash blocked rows: 22

Screen shader state:

- `decoration.screen_shader` is currently writable.
- It remains a writable migration candidate.
- Selected policy from the review sprint: `Policy D`.
- Selected migration option from the gate design sprint: `Option A`.
- Selected production gate enforcement decision: `Option A`.
- Selected production gate architecture option: `Option C`.
- Selected production gate approval option: `Option C`.
- Selected compile-aware validation research option: `Option C`.
- Selected advisory compiler feasibility option: `Option A`.
- Selected advisory compiler integration design option: `Option A`.
- Selected advisory compiler implementation proof option: `Option A`.
- Selected advisory UI exposure design option: `Option A`.
- Selected advisory UI implementation proof option: `Option A`.
- Selected advisory GTK widget wiring proof option: `Option A`.
- Selected advisory file chooser execution proof option: `Option B`.
- Screen shader closure status: closed for now.
- Reusable high-risk pattern status: extracted in `data/reports/high-risk-row-pattern-from-screen-shader.v0.55.2.json`.
- Return-to-341 roadmap status: created in `data/reports/return-to-341-writable-roadmap.v0.55.2.json`.
- Grouped batching report status: created in `data/reports/next-high-risk-bucket-readiness-batching.v0.55.2.json`.
- Display/render readiness report status: created in `data/reports/display-render-blocked-rows-readiness-batching.v0.55.2.json`.
- Selected next bucket: `display-render-bucket-readiness`.
- Candidate rows count: 23 blocked display/render rows.
- Selected candidate batch: `unresolved-display-render-inventory`.
- Next recommended grouped sprint: `Display/render blocked rows source evidence inventory sprint`.
- Dry-run/non-production gate primitive added: yes.
- Primitive name: `screen-shader-dry-run-gated-write-review`.
- Ungated dry-run `decoration.screen_shader` rejected: yes.
- Gated dry-run `decoration.screen_shader` accepted with valid fixture watchdog proof: yes.
- Production apply-flow gate wired: yes.
- Ungated production-flow `decoration.screen_shader` rejected in fixture/temp tests: yes.
- Gated production-flow `decoration.screen_shader` accepted with valid fixture watchdog proof: yes.
- Invalid or mismatched production gate proof rejected: yes.
- Unrelated normal writable rows require screen-shader gate: no.
- Watchdog migration proof status: complete.
- It is not counted as a completed enabled high-risk row.
- Production enforcement changed: yes, only for `decoration.screen_shader`.
- Production gate enforced this sprint: yes, only for `decoration.screen_shader`.
- Production write flow changed: yes, only for `decoration.screen_shader`.
- Normal production review changed: yes, only for `decoration.screen_shader`.
- Normal path-only approval still accepted in production: no for `decoration.screen_shader`; yes for unrelated normal writable rows.
- Compile-aware validation remains deferred.
- Compile-aware validation changed: no.
- Compile-aware validation implemented: no.
- Shader compilation run: no.
- Non-live validation designable: yes, for advisory/research-only fixture/temp checks.
- Standalone compiler commands run: yes, against generated fixture/temp shaders only.
- Chosen advisory compiler research tool: `glslangValidator`.
- Fixture good shader accepted: yes.
- Fixture invalid shader rejected: yes.
- Fixture uses Hyprland vertex pairing: yes.
- Real user shader files read: no.
- Writes outside temp dir: no.
- Advisory compiler integration implemented: no.
- Non-production advisory helper implemented: yes.
- Advisory helper module: `src/screen_shader_advisory.rs`.
- Compiler checks wired into validators: no.
- Compiler checks wired into pending changes: no.
- Compiler checks wired into write planning: no.
- Compiler checks wired into apply flow: no.
- Standalone compiler commands run in the integration design sprint: no.
- Standalone compiler commands run in the implementation proof sprint: yes, against generated fixture/temp shaders only.
- User consent required before shader read by design: yes.
- Explicit user consent required by helper before shader read: yes.
- Background shader scanning allowed: no.
- Original user shader path passed to compiler by design/helper: no.
- Temp copy required by design/helper: yes.
- Missing-tool behavior proven: yes.
- Timeout behavior proven: yes.
- Advisory pass behavior proven: yes.
- Advisory fail behavior proven: yes.
- Cleanup failure behavior proven: yes.
- Advisory UI exposure implemented: no.
- UI exposure design only: yes.
- Non-production advisory UI action/model implemented: yes.
- UI action module/model: `src/ui/model.rs::run_screen_shader_advisory_ui_action`.
- Visible GTK widget/control implemented: yes.
- GTK widget/control module: `src/ui/window.rs::append_screen_shader_advisory_controls`.
- GTK widget placement: advanced/high-risk section separated from write/apply controls.
- Full GTK file chooser execution implemented: no.
- File chooser behavior proven: no.
- Selected-file action model implemented: yes.
- Selected-file action module: `src/ui/model.rs::run_screen_shader_advisory_selected_file_ui_action`.
- Generated fixture/temp selected files used in tests: yes.
- Real user shader files read in tests: no.
- Selected-file boundary proven through action model: yes.
- Selected-file boundary proven through visible GTK file chooser: no.
- Original selected path passed to compiler: no.
- Compiler receives only temp paths: yes.
- Arbitrary config path reads allowed: no.
- Missing selection behavior proven: yes.
- Cancellation/progress behavior proven through visible GTK control: no.
- Advanced/high-risk placement required: yes.
- Explicit user trigger required: yes.
- Separated from apply/write action: yes.
- Result states modeled/rendered: `not_run`, `running`, `passed`, `failed`, `unavailable`, `timed_out`, `temp_copy_failed`, `cleanup_warning`.
- Missing consent behavior proven: yes.
- Advisory pass/fail/unavailable/timeout/temp-copy/cleanup-warning rendering proven: yes.
- Runs on row load: no.
- Runs on value change: no.
- Runs during validation/pending/write planning/apply flow: no.
- Advisory result can approve writes: no.
- Advisory result can block writes: no.
- Advisory result can bypass production gate: no.
- Recommended compile-aware policy: advisory or research-only until compatibility with Hyprland's OpenGL runtime path is proven.
- Screen-shader-specific track closed for now: yes.
- Remaining shader-specific work is deferred by default: direct GTK file chooser visual proof, visible selected-file boundary proof, cancellation/progress behavior through visible GTK UI, Hyprland OpenGL runtime compile/link equivalence, and production compile-aware validation.
- Future screen-shader work should occur only if explicitly approved or if a proven safety failure appears.
- Reusable high-risk row pattern extracted: yes; it is a decision framework, not automatic permission to enable rows.
- Return-to-341 roadmap created: yes; next work should return to grouped high-risk/bucket-level planning.
- No row was enabled during the screen shader closure and high-risk pattern extraction sprint.
- Write allowlist unchanged.
- Recovery gates unchanged.
- Real config untouched.
- Runtime untouched.
- Reload/eval/Lua unused.
- Live shader compile unused.
- Live display/render proof unused.

Decision report:

- `data/reports/screen-shader-production-gate-enforcement-decision.v0.55.2.json`
- `data/reports/screen-shader-production-gate-architecture.v0.55.2.json`
- `data/reports/screen-shader-production-gate-approval.v0.55.2.json`
- `data/reports/screen-shader-compile-aware-validation-research.v0.55.2.json`
- `data/reports/screen-shader-non-live-advisory-compiler-feasibility.v0.55.2.json`
- `data/reports/screen-shader-advisory-compiler-integration-design.v0.55.2.json`
- `data/reports/screen-shader-advisory-compiler-implementation-proof.v0.55.2.json`
- `data/reports/screen-shader-advisory-ui-exposure-design.v0.55.2.json`
- `data/reports/screen-shader-advisory-ui-implementation-proof.v0.55.2.json`
- `data/reports/screen-shader-advisory-gtk-widget-wiring-proof.v0.55.2.json`
- `data/reports/screen-shader-advisory-file-chooser-execution-proof.v0.55.2.json`
- `data/reports/screen-shader-track-closure.v0.55.2.json`
- `data/reports/high-risk-row-pattern-from-screen-shader.v0.55.2.json`
- `data/reports/return-to-341-writable-roadmap.v0.55.2.json`

Decision summary:

- Option A was selected because current Rust source proves the write path still uses normal `review_write_plan` approval and does not route `decoration.screen_shader` through an enforced production watchdog gate.
- The dry-run/temp-only watchdog migration proof is complete, but production enforcement is a separate missing primitive.
- Live/production watchdog execution remains planned-disabled in `src/high_risk_recovery.rs`.
- Option C was selected in the architecture sprint because a dry-run/non-production gated-review primitive could be represented safely without changing production apply behavior.
- The primitive proves the shape needed for later enforcement: ungated screen-shader fixture review is rejected, gated fixture review is accepted, and unrelated rows are not gated.
- Option C was selected in the approval sprint because the existing primitive could be wired into the production apply-flow decision point and proven with fixture/temp tests only.
- The production apply flow now rejects ungated `decoration.screen_shader` before final apply, accepts a valid gated fixture/temp screen-shader write, rejects invalid/mismatched gate proof, and leaves unrelated writable rows on the normal path.
- This is production gate enforcement only. It does not add shader compilation, compile-aware validation, live display/render proof, reload behavior, active runtime mutation, or real config writes in tests.
- Option C was selected in the compile-aware validation research sprint because a standalone non-live check can be researched as advisory syntax/link-shape validation, but cannot prove exact Hyprland runtime OpenGL compile/link or display/render safety.
- Official source shows Hyprland passes the raw user fragment shader to `CShader::createProgram` and compiles/links through OpenGL with either `tex300.vert` or `tex320.vert`. No official non-live screen-shader validation interface was found.
- Compile-aware validation remains deferred and was not implemented.
- Option A was selected in the advisory compiler feasibility sprint because `glslangValidator -l` accepted generated known-good ES 300/320 fixture shader pairs with official Hyprland vertex shaders and rejected intentionally invalid fixture fragments.
- `glslc` was checked but not chosen because the tested SPIR-V-oriented invocation rejected Hyprland-shaped GLSL ES fixtures for requirements that are not Hyprland's OpenGL runtime path.
- The advisory compiler feasibility proof is fixture/temp-only. It does not read real user shader files, does not write outside `/tmp`, and is not equivalent to Hyprland runtime safety.
- Option A was selected in the advisory compiler integration design sprint because the project can design an optional advanced advisory boundary: explicit user action, copy the selected shader into a temp fixture, copy the source-backed Hyprland vertex shader into the same temp directory, run `glslangValidator -l` only on temp paths, and report non-authoritative results.
- Option A was selected in the advisory compiler implementation proof sprint because a non-production helper could be implemented and fixture-proven while staying disconnected from write safety.
- The helper lives in `src/screen_shader_advisory.rs`. It requires explicit user consent, copies the selected fixture shader into a temp directory, copies the matching source-backed Hyprland vertex shader into that temp directory, runs `glslangValidator -l` only on temp paths, and captures advisory result data.
- The helper proves missing-tool, timeout, advisory pass, advisory fail, temp-copy failure, and cleanup-warning behavior. It does not claim Hyprland runtime safety, does not block or approve writes, and does not bypass the production screen-shader gate.
- Option A was selected in the advisory UI exposure design sprint because the project can represent a row-specific, design-only advanced advisory UI projection for `decoration.screen_shader` without invoking the helper or changing write safety.
- `src/ui/model.rs` now projects screen-shader advisory UI metadata only for `decoration.screen_shader`: advanced placement, explicit trigger, consent/temp-copy/runtime-safety/production-gate messages, result policies, and no approve/block/bypass capability.
- Option A was selected in the advisory UI implementation proof sprint because a non-production UI action model could be implemented and fixture-proven while staying disconnected from write safety.
- `src/ui/model.rs::run_screen_shader_advisory_ui_action` is the UI action model. It models `not_run`, `running`, `passed`, `failed`, `unavailable`, `timed_out`, `temp_copy_failed`, and `cleanup_warning`.
- The UI action model proves missing-consent, pass, fail, unavailable, timeout, temp-copy failure, and cleanup-warning rendering as advisory-only.
- Option A was selected in the advisory GTK widget wiring proof sprint because the visible advanced control could be wired while proving only the missing-selection path and staying disconnected from write safety.
- `src/ui/window.rs::append_screen_shader_advisory_controls` renders the visible advanced advisory section for `decoration.screen_shader`.
- Option B was selected in the advisory file chooser execution proof sprint because direct GTK file chooser interaction could not be safely tested yet, but a fixture-selected file action model could be implemented and proven.
- `src/ui/model.rs::run_screen_shader_advisory_selected_file_ui_action` accepts an explicit selected fixture path, builds the existing advisory helper request, and proves selected-file handling with generated temp shader files only.
- The selected-file action model proves original selected paths are not passed to `glslangValidator`, compiler path arguments are temp paths only, and advisory output remains disconnected from write safety.
- Direct GTK file chooser visual interaction, visible selected-file boundary proof, and cancellation/progress proof are still missing.
- Compiler checks remain unwired from validators, pending changes, write planning, and apply flow.
- The screen-shader closure sprint closed the `decoration.screen_shader` track for now. The row remains writable and production-gated, compile-aware validation remains deferred, and remaining shader-specific gaps are no longer the default next sprint.
- The high-risk row pattern extracted from the screen-shader work says parser acceptance, HyprMod exposure, UI metadata, advisory checks, and standalone compiler output are not safety proof. Each future high-risk row still requires source-backed, row-specific proof.
- The return-to-341 roadmap moves the project back to grouped high-risk/bucket-level planning toward all 341 official scalar rows writable where possible.

Validation state from the most recent sprint:

- `cargo fmt`: passed
- `cargo fmt --check`: passed
- `cargo check`: passed
- `cargo test`: passed
- `cargo build --release`: passed
- `desktop-file-validate`: passed
- `appstreamcli validate --pedantic ... || true`: completed with expected non-blocking metadata warnings
- Python export/UI/schema validators: passed

Worktree state should be clean after committing the screen shader closure and high-risk pattern extraction sprint changes.

## 4. Project Architecture Overview

This is a native Rust app targeting GTK4/libadwaita.

The project uses exported Hyprland 0.55.2 data and a report-driven workflow. The report files are not decorative; they are used as proof artifacts and test inputs.

Important concepts:

- `SAFE_WRITABLE_ROWS`: the Rust source-of-truth table for rows the app is allowed to write.
- Readable rows: scalar Hyprland settings the app can read/project.
- Writable rows: scalar settings with validators and safe write policy.
- Blocked rows: scalar settings intentionally not writable yet.
- Validators: Rust-side value-shape rules that reject invalid or unsafe values before write planning.
- High-risk rows: settings that need stronger proof, warning, watchdog/dead-man recovery, or manual review before becoming writable.
- Watchdog/dead-man model: a recovery model where a write can be automatically restored if confirmation does not happen through an independent path.

Unified row-driven pipeline in plain English:

1. Every setting row has metadata.
2. Writable rows have a value kind and validator.
3. The app proves valid and invalid value behavior.
4. Fixture/temp-config proof is used before trusting writes.
5. Write/rewrite/reread behavior is tested with fixture files.
6. Some rows require an approval gate.
7. High-risk rows require recovery/watchdog proof.
8. Reports and tests must be reconciled after every change.

Do not bypass the unified pipeline with one-off behavior. If a row is special, document the special case in reports and tests.

## 5. Important File Map

Project root:

- `/home/kyo/Projects/hyprland-settings`

Important source files:

- `src/write_classification.rs`
  - Controls write classification, `SAFE_WRITABLE_ROWS`, value kinds, high-risk write policies, and session/runtime policy metadata.
  - Be careful: changing this can enable or disable writes.

- `src/pending_change.rs`
  - Stages proposed values and applies validators before write planning.
  - Be careful: validator changes alter accepted user input.

- `src/write_flow.rs`
  - Builds edit/review projections and applies safe write flow using backups and review gates.
  - Recent screen shader work added a warning projection here.

- `src/high_risk_recovery.rs`
  - Contains watchdog/dead-man recovery primitives and dry-run proof support.
  - Be careful: weakening this can make high-risk writes unsafe.

- `src/value/`
  - Contains value parsers/validators for booleans, numbers, colors, gradients, vectors, paths, regex-like strings, CssGap, accel profiles, scroll points, and related value kinds.

- `src/export.rs`
  - Handles export bundle and report-oriented app data.

- `src/ui/`
  - GTK/libadwaita UI model and window code.
  - Keep UI behavior consistent with write safety metadata.

Important test areas:

- `tests/write_flow.rs`
- `tests/pending_change.rs`
- `tests/high_risk_recovery.rs`
- `tests/high_risk_unified_pipeline_reconciliation.rs`
- `tests/writable_value_type_exhaustiveness_audit.rs`
- `tests/writable_validator_research_reports.rs`
- `tests/source_backed_writable_validator_repair.rs`
- `tests/deferred_source_backed_validator_repair.rs`
- `tests/screen_shader_display_render_review.rs`

Important report directory:

- `data/reports/`
  - Machine-readable proof and status reports.
  - Future agents should inspect reports before changing source.

Important external docs:

- `/home/kyo/.config/hypr/docs/`
  - Human-readable sprint reports and historical docs.

- `/home/kyo/.config/hypr/docs/exports/hyprland-0.55.2/`
  - Exported Hyprland 0.55.2 inventory and related data.

- `/home/kyo/Documents/system-audit/next-agent-handoff/`
  - Handoff files for future conversations.

## 6. Major Work Completed

AGS/prototype/export phase:

- The AGS prototype was used as a source/spec/export path.
- AGS is not the final app runtime.
- Rust migration started from the AGS/exported model.
- Official Hyprland 0.55.2 scalar inventory reached 341 rows.

Rust/native migration:

- Rust project path: `/home/kyo/Projects/hyprland-settings`
- App target: GTK4/libadwaita native app.
- App identity: `io.github.kyarorukyo.hyprlandsettings`.
- Build and tests are passing at the current reviewed state.

Read coverage:

- 341 / 341 scalar rows are readable.

Write coverage:

- 278 / 341 scalar rows are writable.
- 63 / 341 scalar rows remain blocked.
- Proof sprints used fixture/temp files and report generation.
- Real Hyprland config and active runtime were not modified during the recent proof/review sprints unless a report explicitly says otherwise. Current screen shader and validator repair reports say real config and runtime were untouched.

Unified pipeline:

- All 341 scalar rows were backfilled into the unified row-driven pipeline.
- High-risk unified pipeline reconciliation completed.
- Current high-risk reconciliation shows no conformance failures.

Session/runtime-sensitive rows:

- 16 session/runtime-sensitive rows were enabled earlier with persistent-config-only, reload-note, or startup-note policy.
- No reload or active runtime mutation was used for those proof paths.

Enabled high-risk rows:

- Ecosystem/permission bucket:
  - `ecosystem.no_update_news`
  - `ecosystem.no_donation_nag`
  - `ecosystem.enforce_permissions`

- Display/render XWayland scaling subset:
  - `xwayland.use_nearest_neighbor`
  - `xwayland.force_zero_scaling`

- Cursor/input:
  - `cursor.sync_gsettings_theme`
  - `cursor.hide_on_touch`
  - `cursor.hide_on_tablet`
  - `cursor.hide_on_key_press`

High-risk templates currently reconciled:

- `high-risk-policy-watchdog-template`
- `display-render-watchdog-template`
- `cursor-input-theme-sync-watchdog-template`
- `cursor-visibility-conditional-watchdog-template`

Screen shader migration/proof status:

- `decoration.screen_shader` has display/render screen-shader watchdog template metadata.
- Fixture/temp-only watchdog migration proof is complete.
- Production apply-flow gate enforcement is wired for `decoration.screen_shader` only.
- Ungated production-flow fixture writes for `decoration.screen_shader` are rejected before final apply.
- Gated production-flow fixture writes for `decoration.screen_shader` are accepted only with valid watchdog proof.
- Invalid or mismatched screen-shader gate proof is rejected.
- Unrelated writable rows remain on the normal write path.
- Compile-aware shader validation is still deferred.
- A non-production advisory helper exists in `src/screen_shader_advisory.rs`.
- The helper is fixture/temp-only in tests, requires explicit user consent, passes only temp paths to `glslangValidator`, and cannot approve, block, or bypass write safety.
- Design-only advisory UI metadata exists in `src/ui/model.rs` for `decoration.screen_shader` only.
- The UI design projection requires advanced placement, explicit user trigger, consent/temp-copy/runtime-safety/production-gate messaging, no automatic runs, and no approve/block/bypass effect.

Validator/value-type work completed:

- Writable value-type exhaustiveness audit.
- Writable validator research.
- Official source validator research.
- Source-backed validator repair.
- Deferred consumer-source research.
- Deferred source-backed validator repair.
- Screen shader display/render review.

## 7. Current Validator State

Important validator repairs already completed:

- Boolean policy:
  - Exact source-backed aliases are accepted where appropriate.
  - UI should offer only `true` / `false`.
  - Do not broaden booleans to arbitrary integers or prefix aliases.

- Numeric bounds:
  - Source-backed numeric bounds and integer/float distinctions were repaired for many rows.

- CssGap:
  - `appearance.gaps_in`
  - `appearance.gaps_out`
  - Validator supports source-backed 1-4 integer component forms.
  - Negative gaps are conservatively rejected as app policy.

- Accel profile:
  - `input.accel_profile`
  - Supports source-backed default/empty, `adaptive`, `flat`, and valid `custom <step> <point...>` forms.

- Scroll points:
  - `input.scroll_points`
  - Supports finite space-separated doubles in the custom acceleration context.
  - Comma-separated values, NaN, inf, and invalid tokens are rejected.

- Parser-only rows repaired/reclassified:
  - `master.center_master_fallback`
  - `scrolling.explicit_column_widths`

- Color:
  - Repaired against official source-backed grammar.

- Gradient:
  - Repaired for official grammar, maximum color count, and angle handling.

- Vector:
  - Repaired to source-backed two-space-separated-float form.

- String/path/font metadata:
  - Metadata improved for locale, XKB file path, and font-family rows.
  - Validators were not over-tightened without source-backed consumer evidence.

Remaining deferred validator rows:

- `input.kb_file`
  - Optional config-relative XKB keymap path.
  - Existence/readability checks are UI policy, not mandatory writer policy yet.

- `misc.swallow_regex`
  - Official source uses RE2 full-match semantics.
  - RE2-compatible validation is deferred because no low-risk compatible Rust-side implementation is currently integrated.

- `misc.swallow_exception_regex`
  - Same RE2 full-match semantics and deferred validation decision as `misc.swallow_regex`.

- `decoration.screen_shader`
  - It is path-shaped but display/render-sensitive.
  - Fixture/temp watchdog migration proof is complete.
  - It remains a writable migration candidate, not a completed enabled high-risk row.
  - Production apply-flow gate enforcement is wired for `decoration.screen_shader` only.
  - Compile-aware validation research selected `Option C`: advisory or research-only.
  - Compile-aware validation remains deferred and was not implemented.

## 8. Current High-Risk / Safety State

Enabled high-risk rows are protected by proof reports, templates, warning metadata, and watchdog/dead-man design artifacts.

The watchdog/dead-man model exists and has separate-process proof reports:

- `data/reports/separate-watchdog-process-proof.v0.55.2.json`
- `data/reports/production-high-risk-watchdog.v0.55.2.json`

No high-risk row should be enabled without proof. Do not use HyprMod exposure, parser acceptance, or lack of warnings as proof of safety.

Current high-risk blocked buckets:

- Display/render blocked rows: 23
- Cursor/input blocked rows: 18
- Debug/crash blocked rows: 22

Current safety confirmations:

- Real config modified: no
- Active runtime modified: no
- Reload/eval/Lua used: no
- Pushed: no
- Main touched: no

## 9. Screen Shader Stopping Point

This section is important.

`decoration.screen_shader` is currently writable.

It is path-shaped, but it is not ordinary path-only safe. Official Hyprland source shows:

- Empty string disables the shader.
- `[[EMPTY]]` disables the shader.
- Non-empty values are config-relative paths.
- Hyprland reads the file.
- Hyprland compiles the file contents as the final screen fragment shader.

Selected policy: `Policy D`.

Policy D means:

- Keep `decoration.screen_shader` writable.
- Do not silently remove it from `SAFE_WRITABLE_ROWS`.
- Do not treat path validation as display/render safety proof.
- Production apply now requires the screen-shader display/render high-risk gate before final apply.

Compile-aware validation is deferred.

Do not continue into shader compile validation without a dedicated sprint. Do not use live compositor/render validation. Do not reload Hyprland. Do not touch real config.

Current proof update:

- Watchdog migration proof status: complete.
- Proof report: `data/reports/screen-shader-watchdog-migration-proof.v0.55.2.json`
- Test proof: `tests/screen_shader_watchdog_migration_proof.rs`
- Human doc: `/home/kyo/.config/hypr/docs/SCREEN-SHADER-WATCHDOG-MIGRATION-PROOF.md`
- Production approval report: `data/reports/screen-shader-production-gate-approval.v0.55.2.json`
- Test proof: `tests/screen_shader_production_gate_approval.rs`
- Human doc: `/home/kyo/.config/hypr/docs/SCREEN-SHADER-PRODUCTION-GATE-APPROVAL.md`
- Selected approval option: `Option C`.
- Production enforcement changed: yes, only for `decoration.screen_shader`.
- Production gate enforced this sprint: yes, only for `decoration.screen_shader`.
- Production write flow changed: yes, only for `decoration.screen_shader`.
- Normal path-only approval still accepted in production: no for `decoration.screen_shader`; yes for unrelated writable rows.
- Compile-aware research report: `data/reports/screen-shader-compile-aware-validation-research.v0.55.2.json`
- Test proof: `tests/screen_shader_compile_aware_validation_research.rs`
- Human doc: `/home/kyo/.config/hypr/docs/SCREEN-SHADER-COMPILE-AWARE-VALIDATION-RESEARCH.md`
- Selected compile-aware research option: `Option C`.
- Non-live validation designable: yes, for advisory/research-only fixture/temp checks.
- Recommended compile-aware policy: advisory or research-only; not required preflight.
- Compile-aware validation changed: no.
- Compile-aware validation status: deferred.
- Counted as enabled high-risk row: no.
- `SAFE_WRITABLE_ROWS` changed: no.
- Write allowlist changed: no.
- Real config/runtime touched: no.
- Reload/eval/Lua/live shader compile/live display proof used: no.

The fixture/temp-only proof covered:

- plan persisted before mutation
- backup exists before mutation
- proposed mutation applied only to temp config
- separate-process confirm
- timeout restore
- result log
- visible-display-independent recovery
- live-render-state-independent recovery
- failure-path diagnostics

Relevant reports:

- `data/reports/screen-shader-display-render-review.v0.55.2.json`
- `data/reports/screen-shader-write-policy-decision.v0.55.2.json`
- `data/reports/screen-shader-validation-boundary.v0.55.2.json`
- `data/reports/screen-shader-high-risk-template-mapping.v0.55.2.json`
- `data/reports/screen-shader-next-step-plan.v0.55.2.json`
- `data/reports/screen-shader-high-risk-gate-migration.v0.55.2.json`
- `data/reports/screen-shader-watchdog-migration-proof.v0.55.2.json`
- `data/reports/screen-shader-production-gate-architecture.v0.55.2.json`
- `data/reports/screen-shader-production-gate-approval.v0.55.2.json`
- `data/reports/screen-shader-compile-aware-validation-research.v0.55.2.json`

## 10. What Still Needs Work

Near-term work:

- Next high-risk bucket readiness and batching sprint.
- Return to grouped high-risk/bucket-level planning toward the 341-row writable goal.
- Use the extracted high-risk pattern as a framework, not automatic permission to enable rows.
- Keep `decoration.screen_shader` closed for now unless explicitly approved or required by a proven current safety failure.
- Keep production gate enforcement intact for `decoration.screen_shader`.
- Keep direct GTK file chooser visual proof, visible selected-file boundary proof, cancellation/progress behavior, Hyprland OpenGL runtime equivalence, and production compile-aware validation deferred by default.
- Do not implement production compile-aware validation.
- Do not wire compiler checks into validators, pending changes, write planning, or production apply flow without a separate approved implementation sprint.
- Do not treat standalone compiler output as Hyprland runtime safety proof.
- Do not run live shader compile, live display/render proof, reload Hyprland, or mutate real config/runtime.

Remaining deferred validators:

- `input.kb_file`
  - Optional config-relative XKB keymap path.
  - Existence/readability checks are a future UI policy question, not a mandatory writer rule yet.

- `misc.swallow_regex`
  - RE2 validation deferred.

- `misc.swallow_exception_regex`
  - RE2 validation deferred.

- `decoration.screen_shader`
  - Display/render watchdog migration proof complete.
  - Production gate enforcement decision selected Option A before architecture/proof.
  - Production gate architecture selected Option C.
  - Dry-run/non-production gate primitive exists and is fixture-proven.
  - Production gate approval selected Option C.
  - Production apply-flow gate enforcement is wired for this row only.
  - Ungated production-flow fixture writes are rejected, gated fixture writes are accepted, and invalid proof is rejected.
  - Compile-aware research selected Option C: advisory/research-only.
  - Advisory compiler feasibility selected Option A.
  - Advisory compiler integration design selected Option A.
  - Standalone candidates checked: `/usr/bin/glslangValidator` and `/usr/bin/glslc`.
  - Chosen advisory research tool: `glslangValidator`.
  - `glslangValidator -l` accepted generated known-good ES 300/320 fixture shader pairs using official Hyprland vertex shaders and rejected intentionally invalid fixture fragments.
  - `glslc` was checked but not chosen because the tested SPIR-V-oriented invocation rejected Hyprland-shaped GLSL ES fixtures for requirements that are not Hyprland's OpenGL runtime path.
  - Future advisory integration must require explicit user action before reading a shader file.
  - Future advisory integration must copy the selected shader into a temp fixture and pass only temp paths to `glslangValidator`.
  - Advisory pass/fail output must remain non-authoritative and non-blocking.
  - Advisory UI exposure design selected Option A.
  - Design-only UI metadata is projected only for `decoration.screen_shader`.
  - Advisory UI implementation proof selected Option A.
  - Non-production UI action model is implemented in `src/ui/model.rs::run_screen_shader_advisory_ui_action`.
  - Advisory GTK widget wiring proof selected Option A.
  - Visible advanced/high-risk GTK widget/control is implemented in `src/ui/window.rs::append_screen_shader_advisory_controls`.
  - Visible control is separated from write/apply controls and proves the missing-selection path only.
  - Advisory file chooser execution proof selected Option B.
  - Fixture-selected file action model is implemented in `src/ui/model.rs::run_screen_shader_advisory_selected_file_ui_action`.
  - Generated fixture/temp selected files were used in tests.
  - Original selected paths are not passed to `glslangValidator`; compiler path arguments are temp paths only.
  - Result states modeled: `not_run`, `running`, `passed`, `failed`, `unavailable`, `timed_out`, `temp_copy_failed`, `cleanup_warning`.
  - Direct GTK file chooser visual interaction and visible selected-file boundary proof are still missing.
  - Cancellation/progress behavior through visible GTK UI is still missing.
  - Compatibility with Hyprland's actual OpenGL runtime compile/link path is not proven.
  - Compile-aware validation remains deferred.
  - Screen-shader track closure report exists: `data/reports/screen-shader-track-closure.v0.55.2.json`.
  - Screen-shader track is closed for now.
  - Remaining shader-specific gaps are deferred by default, not the next default sprint.
  - Future screen-shader work should occur only with explicit approval or if a proven current safety failure appears.

Reusable high-risk pattern:

- Extracted from `decoration.screen_shader` into `data/reports/high-risk-row-pattern-from-screen-shader.v0.55.2.json`.
- This is a decision framework, not automatic permission to enable rows.
- Future high-risk rows still require source-backed, row-specific proof.
- Do not infer safety from parser acceptance, HyprMod exposure, UI metadata alone, advisory checks alone, standalone compiler output alone, or policy names without tests.
- Advisory checks do not replace recovery/watchdog gates.
- Prefer grouped high-risk bucket planning and small approved batches instead of endless one-row deep dives.

Return-to-341 roadmap:

- Created in `data/reports/return-to-341-writable-roadmap.v0.55.2.json`.
- Project goal remains all 341 official Hyprland scalar rows writable where possible.
- Current counts remain 341 readable / 278 writable / 63 blocked.
- Next work mode is grouped high-risk/bucket-level planning.
- Recommended next sprint: `Next high-risk bucket readiness and batching sprint`.

Remaining high-risk blocked settings:

- Display/render: 23
- Cursor/input: 18
- Debug/crash: 22

Future packaging/release work:

- No `.github` workflow setup should be created unless the user explicitly asks.
- No packaging/release files should be created unless requested.
- No release tags should be created unless requested.
- No GitHub release exists yet.
- AppStream warnings for unreachable GitHub URL and missing release info are expected until GitHub/release metadata exists.

## 11. What Must Not Be Done Accidentally

Do not push.

Exact prompt phrase: do not push.

Do not work on `main`.

Do not modify real Hyprland config.

Do not reload Hyprland.

Exact prompt phrase: do not reload Hyprland.

Do not run mutating `hyprctl eval`.

Do not execute or evaluate Lua.

Do not mutate active runtime.

Do not enable rows without proof.

Do not change `SAFE_WRITABLE_ROWS` without an explicit sprint and proof.

Do not change validators without source-backed evidence and tests.

Do not treat parser acceptance as semantic completeness.

Do not treat HyprMod as authoritative.

Do not use live shader compilation/render proof.

Do not remove current writable behavior without explicit user approval.

## 12. Recommended Next ChatGPT/Codex Step

Recommended next sprint title:

Next high-risk bucket readiness and batching sprint.

The sprint should:

- Return to grouped high-risk/bucket-level planning toward the 341-row writable goal.
- Use `data/reports/next-high-risk-bucket-readiness.v0.55.2.json`, `data/reports/return-to-341-writable-roadmap.v0.55.2.json`, and `data/reports/high-risk-row-pattern-from-screen-shader.v0.55.2.json` as starting proof.
- Select a grouped bucket or small subset only when source-backed proof, validators, write safety, recovery requirements, and tests can be defined.
- Keep `decoration.screen_shader` closed for now unless explicitly approved or a proven current safety failure appears.
- Keep existing production apply-flow gate enforcement for `decoration.screen_shader` unchanged.
- Not continue into direct GTK file chooser visual proof by default.
- Not continue into shader compile-aware validation by default.
- Not reload Hyprland.
- Not touch real config.
- Not enable rows.
- Not implement production compile-aware validation.
- Preserve current counts unless an explicit, approved behavior-change sprint records otherwise.

## 13. Validation Commands

Standard validation commands:

```sh
cargo fmt
cargo fmt --check
cargo check
cargo test
cargo build --release
desktop-file-validate data/applications/io.github.kyarorukyo.hyprlandsettings.desktop
appstreamcli validate --pedantic data/metainfo/io.github.kyarorukyo.hyprlandsettings.metainfo.xml || true
python /home/kyo/.config/hypr/ags/validate-hyprland-settings-export-v0552.py
python ~/.config/hypr/ags/validate-settings-ui-design-draft.py
python ~/.config/hypr/ags/validate-schema-draft.py
git status --short
git log --oneline --decorate -30
```

Current AppStream warnings are expected and non-blocking under `|| true`:

- GitHub URL not reachable.
- Releases info missing.

## 14. Appendices

Latest important commits:

- `Close screen shader track and return to 341 roadmap` (this commit)
- `Prove screen shader advisory file chooser execution`
- `e21ee10 Prove screen shader advisory GTK widget wiring`
- `ae6242b Prove screen shader advisory UI implementation`
- `4a32d6b Design screen shader advisory UI exposure`
- `9363dc6 Prove screen shader advisory compiler helper`
- `aea7586 Design screen shader advisory compiler integration`
- `865849f Prove screen shader advisory compiler feasibility`
- `df2626a Research screen shader compile-aware validation`
- `1791924 Wire screen shader production gate approval`
- `6b3bfe1 Design screen shader production gate architecture`
- `4f22d65 Decide screen shader production gate enforcement`
- `4662b86 Prove screen shader watchdog migration flow`
- `e9c665c Design screen shader high risk gate migration`
- `32a7bc0 Add ChatGPT project handoff`
- `1958cc9 Enforce screen shader display render review`
- `cf4fa58 Plan screen shader next step`
- `95edd39 Apply screen shader policy metadata`
- `280aeaa Map screen shader high risk template`
- `3053459 Define screen shader validation boundary`
- `3eedd31 Decide screen shader write policy`
- `aa1f85b Audit screen shader display render state`
- `80a8e4a Enforce deferred source backed validator repair`
- `96b73be Enforce deferred consumer source validator research`
- `6c785cc Enforce official writable validator source research`

Latest restore tag created before the screen shader closure and high-risk pattern extraction sprint:

- `pre-screen-shader-closure-high-risk-pattern-20260608-145104`

Backup paths created before the screen shader closure and high-risk pattern extraction sprint:

- `/home/kyo/Documents/hyprland-settings-pre-screen-shader-closure-high-risk-pattern-backup_20260608_145104/`
- `/home/kyo/Documents/ags-pre-screen-shader-closure-high-risk-pattern-backup_20260608_145104`
- `/home/kyo/hyprland-config-backups/hypr-pre-screen-shader-closure-high-risk-pattern-20260608_145104`
- `/home/kyo/Documents/system-audit/next-agent-handoff-pre-screen-shader-closure-high-risk-pattern-backup_20260608_145104`

Important report filenames:

- `data/reports/scalar-read-write-coverage.v0.55.2.json`
- `data/reports/all-341-unified-pipeline.v0.55.2.json`
- `data/reports/writable-253-unified-pipeline-proof.v0.55.2.json`
- `data/reports/high-risk-unified-pipeline-reconciliation.v0.55.2.json`
- `data/reports/high-risk-enabled-row-pipeline-audit.v0.55.2.json`
- `data/reports/high-risk-pipeline-template-normalization.v0.55.2.json`
- `data/reports/source-backed-writable-validator-repair.v0.55.2.json`
- `data/reports/deferred-source-backed-validator-repair.v0.55.2.json`
- `data/reports/deferred-validator-remaining-items.v0.55.2.json`
- `data/reports/screen-shader-display-render-review.v0.55.2.json`
- `data/reports/screen-shader-write-policy-decision.v0.55.2.json`
- `data/reports/screen-shader-validation-boundary.v0.55.2.json`
- `data/reports/screen-shader-high-risk-template-mapping.v0.55.2.json`
- `data/reports/screen-shader-next-step-plan.v0.55.2.json`
- `data/reports/screen-shader-high-risk-gate-migration.v0.55.2.json`
- `data/reports/screen-shader-watchdog-migration-proof.v0.55.2.json`
- `data/reports/screen-shader-production-gate-enforcement-decision.v0.55.2.json`
- `data/reports/screen-shader-production-gate-architecture.v0.55.2.json`
- `data/reports/screen-shader-production-gate-approval.v0.55.2.json`
- `data/reports/screen-shader-compile-aware-validation-research.v0.55.2.json`
- `data/reports/screen-shader-non-live-advisory-compiler-feasibility.v0.55.2.json`
- `data/reports/screen-shader-advisory-compiler-integration-design.v0.55.2.json`
- `data/reports/screen-shader-advisory-compiler-implementation-proof.v0.55.2.json`
- `data/reports/screen-shader-advisory-ui-exposure-design.v0.55.2.json`
- `data/reports/screen-shader-advisory-ui-implementation-proof.v0.55.2.json`
- `data/reports/screen-shader-advisory-gtk-widget-wiring-proof.v0.55.2.json`
- `data/reports/screen-shader-advisory-file-chooser-execution-proof.v0.55.2.json`
- `data/reports/screen-shader-track-closure.v0.55.2.json`
- `data/reports/high-risk-row-pattern-from-screen-shader.v0.55.2.json`
- `data/reports/return-to-341-writable-roadmap.v0.55.2.json`
- `data/reports/screen-shader-advisory-file-chooser-execution-proof.v0.55.2.json`

Important docs filenames:

- `/home/kyo/.config/hypr/docs/RUST-SCREEN-SHADER-DISPLAY-RENDER-REVIEW-REPORT.md`
- `/home/kyo/.config/hypr/docs/SCREEN-SHADER-DISPLAY-RENDER-REVIEW.md`
- `/home/kyo/.config/hypr/docs/SCREEN-SHADER-WRITE-POLICY-DECISION.md`
- `/home/kyo/.config/hypr/docs/SCREEN-SHADER-VALIDATION-BOUNDARY.md`
- `/home/kyo/.config/hypr/docs/SCREEN-SHADER-HIGH-RISK-GATE-MIGRATION.md`
- `/home/kyo/.config/hypr/docs/SCREEN-SHADER-WATCHDOG-MIGRATION-PROOF.md`
- `/home/kyo/.config/hypr/docs/SCREEN-SHADER-PRODUCTION-GATE-ENFORCEMENT-DECISION.md`
- `/home/kyo/.config/hypr/docs/SCREEN-SHADER-PRODUCTION-GATE-ARCHITECTURE.md`
- `/home/kyo/.config/hypr/docs/SCREEN-SHADER-PRODUCTION-GATE-APPROVAL.md`
- `/home/kyo/.config/hypr/docs/SCREEN-SHADER-COMPILE-AWARE-VALIDATION-RESEARCH.md`
- `/home/kyo/.config/hypr/docs/SCREEN-SHADER-NON-LIVE-ADVISORY-COMPILER-FEASIBILITY.md`
- `/home/kyo/.config/hypr/docs/SCREEN-SHADER-ADVISORY-COMPILER-INTEGRATION-DESIGN.md`
- `/home/kyo/.config/hypr/docs/SCREEN-SHADER-ADVISORY-COMPILER-IMPLEMENTATION-PROOF.md`
- `/home/kyo/.config/hypr/docs/SCREEN-SHADER-ADVISORY-UI-EXPOSURE-DESIGN.md`
- `/home/kyo/.config/hypr/docs/SCREEN-SHADER-ADVISORY-UI-IMPLEMENTATION-PROOF.md`
- `/home/kyo/.config/hypr/docs/SCREEN-SHADER-ADVISORY-GTK-WIDGET-WIRING-PROOF.md`
- `/home/kyo/.config/hypr/docs/SCREEN-SHADER-ADVISORY-FILE-CHOOSER-EXECUTION-PROOF.md`
- `/home/kyo/.config/hypr/docs/NEXT-HIGH-RISK-BUCKET-READINESS.md`
- `/home/kyo/.config/hypr/docs/RUST-DEFERRED-SOURCE-BACKED-VALIDATOR-REPAIR-REPORT.md`
- `/home/kyo/.config/hypr/docs/SOURCE-BACKED-VALIDATOR-DEFERRED-ITEMS.md`

Current row counts:

- Total scalar rows: 341
- Readable scalar rows: 341
- Writable scalar rows: 278
- Blocked scalar rows: 63
- Enabled high-risk rows audited: 9
- Remaining display/render blocked rows: 23
- Remaining cursor/input blocked rows: 18
- Remaining debug/crash blocked rows: 22

Remaining deferred row list:

- `input.kb_file`
- `misc.swallow_regex`
- `misc.swallow_exception_regex`
- `decoration.screen_shader`

Next recommended prompt title:

Next high-risk bucket readiness and batching sprint.
