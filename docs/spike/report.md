# Spike report

The window is `spike-shell`. The numbers come from `spike-measure`, a release binary that does not link AppKit. PDF children re-exec that binary, alternating pdfium and hayro, five times at each scale. Each child reads the GPT-3 paper, 75 pages, and checks that every bitmap length is width times height times 4. That campaign ran on Rust 1.93. The shell later needed 1.98.1 because `nsh-core` 0.3.0 uses `if let` guards.

Retina 1x means 2 pixels per PDF point, drawn at the page point size. Zoom 2x means 4 pixels per point, drawn at twice the point size. The old bitmap is dropped when the scale changes. It is not stretched.

Open-to-first is the child clock from `main` through the first raster. Later-page time is every page after the first, pooled across the five runs. RSS is `ps -o rss` after the last page, one sample per run, not a peak sampler.

The limiter is the raster. A few pages take 150 ms to 320 ms, so the max is not the typical page. The median is. Errors were 0 after pdfium was given ownership of the file bytes. An earlier run borrowed the bytes, freed them, and then failed on page 3. That run is not in the table.

## PDF

| Engine | Scale | Runs | Open to first median | First raster median | Later page median | RSS median | Phrase hits |
|---|---|---|---|---|---|---|---|
| pdfium | retina-1x | 5 | 51.6 ms, range 39.6 to 118 | 47.5 ms | 9.8 ms, range 0.7 to 294 | 238 MB | 2 of 2 |
| hayro | retina-1x | 5 | 9.5 ms, range 9.1 to 37.9 | 7.9 ms | 17.3 ms, range 1.8 to 321 | 412 MB | 1 of 2 |
| pdfium | zoom-2x | 5 | 51.5 ms, range 46.3 to 59.8 | 47.0 ms | 14.0 ms, range 2.2 to 264 | 344 MB | 2 of 2 |
| hayro | zoom-2x | 5 | 19.2 ms, range 17.7 to 20.0 | 17.6 ms | 18.3 ms, range 4.2 to 158 | 527 MB | 1 of 2 |

The phrases are `Few-Shot Learners` and `GPT-3` on page 0. pdfium returned 2467 glyph boxes. hayro returned 2080. hayro's Unicode walk is the experimental glyph-run path, and it missed one phrase.

Raw rows are in `pdf-summary.json`.

![pdfium at Retina 1x](pdfium-1x.png)

![hayro at Retina 1x](hayro-1x.png)

![pdfium at zoom 2x](pdfium-2x.png)

![pdfium highlight for the word Abstract](pdfium-highlight.png)

The zoom capture is a new raster. The type is larger and still sharp. The highlight is the word `Abstract` painted from stored page-point quads. On this page the yellow bar sits just above the word, so the boxes are close and not yet tight. The anchor still holds page 0, the character range, the string, and the quads.

## EPUB

rbook opens both books. Pride and Prejudice chapter 1 is `/epub/text/chapter-1.xhtml`, 4440 plain characters, and it contains "truth universally acknowledged". 朝花夕拾 is 27105 plain characters in the first long spine item.

Blitz 0.3.0-beta.4 did not honor `column-width`. `blitz_columns` was false. With the height lock removed, Pride chapter 1 laid out as 1 page at 720 by 960 points, and 朝花夕拾 laid out as 9 pages. Those 9 pages are vertical slices of one column. Resolve plus the first slice was about 10 to 18 ms. Later slices were about 3 ms. That is not publisher CSS pagination.

TextView, in the NSH window, shows 朝花夕拾 as one scrolling column. CJK is present. There is no page break. NSH does not hand the selection string back to the app, so this view does not produce an anchor.

![TextView showing 朝花夕拾](textview-cjk.png)

A custom WKWebView paginator was not finished. The built-in web view cannot load `file:` or a custom scheme on this tag. That view is not in these screenshots.

## Tests

`cargo test --manifest-path spikes/Cargo.toml -p spike-report -p spike-epub` passed the anchor round-trip, the median check, and the EPUB character-range anchor. `nsh test` does not see inside a native view, so it was not used for the page pixels.

## Recommendation

Use pdfium for the PDF page. Its later-page median stays under 16.7 ms at both scales, and the character boxes are complete enough to hang a comment on. The risk is shipping `libpdfium.dylib`, and a cold open near 50 ms that is mostly the first raster.

hayro opens faster and needs no dylib. The risk is the text. It missed a phrase, returned fewer glyphs, and its later-page median is over 16.7 ms.

For EPUB pages, do not pick Blitz. It did not paginate with CSS columns. TextView is a credible plain chapter, including CJK, and it is not a paginator and it does not yield an anchor. The page engine still to prove in a window is WKWebView.
