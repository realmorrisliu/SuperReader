#!/bin/bash
set -euo pipefail

root="$(cd "$(dirname "$0")/.." && pwd)"
check="$root/scripts/check-reader-bundle.sh"
work="$(mktemp -d)"
trap 'rm -rf "$work"' EXIT

cat > "$work/otool" << 'EOF'
#!/bin/bash
file="$1"
if [ "$file" = "-l" ]; then
  file="$2"
fi
uuid="$(head -n 1 "$file")"
printf '      cmd LC_UUID\n     uuid %s\n' "$uuid"
EOF
chmod +x "$work/otool"
export PATH="$work:$PATH"

make_app() {
  dir="$1"
  uuid="$2"
  body="$3"
  exe_name="${4:-SuperReader}"
  mkdir -p "$dir/Contents/MacOS"
  printf '%s\n%s\n' "$uuid" "$body" > "$dir/Contents/MacOS/$exe_name"
  cat > "$dir/Contents/Info.plist" << EOF
<?xml version="1.0" encoding="UTF-8"?>
<plist version="1.0"><dict>
  <key>CFBundleExecutable</key>
  <string>${exe_name}</string>
</dict></plist>
EOF
}

reader="$work/spike-shell"
printf '%s\nreader\n' "UUID-READER" > "$reader"

make_app "$work/good.app" "UUID-READER" "reader main"
"$check" "$work/good.app" "$reader"

make_app "$work/spike.app" "UUID-READER" "SuperReader spike"
if "$check" "$work/spike.app" "$reader" 2>"$work/spike.err"; then
  echo "spike title was accepted" >&2
  exit 1
fi
grep -F -q "contains 'SuperReader spike'" "$work/spike.err"

make_app "$work/other.app" "UUID-OTHER" "reader main"
if "$check" "$work/other.app" "$reader" 2>"$work/other.err"; then
  echo "foreign executable was accepted" >&2
  exit 1
fi
grep -F -q "is not the reader binary" "$work/other.err"

mkdir -p "$work/empty.app/Contents/MacOS"
printf '%s\n' '<?xml version="1.0" encoding="UTF-8"?><plist version="1.0"><dict></dict></plist>' \
  > "$work/empty.app/Contents/Info.plist"
if "$check" "$work/empty.app" "$reader" 2>"$work/empty.err"; then
  echo "missing CFBundleExecutable was accepted" >&2
  exit 1
fi
grep -F -q "CFBundleExecutable is missing" "$work/empty.err"

pkg="$root/scripts/package-macos.sh"
sign="$root/scripts/sign-macos.sh"
dmg="$root/scripts/make-dmg.sh"
line() {
  file="$1"
  pattern="$2"
  grep -n -F -e "$pattern" "$file" | head -n 1 | cut -d: -f1
}
tool_line="$(line "$pkg" 'install_name_tool')"
check_line="$(line "$pkg" 'check-reader-bundle.sh')"
sign_line="$(line "$pkg" 'sign-macos.sh')"
if [ "$tool_line" -ge "$check_line" ] || [ "$check_line" -ge "$sign_line" ]; then
  echo "binary edits must finish before the bundle check and the first codesign" >&2
  exit 1
fi
if grep -q 'nsh package' "$pkg" "$root/.github/workflows/release.yml"; then
  echo "the release still calls nsh package" >&2
  exit 1
fi
if ! grep -q 'target/release/spike-shell' "$pkg"; then
  echo "the bundle does not copy the spike-shell reader" >&2
  exit 1
fi
dylib_line="$(line "$sign" 'Contents/Frameworks')"
app_sign_line="$(line "$sign" '--entitlements "$entitlements"')"
verify_line="$(line "$sign" 'codesign --verify --deep --strict --verbose=2')"
if [ "$dylib_line" -ge "$app_sign_line" ] || [ "$app_sign_line" -ge "$verify_line" ]; then
  echo "sign inner libraries, then the app, then verify" >&2
  exit 1
fi
dmg_sign_line="$(line "$dmg" 'timestamp "$dmg"')"
mounted_line="$(grep -n -x 'verify_mounted_app' "$dmg" | head -n 1 | cut -d: -f1)"
if [ -z "$dmg_sign_line" ] || [ -z "$mounted_line" ] || [ "$dmg_sign_line" -ge "$mounted_line" ]; then
  echo "sign the disk image before verifying the app inside it" >&2
  exit 1
fi

echo "check-reader-bundle ok"
