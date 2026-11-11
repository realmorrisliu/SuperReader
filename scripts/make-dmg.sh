#!/bin/bash
# dist/<name>-<version>.dmg with a drag-to-Applications layout.
# Signs and notarizes only when SR_SIGN_IDENTITY is a real identity and a notary
# profile or API key is set. Ad hoc builds stay local and unsigned by Apple.
set -euo pipefail

root="$(cd "$(dirname "$0")/.." && pwd)"
manifest="$root/spikes/shell/nsh.json"
name="$(sed -n 's/^[[:space:]]*"name"[[:space:]]*:[[:space:]]*"\([^"]*\)".*/\1/p' "$manifest" | head -n 1)"
if [ -z "$name" ]; then
  echo "nsh.json needs name" >&2
  exit 1
fi
app="$root/dist/${name}.app"
version="0.1.0-alpha.1"
dmg="$root/dist/${name}-${version}.dmg"
identity="${SR_SIGN_IDENTITY:--}"

if [ ! -d "$app" ]; then
  echo "missing $app (run scripts/package-macos.sh)" >&2
  exit 1
fi

icns="$root/dist/AppIcon.icns"
if [ ! -f "$icns" ]; then
  icns="$("$root/scripts/make-icns.sh")"
fi

stage="$(mktemp -d)"
mount="$(mktemp -d)"
trap 'hdiutil detach "$mount" >/dev/null 2>&1 || true; rm -rf "$stage" "$mount"' EXIT
ditto "$app" "$stage/${name}.app"
ln -s /Applications "$stage/Applications"
rm -f "$dmg"
writable="$root/dist/${name}-rw.dmg"
rm -f "$writable"
hdiutil create -volname "$name" -srcfolder "$stage" -ov -format UDRW "$writable" >/dev/null
hdiutil attach "$writable" -mountpoint "$mount" -nobrowse >/dev/null
cp "$icns" "$mount/.VolumeIcon.icns"
SetFile -a C "$mount"
hdiutil detach "$mount" >/dev/null
hdiutil convert "$writable" -format UDZO -ov -o "$dmg" >/dev/null
rm -f "$writable"

if [ "$identity" != "-" ]; then
  codesign --force --sign "$identity" --timestamp "$dmg"
fi

verify_mounted_app() {
  point="$(mktemp -d)"
  hdiutil attach "$dmg" -mountpoint "$point" -nobrowse -readonly >/dev/null
  set +e
  echo "codesign verify dmg app"
  codesign --verify --deep --strict --verbose=2 "$point/${name}.app"
  status=$?
  set -e
  hdiutil detach "$point" >/dev/null || true
  rmdir "$point" || true
  if [ "$status" -ne 0 ]; then
    echo "codesign verify failed for the app inside $dmg" >&2
    exit "$status"
  fi
}
verify_mounted_app

staple() {
  local target="$1"
  local attempt
  for attempt in 1 2 3 4 5 6; do
    if xcrun stapler staple "$target"; then
      return 0
    fi
    echo "stapler attempt ${attempt} did not find a ticket for ${target}" >&2
    sleep 15
  done
  return 1
}

notarize() {
  if [ -n "${SR_NOTARY_PROFILE:-}" ]; then
    xcrun notarytool submit "$dmg" --keychain-profile "$SR_NOTARY_PROFILE" --wait --output-format json
  elif [ -n "${SR_NOTARY_KEY:-}" ]; then
    xcrun notarytool submit "$dmg" \
      --key "$SR_NOTARY_KEY" \
      --key-id "$SR_NOTARY_KEY_ID" \
      --issuer "$SR_NOTARY_ISSUER" \
      --wait \
      --output-format json
  else
    return 0
  fi
}

if [ "$identity" != "-" ] && { [ -n "${SR_NOTARY_PROFILE:-}" ] || [ -n "${SR_NOTARY_KEY:-}" ]; }; then
  mkdir -p "$root/dist"
  set +e
  notarize >"$root/dist/notary.json" 2>"$root/dist/notary.err"
  notary_status=$?
  set -e
  if [ "$notary_status" -ne 0 ]; then
    cat "$root/dist/notary.err" >&2
    if grep -q "agreement" "$root/dist/notary.err"; then
      echo "Apple notarization refused: the Developer Program agreement is missing or expired." >&2
    fi
    echo "$dmg"
    exit 0
  fi
  staple "$dmg"
  staple "$app"
  verify_mounted_app
fi

echo "$dmg"
