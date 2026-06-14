#!/usr/bin/env bash
# Tauri release on Debian/Ubuntu. Copies dist/desktop-linux-debian-amd64.{deb,AppImage} for GitHub Releases.
# Do not pack deb + AppImage into one tarball: AppImage embeds GTK/WebKit and is ~150MB+; .deb uses system libs and is much smaller.
# Requires pnpm on PATH (install Node + pnpm in CI before this script). Run from desktop repo root.
set -euo pipefail

SCRIPT_DIR=$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)
REPO_ROOT=$(cd "${SCRIPT_DIR}/.." && pwd)
cd "${REPO_ROOT}"

# shellcheck source=normalize-tauri-signing-env.sh
source "${SCRIPT_DIR}/normalize-tauri-signing-env.sh"

if ! command -v pnpm >/dev/null 2>&1; then
  echo "ERROR = pnpm must be on PATH (use actions/setup-node + pnpm/action-setup in CI)." >&2
  exit 1
fi

export DEBIAN_FRONTEND=noninteractive
sudo apt-get update
sudo apt-get install -y --no-install-recommends \
  ca-certificates curl git build-essential pkg-config libssl-dev protobuf-compiler \
  libwebkit2gtk-4.1-dev \
  libayatana-appindicator3-dev \
  librsvg2-dev \
  patchelf \
  wget \
  file \
  xdg-utils

pnpm install --frozen-lockfile
bash scripts/build-shell-sidecar.sh
pnpm tauri build -c src-tauri/tauri.bundle.conf.json

BUNDLE="${REPO_ROOT}/target/release/bundle"
if [[ ! -d "${BUNDLE}" ]]; then
  echo "ERROR = Tauri bundle directory missing at ${BUNDLE}" >&2
  exit 1
fi

mkdir -p dist

shopt -s nullglob
debs=( "${BUNDLE}/deb"/*.deb )
appimages=( "${BUNDLE}/appimage"/*.AppImage )
if [[ ${#appimages[@]} -eq 0 ]]; then
  appimages=( "${BUNDLE}/appimage"/*.appimage )
fi

if [[ ${#debs[@]} -ne 1 ]]; then
  echo "ERROR = expected exactly one .deb in ${BUNDLE}/deb, got ${#debs[@]}" >&2
  ls -la "${BUNDLE}/deb" 2>/dev/null || true
  exit 1
fi
if [[ ${#appimages[@]} -ne 1 ]]; then
  echo "ERROR = expected exactly one AppImage in ${BUNDLE}/appimage, got ${#appimages[@]}" >&2
  ls -la "${BUNDLE}/appimage" 2>/dev/null || true
  exit 1
fi

cp "${debs[0]}" "${REPO_ROOT}/dist/desktop-linux-debian-amd64.deb"
cp "${appimages[0]}" "${REPO_ROOT}/dist/desktop-linux-debian-amd64.AppImage"

echo "Release artifacts (sizes differ by design: .deb uses system WebKit; AppImage is self-contained):"
ls -lh "${REPO_ROOT}/dist/desktop-linux-debian-amd64.deb" "${REPO_ROOT}/dist/desktop-linux-debian-amd64.AppImage"
