# NSH 0.4.0

The shell was pinned at NSH 0.3.0. The latest stable release is 0.4.0, published on 2026-11-04 as tag `v0.4.0`, commit `e626a8d`. crates.io reports `0.4.0` as the max stable version of `nsh-core`. The nightly built on 2026-11-08 is not the pin. The license is Apache-2.0.

The app stays a Rust root. `spikes/shell/nsh.json` names `SpikeApp`. Headless checks stay in `spikes/shell/shell.test.tsx`, run by `nsh test`.

## What 0.4.0 changed for this shell

The release notes on [nsh.dev](https://nsh.dev/releases/) list every break between 0.3 and 0.4. The ones this shell hits are these.

Hover handlers are `on_mouse_enter` and `on_mouse_leave`, and each one takes a mouse event. Rust `set_title_bar(style, height)` keeps its two arguments. The TypeScript form is now one object, `Window.setTitleBar({ style, height })`.

`set_frame` measures the origin from the top-left of the primary screen, with y growing down. This shell still places the window with AppKit frames.

`nsh test` compares values the way Jest does. The four shell tests still use `toContain`.

## What moved onto NSH

`set_title_bar(TitleBarStyle::Hidden, Some(px(48)))` now does the window work the shell used to do with objc2.

The height is above 44 points, so NSH installs an empty unified toolbar. That is the current macOS traffic-light row. A height of `None` still installs no toolbar. That is why 0.3 had to add one.

The same call makes the title bar transparent, hides the system title, and turns on a full-size content view.

In full screen, NSH hides the toolbar it owns and shows it again afterwards. AppKit then reveals the title bar when the pointer reaches the top of the screen. The shell no longer adds `window:willUseFullScreenPresentationOptions:` to NSH's window delegate, and it no longer shows or hides the toolbar itself.

The paper `#f3eee4` is the root view background. NSH 0.4 copies an opaque view that fills the window onto the surface layer, so that color is the window background.

The View menu is `set_application_menu`. The app, edit, and window menus are NSH menu roles, so Quit, Copy, and full screen stay the system actions. Each trim item calls the stage command `trim`. The check mark follows the trim id the stage publishes on `status`.

## What still needs objc2

The traffic lights fade by setting the alpha of `standardWindowButton`. NSH does not expose that.

`Toolbar` in `nsh-base` is an in-content bar. It does not add items to the unified toolbar `set_title_bar` installs, and that toolbar stays empty. The reading items are an `NSToolbarItemGroup` and `NSToolbarItem`s on the toolbar NSH already created. NSH has no slider, so the progress scrubber is an `NSSlider` with chapter ticks drawn beside it. The page chevrons and the jump-back buttons are `NSButton`s in the page margin. The PDF page count is two more `NSTextField`s on a paper band at the bottom of the page, so it does not paint over the bitmap. The title field uses the system semibold face.

The book title in the reading window is an `NSTextField` in the content view, and it fades with the lights. The hidden title bar does not draw that title. While an EPUB is open, the NSH title row is hidden, so this field is the title the reader sees.

`setTitlebarSeparatorStyle` clears the titlebar hairline. NSH does not. The hairline would sit on the paper.

`FullScreenPrimary` stays on the window's collection behavior. NSH sets collection behavior only from `set_level`, and that call replaces the behavior with `MoveToActiveSpace`.

Dragging the page mat uses `performWindowDragWithEvent` on the native stage view. NSH `window_drag` marks NSH views. The stage view has to keep the click so a drag on the page can select text.

The window is ordered front with `activateIgnoringOtherApps`. NSH `order_front` calls `activate`.

`SPIKE_FULLSCREEN` calls `toggleFullScreen` from the view timer, which is outside an NSH owner. The Window menu's full screen item is the NSH role.

The fitted window frame is AppKit `setFrame`. The fit math uses `visibleFrame`, whose origin is the bottom left. NSH frames use the other axis.

WKWebView, the `epub://` scheme handler, and the pdfium bitmap stay on objc2. They are the page, not the window chrome. PDF via pdfium-render is unchanged. EPUB in WKWebView is under re-evaluation. See `docs/design/north-star.md`.
