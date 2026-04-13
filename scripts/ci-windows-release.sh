#!/usr/bin/env bash
# Tauri release on Windows. Produces dist/desktop-windows-amd64.zip (bundle output).
# Requires pnpm on PATH and protoc (CI installs via Chocolatey). Run from desktop repo root in Git Bash.
set -euo pipefail

SCRIPT_DIR=$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)
REPO_ROOT=$(cd "${SCRIPT_DIR}/.." && pwd)
cd "${REPO_ROOT}"

_sys=$(uname -s 2>/dev/null || true)
if [[ "${OS:-}" != "Windows_NT" && ! "${_sys}" =~ ^(MINGW|MSYS|CYGWIN) ]]; then
  echo "ERROR = Run this script on Windows." >&2
  exit 1
fi

if ! command -v pnpm >/dev/null 2>&1; then
  echo "ERROR = pnpm must be on PATH." >&2
  exit 1
fi

pnpm install --frozen-lockfile
pnpm tauri build

BUNDLE="${REPO_ROOT}/src-tauri/target/release/bundle"
if [[ ! -d "${BUNDLE}" ]]; then
  echo "ERROR = Tauri bundle directory missing at ${BUNDLE}" >&2
  exit 1
fi

mkdir -p dist
OUT="${REPO_ROOT}/dist/desktop-windows-amd64.zip"
rm -f "${OUT}"
# Full tree under bundle/ (e.g. nsis/*.exe). tar -a creates zip on Windows 10+; Compress-Archive is easy to get wrong.
tar -a -cf "${OUT}" -C "${BUNDLE}" .
echo "Created ${OUT}"
ls -lh "${OUT}" || true
