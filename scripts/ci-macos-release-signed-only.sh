#!/usr/bin/env bash
# Install deps, build a signed and notarized (stapled) Tauri DMG, and normalize its name for the release.
# Run from desktop repo root. Requires macOS arm64, pnpm on PATH.
#
# Signing (CI): APPLE_CERTIFICATE (Base64 .p12), APPLE_CERTIFICATE_PASSWORD, APPLE_SIGNING_IDENTITY
#   -> use a "Developer ID Application" certificate for distribution outside the Mac App Store.
# Notarization (Tauri / notarytool): APPLE_ID (email), APPLE_PASSWORD (app-specific password),
#   APPLE_TEAM_ID (10 characters). See https://v2.tauri.app/distribute/sign/macos
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

# The release workflow installs protobuf before the checks compile in release.
# A local run of this script still needs it.
if ! command -v protoc >/dev/null 2>&1; then
  brew install protobuf
fi

# CI_REUSE_FRONTEND=1: the checks step already installed dependencies, built
# dist/ and compiled target/release. Skip the install. The frontend hook
# (scripts/frontend-build.mjs) sees the same variable and does not run
# `pnpm build` again, and the Tauri config on disk stays unchanged so Cargo
# can reuse target/release.
if [ "${CI_REUSE_FRONTEND:-}" = "1" ]; then
  test -d node_modules
  test -f dist/index.html
else
  pnpm install --frozen-lockfile
fi

bash scripts/build-shell-sidecar.sh
pnpm tauri build -c src-tauri/tauri.bundle.conf.json

bash scripts/rename-release-dmg.sh
