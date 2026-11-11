# Library shelf

The installed app opens on an empty shelf. A dev launch opens the shelf when `SUPERREADER_LIBRARY`, `SUPERREADER_CALIBRE`, or `SPIKE_ONBOARDING` is set, and otherwise keeps the GPT-3 reader so the reading checks still start on the paper. **Import from Apple Books** reads the Apple Books catalog, **calibre library** reads a Calibre folder, and a ready title opens in the existing EPUB or PDF path. Nothing reads the Apple Books container until the import button runs. **Library** returns from a book to the shelf.

## Sub-features

- `shelf-empty` finds the drag hint and the three buttons, and does not find fixture titles.
- `shelf-list` presses **Import from Apple Books** and finds **Plain Essay**, **Field Guide**, **Folder Book**, plus the quiet result line.
- `shelf-open` presses **Plain Essay** after that import and hides the other fixture titles.
- `shelf-open-pdf` presses **Field Guide** and hides the EPUB titles.
- `shelf-switch` opens **Plain Essay**, returns with **Library**, opens **Field Guide**, then returns and opens **Plain Essay** again.
- `shelf-streaming` presses import on a manual clock and sees the fixture shelf grow across at least two frames before the result line.
- `shelf-calibre` presses **calibre library** and finds **Harbor Notes**.
- `shelf-denied` presses import against an unreadable directory and finds **Choose Folder** in place of the Apple Books button.
- `shelf-window-empty`, `shelf-window-epub`, `shelf-window-pdf`, and `shelf-window-switch` are the Mac window shots. The EPUB and PDF shots must contain ink, so a blank PDF page fails.
- `smoke-launch`, `smoke-open`, and `smoke-turn` are the short smoke on the same subset: launch the default PDF, confirm it is open, and turn one page with the right arrow. They are not the full reading suite.
- `open-pdf-from-library` stays pending. The spike still opens gpt3.pdf from SPIKE_GPT3 for the reading bars. The shelf window checks above are the library open.

## How to get to it (user POV)

- Launch the installed Slowpage app, or `spike-shell` with `SUPERREADER_LIBRARY` set and no reader-forcing `SPIKE_*` variables. The middle of the window is the empty shelf: **Drop a book to open it**, **Import from Apple Books**, **calibre library**, and **Folder**.
- Press **Import from Apple Books**. Each catalogued book appears on its own, and the quiet line follows the last one, for example `Imported 3, 1 store-protected unavailable, 1 not downloaded from iCloud`. A store or not-downloaded row is grey and does not open.
- If Apple Books is not readable, that button becomes **Choose Folder** and opens a folder panel.
- Press **calibre library** to read `~/Calibre Library`, or the folder named by `SUPERREADER_CALIBRE`. If neither is a Calibre library, the button opens the folder panel.
- Press **Folder** to choose a library folder. The choice is stored as `libraryFolder` in `settings.json`.
- Press a ready row to open it. The variant line shows the book title. **Library** returns to the shelf so another book can replace the open one. Opening a PDF drops the EPUB web view, and opening an EPUB drops the PDF. A product PDF is fitted inside the window so the page is not the blank margin under the title.
- Drop an EPUB file, an exploded EPUB directory, or a PDF to skip the shelf. Passing that path to the app does the same.

## Driving it with scripts/verify

Preconditions:

- `scripts/verify --doctor` passes.
- `spikes/library/fixtures/apple` and `spikes/library/fixtures/calibre` exist in the checkout.
- `SPIKE_LANG=en` is set by `run_nsh`. The button strings in the headless tests are the English ones.

- **Headless shelf.** `run_nsh` sets `SUPERREADER_LIBRARY` to the apple fixture and `SUPERREADER_CALIBRE` to the calibre fixture, then runs `nsh test shell.test.tsx`. It does not load either fixture until a button is pressed. `child_env` pops both variables so parent shells cannot leak them into spike-shell runs.
- **Empty.** `empty shelf offers Apple Books import` expects the hint and the three buttons, and expects **Plain Essay** and **Harbor Notes** to be absent.
- **Import.** `importing the apple fixture lists the books and a result line` presses **Import from Apple Books** and expects the three ready titles, **Store**, **Not downloaded**, and `Imported 3, 1 store-protected unavailable, 1 not downloaded from iCloud`.
- **Open.** `pressing Plain Essay leaves the other books` imports, then calls `app.press("Plain Essay")` and expects only **Plain Essay** among those titles.
- **Streaming.** `shelf grows across frames while importing` opens with a manual clock, presses import, and expects fewer than the six fixture titles and no result line. `waitUntil` then advances frame by frame until a later frame has more titles and `Imported 3, 1 store-protected unavailable, 1 not downloaded from iCloud`.
- **Calibre.** `calibre fixture opens from its button` presses **calibre library** and expects **Harbor Notes**.
- **Denied.** A second `nsh test shell.access.test.tsx` sets `SUPERREADER_LIBRARY` to a mode `000` directory. `denied apple import turns the button into a folder picker` expects **Choose Folder** and does not press it.
- **Smoke.** After the shelf window, `run_smoke` launches `spike-shell` with no library env so the default PDF opens, then sends the right arrow. `reader.json` must move from page 1 to a later page. Shots are `shelf-smoke-open.png` and `shelf-smoke-turn.png`. `--subset shelf` records only `shelf-*` headless checks plus these window and smoke rows. It does not run the rest of the suite.

## Gotchas

- Reader-forcing launches skip the shelf and keep today's PDF or EPUB entry points. `SPIKE_ONBOARDING` still shows the preview states and does not read a library.
- `nsh test` never constructs `reader.stage`; shelf-open only checks visible text, not pixels.
- Fixture titles must avoid the substrings `night`, `theme`, and `spread` in the four legacy chrome tests. Those tests see the empty shelf.
- `cargo test -p spike-library` is the Linux-safe check. `spike-shell` still requires macOS AppKit.
- The denied directory has to be unreadable by the user running verify. Root can still read a mode `000` directory, so that check fails when verify runs as root.
- `SR_VERIFY_MAX_LOAD` raises the 1-minute load ceiling (default 8). `999` starts without waiting for load.
- S1 removes this bare shelf screen and these checks. Do not repair the centered import buttons so they match the new card. The library card checks are in `shell.md`. `spikes/library` stays.
