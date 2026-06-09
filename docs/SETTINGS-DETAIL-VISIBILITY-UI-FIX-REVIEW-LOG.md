# Settings Detail Visibility UI Fix Review Log

## Sprint summary
- Starting commit: `492fb9f Record final main and GitHub update status`
- Branch: `main`
- User-visible bug: selecting a setting highlighted the row, but the selected setting details were pushed below a large export diagnostics panel and a vertically expanding settings list.
- Files changed: `src/ui/window.rs`, `tests/settings_detail_visibility_ui_fix.rs`, `data/reports/settings-detail-visibility-ui-fix.v0.55.2.json`, this review log.
- Counts before: 341 readable / 341 writable / 0 blocked.
- Counts after: 341 readable / 341 writable / 0 blocked.

## Layout problem
- What caused the details pane to be hidden: `src/ui/window.rs` appended the always-expanded export validation/status frame, search field, tab title, full settings `ScrolledWindow`, and detail frame in one vertical content box. The settings list had vertical expansion, so it consumed the available height before the detail pane.
- Why clicking a row pushed details downward: row selection updated the detail frame content, but the frame itself was still positioned below the full list rather than next to it.

## Layout fix
- Diagnostics/status behavior: export/status diagnostics now live in a collapsed GTK expander. The diagnostics content remains available and is constrained by a bounded scrolled area.
- Search/list behavior: search and the active tab title remain above the settings work area.
- Selected details behavior: the detail pane is now the right-side sibling of the settings list inside a horizontal `gtk::Paned`.
- Wide-window behavior: wide windows show the settings list and selected detail pane side-by-side.
- Narrow-window behavior: the diagnostics no longer dominate the page; the split work area remains immediately below search and tab title, so details are not appended after a giant diagnostics block.

## Manual verification
1. Launch the app.
2. Open Appearance.
3. Select Appearance Blur Enabled.
4. Confirm details are visible immediately.
5. Search for cursor.default_monitor.
6. Confirm high-risk/gated details are visible.
7. Confirm diagnostics are still accessible but no longer dominate the page.

## Safety
- Backend model changed: no.
- Reports/counts changed: no count changes; a UI-fix report was added.
- Real config touched: no.
- Runtime mutation/reload used: no.

## Next recommended sprint
Manual UI smoke review for the all-341 settings browser.
