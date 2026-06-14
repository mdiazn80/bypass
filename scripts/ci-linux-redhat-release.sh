#!/usr/bin/env bash
# Tauri release on Fedora/RHEL family. Produces dist/desktop-linux-redhat-amd64.tar.gz.
# Requires pnpm on PATH. Run from desktop repo root on dnf/yum systems or CI Fedora container.
set -euo pipefail

SCRIPT_DIR=$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)
REPO_ROOT=$(cd "${SCRIPT_DIR}/.." && pwd)
cd "${REPO_ROOT}"

# shellcheck source=normalize-tauri-signing-env.sh
source "${SCRIPT_DIR}/normalize-tauri-signing-env.sh"

if ! command -v pnpm >/dev/null 2>&1; then
  echo "ERROR = pnpm must be on PATH." >&2
  exit 1
fi

if [[ ! -f /etc/redhat-release ]]; then
  echo "ERROR = This script expects a Red Hat family OS. Use CI or Fedora/RHEL." >&2
  exit 1
fi

if [[ $(id -u) -eq 0 ]]; then
  SUDO=()
else
  SUDO=(sudo)
fi

if command -v dnf >/dev/null 2>&1; then
  "${SUDO[@]}" dnf install -y --setopt=install_weak_deps=False \
    ca-certificates curl git gcc openssl-devel pkgconf protobuf-compiler protobuf-devel findutils wget file \
    webkit2gtk4.1-devel gtk3-devel libappindicator-gtk3-devel librsvg2-devel patchelf xdg-utils \
    rpm-build desktop-file-utils
elif command -v yum >/dev/null 2>&1; then
  "${SUDO[@]}" yum install -y \
    ca-certificates curl git gcc openssl-devel pkgconf protobuf-compiler protobuf-devel findutils wget file \
    webkit2gtk4.1-devel gtk3-devel libappindicator-gtk3-devel librsvg2-devel patchelf xdg-utils \
    rpm-build desktop-file-utils
else
  echo "ERROR = Neither dnf nor yum found." >&2
  exit 1
fi

pnpm install --frozen-lockfile
bash scripts/build-shell-sidecar.sh
# AppImage uses linuxdeploy (FUSE/strip/glibc); it often fails in Fedora containers. Debian/Ubuntu CI
# already ships deb + AppImage; this job targets RHEL/Fedora with the native rpm bundle only.
pnpm tauri build -c src-tauri/tauri.bundle.conf.json --config '{"bundle":{"targets":["rpm"]}}'

BUNDLE="${REPO_ROOT}/target/release/bundle"
if [[ ! -d "${BUNDLE}" ]]; then
  echo "ERROR = Tauri bundle directory missing at ${BUNDLE}" >&2
  exit 1
fi

mkdir -p dist
OUT="${REPO_ROOT}/dist/desktop-linux-redhat-amd64.tar.gz"
rm -f "${OUT}"
tar -czvf "${OUT}" -C "${BUNDLE}" .
echo "Created ${OUT}"
