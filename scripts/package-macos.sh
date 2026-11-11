#!/bin/bash
# Build dist/<name>.app around the spike-shell binary main runs.
# Ad hoc unless SR_SIGN_IDENTITY names a real identity.
set -euo pipefail

root="$(cd "$(dirname "$0")/.." && pwd)"
cd "$root"

version="0.1.0-alpha.1"
build_number="1"
manifest="$root/spikes/shell/nsh.json"

json_string() {
  key="$1"
  file="$2"
  sed -n "s/^[[:space:]]*\"${key}\"[[:space:]]*:[[:space:]]*\"\\([^\"]*\\)\".*/\\1/p" "$file" | head -n 1
}

name="$(json_string name "$manifest")"
bundle_id="$(json_string id "$manifest")"
icon_rel="$(json_string icon "$manifest")"
if [ -z "$name" ] || [ -z "$bundle_id" ] || [ -z "$icon_rel" ]; then
  echo "nsh.json needs name, id, and icon" >&2
  exit 1
fi
if ! printf '%s' "$name" | grep -Eq '^[A-Za-z0-9._-]+$'; then
  echo "nsh.json name cannot be CFBundleExecutable: $name" >&2
  exit 1
fi
if ! printf '%s' "$bundle_id" | grep -Eq '^[A-Za-z0-9.-]+$'; then
  echo "nsh.json id is not a bundle identifier: $bundle_id" >&2
  exit 1
fi
if [ "${icon_rel#/}" != "$icon_rel" ]; then
  icon="$icon_rel"
else
  icon="$root/spikes/shell/$icon_rel"
fi
if [ ! -f "$icon" ]; then
  echo "missing app icon at $icon" >&2
  exit 1
fi

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

cargo build --manifest-path "$root/spikes/Cargo.toml" -p spike-shell --release
reader="$root/spikes/target/release/spike-shell"
if [ ! -x "$reader" ]; then
  echo "missing reader binary $reader" >&2
  exit 1
fi

icns="$("$root/scripts/make-icns.sh" "$icon" "$root/dist/AppIcon.icns")"

app="$root/dist/${name}.app"
rm -rf "$app"
mkdir -p "$app/Contents/MacOS" "$app/Contents/Frameworks" "$app/Contents/Resources/pdfium"
cp "$reader" "$app/Contents/MacOS/$name"
chmod 755 "$app/Contents/MacOS/$name"

cp "$pdfium_lib" "$app/Contents/Frameworks/libpdfium.dylib"
install_name_tool -id "@rpath/libpdfium.dylib" "$app/Contents/Frameworks/libpdfium.dylib"
install_name_tool -add_rpath "@executable_path/../Frameworks" "$app/Contents/MacOS/$name"
if [ -f "$license_root/LICENSE" ]; then
  cp "$license_root/LICENSE" "$app/Contents/Resources/pdfium/LICENSE"
fi
if [ -d "$license_root/licenses" ]; then
  cp -R "$license_root/licenses" "$app/Contents/Resources/pdfium/licenses"
fi

cp "$icns" "$app/Contents/Resources/AppIcon.icns"

plist="$app/Contents/Info.plist"
cat > "$plist" << EOF
<?xml version="1.0" encoding="UTF-8"?>
<!DOCTYPE plist PUBLIC "-//Apple//DTD PLIST 1.0//EN" "http://www.apple.com/DTDs/PropertyList-1.0.dtd">
<plist version="1.0">
<dict>
  <key>CFBundleDevelopmentRegion</key>
  <string>en</string>
  <key>CFBundleLocalizations</key>
  <array>
    <string>en</string>
    <string>zh-Hans</string>
  </array>
  <key>LSHasLocalizedDisplayName</key>
  <true/>
  <key>CFBundleDisplayName</key>
  <string>${name}</string>
  <key>CFBundleExecutable</key>
  <string>${name}</string>
  <key>CFBundleIconFile</key>
  <string>AppIcon</string>
  <key>CFBundleIdentifier</key>
  <string>${bundle_id}</string>
  <key>CFBundleInfoDictionaryVersion</key>
  <string>6.0</string>
  <key>CFBundleName</key>
  <string>${name}</string>
  <key>CFBundlePackageType</key>
  <string>APPL</string>
  <key>CFBundleShortVersionString</key>
  <string>${version}</string>
  <key>CFBundleVersion</key>
  <string>${build_number}</string>
  <key>LSMinimumSystemVersion</key>
  <string>14.0</string>
  <key>NSHighResolutionCapable</key>
  <true/>
  <key>CFBundleDocumentTypes</key>
  <array>
    <dict>
      <key>CFBundleTypeExtensions</key>
      <array>
        <string>epub</string>
      </array>
      <key>CFBundleTypeName</key>
      <string>EPUB</string>
      <key>CFBundleTypeRole</key>
      <string>Viewer</string>
      <key>LSHandlerRank</key>
      <string>Alternate</string>
      <key>LSItemContentTypes</key>
      <array>
        <string>org.idpf.epub-container</string>
      </array>
    </dict>
    <dict>
      <key>CFBundleTypeExtensions</key>
      <array>
        <string>pdf</string>
      </array>
      <key>CFBundleTypeName</key>
      <string>PDF</string>
      <key>CFBundleTypeRole</key>
      <string>Viewer</string>
      <key>LSHandlerRank</key>
      <string>Alternate</string>
      <key>LSItemContentTypes</key>
      <array>
        <string>com.adobe.pdf</string>
      </array>
    </dict>
  </array>
  <key>UTImportedTypeDeclarations</key>
  <array>
    <dict>
      <key>UTTypeIdentifier</key>
      <string>org.idpf.epub-container</string>
      <key>UTTypeDescription</key>
      <string>EPUB</string>
      <key>UTTypeConformsTo</key>
      <array>
        <string>public.data</string>
        <string>public.composite-content</string>
        <string>com.apple.package</string>
      </array>
      <key>UTTypeTagSpecification</key>
      <dict>
        <key>public.filename-extension</key>
        <array>
          <string>epub</string>
        </array>
        <key>public.mime-type</key>
        <string>application/epub+zip</string>
      </dict>
    </dict>
  </array>
</dict>
</plist>
EOF

lproj="$app/Contents/Resources/zh-Hans.lproj"
mkdir -p "$lproj"
printf '%s\n' 'CFBundleName = "慢卷";' 'CFBundleDisplayName = "慢卷";' \
  | iconv -f UTF-8 -t UTF-16 > "$lproj/InfoPlist.strings"

echo "check reader bundle"
"$root/scripts/check-reader-bundle.sh" "$app" "$reader"
"$root/scripts/sign-macos.sh" "$app"
echo "$app"
