#!/usr/bin/env bash
set -eu -o pipefail
cd "$(dirname "$0")/.."
python3 - <<'PY'
import importlib.util
from pathlib import Path

spec = importlib.util.spec_from_file_location(
    "verify_plan", Path("scripts/verify_plan.py")
)
mod = importlib.util.module_from_spec(spec)
spec.loader.exec_module(mod)
dedupe_steps = mod.dedupe_steps
relaunch_checks = mod.relaunch_checks
split_checks = mod.split_checks

smoke = ["pdf-title", "epub-turn", "pdf-footer-nm"]
steps = dedupe_steps(smoke)
assert len(steps) <= 3
rel = relaunch_checks()
assert "reading-cold-open" in rel
primary, second = split_checks(["pdf-title", "reading-cold-open", "epub-turn"])
assert "reading-cold-open" in second
assert "pdf-title" in primary
print("verify-plan.test ok")
PY
