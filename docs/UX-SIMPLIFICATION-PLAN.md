# UX Simplification Plan (v0.2.x, local-only prototype)

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

## Follow-ups (next passes)

- Detail-pane simplification: short chip + expander for the full
  classification/proof text (the pane still shows verbose text today).
- Color picker window + gradient row for the 22 color entries (HyprMod
  `options/color.py` pattern).
- Profiles page with a real empty state ("No profiles yet") instead of a
  status frame.
- Merged "Layouts" presentation for dwindle/master subsections.
- Toggleable sidebar search (Ctrl+F) with the entry hidden by default.
- Humanized label overrides for the most-seen rows (e.g. "Inner gaps"
  style) where the official names are terse.

## Safety boundaries (unchanged)

Safe Live Save Mode still gates every Save; scalar and structured-family
save paths untouched; unproven/high-risk stays blocked or quietly
disabled with reasons one hop away; no production activation of
source/include, duplicates, profiles, monitors, gestures, or style; no
`hyprctl reload`; no fake receipts. All existing guard tests must stay
green, plus new guards proving the developer surfaces stay off the user
pages.
