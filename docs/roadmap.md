# Roadmap

## 1. Feasibility spike

Done on 2026-11-03. PDF is pdfium. EPUB is the system WKWebView. UNDER RE-EVALUATION (2026-11-10). Morris wants to consider a fully native, pure-Rust EPUB implementation on NSH, for cross-platform reach and full control of layout. Research is in progress. PDF via pdfium-render is unchanged. The comparison, including why hayro lost on text, is in `docs/spike/report.md`. No product features in this step.

## 2. Verification skill

Add the verify-superreader skill with `/create-verification-skill`, so later changes are checked against the goals and the walkthroughs.

## Next tracer bullets

The next thin slice on each line. The final bar does not move. The rule is How we work in `docs/design/north-star.md`.

- Books-white ([#15](https://github.com/realmorrisliu/SuperReader/pull/15)). Merge #15 as is after one `just verify` pass with the page-bands-white check. Still missing: nothing for white.
- Reading chrome ([#8](https://github.com/realmorrisliu/SuperReader/pull/8)). Split. (a) the toolbar on NSH 0.4's unified bar plus the system-font footer, in its own PR; (b) the 朝花夕拾 narrow-window two-column bug as its own PR; (c) the page scrubber later. Still missing: hover fade and the full Apple Books toolbar item set.
- Window shell ([#14](https://github.com/realmorrisliu/SuperReader/pull/14)). Split PR A into A1, the fixed-width sidebar centered on the measured traffic lights plus its 0.5pt check; A2, the floating card; A3, immersive mode (⌃⌘I, menu, button, Esc). Then B1, light annotation cards aligned to lines, and B2, the connector's three states. Close draft #14 after B2. Still missing: A2, A3, B1, and B2.
- Owned typesetter ([#13](https://github.com/realmorrisliu/SuperReader/pull/13)). Land the crate rendering Pride ch.1 in the real window behind an off-by-default flag, with the gate B measurement committed. WKWebView stays the default until gate B passes. Still missing: passing gate B, CJK, and switching the default.
- Icons. Tooling (`just icons-fetch`, manifest, `.cursor/mcp.json`) plus one icon rendered through resvg in the real toolbar, with before/after shots for approval. Still missing: replacing the rest after Morris approves.
- Bold Word Starts ([#21](https://github.com/realmorrisliu/SuperReader/pull/21)). After Morris picks the rule, merge the pure rule function, its unit tests and the IP lint. Next, WebKit rendering behind the setting, off by default, with shots. Then the breaker measuring the bold prefix and the cache key. Then the owned typesetter. Still missing: the WebKit rendering, the breaker and its cache key, and the owned typesetter.
- PDF paged (after A1–A3). One PDF page fit inside the white card in the real window, arrow keys turn pages, shots for approval. Still missing: animation, trackpad, edge click, footer page number.
- Library. A read-only open of a real calibre metadata.db in pure Rust, listing titles in an NSH window. Still missing: covers, search, writes, and every target in the Library section.

## Core

The library, SQLite annotations, tantivy, and the `superreader` command with JSON output. Writes from that command are applied by the running app, because alan's Seatbelt sandbox denies writes outside mounted directories.

The shelf bar against Calibre and [dalibre](https://github.com/djyde/dalibre) is in Library below. No library work has started.

## Library

The library has to reach Calibre, then pass [dalibre](https://github.com/djyde/dalibre) (Go, native UI, no webview). Read on 2026-11-10 from dalibre's README, changelog, and `docs/mcp` and `docs/plugins`. Dalibre publishes no memory, import-speed, or search-latency figures. None are recorded below. Every number under "Where we must be better" is a target, not a measurement.

License. Dalibre's repository root declares no license. Its README says `internal/calibre/metadata_sqlite.sql` comes from calibre (GPL-3.0). SuperReader is MIT OR Apache-2.0. This section is a design reference only. Never copy code, SQL, or schema files from dalibre or calibre. Implement calibre compatibility ourselves from the publicly documented formats, calibre's user manual and the OPF specification, described in our own words.

Pure Rust. The library is Rust only. That covers reading and writing a calibre library (`metadata.db`, the OPF sidecar, and the naming rules), metadata fetch, the plugin and book-source system including its sandbox, the MCP server, and auto-update. None of that may depend on Go, Python, or a calibre install, at runtime or at build time.

Checklist. Each item is what we match, then where we have to be better.

- Calibre library, read and write. Match: open and write a library whose `metadata.db` is schema user_version 28; supply the SQL functions calibre's triggers call, including title sort and uuid generation; follow calibre's directory and file naming, ASCII transliteration included, and move the folder when the title or the first author changes; on edit, write the OPF sidecar and mark the book metadata-dirtied so calibre refreshes it. Better: the metadata fields keep the original CJK. We do not vendor calibre's schema script.
- Search, filter, sort, sidebar counts. Match: query, author, series, tag, publisher, language, format, recently added, sort, and a count per facet. Better: tantivy searches the books, not the catalogue fields alone. Target, not a measurement: a title or author query over 10,000 books returns in under 50 ms at the median, release build.
- Cover thumbnails. Match: thumbnails stored on disk for the shelf. Better: covers and motion at Apple Books' level. Target, not a measurement: once those thumbnails are on disk, scrolling a shelf of 1,000 covers stays at 60 Hz.
- Metadata plugins. Match: Google Books and NeoDB lookups. The reader accepts a candidate before it fills title, authors, identifiers, or the cover. Better: CJK names, series, and publishers round-trip with the original characters, not an ASCII fold.
- Book-source plugins. Match: Project Gutenberg built in, and further sources as JavaScript plugins in a sandbox. A plugin searches and points at a file; we download it and add the book. The sandbox cannot be GPL or AGPL. Candidates to evaluate later, no choice yet, and only ones we can embed from Rust under a permissive license: Boa (MIT), a JavaScript engine written in Rust, and rquickjs (MIT), Rust bindings to QuickJS-NG (MIT). rquickjs still links that C engine, so it has to clear the pure-Rust rule above before it can be chosen.
- MCP. Match: an external agent can search the open library, edit metadata, and add books. Ours is already the plan in `docs/design/north-star.md`. The `superreader` command prints JSON. The running app applies writes, because alan's Seatbelt sandbox denies writes outside mounted directories. The MCP server is the thin wrapper for clients that cannot run a terminal. It exposes the same search, edit, and add jobs as that command, and it does not open the database itself.
- Auto-update. Match: updates come from signed GitHub releases. Dalibre's changelog 0.3.0 says it checks every hour. Target, not a measurement: a signed check on launch and at most once an hour, and an unsigned payload is refused.

Where we must be better:

- Apple Books-level UI. The shelf, the covers, and the animation. The reading window is already the page. The shelf has to feel like that. No work has started.
- Agent-native features. Ask, summarize, cross-book search, and notes, over the same MCP wrapper. Dalibre's MCP edits the catalogue. Ours also reaches the annotations and the full-text index.
- Memory footprint. Dalibre publishes none. Target, not a measurement: idle RSS under 200 MB on macOS with a 10,000-book library, release build.
- Import speed. Dalibre publishes none. Target, not a measurement: 1,000 EPUBs into a calibre-compatible library in under 60 seconds, covers included, release build, on the machine that runs verify.
- Search latency. The 50 ms median target above. It is a target, not a measurement.
- CJK metadata. File names written into a calibre library stay in calibre's ASCII form so calibre can find them. Display, search, sort, and the OPF text keep the original characters. A Chinese author and title survive our edit and still open in calibre.

## Reading

Done on 2026-11-03. A single-page PDF and EPUB reader on NSH v0.4.0. PDF pages are pdfium through pdfium-render, and that choice is unchanged. EPUB pages are WKWebView. UNDER RE-EVALUATION (2026-11-10). Morris wants to consider a fully native, pure-Rust EPUB implementation on NSH, for cross-platform reach and full control of layout. Research is in progress. The publication's type and layout win. The page box uses the saved trim, or the book's own page size when it declares one. The default trim is paperback, 129 by 198 mm, picked by Morris. 32开 and US Trade stay as the other choices. Pinch zoom and Command zoom re-render the page. Highlights hang off the spike's anchor. No spread, and no night mode.

The reading window landed on 2026-11-04. It is the page, with the lights and the book title fading in at the top. The footer is in the page margin. Full screen scales the page and the type together.

The paper is Books-white `#ffffff`. There is no cream or yellow paper. The chrome and the toolbar are the same white as the page, the way Apple Books' white theme reads as one sheet.

The Knuth-Plass pass is in the EPUB page. It chooses line breaks and the space between words. Font, size, line height, indent, and alignment stay. `SPIKE_JUSTIFY=0` leaves every paragraph native. A paragraph with CJK stays native. 朝花夕拾 and the plain Chinese fixture report `owned` 0 and `forbidden_starts` 0 at paperback, 32开, and US Trade.

`scripts/verify` scored one release run on 2026-11-04. The record is `docs/verify/summary.json`. The run is `docs/verify/runs/20261103T183652Z`. With the pass off, the Pride opening was loose. Paperback `gap_p95` was 3.108 on 10 justified lines, 8 of them over 1.5, with 1 river. 32开 `gap_p95` was 3.599 on 10 justified lines, all 10 over 1.5, with 1 river. `docs/design/justification/before.json` stores that census. The breaker is given the column's content width: the page width minus the padding in the page sheet, `max(14%, (100% - 48ch) / 2)`. On the settled paperback that width is 383.78 px, and a DOM measure of "acknowledged" matched the canvas measure at 138.4 px. `box_err` is the distance between that box and the box the paragraph actually laid out in. It was 0 at every English trim. Each non-last owned line meets both edges within 0.25 px. An earlier pair of screenshots compared the opening frame, about 505 by 775 px and 9 pages, with the settled frame, 534 by 819 px and fewer pages. The pass was the only thing still invalidating the view after the title fade, so only the fitted shot reached the settled frame. Both shots are the settled frame now. At that frame native Pride is 8 pages and the pass is 7, because the breaks fill the same measure more evenly. Pride, whole first chapter: paperback p95 1.359 max 1.491, 0 of 59 loose, 6 of 59 hyphenated, flush 0.23. 32开 p95 1.386 max 1.489, 0 of 62 loose, 6 of 62 hyphenated, flush 0.25. US Trade p95 1.379 max 1.482, 0 of 60 loose, 7 of 60 hyphenated, flush 0.25. Rivers and overflow were 0. The unstyled English fixture, also the whole chapter: paperback 9 of 76 hyphenated, p95 1.379. 32开 5 of 57, p95 1.355. US Trade 9 of 67, p95 1.377. The widest hyphen share is 13.4 percent. The 15 percent check stays. A line may start with an intact `——` or `……`. A comma, a period, or a split pair still counts.

`OWN_PUBLISHER` stays false. A separate run, with that constant temporarily true, opened the Literata fixture whose own sheet sets justify. Paperback failed at p95 3.421 and max 3.421, with 4 of 12 lines loose. US Trade failed at p95 2.105 and max 2.368, with 3 of 12 lines loose. 32开 passed at p95 0.789, with 0 of 10 loose. `docs/design/justification/publisher-own.json` stores that run. The verify summary still expects `owned` 0, because the constant in the tree is false.

A later open reads line breaks from `~/Library/Application Support/SuperReader/breaks/<book>.json`. The key is the book hash, the chapter href, the paragraph index, the content width, the first-line width, the font, the indent, the language, and the incoming hyphen run. The breaker waits until the web view is within 1.5 px of the settled page width, so a reopen is not keyed off a frame from the window animation. Paragraphs the breaker leaves to WebKit are stored too, or the next open would run `breakLines` again for them. `reading-warm-cached` requires `break_source` `cached` and `live_calls` 0. It is not the 300 ms bar.

There is one web view. `SPIKE_NEXT=1` loads the next chapter into that view after the open chapter's layout is done, and `chapter_ms` is that load. `scripts/verify` sets `SUPERREADER_SUPPORT_DIR` to a temp directory for that run.

On the release run in `docs/verify/summary.json`, the one-minute load at start was 9.7, with no other `spike-shell`. During the opens it was 15 to 35. Cold open max 1076.6 ms, median 952.7 ms. Native on that interleave was 881 to 990 ms. Warm open max 1093.3 ms, median 928.0 ms, and all five fitted samples were `cached`. Native on that interleave was 883 to 1070 ms. The cached reopen in `warm_phases` spent 3 ms applying breaks, 355 ms to finish navigation, and 555 ms after navigation before the page was ready. `page_ready_ms` was 910. The 300 ms bar is that whole `page_ready_ms`. Applying the stored breaks is 3 ms. Navigation finishes in 355 ms, and the page is ready 555 ms after that. The same web view then loads the next chapter. Fitted `chapter_ms` max 39.1 ms, median 35.7 ms, at load 14.8. `summary.json` `ok` is false because cold open and warm open stay over 300 ms.

WKWebView remains the EPUB page until an owned pure-Rust typesetter passes. This run keeps the line-break cache.

Pretext, chenglou/pretext, waits until figure placement.

## PDF paged reading

Queued after [PR #14](https://github.com/realmorrisliu/SuperReader/pull/14) lands as PR A then PR B, shell and annotation panel, and before Core (the shelf).

The spike still scrolls PDFs in one column. Replace that with fit-page inside the white reading card, EPUB's page-turn interaction, and pdfium with adjacent-page cache. The hard requirements are in `docs/design/north-star.md`, PDF reading.

The implementing pull request must attach prototype screenshots for Morris's approval before the code change: fit-page in the card at two window sizes, and a mid-turn frame.

Verify:

- The page fits inside the card without cropping.
- Aspect ratio is preserved within 0.5%.
- Turn latency stays within the reading bars in `docs/design/north-star.md`.
- The footer page number is correct.
- A highlight survives a page turn and a window resize.

## Comments

The comment UI is chosen from two or three prototypes. A highlight can carry the reader's note. `@alan` in that note runs the handoff below.

## @alan

Blocked on the external dependency. When the gate is open, `@alan` spawns a fresh one-shot `alan` process with the passage, the note, the earlier comments, and the question on stdin. SuperReader stores stdout as alan's reply. The spawn does not pass `--resume`. The comment shows a thinking state until the whole reply lands.

## External dependency

Owner: Morris. The work is assigned to him on [alan issue 1078](https://github.com/realmorrisliu/alan/issues/1078).

alan needs a launch flag or mode so that a reader-spawned run does not rewrite its global root-rollout pointer. Today every launch rewrites that pointer. A reader-spawned launch would replace Morris's next `alan --resume` session.

Status: pending.

Gate: confirm [alan issue 1078](https://github.com/realmorrisliu/alan/issues/1078) is closed and the flag has landed before the @alan milestone starts. The change lands in the alan repository. This repository does not carry that patch.

## Distribution

WKWebView _setBackgroundColor: SPI, revisit before any Mac App Store build.
