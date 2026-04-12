#!/usr/bin/env bash
# Build Tauri Windows installer and zip it as dist/bypass-windows-amd64.zip (single NSIS or MSI .exe inside).
# Run from repo root in Git Bash (GitHub Actions windows-latest with shell: bash).
set -euo pipefail

SCRIPT_DIR=$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)
REPO_ROOT=$(cd "${SCRIPT_DIR}/.." && pwd)
cd "${REPO_ROOT}"

_sys=$(uname -s 2>/dev/null || true)
if [[ "${OS:-}" != "Windows_NT" && ! "${_sys}" =~ ^(MINGW|MSYS|CYGWIN) ]]; then
  echo "ERROR: Run this script on Windows (native or Git Bash)." >&2
  exit 1
fi

pnpm install --frozen-lockfile
pnpm tauri build -- --bundles nsis

mkdir -p dist
shopt -s nullglob
exes=(src-tauri/target/release/bundle/nsis/*.exe)
if [ "${#exes[@]}" -eq 0 ]; then
  exes=(src-tauri/target/release/bundle/msi/*.exe)
fi
if [ "${#exes[@]}" -eq 0 ]; then
  echo "Expected an NSIS or MSI .exe under src-tauri/target/release/bundle/" >&2
  ls -la src-tauri/target/release/bundle/ 2>/dev/null || true
  exit 1
fi

INSTALLER="${exes[0]}"
REL="${INSTALLER#"${REPO_ROOT}/"}"
ZIP_OUT="dist/bypass-windows-amd64.zip"
rm -f "${ZIP_OUT}"

powershell -NoProfile -Command \
  "Compress-Archive -LiteralPath (Join-Path (Get-Location) '${REL}') -DestinationPath (Join-Path (Get-Location) '${ZIP_OUT}') -Force"

echo "Created ${ZIP_OUT}"
