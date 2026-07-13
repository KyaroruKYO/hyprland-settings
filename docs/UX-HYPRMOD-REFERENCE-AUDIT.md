# UX Reference Audit — HyprMod 0.4.0 (2026-07-13)

Reference: `/home/kyo/Downloads/hyprmod-0.4.0.tar.gz` (HyprMod 0.4.0,
Python + GTK4/libadwaita, **GPL-3.0**, © Ivo Šmerek), inspected unpacked in
a scratch directory.

**License / scope note.** This sprint targets a **local-only private
prototype** (not shared, released, packaged, or published). Studying and
adapting a GPL-3.0 program for private use is permitted by its license.
The adaptation here is structural: HyprMod is Python and this app is Rust,
so the implementation is written fresh, following HyprMod's *organization,
layout, and interaction patterns*; user-facing setting names and
descriptions come from this project's own bundled official Hyprland
metadata (341 rows from `hyprctl`), not from HyprMod's prose. If this
prototype ever moves toward distribution, revisit licensing (this repo is
MIT; GPL-derived material would need a compatibility review).

## What HyprMod does well (and why it feels good)

1. **Task-oriented sidebar, not config-section dump.** Six small category
   headers — *Look & Feel, Input, Display, Window Management, Startup,
   Advanced* — each with 2–4 pages, plus *Profiles* and *Settings* pinned
   at the bottom. Users navigate by intent ("make it look nice"), not by
   `hyprland.conf` section name. (`hyprmod/ui/sidebar.py`)
2. **A curated presentation schema.** `hyprmod/data/schema/options.json`
   exposes only **130 options**, organized groups → sections → options,
   each with a short label ("Inner gaps"), a one-line description, a type,
   and bounds. The schema is the UI: pages are generated from it. The
   lesson is *curation and layering* — a friendly layer above the raw
   config namespace, with everything else reachable but not in the face.
3. **One widget per option type, chosen by a factory.**
   (`hyprmod/ui/options/factory.py`): bool → switch row, int/float → spin
   row, choice → combo row, color → color row with a picker window,
   gradient → gradient row, vec2 → paired spin row, string → entry row.
   Every option is a single compact `ActionRow`-style card: label left,
   short description underneath, control right.
4. **Search is the sometimes path.** The search entry is hidden until the
   user toggles it (Ctrl+F); categories stay at the top of the sidebar.
   (`sidebar.py` docstring makes this explicit.)
5. **Status is a chip, not a wall.** Pending changes appear as a small
   count badge/chip in the header, and the sidebar rows can show a count
   badge. No page opens with paragraphs of status text.
   (`hyprmod/ui/pending_chip.py`, sidebar badges)
6. **Consolidated empty states.** One `EmptyState` component
   (Adw.StatusPage conventions: icon, title, one-line description,
   optional pill action buttons) reused for empty lists, no-results, and
   pre-onboarding. (`hyprmod/ui/empty_state.py`)
7. **Merged "Layouts" page.** Dwindle/Master/Scrolling are one page with a
   view switcher rather than three sidebar entries. (`sidebar.py`,
   `pages/layouts.py`)
8. **Specialized editors get their own pages** (Monitors with a visual
   preview, Keybinds, Window Rules, Autostart, Env Variables), while plain
   scalars share the generic option-row machinery.
9. **Version-aware hiding**: schema groups unavailable on the running
   Hyprland version are silently dropped, so the sidebar never shows dead
   pages.

## Files used as reference

- `hyprmod/ui/sidebar.py` — category structure, pinned rows, toggleable
  search, badge pattern.
- `hyprmod/data/schema/options.json` — schema shape (groups → sections →
  options; labels, one-line descriptions, types, bounds, `depends_on`).
- `hyprmod/ui/options/{factory,base,numeric,combo,color,text,multi}.py` —
  per-type option rows and the color/gradient picker approach.
- `hyprmod/ui/empty_state.py` — empty-state conventions.
- `hyprmod/ui/pending_chip.py`, `banner.py` — quiet status affordances.
- `hyprmod/pages/{section,settings,layouts,profiles}.py` — page assembly.

## What we adapt (mapped to this app)

| HyprMod pattern | Adaptation here |
| --- | --- |
| Task-oriented sidebar with category headers | Grouped sidebar: Look & Feel / Input / Display / Window Management / Advanced, with Dashboard pinned on top; same navigation mechanism (list rows) so existing automation keeps working |
| Curated schema over raw options | `src/ux_presentation.rs`: category grouping + quiet status chips layered over the existing 341-row model (labels/descriptions already come from official Hyprland metadata) |
| Status chips, not walls | Six short chips: Live Preview · Save Only · Requires Safe Live Save Mode · Hardware Required · Not Proven Yet · Blocked |
| Advanced category for rarely-touched groups | All proof receipts, activation reviews, pilot status, and classification detail move to a **Safety Details** page under Advanced |
| Option-row factory (switch/spin/combo/color) | Already present in the detail pane (53 switches, 45 sliders, 22 color entries, 9 dropdowns); follow-up: color picker window + gradient row |
| Consolidated empty state | Follow-up: Profiles empty state ("No profiles yet") replacing the profile/mode status frame |
| Merged Layouts page | Follow-up: dwindle/master rows presented as one "Layouts" section |

## What we deliberately do NOT copy

- HyprMod's write model (it applies changes; our saves stay behind Safe
  Live Save Mode and the gated single-write path — this sprint does not
  touch the safety model).
- HyprMod's description prose and option subset: our labels/descriptions
  derive from the official `hyprctl` metadata this app already bundles,
  covering all 341 rows instead of 130.
- Any Python code (different language; implementation is fresh Rust).
