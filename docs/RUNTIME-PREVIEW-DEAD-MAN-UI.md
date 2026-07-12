# Runtime Preview Dead-Man UI

Supervised dead-man live preview is implemented: an explicitly armed, countdown-guarded runtime mutation that reverts automatically unless the user confirms "Keep changes" in time. It is enabled only where the runtime mechanism and revert path are proven — not blindly for all 78 dead-man rows.

## Honest reclassification of the 78 rows

| Classification | Count | What it means in the UI |
| --- | --- | --- |
| DeadManPreviewCandidate | 38 | armed "Preview with recovery" button: the two animation toggles plus 36 input/cursor rows promoted by passed per-row live proofs (receipts in `PROVEN_INPUT_ROWS`) |
| DeadManPreviewCandidateNeedsLiveProof | 27 | supervised panel rendered disarmed with proof-aware status: 18 touch-family rows need the hardware present, 3 rows need secondary-device proofs, and the rest carry specific evidence-based reasons (see `docs/RUNTIME-PREVIEW-INPUT-CURSOR-LIVE-PROOF.md`) |
| DeadManPreviewModelOnly | 5 | panel disarmed: source-backed string values are unproven as runtime sets for input devices |
| DeadManPreviewBlockedNoSafeRuntimeMechanism | 8 | blocked reason only: vector/list, monitor-name, and path grammars have no proven runtime representation |
| RequiresRelog / RequiresRestart / NoVisibleEffect / TooDangerous | 0 | no row was proven into these buckets; nothing was faked |

Monitor/display/render rows are **not** in this set: they remain `BlockedHighRisk` in the capability matrix, have no dead-man UI state at all (test-enforced), and show only their blocked reason. Any future monitor preview would require identity capture, blind auto-revert, and its own gates — none of which exist yet.

## The supervised flow

1. The detail pane shows the "Dead-man preview required" badge, why supervision is needed, the auto-revert warning with the 10-second countdown, and the recovery instruction.
2. The user clicks **Preview with recovery** (enabled only for candidates). The controller arms: the original runtime value is captured read-only.
3. The entered value applies through the executor's supervised path and the countdown starts, ticking visibly once per second.
4. **Keep changes** stops the countdown and keeps the runtime value (still session-only — saving is a separate step). **Revert now** restores the original immediately, even after Keep. **Cancel** restores and disarms.
5. If the countdown expires unconfirmed, the timeout **auto-reverts** the original value without user action.
6. Session-drop (navigating away) and app-close revert unconfirmed previews; explicitly Kept previews are left in place.

Execution path: UI action → dead-man panel → `RuntimePreviewDeadManController` → supervised session/apply/revert in `runtime_preview_executor` → runner. UI code builds no commands, writes no files, and never reloads.

## Proof

- 10 model tests: full-flow arm/apply/confirm/revert, timeout auto-revert, cancel and session-drop semantics, out-of-phase rejection, per-classification arm gating, monitor/display exclusion, source guards, classification report generation, and normal-preview invariance (341/135 unchanged).
- One env-gated live proof (run once): `animations.enabled` was toggled live under supervision, the countdown expired unconfirmed, and the automatic revert restored the original value.
- GTK evidence (`/tmp/hyprland-settings-gtk-automation/20260712_005942`): the armed panel with all its copy on the Animations detail pane; blocked rows without panels; all safety flags false.

## What remains gated

The 63 needs-live-proof input/cursor rows stay disarmed until each has a supervised, externally recoverable live proof. Model-only and blocked rows stay disarmed/blocked. Hyprland reload remains disabled everywhere.
