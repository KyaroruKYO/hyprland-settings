# Session Config Preview Model Review Log

## Sprint summary
- Starting commit: 01959ad1e68579eba44fb36c6cdd1413b8eb2a53 Add config preview and layered target foundation
- Branch: main
- Files changed: src/session_config_preview.rs; src/lib.rs; src/ui/window.rs; tests/session_config_preview_model.rs; tests/session_config_preview_read.rs; tests/session_config_preview_ui.rs; tests/config_picker_preview_ui.rs; data/reports/session-config-preview-model.v0.55.2.json; docs/SESSION-CONFIG-PREVIEW-MODEL-REVIEW-LOG.md
- Config files changed: no
- Runtime changed: no
- App write model changed: no
- Counts before: 341 readable / 341 writable / 0 blocked
- Counts after: 341 readable / 341 writable / 0 blocked

## Session preview model
- Module: src/session_config_preview.rs
- Session-selected config: represented by SessionConfigPreview selected_config_path.
- Source-follow mode: Review all connected files follows source/include files; Only this file uses root-only graph inspection.
- Read-only behavior: selected configs are read and summarized for display only.
- Persistence: not persisted.
- Clear behavior: Clear session preview clears local preview/session state in the Config page.

## Session read/display behavior
- Graph summary: connected, readable, unreadable, generated/script-managed, cycle, and unsupported pattern state remains available through the graph summary.
- Read/value summary: scalar values are counted from readable files for preview only.
- Layered value summary: settings with multiple locations are counted for preview only.
- Unreadable/error behavior: unreadable selected configs show friendly no-change copy.
- Production model isolation: production ConfigDiscovery, CurrentConfigSnapshot, UiProjection, and Apply are not replaced or rebuilt by session preview.

## Config page UI
- Use for session control: Use for this session preview is active after choosing a file.
- Session-only copy: Using this config for this app session only.
- Not-saved copy: This is not saved.
- Display-only copy: This config is being reread for display only.
- Clear session preview: active clear control hides and clears session state.
- Source-follow copy: Only this file states connected files are not included in this session preview.
- Disabled/future controls: real write-target selection remains inactive.

## Write-flow preservation
- Write target changed: no
- Apply behavior changed: no
- Selected config persisted: no
- Production CurrentConfigSnapshot changed: no
- Production ConfigDiscovery changed: no
- Production UiProjection changed: no
- Real write-target selection active: no

## User-facing wording
- Friendly wording added: Using this config for this app session only; This is not saved; Apply behavior has not changed; Clear session preview.
- Technical wording avoided: source graph, symlink provenance, duplicate scalar conflict, ambiguous write target, parser normalization, write target.

## Tests
- Tests added: tests/session_config_preview_model.rs; tests/session_config_preview_read.rs; tests/session_config_preview_ui.rs
- What they prove: session preview is read-only, non-persistent, clearable, source-follow aware, isolated from production write/model paths, and preserves 341 / 341 / 0.

## Safety
- Real config edited: no
- Symlinks changed: no
- Scripts run: no
- Lua executed: no
- Hyprland reloaded: no
- Mutating hyprctl used: no
- Profile switching active: no
- Layered real writes active: no
- Real write-target selection active: no

## Validation
- cargo fmt: passed
- cargo fmt --check: passed
- cargo check: passed
- cargo test: passed
- cargo build --release: passed
- git status --short: passed; current sprint files were present alongside pre-existing untracked local audit/design artifacts that were not staged for this sprint

## Next recommended sprint
Add an isolated read-only session value projection for setting detail rows so users can compare active config values and session-preview values side by side without changing Apply behavior.
