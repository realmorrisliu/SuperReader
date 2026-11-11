#!/usr/bin/env bash
set -eu -o pipefail
# bash required for pipefail

ROOT="$(cd "$(dirname "$0")/.." && pwd)"
cd "$ROOT"

fail() {
  echo "$*" >&2
  exit 1
}

out="$(python3 scripts/verify --dry-run --smoke)"
echo "$out" | python3 -c 'import json,sys; d=json.load(sys.stdin); assert d["mode"]=="smoke"; assert "pdf-title" in d["checks"]'
test "$(echo "$out" | python3 -c 'import json,sys; print(len(json.load(sys.stdin)["runners"]))')" -ge 1

out="$(python3 scripts/verify --dry-run --only pdf-title epub-turn)"
echo "$out" | python3 -c 'import json,sys; d=json.load(sys.stdin); assert set(d["checks"])=={"pdf-title","epub-turn"}'

if python3 scripts/verify --dry-run --only not-a-real-check 2>/dev/null; then
  fail "expected unknown check to fail"
fi

paths="$(bash scripts/verify-path-checks.sh spikes/shell/src/main.rs)"
echo "$paths" | grep -q epub-turn || fail "path map should include epub-turn for shell changes"

echo "verify-args.test.sh passed"
