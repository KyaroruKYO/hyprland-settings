# Manual Visual Review — Screenshot Checklist

Side-by-side review of Hyprland Settings (commit `8c16572`) against HyprMod 0.4.0.
Each item says exactly where to go, what must be visible, and what counts as a failure.

## Setup

**Hyprland Settings (HS):**

```sh
cd /home/kyo/Projects/hyprland-settings && cargo run --release
```

**HyprMod (HM)** — needs network once to install its `hyprland-*` PyPI deps
(this is why Fable could not run it offline):

```sh
cd ~/Downloads && tar -xzf hyprmod-0.4.0.tar.gz && cd hyprmod-0.4.0
python -m venv --system-site-packages .venv   # reuses system pygobject/GTK4
.venv/bin/pip install -e .
.venv/bin/hyprmod
```

> ⚠️ **HyprMod edits your real Hyprland config.** It is a live config editor —
> browse and screenshot only; do **not** click Save/Apply or accept pending
> changes. Consider `cp -r ~/.config/hypr ~/.config/hypr.bak-review` first.

**Capture the focused window:**

```sh
grim -g "$(hyprctl activewindow -j | jq -r '"\(.at[0]),\(.at[1]) \(.size[0])x\(.size[1])"')" ~/review/NN-name.png
```

**Existing HS baselines** (captured during the pixel-fidelity pass, reusable for
items 1, 5, 7, 9): `/tmp/hyprland-settings-visual-fidelity/settings-general-v3.png`,
`settings-picker-palette.png`, `settings-picker-custom.png`, `settings-decoration.png`,
`settings-animations.png`. Items 3, 11, 13, 14 need fresh captures.

---

## 1. HS — General page

**Open:** sidebar → *Look & Feel* → **General**.

**Must show:**
- Top header bar title reads **General** (not "Hyprland Settings").
- Five separately headed cards, in order: **Gaps**, **Borders**, **Border Colors**, **Layout**, **Snap** — heading text sits *above* each rounded card, not inside it.
- Layout card contains the layout dropdown (Dwindle/Master/…) and the *Allow tearing* switch; Snap card has the snap rows (these are cross-tab claims — their absence is a regression).
- Rows are: name + one short description line + inline control on the right.

