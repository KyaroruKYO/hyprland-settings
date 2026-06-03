# Hyprland Settings

Hyprland Settings is a read-only GTK/libadwaita browser for Hyprland settings metadata.

The current metadata bundle targets Hyprland `0.55.2`.

## Status

Hyprland Settings is currently read-only.

- It does not read live Hyprland config files.
- It does not read current values.
- It does not change settings.
- It does not run Hyprland commands.
- It does not require AGS at runtime.
- It does not require the Python export generator at runtime.

Write support is not implemented yet. Future write support is planned only after a Rust-native write safety architecture is designed and approved.

## What It Does

- Browses exported Hyprland setting metadata.
- Supports local in-memory search.
- Shows read-only row details.
- Shows safety classifications.
- Shows disabled write-safety metadata.

## What It Does Not Do

- No writable settings yet.
- No live config reading yet.
- No apply, save, or reload controls.
- No broad write support.

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

## Dependencies

Runtime dependencies:

- `gtk4`
- `libadwaita`
- `glib2`

Build dependencies:

- `rust`
- `cargo`
- `pkgconf`

## Metadata Provenance

The bundled metadata targets Hyprland `0.55.2`. It is export-backed and validated before display.

AGS was used as a prototype/spec/export source during the transition to Rust. The final Rust app does not require AGS at runtime, and no live user config is included in the metadata bundle.

## Roadmap

- Read-only metadata browser.
- Packaging and install work.
- Future Rust-native write safety design.

This project is not an official Hyprland project and does not claim Hyprland upstream endorsement.
