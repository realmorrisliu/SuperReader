#!/bin/bash
# Launch the signed app the way Finder does and fail unless the reader shell
# opens an empty book, an EPUB, and a PDF.
set -euo pipefail

root="$(cd "$(dirname "$0")/.." && pwd)"
manifest="$root/spikes/shell/nsh.json"
json_string() {
  key="$1"
  sed -n "s/^[[:space:]]*\"${key}\"[[:space:]]*:[[:space:]]*\"\\([^\"]*\\)\".*/\\1/p" "$manifest" | head -n 1
}
name="$(json_string name)"
bundle_id="$(json_string id)"
app="$root/dist/${name}.app"
if [ -z "$name" ] || [ -z "$bundle_id" ] || [ ! -d "$app" ]; then
  echo "missing $app" >&2
  exit 1
fi

dest="${SPIKE_FIXTURES:-/tmp/superreader-fixtures}"
mkdir -p "$dest" "$root/dist"
if [ ! -s "$dest/pride-se.epub" ]; then
  curl -fsSL -A "Mozilla/5.0" -o "$dest/pride-se.epub" \
    "https://standardebooks.org/ebooks/jane-austen/pride-and-prejudice/downloads/jane-austen_pride-and-prejudice.epub?source=download"
fi
if [ ! -s "$dest/gpt3.pdf" ]; then
  curl -fsSL -o "$dest/gpt3.pdf" "https://arxiv.org/pdf/2005.14165"
fi

rm -rf "$HOME/Library/Application Support/$bundle_id" /tmp/superreader-spike
probe="/tmp/superreader-spike/reader.json"
work="$(mktemp -d)"
cleanup() {
  killall "$name" >/dev/null 2>&1 || true
  rm -rf "$work"
}
trap cleanup EXIT

cat > "$work/windows.swift" << 'EOF'
import AppKit
import CoreGraphics
import Foundation

if CommandLine.arguments.count > 2 && CommandLine.arguments[1] == "--lights" {
    let path = CommandLine.arguments[2]
    guard let image = NSImage(contentsOfFile: path),
          let tiff = image.tiffRepresentation,
          let rep = NSBitmapImageRep(data: tiff) else {
        fputs("could not read \(path)\n", stderr)
        exit(1)
    }
    let width = rep.pixelsWide
    let height = rep.pixelsHigh
    var minX = width
    var minY = height
    var maxX = 0
    var maxY = 0
    var lights = 0
    let band = min(80, height)
    let span = min(180, width)
    for y in 0..<band {
        for x in 0..<span {
            guard let color = rep.colorAt(x: x, y: y)?.usingColorSpace(.sRGB) else { continue }
            let r = Int(color.redComponent * 255)
            let g = Int(color.greenComponent * 255)
            let b = Int(color.blueComponent * 255)
            let red = r > 180 && g < 120 && b < 120
            let yellow = r > 180 && g > 140 && b < 100
            let green = g > 140 && r < 140 && b < 140
            if red || yellow || green {
                lights += 1
                minX = min(minX, x)
                minY = min(minY, y)
                maxX = max(maxX, x)
                maxY = max(maxY, y)
            }
        }
    }
    if lights < 100 {
        fputs("traffic lights are missing in \(path)\n", stderr)
        exit(1)
    }
    var ink = 0
    for y in minY...maxY {
        for x in minX...maxX {
            guard let color = rep.colorAt(x: x, y: y)?.usingColorSpace(.sRGB) else { continue }
            let r = Int(color.redComponent * 255)
            let g = Int(color.greenComponent * 255)
            let b = Int(color.blueComponent * 255)
            if r < 70 && g < 70 && b < 70 {
                ink += 1
            }
        }
    }
    if ink > 8 {
        fputs("traffic lights have \(ink) ink pixels in \(path)\n", stderr)
        exit(1)
    }
    var title = 0
    let x0 = width / 2
    let x1 = width * 68 / 100
    let y1 = min(40, height)
    for y in 14..<y1 {
        for x in x0..<x1 {
            guard let color = rep.colorAt(x: x, y: y)?.usingColorSpace(.sRGB) else { continue }
            let r = Int(color.redComponent * 255)
            let g = Int(color.greenComponent * 255)
            let b = Int(color.blueComponent * 255)
            if r < 140 && g < 140 && b < 140 {
                title += 1
            }
        }
    }
    if title < 40 {
        fputs("toolbar title is missing in \(path)\n", stderr)
        exit(1)
    }
    exit(0)
}

let want = CommandLine.arguments.count > 1 ? CommandLine.arguments[1] : ""
let nameWant = CommandLine.arguments.count > 2 ? CommandLine.arguments[2] : ""
let info = CGWindowListCopyWindowInfo([.optionOnScreenOnly, .excludeDesktopElements], kCGNullWindowID) as? [[String: Any]] ?? []

func number(_ value: Any?) -> Int {
    if let value = value as? Int { return value }
    if let value = value as? NSNumber { return value.intValue }
    return 0
}

