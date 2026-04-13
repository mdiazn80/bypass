#!/usr/bin/env bash
# Install deps, build code-signed Tauri DMG (no notarization), normalize filename for release upload.
# Run from desktop repo root. Requires macOS arm64, pnpm on PATH (configure in workflow before this script).
# Env (Tauri bundler): APPLE_CERTIFICATE, APPLE_CERTIFICATE_PASSWORD, APPLE_SIGNING_IDENTITY
# Do not set APPLE_ID / APPLE_PASSWORD / APPLE_TEAM_ID — notarization is skipped.
set -euo pipefail

SCRIPT_DIR=$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)
REPO_ROOT=$(cd "${SCRIPT_DIR}/.." && pwd)
cd "${REPO_ROOT}"

# shellcheck source=normalize-tauri-signing-env.sh
source "${SCRIPT_DIR}/normalize-tauri-signing-env.sh"

: "${APPLE_CERTIFICATE:?}"
: "${APPLE_CERTIFICATE_PASSWORD:?}"
: "${APPLE_SIGNING_IDENTITY:?}"

test "$(uname -s)" = "Darwin"
test "$(uname -m)" = "arm64"

command -v pnpm >/dev/null

brew install protobuf

pnpm install --frozen-lockfile

pnpm tauri build

bash scripts/rename-release-dmg.sh
