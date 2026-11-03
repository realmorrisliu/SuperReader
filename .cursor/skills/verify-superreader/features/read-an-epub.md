# Read an EPUB

Opening an EPUB shows one page of the chapter, the book title, and a footer with `N/M` and the pages left in the chapter. Turning a page replaces this page. A highlight on the opening sentence is still on that sentence after a reload.

## Sub-features

- `epub-title` shows Pride in the toolbar.
- `epub-footer-nm` shows one `N/M`.
- `epub-pages-left-in-chapter` shows `N page left in this chapter` or `N pages left in this chapter`.
- `epub-turn` shows `2/N` after one turn.
- `epub-highlight-on-sentence` shows the anchor text `truth universally acknowledged` and a yellow band with ink on the page.

## How to get to it (user POV)

- Press `3` for the WKWebView chapter, then `8` for Pride.
- Press `right` to turn forward, or `left` to turn back.
- Drag the sentence. The scripted launch uses `SPIKE_SELECT=1`, which paints `truth universally acknowledged`.

## Driving it with scripts/verify

Preconditions:

- `scripts/verify --doctor` passes.
- `pride-se.epub` is the path doctor printed.

- **Open the chapter.** Launch with `SPIKE_MODE=webkit`, `SPIKE_SELECT=1`, and `SPIKE_RELOAD=1`. Wait until Accessibility shows a page count above 1 and the chapter remainder. `epub-pride.png` is that window.
- **Turn.** A second launch with `SPIKE_MODE=webkit` and `SPIKE_TURN=1` waits until the footer starts with `2/`.
- **Highlight.** The first launch writes the anchor. The copied `epub-anchor.json` contains `truth universally acknowledged`, and the PNG's yellow band contains dark ink.
- **Proof.** The five rows above are `pass` in `summary.json`.

## Gotchas

- The page count stays `1/1` until WebKit reports columns. A shot taken before that is a failure, not a one-page book.
- WKWebView hides the NSH footer from Accessibility. The script reads `1/2` and the chapter remainder from the PNG with Vision.
- The mark is inside WKWebView. `nsh test` cannot see it. The pixel count is the proof that the mark is on the ink.
- `SPIKE_RELOAD=1` reloads after the selection. The shot is the mark after that reload.
- Press `4` or `5` and the stage leaves WKWebView. Those modes are not this feature.
