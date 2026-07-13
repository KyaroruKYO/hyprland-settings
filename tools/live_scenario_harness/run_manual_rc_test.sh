#!/usr/bin/env bash
# Manual RC test wrapper: unpacks the PACKAGED v0.2.0-rc.1 binary artifact,
# verifies its checksum, launches it in the REAL session (real HOME), runs
# the AT-SPI driver, closes the app, and verifies the active config bytes
# and runtime state are untouched. Read-only except the reversible runtime
# interactions the driver performs (each verified restored).
set -euo pipefail

repo_root="/home/kyo/Projects/hyprland-settings"
cd "$repo_root"

evidence_dir="${1:-/tmp/hyprland-settings-rc-manual-test/$(date +%Y%m%d_%H%M%S)}"
mkdir -p "$evidence_dir"
stage="$(mktemp -d)"
trap 'rm -rf "$stage"' EXIT

echo "== verify artifact checksums"
sha256sum --check --quiet dist/v0.2.0-rc.1/SHA256SUMS

echo "== unpack RC binary artifact"
tar -C "$stage" -xzf dist/v0.2.0-rc.1/hyprland-settings-v0.2.0-rc.1-linux-x86_64.tar.gz
rc_binary="$stage/hyprland-settings-v0.2.0-rc.1-linux-x86_64/hyprland-settings"
test -x "$rc_binary"

config_path="$HOME/.config/hypr/hyprland.conf"
pre_config_hash="$(sha256sum "$config_path" | cut -d' ' -f1)"
hyprctl getoption misc:disable_autoreload > "$evidence_dir/pre-autoreload.txt"
hyprctl getoption general:gaps_in > "$evidence_dir/pre-gaps.txt"
# Volatile runtime fields (tearing blockers, focus, workspaces) change as
# windows open/close; the mutation check compares the stable geometry.
hyprctl monitors | grep -vE "tearingBlockedBy|focused|activeWorkspace|specialWorkspace" > "$evidence_dir/pre-monitors.txt"

echo "== launch RC binary in the real session"
"$rc_binary" > "$evidence_dir/app-stdout.txt" 2> "$evidence_dir/app-stderr.txt" &
app_pid="$!"
sleep 4
if ! kill -0 "$app_pid" 2>/dev/null; then
  echo "RC binary failed to launch" >&2
  exit 1
fi

echo "== run AT-SPI driver"
driver_status=0
python3 tools/live_scenario_harness/manual_rc_test_driver.py \
  "$evidence_dir/driver-summary.json" "$evidence_dir" || driver_status=$?

echo "== close app"
kill -TERM "$app_pid" 2>/dev/null || true
sleep 2
kill -0 "$app_pid" 2>/dev/null && { kill -9 "$app_pid" || true; }

echo "== post-state verification"
post_config_hash="$(sha256sum "$config_path" | cut -d' ' -f1)"
hyprctl getoption misc:disable_autoreload > "$evidence_dir/post-autoreload.txt"
hyprctl getoption general:gaps_in > "$evidence_dir/post-gaps.txt"
hyprctl monitors | grep -vE "tearingBlockedBy|focused|activeWorkspace|specialWorkspace" > "$evidence_dir/post-monitors.txt"

config_untouched=false
[ "$pre_config_hash" = "$post_config_hash" ] && config_untouched=true
gaps_same=false
cmp -s "$evidence_dir/pre-gaps.txt" "$evidence_dir/post-gaps.txt" && gaps_same=true
monitors_same=false
cmp -s "$evidence_dir/pre-monitors.txt" "$evidence_dir/post-monitors.txt" && monitors_same=true
autoreload_same=false
cmp -s "$evidence_dir/pre-autoreload.txt" "$evidence_dir/post-autoreload.txt" && autoreload_same=true

cat > "$evidence_dir/wrapper-result.json" <<EOF
{
  "rcArtifactChecksumVerified": true,
  "rcBinaryLaunched": true,
  "driverExitStatus": $driver_status,
  "activeConfigHashBefore": "$pre_config_hash",
  "activeConfigHashAfter": "$post_config_hash",
  "activeConfigUntouched": $config_untouched,
  "gapsInUnchanged": $gaps_same,
  "monitorsUnchanged": $monitors_same,
  "autoreloadRestored": $autoreload_same
}
EOF
cat "$evidence_dir/wrapper-result.json"
echo "evidence: $evidence_dir"
[ "$driver_status" -eq 0 ] && $config_untouched && $monitors_same && $autoreload_same
