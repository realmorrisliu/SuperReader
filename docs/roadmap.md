# Roadmap

## 1. Feasibility spike

Done on 2026-11-03. PDF is pdfium. EPUB is the system WKWebView. The comparison, including why hayro lost on text, is in `docs/spike/report.md`. No product features in this step.

## 2. Verification skill

Add the verify-superreader skill with `/create-verification-skill`, so later changes are checked against the goals and the walkthroughs.

## Core

The library, SQLite annotations, tantivy, and the `superreader` command with JSON output. Writes from that command are applied by the running app, because alan's Seatbelt sandbox denies writes outside mounted directories.

## Reading

Done on 2026-11-03. A single-page PDF and EPUB reader on NSH v0.3.0. PDF pages are pdfium. EPUB pages are WKWebView. The publication's type and layout win. The page box uses the saved trim, or the book's own page size when it declares one. The default trim is paperback, 129 by 198 mm, picked by Morris. 32开 and US Trade stay as the other choices. Pinch zoom and Command zoom re-render the page. Highlights hang off the spike's anchor. No spread, and no night mode.

The reading window landed on 2026-11-04. It is the page, with the lights and the book title fading in at the top. The footer is in the page margin. Full screen scales the page and the type together.

The Knuth-Plass pass is in the EPUB page. It chooses line breaks and the space between words. Font, size, line height, indent, and alignment stay. `SPIKE_JUSTIFY=0` leaves every paragraph native. A paragraph with CJK stays native. 朝花夕拾 and the plain Chinese fixture report `owned` 0 and `forbidden_starts` 0 at paperback, 32开, and US Trade.

`scripts/verify` scored one release run on 2026-11-04. The record is `docs/verify/summary.json`. The run is `docs/verify/runs/20261103T183652Z`. With the pass off, the Pride opening was loose. Paperback `gap_p95` was 3.108 on 10 justified lines, 8 of them over 1.5, with 1 river. 32开 `gap_p95` was 3.599 on 10 justified lines, all 10 over 1.5, with 1 river. `docs/design/justification/before.json` stores that census. The breaker is given the column's content width: the page width minus the padding in the page sheet, `max(14%, (100% - 48ch) / 2)`. On the settled paperback that width is 383.78 px, and a DOM measure of "acknowledged" matched the canvas measure at 138.4 px. `box_err` is the distance between that box and the box the paragraph actually laid out in. It was 0 at every English trim. Each non-last owned line meets both edges within 0.25 px. An earlier pair of screenshots compared the opening frame, about 505 by 775 px and 9 pages, with the settled frame, 534 by 819 px and fewer pages. The pass was the only thing still invalidating the view after the title fade, so only the fitted shot reached the settled frame. Both shots are the settled frame now. At that frame native Pride is 8 pages and the pass is 7, because the breaks fill the same measure more evenly. Pride, whole first chapter: paperback p95 1.359 max 1.491, 0 of 59 loose, 6 of 59 hyphenated, flush 0.23. 32开 p95 1.386 max 1.489, 0 of 62 loose, 6 of 62 hyphenated, flush 0.25. US Trade p95 1.379 max 1.482, 0 of 60 loose, 7 of 60 hyphenated, flush 0.25. Rivers and overflow were 0. The unstyled English fixture, also the whole chapter: paperback 9 of 76 hyphenated, p95 1.379. 32开 5 of 57, p95 1.355. US Trade 9 of 67, p95 1.377. The widest hyphen share is 13.4 percent. The 15 percent check stays. A line may start with an intact `——` or `……`. A comma, a period, or a split pair still counts.

`OWN_PUBLISHER` stays false. A separate run, with that constant temporarily true, opened the Literata fixture whose own sheet sets justify. Paperback failed at p95 3.421 and max 3.421, with 4 of 12 lines loose. US Trade failed at p95 2.105 and max 2.368, with 3 of 12 lines loose. 32开 passed at p95 0.789, with 0 of 10 loose. `docs/design/justification/publisher-own.json` stores that run. The verify summary still expects `owned` 0, because the constant in the tree is false.

A later open reads line breaks from `~/Library/Application Support/SuperReader/breaks/<book>.json`. The key is the book hash, the chapter href, the paragraph index, the content width, the first-line width, the font, the indent, the language, and the incoming hyphen run. The file stamp is the trim, or the declared viewport. A trim change deletes the reflowable file. `settings.json` stays the trim id alone. The page applies a stored plan without `breakLines`, checks the line edges, and live-breaks that paragraph on a miss. A miss does not stick as `data-kp="native"`.

The next chapter is a second web view, created only after the shown chapter's tail is done, so the cold open does not start a second WebContent process before `page_ready_ms`. A turn past the last column swaps that view onto the page. Going back uses the chapter just left while it is still the offscreen view. `reading-cold-open`, `reading-warm-open`, and `reading-chapter-turn` each keep the 300 ms maximum. The cold open still includes WebContent startup. `scripts/verify` sets `SUPERREADER_SUPPORT_DIR` to a temp directory for that run. A real launch still uses `~/Library/Application Support/SuperReader`.

The earlier five, at load 5.81, 6.40, and 6.91, missed 300 ms. Fitted min 339.3 ms, median 365.3 ms, max 415.2 ms. Native 321, 322, 312, 301, and 326 ms. `open_ms` sat within a few milliseconds of `page_ready_ms`, so the wait was the web view coming up. The run in `docs/verify/summary.json` splits that check. Cold opens, cache cleared, at load 13.29, 17.00, and 17.04: fitted min 450.6 ms, median 493.4 ms, max 643.9 ms. Native 542, 382, 527, 400, and 612 ms. Errors 0. That check fails. Warm opens, after one priming launch, at load 26.06, 19.35, and 17.88: fitted min 412.8 ms, median 442.1 ms, max 594.6 ms. Native 335, 438, 539, 387, and 532 ms. Errors 0. Every fitted sample reported `break_source` `mixed`, so some visible paragraphs still ran the live breaker. That check fails. Chapter turns, five fitted interleaved with five native, at load 21.05, 19.59, and 18.11: fitted min 0.8 ms, median 0.9 ms, max 1.4 ms. Native 1, 1, 1, 1, and 1 ms. Errors 0. Each fitted turn landed on `/epub/text/chapter-2.xhtml`. That check passes. `summary.json` `ok` is false because cold open and warm open stay over 300 ms.

Pretext, chenglou/pretext, waits until figure placement.

## Comments

The comment UI is chosen from two or three prototypes. A highlight can carry the reader's note. `@alan` in that note runs the handoff below.

## @alan

Blocked on the external dependency. When the gate is open, `@alan` spawns a fresh one-shot `alan` process with the passage, the note, the earlier comments, and the question on stdin. SuperReader stores stdout as alan's reply. The spawn does not pass `--resume`. The comment shows a thinking state until the whole reply lands.

## External dependency

Owner: Morris. The work is assigned to him on [alan issue 1078](https://github.com/realmorrisliu/alan/issues/1078).

alan needs a launch flag or mode so that a reader-spawned run does not rewrite its global root-rollout pointer. Today every launch rewrites that pointer. A reader-spawned launch would replace Morris's next `alan --resume` session.

Status: pending.

Gate: confirm [alan issue 1078](https://github.com/realmorrisliu/alan/issues/1078) is closed and the flag has landed before the @alan milestone starts. The change lands in the alan repository. This repository does not carry that patch.
