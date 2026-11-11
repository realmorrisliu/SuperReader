# Alpha install

The installed `SuperReader.app` opens a book, turns a page, and comes back to that page. The dev runner is a different check.

## Sub-features

- `alpha-empty` shows an Open button when nothing is remembered.
- `alpha-epub-open` opens Pride through File > Open.
- `alpha-epub-turn` moves to the next EPUB page.
- `alpha-epub-reopen` launches again on that page.
- `alpha-pdf-open` opens the GPT-3 PDF through File > Open.
- `alpha-pdf-turn` moves to the next PDF page.
- `alpha-pdf-reopen` launches again on that page.
- `alpha-finder-open` stays pending. Finder does not deliver the file.

## How to get to it (user POV)

- Launch SuperReader with no remembered book. The window is empty and has Open.
- File > Open… (⌘O) chooses an `.epub` or a `.pdf`.
- Drop a book on the window.
- The right arrow turns one page.
- Quit and launch again. The same book opens on the same page.
- Finder Open With lists SuperReader. Choosing a file launches the app and does not open that file.

## Driving it with scripts/verify

Preconditions: `/Applications/SuperReader.app` is the packaged build. Fixtures are `/tmp/superreader-fixtures/pride-se.epub` and `/tmp/superreader-fixtures/gpt3.pdf`. `SUPERREADER_SUPPORT_DIR` for the run is empty, so the check does not use a person's library.

- `scripts/verify --subset alpha` launches the app in Applications, drives File > Open, turns a page, quits, and launches again.
- Shots land in `docs/verify/alpha/`.

## Gotchas

- NSH 0.4.0 `NshAppDelegate` does not implement `application:openFile:` or `application:openURLs:`. Do not mark Finder open as passed because the app launched.
- The dev binary under `spikes/target` is not this check. It still opens the fixture from `SPIKE_*`.
- EPUB page numbers come from `/tmp/superreader-spike/webkit.json`. PDF page numbers come from the Accessibility footer `N/M`.
