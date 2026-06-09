# Hide Export Diagnostics from Normal UI Review Log

## Sprint summary
- Starting commit: 12d835a
- Branch: main
- User-visible issue: normal category pages showed a development diagnostics dropdown above search.
- Files changed: `src/ui/window.rs`, `tests/settings_detail_visibility_ui_fix.rs`, `tests/hide_export_diagnostics_normal_ui.rs`, `data/reports/hide-export-diagnostics-normal-ui.v0.55.2.json`, this review log.
- Counts before: 341 readable / 341 writable / 0 blocked
- Counts after: 341 readable / 341 writable / 0 blocked

## UI change
- Removed from normal UI: the visible export validation diagnostics expander and card.
- Search placeholder: changed from `Search export metadata` to `Search settings`.
- Dashboard behavior: remains card-based and does not show export diagnostics.
- Category page behavior: starts with search, then category title, then the list/details split.

## Preserved behavior
- Sidebar: Dashboard, Appearance, Windows & Layout, Display, Input, Keyboard, Cursor, Permissions, System, Animations.
- Dashboard cards: unchanged.
- Row-list copy: friendly copy from the previous sprint remains intact.
- Detail pane: grouped Setting, Current value, Edit, Safety, and advanced metadata sections remain intact.
- Advanced metadata: remains available inside selected setting details.
- Backend diagnostics/reports: report and test evidence remain available; the unused visible diagnostics UI helper code was removed instead of left as dead code.

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
Manual visual smoke pass in the running GTK app to confirm category pages start with search and no export diagnostics are visible.
