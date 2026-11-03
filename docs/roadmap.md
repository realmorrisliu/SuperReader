# Roadmap

## 1. Feasibility spike

Prove PDF scroll and sharp zoom, single-page EPUB pagination, and a selection anchor that survives re-render. Compare pdfium-render with hayro, and compare a WKWebView paginator with Blitz and with NSH TextView. Record the numbers in `docs/spike/report.md`. No product features in this step.

## 2. Verification skill

Add the verify-superreader skill with `/create-verification-skill`, so later changes are checked against the goals and the walkthroughs.

## Core

The library, SQLite annotations, tantivy, and the `superreader` command with JSON output. Writes from that command are applied by the running app, because alan's Seatbelt sandbox denies writes outside mounted directories.

## Reading

A single-page PDF and EPUB reader on NSH v0.3.0. Pinch zoom and Command zoom re-render the page. Highlights hang off the spike's anchor. No spread, and no night mode.

## Comments

The comment UI is chosen from two or three prototypes. A highlight can carry the reader's note. `@alan` in that note runs the handoff below.

## @alan

Blocked on the external dependency. When the gate is open, `@alan` spawns a fresh one-shot `alan` process with the passage, the note, the earlier comments, and the question on stdin. SuperReader stores stdout as alan's reply. The spawn does not pass `--resume`. The comment shows a thinking state until the whole reply lands.

## External dependency

Owner: Morris. The work is assigned to him on [alan issue 1078](https://github.com/realmorrisliu/alan/issues/1078).

alan needs a launch flag or mode so that a reader-spawned run does not rewrite its global root-rollout pointer. Today every launch rewrites that pointer. A reader-spawned launch would replace Morris's next `alan --resume` session.

Status: pending.

Gate: confirm [alan issue 1078](https://github.com/realmorrisliu/alan/issues/1078) is closed and the flag has landed before the @alan milestone starts. The change lands in the alan repository. This repository does not carry that patch.
