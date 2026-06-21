# Runtime Live Restore Proof Review Log

## Read-Only Retry
- Shell: `/usr/bin/zsh`
- User: `kyo`
- `XDG_RUNTIME_DIR`: `/run/user/1000`
- `HYPRLAND_INSTANCE_SIGNATURE`: `a0136d8c04687bb36eb8a28eb9d1ff92aea99704_1781857006_1638495299`
- `hyprctl`: `/usr/bin/hyprctl`
- Socket directory: observed under `$XDG_RUNTIME_DIR/hypr`

## Result
Read-only `hyprctl` queries succeeded outside the sandbox. `general:gaps_in` prior value was parsed as `5`, and the planned temporary value was `6`.

## Controlled Attempt
- `hyprctl keyword general:gaps_in 6`: failed before value change with `keyword can't work with non-legacy parsers. Use eval.`
- `hyprctl eval 'general:gaps_in = 6'`: failed before value change with a parser syntax error.
- Restore command prepared before successful mutation: `hyprctl eval 'hl.config({ general = { gaps_in = 5 } })'`
- Mutation command: `hyprctl eval 'hl.config({ general = { gaps_in = 6 } })'`
- Mutation output: `ok`
- Post-mutation readback: `css gap data: 6 6 6 6`
- Restore output: `ok`
- Post-restore readback: `css gap data: 5 5 5 5`
- Runtime restored to prior value: yes

## Decision
The preferred `general:gaps_in` live-restore proof is now proven for the low-risk `hl.config` eval path. Production runtime mutation remains disabled.

## Model Work
`RuntimeLiveRestoreProof` now records read-only evidence, prior value, temporary value, restore command, mutation command failure/success, post-mutation readback, post-restore readback, restoration status, and production-disabled state. Tests prove failed read-only evidence blocks mutation, failed mutation syntax does not enable production, and a complete restore proof can reach `LiveRestoreProven` without enabling production.

## UI Evidence
The proof is now displayed in a disabled setting-detail runtime approval surface. The surface shows the exact mutation and restore commands, readbacks, approved-but-default-disabled status, and disabled production runtime/reload state. It has no runtime handler and does not enable production Apply.
