#!/usr/bin/env bash
# Captures the reading window for the IconMCP contents-icon tracer.
# Kills only the spike-shell pid this script starts.
set -euo pipefail

root="$(cd "$(dirname "$0")/.." && pwd)"
bin="$root/spikes/target/release/spike-shell"
out="$root/docs/prototypes/icons"
finder="$root/spikes/target/window-id"
mkdir -p "$out" "$root/spikes/target"

if [[ ! -x "$bin" ]]; then
  echo "missing $bin" >&2
  exit 1
fi

cat > "$root/spikes/target/window-id.swift" <<'SWIFT'
import CoreGraphics
import AppKit
guard CommandLine.arguments.count > 1, let pid = Int32(CommandLine.arguments[1]) else {
  fputs("pid\n", stderr)
  exit(2)
}
let info = CGWindowListCopyWindowInfo([.optionOnScreenOnly], kCGNullWindowID) as! [[String: Any]]
var bestArea: CGFloat = 0
var best = ""
for window in info {
  let owner = window[kCGWindowOwnerPID as String] as? Int32 ?? -1
  let layer = window[kCGWindowLayer as String] as? Int ?? -1
  guard owner == pid, layer == 0, let dict = window[kCGWindowBounds as String] as? [String: CGFloat] else { continue }
  let width = dict["Width"] ?? 0
  let height = dict["Height"] ?? 0
  let area = width * height
  if area <= bestArea { continue }
  bestArea = area
  let number = window[kCGWindowNumber as String] as? Int ?? 0
  best = "\(number)"
}
if best.isEmpty { exit(1) }
print(best)
SWIFT
swiftc -O -o "$finder" "$root/spikes/target/window-id.swift"

shot() {
  local name="$1"
  local appearance="$2"
  local icon="$3"
  local support
  support="$(mktemp -d /tmp/superreader-icon-shot.XXXXXX)"
  local pid
  SUPERREADER_SUPPORT_DIR="$support" \
    SPIKE_BOOK=pride \
    SPIKE_MODE=webkit \
    SPIKE_LIGHTS=1 \
    SPIKE_APPEARANCE="$appearance" \
    SPIKE_ICON="$icon" \
    "$bin" >/tmp/iconmcp-shot-"$name".log 2>&1 &
  pid=$!
  local id=""
  local i
  for i in 1 2 3 4 5 6 7 8 9 10 11 12 13 14 15 16 17 18 19 20; do
    if ! kill -0 "$pid" 2>/dev/null; then
      echo "$name exited early" >&2
      tail -n 40 /tmp/iconmcp-shot-"$name".log >&2 || true
      exit 1
    fi
    if id="$("$finder" "$pid" 2>/dev/null)"; then
      break
    fi
    sleep 0.5
  done
  if [[ -z "$id" ]]; then
    echo "$name: no window for $pid" >&2
    kill "$pid" 2>/dev/null || true
    wait "$pid" 2>/dev/null || true
    exit 1
  fi
  sleep 1.2
  screencapture -l "$id" -o "$out/$name.png"
  kill "$pid" 2>/dev/null || true
  wait "$pid" 2>/dev/null || true
  rm -rf "$support"
  echo "wrote $out/$name.png"
}

shot before-light light 0
shot before-dark dark 0
shot after-light light 1
shot after-dark dark 1
shot reference-light light sf
shot reference-dark dark sf

if [[ -f /tmp/superreader-spike/chrome.json ]]; then
  jq -r '"measured gap \(.icon_gap) pt, vertical delta \(.icon_dy) pt"' /tmp/superreader-spike/chrome.json
fi

cropper="$root/spikes/target/icons-crop"
cat > "$root/spikes/target/icons-crop.swift" <<'SWIFT'
import AppKit
import CoreGraphics
import CoreText
import ImageIO
import UniformTypeIdentifiers

struct Job {
  let source: String
  let dest: String
  let caption: String?
}

let root = CommandLine.arguments[1]
let out = "\(root)/docs/prototypes/icons"
let jobs = [
  Job(source: "before-light.png", dest: "before-light-toolbar.png", caption: nil),
  Job(source: "before-dark.png", dest: "before-dark-toolbar.png", caption: nil),
  Job(source: "after-light.png", dest: "after-light-toolbar.png", caption: nil),
  Job(source: "after-dark.png", dest: "after-dark-toolbar.png", caption: nil),
  Job(source: "reference-light.png", dest: "reference-light-toolbar.png", caption: "reference, not shipped"),
  Job(source: "reference-dark.png", dest: "reference-dark-toolbar.png", caption: "reference, not shipped"),
]

