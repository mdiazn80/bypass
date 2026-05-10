#!/usr/bin/env bash
# Assembles latest.json from per-platform updater fragment files and writes it to ./latest.json.
# Expected fragment files (downloaded as Actions artifacts):
#   darwin-aarch64/updater-darwin-aarch64.json
#   linux-x86_64/updater-linux-x86_64.json
#   windows-x86_64/updater-windows-x86_64.json
# Requires: TAG env var (the release tag, e.g. 26.5.1).
set -euo pipefail

: "${TAG:?TAG must be set}"

python3 << 'PY'
import json
import os
from datetime import datetime, timezone

tag = os.environ["TAG"]
pub_date = datetime.now(timezone.utc).strftime("%Y-%m-%dT%H:%M:%S.000Z")

platforms = {}
entries = [
    ("darwin-aarch64",  "darwin-aarch64/updater-darwin-aarch64.json"),
    ("linux-x86_64",    "linux-x86_64/updater-linux-x86_64.json"),
    ("windows-x86_64",  "windows-x86_64/updater-windows-x86_64.json"),
]
for key, path in entries:
    with open(path) as f:
        platforms[key] = json.load(f)

manifest = {
    "version": tag,
    "notes": "",
    "pub_date": pub_date,
    "platforms": platforms,
}

with open("latest.json", "w") as f:
    json.dump(manifest, f, indent=2)
    f.write("\n")

print(f"Assembled latest.json for version {tag}")
for key in platforms:
    print(f"  {key}: {platforms[key]['url']}")
PY
