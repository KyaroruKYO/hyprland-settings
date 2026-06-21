# Runtime Approval Live-Restore Gate Review

Branch: `future-capability-marathon`

Starting commit: `e7f8b382e61aa9b3a0573e66dac432ac5d25bae8`

## Result

The proven `hl.config` eval live-restore proof is now connected to a default-disabled runtime approval review model.

The review consumes:

- exact setting id: `general:gaps_in`
- prior value: `5`
- temporary/proposed value: `6`
- mutation command: `hyprctl eval 'hl.config({ general = { gaps_in = 6 } })'`
- restore command: `hyprctl eval 'hl.config({ general = { gaps_in = 5 } })'`
- post-mutation readback: `css gap data: 6 6 6 6; set: true`
- post-restore readback: `css gap data: 5 5 5 5; set: true`
- explicit approval token evidence

Valid approval plus the proven live-restore evidence reaches `approved_but_default_disabled`.

The evidence is now projected into a disabled setting-detail UI surface:

- review widget: `hyprland-settings-runtime-approval-review-disabled`
- evidence widget: `hyprland-settings-runtime-live-restore-evidence`
- disabled planned action: `hyprland-settings-runtime-approval-enable-disabled`

The surface is display-only and does not call `hyprctl`.

## Safety

Production runtime/reload remains disabled.

No config file was edited.

No reload was run.

No dispatch command was run.

The only runtime mutation evidence remains the restored low-risk `general:gaps_in` proof from the previous sprint.

## Tests

- `runtime_live_restore_approval_review_consumes_proof_but_keeps_production_disabled`
- `runtime_mutation_syntax_evidence_records_proven_lua_config_restore_without_enabling_production`
- `runtime_live_restore_attempt_records_failed_mutation_syntax_without_enabling_production`
- `runtime_approval_evidence_projection_includes_proof_without_enabling_production`
- `runtime_approval_review_surface_displays_live_restore_evidence_and_stays_disabled`
- `runtime_approval_review_surface_is_called_from_detail_edit_section`

## Remaining Blockers

Production runtime activation still requires a separate explicit activation mechanism. Reload and dispatch require their own restore/recovery proof before they can move beyond default-disabled review.
