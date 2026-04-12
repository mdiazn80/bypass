#!/usr/bin/env bash
# Routes `task build` to build:macos, build:linux, or build:windows from Task {{OS}} / {{ARCH}}.
# Run from desktop repo root via Task (templated OS/ARCH).
set -euo pipefail

OS="${1:?}"
ARCH="${2:?}"

case "${OS}" in
  darwin)
    if [[ "${ARCH}" != "arm64" ]]; then
      echo "macOS native build requires Apple Silicon (arm64)." >&2
      exit 1
    fi
    exec task build:macos
    ;;
  linux)
    exec task build:linux
    ;;
  windows)
    exec task build:windows
    ;;
  *)
    echo "Unsupported OS for task build (expected darwin, linux, or windows)." >&2
    exit 1
    ;;
esac
