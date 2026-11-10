# Contents icon

Before is the reading window with `SPIKE_ICON=0`. After draws the IconMCP contents icon as the first item in the unified toolbar. AppKit places that item after the traffic lights and centers it on them. On this Mac the gap from the zoom button's trailing edge to the item's leading edge is 26 pt, and the vertical delta is 0 pt. The image is a template `NSImage` rasterized at runtime with resvg. The SVG is compiled into the binary.

`SPIKE_ICON=sf` draws the SF Symbol `list.bullet` in that same item. That crop is the reference. It is not shipped.

These captures use Books-white `#ffffff`. That color came from a scratch build so the shots match the paper in PR #15. This branch still paints `#f3eee4`.

`SPIKE_APPEARANCE` sets the window appearance for the shot. Dark aqua paints a template image light, so both glyphs disappear on the white page. There is no sidebar, so there is no sidebar crop.

![Before, light](before-light.png)

![After, light](after-light.png)

![Before, dark](before-dark.png)

![After, dark](after-dark.png)

Toolbar, 2x, left side.

![Before, light, toolbar](before-light-toolbar.png)

![After, light, toolbar](after-light-toolbar.png)

![Before, dark, toolbar](before-dark-toolbar.png)

![After, dark, toolbar](after-dark-toolbar.png)

SF Symbol `list.bullet`, same spot, 2x. Reference, not shipped.

![Reference, light](reference-light-toolbar.png)

![Reference, dark](reference-dark-toolbar.png)

Light and dark sheets. Left is the SF Symbol reference. Right is IconMCP.

![Sheet, light](sheet-light.png)

![Sheet, dark](sheet-dark.png)
