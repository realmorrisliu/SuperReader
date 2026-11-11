# Official open

Launch Services opens an EPUB and a PDF through NSH 0.5.0's file path. The proof is `opens.log`.

## Sub-features

- `open-cold-epub` starts Slowpage with `open -a` on Pride and records that path.
- `open-running-pdf` sends the GPT-3 PDF to the app that is already running and records that path.
- `open-dock-equivalent` sends a second EPUB the same way and records that path. A pointer drag onto the Dock icon is not scripted. `open -a` while the app is running is the same `application:openURLs:` delivery.

## How to get to it (user POV)

- Quit Slowpage. Double-click an EPUB, or run `open -a Slowpage` on it. The book opens.
- With Slowpage already running, open a PDF the same way.
- Drop a book on the Dock icon. The book opens.

## Driving it with scripts/verify

Preconditions: fixtures are `/tmp/superreader-fixtures/pride-se.epub` and `/tmp/superreader-fixtures/gpt3.pdf`. `scripts/verify` packages `dist/Slowpage.app` for these three checks. Slowpage must not already be running. `open -a` does not pass `SUPERREADER_SUPPORT_DIR`, so the script moves `~/Library/Application Support/app.slowpage.reader` aside for the three opens and puts it back afterward.

- The full `just verify` run calls `open -a dist/Slowpage.app` three times and reads `/tmp/superreader-spike/opens.log`.
- Each pass row names the path. A missing `queued <path>` line fails that check.

## Gotchas

- Launching `spike-shell` directly does not go through Launch Services. These checks need the `.app`.
- `open -a` does not pass `SUPERREADER_SUPPORT_DIR`. The real Application Support directory is moved aside and restored, including when a check fails.
- A real Dock drag is not in the script. `open-dock-equivalent` is `open -a` on `dock-drop.epub` while the app is running.
- `alpha-finder-open` stays pending. It is the Finder double-click this driver still does not click.
