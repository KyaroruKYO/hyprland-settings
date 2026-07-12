# Runtime Preview Input/Cursor Live Proof

The 63 input/cursor rows that needed per-row live proof now have a complete proof architecture: row-specific proof plans, fallback requirements, an env-gated live-proof harness, and receipt-gated promotion. One proof ran and passed this sprint; 62 rows remain honestly disarmed.

## Why input rows are different

For input/cursor settings the failure mode is not visual discomfort — it is losing the ability to click, type, focus, or confirm. Every proof plan therefore assumes the user might lose one input path and names the fallback that must hold: cursor rows leave both devices working; keyboard rows keep the pointer usable; pointer rows keep the keyboard usable; touch-family rows leave external devices alone; and for every row the dead-man timeout auto-revert fires without any input at all, with a TTY rollback instruction as the last resort.

## The 63 plans (generated in `src/runtime_preview_input_proof.rs`, exported to `runtime-preview-input-cursor-proof-plan.v0.55.2.json`)

Each plan answers: what the setting controls, what could go wrong live, which subsystems it touches (keyboard/pointer/focus), the fallback path, the original-value capture requirement, the minimal preview value, apply/revert/verification strategies, the manual warning, the per-row env-gated proof command, and the promotion decision.

| Proof classification | Count | Notes |
| --- | --- | --- |
| NeedsLiveProofCursor | 15 | cursor warp/visibility/zoom behavior |
| NeedsLiveProofInputPointer | 11 | speed, handedness, scrolling |
| NeedsLiveProofInputTouchpad | 18 | touchpad + tablet + touch-device rows (needs hardware to verify) |
| NeedsLiveProofFocus | 7 | follow-mouse and focus-jump behavior |
| NeedsLiveProofInputKeyboard | 6 | repeat, numlock, bind resolution, virtual keyboards |
| ProofBlockedNoRuntimeVerification | 4 | cursor rendering pipeline rows (`no_hardware_cursors`, `use_cpu_buffer`, `min_refresh_rate`, `no_break_fs_vrr`) — a glitched cursor cannot be detected by the app |
| ProofBlockedTooDangerous | 1 | `cursor.invisible` — the preview value itself removes cursor visibility |
| ProofPassedArmableCandidate | 1 | `cursor.inactive_timeout` — proof ran and passed |

## The proof that ran

`cursor.inactive_timeout` is the safest possible input-family row: it only changes how quickly the idle cursor hides; keyboard and pointer events keep flowing regardless. The env-gated harness (`HYPRLAND_SETTINGS_RUN_INPUT_LIVE_PROOF=1 HYPRLAND_SETTINGS_INPUT_PROOF_ROW=cursor.inactive_timeout`) captured the original (`0.000000`), applied the minimal preview value (`1`), verified it live (`1.000000` observed via read-only getoption), reverted automatically, and verified the exact original was restored. The receipt is recorded in `PROVEN_INPUT_ROWS`, which is the **only** mechanism that promotes a row: the dead-man layer arms input/cursor rows solely on a recorded receipt (test-enforced), and the promoted row's panel states that its proof passed.

## The harness fails closed

It refuses to run without both env vars; refuses unknown rows, rows outside the proof set, rows without a usable fallback, rows not classified proof-ready — and refuses already-proven rows so a passed proof cannot be accidentally re-run. It never writes config, never reloads, never persists, and reverts automatically regardless of the verification outcome.

## UI

Disarmed input/cursor rows now show proof-aware status in the supervised panel: the risk subsystem, the fallback requirement, the specific proof classification, and either the row's env-gated proof command or what a proof would need to demonstrate. The promoted row arms with its proof provenance displayed. Proven via GTK/AT-SPI evidence with all safety flags false.

## Promoting the next rows

One row at a time: pick the safest remaining cursor rows first (`hide_on_key_press`, `no_warps`, `hotspot_padding`), classify each `ProofReadyForEnvGatedLiveTest` with a concrete minimal preview value, run its env-gated proof, and record the receipt only if apply and revert both verify. Touch-family rows additionally need the hardware present to verify behavior.
