# Runtime Read-Only Evidence Review Log

## Commands Run
- `command -v hyprctl`: `/usr/bin/hyprctl`
- `echo "$HYPRLAND_INSTANCE_SIGNATURE"`: `a0136d8c04687bb36eb8a28eb9d1ff92aea99704_1781857006_1638495299`
- `ls -la /tmp/hypr`: missing
- `ls -la "$XDG_RUNTIME_DIR/hypr"`: socket directory exists for the current signature
- Sandboxed `hyprctl version`: failed without mutation, `Couldn't set socket timeout (2)`
- Sandboxed direct socket connect: failed, `Operation not permitted`
- Outside-sandbox `hyprctl version`: succeeded, Hyprland 0.55.4 commit `a0136d8c04687bb36eb8a28eb9d1ff92aea99704`
- Outside-sandbox `hyprctl monitors -j`: succeeded
- Outside-sandbox `hyprctl getoption general:gaps_in`: `css gap data: 5 5 5 5`
- Outside-sandbox `hyprctl getoption general:gaps_out`: `css gap data: 10 10 10 10`
- Outside-sandbox `hyprctl getoption decoration:blur:enabled`: `bool: true`
- Outside-sandbox `hyprctl getoption misc:disable_hyprland_logo`: `bool: true`

## Result
The previous socket timeout is explained by sandbox socket/process isolation. In the real session, read-only `hyprctl` evidence succeeds.

## Gate Decision
Runtime/reload production mutation remains disabled. Read-only evidence is available, and the low-risk `general:gaps_in` live-restore proof now succeeds through `hyprctl eval 'hl.config({ general = { gaps_in = VALUE } })'`.
