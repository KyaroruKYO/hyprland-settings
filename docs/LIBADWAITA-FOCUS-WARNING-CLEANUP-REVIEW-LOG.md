# Libadwaita and GTK Focus Warning Cleanup Review Log

## Sprint summary
- Starting commit: df5f0a7
- Branch: main
- User-visible issue: libadwaita warned about unsupported GTK dark-theme settings, and earlier runs showed GTK focus/root critical warnings.
- Files changed: `src/ui/app.rs`, `src/ui/window.rs`, `tests/libadwaita_focus_warning_cleanup.rs`, `data/reports/libadwaita-focus-warning-cleanup.v0.55.2.json`, this review log.
- Counts before: 341 readable / 341 writable / 0 blocked
- Counts after: 341 readable / 341 writable / 0 blocked

## Libadwaita warning
- Cause found: no project source used `gtk-application-prefer-dark-theme`; `/home/kyo/.config/gtk-4.0/settings.ini` and `/home/kyo/.config/gtk-3.0/settings.ini` set `gtk-application-prefer-dark-theme=1`.
- Fix: explicitly use libadwaita `StyleManager` with `ColorScheme::Default`, which follows system style without GTK dark-theme settings.
- Result: the warning remains under the user's normal GTK settings because the unsupported flag is external. A timed launch with a temporary GTK config that omits that flag produced no Adwaita warning.

## GTK focus warning investigation
- Interactions tested: timed startup console checks with normal GTK settings and with a temporary GTK config. Full manual GUI interaction was not automated from this shell session.
- Reproduced: no GTK focus warnings appeared in either timed startup console check.
- Source areas inspected: Dashboard synthetic sidebar item, Dashboard Open buttons, sidebar selection, settings-list selection, visibility switching, list rebuilding, search changes, initial selection, and window presentation.
- Fix applied if any: initial sidebar selection now happens after `window.set_content(Some(&root))` and before `window.present()`, so the row is rooted before selection.
- Result: pending console check.

## RADV warning
- Observed: yes, in user-provided console output.
- Treated as app bug: no.
- Reason: it is graphics-stack noise unless app-specific rendering problems appear.

## Preserved behavior
- Dashboard: preserved.
- Sidebar: preserved.
- Search: preserved with `Search settings` placeholder.
- Row list: preserved.
- Details pane: preserved.
- Write model: unchanged.

## Safety
- Backend model changed: no
- Counts changed: no
- Write gates changed: no
- Real config touched: no
- Runtime mutation/reload used: no

## Validation
- cargo fmt: passed
- cargo fmt --check: passed
- cargo check: passed
- cargo test: passed
- cargo build --release: passed
- desktop/AppStream checks if run: passed; AppStream reported the expected non-blocking `releases-info-missing` pedantic warning.
- console warning check: normal GTK settings still emit the external Adwaita warning and RADV warning; temporary GTK settings without `gtk-application-prefer-dark-theme` emit only the RADV warning; no GTK focus critical warnings observed.

## Next recommended sprint
If GTK focus warnings recur with exact interaction steps, capture a fresh console log and inspect the matching callback path.
