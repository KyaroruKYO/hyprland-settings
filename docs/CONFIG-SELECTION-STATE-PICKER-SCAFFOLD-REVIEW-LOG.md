# Config Selection State and Picker Scaffold Review Log

## Sprint summary
- Starting commit: e25c6db1a69f008dc13ab07eceb7689979d27eaa Add connected file detail UI
- Branch: main
- Files changed: src/config_selection.rs, src/lib.rs, src/ui/window.rs, tests/config_selection_state.rs, tests/config_selection_ui.rs, tests/dashboard_config_page_scaffold.rs, data/reports/config-selection-state-picker-scaffold.v0.55.2.json, docs/CONFIG-SELECTION-STATE-PICKER-SCAFFOLD-REVIEW-LOG.md
- Config files changed: no
- Runtime changed: no
- App write model changed: no
- Counts before: 341 readable / 341 writable / 0 blocked
- Counts after: 341 readable / 341 writable / 0 blocked

## Selection state model
- Module: src/config_selection.rs
- Auto-detected state: ConfigSelectionState can represent an auto-detected config path.
- Manual preview state: ManualConfigChoice represents a manually selected path as preview-only.
- Source-follow choices: SourceFollowChoice supports ReviewAllConnectedFiles, OnlySelectedFile, and Cancel.
- Confirm/cancel behavior: confirm_preview records a future-review confirmation state; cancel_preview clears the manual preview.
- Persistence behavior: no persistence exists; manual preview does not become the write target.

## Config page UI
- Config file section: now explains that auto-detection is a starting point.
- Manual picker entry point: Choose Config File... (planned) is visible but disabled.
- Preview-only copy: the page says a selected file has not changed what the app will write and manual selection is preview-only/not saved yet.
- Source-follow scaffold: the page names Review all connected files, Only this file, and Cancel as planned choices.
- Disabled/preview-only controls: Choose Config File, Choose review mode, and Profile switching remain disabled/planned.

## Write-flow preservation
- Write target changed: no
- Apply behavior changed: no
- Selected config persisted: no
- Real config reload from selected path: no

## User-facing wording
- Friendly wording added: Auto-detection is a starting point; Choose another config file to review; This has not changed what the app will write; Review all connected files; Only this file; Cancel.
- Technical wording avoided: source graph, symlink provenance, duplicate scalar conflict, ambiguous write target, parser normalization, ConfigSelectionState, SourceFollowChoice, and write target are not used as main Config page copy.

## Tests
- Tests added: tests/config_selection_state.rs and tests/config_selection_ui.rs.
- What they prove: state model can represent auto-detected config, manual preview, source-follow choices, cancel and confirm states; manual preview is preview-only and not the write target; Config page shows planned picker copy; existing Config route, connected-file cards/details, search, and final counts remain intact.

## Safety
- Real config edited: no
- Symlinks changed: no
- Scripts run: no
- Lua executed: no
- Hyprland reloaded: no
- Mutating hyprctl used: no
- File picker active: no
- Profile switching active: no
- Layered writes active: no

## Validation
- cargo fmt: passed
- cargo fmt --check: passed
- cargo check: passed
- cargo test: passed
- cargo build --release: passed
- git status --short: passed with current sprint changes plus pre-existing untracked local audit/design artifacts intentionally left uncommitted

## Next recommended sprint
Implement a harmless read-only file picker preview flow only after deciding how selected config previews should be displayed and cancelled without affecting writes.
