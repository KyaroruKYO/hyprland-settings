# Connected Files Review UI Review Log

## Sprint summary
- Starting commit: df98f1fe0296ffa10b795b520ea8e97cd679f7ab Add read-only config graph detection
- Branch: main
- Files changed: src/ui/window.rs, tests/dashboard_config_page_scaffold.rs, tests/connected_files_review_ui.rs, data/reports/connected-files-review-ui.v0.55.2.json, docs/CONNECTED-FILES-REVIEW-UI-REVIEW-LOG.md
- Config files changed: no
- Runtime changed: no
- App write model changed: no
- Counts before: 341 readable / 341 writable / 0 blocked
- Counts after: 341 readable / 341 writable / 0 blocked

## Config page UI
- Connected files summary: preserved the friendly graph summary already added by the previous sprint.
- Connected files list: added read-only cards for each detected connected file.
- File labels: cards use Main config, Current profile, Desktop profile, Gaming profile, Theme profile, Host profile, Generated file, and Connected config labels.
- Symlink target display: symlinked files show `Points to:` followed by the resolved target path.
- Generated/script-managed hints: cards show Generated file, May be changed by scripts, Symlinked file, and Profile file labels when detected.
- Unreadable file warnings: the section lists missing or unreadable connected file paths.
- Cycle warnings: the section explains that connected files can refer back to each other and that the app stops following them to avoid looping.
- Unsupported source warnings: the section explains that some connected file patterns are not shown yet.
- Disabled controls: Choose Config File, Choose review mode, and Profile switching remain disabled/planned.

## User-facing wording
- Friendly wording added: “Some connected files could not be read,” “Some connected files refer back to each other,” “The app stopped following them to avoid looping,” and “Review carefully before editing these files in a future version.”
- Technical wording avoided: source graph, symlink provenance, duplicate scalar conflict, ambiguous write target, parser normalization, ConfigManagementHint, and ConfigDetectionConfidence are not used as main UI labels.

## Graph engine
- Changes made, if any: none.
- Read-only behavior preserved: the UI consumes the existing ConfigGraphSummary and ConfigGraphFile data and does not add write, picker, profile, script, Lua, or runtime behavior.

## Tests
- Tests added: tests/connected_files_review_ui.rs.
- What they prove: connected-files review helpers exist, friendly labels exist, issue warning copy exists, future controls remain disabled, Dashboard/Sidebar/normal settings behavior is still wired, and SAFE_WRITABLE_ROWS remains 341.

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
Add a read-only connected-file detail view or review mode that lets users inspect detected file roles, unreadable files, and management hints before implementing any file picker or write-target selection.
