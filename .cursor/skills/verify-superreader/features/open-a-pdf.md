# Open a PDF

Opening a PDF shows one page of the paper, the book title in the toolbar, and `N/M` plus the pages left in the footer. A drag highlight stays on the ink. Asking an agent about the passage is not in the spike.

## Sub-features

- `pdf-title` shows `Language Models are Few-Shot Learners` in the toolbar.
- `pdf-footer-nm` shows one `N/M` and `N pages left`.
- `pdf-highlight-ink` shows a yellow band with dark ink inside it.
- `open-pdf-from-library` is pending. There is no library.
- `open-pdf-ask` is pending. Nothing explains a selection.
- `open-pdf-scroll-column` is pending. The native view scrolls, and this run does not score a scroll.

## How to get to it (user POV)

- Launch the spike. It opens the GPT-3 PDF with pdfium.
- Press `1` for pdfium, or `2` for hayro.
- Press `mod+plus` for zoom 2x, or `mod+minus` for retina 1x.
- Drag across a line of type. The spike's `SPIKE_DRAG=1` launch does the two-line drag from `Recent`.

## Driving it with scripts/verify

Preconditions:

- `scripts/verify --doctor` passes.
- `gpt3.pdf` and `libpdfium.dylib` are the paths doctor printed.

- **Open the paper.** Launch with `SPIKE_DRAG=1`. The script starts `spike-shell`. Accessibility shows `Language Models are Few-Shot Learners`, one `N/M`, and `N pages left`.
- **Highlight.** The same launch drags two lines. `pdf-highlight-1x.png` has a yellow band with dark ink in it. The check fails when the band is missing or the ink is gone.
- **Proof.** `summary.json` rows `pdf-title`, `pdf-footer-nm`, and `pdf-highlight-ink` are `pass`, and the PNG is the evidence.

## Gotchas

- The PDF footer says `74 pages left`. It does not say `in this chapter`. That chapter sentence is the EPUB footer.
- Headless `nsh test` does not see this footer. The stage publishes it only in the real window.
- Zoom 2x draws the page at twice the point size, so a highlight that is not scrolled into view is off the window. `SPIKE_DRAG=1` scrolls to the drag.
- Do not mark `open-pdf-from-library` or `open-pdf-ask` passed because the PDF opened.
