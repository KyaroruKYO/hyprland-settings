# Style Editing Evidence (2026-07-13)

Machine-readable result: `data/reports/style-editing-evidence.v0.55.2.json`.

## Decision

**Blocked: no trusted valid-value evidence.** The animation style field stays
not-editable. Saves preserve the current style unchanged; the UI keeps the
disabled style row with the reason.

## What was searched (trusted local sources only)

1. **Installed official source headers** — the `hyprland` 0.55.4 package
   ships `/usr/include/hyprland/src`. No style value enumeration exists in
   the shipped headers; the validation lives in compiled `.cpp` sources
   that are not installed.
2. **Official typed Lua API stub** — `/usr/share/hypr/stubs/hl.meta.lua`
   (1251 lines) documents the `hl.animation` API and contains **no style
   value documentation at all**.
3. **Official example config** — `/usr/share/hypr/hyprland.lua` shows style
   *usage* only: `style = "popin 87%"` on window leaves and
   `style = "fade"` on layer/workspace leaves. Two important facts follow:
   usage examples are not an enumeration, and styles are **leaf-specific
   and parameterized** (a percentage argument), so even the shape of a
   finite global dropdown would be wrong.
4. **Trusted 0.55.4 descriptions export** — 341 rows, no animation style
   metadata (animation tree records are not config options).
5. **Runtime readback** — `hyprctl animations` reports each record's
   current style but enumerates nothing.

## What would unblock style editing

Official local evidence enumerating valid styles **per animation leaf**
(e.g. a future descriptions export or Lua stub that carries them), then an
env-gated live proof (apply, exact revert, zero residue) before any
receipt, save path, or UI control is added. Guessing, name inference, user
config lines, and web snippets are not acceptable evidence.
