# Config Picker Preview Flow Review Log

## Sprint summary
- Starting commit: 75de6b1be57ac77d5850833d2d22e3798b700391 Add config selection scaffold
- Branch: main
- Files changed: src/ui/window.rs; tests/config_picker_preview_ui.rs; tests/config_selection_ui.rs; tests/dashboard_config_page_scaffold.rs; data/reports/config-picker-preview-flow.v0.55.2.json; docs/CONFIG-PICKER-PREVIEW-FLOW-REVIEW-LOG.md
- Config files changed: no
- Runtime changed: no
- App write model changed: no
- Counts before: 341 readable / 341 writable / 0 blocked
- Counts after: 341 readable / 341 writable / 0 blocked

## Picker behavior
- Button status: Choose Config File... is active on the Config page.
- File picker implementation: gtk::FileChooserNative with gtk::FileChooserAction::Open.
- Selected file preview: accepted file paths are shown under Selected for review.
- Clear/cancel behavior: Clear selected file cancels the preview state and hides the preview area.
- Persistence behavior: selected paths are not saved, persisted, or applied.

## Selection state
- Manual preview state: ConfigSelectionState stores the selected file as preview-only state.
- Source-follow preview: Review all connected files, Only this file, and Cancel are shown as disabled future controls.
- Write target behavior: manual preview paths do not become write targets.
- ConfigDiscovery behavior: manual preview paths do not replace or mutate ConfigDiscovery.
- CurrentConfigSnapshot behavior: manual preview paths do not reload or replace the current config snapshot.

## Config page UI
- Config file section: explains that auto-detection is a starting point and another file can be chosen for review.
- Preview copy: This has not changed what the app will write; This selection is not saved yet.
- Clear/cancel copy: Clear selected file.
- Disabled/future controls: source-follow choices are visible but disabled.

## Write-flow preservation
- Write target changed: no
- Apply behavior changed: no
- Selected config persisted: no
- Real config reload from selected path: no

## User-facing wording
- Friendly wording added: Choose another config file to review; Selected for review; Clear selected file.
- Technical wording avoided: source graph, symlink provenance, duplicate scalar conflict, ambiguous write target, parser normalization, write target.

## Tests
- Tests added: tests/config_picker_preview_ui.rs
- What they prove: the picker button is active, the file picker is preview-only, selected files do not affect writes or app data, clear/cancel preview copy exists, source-follow controls remain disabled, and counts remain 341 / 341 / 0.
- Tests updated: tests/config_selection_ui.rs; tests/dashboard_config_page_scaffold.rs
- What they prove: Config page copy and Dashboard/Config routing still match the preview-only flow.

## Safety
- Real config edited: no
- Symlinks changed: no
- Scripts run: no
- Lua executed: no
- Hyprland reloaded: no
- Mutating hyprctl used: no
- Profile switching active: no
- Layered writes active: no

## Validation
- cargo fmt: passed
- cargo fmt --check: passed
- cargo check: passed
- cargo test: passed
- cargo build --release: passed
- git status --short: passed; current sprint files were present alongside pre-existing untracked local audit/design artifacts that were not staged for this sprint

## Next recommended sprint
Add a read-only selected-file preview inspection panel that can summarize the chosen file without persisting it, reloading app data, changing Apply behavior, enabling profile switching, or introducing layered writes.
