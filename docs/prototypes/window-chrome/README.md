# Window chrome

Morris picked B on 2026-11-04, then the Apple Books toolbar on 2026-11-04. The reading window is the page. At rest the traffic lights, the system title, and the toolbar are hidden. The pointer at the top fades them in. The page chevrons fade in at the top or the side edges. The window background is the paper. Prototypes A and C stay in this folder as the record. They are not in the app.

## The window

The page keeps the trim ratio. Each side mat is 0.094 of the window width, and a 48 point mat sits above the page, clear of the traffic lights. The footer lives in the page's own bottom margin. `N/M页` is centered on the text block and `本章还剩X页` is flush with its right edge. Both use the system font and tabular figures. The column is the space between the top margin and that footer, so a full page ends within one line of the box.

The window opens at 88 percent of the visible frame on the limiting axis. A live resize does not lock the aspect. Extra width becomes even margins. A trim change resizes the window and keeps the top edge. Full screen grows the page to the screen height with the same mats, and the type scales with the page. The page stays centered.

NSH 0.3.0 `set_title_bar(Hidden)` sets `titlebarAppearsTransparent`, `fullSizeContentView`, and a hidden title. From Rust it does not set the window background, clear the titlebar separator, fade the traffic lights, read the visible frame, or scale the web view. The spike does those with objc2 on the `NSWindow` behind the stage. A height of `None` does not install a toolbar, and NSH only removes a toolbar it installed itself.

## Traffic lights

The buttons are AppKit's `standardWindowButton`. The spike only changes their alpha. On this Mac (`sw_vers` 27.0.1, build 26A434, Xcode 27.1, SDK 27.0) the built binary's `LC_BUILD_VERSION` is platform MACOS, minos 11.0, sdk 27.0. The SDK is current, so the old drawing was not an old link. A window with no toolbar still gets the legacy title-bar slots. macOS 27 puts the current, larger corner and the lower button row on a toolbar window. The reading toolbar is a real `NSToolbar` in the unified style. The leading group is table of contents, bookmarks, and search. Add bookmark and notes are separate items. The book title is the window title, in the system face. NSH 0.3.0 sets no appearance and no older-SDK shim that would force the legacy look.

`lights-compare.png` is Finder's corner beside this window, with the pointer at the top.

![Finder and the reading window, traffic lights](lights-compare.png)

## What was tried

| | A | B, chosen | C |
| --- | --- | --- | --- |
| Resting state | Lights hidden, no title | Lights and title hidden | Thin paper bar, title always visible |
| Pointer at the top | Lights fade in | Lights and title fade in | No change |
| Resize | Aspect locked | Even side margins | Aspect locked |

Paperback shots below. A and C are the earlier builds. B is the window in the app now.

### A

![Pride, prototype A, windowed](a-pride.png)

![Pride, prototype A, pointer at the top](a-pride-hover.png)

![朝花夕拾, prototype A, windowed](a-zhaohua.png)

![Pride, prototype A, full screen](a-pride-fullscreen.png)

### B

![Pride, windowed](b-pride.png)

![Pride, pointer at the top](b-pride-hover.png)

![朝花夕拾, windowed](b-zhaohua.png)

![Pride, full screen](b-pride-fullscreen.png)

### C

![Pride, prototype C, windowed](c-pride.png)

![朝花夕拾, prototype C, windowed](c-zhaohua.png)

![Pride, prototype C, full screen](c-pride-fullscreen.png)

`a-hover-resize.mp4` is prototype A. `b-hover-resize.mp4` is the chosen window. The pointer enters the top edge, the lights and the title fade in, and a free resize keeps the page centered.
