# Panel forms

Morris picks a card style from these pictures. The window rule is already fixed. Width only decides how many panels can be open. The open or closed choice is remembered, and widening puts that choice back. It never opens a panel that was closed.

The page keeps its trim. Opening a panel recenters it. A drawer floats over the page and does not move it.

Sample notes are marked Sample or 示例. Replies are marked @alan. They are not saved annotations.

## Card styles

Companion form, wide Mac, one card open and the others collapsed. The crowded sheet is the light card with five notes on neighboring lines.

- Margin. Plain text in the margin, a tint on the line, and a plain @alan reply.
- Light card. A white card, a hairline back to the line, and the reply nested under the note.
- Hairline and quote. A quote of the line, a hairline down the card, and the reply under a rule. The line tint is the only link.

The light card is the one to keep. Margin text sinks into the paper. The quote repeats a line the highlight already shows, and on a hyphenated page that quote is a fragment.

## Collapse

Same choice on every frame. Both panels are open. The window closes the left bar first, then turns the right panel into a drawer.

- Mac wide. Full. Left bar, page, notes.
- Mac medium. Companion. The left bar is closed.
- Mac narrow. Page only, notes as a drawer.
- iPad landscape. Mockup, three columns.
- iPad portrait. Mockup, two columns. The left bar stays closed.
- iPad Split View. Mockup, page plus a drawer.

iPad frames are mockups at 1180 by 820, 820 by 1180, and 375 wide. They are built from the same rendered page. Mac frames are the spike shell.

## What should survive the pick

`spikes/shell/src/panels.rs` is the rule above as a pure model. `Choice` is memory. `Surface` sets the maximum. `resolve` returns docked, drawer, or closed. The page size does not change when a panel opens. After the pick, those checks are the ones to run from verify.
