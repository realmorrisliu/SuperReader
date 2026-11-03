# Goals and hard principles

SuperReader replaces Apple Books for Morris. The agent features matter equally with reading. Ask, explain, or translate a selection. Summarize a chapter. Search across the library. Let other agents read the library and the notes.

The product is opinionated. Settings stay few, and Morris's taste decides.

## Hard principles

No theme switching, and no night mode, ever. A reader with night mode is unacceptable.

Single-page reading only. There is no two-page spread, and there is no setting that turns one on.

The liked reference is Apple Books on one page of 《穷查理宝典》. The toolbar carries the book title. The footer shows the page as `590/1146` and the pages left in the chapter.

![Apple Books single page, the layout SuperReader keeps](references/books-single.png)

The disliked reference is the same book as a two-page spread.

![Apple Books two-page spread, a layout SuperReader does not build](references/books-double.png)

## Stack

The app is Rust on NSH, pinned at v0.3.0, macOS first. PDF pages ship `libpdfium.dylib`. EPUB pages use the system WKWebView. LLM calls go through [openrouter-rs](https://github.com/realmorrisliu/openrouter-rs). Annotations live in SQLite. The full-text index is tantivy. AGPL dependencies stay out, including MuPDF. The GPL `epub` crate stays out. EPUB parsing uses rbook.

## Agents and alan

Other agents, alan included, reach the library and the notes through a `superreader` command that prints JSON, plus a skill. An MCP server is a thin wrapper later, and only for clients that cannot run a terminal.

alan's bash runs under a macOS Seatbelt sandbox that denies writes outside mounted directories. A `superreader` write is applied by the running SuperReader app. The command does not open the database itself.

Picked on 2026-11-03. SuperReader owns the library, the SQLite annotations, tantivy, the comment threads, and the comment UI. alan is not linked into the reader process, and SuperReader does not speak alan's socket protocol. On `@alan` in a comment, SuperReader spawns a fresh one-shot `alan` process and pipes stdin. The body is the passage, the user's note, the earlier comments in the thread, and the new question. SuperReader reads stdout and writes that text into its own SQLite as alan's reply comment. The spawn does not pass `--resume`. The annotation id and the thread id live in SuperReader. The reply arrives whole, not streamed, so the comment shows a thinking state until the reply lands.

The comment UI itself is still open. Two or three prototypes will decide it later. The spike only has to leave a stable anchor that a comment thread can hang on.

alan needs a launch flag so a reader-spawned run does not rewrite alan's global root-rollout pointer. That flag is an external dependency. See the roadmap. This repository does not patch alan.

## Decision log

- 2026-11-03. No night mode, and no theme switching.
- 2026-11-03. Single-page reading only. The spread in the disliked reference is out.
- 2026-11-03. alan is a one-shot process with piped stdin and a whole stdout reply, as written above. Embedding alan, and alan's socket protocol, are out.
- 2026-11-03. PDF rendering is pdfium, because the later-page median stays under 16.7 ms and the glyph boxes are complete.
- 2026-11-03. EPUB pages use the system WKWebView, because it keeps the publisher CSS and a selection that survives reload.
- 2026-11-03. Those two picks mean the app is not strictly pure Rust. It ships `libpdfium.dylib` and uses WebKit. Morris accepted that.
- 2026-11-03. One EPUB reading view. The publication's font and layout win. Fallbacks live in `@layer spike-fallback` and fill only what the book leaves unset. Latin fallback is Iowan Old Style with the large narrow page. Chinese and Japanese fallback is Songti SC with the wider page. The paper is `#f3eee4`. The turn is the 160ms slide. The footer stays visible.
- 2026-11-03. The page box has a fixed ratio, centered and scaled uniformly. The choices are 32开 at 130 by 184 mm, paperback at 129 by 198 mm, and US Trade at 140 by 216 mm. A fixed-layout EPUB viewport and a PDF page box ignore that setting. Changing the trim repaginates a reflowable book and keeps the text anchor.
- 2026-11-03. Default trim is paperback, picked by Morris. 32开 and US Trade stay in the menu. A missing settings file opens at the paperback ratio.

## Measurable bars

`docs/spike/report.md` is the source of the measured numbers. A cache-miss page raster has to stay at or under 16.7 ms, which is one frame at 60 Hz. Open to the first page has to stay under 100 ms. On the GPT-3 PDF at Retina 1x, five release runs gave pdfium an open-to-first median of 52 ms and a later-page median of 9.8 ms, and hayro 9.5 ms and 17.3 ms. A highlight drawn from its anchor still has to line up after a zoom re-render.
