# UX Simplification Plan (v0.2.x, local-only prototype)

> Historical planning artifact. It records the design target at that point in
> development; it is not the current implementation/status source. See
> `README.md` and `PROJECT-STATUS.md`.

Companion to `docs/UX-HYPRMOD-REFERENCE-AUDIT.md`. Goal: the app should
feel settings-first like HyprMod — grouped sidebar, clean cards, short
labels and descriptions, quiet status — with every proof/activation/debug
surface moved out of the everyday path and **zero safety regression**.

## New user-facing layout

Sidebar (grouped, category headers non-selectable; same list mechanism as
today so navigation automation keeps working):

```
Dashboard                     (pinned top)

Look & Feel
  Appearance                  (existing tab: appearance)
  Animations                  (existing tab: animations + hl.animation/hl.curve pickers)
  Cursor                      (existing tab: cursor)

Input
  Input                       (existing tab: input)
  Keyboard                    (existing tab: keybinds)

Display
  Displays                    (existing tab: display)

Window Management
  Windows & Layout            (existing tab: windows-layout)

Advanced
  System                      (existing tab: system)
  Permissions                 (existing tab: permissions)
  Config                      (slimmed: file selection, Safe Live Save Mode, record pickers, save note)
  Safety Details              (NEW page: every developer/proof surface)
```

## Category/page mapping

