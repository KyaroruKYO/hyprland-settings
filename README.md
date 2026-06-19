# Hyprland Settings

Hyprland Settings is a Rust + GTK4/libadwaita settings app for Hyprland. It presents official Hyprland scalar settings with source-backed metadata, validation, and guarded safe-batch config writes for eligible normal scalar settings.

The current metadata and proof reports target Hyprland `0.55.2`.

## Current Status

Hyprland Settings currently models all 341 official scalar Hyprland settings for Hyprland `0.55.2`.

- 341 official scalar settings modeled
- 341 readable
- 341 writable
- 0 blocked
- Safe-batch writes are guarded by exact target-line checks, backup, reread verification, and recovery
- High-risk, display/render-risk, generated, script-managed, symlink/current-profile, duplicate, missing/default, structured-family, runtime, and profile/mode targets remain blocked from the safe-batch path
- Live runtime mutation/reload proof is not claimed

The proven claim is the current v0.55.2 app/export model: all 341 official scalar settings are modeled as readable and writable in the metadata pipeline. Production Apply remains narrower: it can write only eligible normal scalar settings through the guarded safe-batch path, and it blocks settings that need separate safety design. The project does not claim that all 341 settings have been safely live-mutated against an active Hyprland compositor.

## Safety Model

Normal scalar settings use source-backed validators and fixture write/reread proof before they are eligible for safe-batch writing.

High-risk and display/render-risk settings remain blocked from the safe-batch path until separate family-specific proof exists. Future gated paths would require proof such as:

- explicit high-risk approval metadata
- persisted recovery plan validation
- backup proof
- rollback and parser reread proof
- confirmation token proof
- timeout or no-confirmation rollback behavior
- UI warning or advanced placement
- production gate acceptance

`cursor.default_monitor` uses a runtime monitor-name oracle instead of generic freeform string validation. It accepts only monitor names proven by a current non-mutating oracle snapshot and rejects empty, missing, stale, unsafe, path-like, command-like, and malformed values.

`decoration.screen_shader` remains blocked from the normal safe-batch path and needs display/render-specific recovery proof before broader production write support. Advisory shader helper work is not treated as write-safety proof.

## What It Does

- Browses exported Hyprland setting metadata.
- Shows current scalar-row read/write coverage.
- Shows safety classifications and high-risk warning metadata.
- Validates pending values with source-backed or parser-backed validators.
- Uses guarded safe-batch config writes for eligible normal scalar settings.
- Blocks generated, script-managed, symlink/current-profile, duplicate, missing/default, structured-family, runtime, profile/mode, high-risk, and display/render-risk targets.

## What It Does Not Claim

- It does not claim live runtime mutation proof for every setting.
- It does not claim reload/eval testing against the active compositor.
- It does not run crash/debug proof against the active compositor.
- It does not add missing config lines yet.
- It does not auto-resolve duplicate settings.
- It does not switch profiles, change symlinks, reload Hyprland, or run mutating `hyprctl`.
- It does not migrate this v0.55.2 data/model to Hyprland 0.55.4.
- It is not an official Hyprland project and does not claim Hyprland upstream endorsement.

## Run From Source

```sh
cargo run --bin hyprland-settings
```

Run with an explicit metadata path:

```sh
cargo run --bin hyprland-settings -- data/exports/hyprland-0.55.2
```

Build and run the release binary:

```sh
cargo build --release
./target/release/hyprland-settings
```

## Validation

The current validation set includes:

```sh
cargo fmt --check
cargo check
cargo test
cargo build --release
desktop-file-validate data/applications/io.github.kyarorukyo.hyprlandsettings.desktop
appstreamcli validate --no-net data/metainfo/io.github.kyarorukyo.hyprlandsettings.metainfo.xml || true
```

The desktop and AppStream validators are optional local tools. The AppStream check may report non-blocking warnings until release metadata is finalized.

## Dependencies

Runtime dependencies:

- `gtk4`
- `libadwaita`
- `glib2`

Build dependencies:

- `rust`
- `cargo`
- `pkgconf`

## Current Limitations

- Packaging and release artifacts are not finalized.
- Missing/default insertion is intentionally blocked.
- Duplicate auto-resolution is intentionally blocked.
- High-risk/display-render writes are intentionally blocked from the normal safe-batch path.
- Structured-family writes and profile/mode switching are intentionally blocked.
- Live runtime mutation/reload proof remains a separate future safety milestone.
- The app's current proof model is guarded normal-scalar safe-batch config writing, not broad live compositor mutation coverage.

## Metadata Provenance

The bundled metadata targets Hyprland `0.55.2`. It is export-backed and validated before display.

AGS was used as a prototype/spec/export source during the transition to Rust. The Rust app does not require AGS at runtime, and no live user config is included in the metadata bundle.
