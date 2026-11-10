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
