# Window chrome

The reading window is the page. The traffic lights and the book title fade in together when the pointer is near the top. There is no `SPIKE_CHROME` switch.

## Sub-features

- `chrome-b-aspect` opens Pride at paperback. The window matches the page plus the side mats and the top mat, within 2 percent and 8 points.
- `chrome-b-clear` keeps the page box below the traffic lights.
- `chrome-b-footer` shows `N/M页` and `本章还剩X页`.
- `chrome-b-fill` keeps the gap under the last line of a full page smaller than one line.
- `chrome-b-measure` keeps the footer row on the same left and right edges as the text measure.
- `chrome-b-trim` switches the saved trim to US Trade and the window follows.
- `chrome-b-zhaohua-footer` shows the same footer on 朝花夕拾.
- `chrome-b-fullscreen-footer` keeps that footer in full screen.
- `chrome-b-fullscreen-page` scales the page to the screen height, centers it, keeps the footer on the measure, and leaves the CSS page at the windowed design width.
- `chrome-b-fullscreen-paper` keeps the top 60 px of the full-screen capture paper, `#f3eee4`. The toolbar hides with the menu bar.
- `chrome-b-hover` shows the traffic lights when the pointer is at the top of the window.

## How to get to it (user POV)

- Launch with `SPIKE_MODE=webkit`.
- At rest the window is paper. The lights and the title are hidden.
- Move the pointer to the top edge. The lights and the book title fade in.
- Choose a trim from the View menu. The window resizes and keeps the top edge.
- Enter full screen. The page grows to the screen height and stays centered. The type grows with the page.

## Driving it with scripts/verify

Preconditions:

- `scripts/verify --doctor` passes.
- The settings file is restored after the chrome shots.

- **Paperback.** Write `paperback`, launch Pride, and wait until `chrome.json` reports a page and `webkit.json` reports the chapter. The window from `CGWindowListCopyWindowInfo` matches `chrome_plan`. The page top clears the lights. Vision reads the footer. `fill_gap` is under `line_h`. The footer edges match the measure.
- **Trim.** Write `trade` while that process is up. The window ratio follows 140 by 216.
- **朝花夕拾.** Launch with `SPIKE_BOOK=zhaohua` and `SPIKE_AT=3`.
- **Hover.** Park the pointer away, then move it to the top of the window. `lights` in `chrome.json` reaches 1.
- **Full screen.** Launch with `SPIKE_FULLSCREEN=1`. `fullscreen` is true, the footer is in the picture, and the page fills the height under the top mat.
- **Proof.** The rows are `pass`. The pictures are copied to `docs/prototypes/window-chrome/`.

## Gotchas

- `child_env` strips `SPIKE_FULLSCREEN`.
- A resting shot parks the pointer at the bottom of the screen so the lights stay hidden.
- Full screen uses the whole display for a few seconds.
- The window id is the largest layer-0 window for the process. A fullscreen animation can list a short sliver first, and `screencapture -l` of that id fails.
- The page edge used to ask a tagged `NSColor` for `CGColor` and aborted on this OS. It now builds the color with `CGColorCreateSRGB`.
