#!/usr/bin/env bash
# Map changed paths to verify check ids (feature-map.json). Prints one id per line.
set -eu -o pipefail

ROOT="$(cd "$(dirname "$0")/.." && pwd)"
MAP="$ROOT/.cursor/skills/verify-superreader/feature-map.json"

if [[ ! -f "$MAP" ]]; then
  echo "missing $MAP" >&2
  exit 1
fi

export MAP
python3 - "$@" <<'PY'
import fnmatch
import json
import os
import sys
from pathlib import Path

map_path = Path(os.environ["MAP"])
body = json.loads(map_path.read_text())
paths = sys.argv[1:] if len(sys.argv) > 1 else [line.strip() for line in sys.stdin if line.strip()]

runner_checks = {cid for ids in body["runners"].values() for cid in ids}
pending = {
    "open-pdf-from-library",
    "open-pdf-ask",
    "open-pdf-scroll-column",
    "library-search-tantivy",
    "comment-note",
    "comment-ui",
    "alan-spawn",
    "superreader-json",
    "annotations-sqlite",
    "alpha-finder-open",
}

features = set()
for rel in paths:
    rel = rel.lstrip("./")
    for rule in body["path_rules"]:
        if any(fnmatch.fnmatch(rel, pat) for pat in rule["patterns"]):
            features.update(rule.get("features") or [])

checks = set()
for feat in features:
    for check_id in body["features"].get(feat, []):
        if check_id in runner_checks and check_id not in pending:
            checks.add(check_id)

for check_id in sorted(checks):
    print(check_id)
PY
