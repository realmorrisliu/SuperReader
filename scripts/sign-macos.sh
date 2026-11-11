#!/bin/bash
# Sign a SuperReader.app. Inner libraries first, then the bundle.
# SR_SIGN_IDENTITY defaults to ad hoc (-). A real identity gets the hardened runtime.
set -euo pipefail

app="${1:?app bundle}"
root="$(cd "$(dirname "$0")/.." && pwd)"
identity="${SR_SIGN_IDENTITY:--}"
entitlements="$root/scripts/entitlements.plist"

sign_item() {
  target="$1"
  if [ "$identity" = "-" ]; then
    codesign --force --sign - "$target"
  else
    codesign --force --sign "$identity" --options runtime --timestamp "$target"
  fi
}

if [ -d "$app/Contents/Frameworks" ]; then
  while IFS= read -r item; do
    [ -n "$item" ] || continue
    sign_item "$item"
  done < <(find "$app/Contents/Frameworks" -type f -name '*.dylib')
fi

if [ "$identity" = "-" ]; then
  codesign --force --sign - "$app"
else
  codesign --force --sign "$identity" --options runtime --timestamp --entitlements "$entitlements" "$app"
fi
