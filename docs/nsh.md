# NSH 0.5.0

The shell is pinned at NSH 0.5.0, published on 2026-11-11 as tag `v0.5.0`, commit `f4d68f7`. crates.io reports `0.5.0` as the max stable version of `nsh-core`. A nightly after that tag is not the pin. The license is Apache-2.0.

The app stays a Rust root. `spikes/shell/nsh.json` names `SpikeApp`. Headless checks stay in `spikes/shell/shell.test.tsx`, run by `nsh test`.

## What 0.4.0 changed for this shell

The release notes on [nsh.dev](https://nsh.dev/releases/) list every break between 0.3 and 0.4. The ones this shell hits are these.

Hover handlers are `on_mouse_enter` and `on_mouse_leave`, and each one takes a mouse event. In 0.4, Rust `set_title_bar(style, height)` kept its two arguments. 0.5 removed that function. The TypeScript form in 0.4 became one object, `Window.setTitleBar({ style, height })`. 0.5 replaced the window API with `WindowRef`.

`set_frame` measures the origin from the top-left of the primary screen, with y growing down. This shell still places the window with AppKit frames.

`nsh test` compares values the way Jest does. The four shell tests still use `toContain`.

## What 0.5.0 changed for this shell

The release notes on [nsh.dev](https://nsh.dev/releases/) list every break between 0.4 and 0.5. The ones this shell hits are these.

`set_title_bar` is gone. The shell sets the main window with `current_window().set` and `TitleBarOptions`. The structs are `non_exhaustive`, so the shell starts from `Default` and sets the style to `TitleBarStyle::Hidden` and the height to `chrome::TOP`. Attributes left unset keep their values.

A hidden title bar with that height no longer installs an empty toolbar. NSH centers the traffic lights in the height. It stopped attaching the toolbar because macOS 26 gives a window with a toolbar larger corners than a window with only a title bar. The shell does not put the toolbar back.

`NshAppDelegate` implements `application:openURLs:` and sends the URL string to the core as a scheme event. The shell does not replace that method. `nsh.json` `fileTypes` is how NSH's generated root claims the `file` scheme. This shell's `main` is hand-written, so it sets `Root::url_schemes(["file"])` when `fileTypes` is present. A task waits on `next_open_files` and queues the absolute paths. The reader stage opens them.

`nsh test` on macOS starts the test host in the background and does not activate its window. Pass `--foreground` when a check needs the window in front.

`nsh-updater` is in this stable release. This shell does not set `updater` in `nsh.json` and does not write `latest.json`. What α2 needs is in `docs/release.md`.

## What moved onto NSH

The main window's hidden title bar is `current_window().set` with `TitleBarStyle::Hidden` and height `chrome::TOP` (48 points).

0.4 installed an empty unified toolbar when the height was above 44 points, so the traffic lights sat in that row. 0.5 does not install that toolbar. It centers the traffic lights in the height. A height of `None` still installs no toolbar.

The hidden style makes the title bar transparent, hides the system title, and turns on a full-size content view.

In full screen, the title bar area is zero and AppKit reveals the bar when the pointer reaches the top of the screen. The shell does not add `window:willUseFullScreenPresentationOptions:` to NSH's window delegate, and it does not show or hide a toolbar itself.

The paper is Books-white `#ffffff`. There is no cream or yellow paper. It is the root view background. NSH copies an opaque view that fills the window onto the surface layer, so the title bar and the page are one white sheet.

The View menu is `set_application_menu`. The app, edit, and window menus are NSH menu roles, so Quit, Copy, and full screen stay the system actions. Each trim item calls the stage command `trim`. The check mark follows the trim id the stage publishes on `status`.

## What still needs objc2

The traffic lights fade by setting the alpha of `standardWindowButton`. NSH does not expose that.

The book title in the reading window is an `NSTextField` in the content view, and it fades with the lights. The hidden title bar does not draw that title. While an EPUB is open, the NSH title row is hidden, so this field is the title the reader sees.

`setTitlebarSeparatorStyle` clears the titlebar hairline. NSH does not. The hairline would sit on the paper.

`FullScreenPrimary` stays on the window's collection behavior. NSH sets collection behavior only from `set_level`, and that call replaces the behavior with `MoveToActiveSpace`.

Dragging the page mat uses `performWindowDragWithEvent` on the native stage view. NSH `window_drag` marks NSH views. The stage view has to keep the click so a drag on the page can select text.

The window is ordered front with `activateIgnoringOtherApps`. NSH `order_front` calls `activate`.

`SPIKE_FULLSCREEN` calls `toggleFullScreen` from the view timer, which is outside an NSH owner. The Window menu's full screen item is the NSH role.

The fitted window frame is AppKit `setFrame`. The fit math uses `visibleFrame`, whose origin is the bottom left. NSH frames use the other axis.

## Opened files

File > Open and a drop on the window are NSH. `pick_paths` is the open panel. `on_file_drop` is the drop. Both reach the stage command `open`. A drop on the Dock icon and `open -a` are `application:openURLs:`.

`NshAppDelegate` forwards each URL to the core. The core turns a `file:` URL into a path only when the root claims the `file` scheme, and it keeps that path until `next_open_files` is waiting. The shell's task queues the path. The reader stage opens it. `opens.log` records `queued` plus the path.

`Info.plist` still lists EPUB, including the package form, and PDF. The package script writes those types, and `zh-Hans.lproj/InfoPlist.strings`, before the first `codesign`. The disk image is `Slowpage.app`. This release does not call `nsh package`. That switch is a later cleanup. `nsh package` would write `CFBundleDocumentTypes` from `fileTypes`, and it would not write the imported EPUB type that conforms to `com.apple.package`.

WKWebView, the `epub://` scheme handler, and the pdfium bitmap stay on objc2. They are the page, not the window chrome. PDF via pdfium-render is unchanged. EPUB in WKWebView is under re-evaluation. See `docs/design/north-star.md`.
