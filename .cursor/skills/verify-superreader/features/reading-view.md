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
- `justify-pride-*` and `justify-plain-en-*` score every page of the first chapter. Gap p95 stays at or under 1.5, the widest gap at or under 2, loose lines at or under 3 percent, hyphenated lines at or under 15 percent with at most two in a row. The column's padding box stays the page's own measure: `box_err` is at most 0.5 px, and every non-last owned line meets both edges of that box within 0.5 px.
- `justify-pride-32kai`, `justify-pride-paperback`, and `justify-pride-trade` score the Pride opening. Gap p95 stays at or under 1.5, the max at or under 2.0, loose lines at or under 3 percent, rivers at or under 1, English hyphens at or under 15 percent and at most two in a row, and overflow at 0.
- `justify-plain-en-paperback`, `justify-plain-en-32kai`, and `justify-plain-en-trade` use the same bar on the unstyled English fixture.
- `justify-zhaohua-32kai`, `justify-zhaohua-paperback`, `justify-zhaohua-trade`, `justify-plain-zh-paperback`, `justify-plain-zh-32kai`, and `justify-plain-zh-trade` keep CJK on the native line boxes. `owned` is 0 and `forbidden_starts` is 0. `dash_starts` is 0: no line opens on a lone em dash. `orphan_dash_ends` is 0: no line ends with one half of `——` while the next line opens on the other half. A line may still open on an intact `——`.
- `justify-styled` keeps Literata left aligned and `owned` at 0.
- `justify-publisher-paperback`, `justify-publisher-32kai`, and `justify-publisher-trade` are the Literata book whose own sheet sets justify. While `OWN_PUBLISHER` is false, `owned` stays 0. The gap bar applies only when that constant is true.
- `reading-cold-open` is five release opens of Pride at paperback with the break cache deleted before each fitted open, interleaved with native opens. The p95 is the maximum, and the floor is 300 ms.
- `reading-warm-open` primes the cache once, then repeats that five-open pattern. The floor is 300 ms.
- `reading-warm-cached` reopens the primed chapter once. `break_source` is `cached` and `live_calls` is 0. It does not use the 300 ms bar.
- `reading-chapter-turn` is five fitted turns into the next chapter, interleaved with native turns. After the open chapter's layout is done, the same web view loads the next chapter. The timer starts at that load and stops when the new chapter reports a page. The floor is 300 ms. The offscreen preload is gone.

## How to get to it (user POV)

- Launch with `SPIKE_MODE=webkit`.
- Open the View menu and choose 32开, Paperback, or US Trade. Paperback is the default when the settings file is missing. The check mark is the saved trim.
- A fixed-layout book and a PDF keep their own page box.
- Press the arrow keys, or swipe horizontally on the trackpad, to turn one page.

## Driving it with scripts/verify

Preconditions:

- `scripts/verify --doctor` passes.
- `spikes/epub/fixtures/Literata.ttf` is present. The script builds `styled.epub`, `publisher.epub`, `plain-en.epub`, `plain-zh.epub`, and `fixed.epub` under the fixtures directory.

- **Trim.** Write `settings.json`, launch Pride with `SPIKE_SELECT=1`, and wait until `webkit.json` reports `page_w / page_h` within 1 percent of the trim and the anchor text is unchanged. Capture each ratio.
- **朝花夕拾.** Launch with `SPIKE_BOOK=zhaohua` and `SPIKE_AT=3`. Capture page 3, then the same prose after the trim file changes.
- **Type.** Launch `styled`, `plain-en`, and `plain-zh`. The `sample` object is the computed paragraph style.
- **Fixed.** Launch `SPIKE_BOOK=fixed`. Change the trim file to `trade`. `page_source` stays `declared` and the ratio stays square.
- **Slide.** Launch Pride with `SPIKE_PLAY=1`. Record five seconds. Copy `frame_ms`.
- **Proof.** The rows are `pass`. The pictures and `slide.mp4` are copied to `docs/prototypes/reading-view/`.

## Gotchas

- `child_env` strips `SPIKE_PLAY`, `SPIKE_AT`, `SPIKE_JUSTIFY`, and `SPIKE_NEXT`, so a parent value cannot leak into the Pride shot. Unset `SPIKE_JUSTIFY` leaves the line-breaking pass on. `SPIKE_JUSTIFY=0` posts a native census. `SPIKE_NEXT=1` loads the next chapter in the same web view and records `chapter_ms`.
- The footer is `N/M页` and `本章还剩X页`. The older English chapter line is the PDF footer.
- An em dash that follows a word gets a word joiner (U+2060) before it. A pair of dashes stays a pair, so a line may still open on `——`. The joiner is not a nowrap span.
- The title fades, so the Pride check reads `title` from `webkit.json` as well as the picture.
- `frame_ms` is the animation-frame gap inside the page. A gap around 17 ms is one refresh. A gap of 33 ms fails.
- The settings file for a verify run is inside that run's `SUPERREADER_SUPPORT_DIR`, not the reader's `~/Library` folder. The script restores the file it saved at the start of the reading checks.
