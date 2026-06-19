#!/usr/bin/env bash
set -euo pipefail

scenario_home="${1:?usage: run_gtk_safe_env_scenario.sh <scenario-home> [evidence-root]}"
evidence_root="${2:-/tmp/hyprland-settings-gtk-automation}"
timestamp="$(date +%Y%m%d_%H%M%S)"
evidence_dir="${evidence_root%/}/${timestamp}"

mkdir -p "$evidence_dir"

if [ ! -f "$scenario_home/.config/hypr/hyprland.conf" ]; then
  printf 'scenario home does not contain .config/hypr/hyprland.conf: %s\n' "$scenario_home" >&2
  exit 2
fi

tools/live_scenario_harness/gtk_automation_probe.sh "$scenario_home" "$evidence_dir"
printf '%s\n' "$evidence_dir"
