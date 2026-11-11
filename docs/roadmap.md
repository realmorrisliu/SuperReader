# Roadmap

## 1. Feasibility spike

Done on 2026-11-03. PDF is pdfium. EPUB is the system WKWebView. UNDER RE-EVALUATION (2026-11-10). Morris wants a fully native, pure-Rust EPUB implementation on NSH, for cross-platform reach and full control of layout. The first step is `crates/superreader-typeset`. The reader is still WKWebView. PDF via pdfium-render is unchanged. The comparison, including why hayro lost on text, is in `docs/spike/report.md`. No product features in this step.

## 2. Verification skill

Add the verify-superreader skill with `/create-verification-skill`, so later changes are checked against the goals and the walkthroughs.

## Next tracer bullets

The shell comes first. Morris decided on 2026-11-11 to finish the app shell before any other feature. The layout reference is `docs/design/shell/spec.md`. The final bar does not move. The rule is How we work in `docs/design/north-star.md`.

Each slice below is a thin cut of that one form.

- S1. The gray base, the 220 point version B sidebar, the white card, and the library chat panel frame, including empty states. Replace the bare shelf window. Remove the Open button flow and the `open -a` and Open With checks. Do not repair them. `spikes/library` stays.
- S2. The cover grid and **继续阅读** in the card, filled from `Catalog`.
- S3. The reading form, using the existing reader. The sidebar hides, the page is in the card, and the right region is the annotation margin. S3 waits until Morris approves a mockup of the exact gap, the horizontal position, the margin width, and the toggle placement. The margin's contents are the 2026-11-10 hard requirement in `docs/design/north-star.md`.
- S4. The immersive form. Control-Command-I, the menu item, and the toolbar button enter it. Escape leaves it.
- S5. Motion, and the fit of the three forms. Night mode and a theme switch stay out. Light and dark join this slice only if a later decision revises the 2026-11-03 rule.

## After the shell

These wait until S1 through S5 have landed. Do not start them ahead of the shell.

