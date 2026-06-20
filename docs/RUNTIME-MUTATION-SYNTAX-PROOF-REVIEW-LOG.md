# Runtime Mutation Syntax Proof Review Log

## Outside-Sandbox Evidence
- All runtime commands in this sprint were run outside the sandbox.
- `hyprctl version`, `hyprctl monitors -j`, and the required `getoption` reads succeeded.
- Prior `general:gaps_in` value: `5`.
- Temporary proof value: `6`.

## Syntax Inspection
- `hyprctl keyword general:gaps_in 6` is rejected by Hyprland 0.55.4 with `keyword can't work with non-legacy parsers. Use eval.`
- `hyprctl eval 'general:gaps_in = 6'` fails before value change with a parser error.
- The active Lua config uses `hl.config({ general = { gaps_in = 5 } })`.

## Controlled Live Restore Proof
- Mutation command prepared: `hyprctl eval 'hl.config({ general = { gaps_in = 6 } })'`
- Restore command prepared before mutation: `hyprctl eval 'hl.config({ general = { gaps_in = 5 } })'`
- Mutation output: `ok`.
- Post-mutation readback: `css gap data: 6 6 6 6`.
- Restore output: `ok`.
- Post-restore readback: `css gap data: 5 5 5 5`.
- Runtime was restored to the prior value.
- No config files were edited.
- No reload was run.

## Gate Decision
Runtime live restore is proven for this low-risk `general:gaps_in` keyword path, but production runtime/reload remains disabled. The proof is readiness evidence for a future default-disabled runtime approval flow; it is not production activation.
