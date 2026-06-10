# Config Preview, Layered Values, and Write Target Foundation Review Log

## Sprint summary
- Starting commit: 06435635d5f666c3bcfc317a093a6c0bd8ca2ef9 Add config picker preview flow
- Branch: main
- Files changed: src/config_graph.rs; src/config_selection.rs; src/config_layered_values.rs; src/write_target_candidate.rs; src/lib.rs; src/ui/window.rs; tests/config_picker_preview_inspection.rs; tests/config_source_follow_preview.rs; tests/session_config_selection.rs; tests/layered_current_values.rs; tests/write_target_candidates_fixture.rs; tests/config_graph.rs; tests/config_picker_preview_ui.rs; data/reports/config-preview-layered-write-target-foundation.v0.55.2.json; docs/CONFIG-PREVIEW-LAYERED-WRITE-TARGET-FOUNDATION-REVIEW-LOG.md
- Config files changed: no
- Runtime changed: no
- App write model changed: no
- Counts before: 341 readable / 341 writable / 0 blocked
- Counts after: 341 readable / 341 writable / 0 blocked

## Stage 1: selected-file preview inspection
- UI: selected manual config previews now show a Selected file preview panel.
- Read-only graph inspection: the preview uses the config graph engine to count connected files, unreadable files, profile hints, script-managed hints, generated hints, cycles, and unsupported source patterns.
- Preview copy: This file is only being reviewed; This has not changed what the app will write; This selection is not saved yet.
- Safety: the preview does not rebuild UiProjection, replace ConfigDiscovery, replace CurrentConfigSnapshot, enable Apply, change write targets, or persist the selected path.

## Stage 2: source-follow preview behavior
- Review all connected files: follows source/include files in the preview graph only.
- Only this file: uses a root-only graph preview and says connected files are not included.
- Cancel: clears selected preview state and hides the preview panel.
- Safety: source-follow choices do not affect active app config or writes.

## Stage 3: session config selection
- Implemented or deferred: session-selection state is implemented; full session reread is deferred.
- Session-only behavior: ConfigSelectionLifecycle::SessionReadOnly can represent a read-only session choice.
- Persistence: no selected config is persisted.
- Write behavior: session-selected config does not become the write target; the visible Use for this session control remains disabled/planned.

## Stage 4: layered current values
- Model: config_layered_values::LayeredSettingValues records scalar occurrences across connected config files.
- UI: setting details can show This setting is controlled in more than one place and read-only occurrence lines.
- Fixture proof: base, desktop, and gaming profile values are proven with fixtures.
- Safety: structured block false positives are not treated as scalar layered values, and layered values are read-only.

## Stage 5: fixture write-target foundation
- Model: write_target_candidate::WriteTargetCandidate represents possible future targets.
- Fixture behavior: main config, desktop profile, gaming profile, generated/script-managed, and occurrence-line targets are fixture-tested.
- Real write behavior: production apply path is unchanged.
- Safety: generated or script-managed targets are flagged as unsafe/requiring advanced confirmation in the fixture model only.

## User-facing wording
- Friendly wording added: This file is only being reviewed; This setting is controlled in more than one place; Choose where to save changes in a future version.
- Technical wording avoided: source graph, symlink provenance, duplicate scalar conflict, ambiguous write target, parser normalization, write target.

## Tests
- Tests added: config_picker_preview_inspection; config_source_follow_preview; session_config_selection; layered_current_values; write_target_candidates_fixture.
- What they prove: preview inspection is read-only, source-follow only changes the preview, session config state is non-persistent and not a write target, layered values are read-only, fixture write-target candidates are generated without writing real files, and counts remain 341 / 341 / 0.

## Safety
- Real config edited: no
- Symlinks changed: no
- Scripts run: no
- Lua executed: no
- Hyprland reloaded: no
- Mutating hyprctl used: no
- Profile switching active: no
- Layered real writes active: no
- Real write target selection active: no

## Validation
- cargo fmt: passed
- cargo fmt --check: passed
- cargo check: passed
- cargo test: passed
- cargo build --release: passed
- git status --short: passed; current sprint files were present alongside pre-existing untracked local audit/design artifacts that were not staged for this sprint

## Next recommended sprint
Implement an isolated read-only session preview model that can reread selected config values for display without changing persistence, Apply, profile switching, or real write-target selection.
