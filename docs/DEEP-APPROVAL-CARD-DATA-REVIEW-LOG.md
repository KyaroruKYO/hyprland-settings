# Deep Approval Card Data Review

Branch: `future-capability-marathon`

Starting commit: `d23af73a07c7edcf541a17ad297dcd3db53c18fb`

## Result

The Config-page disabled approval cards now use structured proof data instead of broad summary-only copy.

Implemented deep card data for:

- source/include insertion: copied-config-tree proof, selected target, source depth, dry-run status, planned line, copied restore, original unchanged
- duplicate replacement: copied-config-tree proof, selected occurrence, target, source depth, line/raw-line/old-value preconditions, copied replacement status, copied restore, original unchanged
- structured `hl.bind`: copied-config-tree proof, target/source depth, exact old/new raw lines, candidate validation, comment/order preservation, copied edit status, copied restore, original unchanged
- profile/mode: copied symlink proof, current symlink, original target, proposed target, copied switch status, copied restore, real symlink untouched
- high-risk/display: readiness proof, runtime read-only evidence, low-risk runtime live restore, explicit insufficiency reason, and missing recovery/dead-man/restore/backup/snapshot blockers
- Hyprland 0.55.4 migration: runtime/package evidence, active v0.55.2 model, advisory-only status, missing official exports, row diff, write-safety review, safe-env evidence, approval, and inactive migration status

## Safety

All cards remain disabled and review-only.

No runtime mutation was run in this sprint.

No real config was touched.

No reload was run.

No production source/include insertion, duplicate write, structured write, profile switch, high-risk write, runtime/reload mutation, or 0.55.4 migration activation was enabled.

## Reports

- `data/reports/deep-approval-card-data.v0.55.2.json`
- `data/reports/disabled-approval-ui-cards.v0.55.2.json`

## Tests

- `disabled_future_approval_card_projections_cover_all_remaining_gates_without_enablement`
- `disabled_future_approval_cards_are_visible_and_non_mutating`
- `deep_approval_card_data_report_records_proof_backed_fields_without_enablement`

## Next

Feed disabled approval cards from serialized proof records or report data, then add screenshot-level assertions for each card while production behavior remains disabled.
