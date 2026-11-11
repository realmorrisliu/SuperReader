#!/bin/bash
# install_name_tool rewrites load commands and leaves LC_UUID in place, so the UUID
# still identifies the binary after the rpath edit.
set -euo pipefail

app="${1:?app bundle}"
reader="${2:?reader binary}"
plist="$app/Contents/Info.plist"

if [ ! -f "$plist" ]; then
  echo "missing $plist" >&2
  exit 1
fi
if [ ! -f "$reader" ]; then
  echo "missing reader binary $reader" >&2
  exit 1
fi
if ! command -v otool >/dev/null 2>&1; then
  echo "otool is required to identify the reader binary" >&2
  exit 1
fi
if ! command -v strings >/dev/null 2>&1; then
  echo "strings is required to read the reader binary" >&2
  exit 1
fi

exe_name="$(sed -n '/<key>CFBundleExecutable<\/key>/{n;s/.*<string>\([^<]*\)<\/string>.*/\1/p;}' "$plist" | head -n 1)"
if [ -z "$exe_name" ]; then
  echo "CFBundleExecutable is missing" >&2
  exit 1
fi
exe="$app/Contents/MacOS/$exe_name"
if [ ! -f "$exe" ]; then
  echo "CFBundleExecutable $exe_name is missing" >&2
  exit 1
fi

if strings -a "$exe" | grep -F -q 'SuperReader spike'; then
  echo "CFBundleExecutable contains 'SuperReader spike'" >&2
  exit 1
fi

macho_uuid() {
  otool -l "$1" | awk '
    $0 ~ /LC_UUID/ { found = 1; next }
    found && $1 == "uuid" { print $2; exit }
  '
}

app_uuid="$(macho_uuid "$exe")"
reader_uuid="$(macho_uuid "$reader")"
if [ -z "$app_uuid" ] || [ "$app_uuid" != "$reader_uuid" ]; then
  echo "CFBundleExecutable is not the reader binary" >&2
  exit 1
fi
