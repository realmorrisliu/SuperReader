# Reading bars

The bars in `docs/design/north-star.md` are the single page, the footer, the absent night mode and theme switch, the highlight after a zoom re-render, and the two raster floors. The stack pins NSH at 0.3.0 and keeps the AGPL and GPL crates out.

## Sub-features

- `nsh-title` shows `SuperReader spike` on the headless tree.
- `nsh-no-night-mode` finds no `night` in the headless tree.
- `nsh-no-theme-switch` finds no `theme` in the headless tree.
- `nsh-no-spread` finds no `spread` in the headless tree.
- `single-page` finds one `N/M` on the PDF window and one on the EPUB window.
- `pdf-highlight-ink-zoom` finds ink inside the yellow band at zoom 2x.
- `pdf-highlight-zoom-scale` finds the zoom band at least 1.5 times as tall as the retina band.
- `pdf-open-to-first` keeps the five-run median under 100 ms.
- `pdf-later-page` keeps the five-run later-page median at or under 16.7 ms.
- `nsh-pin` and `nsh-cargo-pin` require NSH 0.3.0.
- `no-agpl-gpl` rejects the `epub` crate and `mupdf`.
- `library-search-tantivy` is pending. There is no index.

## How to get to it (user POV)

- The window has a toolbar, one page, and a footer. There is no theme control and no spread control.
- Press `mod+plus`. The page is rastered again at zoom 2x, and the highlight is redrawn from the same anchor.
- Night mode is not a setting. Adding one fails `nsh-no-night-mode`.

## Driving it with scripts/verify

Preconditions:

- `scripts/verify --doctor` passes.
- The machine can run five release rasters of the 75-page GPT-3 PDF.

- **Chrome.** `nsh test spikes/shell/shell.test.tsx` opens `SpikeApp` headless. The four `nsh-*` rows follow the JUnit file.
- **One page.** The PDF and EPUB launches each expose one `N/M` string. Two numbers fail `single-page`.
- **Zoom.** The zoom 2x launch uses `SPIKE_DRAG=1` and `SPIKE_SCALE=zoom-2x`. The band is taller than the retina band, and the ink is still inside it.
- **Floors.** `spike-measure pdf pdfium retina-1x` runs five times. `pdf-open-to-first` fails when the median is 100 ms or more. `pdf-later-page` fails when the median is over 16.7 ms. Errors, a short bitmap, or fewer than two phrase hits fail both rows.
- **Proof.** The summary detail for the floors includes the median, the range, the load average, the error count, and the run count.

## Gotchas

- One open-to-first sample in the spike report reached 118 ms while the median stayed under 100. Judge the median of five runs, and keep the range in the detail.
- The later-page max is often far above 16.7 ms. The floor is the median.
- `nsh test` passing the chrome does not prove the footer numbers. Those are window checks.
- Building `spike-shell` and `spike-measure` in one `cargo` command enables taffy's `calc` feature and breaks `nsh-core` 0.3.0. The script builds them separately.
- `library-search-tantivy` stays pending until an index exists.
