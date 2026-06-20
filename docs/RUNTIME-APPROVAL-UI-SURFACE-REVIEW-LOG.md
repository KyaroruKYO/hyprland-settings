# Runtime Approval UI Surface Review

Branch: `future-capability-marathon`

Starting commit: `59a95142aed8714c72e6871d87363765d08cea51`

## Result

The setting detail pane now includes a disabled runtime approval review surface for the proven `general:gaps_in` runtime path.

The surface displays:

- setting: `general:gaps_in`
- prior value: `5`
- temporary value: `6`
- mutation command: `hyprctl eval 'hl.config({ general = { gaps_in = 6 } })'`
- restore command: `hyprctl eval 'hl.config({ general = { gaps_in = 5 } })'`
- post-mutation readback: `css gap data: 6 6 6 6; set: true`
- post-restore readback: `css gap data: 5 5 5 5; set: true`
- approval status: `Approved but default-disabled`
- production runtime/reload: `Disabled`

## Widgets

- `hyprland-settings-runtime-approval-review-disabled`
- `hyprland-settings-runtime-live-restore-evidence`
- `hyprland-settings-runtime-approval-enable-disabled`

The planned enable control is insensitive and has no runtime handler.

## Safety

No runtime mutation was run in this sprint.

No config file was edited.

No reload was run.

Production runtime/reload remains disabled.

## Tests

- `runtime_approval_evidence_projection_includes_proof_without_enabling_production`
- `runtime_approval_review_surface_displays_live_restore_evidence_and_stays_disabled`
- `runtime_approval_review_surface_is_called_from_detail_edit_section`

## Next

The same disabled approval surface pattern now exists for source/include insertion, duplicate replacement, structured `hl.bind`, profile/mode switching, high-risk/display writes, and Hyprland 0.55.4 migration. See `docs/DISABLED-APPROVAL-UI-CARDS-REVIEW-LOG.md`.
