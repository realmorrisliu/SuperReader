# Shell

The approved window is the gray base, one white card, the version B sidebar, and a right region that changes with the form. The library form's right region is the chat panel. The reading form's right region is the annotation margin. The layout reference is `docs/design/shell/spec.md`. None of these checks exist in the spike yet. Each stays `pending` until the slice that owns it passes. Pending does not fail the exit code.

## Sub-features

- `shell-card-inset` is S1. The card is inset 6 points from the base on every side in Full, with the sidebar and the chat panel open.
- `shell-no-dividers` is S1. The gaps between the sidebar, the card, and the chat panel contain no divider pixels.
- `shell-sidebar` is S1. The sidebar is 220 points wide and lists **书库**, **笔记**, **书单**, **来源**, and **设置** in that order, with the rows named in the spec.
- `shell-panel-open` is S1. Launch shows the chat panel open, with the placeholder **问问 Slowpage…**.
- `shell-cover-grid` is S2. The card shows a 2 by 3 cover grid, the title and the author under each cover, and a **继续阅读** row. The books come from `Catalog`.
- `shell-reading-hides-sidebar` is S3. Opening a book hides the sidebar and keeps the page in the card. The right region is the annotation margin.
- `shell-reading-margin` is S3, and it has no pixel bar yet. Alignment, collision stacking, and the way back to the free-form chat wait for Morris's mockup. Do not invent those pixels in the check.
- `shell-immersive-no-gray` is S4. Immersive reading shows no gray field. The card meets the window edges.

## How to get to it (user POV)

- Launch the app. Full shows the sidebar, the library card, and the chat panel.
- Open a book from a cover or from **继续阅读**. The sidebar hides. The page is in the card. The right region is the annotation margin.
- Press Control-Command-I, or use the View menu item, or use the toolbar button. The card goes edge to edge. Escape leaves that form.
- Choose **全部书籍** to return to the library and the chat panel.

S3's reading checks wait until Morris approves the margin mockup. Do not drive a substitute margin.

## Driving it with scripts/verify

Preconditions:

- `scripts/verify --doctor` passes.
- No other `spike-shell` is running.
- The ids below are not in `scripts/verify` yet. The slice that builds the form adds its ids. Until then the summary has no row, and a missing row is not a pass.

- **Inset.** In Full, measure the card against the base on all four sides. Each gap is 6 points, within 0.5 points. `shell-card-inset` stays `pending` until that measurement passes.
- **Dividers.** Sample the bands between the sidebar and the card, and between the card and the chat panel. A divider pixel fails. `shell-no-dividers` stays `pending` until a clean sample passes.
- **Sidebar.** The sidebar width is 220 points, within 0.5 points. The group labels are **书库**, **笔记**, **书单**, **来源**, and pinned **设置**. `shell-sidebar` stays `pending` until that reading passes.
- **Chat panel.** At launch the chat panel is open and the field shows **问问 Slowpage…**. `shell-panel-open` stays `pending` until that reading passes.
- **Grid.** With a non-empty `Catalog`, the card shows the cover grid and **继续阅读**. `shell-cover-grid` stays `pending` until that reading passes.
- **Reading.** Open a book. The sidebar is absent and the page is inside the card. `shell-reading-hides-sidebar` stays `pending` until that reading passes. `shell-reading-margin` stays `pending` with no pixel bar until the mockup lands and a later check names the pixels.
- **Immersive.** Enter immersive reading. The gray field is absent. `shell-immersive-no-gray` stays `pending` until that sample passes.
- **Proof.** A passing row names the check id and attaches the window PNG. A pending row is not `pass`.

## Gotchas

- `sidebar-centered` still scores the 2026-11-10 measured-width bar. Do not pass `shell-sidebar` from it.
- `shelf-empty` scores the bare shelf buttons. Do not pass `shell-cover-grid` or the empty card from it. S1 deletes those shelf checks.
- Pull request 39 insets a page sleeve on the old bar. Do not pass `shell-card-inset` from that branch.
- Hiding the chat panel is not immersive reading, and it is not the annotation margin. `shell-immersive-no-gray` requires the gray field to be gone.
- Do not pass `shell-panel-open` while a book is open. That check is the library chat panel at launch. The margin mockup has not settled whether a chat field can appear anywhere else during reading.
- S5 adds no night-mode check. The 2026-11-03 rule still forbids night mode and a theme switch.
