# Native EPUB typesetter spike

This crate is a throwaway instrument. It answers whether a pure Rust typesetter can replace WKWebView for EPUB chapters. It is not wired into the reader. PDF stays on pdfium. The pull request is a draft and should not be merged as the product.

## Recommendation

Own the CSS subset on low level crates. That is engine B in the request, and option C in the desk research at the note this spike was given. Keep WKWebView as the reader until a paperback chapter meets the current line quality bar. The layout time bar is already met on the two chapters measured here.

Engine A, Blitz with Stylo, Taffy, and Parley, can paint a page. It does not paginate, it does not hyphenate, and it has no vertical text. Those are the reasons the research prices that path at 30 to 47 agent weeks. This spike did not remeasure that estimate.

## Reported run

The numbers below are one release run of `epub-native-spike` on 10 Nov 2026.

The machine had 4 cores. Uptime was 57 minutes. Load was 0.07, 0.18, 0.41 when the run started. Font bytes were already on disk. Cold layout is the first layout after the fonts are open. Warm layout is the next five layouts of the same chapter. The median is the reported figure. Font load is not included in layout time.

| Claim under test | Result |
|---|---|
| Warm Pride chapter 1 layout at or under 50 ms | Median 9.22 ms, range 8.87 to 9.51. Pass. |
| Warm 朝花夕拾 layout at or under 50 ms | Median 38.27 ms, range 36.36 to 44.67. Pass. |
| Word gap p95 at or under 1.37 | Pride p95 2.185, max 2.677. Miss. |
| Hyphenated lines at or under 10 percent | 9 of 112 lines, 8.0 percent. Pass. |
| Kinsoku violations at 0 | Pride 0. 朝花夕拾 10. Miss on the CJK chapter. |
| Overflow lines at 0 | Pride 0. 朝花夕拾 4. Miss on the CJK chapter. |

Pride cold layout was 9.50 ms. Item build and shaping took 8.98 ms. Breaking took 0.11 ms. Shaping is the limiter.

朝花夕拾 cold layout was 37.76 ms. Shaping took 28.25 ms. Breaking took 7.10 ms. Shaping is still the limiter. Breaking is the second cost on the longer book.

Font load on this run was 26.57 ms. That is the time to open EB Garamond and Noto Serif CJK SC, and it is not part of the 50 ms claim.

Page turn is a raster of an already laid out Pride page, five times. Paint plus PNG encode median is 9.95 ms, range 9.75 to 11.08. Paint alone median is 5.51 ms, range 5.35 to 5.81. Encode is about half of the turn.

The release typesetter binary is 4.2 MB. It does not link Blitz or Stylo.

## Where this disagrees with the desk research

The research pass bar of word gap p95 about 1.37 is the current reader. `docs/verify/summary.json` records the plain English paperback fixture at 1.3728. `docs/roadmap.md` records Pride chapter 1 at the settled paperback frame at 1.359, max 1.491, with 6 hyphenated lines out of 59. This typesetter measured 2.185 on that chapter. The research figures and this figure are different programs.

The research 50 ms budget is warm layout after shaping, not WebKit startup. This typesetter meets that budget on both chapters. The 朝花夕拾 warm maximum in the reported run is 44.67 ms, still under 50. An earlier warm median of 54.42 ms was taken while the load was 2.29 and is not this run. The research week counts, 30 to 47 for Blitz and 10 to 14 for a minimal owned engine, were not remeasured. They stay the research report's estimates.

The WebKit screenshots in `docs/spike` are a wide window, about 1100 by 887, with an ink column near 880 px. Pride chapter 1 is 2 pages there. The settled paperback frame in `docs/roadmap.md` is the same 534 by 819 used here. At that frame the current reader is 8 pages before the Knuth-Plass pass and 7 pages after it. This spike lays the chapter out in 6 pages, in EB Garamond rather than Iowan Old Style, so the 6 is not a score against those 7.

`docs/spike/report.md` recorded a Blitz resolve of 10 to 18 ms. The single Blitz run in this session resolved Pride in 26.6 ms. That is one shot, not a five run median, so it does not replace the earlier note. It also does not match this typesetter's 9.22 ms median, because the page boxes differ.

## Page and type

The page is 534 by 819. Paper is `#f3eee4`. Ink is `#1c1915`.

Latin side margin is 14 percent of the width. Top is 88 px. Bottom is 72 px. The built in Latin sheet is 22 px, line height 1.48, indent 2 em, justified, oldstyle figures, hyphenation on. The face on this machine is EB Garamond, OFL, standing in for Iowan Old Style. Iowan is not installed here.

