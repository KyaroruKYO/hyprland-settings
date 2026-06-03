# Contributing

Hyprland Settings is currently a read-only metadata browser. Contributions must preserve that boundary unless a separate design phase explicitly approves changing it.

## Safety Boundaries

- No writes without explicit design approval.
- No live config reads without explicit design approval.
- No Hyprland commands in read-only paths.
- No AGS runtime dependency.
- No IPC unless explicitly approved.
- No image or generated assets without approval.
- No packaging changes without scoped approval.

## Validation

Before submitting changes, run:

```sh
cargo fmt --check
cargo check
cargo test
```

Metadata changes must preserve validation, and official sources must be provenanced.
