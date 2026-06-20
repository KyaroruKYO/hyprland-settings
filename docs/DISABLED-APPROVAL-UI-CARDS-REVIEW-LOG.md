# Disabled Approval UI Cards Review

Branch: `future-capability-marathon`

Starting commit: `ce41772a62605c3291e69feed5a19fb80f9c589d`

## Result

The Config page now includes a disabled future approval review section with review-only cards for:

- source/include insertion
- duplicate replacement
- structured `hl.bind` writes
- profile/mode switching
- high-risk/display writes
- Hyprland 0.55.4 migration

Each card displays structured proof data, production-disabled status, blockers, and an insensitive planned enable action.

## Deep Data

The approval cards now render:

- proof source and proof status
- proof-backed fields such as copied target, source depth, copied replacement/edit/switch status, runtime evidence, package evidence, or trusted-data state
- preconditions such as selected target, fingerprint/raw-line matching, candidate validation, recovery requirements, or official export requirements
- restore or unchanged evidence, including copied target restore, real config unchanged, real symlink untouched, high-risk restoration missing, and v0.55.2 migration preservation

The deep data is still review-only projection data. It does not enable production Apply, production profile switching, runtime mutation, high-risk writes, or migration activation.

## Safety

No card has a mutation handler.

No production behavior was enabled.

No runtime mutation was run in this sprint.

No config file was edited.

No reload was run.

The v0.55.2 app model remains active and Hyprland 0.55.4 migration remains inactive.

## Widgets

- `hyprland-settings-source-include-approval-review-disabled`
- `hyprland-settings-duplicate-approval-review-disabled`
- `hyprland-settings-structured-approval-review-disabled`
- `hyprland-settings-profile-approval-review-disabled`
- `hyprland-settings-high-risk-approval-review-disabled`
- `hyprland-settings-0554-approval-review-disabled`

## Tests

- `disabled_future_approval_card_projections_cover_all_remaining_gates_without_enablement`
- `disabled_future_approval_cards_are_visible_and_non_mutating`
- `deep_approval_card_data_report_records_proof_backed_fields_without_enablement`

## Next

Feed disabled approval cards from serialized proof records or report data, then add screenshot-level assertions for each card while production behavior remains disabled.
# 2026-06-20 Report-Backed Card Data

- Disabled approval cards now load through a typed serialized report adapter from `data/reports/disabled-approval-ui-cards.v0.55.2.json`.
- The adapter preserves stable widget names and disabled planned actions while deriving proof source, proof status, proof fields, preconditions, restore evidence, production status, active model, and migration status from report records.
- Missing or unavailable serialized fields render explicit `Missing from report` / `Report unavailable` copy.
- GTK safe-env screenshot-level assertions now cover all six approval cards through screenshot capture plus AT-SPI accessibility-tree text; no OCR, clicking, runtime mutation, reload, or production activation is involved.
