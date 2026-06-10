# Connected File Detail UI Review Log

## Sprint summary
- Starting commit: c0dbf70f25d768b8c1e5da3f797451b7a52c764d Add connected files review UI
- Branch: main
- Files changed: src/ui/window.rs, tests/connected_file_detail_ui.rs, data/reports/connected-file-detail-ui.v0.55.2.json, docs/CONNECTED-FILE-DETAIL-UI-REVIEW-LOG.md
- Config files changed: no
- Runtime changed: no
- App write model changed: no
- Counts before: 341 readable / 341 writable / 0 blocked
- Counts after: 341 readable / 341 writable / 0 blocked

## Config page UI
- Connected files list: preserved the read-only connected-file cards from the previous sprint.
- Detail view pattern: added a collapsed `Details` expander inside each connected-file card.
- Root file explanation: root files say they are the config file the app is currently reviewing.
- Source-connected file explanation: sourced files say they are connected from another config file.
- Source line display: when the graph can match a connected file to a source/include reference, the detail view shows source file and line number.
- Symlink target display: symlinked files keep showing `Points to` in the card and in the detail expander.
- Readable state display: detail view shows `Readable: Yes` or `Readable: No`.
- Notes/hints: detail view summarizes generated, script-managed, symlinked, and profile-style hints in friendly wording.
- Disabled controls: Choose Config File, Choose review mode, and Profile switching remain disabled/planned.

## User-facing wording
- Friendly wording added: Why this file is listed, Role, Readable, Symlink, Points to, Connected from, Notes, This file may be changed by scripts, This file appears to be generated.
- Technical wording avoided: source graph, symlink provenance, duplicate scalar conflict, ambiguous write target, parser normalization, ConfigManagementHint, ConfigDetectionConfidence, and canonical path are not used as main UI copy.

## Graph engine
- Changes made, if any: none.
- Read-only behavior preserved: the detail UI consumes existing ConfigGraphFile and ConfigSourceReference data and does not add writes, picker behavior, profile switching, script execution, Lua execution, Hyprland reloads, or runtime mutation.

## Tests
- Tests added: tests/connected_file_detail_ui.rs.
- What they prove: detail helpers exist, user-facing detail copy is present, root/source-connected explanations exist, source-line metadata is used without raw config contents, generated/script/profile notes exist, future controls remain disabled, and SAFE_WRITABLE_ROWS remains 341.

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
Add a read-only connected-file detail pane or review mode with filtering/sorting if the inline expanders become too dense, still before any file picker or write-target implementation.
