#!/usr/bin/env bash
# Refresh the trusted Hyprland descriptions export and rerun the pinned
# migration audit.
#
# Read-only against the compositor: the only hyprctl commands used are
# `hyprctl version` and `hyprctl -j descriptions`. This script never runs
# the compositor reload command, never dispatches, and never mutates
# runtime or the active config.
#
# Behavior:
# - If the live Hyprland is the pinned 0.55.4, the capture under
#   data/exports/hyprland-0.55.4/ is refreshed in place, the diff against
#   the previous capture is reported (rows added/removed, numeric bounds
#   drift, cosmetic description diffs), and the pinned migration test is
#   rerun.
# - If the live Hyprland is any other version, the 0.55.4 capture is NOT
#   overwritten: a new versioned capture is written under
#   data/exports/hyprland-<version>/ and the report states the migration
#   step needed. No guessed rows are ever hand-edited.
set -euo pipefail

REPO_ROOT="$(cd "$(dirname "$0")/.." && pwd)"
PINNED_VERSION="0.55.4"
PINNED_EXPORT_DIR="$REPO_ROOT/data/exports/hyprland-$PINNED_VERSION"
PINNED_CAPTURE="$PINNED_EXPORT_DIR/hyprctl-descriptions.v$PINNED_VERSION.json"
PINNED_VERSION_FILE="$PINNED_EXPORT_DIR/hyprland-version.txt"
REPORT_PATH="$REPO_ROOT/data/reports/hyprland-0.55.4-export-refresh.v0.55.2.json"

command -v hyprctl >/dev/null || { echo "hyprctl not found; run inside a Hyprland session" >&2; exit 1; }
command -v jq >/dev/null || { echo "jq not found" >&2; exit 1; }

VERSION_LINE="$(hyprctl version | head -1)"
LIVE_VERSION="$(echo "$VERSION_LINE" | awk '{print $2}')"
LIVE_COMMIT="$(echo "$VERSION_LINE" | sed -n 's/.*at commit \([0-9a-f]*\).*/\1/p')"
echo "live: Hyprland $LIVE_VERSION at commit $LIVE_COMMIT"

CAPTURE_TMP="$(mktemp)"
trap 'rm -f "$CAPTURE_TMP"' EXIT
hyprctl -j descriptions > "$CAPTURE_TMP"
jq empty "$CAPTURE_TMP"
NEW_ROW_COUNT="$(jq length "$CAPTURE_TMP")"

if [[ "$LIVE_VERSION" == "$PINNED_VERSION" ]]; then
    TARGET_DIR="$PINNED_EXPORT_DIR"
    TARGET_CAPTURE="$PINNED_CAPTURE"
    MATCHED="true"
else
    TARGET_DIR="$REPO_ROOT/data/exports/hyprland-$LIVE_VERSION"
    TARGET_CAPTURE="$TARGET_DIR/hyprctl-descriptions.v$LIVE_VERSION.json"
    MATCHED="false"
    echo "live version $LIVE_VERSION differs from pinned $PINNED_VERSION:"
    echo "  the pinned 0.55.4 capture is preserved; capturing to $TARGET_DIR instead."
fi

PREVIOUS_CAPTURE=""
if [[ "$MATCHED" == "true" && -f "$PINNED_CAPTURE" ]]; then
    PREVIOUS_CAPTURE="$(mktemp)"
    cp "$PINNED_CAPTURE" "$PREVIOUS_CAPTURE"
fi

mkdir -p "$TARGET_DIR"
cp "$CAPTURE_TMP" "$TARGET_CAPTURE"
hyprctl version | head -3 > "$TARGET_DIR/hyprland-version.txt"
echo "captured $NEW_ROW_COUNT option rows to $TARGET_CAPTURE"

# Diff the refreshed capture against the previous one (pinned path only).
DIFF_JSON='{"previousCaptureAvailable": false}'
if [[ -n "$PREVIOUS_CAPTURE" ]]; then
    DIFF_JSON="$(jq -n \
        --slurpfile old "$PREVIOUS_CAPTURE" \
        --slurpfile new "$CAPTURE_TMP" '
        ($old[0] | map({key: .name, value: .}) | from_entries) as $o |
        ($new[0] | map({key: .name, value: .}) | from_entries) as $n |
        {
            previousCaptureAvailable: true,
            previousRowCount: ($old[0] | length),
            newRowCount: ($new[0] | length),
            rowsAdded: [($n | keys[]) | select($o[.] == null)],
            rowsRemoved: [($o | keys[]) | select($n[.] == null)],
            rowsChangedBounds: [($n | keys[])
                | select($o[.] != null)
                | select(($o[.].min != $n[.].min) or ($o[.].max != $n[.].max))
                | {name: ., oldMin: $o[.].min, newMin: $n[.].min, oldMax: $o[.].max, newMax: $n[.].max}],
            rowsChangedDescriptionOnly: [($n | keys[])
                | select($o[.] != null)
                | select(($o[.].min == $n[.].min) and ($o[.].max == $n[.].max))
                | select($o[.].description != $n[.].description)] | length,
            rowsChangedTypeOrData: [($n | keys[])
                | select($o[.] != null)
                | select($o[.] != $n[.])
                | select(($o[.].min == $n[.].min) and ($o[.].max == $n[.].max))
                | select($o[.].description == $n[.].description)]
        }')"
    rm -f "$PREVIOUS_CAPTURE"
fi

# Rerun the pinned migration audit (only meaningful for the pinned path;
# for another live version the audit still validates the preserved capture).
MIGRATION_TEST_PASSED="false"
if (cd "$REPO_ROOT" && cargo test --test hyprland_0554_migration_audit) ; then
    MIGRATION_TEST_PASSED="true"
fi

jq -n \
    --arg versionLine "$VERSION_LINE" \
    --arg liveVersion "$LIVE_VERSION" \
    --arg liveCommit "$LIVE_COMMIT" \
    --arg matched "$MATCHED" \
    --arg capturePath "${TARGET_CAPTURE#"$REPO_ROOT/"}" \
    --arg newRowCount "$NEW_ROW_COUNT" \
    --arg migrationTestPassed "$MIGRATION_TEST_PASSED" \
    --arg generatedAt "$(date -u +%Y-%m-%dT%H:%M:%SZ)" \
    --argjson diff "$DIFF_JSON" '
    {
        artifactKind: "hyprland-0554-export-refresh-report",
        projectDataVersion: "v0.55.2",
        generatedAt: $generatedAt,
        liveVersionLine: $versionLine,
        liveVersion: $liveVersion,
        liveCommit: $liveCommit,
        captureMatchedPinnedVersion: ($matched == "true"),
        pinnedCapturePreserved: true,
        capturePath: $capturePath,
        capturedRowCount: ($newRowCount | tonumber),
        diff: $diff,
        migrationTestCommand: "cargo test --test hyprland_0554_migration_audit",
        migrationTestPassed: ($migrationTestPassed == "true"),
        readOnlyCommandsUsed: ["hyprctl version", "hyprctl -j descriptions"],
        hyprctlReloadRan: false,
        runtimeMutationRan: false,
        handEditedRows: false,
        nextRecommendedWork: (if $matched == "true"
            then "none: capture refreshed in place and the pinned migration audit reran"
            else "live Hyprland is no longer 0.55.4: review the new versioned capture, then run a model migration audit against it before changing the v0.55.2 model"
            end)
    }' > "$REPORT_PATH"
echo "report written to ${REPORT_PATH#"$REPO_ROOT/"}"
echo "migration test passed: $MIGRATION_TEST_PASSED"
[[ "$MIGRATION_TEST_PASSED" == "true" ]]
