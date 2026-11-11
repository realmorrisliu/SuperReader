#!/bin/bash
# Rasterize the Slowpage SVG into AppIcon.icns. 16 and 32 @1x are the small variant.
set -euo pipefail

root="$(cd "$(dirname "$0")/.." && pwd)"
src="${1:-$root/assets/icon/Slowpage.svg}"
out="${2:-$root/dist/AppIcon.icns}"

if [ ! -f "$src" ]; then
  echo "missing app icon at $src" >&2
  exit 1
fi
case "$src" in
  *.svg) ;;
  *)
    echo "app icon must be an SVG: $src" >&2
    exit 1
    ;;
esac
if ! command -v iconutil >/dev/null 2>&1; then
  echo "iconutil is required to build the icns" >&2
  exit 1
fi

renderer="$root/spikes/target/release/spike-icon"
if [ ! -x "$renderer" ]; then
  cargo build --manifest-path "$root/spikes/Cargo.toml" -p spike-icon --release
fi

work="$(mktemp -d)"
trap 'rm -rf "$work"' EXIT
iconset="$work/AppIcon.iconset"
mkdir -p "$iconset" "$(dirname "$out")" "$root/dist"
"$renderer" "$src" "$iconset" "$root/dist"
iconutil -c icns "$iconset" -o "$out"
echo "$out"
