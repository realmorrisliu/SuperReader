#!/bin/bash
set -euo pipefail
root="$(cd "$(dirname "$0")/.." && pwd)"
app="$root/dist/SuperReader.app"
if [ ! -d "$app" ]; then
  echo "missing $app" >&2
  exit 1
fi
ditto "$app" /Applications/SuperReader.app
echo "/Applications/SuperReader.app"