CJK side margin is 11 percent. Top is 72 px. Bottom is 64 px. The built in CJK sheet is 20 px, line height 1.70, indent 2 em, justified, hyphenation off. The face is Noto Serif CJK SC, OFL, standing in for Songti SC. Songti SC is not installed here. WenQuanYi Micro Hei was not used.

Publisher declarations override that sheet. Gutenberg 朝花夕拾 sets line height 1.4em, and the pages use 1.4em.

## Engine A

Blitz stays behind the existing `blitz` feature of `spikes/epub`. The typesetter crate does not depend on it.

`spike-measure epub` was built in release and run once. The Blitz page is 720 by 960, painted at scale 2, so the buffer is 1440 by 1920. Column layout was not honored, so pagination is a vertical slice of one long column. Parley is a greedy breaker. This run counted lines by walking `inline_layout_data` and counted an emergency break when the debug form of the break reason contained `Emergency`.

Pride chapter 1 resolved in 26.6 ms and the first raster took 20.1 ms. Parley produced 76 lines, 0 emergency breaks, and 1 slice. The half scale PNG is `docs/spike/epub-native/blitz-pride.png`. The text is one wide ragged column. There are no hyphenation points.

The 朝花夕拾 file Blitz opened is the second HTML spine item, 28356 plain characters. The owned engine laid out the first spine item, the one that contains 小引. Blitz produced 7 slices, 540 Parley lines, and 0 emergency breaks. Resolve was 191 ms. Later slice paints were about 6 to 12 ms. These two runs are not the same chapter and not the same measure, so the times are not a head to head score.

Stylo, which Blitz links, is MPL-2.0. That license is acceptable and has to stay visible if engine A is ever taken further. Parley does not hyphenate, and Blitz has no fragmentation and no vertical text. A comparison that ignores those gaps would pick a winner for the wrong reason.

`spike-measure` itself is a 35 MB binary. It also links the PDF spike, so 35 MB is not the cost of Blitz alone.

## Engine B

`crates/epub-native-spike` parses one XHTML chapter with a hand rolled subset parser, shapes with harfrust 0.15.0, draws outlines with skrifa 0.49.0 into tiny-skia, and breaks lines with a Knuth-Plass port of the breaker already in `spikes/justify`. The container, OPF, and spine come from rbook 0.7.12. The `epub` crate was not used. The research report identifies that crate as GPL-3.0-or-later.

html5ever and cssparser were not linked. The hand rolled parser is enough for the two chapters, and it keeps this binary on MIT and Apache-2.0. icu_segmenter was not linked. Kinsoku is an explicit class table. A product version should use icu_segmenter for `line-break: strict`. This spike did not measure ICU.

Hyphenation uses the vendored en-US TeX patterns in `spikes/justify/vendor/hyphenated-en-us`. That file's LICENSE is MIT. The patterns match the JavaScript breaker, which is why they were reused instead of hypher.

Public layout does not expose box, glue, or penalty. A caret is a text node and a character offset. It does not store a page number. Two type sketches were written after the crate already had this shape. One keeps a styled tree and a caret, and hides box, glue, and penalty. The other flattens the chapter into a columnar tape and a page index, and also keeps the caret free of a page number. This crate follows the tree. The tape was not adopted. The caret rule from both sketches was kept.

## Line quality

The product breaker treats stretch as half the space width and shrink as a third. An adjustment ratio of 0.75 therefore draws a gap of 1.375 times the natural space. That is the 1.37 bar.

This breaker retries from the requested ratio, multiplies the limit by 1.6 up to eight times, then tries a limit of 8, then forces a break. An empty active set returns no solution so the limit can rise. The drawn ratio is not clamped to 0.75. Clamping the drawing would make the metric pass and would overflow the measure.

On Pride, moderate hyphenation leaves lines whose next word does not fit and whose hyphen points do not land in the slack. The line stretches. p95 is 2.185 and the worst gap is 2.677. A ratio clamped at 4 would draw a gap of 3 times the space. This run does not hit that clamp.

The same chapter under three hyphen settings:

| Setting | Hyphenated lines | Lines | Gap p95 | Overflow |
|---|---:|---:|---:|---:|
| Off | 0 | 113 | 2.952 | 0 |
| Moderate, penalty 200, minimum word 6, left and right minimum 3, max ratio 0.75 | 9 | 112 | 2.185 | 0 |
| TeX, penalty 50, left minimum 2, right minimum 3, max ratio 1.26 | 14 | 112 | 2.185 | 0 |

