#!/usr/bin/env bash
set -euo pipefail

usage() {
  cat >&2 <<'EOF'
usage: gtk_automation_probe.sh <scenario-home> <evidence-dir> [timeout-seconds]

Launches Hyprland Settings in safe-env mode only. It never runs live-swap,
never clicks Apply, never reloads Hyprland, and never mutates runtime.
EOF
}

scenario_home="${1:-}"
evidence_dir="${2:-}"
timeout_seconds="${3:-12}"

if [ -z "$scenario_home" ] || [ -z "$evidence_dir" ]; then
  usage
  exit 2
fi

mkdir -p "$evidence_dir"

app_pid=""
graceful_close="false"
forced_kill="false"
launch_succeeded="false"
accessibility_attempted="false"
accessibility_succeeded="false"

cleanup() {
  if [ -n "$app_pid" ] && kill -0 "$app_pid" 2>/dev/null; then
    tools/live_scenario_harness/close_app_window.sh "$app_pid" "$evidence_dir" || true
    sleep 1
    if kill -0 "$app_pid" 2>/dev/null; then
      forced_kill="true"
      kill "$app_pid" 2>/dev/null || true
      sleep 1
    else
      graceful_close="true"
    fi
  fi
  cat > "$evidence_dir/probe-result.json" <<EOF
{
  "safeEnvModeUsed": true,
  "liveSwapModeUsed": false,
  "scenarioHome": "$scenario_home",
  "appPid": "${app_pid:-}",
  "appLaunchAttempted": true,
  "appLaunchSucceeded": $launch_succeeded,
  "accessibilityInspectionAttempted": $accessibility_attempted,
  "accessibilityInspectionSucceeded": $accessibility_succeeded,
  "closeAttempted": true,
  "closeSucceeded": $graceful_close,
  "forcedKillUsed": $forced_kill,
  "applyClicked": false,
  "realConfigEdited": false,
  "hyprlandReloaded": false,
  "mutatingHyprctlUsed": false,
  "runtimeMutated": false
}
EOF
}
trap cleanup EXIT INT TERM

(
  cd /home/kyo/Projects/hyprland-settings
  if [ ! -x target/debug/hyprland-settings ]; then
    cargo build --quiet
  fi
  HOME="$scenario_home" XDG_CONFIG_HOME="$scenario_home/.config" \
    timeout "$timeout_seconds" target/debug/hyprland-settings
) > "$evidence_dir/stdout.txt" 2> "$evidence_dir/stderr.txt" &
app_pid="$!"

sleep 3
if kill -0 "$app_pid" 2>/dev/null; then
  launch_succeeded="true"
fi

accessibility_attempted="true"
if tools/live_scenario_harness/collect_accessibility_tree.py "$evidence_dir/accessibility.json" >/dev/null 2>"$evidence_dir/accessibility.err"; then
  accessibility_succeeded="true"
fi

tools/live_scenario_harness/close_app_window.sh "$app_pid" "$evidence_dir" || true
sleep 1
if ! kill -0 "$app_pid" 2>/dev/null; then
  graceful_close="true"
fi

wait "$app_pid" >/dev/null 2>&1 || true
app_pid=""
