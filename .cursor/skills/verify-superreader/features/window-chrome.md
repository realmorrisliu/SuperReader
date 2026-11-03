# Window chrome

Three reading-window prototypes, selected with `SPIKE_CHROME`. Unset, the spike keeps the system title bar and the 1100 by 1400 window. None of the prototypes is the default.

## Sub-features

- `chrome-a-aspect`, `chrome-b-aspect`, and `chrome-c-aspect` open Pride at paperback. The window matches the page plus the mats, within 2 percent and 8 points, and it fits the visible frame.
- `chrome-a-clear`, `chrome-b-clear`, and `chrome-c-clear` keep the page box below the traffic lights or the thin bar.
- `chrome-a-footer`, `chrome-b-footer`, and `chrome-c-footer` show `N/M页` and `本章还剩X页`.
- `chrome-a-trim`, `chrome-b-trim`, and `chrome-c-trim` switch the saved trim to US Trade and the window follows.
- `chrome-a-zhaohua-footer`, `chrome-b-zhaohua-footer`, and `chrome-c-zhaohua-footer` show the same footer on 朝花夕拾.
- `chrome-a-fullscreen-footer`, `chrome-b-fullscreen-footer`, and `chrome-c-fullscreen-footer` keep that footer in full screen.
- `chrome-a-hover` and `chrome-b-hover` show the traffic lights when `SPIKE_CHROME_SHOW=1`.

## How to get to it (user POV)

- Launch with `SPIKE_MODE=webkit` and `SPIKE_CHROME` set to `a`, `b`, or `c`.
- `a` hides the title. The traffic lights sit on the paper and leave when the pointer leaves the top edge. The window aspect is locked.
- `b` fades the traffic lights and the book title in together near the top. The window is not locked. Extra space becomes even margins.
- `c` keeps a 32 point paper bar with the book title centered. The window aspect is locked.
- Choose a trim from the View menu. The window resizes and keeps the top edge.
- Enter full screen from the Window menu. The page stays centered, the footer stays, and the aspect lock is cleared.

## Driving it with scripts/verify

Preconditions:

- `scripts/verify --doctor` passes.
- The settings file is restored after the chrome shots.

- **Paperback.** Write `paperback`, launch Pride with `SPIKE_CHROME`, and wait until `chrome.json` reports a page and `webkit.json` reports the chapter. The window from `CGWindowListCopyWindowInfo` matches `chrome_plan`. The page top clears the title bar. Vision reads the footer.
- **Trim.** Write `trade` while that process is up. The window ratio follows 140 by 216.
- **朝花夕拾.** Launch with `SPIKE_BOOK=zhaohua` and `SPIKE_AT=3`.
- **Hover.** Launch `a` and `b` with `SPIKE_CHROME_SHOW=1`. `lights` in `chrome.json` reaches 1.
- **Full screen.** Launch with `SPIKE_FULLSCREEN=1`. `fullscreen` is true and the footer is in the picture.
- **Proof.** The rows are `pass`. The pictures are copied to `docs/prototypes/window-chrome/`.

## Gotchas

- `child_env` strips `SPIKE_CHROME`, `SPIKE_CHROME_SHOW`, and `SPIKE_FULLSCREEN`.
- A resting shot parks the pointer at the bottom of the screen so the lights stay hidden.
- Full screen uses the whole display for a few seconds.
- The page edge used to ask a tagged `NSColor` for `CGColor` and aborted on this OS. It now builds the color with `CGColorCreateSRGB`.
