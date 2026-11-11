# Floating card

The content area to the right of the sidebar is one white card. The page is fitted inside that card. Immersive reading removes the inset and the gray field.

## Sub-features

- `card-bounds` at 1100×830 requires the card inset from the content region on all four sides. The content region starts at the bar's right edge and at the 48pt title band. Each inset is 6pt within 0.5pt. `card_inset` is 6. The window is not full screen and not immersive. A pixel in the top gutter, just right of the bar, is field gray `#E4E4E6`. A pixel inside the card and beside the page is `#ffffff`.
- `immersive-button` requires `immersive_button`, and the glyph's right edge within 0.5pt of the rightmost toolbar item. That edge is on the right half of the stage. The control is the IconMCP glyph, not the word Immersive.
- `immersive-roundtrip` posts Control-Command-I. The View menu title becomes `Exit Immersive Reading`. The card is the stage, so each inset from the window edge is 0 within 0.5pt, and the page touches a stage edge. Field-gray pixels outside the page are zero. Escape returns `Enter Immersive Reading`, the 6pt insets, a centered page, and the contents icon placement.

## How to get to it (user POV)

- Launch with `SPIKE_MODE=webkit`. The white card sits 6pt inside the area to the right of the sidebar, below the title band. The page is inside the card. Gray shows only in that gutter.
- Press Control-Command-I, or choose Enter Immersive Reading from the View menu, or click the glyph at the far right of the toolbar. The page goes edge to edge. There is no gray field. The menu item does not bind the shortcut. The key monitor owns Control-Command-I and Escape, so one press toggles once.
- Press Escape, or choose Exit Immersive Reading, or click the glyph again. The 6pt card returns.
- The same shortcut is Control-Command-I. Escape leaves immersive reading and does not enter it.

## Driving it with scripts/verify

Preconditions:

- `scripts/verify --doctor` passes.
- No other `spike-shell` is running.
- `/tmp/superreader-spike/shell-frame`, `bar-immersive`, and `bar-fullscreen` are removed before the shot. The card shots run in the sidebar drive, at 1100×830, before that drive resizes the window.

- **Windowed card.** Launch Pride at 1100 by 830. Hover the top edge so the toolbar is visible. Read `chrome-<pid>.json`. The four insets are 6pt within 0.5pt. Capture `card-reading.png`. The gutter pixel is `#E4E4E6` and the letterbox beside the page is `#ffffff`.
- **Glyph.** The same json has `immersive_button` true. `immersive_frame` right edge matches `toolbar_max_x` within 0.5pt and sits on the right half of `stage_w`.
- **Immersive.** Post Control-Command-I with the key helper (`immersive`, virtual key 34, control and command). Wait until `immersive` is true, `card_inset` is 0, and `immersive_label` is `Exit Immersive Reading`. Capture `card-immersive.png`. Count `#E4E4E6` pixels outside the page rectangle. The count is 0. The page touches a stage edge.
- **Leave.** Post Escape (virtual key 53). `immersive` is false, the label is `Enter Immersive Reading`, the inset is 6, the page is centered in the area beside the bar, and the contents icon is back at its 26pt gap.
- **Proof.** `card-reading.png` is the windowed card at 1100×830. `card-immersive.png` is the same window after Control-Command-I. The sidebar drive copies both to `docs/design/shell/`.

## Gotchas

- The card is the content region inset by 6pt. A card that outlines the page by 6pt is the old geometry and fails `card-bounds`.
- Immersive reading fits the page to the stage. Recentering the windowed page leaves a gray field and fails the zero-gray count.
- The toolbar item id is `books.immersive`, last in `chrome-b-toolbar-order`. The visible control is the glyph. Its accessibility label is Immersive.
- The file `/tmp/superreader-spike/bar-immersive` wins over the key. The roundtrip runs before that flag is written.
- Full screen keeps today's page box and a gray sidebar column. Immersive reading wins when both are on.
- This Linux VM cannot launch the shell. `scripts/verify --self-test` covers the inset math and the pixel rules. The three window rows run on the Mac.
