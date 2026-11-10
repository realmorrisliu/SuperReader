# Panel forms

Morris picks a card style from these pictures. The window rule is already fixed. Width only decides how many panels can be open. The open or closed choice is remembered, and widening puts that choice back. It never opens a panel that was closed.

The page keeps its trim. Opening a panel recenters it. A drawer floats over the page and does not move it.

The window base is `#E4E4E6`. The page is a white floating card, `#FFFFFF`, inset 6 points, with a 9 point corner radius and a soft shadow. Companion and full keep the left bar and the notes on the gray base. The card is the page column, about 55 percent of the width beside the left bar, and it grows when 55 percent would shrink the trim. The default reading card keeps that thin margin. Immersive reading is a separate explicit mode, View menu Enter Immersive Reading, Control-Command-I. It fills the window edge to edge, and Esc or the same shortcut leaves it. Full screen stays Control-Command-F and does not enter immersive. The narrow frame puts the notes drawer over the white card.

Sample notes are marked Sample or 示例. Replies are marked @alan. They are not saved annotations.

## Card styles

Companion form, one card open and the others collapsed. The crowded picture is the light card with five notes on neighboring lines.

- Margin. Plain text in the margin, a tint on the line, and a plain @alan reply.
- Light card. A white card, a hairline back to the line, and the reply nested under the note.
- Hairline and quote. A quote of the line, a hairline down the card, and the reply under a rule. The line tint is the only link.

The light card is the one to keep. On the gray base it still reads as marginalia. Margin text has no edge of its own. The quote repeats a line the highlight already shows, and on a hyphenated page that quote is a fragment.

## The four points

`docs/design/north-star.md` fixes the panel. Every card style shows the same four points.

- Cards sit on their lines. A crowded group flows down, and the selected card snaps to its line.
- The open card has an @alan thread under the annotation.
- The input stays at the bottom. A note labeled Current page sits above it. It is attached to the current page and moves when the page turns. The before frame is the companion picture. The after frame is `page-after-`.
- A closed card shows the annotator and the first sentence. The selected card is expanded, and that is where the rest of the note and the @alan reply appear.

`requirements-pride.png` and `requirements-zhaohua.png` label the four points. Margin, light card, and quote each have a crowded frame and a page-turn frame. The Pride crowded frames are page 2, so the stack has room to flow down.

## Collapse

Same choice on every frame. Both panels are open. The window closes the left bar first, then turns the right panel into a drawer.

- Mac wide. Full. Gray base, floating white page, notes on the base.
- Mac medium. Companion. The left bar is closed. The page stays a floating card.
- Mac narrow. Drawer over the floating card.
- iPad landscape. Mockup, three columns, floating card.
- iPad portrait. Mockup, two columns. The left bar stays closed.
- iPad Split View. Mockup, floating card plus a drawer.
- Default. Thin margin around the white card. This is the reading card.
- Immersive. Edge to edge, entered only from the View menu or Control-Command-I.
- Enter and exit. `enter-exit-pride.png` shows the menu, the transition, and the return.

iPad frames are mockups at 1180 by 820, 820 by 1180, and 375 wide. They are built from the same rendered page. Mac frames are the spike shell.

## What should survive the pick

`spikes/shell/src/panels.rs` is the rule above as a pure model. `Choice` is memory. `Surface` sets the maximum. `resolve` returns docked, drawer, or closed. The page size does not change when a panel opens. After the pick, those checks are the ones to run from verify.
