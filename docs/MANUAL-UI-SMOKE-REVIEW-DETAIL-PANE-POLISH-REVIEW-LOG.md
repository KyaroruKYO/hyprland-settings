# Manual UI Smoke Review and Detail Pane Polish Review Log

## Sprint summary
- Starting commit: `5b75b88 Fix setting detail visibility layout`
- Branch: `main`
- User-visible issue: the details pane was visible, but it read like raw debug metadata instead of a setting editor/review surface.
- Files changed: `src/ui/window.rs`, `tests/manual_ui_smoke_detail_pane_polish.rs`, `data/reports/manual-ui-smoke-review-detail-pane-polish.v0.55.2.json`, this review log.
- Counts before: 341 readable / 341 writable / 0 blocked.
- Counts after: 341 readable / 341 writable / 0 blocked.

## Manual smoke review
- Rows reviewed: `appearance.blur.enabled`, `cursor.default_monitor`, `debug.manual_crash`, `decoration.screen_shader`, `render.direct_scanout`, `windows.snap.enabled`.
- What worked: selection details were visible immediately after the previous split-pane fix; the projection already contained current-value, edit, safety, high-risk, and screen-shader advisory metadata.
- What was confusing: raw labels such as read support, write support, comparison detail, write candidate status, pending review availability, source line, and duplicate line numbers were shown as one long flat list.
- What was changed: the detail pane now shows user-facing grouped sections first and moves raw/internal metadata into a collapsed advanced expander.

## Detail pane polish
- Setting section: shows display name, official setting, category/subsection, and description.
- Current value section: shows configured/default/conflict/read-unavailable state, current value, source, and a plain duplicate-conflict warning when needed.
- Edit section: shows editability, proposed value, a user-facing Apply enabled/disabled reason, and the existing write controls.
- Safety section: shows risk class, write path, high-risk production gate wording, cursor monitor-name oracle wording, debug/crash wording, and screen-shader gate/advisory wording when applicable.
- Advanced metadata section: keeps row ID, raw read/write labels, report-only status, write candidate status, source line, duplicate line numbers, validation labels, review summary, and safety notes in a collapsed `Source / advanced metadata` expander.

## Row-specific verification
### appearance.blur.enabled
- Visibility: detail projection exists and remains editable.
- Current value/conflict behavior: duplicate config fixtures produce `DuplicateConflict` plus the user-facing duplicate-resolution blocker.
- Apply button behavior: Apply is blocked when duplicate entries or unreadable config prevent review.

### cursor.default_monitor
- Oracle wording: detail includes runtime monitor-name oracle validation text and explicit non-freeform wording.
- High-risk wording: detail uses cursor/input high-risk policy wording.
- Apply behavior: Apply remains gated by oracle and high-risk proof.

### debug.manual_crash
- Crash-risk wording: detail adds explicit crash/debug sensitive wording and high-risk policy warning.
- Gate behavior: the high-risk production gate requirement remains the apply path.

### decoration.screen_shader
- Screen-shader gate/advisory wording: screen-shader advisory and production gate wording remain present.
- Gate behavior: the screen-shader production gate is not weakened.

### render.direct_scanout
- Display/render high-risk wording: detail uses display/render high-risk policy warning.

### windows.snap.enabled
- Normal writable-row comparison: detail remains editable through the normal reviewed config-write flow, not a high-risk path.

## Safety
- Backend model changed: no.
- Counts changed: no.
- Write gates changed: no.
- Real config touched: no.
- Runtime mutation/reload used: no.

## Validation
- cargo fmt: passed.
- cargo fmt --check: passed.
- cargo check: passed.
- cargo test: passed.
- cargo build --release: passed.
- desktop/AppStream checks if run: `desktop-file-validate` passed; `appstreamcli validate --pedantic` passed with the expected non-blocking `releases-info-missing` note.

## Next recommended sprint
Manual UI visual pass and copy polish sprint.
