#!/usr/bin/env bash
# Build Tauri bundle on Fedora/RHEL-family, then ship the .rpm inside a tarball (workflow asset name).
# Run from repo root inside fedora/dnf (e.g. GitHub Actions job with container: fedora).
set -euo pipefail

SCRIPT_DIR=$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)
REPO_ROOT=$(cd "${SCRIPT_DIR}/.." && pwd)
cd "${REPO_ROOT}"

if [[ ! -f /etc/redhat-release ]]; then
  echo "ERROR: This script expects a Red Hat family OS (dnf/yum)." >&2
  exit 1
fi

if [[ $(id -u) -eq 0 ]]; then
  SUDO=()
else
  SUDO=(sudo)
fi

if command -v dnf >/dev/null 2>&1; then
  "${SUDO[@]}" dnf install -y --setopt=install_weak_deps=False \
    ca-certificates curl file git gcc gcc-c++ make cmake \
    openssl-devel pkgconf patchelf rpm-build \
    webkit2gtk4.1-devel gtk3-devel \
    libappindicator-gtk3-devel librsvg2-devel
elif command -v yum >/dev/null 2>&1; then
  "${SUDO[@]}" yum install -y \
    ca-certificates curl file git gcc gcc-c++ make cmake \
    openssl-devel pkgconf patchelf rpm-build \
    webkit2gtk4.1-devel gtk3-devel \
    libappindicator-gtk3-devel librsvg2-devel
else
  echo "ERROR: Neither dnf nor yum found." >&2
  exit 1
fi

pnpm install --frozen-lockfile
# Solo RPM en entorno dnf (evita intentar .deb/AppImage de "targets": "all").
pnpm exec tauri build --bundles rpm

mkdir -p dist
shopt -s nullglob
rpms=(src-tauri/target/release/bundle/rpm/*.rpm)
if [ "${#rpms[@]}" -lt 1 ]; then
  echo "Expected at least one .rpm in bundle/rpm/; Tauri may need rpm-build and RPM targets enabled." >&2
  ls -la src-tauri/target/release/bundle/ 2>/dev/null || true
  exit 1
fi
OUT="${REPO_ROOT}/dist/bypass-linux-redhat-amd64.tar.gz"
rm -f "${OUT}"
( cd "${REPO_ROOT}/src-tauri/target/release/bundle/rpm" && tar -czvf "${OUT}" ./*.rpm )
echo "Created dist/bypass-linux-redhat-amd64.tar.gz"
