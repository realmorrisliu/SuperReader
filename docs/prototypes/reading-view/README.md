# Reading view

One EPUB page. The publication's type and layout win. SuperReader owns the page box, the paper, the footer, and the slide.

Publisher rules for font, size, line height, indent, paragraph spacing, alignment, hyphenation, and headings stay in force. The fallback sheet is `@layer spike-fallback` with `:where()` selectors, inserted at the start of `head`. It has no `!important`. A property is filled only when the book leaves it unset.

`lang` on `html` comes from `dc:language`. WebKit hyphenates only when `lang` is set.

Latin fallback, used when the language is not Chinese or Japanese. Iowan Old Style, then Songti SC, then serif. 22px, line height 1.48, indent 2em, justify, hyphens auto. The page is 48ch with margins 88px, 14%, 72px.

Chinese and Japanese fallback. The same font stack, so a Latin run still uses Iowan and the ideographs use Songti SC. 20px, line height 1.70, indent 2em, justify, hyphens auto. The page is 64ch with margins 72px, 11%, 64px. The sheet also sets `text-autospace: normal` and `line-break: strict`.

Both languages set `-webkit-hyphenate-limit-before: 2`, `-webkit-hyphenate-limit-after: 3`, and `-webkit-hyphenate-limit-lines: 2`. Both set `hanging-punctuation: first allow-end last`. That value is the CSS Text 3 combination of `first`, `allow-end`, and `last`.

Justified lines do not set text-wrap to pretty, because that widens the spaces. There is no script that breaks lines. The next step, Knuth-Plass, is on the roadmap and is not in this view.

The paper is `#f3eee4`. It is the warmer of the two papers tried under Iowan and Songti.

The footer stays visible. It reads `590/1146页` and `本章还剩34页`. The title fades after 400ms. The turn is a 160ms horizontal slide. One column, one page.

## Trim

The page has a fixed ratio. It is centered on the paper, scaled to fit the reading area, and never stretched. Pagination uses that box, not the window.

Books that declare a page size keep it. A fixed-layout EPUB with `rendition:layout` of `pre-paginated` and `rendition:viewport` uses that viewport. A PDF uses its page box. The trim setting does not change those books.

Other books use the saved trim. The choices are 32kai at 130 by 184 mm, paperback at 129 by 198 mm, and US Trade at 140 by 216 mm. 32kai is the default. The View menu lists the three, with a check on the saved one. The file is `~/Library/Application Support/SuperReader/settings.json`. Changing it repaginates the open reflowable book and keeps the same text anchor.

## Overrides

These are the only publisher rules the page sheet replaces. Each one breaks the column box.

- html and body background, margin, width, max-width, and height. A publisher body width or margin shrinks the column box, so the chapter stays one page.
- body columns, overflow, and page padding. The page box, the measure, and the page margins belong to SuperReader.
- direct children width, height, side margins, and float. A wrapper with a fixed width or height becomes a single column.
- position fixed rewritten to absolute. A fixed element paints on every column.

The web view is the page box, so `vw` and `vh` are the page, not the monitor.

## Pictures

`trim-32kai-pride.png`, `trim-paperback-pride.png`, and `trim-trade-pride.png` are the Pride opening at the three trims. `trim-32kai-zhaohua.png`, `trim-paperback-zhaohua.png`, and `trim-trade-zhaohua.png` are the same 朝花夕拾 prose, the page reached at 32kai page 3, kept across the other trims. `styled.png` is the embedded Literata book. `slide.mp4` is the 160ms slide.

A turn's frame gaps are the time between animation frames inside the page. They cluster around 17 ms, one refresh. The check fails a gap of 33 ms, which is a missed refresh.

On the Pride opening at 32开, the widest same-line gap on the visible page was 7.9 times the font's own space. The narrower trims hyphenate, so "acknowledged" breaks across a line. That gap is the reason the next step is Knuth-Plass.