**Failure signs:** a combined **"Gaps & Borders"** heading; any row showing **"Uses Hyprland default · Live Preview"**, "Current: …", or similar status prose; any button labeled **"Color..."**/**"Color…"**; header showing the app name; Layout/Snap sections missing.

## 2. HM — General page

**Open:** sidebar → **General**.

**Use to compare with #1:** same five section names (HyprMod's schema defines exactly Gaps / Borders / Border Colors / Layout / Snap), heading-above-card placement, section spacing, row height, right-aligned controls, and color rows rendered as swatch chips.

**Legit differences (not failures):** HyprMod shows *live* values from your running Hyprland; HS may show official defaults where you haven't set a value. HyprMod may show a pending-changes chip in its header.

## 3. HS — Active border color row (close-up)

**Open:** *Look & Feel* → **General** → **Border Colors** card → **Active border color** row. Crop/zoom the screenshot to this row.

**Must show:**
- One **checkered swatch** per gradient stop (checkerboard visible under any transparency).
- A circular **remove** button (× / clear icon) beside each stop.
- An **add stop** button (works up to 10 stops).
- An **angle spinner** ("45 − +" style) since this row is gradient-capable.
- A **discard** (undo-arrow) button restoring the original value.

**Failure signs:** a text button reading "Color..."; a bare hex text entry as the only control; no angle control; swatches without the checkerboard (alpha becomes unreadable).

## 4. HM — Active border color row (close-up)

**Open:** HM sidebar → **General** → Border Colors → active border row.

**Use to compare with #3:** stop chip size and ordering, where the delete affordance sits relative to each chip, add-button placement, and the angle control's position/format.

## 5. HS — Color picker, palette view

**Open:** from #3, click any color swatch.

**Must show:**
- Dialog header: **Cancel** (left) / title **"Pick a Color"** / **Select** (right).
- A grid of palette swatches as the primary content.
- A visible way into the custom editor ("Custom").

**Failure signs:** the stock GTK ColorChooser dialog; a raw text/hex entry as the primary interface; no explicit Select confirmation (color applied on click alone).

## 6. HM — Color picker, palette view

**Open:** HM → General → click a color chip.

**Use to compare with #5:** header button arrangement, palette grid density, path to the custom view. **Known accepted difference:** HS generates its palette; HM's palette and recent-swatch memory differ.

## 7. HS — Color picker, custom/HSV view

**Open:** from #5, choose **Custom**.

**Must show:**
- Live **preview strip** that updates while dragging.
- **Hex entry** accepting manual input.
- 2D **saturation/value area** with a marker that follows click and drag.
- **Hue** and **alpha** sliders.

**Failure signs:** hex entry missing or not applied; marker not tracking clicks; preview strip frozen; Select returning a value in the wrong format family (e.g. hex where the config used `rgba(...)`).

**Known accepted difference:** the SV area renders as stepped cells, not a continuous gradient (recorded deferral).

## 8. HM — Color picker, custom/HSV view

**Open:** HM picker → custom view.

**Use to compare with #7:** layout order of preview/hex/SV/hue/alpha, and marker behavior.

## 9. HS — Animations page

**Open:** sidebar → *Look & Feel* → **Animations**.

**Must show:**
- A **Bezier Curve Editor** row (chevron) that opens the editor window with the multi-curve graph.
- An **Animation Records** card where each supported record has a **menu button** (☰/open-menu) exposing enabled/speed/curve plus Preview / Keep / Revert / Save.

**Failure signs:** the Bezier editor reachable mainly from Settings/Config; record rows with no menu; proof-wall or receipt text on the page.

## 10. HM — Animations page

**Open:** HM sidebar → **Animations** (its curve editor lives here; see `data/screenshots/curves.png` in the HM tarball as a second reference).

**Use to compare with #9:** curve-editor placement under Animations and per-record row controls.

## 11. HS — Sidebar

**Open:** any page; capture the full window so the sidebar is intact.

**Must show:**
- Uppercase category captions: **LOOK & FEEL, INPUT, DISPLAY, WINDOW MANAGEMENT, STARTUP, ADVANCED**.
- Human page names with icons: General, Decoration, Animations, Cursor / Keybinds, Devices, Gestures / Monitors, Workspaces / Layouts, Window Rules, Layer Rules / Autostart, Env Variables / XWayland, Ecosystem, System, Permissions, Windows & Layout, Profiles, Settings, Safety Details.
- A small **search icon button** in the sidebar header next to the app identity label (clicking it — or Ctrl+F — reveals the search entry).
- Sidebar at a fixed sensible width (~260 px), readable label size.

**Failure signs:** a large **"Search"** text button in the main window; sidebar consuming half the window; developer-style names (`xwayland`, `env-variables`); missing category headers.

## 12. HM — Sidebar

**Open:** HM main window.

**Use to compare with #11:** naming style and search placement. **Legit difference:** HM lists Dwindle / Master / Scrolling as separate top-level pages; HS deliberately merges them into one **Layouts** page. HM also has a Pending Changes page HS doesn't need (different save model).

## 13. HS — Settings page

**Open:** sidebar → *Advanced* → **Settings**.

**Must show:** config-file selection, the **Safe Live Save Mode** control, and a short explanatory note — nothing else.

**Failure signs:** a Bezier/animation editor as main content (its old home — this was the point of the move); proof frames, receipts, or debug output (those belong on **Safety Details**).

## 14. HS — Profiles page

**Open:** sidebar → *Advanced* → **Profiles**.

**Must show:** a clean empty state — centered icon, "No Profiles" title, one explanatory sentence.

**Failure signs:** developer/status-frame styling, raw JSON or metadata dumps, bordered debug boxes, classification/proof text.

---

## Filing results

Name captures `01-hs-general.png` … `14-hs-profiles.png` to match the item
numbers. Anything that trips a failure sign: note item number + what you saw;
each of these is covered by a guard in `tests/visual_fidelity.rs` /
`tests/gui_correction.rs`, so a real regression means a guard gap worth closing.
