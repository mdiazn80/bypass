#!/usr/bin/env bash
# Assembles latest.json from the macOS updater fragment and writes it to ./latest.json.
# Expected fragment file (downloaded as Actions artifact):
#   darwin-aarch64/updater-darwin-aarch64.json
# Requires: TAG env var (the release tag, e.g. 26.5.1).
set -euo pipefail

: "${TAG:?TAG must be set}"

python3 << 'PY'
import json
import os
from datetime import datetime, timezone

tag = os.environ["TAG"]
pub_date = datetime.now(timezone.utc).strftime("%Y-%m-%dT%H:%M:%S.000Z")

with open("darwin-aarch64/updater-darwin-aarch64.json") as f:
    darwin = json.load(f)

manifest = {
    "version": tag,
    "notes": "",
    "pub_date": pub_date,
    "platforms": {
        "darwin-aarch64": darwin,
    },
}

with open("latest.json", "w") as f:
    json.dump(manifest, f, indent=2)
    f.write("\n")

print(f"Assembled latest.json for version {tag}")
print(f"  darwin-aarch64: {darwin['url']}")
PY
