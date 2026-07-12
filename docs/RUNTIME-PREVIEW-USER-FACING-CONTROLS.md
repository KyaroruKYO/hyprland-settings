# Runtime Preview User-Facing Controls

Per-setting live preview controls are now wired into the settings detail pane for all 135 default-previewable scalar rows. The app feels alive: move a slider, flip a switch, or enter a color, and the running Hyprland session updates immediately — reversibly, and without a single config write.

## What the user sees

Every setting's detail pane has a **Live preview** section:

- A capability badge: "Live Preview: Supported", "Live Preview: Supported with throttle", "Dead-man preview required", "Preview blocked: high-risk setting", "Preview unavailable: persists through config write", or "Preview unavailable: not proven yet".
- For the 135 supported rows, the right control for the value type: **53 switches** (booleans), **45 sliders** (bounded numerics and percents), **3 spin rows** (unbounded numerics), **22 color entries**, **3 value entries** (css-gap-style grammars), **9 dropdowns** (enum-like choices).
- A live status line that reads "Previewing Live: <setting> = <value> (original <value>)" once a preview is active.
- **Save previewed value** — persists the final value exactly once through the app's existing safe scalar apply flow (backup, write, reread verification). Enabled only when that flow's review gates are open; otherwise disabled with the reason shown.
- **Revert preview** — restores the runtime value captured when the session started.
- **Cancel preview** — reverts and clears the session.

Unsupported, blocked, dead-man, and not-proven rows show their classification and the reason; they expose no enabled preview control.

## How it works underneath

The GTK layer renders `RuntimePreviewUiRowState` projections and calls `RuntimePreviewUiController` actions (`src/runtime_preview_ui_projection.rs`). The controller owns the session and the trailing-edge throttle and routes everything through the runtime preview executor. UI code contains no `hyprctl` strings, no command construction, no executor calls, and no file-write APIs — all test-enforced.

- **Session start** happens on the first interaction and is a read-only `getoption` capture of the original value. Rendering a detail pane mutates nothing.
- **Slider drags** are throttled: offers within the row's interval keep only the latest pending value, and one trailing drain timer applies it — at most one runtime set per interval, never a config write per tick.
- **Session-drop and app-close recovery**: controllers register in a window-level registry; re-rendering the detail pane or closing the window reverts any still-active preview with an applied value.

## Proof

- 8 model tests: all 341 rows project, exactly 135 enabled, mapping determinism, action routing through the executor, throttle behavior, fail-closed rejection of unsupported/blocked/dead-man rows, app-close revert semantics, and source guards.
- One env-gated live smoke (run once): the controller drove a throttled drag on `general.gaps_in` against the live compositor, verified the change, cancelled, and verified restoration.
- GTK evidence (`/tmp/hyprland-settings-gtk-automation/20260712_001609`): a supported row shows the badge and Save/Revert/Cancel controls; a high-risk row shows "Dead-man preview required" with no enabled control; all safety flags false.

## Still gated

Dead-man rows now have the supervised countdown/confirm/auto-revert UI (`docs/RUNTIME-PREVIEW-DEAD-MAN-UI.md`): the two proven animation candidates arm, and the remaining 76 stay disarmed or blocked with their specific reason until per-row live proofs exist. Structured families show their honest classification and have no preview controls. Hyprland reload remains disabled everywhere.
