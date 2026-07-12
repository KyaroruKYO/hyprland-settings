# Runtime Preview Capability Matrix

Every one of the 341 scalar settings and all 7 structured families is classified for live runtime preview in `data/reports/runtime-preview-capability-matrix.v0.55.2.json`, generated deterministically from `src/runtime_preview.rs` (the generator test regenerates it on every run, so the checked-in file always matches the code).

## Mechanism and evidence

The runtime path is `hyprctl eval 'hl.config({ <section> = { <option> = <value> } })'`, proven live in this project for `general.gaps_in` — including, this sprint, a full session round trip: capture original read-only, apply preview, verify via `getoption`, revert, verify restoration. The live proof also established the `css_gap` grammar (integer or `top/right/bottom/left` table) and that `hyprctl eval` reports errors on stdout with a zero exit status; both are now handled and tested. Rows are marked supported only where this scalar mechanism, a runtime-safe value grammar, and a low-risk classification hold together. Nothing else is guessed: unproven rows are `NotProvenYet`.

## Scalar results (341 rows)

| Classification | Count | Meaning |
| --- | --- | --- |
| LivePreviewSupported | 62 | low-risk visual/layout toggles and choices; instant live preview |
| LivePreviewSupportedWithThrottle | 73 | low-risk continuous values (gaps, borders, rounding, colors, opacity, blur, shadow); throttled to one runtime set per 150 ms |
| LivePreviewSupportedWithDeadMan | 78 | input/cursor/animation rows; preview only inside a confirmed dead-man session, disabled by default |
| RequiresConfigWrite | 43 | behavioral or string/path/regex grammar rows; persist through the config write path |
| BlockedHighRisk | 74 | display/monitor/render/shader, exec/script, env/session, window-rule rows |
| NotProvenYet | 11 | debug/ecosystem/experimental sections with no runtime-safety evidence |
| RequiresReload / RequiresRelog / RequiresRestart | 0 | no row was proven to require these; unproven rows are honestly `NotProvenYet` instead |

135 rows (~40%) are live-previewable by default today.

## Structured families

| Family | Classification | Risk | Why |
| --- | --- | --- | --- |
| hl.monitor | BlockedHighRisk | HighRiskDisplay | monitor records can disable displays |
| hl.bind | BlockedHighRisk | HighRiskInput | can lock the user out |
| hl.animation | NotProvenYet | LowRiskVisual | visually safe, but no runtime record-application mechanism proven |
| hl.curve | NotProvenYet | LowRiskVisual | inert records, same missing mechanism |
| hl.gesture | BlockedStructuredFamilySemantics | MediumRiskBehavior | record semantics unsafe mid-session without proof |
| hl.device | BlockedHighRisk | HighRiskInput | reconfigures input hardware |
| hl.permission | BlockedHighRisk | HighRiskSecurity | security policy is never live previewed |

## Preview lifecycle

Start (read-only original-value capture) → throttled applies (latest pending value only, no config writes, no reload) → Save (marks session; persistence happens once through the existing backup/write/reread path) or Cancel/Revert (reapplies the captured original and verifies). High-risk rows additionally require a confirmed dead-man session that reverts on timeout and records a recovery instruction.
