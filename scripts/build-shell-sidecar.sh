#!/usr/bin/env bash
# Builds the bypass-shell client (release) and stages it as a Tauri sidecar
# binary named "bypass-shell-<target-triple>" under src-tauri/binaries, so the
# `externalBin` entry in tauri.bundle.conf.json bundles it next to the app.
#
# This is only needed for release builds. `tauri dev` does NOT use the sidecar:
# beforeDevCommand compiles bypass-shell straight into the workspace target dir,
# right next to the dev binary, where the installer locates it at runtime.
set -euo pipefail

SCRIPT_DIR=$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)
REPO_ROOT=$(cd "${SCRIPT_DIR}/.." && pwd)
cd "${REPO_ROOT}"

TRIPLE=$(rustc -vV | sed -n 's/^host: //p')
if [[ -z "${TRIPLE}" ]]; then
  echo "ERROR = could not determine host target triple" >&2
  exit 1
fi

EXT=""
case "${TRIPLE}" in
  *windows*) EXT=".exe" ;;
esac

cargo build --release -p bypass-shell

# Resolve the actual target dir (honors CARGO_TARGET_DIR / .cargo config).
TARGET_DIR=$(cargo metadata --format-version 1 --no-deps \
  | python3 -c "import sys, json; print(json.load(sys.stdin)['target_directory'])")
SRC="${TARGET_DIR}/release/bypass-shell${EXT}"
if [[ ! -f "${SRC}" ]]; then
  echo "ERROR = built sidecar not found at ${SRC}" >&2
  exit 1
fi

DEST_DIR="src-tauri/binaries"
mkdir -p "${DEST_DIR}"
cp "${SRC}" "${DEST_DIR}/bypass-shell-${TRIPLE}${EXT}"
echo "Staged sidecar: ${DEST_DIR}/bypass-shell-${TRIPLE}${EXT}"
