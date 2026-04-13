#!/usr/bin/env bash
# Install deps, build Tauri DMG firmado y notarizado (staple), normaliza nombre para el release.
# Run from desktop repo root. Requires macOS arm64, pnpm on PATH.
#
# Firma (CI): APPLE_CERTIFICATE (Base64 .p12), APPLE_CERTIFICATE_PASSWORD, APPLE_SIGNING_IDENTITY
#   → usar certificado "Developer ID Application" para distribución fuera de la Mac App Store.
# Notarización (Tauri / notarytool): APPLE_ID (email), APPLE_PASSWORD (contraseña específica de app),
#   APPLE_TEAM_ID (10 caracteres). Ver https://v2.tauri.app/distribute/sign/macos
set -euo pipefail

SCRIPT_DIR=$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)
REPO_ROOT=$(cd "${SCRIPT_DIR}/.." && pwd)
cd "${REPO_ROOT}"

# shellcheck source=normalize-tauri-signing-env.sh
source "${SCRIPT_DIR}/normalize-tauri-signing-env.sh"

: "${APPLE_CERTIFICATE:?}"
: "${APPLE_CERTIFICATE_PASSWORD:?}"
: "${APPLE_SIGNING_IDENTITY:?}"
: "${APPLE_ID:?}"
: "${APPLE_PASSWORD:?}"
: "${APPLE_TEAM_ID:?}"

test "$(uname -s)" = "Darwin"
test "$(uname -m)" = "arm64"

command -v pnpm >/dev/null

brew install protobuf

pnpm install --frozen-lockfile

pnpm tauri build

bash scripts/rename-release-dmg.sh
