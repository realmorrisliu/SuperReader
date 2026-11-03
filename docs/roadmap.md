# Roadmap

## 1. Feasibility spike

Done on 2026-11-03. PDF is pdfium. EPUB is the system WKWebView. The comparison, including why hayro lost on text, is in `docs/spike/report.md`. No product features in this step.

## 2. Verification skill

Add the verify-superreader skill with `/create-verification-skill`, so later changes are checked against the goals and the walkthroughs.

## Core

The library, SQLite annotations, tantivy, and the `superreader` command with JSON output. Writes from that command are applied by the running app, because alan's Seatbelt sandbox denies writes outside mounted directories.

## Reading

Done on 2026-11-03. A single-page PDF and EPUB reader on NSH v0.3.0. PDF pages are pdfium. EPUB pages are WKWebView. The publication's type and layout win. The page box uses the saved trim, or the book's own page size when it declares one. The default trim is paperback, 129 by 198 mm, picked by Morris. 32开 and US Trade stay as the other choices. Pinch zoom and Command zoom re-render the page. Highlights hang off the spike's anchor. No spread, and no night mode.

Next. Knuth-Plass justification, tex-linebreak, MIT, in the EPUB WKWebView. Approved by Morris 2026-11-03. Quality bar. Word gap p95 at or under 1.5 times a normal space, and max at or under 2.0 times. At most 3 percent of lines over 1.5 times. At most one river per page. Hyphenated lines at or under 15 percent in English, and at most two in a row. Zero line overflow. Zero CJK lines that start with forbidden punctuation. Chapter open p95 at or under 300 ms. Publisher computed styles unchanged. Pretext, chenglou/pretext, waits until figure placement.

## Comments

The comment UI is chosen from two or three prototypes. A highlight can carry the reader's note. `@alan` in that note runs the handoff below.

## @alan

Blocked on the external dependency. When the gate is open, `@alan` spawns a fresh one-shot `alan` process with the passage, the note, the earlier comments, and the question on stdin. SuperReader stores stdout as alan's reply. The spawn does not pass `--resume`. The comment shows a thinking state until the whole reply lands.

## External dependency

Owner: Morris. The work is assigned to him on [alan issue 1078](https://github.com/realmorrisliu/alan/issues/1078).

alan needs a launch flag or mode so that a reader-spawned run does not rewrite its global root-rollout pointer. Today every launch rewrites that pointer. A reader-spawned launch would replace Morris's next `alan --resume` session.

Status: pending.

Gate: confirm [alan issue 1078](https://github.com/realmorrisliu/alan/issues/1078) is closed and the flag has landed before the @alan milestone starts. The change lands in the alan repository. This repository does not carry that patch.
