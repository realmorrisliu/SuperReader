# SuperReader verification map

This directory is the maintained source for the checks `scripts/verify` runs. Read it before driving the spike by hand, then use the matching feature file.

## Baseline preconditions

- macOS, with `nsh` 0.4.0 on `PATH`.
- The shell crate depends on `nsh-core`, `nsh-base`, and `nsh-platform-macos` at `=0.4.0`.
- Fixtures at `/tmp/superreader-fixtures/gpt3.pdf`, `/tmp/superreader-fixtures/pride-se.epub`, and `/tmp/pdfium/lib/libpdfium.dylib`, or the `SPIKE_GPT3`, `SPIKE_PRIDE`, and `SPIKE_PDFIUM` overrides.
- No `spike-shell` process is already running.
- Run `scripts/verify --doctor` and require every doctor row to pass.
- Never drive a shell this run did not start.

## Driving conventions

- Start from the repo root.
- Prefer `just verify-only …` plus `just smoke` for a PR. Use `just verify` only before a release tag. See [feature-map.json](../feature-map.json) for path-to-check mapping.
- One verify at a time, and only after the 1-minute load average is under 8.
- Headless UI state goes through `nsh test spikes/shell/shell.test.tsx`.
- Pixels and the raster floors go through the window driver and `spike-measure`.
- Treat footer strings as literal. `1/75` is not `1 / 75`. `1 page left in this chapter` is not `74 pages left`.
- Restore nothing in the spike. There is no account data. Do not delete proof files during cleanup.

## Proof and skip reporting

- Capture the launch and the resulting footer, not only the final picture.
- A window proof includes the Accessibility names and a `screencapture -l` PNG of that pid's window.
- A headless proof includes `nsh-junit.xml`.
- A raster proof includes the five-run median, the range, the error count, and the bitmap check.
- Record the check id on the summary row that holds the artifact.
- A feature the spike does not implement is `pending` in `summary.json`. Do not mark it passed because a nearby check passed.
- Do not report a skipped entry point as verified through a different path.

## Feature entry contract

Each feature file starts with an H1 and one paragraph. It then uses these four H2 sections in order.

1. `Sub-features` lists the check ids.
2. `How to get to it (user POV)` lists every user entry point.
3. `Driving it with scripts/verify` starts with `Preconditions:` and pairs each action with the command and the observable result.
4. `Gotchas` lists traps that waste a run.

## Features

- [Open a PDF](open-a-pdf.md) covers the paper title, the `N/M` footer, the highlight on the ink, and the library and ask paths that are still pending.
- [Read an EPUB](read-an-epub.md) covers the chapter footer, one page turn, and the highlight on the opening sentence.
- [Highlight, comment, and alan](highlight-comment-alan.md) covers the comment, `@alan`, and the `superreader` command. Those checks are pending.
- [Reading bars](reading-bars.md) covers the single page, the absent night mode and theme switch, the zoom re-render, and the two raster floors.
- [Reading view](reading-view.md) covers the one EPUB page, the three trims, the publisher type, and the slide.
- [Window chrome](window-chrome.md) covers the reading window, the page ratio, the column fill, and the footer on the measure.
- [Sidebar](sidebar.md) covers the fixed left bar centered on the traffic lights.
- [Alpha install](alpha.md) covers the packaged app opening an EPUB and a PDF, turning a page, and restoring that page.
