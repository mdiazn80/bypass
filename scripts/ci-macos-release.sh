#!/usr/bin/env bash
# Build Tauri DMG on macOS Apple Silicon. Copies the bundle to dist/bypass-macos-arm64.dmg.
# Signing: solo si APPLE_CERTIFICATE, APPLE_CERTIFICATE_PASSWORD y APPLE_SIGNING_IDENTITY están
# definidas y no vacías (Base64 del .p12, contraseña, identidad "Developer ID Application: …").
# Si falta alguna, se usa --no-sign (evita que Tauri intente import con cadenas vacías y falle).
# Notarización no se usa.
# Run from repo root.
set -euo pipefail

SCRIPT_DIR=$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)
REPO_ROOT=$(cd "${SCRIPT_DIR}/.." && pwd)
cd "${REPO_ROOT}"

test "$(uname -s)" = "Darwin"
test "$(uname -m)" = "arm64"

pnpm install --frozen-lockfile

if [ -n "${APPLE_CERTIFICATE:-}" ] && [ -n "${APPLE_CERTIFICATE_PASSWORD:-}" ] && [ -n "${APPLE_SIGNING_IDENTITY:-}" ]; then
  echo "Building with code signing (APPLE_* secrets present)."
  pnpm exec tauri build --bundles dmg
else
  echo "Building without code signing (set all APPLE_* secrets to enable signing)."
  unset APPLE_CERTIFICATE APPLE_CERTIFICATE_PASSWORD APPLE_SIGNING_IDENTITY || true
  pnpm exec tauri build --bundles dmg --no-sign
fi

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
