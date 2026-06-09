# Dashboard Sidebar Simplification Review Log

## Sprint summary
- Starting commit: c6e9be7
- Branch: main
- User-visible issue: Overview, Gaming, and Diagnostics appeared as empty 0-row sidebar categories, and every sidebar item showed row counts.
- Files changed: `src/ui/window.rs`, `tests/dashboard_sidebar_simplification.rs`, `data/reports/dashboard-sidebar-simplification.v0.55.2.json`, this review log.
- Counts before: 341 readable / 341 writable / 0 blocked
- Counts after: 341 readable / 341 writable / 0 blocked

## Sidebar changes
- Removed: Overview, Gaming, Diagnostics
- Added: Dashboard as the first sidebar item
- Renamed: Keybinds is displayed as Keyboard while preserving the internal `keybinds` tab id.
- Row counts removed: sidebar rows now show only the navigation label.

## Dashboard behavior
- Dashboard layout: a synthetic page that does not use the settings list and selected-setting details pane.
- Cards: Appearance, Windows & Layout, Input, Displays, Shortcuts, Advanced.
- Needs attention behavior: shown only when the current config projection reports duplicate/conflicting rows, parser warnings, or structured warning entries.
- Technical details behavior: Dashboard does not show backend coverage counts, proof wording, or diagnostics. Export diagnostics remain available on normal settings pages through the collapsed diagnostics expander.

## Normal settings behavior
- Appearance: remains a normal settings category with list and detail pane.
- Windows & Layout: remains a normal settings category with list and detail pane.
- Display: remains a normal settings category with list and detail pane.
- Input: remains a normal settings category with list and detail pane.
- Keyboard: displays the existing keybind-related settings under the friendlier label.
- Cursor: remains a normal settings category with list and detail pane.
- Permissions: remains a normal settings category with list and detail pane.
- System: remains a normal settings category with list and detail pane.
- Animations: remains available and is moved to the bottom of the sidebar.

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

## Next recommended sprint
Manual visual smoke pass for Dashboard card spacing and optional card navigation polish, without changing the write model.
