#!/usr/bin/env bash
set -euo pipefail

cd /home/kyo/Projects/hyprland-settings
cargo test --test live_scenario_harness
