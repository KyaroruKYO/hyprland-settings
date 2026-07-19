# Hermetic Test Stabilization

## Default-suite contract

Normal `cargo test` runs are machine-independent. They use committed fixtures, temporary directories, explicit `ConfigDiscoveryEnv` values, mock runtime runners, and deterministic generated data.

The default suite must not:

- discover or read the real `~/.config/hypr` tree;
- require `HYPRLAND_INSTANCE_SIGNATURE` or a running compositor;
- execute `hyprctl`;
- write active config or mutate runtime;
- rewrite tracked reports;
- depend on the current user's setting count or config shape.

## Real-machine audit boundary

Read-only real-config audits are `#[ignore]` and additionally require:

```text
HYPRLAND_SETTINGS_RUN_REAL_CONFIG_AUDIT=1
```

Live runtime and active-config write proofs retain their own ignored, explicit environment gates. They are not part of normal validation.

## Report boundary

Normal report tests serialize to a test-owned temporary path and compare deterministic values with committed artifacts. They do not update `data/reports/`.

Intentional expected-report regeneration separately requires:

```text
HYPRLAND_SETTINGS_REGENERATE_REPORTS=1
```

## Verification procedure

Final stabilization validation runs the complete suite twice and compares `git status --short` before and after each run. It then runs the suite with isolated `HOME`, `XDG_CONFIG_HOME`, and `XDG_STATE_HOME`, and a marker `hyprctl` shim at the front of `PATH`. The marker must remain absent.

The isolated environment is a test input only; Codex and all project work still run in the normal real user session.

## Final evidence

On 2026-07-18, two normal full-suite runs and one isolated HOME/XDG run each
completed with 1,154 passed, zero failed, and 25 ignored tests. Before/after
comparisons showed unchanged `git status`, tracked diff hash, and checksums for
every JSON report on all three runs. The isolated run removed the Hyprland
session variables and placed a failing marker `hyprctl` first in `PATH`; that
marker was never invoked.
