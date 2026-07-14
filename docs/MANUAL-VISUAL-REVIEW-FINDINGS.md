# Manual Visual Review — Findings (HyprMod fidelity, screenshot-grounded)

> **Status update (2026-07-13, same day):** every comparison below was
> implemented and re-verified against live re-captures in
> `/home/kyo/review-after/` — see
> `data/reports/manual-visual-review-implementation.v0.55.2.json`.
> Deferred: eyedropper screen picking (portal integration; button ships
> disabled), cross-session custom-swatch persistence, and the
> inherited-record subtitle values (the app shows the honest runtime
> readback, the reference computes schema-effective values).

Date: 2026-07-13 · Model version: v0.55.2 · App commit at capture: `8c16572`
Screenshots: `/home/kyo/review/` (19 files, user-captured, both apps live side by side)
Machine-readable mirror: `data/reports/manual-visual-review-findings.v0.55.2.json`

## Inspection honesty statement

- The screenshots were **actually inspected as images** by Fable: all 19 files
  were loaded as pixels into the model's visual context, and
  `03-closeup-hs-active-border-color-row.png` was additionally re-read from
  disk with the Read tool to verify an anomaly. No finding below is inferred
  from a filename.
- **One file is unusable**: `03-closeup-hs-active-border-color-row.png`
  contains two lines of rendered document text ("additional pre-selected
  region + delayed capture commands." / "Settings — Active border color row
  closeup"), not the app row — the crop missed. The HS side of comparison 2
  therefore comes from `01-side-by-side-general.png` at full-page resolution
  (readable, but not closeup); it should be re-captured after fixes.
- Where a conclusion also relied on source code (not pixels), the finding
  says so explicitly.
- HS = Hyprland Settings (left window in side-by-side shots).
  HM = HyprMod 0.4.0 (right window). HM shows *live* config values; HS shows
  model values — value/content differences are expected and are **not**
  reported as defects, only control chrome and layout are.

---

## 1. General page

**Files inspected:** `01-side-by-side-general.png`

**Visual observations (HS):** header title "General" centered; sidebar with
uppercase category captions, per-row icons, search icon next to "Hyprland
Settings"; five headed cards Gaps / Borders / Border Colors / Layout / Snap;
Gaps rows use wide plain text entries showing "5"; Border size shows
"0.00 − +"; Border Colors has four rows (Inactive, Active, Nogroup Border,
Nogroup Border Active), each with two small solid square swatches, a ⌫ after
each, then "+", then "45 − +"; Layout row carries a "Needs attention" badge
and a Dwindle dropdown; Snap has five rows.

**Visual observations (HM):** same five section names; Gaps rows use compact
"5 − +" / "10 − +" spin groups; Borders has Border size "1 − +" plus a
"Resize on border" switch row; Border Colors has two rows with large rounded
checkered tiles, ⌫ after each tile, then "45 − +", then "+" at the row end;
Layout and Snap sections minimal; no status badges anywhere.

**Concrete HS/HM differences:**
1. **Section names match exactly** (Gaps, Borders, Border Colors, Layout,
   Snap) — PASS, no fix needed.
2. **Numeric control style mismatch:** HS Gaps rows are wide flat text
   entries; HM uses compact − / + spinners. HS also formats integers as
   "0.00" where HM shows "1".
3. **Color-row chrome mismatch:** HS swatches are ~22 px sharp squares and
   the add-stop "+" sits *before* the angle spinner; HM tiles are ~44×28 px
   rounded rects and "+" sits at the *far right, after* the angle spinner.
4. **HS exposes extra rows:** Nogroup Border / Nogroup Border Active under
   Border Colors; Window snap gap, Monitor snap gap, Border overlap, Respect
   gaps under Snap. This is the deliberate 341-row model (not a defect), but
   it makes the page denser than HM. HM additionally shows "Resize on
   border" under Borders, which HS keeps on another page.
5. **HS shows a status badge** ("Needs attention" on Layout); HM shows no
   inline statuses. The badge is explicitly permitted by the prior
   acceptance rules — flagged for user decision, not auto-fixed.
6. **Subtitle copy style:** HS uses raw official descriptions with
   inconsistent capitalization and occasional two-line wraps ("inactive
   border color for window that cannot be added to a group"); HM uses
   single-line sentence-case summaries. HS titles "Nogroup Border" /
   "Nogroup Border Active" have odd casing.
7. **Sidebar difference:** HS is missing **XWayland** and **Ecosystem**
   under ADVANCED; HM shows both. Source inspection
   (`sidebar_items`, src/ui/window.rs:607) shows model-backed pages hide
   when they claim zero rows at runtime — so those two pages claimed
   nothing against the live model. Either their rows are being routed to
   another page (misrouting) or the live export puts them in
   differently-named tabs. Needs runtime investigation; the 341-reachability
   guard passes on the test bundle, so this is runtime-model-specific.
8. Spacing/card width/row height are otherwise comparable; both apps place
   headings above cards; HM rows have slightly roomier padding.

**Required fixes:** compact spin controls for numeric General rows (plain
entry only where multi-value syntax truly requires it); integer formatting
for integer options; swatch tile geometry + "+"-after-angle reorder (see
comparison 2); sentence-case single-line subtitles and title casing;
investigate and fix XWayland/Ecosystem page hiding at runtime.

**Acceptance criteria:** a re-captured side-by-side shows − / + spinners on
Gaps/Borders; "1" not "0.00" style values on integer rows; color-row control
order matching HM; XWayland and Ecosystem visible in the HS sidebar (or a
documented reason they are empty at runtime).

**Confidence:** high. **Method:** image inspection; item 7's mechanism from
source inspection.

---

## 2. Active border color row

**Files inspected:** `04-closeup-hm-active-border-color-row.png` (HM);
`01-side-by-side-general.png` (HS side — `03-…` closeup unusable, see
honesty statement).

**Visual observations:** HM row: large rounded checkered tile, ⌫, second
tile, ⌫, segmented "45 − +" spinner, then a standalone "+" at the row end.
HS row (from 01): small solid square, ⌫, small solid square, ⌫, "+",
"45 − +".

**Concrete HS/HM differences:**
- **Tile size:** HS ~22×22 px vs HM ~44×28 px.
- **Tile shape:** HS square/sharp vs HM wide rounded rect.
- **Checkerboard:** HM tiles clearly show checkerboard under translucent
  color. HS's tiles in this capture are opaque white/black so the
  checkerboard cannot be confirmed or denied from pixels; the swatch code
  draws one, but visual proof is missing until the closeup is re-taken.
- **Spacing:** tile→⌫ spacing similar in both; not a defect.
- **Remove placement:** both put ⌫ immediately right of each tile — matches.
- **Plus placement:** HS before the angle spinner; HM after it, at the row
  end — mismatch.
- **Angle spinner:** both segmented "value − +" groups, visually close;
  position relative to "+" differs as above.
- **Overall:** structure matches, geometry and ordering do not; HS does not
  yet visually pass as HM-equivalent on this row.

**Required fixes:** enlarge swatch tiles to HM proportions (~44×28, rounded);
move add-stop "+" to the end of the control group after the angle spinner.

**Acceptance criteria:** a valid HS closeup (re-captured) shows rounded
~44×28 checkered tiles, ⌫ per tile, angle spinner, then "+" last — same
left-to-right order as `04-closeup-hm-active-border-color-row.png`.

**Confidence:** high for HM; medium-high for HS (full-page resolution, no
closeup). **Method:** image inspection.

---

## 3. Color picker — palette view

**Files inspected:** `05-closeup-hs-color-picker-palette.png`,
`05-hs-color-picker-palette.png`, `06-closeup-hm-color-picker-palette.png`,
`06-hm-color-picker-palette.png`

**Visual observations (HS):** popover attached to the swatch; **page content
visibly bleeds through the picker surface** (the underlying "45", ⌫ icons and
"Needs attention" text are readable through it); header = plain-text
"Cancel", bold centered "Pick a Color", blue "Select" pill; 8×5 grid of
small (~40 px) isolated square swatches with wide gaps, row 1 a black→white
grayscale ramp, rows 2–5 hue rows by brightness; bottom row "Custom" label
with a bare "+" icon; no stored custom swatches.

**Visual observations (HM):** centered opaque floating dialog with shadow,
content behind slightly dimmed; "Cancel" is a pill *button*; palette is
**9 hue columns × 5 vertically contiguous shades** — each column a rounded
stack (blue, green, yellow, orange, red, purple, brown, white/grays,
dark/black) with big ~72×44 tiles and tight gaps; "Custom" caption above a
row of [+ tile][saved checkered custom swatches][✓ on the selected one].

**Concrete HS/HM differences:**
- **Surface/dimming:** HS translucent popover vs HM opaque elevated dialog —
  the single biggest "too raw" signal.
- **Dialog size:** HS ~370 px wide and short; HM wider (~430 px) and taller.
- **Header buttons:** HS Cancel lacks button chrome; HM Cancel is a pill.
- **Palette structure:** transposed — HS shade-rows × hue-columns of small
  detached squares; HM hue-columns of contiguous shade-stacks with rounded
  column ends and much larger tiles.
- **Custom memory:** HS has none; HM keeps previously picked customs with a
  selected checkmark.
- **Verdict on "too raw/simple":** yes — HS reads as a bare popover grid,
  HM reads as a designed picker.

**Required fixes:** present the picker as an opaque in-window dialog
(`adw::Dialog`); rebuild the palette as hue-column shade-stacks with
HM-scale tiles; pill-style Cancel; add a Custom row with persisted custom
swatches and a selected checkmark.

**Acceptance criteria:** re-captured HS palette shows an opaque dialog (no
page text readable through it), ≥9 hue columns of 5 contiguous shades,
Cancel as a button, and a Custom row that grows as custom colors are picked.

**Confidence:** high. **Method:** image inspection.

---

## 4. Color picker — custom/HSV view

**Files inspected:** `07a-closeup-hs-color-picker-custom-top.png`,
`07-hs-color-picker-custom.png`, `08a-closeup-hm-color-picker-custom-top.png`,
`08a-hm-color-picker-custom-top.png`,
`08b-closeup-hm-color-picker-custom-bottom.png`,
`08b-hm-color-picker-custom-scrolled.png`

**Visual observations (HS):** same translucent popover; vertical stack:
full-width flat preview strip → full-width hex entry ("ffffffff") →
SV rectangle rendered as **visibly stepped/blocky cells** with a small
hollow-circle marker at the corner → **horizontal hue slider with a plain
gray trough (no rainbow gradient visible)** → horizontal alpha slider drawn
as a standard blue-filled GTK scale (no checkerboard).

**Visual observations (HM):** larger opaque dialog; top row = **eyedropper
button | wide rounded preview swatch | hex entry ("#BF4040")**; main area =
**vertical rainbow hue bar (left) + large continuous SV square (right)**
with thin full-length crosshair lines marking the current point; bottom =
horizontal **checkerboard-under-gradient alpha slider**. The top and
scrolled captures show different regions of the same dialog — HM's custom
view scrolls (or is tall enough to need it at this size).

**Concrete HS/HM differences:**
- **Layout order:** does not match. HS stacks everything full-width;
  HM uses [eyedropper|preview|hex] row, then hue-left/SV-right, then alpha.
- **Eyedropper/preview/hex area:** HS has preview and hex but as separate
  full-width rows and **no eyedropper**; HM has the compact three-element row.
- **Hue placement/appearance:** HS horizontal with a plain trough (reads as
  an unlabeled slider); HM vertical with a rainbow gradient trough.
- **SV area:** HS stepped cells + dot marker; HM continuous gradient +
  crosshair lines. The stepped rendering was a recorded deferral; the
  screenshots confirm it reads clearly worse than HM.
- **Alpha:** HS plain blue scale, no checkerboard; HM checkered gradient
  trough.
- **Scroll/resize:** HM's dialog is larger and scrollable; HS's fixed
  compact popover would clip an HM-scale layout — HS needs a bigger dialog
  and/or scrollable content to match.
- **Bottom/custom controls:** nothing below alpha in either view's captures;
  no missing bottom controls beyond the eyedropper row placement.

**Required fixes:** restructure to HM's order (eyedropper+preview+hex row;
vertical rainbow hue; large continuous SV with crosshair; checkered alpha);
implement the SV area as a continuous gradient (per-pixel/cairo gradient
rendering instead of stepped cells); custom-draw hue and alpha troughs; add
an eyedropper using the XDG desktop portal color-picking API; host in the
same opaque `adw::Dialog` as the palette view with scrollable content.

**Acceptance criteria:** re-captured HS custom view is pixel-comparable to
`08a`/`08b`: three-element top row, vertical rainbow hue, smooth SV with
crosshair, checkered alpha, opaque surface.

**Confidence:** high. **Method:** image inspection.

---

## 5. Animations page

**Files inspected:** `09-side-by-side-animations.png`,
`09a-closeup-hs-animations-bezier-row.png`,
`10a-closeup-hm-animations-bezier-row.png`,
`09b-closeup-hs-animation-record-menu.png`,
`10b-closeup-hm-animation-row-controls.png`

**Visual observations (HS):** top card "Bezier Curve Editor / Create and
manage animation curves" with **no icon and no chevron**; "Animation
Records" list showing raw text rows — "Fade / speed 3.00 (enabled 1, bezier
quick)", "Workspaces / speed 3.00 (enabled 1, bezier easeOutQuint, style
fade)" etc. — with ☰ menu buttons (visible in 09b) but no switches; below
it a large "Animations & curves" workbench card: instructional prose, record
dropdown, Speed/Enabled/Curve controls, button row "Preview with recovery /
Keep changes / Revert now / Cancel / Save previewed value", the text
"Supervised preview: modify-existing only, readback-verified, auto-revert on
timeout.", an "Animation records not yet supported" expander, a second
Bezier-curves block with X0/Y0/X1/Y1 spinners and another five-button row,
and the footer "Save writes once with a backup · Requires Safe Live Save
Mode · Full safety detail: Safety Details page".

**Visual observations (HM):** Bezier Curve Editor row with curve icon and
chevron; sections **General** (Enable animations master switch), **Global**,
**Windows & Layers** (Windows, Layers), **Fading** (Fade), **Workspaces**,
**Other** (Border, Border Angle, Zoom Factor, Monitor Added); every record
is a **switch row with a friendly summary subtitle** ("4.0ds · easeOutQuint",
"3.0ds · easeOutQuint · fade", "inherited · 8.0ds · default") plus a ☰ menu
and an expander chevron; no proof/safety prose anywhere.

**Concrete HS/HM differences (each claim from the prompt, verified):**
- HS still record/debug-oriented — **confirmed**.
- HM grouped into General / Global / Windows & Layers / Fading / Workspaces
  / Other — **confirmed visible**.
- HS exposes raw `speed/enabled/bezier` text — **confirmed**.
- HM uses friendly summaries and toggle rows — **confirmed**.
- HS Bezier row lacks HM's icon/chevron/compact style — **confirmed**
  (09a vs 10a).
- HS record menu differs from HM row control behavior — **partially**: HS
  rows do have ☰ menus, but no enable switch on the row, no expander
  chevron, and raw subtitles.
- HS proof/safety text leaks into the normal page — **confirmed** (the
  supervised-preview line, the save-gating footer, and the style-not-
  editable prose are all on the normal Animations page).

**Required fixes (presentation only — the gated preview/save flow must not
change):** regroup records into HM's six sections; make each record a
switch row (enable receipt-gated where applicable) with a
"speed · curve [· style]" friendly subtitle; move the workbench's
Speed/Curve/Preview/Save controls into the per-record ☰ menu / expander;
move supervised-preview and save-gating prose to the detail popover and
Safety Details; add icon + chevron to the Bezier row.

**Acceptance criteria:** re-captured side-by-side shows HS with grouped
switch rows and friendly subtitles, no raw "(enabled 1, bezier …)" text, no
proof prose on the page, Bezier row with icon+chevron; all existing safety
guards still green.

**Confidence:** high. **Method:** image inspection.

---

## 6. Bezier editor windowing behavior

**Files inspected:** `09d-hs-bezier-editor-opens-as-separate-tiled-window.png`
(also corroborated by `09b`, `10b`)

**Visual observations:** in 09d the HS Bezier editor is a **separate
top-level window that Hyprland has tiled** — it occupies its own tile
stacked above the main Hyprland Settings window, each with its own frame.
On the right, HM's Bezier Curve Editor is an **in-window modal overlay**:
a rounded dialog with an X close button floating centered *inside* the HM
window, page content dimmed behind it (10b shows it in closeup: Preset
dropdown, draggable control-point handles on the curve graph, animated
preview slider, Control Points spinners).

**Concrete HS/HM differences:**
- HS Bezier editor **is** incorrectly a separate top-level window — confirmed
  from pixels (two independent tiled frames).
- It **should** be an attached modal/dialog overlay like HM's — confirmed
  behavior mismatch.

**Required fix (GTK/libadwaita approach):** present the editor as an
**`adw::Dialog`** on the main window. `adw::Dialog` renders as an overlay
widget *inside* the parent window rather than mapping a new toplevel
surface, so a tiling compositor never sees a second window — this is the
deterministic fix. (`gtk::Window` + `set_transient_for` + `set_modal` is not
sufficient on Hyprland: transients may still tile or depend on user
windowrules.) The same approach should host the color picker (comparisons
3–4) instead of the current popover.

**Acceptance criteria:** opening the Bezier editor produces zero new
Hyprland clients (`hyprctl clients` count unchanged); a re-captured
screenshot shows it overlaying the Animations page inside the main window,
matching HM's presentation.

**Confidence:** high. **Method:** image inspection (windowing observed from
pixels); the fix approach from GTK/libadwaita knowledge, not pixels.

---

## Fix priority (proposed)

1. **Bezier editor windowing** (comparison 6) — behavioral break, worst UX.
2. **Animations page regrouping** (5) — largest visual divergence + safety
   prose leak on a normal page.
3. **Picker surface + palette redesign** (3, and dialog host for 4) —
   opacity and structure.
4. **Custom/HSV view rebuild** (4) — continuous SV, rainbow hue, checkered
   alpha, eyedropper.
5. **Color row geometry/order** (2) and **General numeric controls** (part
   of 1).
6. **XWayland/Ecosystem sidebar hiding investigation** (1.7) — correctness
   check on runtime page claiming.

Deliberate non-fixes: HS's extra rows (Nogroup/Snap detail — the 341-row
model is a feature) and the "Needs attention" badge (explicitly allowed by
prior acceptance rules; user decides).