TeX hyphenates more lines and does not improve p95. The sheet is `docs/spike/epub-native/choices-hyphenation.png`, page 2, which is the first page where the moderate and TeX ink differ. Rightmost ink on that sheet, median per text row, is 396, 415, and 434 for off, moderate, and TeX.

A short last line used to loosen every earlier line, and p95 sat at 3.0 because the ratio hit the clamp. A zero width glue with stretch 10000 sits before the forced paragraph end, and the last line of a justified paragraph is placed at ratio 0. That is why p95 is 2.185 rather than 3.0. It is still above the bar.

## CJK rules and the three sheets

Opening punctuation is a half em box plus leading aki glue. The glue is dropped at the start of a line. Closing punctuation, commas, and periods are a half em box plus trailing aki glue. The glue is dropped at the end of a line. There is no break before a closing mark or after an opening mark.

Hanging mode draws a closing mark as a zero width box followed by glue of the full advance. Mid line, the glue keeps the next character off the mark. When the line breaks at that glue, the glue is discarded and the mark sits in the right margin.

The punctuation sheet is `docs/spike/epub-native/choices-punctuation.png`. Rightmost dark pixel per text row, median, is 470 for full width, 480 for JLREQ, and 490 for JLREQ plus hanging. The hanging panel reaches 501. The content edge of the other panels stays near 474 and 484.

CJK and Latin spacing replaces a whitespace run between the two scripts with a fixed glue of 0, one eighth em, or one quarter em. The sheet is `docs/spike/epub-native/choices-spacing.png`. The title line is `The Project Gutenberg eBook of 朝花夕拾`. At 0 the Han characters sit against `of`. At one eighth em a gap appears. At one quarter em the gap is wider. Median rightmost ink on that sheet is 399, 409, and 419.

A newline inside a text node is ordinary whitespace. Between two Han characters it adds no glue, so HTML wrapping does not invent a space. Between Latin words it becomes a space. An earlier render dropped those newlines and glued `restrictions` to `whatsoever` in the Gutenberg header. The committed sheet collapses them.

Product defaults are JLREQ, one quarter em, and moderate hyphenation.

朝花夕拾 still has 10 kinsoku violations and 4 overflow lines. Samples from the problem log include a line whose first glyph is a closing quote, lines that start with an ideographic comma, and a mixed phonetic line `la者“的”也；“兒”` that both overflows and breaks the class rule. Those counts miss the zero bar. Paragraph start lines are included in the count, so a source paragraph that begins with a closing mark is counted. The JavaScript pass leaves every CJK paragraph on WebKit. `docs/roadmap.md` records `owned` 0 and `forbidden_starts` 0 for 朝花夕拾 at paperback. That zero is WebKit's breaks. This spike's 10 are its own class table.

## Selection

The highlight phrase is `truth universally acknowledged`. Hit testing maps a point to a caret. The caret maps to an EPUB CFI and back to the same caret. `cfi_roundtrip` in the metrics file is true. The CFI does not contain a page number. Glyph rectangles for that range are tinted on `docs/spike/epub-native/pride-highlight.png`. The tint covers the three words as one bar. The opening and closing quotes on that page are EB Garamond.

The live reader does not store a CFI. `Anchor::Epub` is a spine href plus a start and an end. The page script counts those with JavaScript string length, which is UTF-16 code units. This spike counts Unicode scalars. Pride chapter 1 and the BMP text in 朝花夕拾 use the same numbers either way. A character outside the BMP would not. `docs/spike/report.md` records the Pride sample as range 20 to 50 in `/epub/text/chapter-1.xhtml`.

## CSS the books actually use

The counter is `crates/epub-native-spike/survey_css.py`. It walked 12 EPUBs already in the fixture set plus public domain books fetched for this spike. Counts are declaration occurrences. The books are Standard Ebooks Pride and Prejudice, Alice, Hamlet, and Frankenstein, Project Gutenberg Frankenstein and Pride and Prejudice, Project Gutenberg 25271 朝花夕拾, and the small in repo fixtures. No second Japanese book was fetched. Gutenberg 38266 is an English memoir and was discarded. Gutendex returned 403.

12 files of stylesheets, 66 distinct properties, 1241 declarations. No `writing-mode`, no ruby, and no vertical text property appears in this corpus.

| Book | Stylesheet files | Declarations | Distinct properties |
|---|---:|---:|---:|
| alice-se.epub | 26 | 241 | 49 |
| fixed.epub | 2 | 0 | 0 |
| frankenstein-se.epub | 44 | 231 | 46 |
| frankenstein.epub | 34 | 77 | 25 |
| hamlet-se.epub | 18 | 256 | 47 |
| plain-en.epub | 2 | 0 | 0 |
| plain-zh.epub | 2 | 0 | 0 |
| pride-pg.epub | 20 | 170 | 31 |
| pride-se.epub | 71 | 183 | 39 |
| publisher.epub | 3 | 14 | 8 |
| styled.epub | 3 | 14 | 8 |
| zhaohua.epub | 6 | 55 | 23 |

