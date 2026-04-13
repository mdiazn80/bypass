#!/usr/bin/env bash
# Canonical release id: YY.M.X (UTC) — M is month 1–12 (one or two digits), X is sequence within that month (resets when M or YY changes).
# Git tag, VERSION, src-tauri/Cargo.toml, src-tauri/tauri.conf.json, and package.json use the same version string.
# Requires: TZ (e.g. UTC), GITHUB_OUTPUT, GITHUB_EVENT_NAME, GITHUB_REF_NAME.
set -euo pipefail

SCRIPT_DIR=$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)
REPO_ROOT=$(cd "${SCRIPT_DIR}/.." && pwd)
cd "${REPO_ROOT}"

: "${GITHUB_OUTPUT:?GITHUB_OUTPUT must be set}"
: "${GITHUB_EVENT_NAME:?GITHUB_EVENT_NAME must be set}"
: "${GITHUB_REF_NAME:?GITHUB_REF_NAME must be set}"

YY=$(date +%y)
M=$((10#$(date +%m)))
PREFIX="${YY}.${M}"
git fetch --tags --force

MAX=0
while IFS= read -r t; do
  [ -z "${t:-}" ] && continue
  case "${t}" in
    "${PREFIX}".*)
      SEQ="${t#"${PREFIX}".}"
      if [[ "${SEQ}" =~ ^[0-9]+$ ]]; then
        SEQ_NUM=$((10#$SEQ))
        if [ "${SEQ_NUM}" -gt "${MAX}" ]; then
          MAX="${SEQ_NUM}"
        fi
      fi
      ;;
  esac
done < <(git tag -l "${PREFIX}.*")

NEXT=$((MAX + 1))
TAG="${PREFIX}.${NEXT}"
echo "Next tag: ${TAG} (prefix ${PREFIX}, max existing seq ${MAX})"

if git rev-parse -q --verify "refs/tags/${TAG}" >/dev/null; then
  echo "Tag ${TAG} already exists locally; aborting."
  exit 1
fi

echo "VERSION / manifests / tag: ${TAG}"
export RELEASE_VER="${TAG}"
python3 <<'PY'
import os
import re

ver = os.environ["RELEASE_VER"]

with open("VERSION", "w", encoding="utf-8") as f:
    f.write(ver + "\n")


def bump_cargo(path: str) -> None:
    with open(path, encoding="utf-8") as f:
        text = f.read()
    text2, n = re.subn(
        r'^version = "[^"]*"',
        f'version = "{ver}"',
        text,
        count=1,
        flags=re.M,
    )
    if n != 1:
        raise SystemExit(f"{path}: expected one [package] version line, matched {n}")
    with open(path, "w", encoding="utf-8") as f:
        f.write(text2)


def bump_json_version(path: str) -> None:
    with open(path, encoding="utf-8") as f:
        text = f.read()
    text2, n = re.subn(
        r'^(\s*"version":\s*)"(?:[^"\\]|\\.)*"',
        rf'\1"{ver}"',
        text,
        count=1,
        flags=re.M,
    )
    if n != 1:
        raise SystemExit(f"{path}: expected one top-level \"version\" key, matched {n}")
    with open(path, "w", encoding="utf-8") as f:
        f.write(text2)


bump_cargo("src-tauri/Cargo.toml")
bump_json_version("src-tauri/tauri.conf.json")
bump_json_version("package.json")
PY

git config user.name "github-actions[bot]"
git config user.email "41898282+github-actions[bot]@users.noreply.github.com"
git add VERSION src-tauri/Cargo.toml src-tauri/tauri.conf.json package.json
if git diff --cached --quiet; then
  echo "Version files already match ${TAG}; skipping commit."
else
  git commit -m "chore: release ${TAG}"
fi

if [ "${GITHUB_EVENT_NAME}" = "workflow_dispatch" ]; then
  TARGET_BRANCH="${GITHUB_REF_NAME}"
else
  TARGET_BRANCH="main"
fi

git tag -a "${TAG}" -m "Version ${TAG}"
git push origin "HEAD:${TARGET_BRANCH}"
git push origin "refs/tags/${TAG}"
echo "tag=${TAG}" >> "${GITHUB_OUTPUT}"
