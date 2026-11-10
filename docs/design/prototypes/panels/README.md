# Panel forms

Morris picks a card style from these pictures. The window rule is already fixed. Width only decides how many panels can be open. The open or closed choice is remembered, and widening puts that choice back. It never opens a panel that was closed.

The page keeps its trim. Opening a panel recenters it. A drawer floats over the page and does not move it.

Companion and full draw the page as an inset rounded sheet. The gap is 12 points. The corner radius is 14. The sheet is the page column, about 55 percent of the width beside the left bar, and it grows when 55 percent would shrink the trim. The panel sits on the window ground and has no border of its own. A rounded input at the bottom of the panel asks about the page. The left bar is 8 percent of the window, full height, with the traffic lights at the top and circular buttons under them for Now reading, Library, and Agents. Immersive and the narrow drawer stay a flush page, the same shape as window B.

Sample notes are marked Sample or 示例. Replies are marked @alan. They are not saved annotations.

## Card styles

Companion form, one card open and the others collapsed. The crowded picture is the light card with five notes on neighboring lines.

- Margin. Plain text in the margin, a tint on the line, and a plain @alan reply.
- Light card. A white card, a hairline back to the line, and the reply nested under the note.
- Hairline and quote. A quote of the line, a hairline down the card, and the reply under a rule. The line tint is the only link.

The light card is the one to keep. Margin text sinks into the paper. The quote repeats a line the highlight already shows, and on a hyphenated page that quote is a fragment.

## Collapse

Same choice on every frame. Both panels are open. The window closes the left bar first, then turns the right panel into a drawer.

- Mac wide. Full. Left bar, inset page, notes on the ground.
- Mac medium. Companion. The left bar is closed. The page stays an inset sheet.
- Mac narrow. Flush page, notes as a drawer.
- iPad landscape. Mockup, three columns.
- iPad portrait. Mockup, two columns. The left bar stays closed.
- iPad Split View. Mockup, page plus a drawer.

iPad frames are mockups at 1180 by 820, 820 by 1180, and 375 wide. They are built from the same rendered page. Mac frames are the spike shell.

## What should survive the pick

`spikes/shell/src/panels.rs` is the rule above as a pure model. `Choice` is memory. `Surface` sets the maximum. `resolve` returns docked, drawer, or closed. The page size does not change when a panel opens. After the pick, those checks are the ones to run from verify.
