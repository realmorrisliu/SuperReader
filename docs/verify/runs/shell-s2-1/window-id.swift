
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
  let screen = NSScreen.main
  let visible = screen?.visibleFrame ?? .zero
  let scale = screen?.backingScaleFactor ?? 1
  let screenH = screen?.frame.height ?? 0
  let x = dict["X"] ?? 0
  let y = dict["Y"] ?? 0
  best = "\(number) \(width) \(height) \(visible.width) \(visible.height) \(scale) \(screenH) \(x) \(y)"
}
if best.isEmpty { exit(1) }
print(best)
exit(0)
