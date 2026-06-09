# Dashboard Config Page Scaffold Review Log

## Sprint summary
- Starting commit: `8a2dde1 Fix libadwaita style warning and focus cleanup`
- Branch: main
- Files changed: `src/ui/window.rs`, `tests/dashboard_config_page_scaffold.rs`, `data/reports/dashboard-config-page-scaffold.v0.55.2.json`, `docs/DASHBOARD-CONFIG-PAGE-SCAFFOLD-REVIEW-LOG.md`
- Config files changed: no
- Runtime changed: no
- App write model changed: no
- Counts before: 341 readable / 341 writable / 0 blocked
- Counts after: 341 readable / 341 writable / 0 blocked

## UI changes
- Dashboard: added a Config card with the copy “Choose which Hyprland config the app reviews and where future changes should be saved.”
- Sidebar: added Config after Dashboard, before normal settings categories.
- Config page: added a read-only page with Config file, Connected files, Profiles, and Future changes sections.
- Normal settings pages: unchanged; they still render search, category title, settings list, and selected setting details.

## Config page behavior
- Current config display: shows the app’s currently detected config path when available, or a friendly unavailable message.
- Connected files wording: says connected-file review is planned and not active yet.
- Manual picker status: `Choose Config File...` is present but disabled.
- Profile switching status: profile switching is present as planned copy only and disabled.
- Future write-target wording: explains that when a setting is controlled in more than one place, the app will ask where to save before applying.

## Safety
- Real config edited: no
- Symlinks changed: no
- Mode scripts run: no
- Hyprland reloaded: no
- Mutating hyprctl used: no
- File picker active: no
- Profile switching active: no
- Layered writes active: no

## Tests
- Tests added: `tests/dashboard_config_page_scaffold.rs`
- What they prove: Dashboard has a Config card, sidebar includes Config after Dashboard, the Config page copy exists, future controls are disabled, normal search/detail behavior still works, and `SAFE_WRITABLE_ROWS.len()` remains 341.

## Validation
- cargo fmt: passed
- cargo fmt --check: passed
- cargo check: passed
- cargo test: passed
- cargo build --release if run: passed
- git status --short: passed with unrelated local-only audit/design artifacts still untracked

## Next recommended sprint
Implement read-only source/include-aware config detection with fixture tests, then wire its summary into the Config page.
