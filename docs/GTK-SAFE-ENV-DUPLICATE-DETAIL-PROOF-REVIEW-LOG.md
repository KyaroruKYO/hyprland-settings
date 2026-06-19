# GTK Safe-env Duplicate Detail Proof Review Log

## Sprint summary
- Starting commit: ff552d8b507f9e6842874a3c146ae28deee23b68
- Branch: main
- Files changed: GTK probe, AT-SPI collector, evidence matrix, summarizer, UI duplicate anchors, tests, reports
- Config files changed by Codex: none
- Runtime changed: no
- App write model changed: no
- Project data migrated to Hyprland 0.55.4: no
- Counts before: 341 readable / 341 writable / 0 blocked
- Counts after: 341 readable / 341 writable / 0 blocked

## Build/probe reporting
- App build attempted: yes
- App build succeeded: yes
- App binary rebuilt before probe: yes
- Stale binary risk removed: yes

## Duplicate detail proof
- Duplicate scenario: hyprland.conf sets decoration:blur:enabled and sources appearance.conf, which sets decoration:blur:enabled again
- Duplicate row found: yes
- Duplicate row opened: yes, through AT-SPI parent selection fallback
- Detail pane collected: yes
- Duplicate blocked reason collected: yes
- Proof level before: source_model_fallback
- Proof level after: live_gtk_atspi_proof
- Fallback proof still used: no

## UI accessibility changes
- Duplicate row accessibility: duplicate-specific tooltip text added
- Detail pane accessibility: duplicate-specific detail pane tooltip text added
- Blocked reason accessibility: duplicate blocked reason label remains stable and named
- Apply guard: collector refuses Apply and reports no forbidden Apply action

## Safety
- Safe-env mode: yes
- Live-swap: no
- Real user config edited: no
- Real backups created: no
- AGS touched: no
- Waybar touched: no
- Hyprland reloaded: no
- Mutating hyprctl used: no
- Runtime mutated: no
- Scripts executed: no
- Lua executed: no
- Screenshots committed: no
- Apply clicked: no

## Issues found
- Critical: none
- Major: none
- Minor: duplicate row opening currently uses AT-SPI parent selection fallback because GTK list rows do not expose a direct click action
- Missing proof: none for duplicate blocked-copy

## Validation
- bash -n scripts: passed
- python py_compile: passed
- cargo fmt: passed
- cargo fmt --check: passed
- cargo check: passed
- cargo test: passed
- cargo build --release: passed
- jq reports: passed
- git diff --check: passed
- git status --short: passed with unrelated pre-existing untracked audit/design files omitted from commit

## Next recommended sprint
Expand row-detail AT-SPI probes across representative blocked categories while keeping live-swap disabled.
