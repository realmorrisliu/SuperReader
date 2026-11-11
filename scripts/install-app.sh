#!/bin/bash
set -euo pipefail
root="$(cd "$(dirname "$0")/.." && pwd)"
manifest="$root/spikes/shell/nsh.json"
name="$(sed -n 's/^[[:space:]]*"name"[[:space:]]*:[[:space:]]*"\([^"]*\)".*/\1/p' "$manifest" | head -n 1)"
app="$root/dist/${name}.app"
if [ -z "$name" ] || [ ! -d "$app" ]; then
  echo "missing $app" >&2
  exit 1
fi
ditto "$app" "/Applications/${name}.app"
echo "/Applications/${name}.app"
