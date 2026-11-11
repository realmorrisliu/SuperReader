#!/bin/bash
# dist/SuperReader-<version>.dmg with a drag-to-Applications layout.
# Signs and notarizes only when SR_SIGN_IDENTITY is a real identity and a notary
# profile or API key is set. Ad hoc builds stay local and unsigned by Apple.
set -euo pipefail

root="$(cd "$(dirname "$0")/.." && pwd)"
app="$root/dist/SuperReader.app"
version="0.1.0-alpha.1"
dmg="$root/dist/SuperReader-${version}.dmg"
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
cp -R "$app" "$stage/SuperReader.app"
ln -s /Applications "$stage/Applications"
rm -f "$dmg"
writable="$root/dist/SuperReader-rw.dmg"
rm -f "$writable"
hdiutil create -volname SuperReader -srcfolder "$stage" -ov -format UDRW "$writable" >/dev/null
hdiutil attach "$writable" -mountpoint "$mount" -nobrowse >/dev/null
cp "$icns" "$mount/.VolumeIcon.icns"
SetFile -a C "$mount"
hdiutil detach "$mount" >/dev/null
hdiutil convert "$writable" -format UDZO -ov -o "$dmg" >/dev/null
rm -f "$writable"

if [ "$identity" != "-" ]; then
  codesign --force --sign "$identity" --timestamp "$dmg"
fi

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
  xcrun stapler staple "$dmg"
  xcrun stapler staple "$app"
fi

echo "$dmg"