- [Pull request 39](https://github.com/realmorrisliu/slowpage/pull/39), the 6 point reading card on the old bar. The spec replaces that card. Leave the pull request until the shell slices land.
- [Pull request 45](https://github.com/realmorrisliu/slowpage/pull/45), PDF pages in the EPUB view. One PDF page fit inside the white card, with arrow keys turning pages. Still missing after that: animation, trackpad, edge click, and the footer page number.
- [Pull request 47](https://github.com/realmorrisliu/slowpage/pull/47), per-PR verify selection and a single `spike-shell` session.
- Repo cleanup, including worktree cleanup in `scripts/cleanup-worktrees.sh`.
- Calibre compatibility beyond the catalog `spikes/library` already reads. Writes, schema user_version 28, search, and the rest of the Library checklist below.
- [Pull request 14](https://github.com/realmorrisliu/slowpage/pull/14). The shell spec replaces its split into A1, A2, A3, B1, and B2. The margin mockup replaces B1 and B2 as the annotation next step, and that mockup is an approval, not a code start.
- Reading chrome leftovers from [pull request 8](https://github.com/realmorrisliu/slowpage/pull/8). Still missing: hover fade and the full Apple Books toolbar item set. The 朝花夕拾 narrow-window two-column bug and the page scrubber stay here too.
- Owned typesetter follow-ups from [pull request 13](https://github.com/realmorrisliu/slowpage/pull/13). Still missing: passing gate B, CJK, and switching the default off WKWebView.
- The rest of the Books toolbar icons. S1 uses the list glyph the contents control already ships.
- Bold Word Starts, [pull request 21](https://github.com/realmorrisliu/slowpage/pull/21). Still missing: the WebKit rendering, the breaker and its cache key, and the owned typesetter.
- Books-white, [pull request 15](https://github.com/realmorrisliu/slowpage/pull/15), is already merged. Nothing for white is still missing.

## Core

The library, SQLite annotations, tantivy, and the `superreader` command with JSON output. Writes from that command are applied by the running app, because alan's Seatbelt sandbox denies writes outside mounted directories.

The calibre and [dalibre](https://github.com/djyde/dalibre) targets are in Library below. They wait until the shell slices land. S2 draws the `Catalog` the shelf slice already reads. S2 does not start those targets.

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

- Apple Books-level UI. The shell spec is that shelf and that reading window. S1 and S2 build the shelf. S5 is the motion. The calibre targets on this list still wait.
- Agent-native features. Ask, summarize, cross-book search, and notes, over the same MCP wrapper. Dalibre's MCP edits the catalogue. Ours also reaches the annotations and the full-text index.
- Memory footprint. Dalibre publishes none. Target, not a measurement: idle RSS under 200 MB on macOS with a 10,000-book library, release build.
- Import speed. Dalibre publishes none. Target, not a measurement: 1,000 EPUBs into a calibre-compatible library in under 60 seconds, covers included, release build, on the machine that runs verify.
- Search latency. The 50 ms median target above. It is a target, not a measurement.
- CJK metadata. File names written into a calibre library stay in calibre's ASCII form so calibre can find them. Display, search, sort, and the OPF text keep the original characters. A Chinese author and title survive our edit and still open in calibre.

## Reading

Done on 2026-11-03. A single-page PDF and EPUB reader on NSH v0.4.0. PDF pages are pdfium through pdfium-render, and that choice is unchanged. EPUB pages are WKWebView. UNDER RE-EVALUATION (2026-11-10). The owned typesetter's first step is in the tree. The page is still the WebKit page. The publication's type and layout win. The page box uses the saved trim, or the book's own page size when it declares one. The default trim is paperback, 129 by 198 mm, picked by Morris. 32开 and US Trade stay as the other choices. Pinch zoom and Command zoom re-render the page. Highlights hang off the spike's anchor. No spread, and no night mode.

The reading window landed on 2026-11-04. The lights and the book title fade in at the top. The footer is in the page margin. Full screen scales the page and the type together. The 2026-11-11 shell puts that page inside the card. See `docs/design/shell/spec.md`.

The page and the card are Books-white `#ffffff`. There is no cream or yellow paper. The shell base is the light gray field in `docs/design/shell/spec.md`. Until S1 the spike still paints the chrome the same white as the page.

The Knuth-Plass pass is in the EPUB page. It chooses line breaks and the space between words. Font, size, line height, indent, and alignment stay. `SPIKE_JUSTIFY=0` leaves every paragraph native. A paragraph with CJK stays native. 朝花夕拾 and the plain Chinese fixture report `owned` 0 and `forbidden_starts` 0 at paperback, 32开, and US Trade.

`crates/superreader-typeset` is the headless first step of the native engine. It does not replace the WKWebView page. Phase 1 passed on this Mac on 2026-11-10. The run is commit 64fa573, 1-minute load 7.68, release build. Pride chapter 1 at column 343.47, Iowan Old Style 22px, scored crate p95 1.978 and max 3.9429 against the reader's 3.4286 and 5.4286. At column 383.78 the crate scored 1.7174 and 2.1754 against the reader's 3.1429 and 4.5714. 朝花夕拾 范愛農, Songti SC 20px, column 372.06, has no scored Latin gaps, so that p95 compare is skipped. Kinsoku was 0, overflow was 0, and the warm chapter medians were 5.585 ms, 5.189 ms, and 6.527 ms. Phase 1 of the typesetter gate replaces the word-gap p95 ≤ 1.37 bar and the owned-line-only set. One gap is the drawn inter-word space divided by that font's natural space, sampled per space, nearest-rank p95. The sample is every justified line in the chapter. A paragraph's last line is out, and so is a line with fewer than two spaces, a CJK line, and a line that is not justified. The four conditions are: the crate's all-lines p95 is no worse than the reader's all-lines p95 measured on the same build, fonts, and width; zero CJK kinsoku violations; zero overflow; and at most 50 ms per chapter. The recorded baseline, which is not the bar, is Pride chapter 1, paperback, Iowan 22px, column 343.47 CSS px, where the reader scored p95 3.4286 and max 5.4286 (owned lines 1.2478/1.4799, WebKit-fallback lines 3.5714/5.4286), measured by bc-498e04f9 on cfe3404 at load 8.9. The gate runs on strict Knuth-Plass. `--font-expand` and `--letter-space` stay off by default, and a run with either on does not pass. Tighten later. Latin breaking is TeX's three passes held to the 0.75 cap, then a minimum-demerit set when a paragraph still cannot fit. The breaker matches the JavaScript breaker on the same items. The page images are Apple Books white, #FFFFFF. Still missing from the final bar is the Mac glyph-run view behind a debug switch, then the switch that opens that view, then that view becoming the page in place of WKWebView. The shell slices build the shelf UI. The library targets that beat dalibre stay after those slices.

`scripts/verify` scored one release run on 2026-11-04. The record is `docs/verify/summary.json`. The run is `docs/verify/runs/20261103T183652Z`. With the pass off, the Pride opening was loose. Paperback `gap_p95` was 3.108 on 10 justified lines, 8 of them over 1.5, with 1 river. 32开 `gap_p95` was 3.599 on 10 justified lines, all 10 over 1.5, with 1 river. `docs/design/justification/before.json` stores that census. The breaker is given the column's content width: the page width minus the padding in the page sheet, `max(14%, (100% - 48ch) / 2)`. On the settled paperback that width is 383.78 px, and a DOM measure of "acknowledged" matched the canvas measure at 138.4 px. `box_err` is the distance between that box and the box the paragraph actually laid out in. It was 0 at every English trim. Each non-last owned line meets both edges within 0.25 px. An earlier pair of screenshots compared the opening frame, about 505 by 775 px and 9 pages, with the settled frame, 534 by 819 px and fewer pages. The pass was the only thing still invalidating the view after the title fade, so only the fitted shot reached the settled frame. Both shots are the settled frame now. At that frame native Pride is 8 pages and the pass is 7, because the breaks fill the same measure more evenly. Pride, whole first chapter: paperback p95 1.359 max 1.491, 0 of 59 loose, 6 of 59 hyphenated, flush 0.23. 32开 p95 1.386 max 1.489, 0 of 62 loose, 6 of 62 hyphenated, flush 0.25. US Trade p95 1.379 max 1.482, 0 of 60 loose, 7 of 60 hyphenated, flush 0.25. Rivers and overflow were 0. The unstyled English fixture, also the whole chapter: paperback 9 of 76 hyphenated, p95 1.379. 32开 5 of 57, p95 1.355. US Trade 9 of 67, p95 1.377. The widest hyphen share is 13.4 percent. The 15 percent check stays. A line may start with an intact `——` or `……`. A comma, a period, or a split pair still counts.

`OWN_PUBLISHER` stays false. A separate run, with that constant temporarily true, opened the Literata fixture whose own sheet sets justify. Paperback failed at p95 3.421 and max 3.421, with 4 of 12 lines loose. US Trade failed at p95 2.105 and max 2.368, with 3 of 12 lines loose. 32开 passed at p95 0.789, with 0 of 10 loose. `docs/design/justification/publisher-own.json` stores that run. The verify summary still expects `owned` 0, because the constant in the tree is false.

A later open reads line breaks from `~/Library/Application Support/app.slowpage.reader/breaks/<book>.json`. The key is the book hash, the chapter href, the paragraph index, the content width, the first-line width, the font, the indent, the language, and the incoming hyphen run. The breaker waits until the web view is within 1.5 px of the settled page width, so a reopen is not keyed off a frame from the window animation. Paragraphs the breaker leaves to WebKit are stored too, or the next open would run `breakLines` again for them. `reading-warm-cached` requires `break_source` `cached` and `live_calls` 0. It is not the 300 ms bar.

There is one web view. `SPIKE_NEXT=1` loads the next chapter into that view after the open chapter's layout is done, and `chapter_ms` is that load. `scripts/verify` sets `SUPERREADER_SUPPORT_DIR` to a temp directory for that run.

On the release run in `docs/verify/summary.json`, the one-minute load at start was 9.7, with no other `spike-shell`. During the opens it was 15 to 35. Cold open max 1076.6 ms, median 952.7 ms. Native on that interleave was 881 to 990 ms. Warm open max 1093.3 ms, median 928.0 ms, and all five fitted samples were `cached`. Native on that interleave was 883 to 1070 ms. The cached reopen in `warm_phases` spent 3 ms applying breaks, 355 ms to finish navigation, and 555 ms after navigation before the page was ready. `page_ready_ms` was 910. The 300 ms bar is that whole `page_ready_ms`. Applying the stored breaks is 3 ms. Navigation finishes in 355 ms, and the page is ready 555 ms after that. The same web view then loads the next chapter. Fitted `chapter_ms` max 39.1 ms, median 35.7 ms, at load 14.8. `summary.json` `ok` is false because cold open and warm open stay over 300 ms.

WKWebView remains the EPUB page until an owned pure-Rust typesetter passes. This run keeps the line-break cache.

Pretext, chenglou/pretext, waits until figure placement.

## Icons

Morris picked `generic` / `list` for the contents control. That glyph is on main. S1 uses it as the sidebar toggle. The remaining toolbar icons wait until the shell slices land. [Pull request 14](https://github.com/realmorrisliu/slowpage/pull/14) is not the shell plan. The layout reference is `docs/design/shell/spec.md`.

## PDF paged reading

Queued after the shell slices, with [pull request 45](https://github.com/realmorrisliu/slowpage/pull/45), and before the calibre targets in Library.

The spike still scrolls PDFs in one column. Replace that with fit-page inside the white reading card, EPUB's page-turn interaction, and pdfium with adjacent-page cache. The hard requirements are in `docs/design/north-star.md`, PDF reading.

The implementing pull request must attach prototype screenshots for Morris's approval before the code change: fit-page in the card at two window sizes, and a mid-turn frame.

Verify:

- The page fits inside the card without cropping.
- Aspect ratio is preserved within 0.5%.
- Turn latency stays within the reading bars in `docs/design/north-star.md`.
- The footer page number is correct.
- A highlight survives a page turn and a window resize.

## Comments

The reading form shows a highlight, a note, and an agent response in the annotation margin. `docs/design/shell/spec.md` is that decision. The exact gap, the horizontal position, the margin width, and the toggle placement wait for one mockup Morris approves. S3 waits on that mockup. A highlight can carry the reader's note. `@alan` in that note runs the handoff below.

## @alan

Blocked on the external dependency. When the gate is open, `@alan` spawns a fresh one-shot `alan` process with the passage, the note, the earlier comments, and the question on stdin. SuperReader stores stdout as alan's reply. The spawn does not pass `--resume`. The comment shows a thinking state until the whole reply lands.

## External dependency

Owner: Morris. The work is assigned to him on [alan issue 1078](https://github.com/realmorrisliu/alan/issues/1078).

alan needs a launch flag or mode so that a reader-spawned run does not rewrite its global root-rollout pointer. Today every launch rewrites that pointer. A reader-spawned launch would replace Morris's next `alan --resume` session.

Status: pending.

Gate: confirm [alan issue 1078](https://github.com/realmorrisliu/alan/issues/1078) is closed and the flag has landed before the @alan milestone starts. The change lands in the alan repository. This repository does not carry that patch.

## Distribution

WKWebView _setBackgroundColor: SPI, revisit before any Mac App Store build.
