# Reading view

One EPUB page on the WKWebView. The book's font, size, line height, indent, alignment, and paragraph spacing win. Unstyled English uses Iowan Old Style at 22px. Unstyled Chinese uses Songti SC at 20px. The page box uses the saved trim, and a fixed-layout book keeps its own viewport.

## Sub-features

- `trim-32kai-pride`, `trim-paperback-pride`, and `trim-trade-pride` show the Pride opening. Each page ratio matches that trim within 1 percent. Switching trim keeps the highlight text.
- `trim-32kai-zhaohua`, `trim-paperback-zhaohua`, and `trim-trade-zhaohua` show 朝花夕拾 prose from page 3 of 32开, then the same prose at the other trims.
- `reading-styled` keeps Literata at 19px, line height 1.65, indent 1.5em, and left alignment, and the face loads.
- `reading-plain-en` falls back to Iowan, 22px, line height 1.48, indent 2em, and justify.
- `reading-plain-zh` falls back to Songti, 20px, line height 1.70, indent 2em, and justify.
- `reading-fixed-ignores-trim` keeps a square viewport when the saved trim changes.
- `reading-slide` records the 160ms slide. Each animation-frame gap stays under two refreshes, 33.4 ms.
- `reading-default-trim` deletes the settings file and checks that Pride and 朝花夕拾 open at the paperback ratio.
- `reading-page-box` requires the page box center within 2 px of the stage center, and no dark vertical rule in the picture.
- `reading-text-complete` requires `Mr. Bennet` and `Mrs. Long` in Pride and the English test books, the Chinese source phrase in the Chinese books, and no two identical blocks in a row.
- `reading-text-clean` fails on U+FFFD, U+25A1, a private-use character, or a space between two CJK characters.

## How to get to it (user POV)

- Launch with `SPIKE_MODE=webkit`.
- Open the View menu and choose 32开, Paperback, or US Trade. Paperback is the default when the settings file is missing. The check mark is the saved trim.
- A fixed-layout book and a PDF keep their own page box.
- Press the arrow keys, or swipe horizontally on the trackpad, to turn one page.

## Driving it with scripts/verify

Preconditions:

- `scripts/verify --doctor` passes.
- `spikes/epub/fixtures/Literata.ttf` is present. The script builds `styled.epub`, `plain-en.epub`, `plain-zh.epub`, and `fixed.epub` under the fixtures directory.

- **Trim.** Write `settings.json`, launch Pride with `SPIKE_SELECT=1`, and wait until `webkit.json` reports `page_w / page_h` within 1 percent of the trim and the anchor text is unchanged. Capture each ratio.
- **朝花夕拾.** Launch with `SPIKE_BOOK=zhaohua` and `SPIKE_AT=3`. Capture page 3, then the same prose after the trim file changes.
- **Type.** Launch `styled`, `plain-en`, and `plain-zh`. The `sample` object is the computed paragraph style.
- **Fixed.** Launch `SPIKE_BOOK=fixed`. Change the trim file to `trade`. `page_source` stays `declared` and the ratio stays square.
- **Slide.** Launch Pride with `SPIKE_PLAY=1`. Record five seconds. Copy `frame_ms`.
- **Proof.** The rows are `pass`. The pictures and `slide.mp4` are copied to `docs/prototypes/reading-view/`.

## Gotchas

- `child_env` strips `SPIKE_PLAY` and `SPIKE_AT`, so a parent value cannot leak into the Pride shot.
- The footer is `N/M页` and `本章还剩X页`. The older English chapter line is the PDF footer.
- The title fades, so the Pride check reads `title` from `webkit.json` as well as the picture.
- `frame_ms` is the animation-frame gap inside the page. A gap around 17 ms is one refresh. A gap of 33 ms fails.
- The settings file is restored after the reading checks.
