#!/usr/bin/env bash
# Build Tauri .deb and AppImage on Debian/Ubuntu. Copies artifacts to dist/ with fixed names.
# Run from repo root (e.g. ubuntu-latest).
set -euo pipefail

SCRIPT_DIR=$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)
REPO_ROOT=$(cd "${SCRIPT_DIR}/.." && pwd)
cd "${REPO_ROOT}"

export DEBIAN_FRONTEND=noninteractive
sudo apt-get update
sudo apt-get install -y --no-install-recommends \
  ca-certificates curl file \
  build-essential pkg-config libssl-dev \
  libwebkit2gtk-4.1-dev libgtk-3-dev \
  libayatana-appindicator3-dev librsvg2-dev patchelf

pnpm install --frozen-lockfile
pnpm exec tauri build --bundles deb,appimage

mkdir -p dist
shopt -s nullglob
debs=(src-tauri/target/release/bundle/deb/*.deb)
appimages=(src-tauri/target/release/bundle/appimage/*.AppImage)
if [ "${#debs[@]}" -ne 1 ]; then
  echo "Expected exactly one .deb in bundle/deb/, found ${#debs[@]}" >&2
  ls -la src-tauri/target/release/bundle/deb/ 2>/dev/null || true
  exit 1
fi
if [ "${#appimages[@]}" -ne 1 ]; then
  echo "Expected exactly one AppImage in bundle/appimage/, found ${#appimages[@]}" >&2
  ls -la src-tauri/target/release/bundle/appimage/ 2>/dev/null || true
  exit 1
fi
cp "${debs[0]}" dist/bypass-linux-debian-amd64.deb
cp "${appimages[0]}" dist/bypass-linux-debian-amd64.AppImage
echo "Created dist/bypass-linux-debian-amd64.deb and dist/bypass-linux-debian-amd64.AppImage"
