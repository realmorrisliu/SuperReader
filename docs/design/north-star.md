# Goals and hard principles

SuperReader replaces Apple Books for Morris. The agent features matter equally with reading. Ask, explain, or translate a selection. Summarize a chapter. Search across the library. Let other agents read the library and the notes.

The product is opinionated. Settings stay few, and Morris's taste decides.

## How we work

Every line of work first lands the thinnest end-to-end path on main: a real AppKit window, real data, from input to screen. Then it thickens layer by layer. No big PRs accumulating on a branch.

One PR does one independently verifiable thing and merges as soon as it is verified. Push at least at every visible step, even as a draft.

Every step says what is still missing from the final bar: Apple Books-level UI, typesetter gate B, and the library targets that beat dalibre. Acceptance criteria are never lowered, only split into smaller steps.

Every change Morris can see gets screenshots.

The icon flow is in `CONTRIBUTING.md`, under Icons.

## Hard principles

No theme switching, and no night mode, ever. A reader with night mode is unacceptable.

Single-page reading only. There is no two-page spread, and there is no setting that turns one on.

The liked reference is Apple Books on one page of 《穷查理宝典》. The toolbar carries the book title. The footer shows the page as `590/1146` and the pages left in the chapter.

![Apple Books single page, the layout SuperReader keeps](references/books-single.png)

The disliked reference is the same book as a two-page spread.

![Apple Books two-page spread, a layout SuperReader does not build](references/books-double.png)

## Stack