| Property | Count |
|---|---:|
| margin-top | 126 |
| margin-bottom | 98 |
| margin-left | 93 |
| margin-right | 92 |
| text-indent | 74 |
| width | 71 |
| text-align | 70 |
| display | 51 |
| font-size | 50 |
| font-style | 42 |
| font-variant | 31 |
| hyphens | 19 |
| adobe-hyphenate | 19 |
| -webkit-hyphens | 19 |
| -moz-hyphens | 19 |
| padding-left | 18 |
| font-weight | 17 |
| margin | 17 |
| border | 16 |
| background | 16 |
| text-decoration | 16 |
| adobe-text-layout | 15 |
| page-break-after | 14 |
| vertical-align | 14 |
| hanging-punctuation | 12 |
| max-width | 12 |
| color | 12 |
| content | 11 |
| padding | 11 |
| break-after | 10 |
| break-inside | 10 |
| page-break-inside | 10 |
| page-break-before | 10 |
| border-top | 9 |
| line-height | 9 |
| font-family | 9 |
| clear | 9 |
| filter | 8 |
| pointer-events | 8 |
| padding-top | 8 |
| background-color | 8 |
| text-transform | 5 |
| font-variant-numeric | 4 |
| text-wrap | 4 |
| height | 4 |
| white-space | 4 |
| left | 4 |
| position | 4 |
| -epub-hyphens | 3 |
| max-height | 2 |
| align-items | 2 |
| box-sizing | 2 |
| flex-direction | 2 |
| justify-content | 2 |
| min-height | 2 |
| padding-bottom | 2 |
| border-left | 2 |
| src | 2 |
| break-before | 1 |
| letter-spacing | 1 |
| word-spacing | 1 |
| list-style | 1 |
| border-collapse | 1 |
| padding-right | 1 |
| border-bottom | 1 |
| border-right | 1 |

| Selector feature | Count |
|---|---:|
| type | 1177 |
| selector | 751 |
| compound | 751 |
| descendant | 628 |
| class | 409 |
| adjacent sibling | 199 |
| selector list | 112 |
| child | 108 |
| pseudo class | 83 |
| attribute | 75 |
| :root | 72 |
| universal | 20 |
| pseudo element | 15 |
| ::before | 9 |
| id | 9 |
| ::after | 6 |
| :not | 5 |
| :link | 2 |
| :visited | 2 |
| :hover | 2 |

At rules: `@charset` 15, `@namespace` 12, `@media` 9, `@supports` 6, `@font-face` 2.

Chapter elements, highest first: `p` 7245, `span` 3757, `br` 2721, `td` 2690, `tr` 1359, `abbr` 1217, `i` 859, `a` 851, `em` 659, `div` 310, `b` 228, `h2` 219, `section` 140, `li` 84, `blockquote` 71, `img` 45, `figure` 42, `table` 23. The `td` and `tr` counts are mostly Hamlet. Tables are the next slice. They are not required for Pride chapter 1 or 朝花夕拾.

`font-family` occurs 9 times. The page face is the common case, and the built in sheet is what the reader sees when the book does not name a face.

The minimum subset for these two chapters is `p`, headings, `em`, `i`, `abbr`, `span`, `a`, `br`, `blockquote`, `section`, and `div`, plus margin top and bottom, text indent, text align, font size, style, weight, variant, variant numeric, family, line height, hyphens including the vendor aliases, white space, and display of block, inline, or none. Inline `style` attributes are applied and win. Simple type, class, and id selectors apply. Specificity is id, then class, then type. A selector with more than one compound is skipped. A paragraph that follows a heading loses its indent in a post pass, which covers the `h2 + p` case without applying that combinator to every paragraph.

On Pride the honored names were the hyphen properties and their Mozilla and WebKit aliases, display, font size, font style, font variant, font variant numeric, font weight, line height, margin top, margin bottom, text align, text indent, and white space. Dropped names included adobe hyphenate, background, border, break after, break inside, content, hanging punctuation, position, text transform, text wrap, width, and unsupported selectors. Body margin, width, and columns are the page box, not the publisher sheet. `font-family` is recorded as dropped. The spike keeps EB Garamond or Noto Serif CJK SC and does not switch faces from the property.

