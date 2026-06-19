#!/usr/bin/env bash
set -euo pipefail

backup_dir="${1:?usage: restore_desktop_state.sh <backup-dir>}"

restore_tree() {
  local name="$1"
  local target="$2"
  local source="$backup_dir/files/$name"
  local missing="$backup_dir/files/$name.missing"

  mkdir -p "$(dirname "$target")"
  if [ -e "$source" ] || [ -L "$source" ]; then
    rm -rf "$target"
    cp -a "$source" "$target"
  elif [ -f "$missing" ]; then
    rm -rf "$target"
  else
    printf 'backup tree missing for %s\n' "$name" >&2
    return 1
  fi
}

restore_tree hypr "$HOME/.config/hypr"
restore_tree waybar "$HOME/.config/waybar"
restore_tree ags "$HOME/.config/ags"

printf 'restore complete: %s\n' "$backup_dir"
