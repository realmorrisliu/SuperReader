# Reading view prototypes

`SPIKE_READING` opens one of three single-page EPUB views on the WKWebView. The footer is Chinese. The title fades. A, B, and C turn the page in different ways. The switch is not a setting.

## Sub-features

- `reading-a-pride`, `reading-b-pride`, and `reading-c-pride` show Pride chapter 1 with `N/M页` and `本章还剩`.
- `reading-a-zhaohua`, `reading-b-zhaohua`, and `reading-c-zhaohua` show 范愛農, not the Gutenberg header.
- `reading-a-turn`, `reading-b-turn`, and `reading-c-turn` record a page turn. Each animation-frame gap stays under two refreshes, 33.4 ms. A gap around 17 ms is one refresh. A gap of 33 ms is a missed refresh and fails.
- `reading-text-clean` fails if a captured page contains U+FFFD, U+25A1, a private-use character, or a space between two CJK characters. 朝花夕拾 stills start at page 3, after the date line.

## How to get to it (user POV)

- Launch with `SPIKE_MODE=webkit` and `SPIKE_READING=a`, `b`, or `c`.
- Press `0` to cycle A, then B, then C.
- Press the arrow keys, or swipe horizontally on the trackpad, to turn one page.
- `9` does not reload a chapter that is already open. Set `SPIKE_BOOK=zhaohua` at launch.

## Driving it with scripts/verify

Preconditions:

- `scripts/verify --doctor` passes.
- `pride-se.epub` and `zhaohua.epub` are the paths doctor printed.

- **Still.** After the older window shots exit, launch each letter with Pride, then with `SPIKE_BOOK=zhaohua`. Wait until `webkit.json` reports more than one page and the chapter href. Sleep one second, then capture that window. The PNG shows body text and both footer phrases.
- **Turn.** Launch Pride again with `SPIKE_PLAY=1`. Record five seconds with `screencapture -l -V 5`. The copied `frame_ms` values are all at or under 16.7.
- **Proof.** The nine rows are `pass`. The pictures and the latency file are copied to `docs/prototypes/reading-view/`.

## Gotchas

- An unset `SPIKE_READING` keeps the English footer. `child_env` strips `SPIKE_READING` and `SPIKE_PLAY`, so a parent value cannot leak into `epub-pride`.
- The Chinese footer does not match `^\d+/\d+$`. The older EPUB checks stay on the English shot.
- A shot taken before the columns exist is one page. The watch requires a page count above 1.
- `frame_ms` is the animation-frame gap inside the page. `turn_ms` is a different clock and is not the floor.
- Key `0` reloads the open web view. Key `3` still does not mount one.
