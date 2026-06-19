#!/usr/bin/env bash
set -euo pipefail

app_pid="${1:?usage: close_app_window.sh <pid> [evidence-dir]}"
evidence_dir="${2:-/tmp/hyprland-settings-gtk-automation-close}"
mkdir -p "$evidence_dir"

if ! kill -0 "$app_pid" 2>/dev/null; then
  printf '{"closeAttempted":true,"closeSucceeded":true,"reason":"process already exited"}\n' > "$evidence_dir/close-result.json"
  exit 0
fi

kill -TERM "$app_pid" 2>/dev/null || true
sleep 1

if kill -0 "$app_pid" 2>/dev/null; then
  printf '{"closeAttempted":true,"closeSucceeded":false,"reason":"process still running after SIGTERM"}\n' > "$evidence_dir/close-result.json"
  exit 1
fi

printf '{"closeAttempted":true,"closeSucceeded":true,"reason":"SIGTERM closed app process"}\n' > "$evidence_dir/close-result.json"
