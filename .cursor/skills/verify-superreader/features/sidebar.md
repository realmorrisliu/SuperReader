# Sidebar

The left bar has one fixed width. That width centers the traffic lights. The width is read from AppKit after the title bar lays the buttons out. It is the zoom button's right edge plus the close button's left edge.

## Sub-features

- `sidebar-centered` reads the close, miniaturize, and zoom frames from the running window and the bar width from `chrome.json`. The left and right gaps differ by at most 0.5 points.
- `sidebar-fixed` resizes the window. The bar width stays within 0.5 points, and the gaps stay centered.
- `sidebar-immersive` flips the immersive flag on and off. The bar is measured again and the width stays within 0.5 points.
- `sidebar-fullscreen` enters and leaves full screen. The lights stay centered, and the windowed width returns. Full screen keeps the buttons out of the accessibility tree, so the check measures the painted column against the AppKit width.

## How to get to it (user POV)

- Launch with `SPIKE_MODE=webkit`.
- The left column is as wide as the traffic lights plus the same gap on both sides.
- Resize the window. The column does not grow or shrink with it.
- Full screen and immersive reading measure the buttons again. A plain resize does not.

## Driving it with scripts/verify

Preconditions:

- `scripts/verify --doctor` passes.
- No other `spike-shell` is running.
- `/tmp/superreader-spike/shell-frame`, `bar-immersive`, and `bar-fullscreen` are removed before the chrome shots.

- **Centered.** Launch Pride at 1100 by 830. Hover the top edge so the buttons are visible. Read the button frames and the bar width. The gaps differ by at most 0.5 points.
- **Resize.** Write `1280 830` to `shell-frame`. The bar width stays within 0.5 points.
- **Immersive.** Write `1` to `bar-immersive`, then `0`. The width stays within 0.5 points after each measurement.
- **Full screen.** Write `1` to `bar-fullscreen`, then `0`. The painted column matches the AppKit width within 0.5 points, and the AppKit gaps match. After leaving full screen, Accessibility reads the buttons and the width matches the windowed width within 0.5 points.
- **Proof.** The pictures are `bar-window.png` and `bar-lights.png`.

## Gotchas

- The width is not a constant. A check that types in 72 or 52 is wrong.
- The buttons are hidden until the pointer is at the top. Measure them after the hover. In full screen the pointer has to sit on the screen's top edge, and the buttons may live on the menu bar rather than the window.
- The bar rows read `/tmp/superreader-spike/chrome-<pid>.json`. A shared `chrome.json` from another shell is ignored.
- `shell-frame` pins the window. Chrome shots must not inherit that file.
- Immersive reading's card, menu, and toolbar button are not this check. The flag only asks the bar to measure again.
