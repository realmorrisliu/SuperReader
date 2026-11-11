
import AppKit
import ApplicationServices
import CoreGraphics
guard CommandLine.arguments.count > 1, let pid = pid_t(CommandLine.arguments[1]) else { exit(2) }
let app = AXUIElementCreateApplication(pid)
var raw: CFTypeRef?
guard AXUIElementCopyAttributeValue(app, kAXWindowsAttribute as CFString, &raw) == .success,
      let windows = raw as? [AXUIElement], !windows.isEmpty else {
  fputs("no window\n", stderr)
  exit(1)
}
func axValue(_ el: AXUIElement, _ name: String) -> CFTypeRef? {
  var value: CFTypeRef?
  guard AXUIElementCopyAttributeValue(el, name as CFString, &value) == .success else { return nil }
  return value
}
func box(_ el: AXUIElement) -> (CGFloat, CGFloat, CGFloat, CGFloat)? {
  guard let p = axValue(el, kAXPositionAttribute as String),
        let s = axValue(el, kAXSizeAttribute as String),
        CFGetTypeID(p) == AXValueGetTypeID(),
        CFGetTypeID(s) == AXValueGetTypeID() else { return nil }
  var pos = CGPoint.zero
  var size = CGSize.zero
  guard AXValueGetValue(p as! AXValue, .cgPoint, &pos),
        AXValueGetValue(s as! AXValue, .cgSize, &size) else { return nil }
  return (pos.x, pos.y, size.width, size.height)
}
var found: [String: (CGFloat, CGFloat, CGFloat, CGFloat)] = [:]
func walk(_ el: AXUIElement, _ depth: Int) {
  if depth > 14 { return }
  let role = axValue(el, kAXSubroleAttribute as String) as? String ?? ""
  if role == "AXCloseButton" || role == "AXMinimizeButton" || role == "AXZoomButton" || role == "AXFullScreenButton" {
    if let frame = box(el) { found[role] = frame }
  }
  var kids: CFTypeRef?
  if AXUIElementCopyAttributeValue(el, kAXChildrenAttribute as CFString, &kids) == .success,
     let kids = kids as? [AXUIElement] {
    for kid in kids { walk(kid, depth + 1) }
  }
}
var origin: (CGFloat, CGFloat, CGFloat, CGFloat)?
var bestArea: CGFloat = -1
for window in windows {
  walk(window, 0)
  if let frame = box(window) {
    let area = frame.2 * frame.3
    if area > bestArea {
      bestArea = area
      origin = frame
    }
  }
}
guard let origin,
      let close = found["AXCloseButton"],
      let mini = found["AXMinimizeButton"],
      let zoom = found["AXZoomButton"] ?? found["AXFullScreenButton"] else {
  fputs("missing \(Array(found.keys))\n", stderr)
  exit(1)
}
func local(_ item: (CGFloat, CGFloat, CGFloat, CGFloat)) -> String {
  let minX = item.0 - origin.0
  return "\(minX) \(minX + item.2) \(item.1 - origin.1) \(item.1 - origin.1 + item.3)"
}
print("close \(local(close)) mini \(local(mini)) zoom \(local(zoom))")
