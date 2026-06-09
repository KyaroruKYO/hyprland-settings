# Hyprland Settings

Hyprland Settings is a Rust + GTK4/libadwaita settings app for Hyprland. It presents official Hyprland scalar settings with source-backed metadata, validation, and a config-write model designed around explicit safety gates.

The current metadata and proof reports target Hyprland `0.55.2`.

## Current Status

Hyprland Settings currently models all 341 official scalar Hyprland settings for Hyprland `0.55.2`.

- 341 official scalar settings modeled
- 341 readable
- 341 writable
- 0 blocked
- High-risk settings require gated recovery and confirmation paths
- Live runtime mutation/reload proof is not claimed

The proven claim is config-write coverage: all 341 official scalar settings are readable and writable through the app's config-write or gated high-risk write model. The project does not claim that all 341 settings have been safely live-mutated against an active Hyprland compositor.

## Safety Model

Low-risk settings use source-backed validators and fixture write/reread proof before they are writable.

High-risk settings are writable only through gated paths. Those paths require proof such as:

- explicit high-risk approval metadata
- persisted recovery plan validation
- backup proof
- rollback and parser reread proof
- confirmation token proof
- timeout or no-confirmation rollback behavior
- UI warning or advanced placement
- production gate acceptance

`cursor.default_monitor` uses a runtime monitor-name oracle instead of generic freeform string validation. It accepts only monitor names proven by a current non-mutating oracle snapshot and rejects empty, missing, stale, unsafe, path-like, command-like, and malformed values.

`decoration.screen_shader` remains writable behind its production screen-shader gate. Advisory shader helper work is not treated as write-safety proof.

## What It Does

- Browses exported Hyprland setting metadata.
- Shows current scalar-row read/write coverage.
- Shows safety classifications and high-risk warning metadata.
- Validates pending values with source-backed or parser-backed validators.
- Models safe config writes through fixture-tested write/reread and recovery paths.

## What It Does Not Claim

- It does not claim live runtime mutation proof for every setting.
- It does not claim reload/eval testing against the active compositor.
- It does not run crash/debug proof against the active compositor.
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
appstreamcli validate --pedantic data/metainfo/io.github.kyarorukyo.hyprlandsettings.metainfo.xml || true
python /home/kyo/.config/hypr/ags/validate-hyprland-settings-export-v0552.py
python ~/.config/hypr/ags/validate-settings-ui-design-draft.py
python ~/.config/hypr/ags/validate-schema-draft.py
```

The AppStream check currently has expected non-blocking warnings for the GitHub URL and missing releases info.

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
- Live runtime mutation/reload proof remains a separate future safety milestone.
- The app's current proof model is persistent config-write and gated high-risk write coverage, not broad live compositor mutation coverage.

## Metadata Provenance

The bundled metadata targets Hyprland `0.55.2`. It is export-backed and validated before display.

AGS was used as a prototype/spec/export source during the transition to Rust. The Rust app does not require AGS at runtime, and no live user config is included in the metadata bundle.