var alert = false
var windowId = 0
var best = 0.0
var title = ""
for window in info {
    let owner = window[kCGWindowOwnerName as String] as? String ?? ""
    let name = window[kCGWindowName as String] as? String ?? ""
    let bounds = window[kCGWindowBounds as String] as? [String: Any] ?? [:]
    let width = bounds["Width"] as? Double ?? (bounds["Width"] as? NSNumber)?.doubleValue ?? 0
    let height = bounds["Height"] as? Double ?? (bounds["Height"] as? NSNumber)?.doubleValue ?? 0
    let text = (owner + " " + name).lowercased()
    if text.contains("could not be opened") || text.contains("cannot open") || text.contains("cannot be opened") {
        fputs("alert \(owner): \(name)\n", stderr)
        alert = true
    }
    if owner == want && (nameWant.isEmpty || name.localizedCaseInsensitiveContains(nameWant)) {
        let area = width * height
        if area > best {
            best = area
            windowId = number(window[kCGWindowNumber as String])
            title = name
        }
    }
}
if alert { exit(2) }
if windowId == 0 { exit(3) }
print("\(windowId)\t\(title)")
EOF
swiftc -O -o "$work/windows" "$work/windows.swift"

ready() {
  mode="$1"
  suffix="$2"
  NAME="$name" MODE="$mode" SUFFIX="$suffix" PROBE="$probe" python3 - << 'PY'
import json, os, sys
path = os.environ["PROBE"]
if not os.path.exists(path):
    sys.exit(1)
data = json.load(open(path))
ok = (
    data.get("window_title") == os.environ["NAME"]
    and data.get("mode") == os.environ["MODE"]
    and data.get("product") is True
    and float(data.get("bar_w") or 0) > 8
    and float(data.get("lights") or 0) >= 0.9
    and float(data.get("close_w") or 0) > 4
    and data.get("toolbar") is True
)
if os.environ["MODE"] != "empty":
    ok = ok and str(data.get("path") or "").endswith(os.environ["SUFFIX"])
    ok = ok and int(data.get("pages") or 0) > 1 and int(data.get("page") or 0) >= 1
sys.exit(0 if ok else 1)
PY
}

wait_reader() {
  mode="$1"
  suffix="$2"
  for _ in $(seq 1 240); do
    set +e
    "$work/windows" "$name" >/dev/null 2>"$work/alert.txt"
    status=$?
    set -e
    if [ "$status" -eq 2 ]; then
      echo "an alert window is on screen" >&2
      cat "$work/alert.txt" >&2
      exit 1
    fi
    if ready "$mode" "$suffix"; then
      return 0
    fi
    sleep 0.5
  done
  echo "reader did not become $mode" >&2
  if [ -f "$probe" ]; then
    cat "$probe" >&2
  fi
  if [ -f /tmp/superreader-spike/opens.log ]; then
    cat /tmp/superreader-spike/opens.log >&2
  fi
  exit 1
}

capture() {
  file="$1"
  line="$("$work/windows" "$name")"
  id="${line%%$'\t'*}"
  title="${line#*$'\t'}"
  if [ -n "$title" ] && [ "$title" != "$name" ]; then
    echo "window title is $title" >&2
    exit 1
  fi
  screencapture -l "$id" -o "$root/dist/$file"
  if [ ! -s "$root/dist/$file" ]; then
    echo "screencapture wrote no $file" >&2
    exit 1
  fi
}

capture_owner() {
  owner="$1"
  match="$2"
  file="$3"
  set +e
  if [ -n "$match" ]; then
    line="$("$work/windows" "$owner" "$match" 2>/dev/null)"
  else
    line="$("$work/windows" "$owner" 2>/dev/null)"
  fi
  status=$?
  set -e
  if [ "$status" -ne 0 ] || [ -z "$line" ]; then
    echo "did not capture $file" >&2
    return 0
  fi
  id="${line%%$'\t'*}"
  screencapture -l "$id" -o "$root/dist/$file" || echo "screencapture failed for $file" >&2
}

show_finder() {
  set +e
  osascript <<EOF
tell application "Finder"
  activate
  set distFolder to POSIX file "$root/dist" as alias
  set w to make new Finder window to distFolder
  set current view of w to icon view
  set toolbar visible of w to false
  set bounds of w to {80, 80, 720, 520}
  tell icon view options of w
    set icon size to 128
  end tell
  select file "${name}.app" of distFolder
end tell
EOF
  status=$?
  set -e
  if [ "$status" -ne 0 ]; then
    echo "Finder did not open the built app" >&2
    return 0
  fi
  sleep 0.8
}

open "$app"
wait_reader empty ""
sleep 0.4
capture "${name}-launch.png"

open -a "$app" "$dest/pride-se.epub"
wait_reader epub "pride-se.epub"
sleep 0.4
capture "${name}-epub.png"

open -a "$app" "$dest/gpt3.pdf"
wait_reader pdf "gpt3.pdf"
sleep 0.4
capture "${name}-pdf.png"
if ! "$work/windows" --lights "$root/dist/${name}-pdf.png"; then
  echo "the PDF shot draws in the traffic lights" >&2
  exit 1
fi

capture_owner "Dock" "" "${name}-dock.png"
show_finder
capture_owner "Finder" "dist" "${name}-finder.png"

echo "packaged open ok"
