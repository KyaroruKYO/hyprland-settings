#!/usr/bin/env bash
set -euo pipefail

backup_dir="${1:?usage: verify_restore.sh <backup-dir>}"

verify_tree() {
  local name="$1"
  local current="$2"
  local expected="$backup_dir/checksums/$name.sha256"
  local actual
  actual="$(mktemp)"
  trap 'rm -f "$actual"' RETURN

  if [ -d "$current" ]; then
    (
      cd "$current"
      find . -type f -print0 | sort -z | xargs -0 sha256sum
    ) > "$actual"
  else
    : > "$actual"
  fi

  diff -u "$expected" "$actual"
}

verify_tree hypr "$HOME/.config/hypr"
verify_tree waybar "$HOME/.config/waybar"
verify_tree ags "$HOME/.config/ags"

printf 'restore verification passed: %s\n' "$backup_dir"
