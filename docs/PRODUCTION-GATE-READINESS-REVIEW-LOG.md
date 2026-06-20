# Production Gate Readiness Review Log

## Sprint summary
- Branch: future-capability-marathon
- Starting commit: 35c96c4374e56327cc67224060d10227eda54a40
- Project data/model: v0.55.2
- Counts preserved: 341 readable / 341 writable / 0 blocked
- Real config touched: no
- Runtime touched: yes, controlled `general:gaps_in` mutation was restored immediately
- main modified: no
- v0.1.0 tag modified: no
- dist/v0.1.0 modified: no

## Default-disabled production gate review implemented
- Source/include selected-target insertion: copied-config-tree proof can reach `ReadyButDefaultDisabled`.
- Duplicate occurrence replacement: copied-config-tree proof plus confirmed occurrence can reach `ReadyButDefaultDisabled`.
- Structured `hl.bind` exact-line replacement: copied-config-tree proof plus candidate validation can reach `ReadyButDefaultDisabled`.
- Profile/mode switching: copied symlink proof can reach `ReadyButDefaultDisabled`, but real-session proof is still required.
- Runtime/reload mutation: gate has read-only evidence, prior snapshot, restore command, and proven low-risk live restore for `general:gaps_in`; production remains default-disabled.
- Runtime approval UI: setting detail now displays the proven live-restore evidence in a disabled review surface with no runtime handler and an insensitive planned enable action.
- High-risk/display writes: gate exists and blocks without out-of-band recovery, dead-man timeout, restore command, config backup, runtime snapshot, and approval.
- Hyprland 0.55.4 activation: gate exists and blocks advisory-only evidence without official exports, row diff, write-safety review, safe-env evidence, and approval.
- Disabled approval UI cards: Config page now shows review-only cards for source/include insertion, duplicate replacement, structured `hl.bind`, profile/mode switching, high-risk/display writes, and Hyprland 0.55.4 migration. Every planned enable action is insensitive and no card has a mutation handler.

## Explicit approval flow implemented
- Approval requests now name the exact scope, exact target path or runtime command, old state, proposed state, restore plan, one-shot/expiry behavior, and copied-config-tree or live-restore proof.
- Source/include, duplicate, structured `hl.bind`, and profile/mode approvals can reach `ApprovedButDefaultDisabled` from copied-config-tree proof.
- Runtime keyword approval can reach `ReadyButDefaultDisabled` in model tests when live-restore proof exists. Real read-only evidence and low-risk live restore proof now both succeed, but production remains disabled by default.
- High-risk/display and Hyprland 0.55.4 approvals remain blocked unless their recovery/trusted-data evidence is complete.
- Approval never flips production behavior on by default.

## Not ready for production activation
- High-risk/display writes: no out-of-band recovery proof.
- Real profile/mode switching: no live symlink proof against the real session.
- Runtime/reload mutation: sandbox socket access is blocked by `Operation not permitted`; outside-sandbox read-only evidence succeeds, and `hl.config` eval live restore is proven for `general:gaps_in`. Production activation still requires explicit runtime approval gates.
- Hyprland 0.55.4 migration: official exports, row-count diff, write-safety review, safe-env evidence, and explicit approval are missing.

## Required gate behavior
- Default disabled.
- Explicit target/occurrence/line selection.
- Exact old line and new line review.
- Copied-config-tree proof linked in review.
- Backup and restore plan visible before any live test.
- No real config or runtime mutation without explicit approval.

## Read-only runtime evidence
- Sandboxed direct socket connect: failed, `Operation not permitted`.
- Outside-sandbox `hyprctl version`: succeeded, Hyprland 0.55.4 commit `a0136d8c04687bb36eb8a28eb9d1ff92aea99704`.
- Outside-sandbox `hyprctl monitors -j`: succeeded.
- Outside-sandbox `hyprctl getoption general:gaps_in`: `css gap data: 5 5 5 5`.
- Outside-sandbox `hyprctl getoption general:gaps_out`: `css gap data: 10 10 10 10`.
- Outside-sandbox `hyprctl getoption decoration:blur:enabled`: `bool: true`.
- Outside-sandbox `hyprctl getoption misc:disable_hyprland_logo`: `bool: true`.
- Controlled `hyprctl keyword general:gaps_in 6`: failed before value change because non-legacy parsers require eval.
- Controlled `hyprctl eval 'general:gaps_in = 6'`: failed before value change with parser syntax error.
- Controlled `hyprctl eval 'hl.config({ general = { gaps_in = 6 } })'`: succeeded.
- Post-mutation readback: `css gap data: 6 6 6 6`.
- Restore command: `hyprctl eval 'hl.config({ general = { gaps_in = 5 } })'`.
- Post-restore readback: `css gap data: 5 5 5 5`.
- `pacman -Q hyprland`: `hyprland 0.55.4-1`.

## Next exact work
Add deeper per-card approval data fed by live or copied proof records, still keeping all production behavior default-disabled.
