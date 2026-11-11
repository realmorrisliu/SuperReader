#!/bin/bash
# Build AppIcon.icns from the 1024 source. sips writes every standard size.
set -euo pipefail

root="$(cd "$(dirname "$0")/.." && pwd)"
src="${1:-$root/assets/app-icon/AppIcon-1024.png}"
out="${2:-$root/dist/AppIcon.icns}"

if [ ! -f "$src" ]; then
  echo "missing app icon at $src" >&2
  exit 1
fi

work="$(mktemp -d)"
trap 'rm -rf "$work"' EXIT
iconset="$work/AppIcon.iconset"
mkdir -p "$iconset"

write_size() {
  px="$1"
  name="$2"
  sips -z "$px" "$px" "$src" --out "$iconset/$name" >/dev/null
}

write_size 16 icon_16x16.png
write_size 32 icon_16x16@2x.png
write_size 32 icon_32x32.png
write_size 64 icon_32x32@2x.png
write_size 128 icon_128x128.png
write_size 256 icon_128x128@2x.png
write_size 256 icon_256x256.png
write_size 512 icon_256x256@2x.png
write_size 512 icon_512x512.png
write_size 1024 icon_512x512@2x.png

mkdir -p "$(dirname "$out")"
iconutil -c icns "$iconset" -o "$out"
echo "$out"
