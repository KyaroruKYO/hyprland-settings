# Changelog

## 0.2.0-rc.1 (2026-07-13) — release candidate

- Runtime preview capability system: all 341 scalar rows classified honestly,
  with live preview (direct or throttled) for 135 rows and revert/cancel.
- Dead-man supervised preview with countdown auto-revert for risky rows;
  36 input/cursor rows promoted by live proofs on real hardware.
- Safe Live Save Mode: config autoreload is disabled at runtime (no file
  write, instantly reversible) so saving cannot reload the compositor;
  every production Save requires the mode and verifies it live.
- Gated Save: one atomic write with a byte-exact backup, reread verification
  through the parser, and automatic backup restore on verification failure.
- Save as default for Safe Live Save Mode: persist
  `misc:disable_autoreload = true` through the same gated Save — user-chosen,
  never automatic.
- Structured-family record picker: modify existing animation records
  (enabled, speed, and bezier reference — existing curves only) and bezier
  curves (all four control points) with supervised live preview and gated
  Save. Every editable field shape carries a passed live-proof receipt;
  records and fields that cannot be safely edited say why. Creating and
  deleting records stays blocked.
- Hyprland 0.55.4 compatibility audited at zero drift against a trusted
  `hyprctl -j descriptions` export from the official binary, with a
  repeatable read-only refresh workflow.
- 18 touch-family rows and 3 secondary-device rows remain disarmed pending
  real hardware proofs (honest limitation, not a defect).

## 0.1.0 (2026-06-19)

- Added a read-only GTK/libadwaita app for browsing Hyprland `0.55.2` setting metadata.
- Bundled validated Hyprland `0.55.2` export metadata.
- Added local in-memory search and read-only row details.
- Added safety classifications and disabled write-safety metadata.
- Added desktop launcher, AppStream/metainfo, and PNG icon artifacts.
- Added explicit `hyprland-settings` binary target.
- Guarded safe-batch config writes for eligible normal scalar settings.