The app is Rust on NSH, pinned at v0.4.0, macOS first. PDF pages ship `libpdfium.dylib` through pdfium-render. EPUB pages use the system WKWebView. That EPUB choice is under re-evaluation as of 2026-11-10. The headless typesetter in `crates/superreader-typeset` is the first step. The reader stays on WKWebView until the word-gap bar passes. LLM calls go through [openrouter-rs](https://github.com/realmorrisliu/openrouter-rs). Annotations live in SQLite. The full-text index is tantivy. AGPL dependencies stay out, including MuPDF. The GPL `epub` crate stays out. EPUB parsing uses rbook.

## NSH alignment

Recorded on 2026-11-10, against the 0.4.0 docs. The shell follows the Rust app in [Project structure](https://nsh.dev/docs/project-structure.md), the window and menu APIs in [nsh/desktop](https://nsh.dev/modules/nsh-desktop.md), and the headless checks in [Testing](https://nsh.dev/docs/testing-and-trace.md). The menu shape is the `examples/desktop` application menu. What still sits on objc2, and why, is in `docs/nsh.md`.

Patterns we follow:

- `spikes/shell/nsh.json` is the app root. `rust` is `.` and `root` is `SpikeApp`. `lib.rs` registers that component with `#[native_module]` and `#[component]`, as the Rust example does.
- The window calls `set_title_bar(TitleBarStyle::Hidden, Some(px(48)))`. A height above 44 points selects regular macOS chrome: the empty unified toolbar, a transparent title bar, and a full-size content view. NSH hides the toolbar it owns while the window is in full screen, and shows it again afterwards.
- The paper is Books-white `#ffffff`, with no cream or yellow paper. It is the root view background. NSH copies an opaque view that fills the window onto the surface layer, so the chrome and the toolbar blend into the page the way Apple Books does.
- The menu is the documented `AppMenu` pattern. An `effect` calls `set_application_menu` and rebuilds it when the trim signal changes. On macOS the first menu is the app menu of roles (About, Hide, Hide Others, Quit). Edit is the edit roles, including Select All. View items use `.checked` against the current trim. The Window menu's full screen item is `MenuRole::ToggleFullScreen`.
- `spikes/shell/shell.test.tsx` imports `expect`, `open`, and `test` from `nsh/test`. The four checks use `toContain`. `scripts/verify` runs them with `nsh test`.

Where we differ, on purpose:

- The book title is an `NSTextField` in the content view, and it fades with the traffic lights. The documented title bar is an NSH view with `window_drag` and padding from `title_bar().left`. That row stays hidden while an EPUB is open, because the title the reader sees has to fade.
- A drag on the page mat uses `performWindowDragWithEvent` on the native stage view. `window_drag` marks NSH views. The stage view keeps the click so a drag on the page can select text.
- The fitted window uses AppKit `setFrame`. The fit math reads `visibleFrame`, whose origin is the bottom left. `set_frame` measures from the top left of the primary screen, with y growing down.
- `SPIKE_FULLSCREEN` calls `toggleFullScreen` on the timer that owns the view. `toggle_full_screen` has to run under an NSH owner, and that timer has none. The menu item stays the role.
- The window is ordered front with `activateIgnoringOtherApps`. `order_front` calls `activate`.
- `FullScreenPrimary` is added to the window's collection behavior. `set_level` is the NSH call that sets collection behavior, and it replaces the behavior with `MoveToActiveSpace`.
- The titlebar hairline is cleared with `setTitlebarSeparatorStyle`. The traffic lights fade by setting the alpha of `standardWindowButton`. NSH exposes neither.
- EPUB pages are a system WKWebView, and PDF pages are a pdfium bitmap. The page is outside NSH's `WebView`. The EPUB half is under re-evaluation. The first native step is the headless typesetter, and the Mac glyph-run view is not built yet. PDF via pdfium-render stays.
- `main.rs` calls `run_app` so the spike can register the native stage and set the opening size. Verify launches that binary. `nsh test` still opens the app the documented way, from `nsh.json`.

## Agents and alan

Other agents, alan included, reach the library and the notes through a `superreader` command that prints JSON, plus a skill. An MCP server is a thin wrapper later, and only for clients that cannot run a terminal.

alan's bash runs under a macOS Seatbelt sandbox that denies writes outside mounted directories. A `superreader` write is applied by the running SuperReader app. The command does not open the database itself.

Picked on 2026-11-03. SuperReader owns the library, the SQLite annotations, tantivy, the comment threads, and the comment UI. alan is not linked into the reader process, and SuperReader does not speak alan's socket protocol. On `@alan` in a comment, SuperReader spawns a fresh one-shot `alan` process and pipes stdin. The body is the passage, the user's note, the earlier comments in the thread, and the new question. SuperReader reads stdout and writes that text into its own SQLite as alan's reply comment. The spawn does not pass `--resume`. The annotation id and the thread id live in SuperReader. The reply arrives whole, not streamed, so the comment shows a thinking state until the reply lands.

The comment UI itself is still open. Two or three prototypes will decide it later. The spike only has to leave a stable anchor that a comment thread can hang on.

alan needs a launch flag so a reader-spawned run does not rewrite alan's global root-rollout pointer. That flag is an external dependency. See the roadmap. This repository does not patch alan.

## Icons

IconMCP is the only icon library. The global parameters are outline, stroke 1.25, and radius 2. Solid is for a selected or active state only.

The shell rasterizes each SVG at runtime with resvg into a template image. AppKit tints that image for the window appearance. A selected icon can use the accent color. The SVG bytes are in the binary, so a run does not contact IconMCP.

The contents control is the first item in the unified toolbar. AppKit places it after the traffic lights and centers it on them. The glyph is IconMCP `generic` / `list`, outline, stroke 1.25, radius 2.

The paper is Books-white `#ffffff`. There is no night mode. A dark window appearance paints a template icon light, so the icon disappears on that paper. The light appearance is the one that sits on the page.

Attribution for IconMCP and each set in use is in `assets/icons/NOTICE`.

## Decision log

- 2026-11-03. No night mode, and no theme switching.
- 2026-11-03. Single-page reading only. The spread in the disliked reference is out.
- 2026-11-03. alan is a one-shot process with piped stdin and a whole stdout reply, as written above. Embedding alan, and alan's socket protocol, are out.
- 2026-11-03. PDF rendering is pdfium, because the later-page median stays under 16.7 ms and the glyph boxes are complete.
- 2026-11-03. EPUB pages use the system WKWebView, because it keeps the publisher CSS and a selection that survives reload. UNDER RE-EVALUATION (2026-11-10). Morris wants to consider a fully native, pure-Rust EPUB implementation on NSH, for cross-platform reach and full control of layout. Research is in progress. PDF via pdfium-render is unchanged.
- 2026-11-10. PDF still ships `libpdfium.dylib` through pdfium-render. The WebKit EPUB page is the part under re-evaluation, so a pure-Rust reader is open again for EPUB only.
- 2026-11-10. The shell follows NSH's Rust app, hidden title bar, application menu, and `nsh test` patterns. The fades, the fitted frame, the stage drag, and the page surfaces stay on objc2. The list is under NSH alignment above.
- 2026-11-10. The owned EPUB typesetter is `crates/superreader-typeset`. `Typesetter::open` loads the faces. `compose` returns glyph runs, a caret of node and offset with no page, hit testing, and selection rects. The Mac view that paints those runs is the next change. WKWebView stays the reader until the word-gap bar passes.
- 2026-11-10. Three fallbacks, and a book's own CSS wins over each one. CJK punctuation compresses by JLREQ and hangs a closing mark at the line end. English hyphenation uses the TeX defaults, penalty 50, two letters before the hyphen and three after. A CJK and Latin boundary takes a quarter em of glue. The checks are `jlreq_hang_drops_edge_aki_and_hangs_closing_marks`, `tex_hyphen_defaults_break_hyphenation_at_two_and_six`, and `cjk_latin_quarter_em_on_of_zhaohua_and_mid_line`. `publisher_hanging_punctuation_none_beats_the_fallback` and `publisher_hyphens_none_beats_the_fallback` keep an explicit publisher value in force.
- 2026-11-10. Latin lines use a port of the product TeX breaker. Glue stretch equals the space width. The adjustment cap stays 0.75, and a paragraph that exceeds it is itemized once more with the TeX hyphen penalties and retried. `pride_chapter_one_breakpoints_match_the_javascript_breaker` matches the JavaScript breaker on the same item widths. CJK-heavy paragraphs use ICU strict line breaks and a longest fit, because a per-character active set does not finish. `zhaohua_has_no_kinsoku_violations_or_overflow` is clear. `mixed_phonetic_line_does_not_overflow` covers the phonetic line `la者“的”也；“兒”`.
- 2026-11-10. Release timing on a 4-core machine, uptime 2 hours 14 minutes, load 0.10, 0.22, 0.30. Fonts were already open. Five warm layouts followed one cold layout. Pride and Prejudice chapter 1 had a warm median of 6.280 ms, range 6.183 to 6.406, cold 6.920. Shaping took 3.412 ms and breaking 1.694 ms, so shaping is the limiter. 7 pages, 127 lines, 14 hyphenated, kinsoku 0, overflow 0. Word-gap p95 was 2.1275 and the max was 3.3863. The bar is 1.37, from `docs/verify/summary.json`. This misses. The 朝花夕拾 opening file had a warm median of 16.243 ms, range 16.182 to 17.363, cold 18.595. Shaping took 5.090 ms and breaking 6.728 ms, so breaking is the limiter. 65 pages, 1286 lines, kinsoku 0, overflow 0. Both chapters are under the 50 ms layout bar. The faces on this run are EB Garamond and Noto Serif CJK SC, the OFL stand-ins for Iowan Old Style and Songti SC.
- 2026-11-10. The gap miss is the paragraphs the breaker rejects. The product leaves those on WebKit, which is the layout behind p95 1.359 on Iowan. This crate draws the emergency breaks at their true ratio, including ratios above 0.75, and counts those gaps. The figure is not clamped.
- 2026-11-03. One EPUB reading view. The publication's font and layout win. Fallbacks live in `@layer spike-fallback` and fill only what the book leaves unset. Latin fallback is Iowan Old Style with the large narrow page. Chinese and Japanese fallback is Songti SC with the wider page. The paper is Books-white `#ffffff`, with no cream or yellow. The turn is the 160ms slide. The footer stays visible.
- 2026-11-03. The page box has a fixed ratio, centered and scaled uniformly. The choices are 32开 at 130 by 184 mm, paperback at 129 by 198 mm, and US Trade at 140 by 216 mm. A fixed-layout EPUB viewport and a PDF page box ignore that setting. Changing the trim repaginates a reflowable book and keeps the text anchor.
- 2026-11-03. Default trim is paperback, picked by Morris. 32开 and US Trade stay in the menu. A missing settings file opens at the paperback ratio.
- 2026-11-04. The reading window is the page. Traffic lights and the book title fade in at the top. The footer sits in the page's bottom margin, on the measure. Full screen scales the page with the type. The lights are the system buttons on an empty unified toolbar, so they match the current macOS window.

## PDF reading

PDF reading is paged like EPUB. There is no continuous scroll.

A PDF is never reflowed. Keep the original page aspect ratio and layout. Scale each page uniformly so the whole page fits inside the white reading card, and recompute on window resize.

Reuse EPUB's page-turn interaction: keyboard, trackpad swipe, clicking the left and right edges, the same 160ms slide, and the footer page number.

Two-page spreads are out of scope for now. Revisit them later, depending on window width.

Rendering stays on pdfium through pdfium-render. No AGPL. Pre-render or cache adjacent pages so turns stay smooth. Selection, highlights, and annotation anchors must keep working per page.

The spike's pdfium path still renders as a continuous scroll, so it conflicts with this section. Before changing it, the implementing pull request must attach prototype screenshots for Morris's approval: fit-page in the card at two window sizes, and one mid-turn frame.

## Measurable bars

`docs/spike/report.md` is the source of the measured numbers. A cache-miss page raster has to stay at or under 16.7 ms, which is one frame at 60 Hz. Open to the first page has to stay under 100 ms. On the GPT-3 PDF at Retina 1x, five release runs gave pdfium an open-to-first median of 52 ms and a later-page median of 9.8 ms, and hayro 9.5 ms and 17.3 ms. A highlight drawn from its anchor still has to line up after a zoom re-render.
