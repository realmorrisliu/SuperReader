#!/bin/bash
# Build dist/SuperReader.app. Ad hoc unless SR_SIGN_IDENTITY names a real identity.
set -euo pipefail

root="$(cd "$(dirname "$0")/.." && pwd)"
cd "$root"

version="0.1.0-alpha.1"
build_number="1"
icon="$root/assets/app/placeholder.png"
pdfium_root="${SPIKE_PDFIUM:-/tmp/pdfium}"
if [ -d "$pdfium_root/lib" ]; then
  pdfium_lib="$pdfium_root/lib/libpdfium.dylib"
  license_root="$pdfium_root"
else
  pdfium_lib="$pdfium_root/libpdfium.dylib"
  license_root="$(dirname "$pdfium_root")"
fi

if [ ! -f "$pdfium_lib" ]; then
  mkdir -p /tmp/pdfium
  curl -fsSL -L -o /tmp/pdfium/pdfium-mac-arm64.tgz \
    "https://github.com/bblanchon/pdfium-binaries/releases/download/chromium%2F8125/pdfium-mac-arm64.tgz"
  tar -xzf /tmp/pdfium/pdfium-mac-arm64.tgz -C /tmp/pdfium
  pdfium_lib="/tmp/pdfium/lib/libpdfium.dylib"
  license_root="/tmp/pdfium"
fi

if [ ! -f "$icon" ]; then
  echo "missing placeholder icon at $icon" >&2
  exit 1
fi

out="$root/dist/nsh"
rm -rf "$out"
mkdir -p "$out"

(
  cd "$root/spikes/shell"
  nsh package --target macos \
    --name SuperReader \
    --id com.superreader.app \
    --app-version "$version" \
    --build "$build_number" \
    --icon "$icon" \
    --out "$out"
)

app="$(find "$out" -name 'SuperReader.app' -print -quit)"
if [ -z "$app" ]; then
  echo "nsh package did not write SuperReader.app" >&2
  exit 1
fi

rm -rf "$root/dist/SuperReader.app"
mv "$app" "$root/dist/SuperReader.app"
app="$root/dist/SuperReader.app"

mkdir -p "$app/Contents/Frameworks" "$app/Contents/Resources/pdfium"
cp "$pdfium_lib" "$app/Contents/Frameworks/libpdfium.dylib"
install_name_tool -id "@rpath/libpdfium.dylib" "$app/Contents/Frameworks/libpdfium.dylib"
if [ -f "$license_root/LICENSE" ]; then
  cp "$license_root/LICENSE" "$app/Contents/Resources/pdfium/LICENSE"
fi
if [ -d "$license_root/licenses" ]; then
  cp -R "$license_root/licenses" "$app/Contents/Resources/pdfium/licenses"
fi

exe="$(find "$app/Contents/MacOS" -type f ! -name '.*' | head -n 1)"
if [ -z "$exe" ]; then
  echo "SuperReader.app has no executable" >&2
  exit 1
fi
install_name_tool -add_rpath "@executable_path/../Frameworks" "$exe"

plist="$app/Contents/Info.plist"
/usr/libexec/PlistBuddy -c "Delete :CFBundleDocumentTypes" "$plist" 2>/dev/null || true
/usr/libexec/PlistBuddy -c "Add :CFBundleDocumentTypes array" "$plist"
add_type() {
  index="$1"
  name="$2"
  ext="$3"
  uti="$4"
  /usr/libexec/PlistBuddy -c "Add :CFBundleDocumentTypes:$index dict" "$plist"
  /usr/libexec/PlistBuddy -c "Add :CFBundleDocumentTypes:$index:CFBundleTypeName string $name" "$plist"
  /usr/libexec/PlistBuddy -c "Add :CFBundleDocumentTypes:$index:CFBundleTypeRole string Viewer" "$plist"
  /usr/libexec/PlistBuddy -c "Add :CFBundleDocumentTypes:$index:LSHandlerRank string Owner" "$plist"
  /usr/libexec/PlistBuddy -c "Add :CFBundleDocumentTypes:$index:CFBundleTypeExtensions array" "$plist"
  /usr/libexec/PlistBuddy -c "Add :CFBundleDocumentTypes:$index:CFBundleTypeExtensions:0 string $ext" "$plist"
  /usr/libexec/PlistBuddy -c "Add :CFBundleDocumentTypes:$index:LSItemContentTypes array" "$plist"
  /usr/libexec/PlistBuddy -c "Add :CFBundleDocumentTypes:$index:LSItemContentTypes:0 string $uti" "$plist"
}
add_type 0 EPUB epub org.idpf.epub-container
add_type 1 PDF pdf com.adobe.pdf

"$root/scripts/sign-macos.sh" "$app"
echo "$app"
