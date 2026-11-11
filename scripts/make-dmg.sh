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

stage="$(mktemp -d)"
trap 'rm -rf "$stage"' EXIT
cp -R "$app" "$stage/SuperReader.app"
ln -s /Applications "$stage/Applications"
rm -f "$dmg"
hdiutil create -volname SuperReader -srcfolder "$stage" -ov -format UDZO "$dmg" >/dev/null

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
  notarize | tee "$root/dist/notary.json"
  xcrun stapler staple "$dmg"
  xcrun stapler staple "$app"
fi

echo "$dmg"
