#!/usr/bin/env bash
# Normalize Tauri DMG name for CI upload (single expected filename).
set -euo pipefail

SCRIPT_DIR=$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)
REPO_ROOT=$(cd "${SCRIPT_DIR}/.." && pwd)
cd "${REPO_ROOT}"

DMG_DIR="src-tauri/target/release/bundle/dmg"
TARGET_NAME="Bypass-arm64.dmg"

if [ ! -d "${DMG_DIR}" ]; then
  echo "ERROR: ${DMG_DIR} does not exist (run pnpm tauri build first)"
  exit 1
fi

shopt -s nullglob
dmgs=("${DMG_DIR}"/*.dmg)
if [ "${#dmgs[@]}" -eq 0 ]; then
  echo "ERROR: no .dmg under ${DMG_DIR}"
  exit 1
fi

target_path="${DMG_DIR}/${TARGET_NAME}"
for f in "${dmgs[@]}"; do
  if [ "$(basename "${f}")" = "${TARGET_NAME}" ]; then
    echo "DMG already ${TARGET_NAME}"
    exit 0
  fi
done

for f in "${dmgs[@]}"; do
  if [ "$(basename "${f}")" != "${TARGET_NAME}" ]; then
    mv -f "${f}" "${target_path}"
    echo "Renamed $(basename "${f}") -> ${TARGET_NAME}"
    exit 0
  fi
done

echo "ERROR: could not normalize DMG name"
exit 1
