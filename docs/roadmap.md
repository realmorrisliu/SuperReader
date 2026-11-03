# Roadmap

## 1. Feasibility spike

Done on 2026-11-03. PDF is pdfium. EPUB is the system WKWebView. The comparison, including why hayro lost on text, is in `docs/spike/report.md`. No product features in this step.

## 2. Verification skill

Add the verify-superreader skill with `/create-verification-skill`, so later changes are checked against the goals and the walkthroughs.

## Core

The library, SQLite annotations, tantivy, and the `superreader` command with JSON output. Writes from that command are applied by the running app, because alan's Seatbelt sandbox denies writes outside mounted directories.

## Reading

Done on 2026-11-03. A single-page PDF and EPUB reader on NSH v0.3.0. PDF pages are pdfium. EPUB pages are WKWebView. The publication's type and layout win. The page box uses the saved trim, or the book's own page size when it declares one. The default trim is paperback, 129 by 198 mm, picked by Morris. 32开 and US Trade stay as the other choices. Pinch zoom and Command zoom re-render the page. Highlights hang off the spike's anchor. No spread, and no night mode.

The Knuth-Plass pass is in the EPUB page. It chooses line breaks and the space between words. Font, size, line height, indent, and alignment stay. `SPIKE_JUSTIFY=0` leaves every paragraph native. A paragraph with CJK stays native. 朝花夕拾 and the plain Chinese fixture report `owned` 0 and `forbidden_starts` 0 at paperback, 32开, and US Trade.

`scripts/verify` scored one release run on 2026-11-04. The record is `docs/verify/summary.json`. With the pass off, the Pride opening was loose. Paperback `gap_p95` was 3.108 on 10 justified lines, 8 of them over 1.5, with 1 river. 32开 `gap_p95` was 3.599 on 10 justified lines, all 10 over 1.5, with 1 river. `docs/design/justification/before.json` stores that census. With the pass on, Pride passed the gap bar at all three trims. Each of those pages had p95 0.982 and max 0.982. Loose lines were 0 of 9 at 32开, 0 of 10 at paperback, and 0 of 10 at US Trade. Hyphenated lines were 0 at 32开, 1 at paperback, and 1 at US Trade. Rivers and overflow were 0. The unstyled English fixture passed paperback at p95 1.472 and max 1.472, with 0 of 12 lines loose and 1 of 12 hyphenated. It passed 32开 at p95 0.925 and max 0.982, with 0 of 12 loose and 1 of 12 hyphenated. US Trade of that fixture missed the hyphen share. 2 of 13 justified lines are hyphenated, and the check allows 15 percent. Gaps on that page passed, at p95 0.982 and max 0.982, with 0 loose lines, 0 rivers, and 0 overflow. The 15 percent check stays.

`OWN_PUBLISHER` stays false. A separate run, with that constant temporarily true, opened the Literata fixture whose own sheet sets justify. Paperback failed at p95 3.421 and max 3.421, with 4 of 12 lines loose. US Trade failed at p95 2.105 and max 2.368, with 3 of 12 lines loose. 32开 passed at p95 0.789, with 0 of 10 loose. `docs/design/justification/publisher-own.json` stores that run. The verify summary still expects `owned` 0, because the constant in the tree is false.

Chapter open missed 300 ms. Five release paperback Pride opens, interleaved with five native opens, measured min 313.8 ms, median 342.5 ms, and max 478.4 ms. The check uses the max as p95. Load averages were 6.70, 8.87, and 12.42 on 10 cores. The error count was 0. The native opens were 313, 312, 343, 345, and 330 ms. All five were over 300 ms, so turning the pass off does not clear the floor. The native median is 330 ms. The median difference is 12.5 ms, and the justify range is 313.8 ms to 478.4 ms, so the pass is not a measurable addition at the median. A sampled open, not one of those ten, put 300 of 524 shell samples in AppKit `finishLaunching`, blocked in dyld while loading WritingToolsUI. The new WebContent process put 433 of 600 samples in `mach_msg`, 53 of them waiting on a backing-store message, and 24 inside JavaScript evaluate. Sampling suspends the process, so that open is not a timing result. The line-breaking function is not the stack that fills the sample. The 300 ms check stays.

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
