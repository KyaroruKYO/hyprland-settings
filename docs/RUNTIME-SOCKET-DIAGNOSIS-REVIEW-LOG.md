# Runtime Socket Diagnosis Review Log

## Result
- Sandbox diagnosis: the Hyprland socket path was visible, but direct Unix socket connection failed with `Operation not permitted`, and `hyprctl` reported `Couldn't set socket timeout (2)`.
- Real-session diagnosis: running the same read-only checks outside the sandbox succeeded.
- Root cause: sandbox process/socket isolation, not a stale signature or missing Hyprland process.

## Working Runtime Evidence
- Hyprland process: `811 Hyprland --watchdog-fd 4`
- Signature: `a0136d8c04687bb36eb8a28eb9d1ff92aea99704_1781857006_1638495299`
- Socket: `/run/user/1000/hypr/a0136d8c04687bb36eb8a28eb9d1ff92aea99704_1781857006_1638495299/.socket.sock`
- `hyprctl version`: succeeded, Hyprland 0.55.4 commit `a0136d8c04687bb36eb8a28eb9d1ff92aea99704`
- `hyprctl monitors -j`: succeeded
- `hyprctl getoption general:gaps_in`: `css gap data: 5 5 5 5`
- `hyprctl getoption general:gaps_out`: `css gap data: 10 10 10 10`
- `hyprctl getoption decoration:blur:enabled`: `bool: true`
- `hyprctl getoption misc:disable_hyprland_logo`: `bool: true`

## Controlled Mutation Attempt
- Setting: `general:gaps_in`
- Prior value: `5`
- Temporary value: `6`
- Restore command prepared before successful mutation: `hyprctl eval 'hl.config({ general = { gaps_in = 5 } })'`
- `hyprctl keyword general:gaps_in 6`: failed before value change because non-legacy parsers require eval.
- `hyprctl eval 'general:gaps_in = 6'`: failed before value change with parser syntax error.
- `hyprctl eval 'hl.config({ general = { gaps_in = 6 } })'`: succeeded.
- Post-mutation readback: `css gap data: 6 6 6 6`
- Restore output: `ok`
- Post-restore readback: `css gap data: 5 5 5 5`
- Restoration verified: yes.

## Gate Decision
Runtime/reload production mutation remains disabled. Read-only evidence is available and the low-risk live restore proof is proven, but production runtime activation still requires explicit approval gates and must remain default-disabled.
