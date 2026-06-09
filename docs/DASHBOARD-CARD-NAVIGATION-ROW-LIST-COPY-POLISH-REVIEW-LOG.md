# Dashboard Card Navigation and Row-list Copy Polish Review Log

## Sprint summary
- Starting commit: eb213c9
- Branch: main
- User-visible issue: the settings row list still exposed internal proof/debug metadata instead of simple setting summaries.
- Files changed: `src/ui/window.rs`, `tests/dashboard_card_navigation_row_list_copy_polish.rs`, `data/reports/dashboard-card-navigation-row-list-copy-polish.v0.55.2.json`, this review log.
- Counts before: 341 readable / 341 writable / 0 blocked
- Counts after: 341 readable / 341 writable / 0 blocked

## Dashboard navigation
- Card behavior: cards remain simple Dashboard cards.
- Button behavior: each card keeps an `Open` button that selects the mapped sidebar category.
- Deferred behavior if any: whole-card click behavior is deferred; the tested navigation path is the Open button.

## Row-list copy changes
- Removed from visible rows: read/write allowlist labels, report-only state, editable pilot wording, preview status, raw risk class, official setting key, internal row id, and technical proof labels.
- Kept in details: official setting, row id, source line, write/read labels, preview status, risk class, validation labels, and proof metadata.
- Moved to advanced metadata: raw/internal labels remain in the collapsed `Source / advanced metadata` details expander.
- Friendly labels added: `Current: On`, `Current: Off`, `Uses Hyprland default`, `Needs attention`, `Extra care needed`, and `Not available right now`.

## Smoke-reviewed rows
### appearance.blur.enabled
- Row-list wording: projects as a friendly setting name, description, and current/default/attention state.
- Detail behavior: polished details pane remains available.

### cursor.default_monitor
- Row-list wording: projects a friendly setting name and `Extra care needed` rather than runtime-oracle proof labels.
- Detail behavior: monitor-name oracle and high-risk gate wording remain available in details.

### debug.manual_crash
- Row-list wording: projects `Extra care needed` rather than raw debug/crash risk labels.
- Detail behavior: strong crash-risk and gate wording remains available in details.

### decoration.screen_shader
- Row-list wording: projects `Extra care needed` rather than screen-shader proof labels.
- Detail behavior: screen-shader gate/advisory wording remains available in details.

### render.direct_scanout
- Row-list wording: projects `Extra care needed` rather than display/render raw risk labels.
- Detail behavior: high-risk details remain available.

### windows.snap.enabled
- Row-list wording: projects normal friendly current/default status.
- Detail behavior: normal writable-row details remain available.

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
Manual visual smoke pass for row-list copy in the running GTK app, followed by small interaction polish if whole-card navigation is desired.
