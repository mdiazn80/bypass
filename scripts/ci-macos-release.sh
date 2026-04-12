#!/usr/bin/env bash
# Build Tauri DMG on macOS Apple Silicon. Copies the bundle to dist/bypass-macos-arm64.dmg.
# Signing: if APPLE_CERTIFICATE, APPLE_CERTIFICATE_PASSWORD and APPLE_SIGNING_IDENTITY are set
# (Tauri env vars), `tauri build` signs the app; notarization is not used.
# Run from repo root.
set -euo pipefail

SCRIPT_DIR=$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)
REPO_ROOT=$(cd "${SCRIPT_DIR}/.." && pwd)
cd "${REPO_ROOT}"

test "$(uname -s)" = "Darwin"
test "$(uname -m)" = "arm64"

pnpm install --frozen-lockfile
pnpm exec tauri build --bundles dmg

mkdir -p dist
shopt -s nullglob
dmgs=(src-tauri/target/release/bundle/dmg/*.dmg)
if [ "${#dmgs[@]}" -ne 1 ]; then
  echo "Expected exactly one DMG in src-tauri/target/release/bundle/dmg/, found ${#dmgs[@]}" >&2
  ls -la src-tauri/target/release/bundle/dmg/ 2>/dev/null || true
  exit 1
fi
cp "${dmgs[0]}" dist/bypass-macos-arm64.dmg
echo "Created dist/bypass-macos-arm64.dmg"
