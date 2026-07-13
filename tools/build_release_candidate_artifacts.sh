#!/usr/bin/env bash
# Build LOCAL release artifacts for one version (release candidates and
# final releases), mirroring the v0.1.0 artifact layout exactly
# (dist/<version>/ with a binary tarball, a source tarball, and
# SHA256SUMS).
#
# Safety boundaries:
# - never creates or moves any tag, and never publishes anything anywhere
#   (no release upload, no draft release, no push) — tagging and publishing
#   are separate, user-approved steps;
# - never touches dist/v0.1.0 or any existing dist directory;
# - never runs any compositor command and never touches the Hyprland
#   session or config;
# - writes only dist/v<version>/ (and cargo's target/ via the build).
set -euo pipefail

cd "$(dirname "$0")/.."

RC_VERSION="${1:?usage: build_release_candidate_artifacts.sh <version, e.g. 0.2.0-rc.1 or 0.2.0>}"
RC_NAME="v${RC_VERSION}"
DIST_DIR="dist/${RC_NAME}"
BINARY_STAGE="hyprland-settings-${RC_NAME}-linux-x86_64"
SOURCE_PREFIX="hyprland-settings-${RC_NAME}"

# The manifest version must match Cargo.toml — refuse to package a
# mismatched tree.
CARGO_VERSION="$(grep -m1 '^version' Cargo.toml | cut -d'"' -f2)"
if [[ "${CARGO_VERSION}" != "${RC_VERSION}" ]]; then
    echo "refusing: Cargo.toml version is ${CARGO_VERSION}, expected ${RC_VERSION}" >&2
    exit 1
fi

# Never overwrite existing artifacts (and never touch dist/v0.1.0).
if [[ -e "${DIST_DIR}" ]]; then
    echo "refusing: ${DIST_DIR} already exists; remove it manually to rebuild" >&2
    exit 1
fi

echo "== cargo build --release"
cargo build --release

echo "== staging binary artifact"
STAGE_ROOT="$(mktemp -d)"
trap 'rm -rf "${STAGE_ROOT}"' EXIT
mkdir -p "${STAGE_ROOT}/${BINARY_STAGE}/data/applications" \
    "${STAGE_ROOT}/${BINARY_STAGE}/data/metainfo"
cp target/release/hyprland-settings "${STAGE_ROOT}/${BINARY_STAGE}/"
cp README.md LICENSE "${STAGE_ROOT}/${BINARY_STAGE}/"
cp data/applications/io.github.kyarorukyo.hyprlandsettings.desktop \
    "${STAGE_ROOT}/${BINARY_STAGE}/data/applications/"
cp data/metainfo/io.github.kyarorukyo.hyprlandsettings.metainfo.xml \
    "${STAGE_ROOT}/${BINARY_STAGE}/data/metainfo/"

mkdir -p "${DIST_DIR}"
tar -C "${STAGE_ROOT}" -czf "${DIST_DIR}/${BINARY_STAGE}.tar.gz" "${BINARY_STAGE}"

echo "== creating source archive from HEAD"
git archive --format=tar.gz --prefix="${SOURCE_PREFIX}/" \
    -o "${DIST_DIR}/hyprland-settings-${RC_NAME}-source.tar.gz" HEAD

echo "== checksums"
sha256sum "${DIST_DIR}/hyprland-settings-${RC_NAME}-source.tar.gz" \
    "${DIST_DIR}/${BINARY_STAGE}.tar.gz" >"${DIST_DIR}/SHA256SUMS"

echo "== dist/v0.1.0 untouched check"
sha256sum --check --quiet dist/v0.1.0/SHA256SUMS
echo "dist/v0.1.0 checksums still match"

echo "== done"
cat "${DIST_DIR}/SHA256SUMS"
echo "Artifacts are LOCAL ONLY. Nothing was tagged, pushed, or published."
