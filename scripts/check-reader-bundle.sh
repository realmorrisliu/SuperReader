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

plist_string() {
  key="$1"
  sed -n "/<key>${key}<\\/key>/{n;s/.*<string>\\([^<]*\\)<\\/string>.*/\\1/p;}" "$plist" | head -n 1
}

exe_name="$(plist_string CFBundleExecutable)"
if [ -z "$exe_name" ]; then
  echo "CFBundleExecutable is missing" >&2
  exit 1
fi
exe="$app/Contents/MacOS/$exe_name"
if [ ! -f "$exe" ]; then
  echo "CFBundleExecutable $exe_name is missing" >&2
  exit 1
fi

listing="$(mktemp)"
trap 'rm -f "$listing"' EXIT
strings -a "$exe" > "$listing"

if grep -F -q 'SuperReader spike' "$listing"; then
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

if grep -F -q 'SuperReader' "$plist"; then
  echo "Info.plist contains SuperReader" >&2
  exit 1
fi
if [ "$exe_name" != "Slowpage" ]; then
  echo "CFBundleExecutable is not Slowpage" >&2
  exit 1
fi
if [ "$(plist_string CFBundleIdentifier)" != "app.slowpage.reader" ]; then
  echo "CFBundleIdentifier is not app.slowpage.reader" >&2
  exit 1
fi
if [ "$(plist_string CFBundleName)" != "Slowpage" ]; then
  echo "CFBundleName is not Slowpage" >&2
  exit 1
fi
if [ "$(plist_string CFBundleDisplayName)" != "Slowpage" ]; then
  echo "CFBundleDisplayName is not Slowpage" >&2
  exit 1
fi
for needle in org.idpf.epub-container com.adobe.pdf Alternate com.apple.package; do
  if ! grep -F -q "$needle" "$plist"; then
    echo "Info.plist is missing $needle" >&2
    exit 1
  fi
done

strings_file="$app/Contents/Resources/zh-Hans.lproj/InfoPlist.strings"
if [ ! -f "$strings_file" ]; then
  echo "missing zh-Hans InfoPlist.strings" >&2
  exit 1
fi
bom="$(od -An -t x1 -N 2 "$strings_file" | tr -d ' \n')"
if [ "$bom" = "fffe" ] || [ "$bom" = "feff" ]; then
  localized="$(iconv -f UTF-16 -t UTF-8 "$strings_file")"
else
  localized="$(cat "$strings_file")"
fi
if printf '%s' "$localized" | grep -F -q 'SuperReader'; then
  echo "zh-Hans InfoPlist.strings contains SuperReader" >&2
  exit 1
fi
for key in CFBundleName CFBundleDisplayName; do
  if ! printf '%s' "$localized" | grep -F -q "${key} = \"慢卷\""; then
    echo "zh-Hans ${key} is not 慢卷" >&2
    exit 1
  fi
done

if grep -F -q -x 'SuperReader' "$listing"; then
  echo "window title or menu contains SuperReader" >&2
  exit 1
fi
if ! grep -F -q 'Slowpage' "$listing"; then
  echo "window title or menu is not Slowpage" >&2
  exit 1
fi
