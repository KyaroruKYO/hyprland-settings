#!/usr/bin/env bash
set -euo pipefail

backup_parent="${1:-/home/kyo/Documents/system-audit/hyprland-settings-live-test-backups}"
timestamp="$(date +%Y%m%d_%H%M%S)"
backup_dir="${backup_parent%/}/${timestamp}"

mkdir -p "$backup_dir/files" "$backup_dir/processes" "$backup_dir/checksums" "$backup_dir/symlinks"

copy_tree() {
  local name="$1"
  local source="$2"
  if [ -e "$source" ] || [ -L "$source" ]; then
    cp -a "$source" "$backup_dir/files/$name"
  else
    printf '%s\n' "missing" > "$backup_dir/files/$name.missing"
  fi
}

write_checksums() {
  local name="$1"
  local source="$2"
  if [ -d "$source" ]; then
    (
      cd "$source"
      find . -type f -print0 | sort -z | xargs -0 sha256sum
    ) > "$backup_dir/checksums/$name.sha256"
  else
    : > "$backup_dir/checksums/$name.sha256"
  fi
}

write_symlinks() {
  local name="$1"
  local source="$2"
  if [ -d "$source" ]; then
    find "$source" -type l -printf '%P -> %l\n' | sort > "$backup_dir/symlinks/$name.txt"
  else
    : > "$backup_dir/symlinks/$name.txt"
  fi
}

copy_tree hypr "$HOME/.config/hypr"
copy_tree waybar "$HOME/.config/waybar"
copy_tree ags "$HOME/.config/ags"

write_checksums hypr "$HOME/.config/hypr"
write_checksums waybar "$HOME/.config/waybar"
write_checksums ags "$HOME/.config/ags"

write_symlinks hypr "$HOME/.config/hypr"
write_symlinks waybar "$HOME/.config/waybar"
write_symlinks ags "$HOME/.config/ags"

pgrep -a ags > "$backup_dir/processes/ags.before.txt" || true
pgrep -a waybar > "$backup_dir/processes/waybar.before.txt" || true
readlink -f "$HOME/.config/hypr/hyprland.conf" > "$backup_dir/hyprland-conf-resolved.txt" || true
git -C /home/kyo/Projects/hyprland-settings status --short > "$backup_dir/repo-status.before.txt" || true
git -C /home/kyo/Projects/hyprland-settings rev-parse HEAD > "$backup_dir/repo-head.before.txt" || true
date -Is > "$backup_dir/created-at.txt"

cat > "$backup_dir/RESTORE-INSTRUCTIONS.md" <<'EOF'
# Hyprland Settings Live Test Restore

From a TTY:

```sh
cd /home/kyo/Projects/hyprland-settings
tools/live_scenario_harness/restore_desktop_state.sh BACKUP_DIR
tools/live_scenario_harness/verify_restore.sh BACKUP_DIR
```

Replace `BACKUP_DIR` with this backup directory path.
EOF

printf '%s\n' "$backup_dir"