**All 341 scalar rows** keep their existing model-assigned tab and
subsection (both derived from the bundled official metadata); the change
is presentational: tabs are grouped under the category headers above, and
each page renders its subsections as card groups. No row moves between
tabs in this pass, so search, the detail pane, save paths, and every
guard keep working unchanged. Rows per page = the model's current tab
row counts (dynamic; pages with zero rows stay hidden, matching
HyprMod's version-aware hiding).

**7 structured families**:

| Family | Where it appears |
| --- | --- |
| hl.animation | Animations context — record picker card (enabled / speed / curve), Config page card retained this pass |
| hl.curve | Same picker card (Bezier curves group) |
| hl.gesture | Safety Details (blocked: no readback mechanism; hardware deferred) |
| hl.bind | Safety Details (blocked high-risk, reason shown) |
| hl.device | Safety Details (blocked high-risk) |
| hl.monitor | Safety Details (blocked high-risk; display recovery required) |
| hl.permission | Safety Details (blocked high-risk) |

## What moves into Safety Details (out of the everyday path)

Everything below moves off the Config page verbatim (same widgets, same
assertions, one navigation hop away — nothing is deleted or weakened):

1. Connected files review (generated/script-managed/symlink diagnostics).
2. Profile/mode status frame (until the HyprMod-style empty state lands).
3. Structured-family read-only editor review (raw records + parser
   status).
4. Disabled future approval cards + all nested production activation
   review sections (decisions, paths, controls, forms, drafts, draft
   edits, persistence boundary, safety gates, gate proofs, final
   decisions, approval/dry-run, opt-in requirements, caps).
5. Controlled write / active pilot status.
6. Runtime preview readiness classification summary.
7. Structured-family runtime preview status (proof receipt text).

The slimmed **Config** page keeps only what a user acts on: which config
file is used, the Safe Live Save Mode card, the record pickers, and the
short save-behavior note.

## Quiet status vocabulary (honest, short)

Per-row status collapses to one short chip (`src/ux_presentation.rs`),
replacing paragraph-length classification text in list rows:

- **Live Preview** — previews instantly, revert/cancel.
- **Save Only** — persists via gated Save; no live preview.
- **Requires Safe Live Save Mode** — save blocked until the mode is on.
- **Hardware Required** — proof needs hardware not present (touch/tablet,
  secondary devices).
- **Not Proven Yet** — no passed proof receipt; disabled.
- **Blocked** — high-risk or no safe mechanism; reason one tap away.

Full reasons stay available in the detail pane and Safety Details.

## Implemented in this pass

1. `src/ux_presentation.rs`: sidebar category model + status chips +
   guard-tested vocabulary.
2. Grouped sidebar with category headers.
3. New Safety Details page; Config slimmed to the four user-facing cards.
4. Dashboard card for Safety Details (navigation parity for automation).
5. Setting-row status text switched to the chip vocabulary.
6. GTK harness updated (activation-card assertions now probe Safety
   Details); regression guards keep proof-wall text off user pages.

## Implemented in the second pass (presentation adoption slice)

1. **Full 341-row presentation resolution** (`src/presentation_labels.rs` +
   `ux_presentation::resolved_row_label`): 127 rows matched through the
   existing normalized key mapping adopt friendly short labels (provenance
   recorded per row as `reference_key`, test-verified against the raw
   official setting keys, which are unchanged); 214 unmatched rows keep
   their official metadata labels — reported, never guessed. Full
   accounting: `data/reports/hyprmod-full-presentation-adoption.v0.55.2.json`.
   Descriptions stay original one-line text derived from the official
   metadata (the reference app's authored prose is GPL-licensed third-party
   work; verbatim adoption would need an owner-level attribution/licensing
   decision, recorded in the report).
2. **Hidden search**: the entry hides by default; Ctrl+F reveals and
   focuses it; Escape clears and hides it (from the window or inside the
   entry); categories stay the primary navigation.
3. **Friendly dropdown display labels** for finite-choice rows (generic
   humanization); the raw value remains the id that gets applied,
   validated, and saved.
4. **Quiet picker card**: retitled "Animations & curves" with a two-line
   intro and a gate-chip footer pointing to Safety Details; proof prose
   removed from the everyday path; friendlier empty state.
5. Detail-pane heading uses the resolved friendly label.
6. Guards: `tests/hyprmod_presentation_adoption.rs` (counts, provenance,
   raw-value preservation, unchanged classifications, Ctrl+F wiring, quiet
   picker card, honest report pins).

## Implemented in the third pass (whole-GUI overhaul)

1. **The split metadata-browser layout is gone.** Settings pages are one
   centered clamped column (max 800, tighten 600) holding a rounded
   boxed-list card with section headings per subsection; the permanent
   "Setting details" pane was removed and the full detail surface (same
   controls, save flow, and reasons) now opens on demand as a popover
   anchored to the opened row.
2. **Inline controls on rows**: the 135 default-previewable rows carry
   right-aligned controls (switch / spin / dropdown / color swatch /
   compact entry) driving the existing reversible preview executor through
   a lazily created, session-drop-registered controller. Save remains the
   separate gated step. All other rows keep the quiet chip.
3. **Color rows (all 22)**: inline swatch, picker popover with live
   preview, manual entry, validated Apply and Cancel; exact raw text
   preserved; unparseable values fail closed to text-only.
4. **Gradient strips** for values that parse as two or more colors with an
   optional angle; everything else fails closed.
5. **Search index** now spans friendly labels, chip text, dotted and
   colon-form raw keys, categories/sections, and descriptions with
   normalization.
6. **Profiles**: centered inert empty state. **Layouts**: merged
   Dwindle/Master/Scrolling tab page over the same model rows.
7. **Fallback labels**: unmatched rows drop the redundant page-name prefix
   (formatting only; matched labels stay first).
8. Guards: `tests/gui_overhaul.rs`; harness DetailPane probe now opens a
   row first (the pane is on-demand).

Deferred with reasons (recorded in the overhaul report): Monitors
cards/badges, source-grouped rules/autostart/env lists, and the Workspaces
page all need safe read-only model projections before an honest shell can
exist; inline dead-man supervision stays in the detail surface.

## Implemented in the fourth pass (correction/precision)

1. **Tooltips removed app-wide** (146 calls) except the documented
   harness/accessibility identifiers and the Safety-Details review-card
   descriptors; inline-control feedback moved to an error style class.
2. **Section headings hang above the cards**: per-section standalone
   heading label + separate rounded card, with natural section names
   (Blur, Shadow, Border Colors, Snap, Rounding and Opacity, ...).
3. **Stop-based color rows**: checkered per-stop swatches with their own
   validated pickers, per-stop remove, add-stop, gradient angle stepper,
   and a back-arrow discard that restores the original value — all through
   the reversible preview path, fail-closed on anything unparseable.
4. **Bezier Curve Editor moved under Animations** (chevron row opening an
   editor window with a multi-curve graph + the proven curve picker), and
   editable animation records got menu buttons with compact per-record
   controls. The Settings page no longer hosts animation/curve controls.
5. **Sidebar matched to the target model**: LOOK & FEEL / INPUT / DISPLAY /
   WINDOW MANAGEMENT / STARTUP / ADVANCED with General, Decoration,
   Keybinds, Devices, Gestures, Monitors, Workspaces, Layouts, Window
   Rules, Layer Rules, Autostart, Env Variables, XWayland, Ecosystem,
   Profiles, Settings; uppercase caption headers and larger row labels.
6. **Page partition model** (`SIDEBAR_PAGE_LAYOUT`): 341/341 rows land on
   exactly one page (guard-tested); all 7 families mapped (read-only
   source-entry cards on Keybinds/Monitors/Gestures/Devices/Ecosystem;
   Window Rules is a real source-grouped locked list).
7. **Startup pages**: Autostart and Env Variables as honest empty states
   (the parser does not preserve exec/env lines — documented blocker).

## Implemented in the fifth pass (pixel fidelity)

Header shows the page title; General renders Gaps/Borders/Border Colors/Layout/Snap (cross-tab claims); routine status text left normal rows (badge-only); color rows seed from defaults with no generic button; the picker gained palette + custom HSV views; the sidebar gained an identity header, icon search, and page icons. Guards: `tests/visual_fidelity.rs`. Screenshots: `/tmp/hyprland-settings-visual-fidelity/`.

## Implemented in the sixth pass (screenshot-grounded fidelity fixes)

Driven by 19 user-captured side-by-side screenshots of both apps running
live (`/home/kyo/review/`, findings in
`docs/MANUAL-VISUAL-REVIEW-FINDINGS.md`): the Bezier editor became an
in-window `adw::Dialog` (no second tiled compositor client); the
Animations page regrouped into General / Global / Windows & Layers /
Fading / Workspaces / Other with switch rows and friendly
"speed · curve" subtitles (raw record text and preview/save prose off the
page; the workbench moved to Safety Details); the color picker became an
opaque `adw::Dialog` with a hue-column shade-stack palette, session
custom-swatch memory, and a rebuilt custom view (eyedropper placeholder,
vertical rainbow hue bar, continuous SV plane with crosshair,
checkerboard alpha); color row tiles grew to rounded 44×26 with the
add-stop button after the angle spinner; numeric rows use compact
spinners with integer formatting; XWayland/Ecosystem pages un-hidden
(tab-claim routing bug). Guards: `tests/screenshot_fidelity_fixes.rs`.

## Implemented in the seventh pass (pending-changes fidelity)

Screenshot-grounded match of the reference unsaved-changes system
(`/home/kyo/review/`, report
`data/reports/pending-changes-fidelity.v0.55.2.json`): changed rows carry
an amber left-edge accent; nav rows carry per-page count pills; an amber
header chip (icon + count, hidden at zero) opens a hidden Pending Changes
review page — count header, page-grouped rows with Added/Modified pills,
per-row revert/navigate, a config diff preview rendered read-only with the
real writer's helpers, and a calm empty state; a slide-up bottom bar
offers Discard and a gated Save now split button (profile item disabled).
Unset boolean switches now seed from generated trusted 0.55.4 official
defaults instead of the edit projection's flip-suggestion. Guards:
`tests/pending_changes_fidelity.rs`.

## Follow-ups (next passes)

- Detail-pane simplification: short chip + expander for the full
  classification/proof text (the pane still shows verbose text today).
- Eyedropper: wire the picker's screen-pick button to the XDG desktop
  portal PickColor call (button ships disabled today).
- Custom-swatch persistence across sessions (in-memory only today).
- Structured-family record previews in the pending ledger (their
  supervised auto-revert flow does not fit applied-live-until-discarded).
- SIGTERM/SIGINT preview revert (glib 0.22 exposes no unix-signal
  binding; normal window close reverts, plain kill does not).
- GtkComboBoxText popups are invisible to AT-SPI (harness drives the
  Selection interface instead); consider gtk::DropDown for better
  accessibility.

## Backend completion follow-ups (2026-07-14)

- **Dead-man kept page badge**: a kept dead-man change resolves its sidebar
  badge via `page_for_official_setting`, which put `input.repeat_rate`'s
  badge on General instead of Devices. Cosmetic (chip + row accent + bar are
  correct); align the resolver with the model tab so the badge lands on the
  row's actual page.
- **Save-only text-entry commit**: save-only `Entry` controls (kb layout,
  fonts, swallow regexes) stage on Enter/activate only; consider staging on
  focus-leave too for parity with switches/spinners.
- **Monitor/display rollback**: the display-variant dead-man dialog text is
  built and unit-tested but has no live trigger until a proven monitor
  capture/apply/verify/revert flow exists — the gate to unblocking
  `hl.monitor`.
- **Profiles**: still disabled; needs create-from-current + load-into-pending
  + whole-config write/reread/rollback before enabling apply.
- **Structured-family ledger**: hl.animation/hl.curve stay on the supervised
  Preview/Keep/Revert/Save flow; unifying them with the applied-live ledger
  would need their record lifecycle reconciled with DeadManKept semantics.
- **Stale status chip on now-editable rows (2026-07-14)**: broadening save-only
  to 117 rows left the status chip reading "Hardware required" / "Blocked" /
  "Not Proven Yet" next to a working save-only control (the chip describes live
  preview, not the save-only editor). Refine `status_chip_for_row` to read
  "Save Only" when `is_save_only_editable`, without disturbing the reliability
  matrix classification.
- **Production-gated rows (51)**: render/debug/cursor/xwayland settings stay
  uneditable because `apply_setting_change` requires a HighRiskProductionGate
  recovery-plan proof to save. Making them editable needs that recovery flow
  wired through save-only (capture original + recovery plan; the backup is the
  recovery), NOT a gate bypass — a deliberate, separately-reviewed change.

## Safety boundaries (unchanged)

Safe Live Save Mode still gates every Save; scalar and structured-family
save paths untouched; unproven/high-risk stays blocked or quietly
disabled with reasons one hop away; no production activation of
source/include, duplicates, profiles, monitors, gestures, or style; no
`hyprctl reload`; no fake receipts. All existing guard tests must stay
green, plus new guards proving the developer surfaces stay off the user
pages.