## Licenses and maturity

Versions are the crates resolved on this machine. A license marked research is taken from the desk note, not from a crate that was downloaded here.

| Crate | Version seen | License | Role in this spike |
|---|---|---|---|
| rbook | 0.7.12 | Apache-2.0 | Used for container, OPF, and spine |
| harfrust | 0.15.0 | MIT | Used for shaping |
| skrifa | 0.49.0 | MIT OR Apache-2.0 | Used for outlines |
| tiny-skia | 0.12.0 | BSD-3-Clause | Used for the PNG |
| TeX en-US patterns | vendored | MIT | Used for hyphenation |
| blitz-dom, blitz-html | 0.3.0-beta.4 | MIT OR Apache-2.0 | Engine A only, beta, no pagination |
| taffy | 0.15.0 | MIT | Pulled by Blitz |
| parley | 0.12.0 | Apache-2.0 OR MIT | Engine A line breaking, greedy, no hyphens |
| stylo | 0.22.0 | MPL-2.0 | Pulled by Blitz. Flag it. Do not treat MPL as forbidden |
| cssparser | 0.38.0 | MPL-2.0 | Not linked. Flag if a later parser adopts it |
| html5ever | 0.40.1 | MIT OR Apache-2.0 | Not linked. Heavier than the hand rolled parser for these chapters |
| icu_segmenter | 2.4.0 | Unicode-3.0 | Not linked. The product line breaker should use it |
| epub | not downloaded | GPL-3.0-or-later, research | Not used |
| lib-epub | not downloaded | research | Not used. The research note points at it for font deobfuscation. This spike did not deobfuscate fonts |
| mupdf | not used | AGPL, research | Out of scope. PDF stays on pdfium |

## Drawing on a Mac

This spike only writes PNGs. NSH's canvas can measure text. It cannot draw glyphs at positions this typesetter computes. The macOS integration path is an objc2 `NSView` that draws glyph runs with `CTFontDrawGlyphs` or `CGContextShowGlyphsAtPositions`. Other platforms need their own glyph run surface, or they can keep a CPU rasterizer. Selection, hit testing, and accessibility stay with the typesetter once that view exists. The WebView remains the default until the quality bar passes.

## Effort

These figures are the desk research report's estimates. This spike did not produce a new schedule.

| Path | Research estimate |
|---|---|
| Blitz, plus a paginator, plus replacing Parley with Knuth-Plass, plus CJK rules, plus vertical text | 30 to 47 agent weeks |
| Owned subset, minimal, clean novels, no tables, floats, or vertical text | 10 to 14 agent weeks |
| Owned subset, horizontal, to a higher grade | 30 to 45 agent weeks, plus 6 to 10 for vertical text |
| No third party crates at all | research option D, an extra 40 to 70, not recommended |

The work inside the minimal owned path is the parser and the style subset, block layout, pagination, the breaker, CJK punctuation and spacing, selection and CFI, and a glyph run view. Vertical text is extra and was not present in this corpus. Accessibility is not started.

## Risks

The gap p95 miss is the main one. More hyphenation did not close it. A product port has to place the same breaks the JavaScript breaker places on the paperback measure, or the 1.37 bar stays missed.

朝花夕拾 still overflows four lines and breaks kinsoku on ten lines. Mixed Latin and Han phonetic lines are the overflow case. The class table is not a full JLREQ implementation.

Combinator selectors, tables, images, floats, and generated content are dropped. Hamlet's drama tables are the largest omitted slice in this corpus.

Publisher `hanging-punctuation` is dropped. The choice sheet implements a hanging mode as a prototype switch, not as that CSS property.

Font deobfuscation, fixed layout, and media overlays are not in this crate.

## How to rerun

Fonts and books are not in the repo. On this machine they lived in `/tmp/superreader-fixtures`: `EBGaramond-Regular.ttf`, `EBGaramond-Italic.ttf`, `NotoSerifCJKsc-Regular.otf`, `pride-se.epub`, `zhaohua.epub`.

From `crates/epub-native-spike`, with Rust 1.98.1:

```
rustup run 1.98.1 cargo test
rustup run 1.98.1 cargo build --release
./target/release/epub-native-spike artifacts docs/spike/epub-native
```

The program writes the PNGs to both directories and `metrics.json` to the first directory. `survey_css.py` reprints the CSS tables.

Blitz, from `spikes`, after `pkg-config` and `libfontconfig1-dev` are installed:

```
cargo build -p spike-measure --features blitz --release
./target/release/spike-measure epub
```

That command prints one JSON object and writes `/tmp/blitz-pride.rgba`.