func load(_ name: String) -> CGImage {
  let url = URL(fileURLWithPath: "\(out)/\(name)")
  guard let src = CGImageSourceCreateWithURL(url as CFURL, nil),
        let image = CGImageSourceCreateImageAtIndex(src, 0, nil) else {
    fputs("missing \(name)\n", stderr)
    exit(1)
  }
  return image
}

func write(_ image: CGImage, _ name: String) {
  let url = URL(fileURLWithPath: "\(out)/\(name)") as CFURL
  guard let dest = CGImageDestinationCreateWithURL(url, UTType.png.identifier as CFString, 1, nil) else {
    exit(1)
  }
  CGImageDestinationAddImage(dest, image, nil)
  CGImageDestinationFinalize(dest)
}

func toolbarCrop(_ image: CGImage) -> CGImage {
  let width = min(520, image.width)
  let height = min(180, image.height)
  guard let crop = image.cropping(to: CGRect(x: 0, y: 0, width: width, height: height)) else {
    exit(1)
  }
  let scaled = CGSize(width: width * 2, height: height * 2)
  let color = CGColorSpace(name: CGColorSpace.sRGB)!
  guard let ctx = CGContext(
    data: nil,
    width: Int(scaled.width),
    height: Int(scaled.height),
    bitsPerComponent: 8,
    bytesPerRow: 0,
    space: color,
    bitmapInfo: CGImageAlphaInfo.premultipliedLast.rawValue
  ) else { exit(1) }
  ctx.interpolationQuality = .high
  ctx.draw(crop, in: CGRect(origin: .zero, size: scaled))
  return ctx.makeImage()!
}

func labelled(_ image: CGImage, _ caption: String) -> CGImage {
  let pad = 28
  let width = image.width
  let height = image.height + pad
  let color = CGColorSpace(name: CGColorSpace.sRGB)!
  guard let ctx = CGContext(
    data: nil,
    width: width,
    height: height,
    bitsPerComponent: 8,
    bytesPerRow: 0,
    space: color,
    bitmapInfo: CGImageAlphaInfo.premultipliedLast.rawValue
  ) else { exit(1) }
  ctx.setFillColor(CGColor(gray: 1, alpha: 1))
  ctx.fill(CGRect(x: 0, y: 0, width: width, height: height))
  ctx.draw(image, in: CGRect(x: 0, y: pad, width: image.width, height: image.height))
  drawCaption(ctx, caption, 16, 6)
  return ctx.makeImage()!
}

func drawCaption(_ ctx: CGContext, _ text: String, _ x: CGFloat, _ baseline: CGFloat) {
  let font = NSFont.systemFont(ofSize: 22, weight: .medium)
  let attrs: [NSAttributedString.Key: Any] = [
    .font: font,
    .foregroundColor: NSColor.black,
  ]
  let line = CTLineCreateWithAttributedString(NSAttributedString(string: text, attributes: attrs))
  ctx.saveGState()
  ctx.textMatrix = .identity
  ctx.textPosition = CGPoint(x: x, y: baseline)
  CTLineDraw(line, ctx)
  ctx.restoreGState()
}

var cropped: [String: CGImage] = [:]
for job in jobs {
  let image = toolbarCrop(load(job.source))
  cropped[job.source] = image
  if let caption = job.caption {
    write(labelled(image, caption), job.dest)
  } else {
    write(image, job.dest)
  }
  print("wrote \(out)/\(job.dest)")
}

func sheet(_ leftName: String, _ rightName: String, _ dest: String) {
  let left = cropped[leftName]!
  let right = cropped[rightName]!
  let gap = 24
  let captionH = 36
  let width = left.width + gap + right.width
  let height = max(left.height, right.height) + captionH
  let color = CGColorSpace(name: CGColorSpace.sRGB)!
  guard let ctx = CGContext(
    data: nil,
    width: width,
    height: height,
    bitsPerComponent: 8,
    bytesPerRow: 0,
    space: color,
    bitmapInfo: CGImageAlphaInfo.premultipliedLast.rawValue
  ) else { exit(1) }
  ctx.setFillColor(CGColor(gray: 1, alpha: 1))
  ctx.fill(CGRect(x: 0, y: 0, width: width, height: height))
  ctx.draw(left, in: CGRect(x: 0, y: captionH, width: left.width, height: left.height))
  ctx.draw(right, in: CGRect(x: left.width + gap, y: captionH, width: right.width, height: right.height))
  drawCaption(ctx, "reference, not shipped", 16, 10)
  drawCaption(ctx, "IconMCP", CGFloat(left.width + gap + 16), 10)
  write(ctx.makeImage()!, dest)
  print("wrote \(out)/\(dest)")
}

sheet("reference-light.png", "after-light.png", "sheet-light.png")
sheet("reference-dark.png", "after-dark.png", "sheet-dark.png")
SWIFT
swiftc -O -o "$cropper" "$root/spikes/target/icons-crop.swift"
"$cropper" "$root"
