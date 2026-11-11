use std::cell::{Cell, RefCell};

use nsh_platform_macos::objc2::rc::Retained;
use nsh_platform_macos::objc2::runtime::{AnyObject, NSObjectProtocol, ProtocolObject};
use nsh_platform_macos::objc2::{
    define_class, msg_send, DefinedClass, MainThreadMarker, MainThreadOnly,
};
use objc2_app_kit::{
    NSAppearance, NSAppearanceCustomization, NSAppearanceNameAqua, NSAppearanceNameDarkAqua,
    NSButton, NSColor, NSFont, NSFontWeightRegular, NSImage, NSImageAlignment, NSImageScaling,
    NSImageSymbolConfiguration, NSImageView, NSLineBreakMode, NSScreen, NSTextField,
    NSTitlebarSeparatorStyle, NSToolbar, NSToolbarDelegate, NSToolbarDisplayMode,
    NSToolbarFlexibleSpaceItemIdentifier, NSToolbarItem, NSView, NSWindow, NSWindowButton,
    NSWindowCollectionBehavior, NSWindowStyleMask,
};
use objc2_foundation::{NSArray, NSPoint, NSRect, NSSize, NSString};

#[path = "icons.rs"]
mod icons;

pub const SIDE: f64 = 52.0;
pub const TOP: f64 = 48.0;
pub const FIT: f64 = 0.88;

#[derive(Clone, Copy, Debug)]
pub struct Frame {
    pub window_w: f64,
    pub window_h: f64,
    pub page_w: f64,
    pub page_h: f64,
}

#[derive(Clone, Copy, Debug)]
pub struct PageBox {
    pub x: f64,
    pub y: f64,
    pub w: f64,
    pub h: f64,
    pub stage_w: f64,
    pub stage_h: f64,
    pub ratio_w: f64,
    pub ratio_h: f64,
}

#[derive(Clone, Copy, Debug)]
pub struct TrafficLights {
    pub close_min_x: f64,
    pub close_max_x: f64,
    pub close_min_y: f64,
    pub close_max_y: f64,
    pub mini_min_x: f64,
    pub mini_max_x: f64,
    pub mini_min_y: f64,
    pub mini_max_y: f64,
    pub zoom_min_x: f64,
    pub zoom_max_x: f64,
    pub zoom_min_y: f64,
    pub zoom_max_y: f64,
    pub width: f64,
}

pub struct Runtime {
    settled: Cell<bool>,
    stable: Cell<u8>,
    attempts: Cell<u8>,
    shown: Cell<bool>,
    pub full: Cell<bool>,
    pub product: Cell<bool>,
    fullscreen_sent: Cell<bool>,
    pub lights: Cell<f64>,
    ratio_key: Cell<(i32, i32)>,
    page: Cell<PageBox>,
    title: RefCell<String>,
    styled: Cell<bool>,
    entering: Cell<bool>,
    field: RefCell<Option<Retained<NSTextField>>>,
    contents_bar: RefCell<Option<Retained<ContentsBar>>>,
    icon_gap: Cell<f64>,
    icon_dy: Cell<f64>,
    pub design_visible: Cell<(f64, f64)>,
    pub bar_w: Cell<f64>,
    pub bar_dirty: Cell<bool>,
    bar_seen: Cell<f64>,
    bar_stable: Cell<u8>,
    frame_key: Cell<(i32, i32)>,
    toolbar_style: Cell<isize>,
    style_mask: Cell<usize>,
    pub bar_gen: Cell<u32>,
    pub immersive: Cell<bool>,
    bar_fullscreen_sent: Cell<bool>,
    pub laying_out: Cell<bool>,
    traffic: RefCell<Option<TrafficLights>>,
}

impl Default for Runtime {
    fn default() -> Self {
        Self {
            settled: Cell::new(false),
            stable: Cell::new(0),
            attempts: Cell::new(0),
            shown: Cell::new(false),
            full: Cell::new(false),
            product: Cell::new(false),
            fullscreen_sent: Cell::new(false),
            lights: Cell::new(0.0),
            ratio_key: Cell::new((0, 0)),
            page: Cell::new(PageBox {
                x: 0.0,
                y: 0.0,
                w: 0.0,
                h: 0.0,
                stage_w: 0.0,
                stage_h: 0.0,
                ratio_w: 0.0,
                ratio_h: 0.0,
            }),
            title: RefCell::new(String::new()),
            styled: Cell::new(false),
            entering: Cell::new(false),
            field: RefCell::new(None),
            contents_bar: RefCell::new(None),
            icon_gap: Cell::new(f64::NAN),
            icon_dy: Cell::new(f64::NAN),
            design_visible: Cell::new((0.0, 0.0)),
            bar_w: Cell::new(0.0),
            bar_dirty: Cell::new(true),
            bar_seen: Cell::new(0.0),
            bar_stable: Cell::new(0),
            frame_key: Cell::new((0, 0)),
            toolbar_style: Cell::new(-2),
            style_mask: Cell::new(usize::MAX),
            bar_gen: Cell::new(0),
            immersive: Cell::new(false),
            bar_fullscreen_sent: Cell::new(false),
            laying_out: Cell::new(false),
            traffic: RefCell::new(None),
        }
    }
}

impl Runtime {
    pub fn set_title(&self, title: &str) {
        *self.title.borrow_mut() = title.to_string();
    }

    pub fn mark_bar_dirty(&self) {
        self.bar_dirty.set(true);
        self.bar_stable.set(0);
        self.bar_seen.set(0.0);
        self.frame_key.set((0, 0));
    }
}

/// Width that leaves the same gap on both sides of the traffic lights.
/// `close_min_x` and `zoom_max_x` are in window coordinates.
pub fn centered_bar_width(close_min_x: f64, zoom_max_x: f64) -> f64 {
    zoom_max_x + close_min_x
}

/// Horizontal origin of a page centered in the area from the bar's right edge
/// to the window's right edge. `None` when there is no bar or the page does
/// not fit in that area.
pub fn content_page_x(stage_w: f64, page_w: f64, bar: f64) -> Option<f64> {
    if bar <= 1.0 || page_w <= 0.0 || stage_w <= bar + 1.0 || page_w > stage_w - bar + 0.5 {
        return None;
    }
    Some(bar + (stage_w - bar - page_w) / 2.0)
}

pub fn title_face(title: &str) -> &'static str {
    if title
        .chars()
        .any(|c| ('\u{4e00}'..='\u{9fff}').contains(&c))
    {
        "Songti SC"
    } else {
        "Iowan Old Style"
    }
}

pub fn fit(ratio_w: f64, ratio_h: f64, visible_w: f64, visible_h: f64) -> Frame {
    let ratio = if ratio_h <= 0.0 {
        1.0
    } else {
        (ratio_w / ratio_h).max(0.05)
    };
    let max_w = (visible_w * FIT).max(1.0);
    let max_h = (visible_h * FIT).max(1.0);
    let from_h = max_h - TOP;
    let from_w = (max_w - SIDE * 2.0) / ratio;
    let page_h = from_h.min(from_w).max(1.0);
    let page_w = page_h * ratio;
    Frame {
        window_w: page_w + SIDE * 2.0,
        window_h: page_h + TOP,
        page_w,
        page_h,
    }
}

pub fn content_box(stage_w: f64, stage_h: f64) -> (f64, f64, f64, f64) {
    let box_w = (stage_w - SIDE * 2.0).max(1.0);
    let box_h = (stage_h - TOP).max(1.0);
    (box_w, box_h, SIDE, TOP)
}

pub fn reading_page(
    ratio_w: f64,
    ratio_h: f64,
    full: bool,
    remembered: &Cell<(f64, f64)>,
) -> (f64, f64) {
    let Some(mtm) = MainThreadMarker::new() else {
        return (1.0, 1.0);
    };
    let Some(screen) = NSScreen::mainScreen(mtm) else {
        return (1.0, 1.0);
    };
    let visible = screen.visibleFrame().size;
    let saved = remembered.get();
    if !full && (saved.0 < 1.0 || visible.height <= saved.1 + 1.0) {
        remembered.set((visible.width, visible.height));
    }
    let saved = remembered.get();
    let (width, height) = if saved.0 > 1.0 {
        saved
    } else {
        (visible.width, visible.height)
    };
    let frame = fit(ratio_w, ratio_h, width, height);
    (frame.page_w, frame.page_h)
}

pub fn should_drag_margin(x: f64, y: f64, placed: Option<(i32, i32, i32, i32)>) -> bool {
    let Some((left, top, width, height)) = placed else {
        return false;
    };
    let right = f64::from(left + width);
    let bottom = f64::from(top + height);
    x < f64::from(left) || y < f64::from(top) || x > right || y > bottom
}

pub fn opening_size() -> (i32, i32) {
    if let Some((width, height)) = spike_window() {
        return (width.round() as i32, height.round() as i32);
    }
    let id = crate::settings::load(&crate::settings::path());
    let item = crate::reading::trim(&id);
    let Some(mtm) = MainThreadMarker::new() else {
        return (700, 1040);
    };
    let Some(screen) = NSScreen::mainScreen(mtm) else {
        return (700, 1040);
    };
    let visible = screen.visibleFrame();
    let frame = fit(
        f64::from(item.width_mm),
        f64::from(item.height_mm),
        visible.size.width,
        visible.size.height,
    );
    (frame.window_w.round() as i32, frame.window_h.round() as i32)
}

struct Release<'a>(&'a Cell<bool>);

impl Drop for Release<'_> {
    fn drop(&mut self) {
        self.0.set(false);
    }
}

pub fn sync(window: &NSWindow, rt: &Runtime, page: PageBox) {
    if rt.entering.replace(true) {
        return;
    }
    let _release = Release(&rt.entering);
    rt.page.set(page);
    style_window(window, rt);
    note_immersive(rt);
    settle_bar(window, rt);
    let fullscreen = window.styleMask().contains(NSWindowStyleMask::FullScreen);
    if !fullscreen && !rt.laying_out.get() {
        if let Some((width, height)) = pinned_frame() {
            if let Some(screen) = window
                .screen()
                .or_else(|| MainThreadMarker::new().and_then(NSScreen::mainScreen))
            {
                let frame = Frame {
                    window_w: width,
                    window_h: height,
                    page_w: page.w,
                    page_h: page.h,
                };
                apply_frame(window, &screen, &frame, false, !rt.shown.get());
                rt.shown.set(true);
                rt.settled.set(true);
            }
        }
    }
    note_fullscreen_hook(window, rt);
    if fullscreen {
        paint_lights(window, 1.0);
        rt.lights.set(1.0);
        rt.full.set(true);
        let shown = hover_target(window);
        place_contents(window, rt, shown);
        place_title(window, rt, shown);
        write_json(window, rt, true);
        return;
    }
    if rt.full.replace(false) {
        rt.settled.set(false);
        rt.stable.set(0);
        rt.attempts.set(0);
    }
    let key = (page.ratio_w.round() as i32, page.ratio_h.round() as i32);
    if key.0 > 0 && key.1 > 0 && rt.ratio_key.get() != key {
        rt.ratio_key.set(key);
        rt.settled.set(false);
        rt.stable.set(0);
        rt.attempts.set(0);
    }
    if pinned_frame().is_none() && !rt.settled.get() && page.ratio_w > 0.0 && page.ratio_h > 0.0 {
        if let Some(screen) = window
            .screen()
            .or_else(|| MainThreadMarker::new().and_then(NSScreen::mainScreen))
        {
            let visible = screen.visibleFrame();
            let frame = if let Some((width, height)) = spike_window() {
                Frame {
                    window_w: width,
                    window_h: height,
                    page_w: (width - SIDE * 2.0).max(1.0),
                    page_h: (height - TOP).max(1.0),
                }
            } else {
                fit(
                    page.ratio_w,
                    page.ratio_h,
                    visible.size.width,
                    visible.size.height,
                )
            };
            let place = !rt.shown.get();
            let animate = rt.shown.get();
            apply_frame(window, &screen, &frame, animate, place);
            rt.shown.set(true);
            let got = window.frame().size;
            let close = (got.width - frame.window_w).abs() < 3.0
                && (got.height - frame.window_h).abs() < 3.0;
            let tries = rt.attempts.get().saturating_add(1);
            rt.attempts.set(tries);
            if close {
                let count = rt.stable.get().saturating_add(1);
                rt.stable.set(count);
                if count >= 3 || tries >= 12 {
                    rt.settled.set(true);
                }
            } else if tries >= 12 {
                rt.settled.set(true);
            } else {
                rt.stable.set(0);
            }
        }
    }
    // The installed app keeps the traffic lights and the toolbar on screen.
    // The dev spike still fades them until the pointer is at the top.
    let next = if flag("SPIKE_LIGHTS") || rt.product.get() {
        1.0
    } else {
        approach(rt.lights.get(), hover_target(window))
    };
    rt.lights.set(next);
    paint_lights(window, next);
    place_contents(window, rt, next);
    place_title(window, rt, next);
    if rt.settled.get() && !rt.fullscreen_sent.get() && flag("SPIKE_FULLSCREEN") {
        window.toggleFullScreen(None);
        rt.fullscreen_sent.set(true);
    }
    write_json(window, rt, false);
}

pub fn paint_paper_layer(view: &AnyObject) {
    unsafe {
        let _: () = msg_send![view, setWantsLayer: true];
        let layer: Retained<AnyObject> = msg_send![view, layer];
        let (red, green, blue) = crate::reading::paper_rgb();
        let color = objc2_core_graphics::CGColor::new_srgb(red, green, blue, 1.0);
        let _: () = msg_send![&layer, setBackgroundColor: &*color];
    }
}

fn style_window(window: &NSWindow, rt: &Runtime) {
    if rt.styled.replace(true) {
        return;
    }
    apply_appearance(window);
    window.setTitlebarSeparatorStyle(NSTitlebarSeparatorStyle::None);
    window.setMovableByWindowBackground(true);
    window.setCollectionBehavior(
        window.collectionBehavior() | NSWindowCollectionBehavior::FullScreenPrimary,
    );
    window.makeKeyAndOrderFront(None);
    let mtm = MainThreadMarker::from(&*window);
    let app = objc2_app_kit::NSApplication::sharedApplication(mtm);
    unsafe {
        let _: () = msg_send![&*app, activateIgnoringOtherApps: true];
    }
    rt.mark_bar_dirty();
}

fn button_window_frame(button: &NSButton) -> NSRect {
    let view: &NSView = unsafe { &*std::ptr::from_ref(button).cast::<NSView>() };
    let frame = view.frame();
    match unsafe { view.superview() } {
        Some(parent) => parent.convertRect_toView(frame, None),
        None => frame,
    }
}

fn edge_box(rect: NSRect) -> (f64, f64, f64, f64) {
    (
        rect.origin.x,
        rect.origin.x + rect.size.width,
        rect.origin.y,
        rect.origin.y + rect.size.height,
    )
}

pub fn measure_traffic_bar(window: &NSWindow) -> Option<TrafficLights> {
    let close = window.standardWindowButton(NSWindowButton::CloseButton)?;
    let mini = window.standardWindowButton(NSWindowButton::MiniaturizeButton)?;
    let zoom = window.standardWindowButton(NSWindowButton::ZoomButton)?;
    let (close_min_x, close_max_x, close_min_y, close_max_y) =
        edge_box(button_window_frame(&close));
    let (mini_min_x, mini_max_x, mini_min_y, mini_max_y) = edge_box(button_window_frame(&mini));
    let (zoom_min_x, zoom_max_x, zoom_min_y, zoom_max_y) = edge_box(button_window_frame(&zoom));
    if !close_min_x.is_finite() || !zoom_max_x.is_finite() {
        return None;
    }
    if close_min_x < 0.0 || zoom_max_x <= close_min_x {
        return None;
    }
    let width = centered_bar_width(close_min_x, zoom_max_x);
    if !(8.0..=400.0).contains(&width) {
        return None;
    }
    Some(TrafficLights {
        close_min_x,
        close_max_x,
        close_min_y,
        close_max_y,
        mini_min_x,
        mini_max_x,
        mini_min_y,
        mini_max_y,
        zoom_min_x,
        zoom_max_x,
        zoom_min_y,
        zoom_max_y,
        width,
    })
}

fn toolbar_style(window: &NSWindow) -> isize {
    let Some(toolbar) = window.toolbar() else {
        return -1;
    };
    let sel = nsh_platform_macos::objc2::runtime::Sel::register(c"toolbarStyle");
    let responds: bool = unsafe { msg_send![&*toolbar, respondsToSelector: sel] };
    if !responds {
        return -1;
    }
    unsafe { msg_send![&*toolbar, toolbarStyle] }
}

fn note_bar_context(window: &NSWindow, rt: &Runtime) {
    let style = toolbar_style(window);
    let mask = window.styleMask().bits();
    let unseen = rt.toolbar_style.get() == -2;
    if !unseen && (rt.toolbar_style.get() != style || rt.style_mask.get() != mask) {
        rt.mark_bar_dirty();
    }
    rt.toolbar_style.set(style);
    rt.style_mask.set(mask);
}

fn settle_bar(window: &NSWindow, rt: &Runtime) {
    note_bar_context(window, rt);
    if !rt.bar_dirty.get() {
        return;
    }
    if window.inLiveResize() {
        rt.bar_seen.set(0.0);
        rt.bar_stable.set(0);
        rt.frame_key.set((0, 0));
        return;
    }
    let Some(measured) = measure_traffic_bar(window) else {
        return;
    };
    let frame = window.frame().size;
    let frame_key = (frame.width.round() as i32, frame.height.round() as i32);
    let width_ok = rt.bar_seen.get() > 1.0 && (rt.bar_seen.get() - measured.width).abs() <= 0.5;
    let frame_ok = rt.frame_key.get() == frame_key && frame_key != (0, 0);
    if width_ok && frame_ok {
        let count = rt.bar_stable.get().saturating_add(1);
        rt.bar_stable.set(count);
        if count >= 1 {
            rt.bar_dirty.set(false);
            rt.bar_gen.set(rt.bar_gen.get().saturating_add(1));
        }
    } else {
        rt.bar_stable.set(0);
    }
    rt.bar_seen.set(measured.width);
    rt.frame_key.set(frame_key);
    rt.bar_w.set(measured.width);
    *rt.traffic.borrow_mut() = Some(measured);
}

fn file_flag(path: &str) -> Option<bool> {
    let raw = std::fs::read_to_string(path).ok()?;
    let text = raw.trim();
    if text.is_empty() {
        return None;
    }
    Some(text == "1")
}

fn note_immersive(rt: &Runtime) {
    let Some(want) = file_flag("/tmp/superreader-spike/bar-immersive") else {
        return;
    };
    if rt.immersive.get() != want {
        rt.immersive.set(want);
        rt.mark_bar_dirty();
    }
}

fn note_fullscreen_hook(window: &NSWindow, rt: &Runtime) {
    let Some(want) = file_flag("/tmp/superreader-spike/bar-fullscreen") else {
        return;
    };
    let is_full = window.styleMask().contains(NSWindowStyleMask::FullScreen);
    if want == is_full {
        rt.bar_fullscreen_sent.set(false);
        return;
    }
    if rt.bar_fullscreen_sent.get() {
        return;
    }
    window.toggleFullScreen(None);
    rt.bar_fullscreen_sent.set(true);
}

fn pinned_frame() -> Option<(f64, f64)> {
    let raw = std::fs::read_to_string("/tmp/superreader-spike/shell-frame").ok()?;
    let mut parts = raw.split_whitespace();
    let width: f64 = parts.next()?.parse().ok()?;
    let height: f64 = parts.next()?.parse().ok()?;
    if width > 50.0 && height > 50.0 {
        Some((width, height))
    } else {
        None
    }
}

fn apply_frame(window: &NSWindow, screen: &NSScreen, frame: &Frame, animate: bool, place: bool) {
    let visible = screen.visibleFrame();
    let width = frame.window_w;
    let height = frame.window_h;
    let current = window.frame();
    let size_ok =
        (current.size.width - width).abs() < 2.0 && (current.size.height - height).abs() < 2.0;
    if size_ok && !place {
        return;
    }
    let x = if place {
        visible.origin.x + (visible.size.width - width) / 2.0
    } else {
        current.origin.x + (current.size.width - width) / 2.0
    };
    let y = if place {
        visible.origin.y + (visible.size.height - height) / 2.0
    } else {
        current.origin.y + current.size.height - height
    };
    window.setFrame_display_animate(
        NSRect::new(
            NSPoint::new(x.max(visible.origin.x), y.max(visible.origin.y)),
            NSSize::new(width, height),
        ),
        true,
        animate,
    );
}

fn hover_target(window: &NSWindow) -> f64 {
    if near_top(window) {
        1.0
    } else {
        0.0
    }
}

fn near_top(window: &NSWindow) -> bool {
    let mouse = objc2_app_kit::NSEvent::mouseLocation();
    let frame = window.frame();
    let top = frame.origin.y + frame.size.height;
    mouse.x >= frame.origin.x
        && mouse.x <= frame.origin.x + frame.size.width
        && mouse.y <= top
        && mouse.y >= top - 56.0
}

fn approach(current: f64, target: f64) -> f64 {
    let delta = (target - current).clamp(-0.25, 0.25);
    (current + delta).clamp(0.0, 1.0)
}

fn paint_lights(window: &NSWindow, alpha: f64) {
    for button in [
        NSWindowButton::CloseButton,
        NSWindowButton::MiniaturizeButton,
        NSWindowButton::ZoomButton,
    ] {
        let Some(button) = window.standardWindowButton(button) else {
            continue;
        };
        // Fade with alpha while the lights are showing. Hide them only once
        // they are gone, so a capture does not keep the colored discs, and
        // hide them after the contents item has already taken its slot.
        unsafe {
            if alpha < 0.02 {
                let _: () = msg_send![&*button, setAlphaValue: 0.0_f64];
                let _: () = msg_send![&*button, setHidden: true];
                let _: () = msg_send![&*button, setEnabled: false];
            } else {
                let _: () = msg_send![&*button, setHidden: false];
                let _: () = msg_send![&*button, setAlphaValue: alpha];
                let _: () = msg_send![&*button, setEnabled: true];
            }
        }
    }
}

fn place_title(window: &NSWindow, rt: &Runtime, alpha: f64) {
    let shown = rt.title.borrow().clone();
    // The installed app keeps the product name on the window. The dev spike
    // still puts the book title there.
    if !shown.is_empty() && !rt.product.get() {
        window.setTitle(&NSString::from_str(&shown));
    }
    let Some(field) = rt.field.borrow().clone() else {
        return;
    };
    let face = NSString::from_str(title_face(&shown));
    if let Some(font) = NSFont::fontWithName_size(&face, 13.0) {
        unsafe {
            let _: () = msg_send![&*field, setFont: &*font];
        }
    }
    unsafe {
        let value = NSString::from_str(&shown);
        let _: () = msg_send![&*field, setStringValue: &*value];
        let _: () = msg_send![&*field, setAlignment: objc2_app_kit::NSTextAlignment(1)];
        let _: () = msg_send![&*field, setAlphaValue: alpha];
        let _: () = msg_send![&*field, setHidden: alpha < 0.02];
    }
}

fn apply_appearance(window: &NSWindow) {
    // AppOptions pins the window to aqua. This overrides that for the icon shots.
    let Ok(mode) = std::env::var("SPIKE_APPEARANCE") else {
        return;
    };
    let name: &NSString = if mode == "dark" {
        unsafe { NSAppearanceNameDarkAqua }
    } else {
        unsafe { NSAppearanceNameAqua }
    };
    let Some(appearance) = NSAppearance::appearanceNamed(name) else {
        return;
    };
    window.setAppearance(Some(&appearance));
}

const TITLE_ID: &str = "books.title";
const CONTENTS_ID: &str = "books.contents";
const TITLE_W: f64 = 120.0;

define_class!(
    #[unsafe(super(NSTextField))]
    #[thread_kind = MainThreadOnly]
    #[ivars = f64]
    struct TitleField;

    unsafe impl NSObjectProtocol for TitleField {}

    impl TitleField {
        #[unsafe(method(intrinsicContentSize))]
        fn intrinsic_content_size(&self) -> NSSize {
            let full: NSSize = unsafe { msg_send![super(self), intrinsicContentSize] };
            let width = if full.width.is_finite() {
                full.width.min(TITLE_W)
            } else {
                TITLE_W
            };
            NSSize::new(width, *self.ivars())
        }
    }
);

fn title_label(mtm: MainThreadMarker, row: f64) -> Retained<NSTextField> {
    let height = row.max(18.0);
    let field = mtm.alloc::<TitleField>().set_ivars(height);
    let field: Retained<TitleField> = unsafe {
        msg_send![
            super(field),
            initWithFrame: NSRect::new(NSPoint::new(0.0, 0.0), NSSize::new(TITLE_W, height))
        ]
    };
    let field = field.into_super();
    field.setBezeled(false);
    field.setBordered(false);
    field.setDrawsBackground(false);
    field.setEditable(false);
    field.setSelectable(false);
    field.setMaximumNumberOfLines(1);
    field.setLineBreakMode(NSLineBreakMode::ByTruncatingMiddle);
    // The toolbar measures this label from its text. A PDF title is wider
    // than the row, and that width hides the label and the right-hand icons
    // behind the overflow chevron.
    field.setPreferredMaxLayoutWidth(TITLE_W);
    field
}
pub const TOOLBAR_ORDER: [&str; 5] = [
    "books.contents",
    "books.bookmarks",
    "books.search",
    "books.add-bookmark",
    "books.notes",
];

fn place_contents(window: &NSWindow, rt: &Runtime, alpha: f64) {
    let Some(toolbar) = window.toolbar() else {
        return;
    };
    if std::env::var("SPIKE_ICON").ok().as_deref() == Some("0") {
        clear_contents(&toolbar);
        return;
    }
    // Inserting the item while the lights are faded puts it in the
    // traffic-light row, and it stays there after they return. Wait until
    // the buttons are shown, which is the placement that leaves 26 pt.
    if rt.contents_bar.borrow().is_none() && alpha < 0.9 {
        return;
    }
    ensure_contents(window, rt, &toolbar);
    for item in toolbar.items() {
        let Some(view) = item.view() else {
            continue;
        };
        view.setHidden(alpha < 0.02);
        view.setAlphaValue(alpha);
    }
    if let Some(view) = item_view(&toolbar, CONTENTS_ID) {
        record_gap(window, rt, &view);
    }
}

fn clear_contents(toolbar: &NSToolbar) {
    if item_view(toolbar, CONTENTS_ID).is_none() {
        return;
    }
    toolbar.setItemIdentifiers(&NSArray::from_slice(&[] as &[&NSString]));
}

fn ensure_contents(window: &NSWindow, rt: &Runtime, toolbar: &NSToolbar) {
    if rt.contents_bar.borrow().is_some() {
        return;
    }
    let contents = if std::env::var("SPIKE_ICON").ok().as_deref() == Some("sf") {
        reference_symbol()
    } else {
        icons::contents_image()
    };
    let Some(contents) = contents else {
        return;
    };
    let Some(bookmark) = icons::bookmark_image() else {
        return;
    };
    let Some(search) = icons::search_image() else {
        return;
    };
    let Some(add) = icons::add_bookmark_image() else {
        return;
    };
    let Some(notes) = icons::notes_image() else {
        return;
    };
    let mtm = MainThreadMarker::from(&*window);
    let row = row_height(window);
    let field = title_label(mtm, row);
    field.setTextColor(Some(&NSColor::colorWithRed_green_blue_alpha(
        0.36, 0.33, 0.29, 1.0,
    )));
    *rt.field.borrow_mut() = Some(field.clone());
    let bar = mtm.alloc::<ContentsBar>().set_ivars(ContentsIvars {
        contents,
        bookmark,
        search,
        add,
        notes,
        title: field,
        row,
    });
    let bar: Retained<ContentsBar> = unsafe { msg_send![super(bar), init] };
    toolbar.setAutosavesConfiguration(false);
    toolbar.setDisplayMode(NSToolbarDisplayMode::IconOnly);
    toolbar.setDelegate(Some(ProtocolObject::from_ref(&*bar)));
    toolbar.setItemIdentifiers(&bar_ids());
    *rt.contents_bar.borrow_mut() = Some(bar);
}

fn item_view(toolbar: &NSToolbar, identifier: &str) -> Option<Retained<NSView>> {
    toolbar.items().iter().find_map(|item| {
        if item.itemIdentifier().to_string() == identifier {
            item.view()
        } else {
            None
        }
    })
}

fn bar_ids() -> Retained<NSArray<NSString>> {
    let contents = NSString::from_str(CONTENTS_ID);
    let bookmarks = NSString::from_str("books.bookmarks");
    let search = NSString::from_str("books.search");
    let title = NSString::from_str(TITLE_ID);
    let add = NSString::from_str("books.add-bookmark");
    let notes = NSString::from_str("books.notes");
    let space = unsafe { &*NSToolbarFlexibleSpaceItemIdentifier };
    NSArray::from_slice(&[
        &*contents,
        &*bookmarks,
        &*search,
        space,
        &*title,
        space,
        &*add,
        &*notes,
    ])
}

fn row_height(window: &NSWindow) -> f64 {
    let Some(button) = window.standardWindowButton(NSWindowButton::CloseButton) else {
        return 18.0;
    };
    let host: Option<Retained<NSView>> = unsafe { msg_send![&*button, superview] };
    let Some(host) = host else {
        return 18.0;
    };
    let height = host.frame().size.height;
    if height.is_finite() && height > 18.0 {
        height
    } else {
        18.0
    }
}

fn reference_symbol() -> Option<Retained<NSImage>> {
    let base = NSImage::imageWithSystemSymbolName_accessibilityDescription(
        &NSString::from_str("list.bullet"),
        None,
    )?;
    let weight = unsafe { NSFontWeightRegular };
    let config = NSImageSymbolConfiguration::configurationWithPointSize_weight(16.0, weight);
    let image = base.imageWithSymbolConfiguration(&config).unwrap_or(base);
    image.setTemplate(true);
    Some(image)
}

fn record_gap(window: &NSWindow, rt: &Runtime, view: &NSView) {
    let Some(zoom) = window.standardWindowButton(NSWindowButton::ZoomButton) else {
        return;
    };
    let zoom_bounds: NSRect = unsafe { msg_send![&*zoom, bounds] };
    let zoom_win: NSRect =
        unsafe { msg_send![&*zoom, convertRect: zoom_bounds, toView: std::ptr::null::<NSView>()] };
    let item_win = view.convertRect_toView(view.bounds(), None);
    let gap = item_win.origin.x - (zoom_win.origin.x + zoom_win.size.width);
    let zoom_mid = zoom_win.origin.y + zoom_win.size.height / 2.0;
    let item_mid = item_win.origin.y + item_win.size.height / 2.0;
    let dy = item_mid - zoom_mid;
    let first = rt.icon_gap.get().is_nan();
    rt.icon_gap.set(gap);
    rt.icon_dy.set(dy);
    if first {
        eprintln!("contents toolbar gap {gap:.1} pt, vertical delta {dy:.1} pt");
    }
}

struct ContentsIvars {
    contents: Retained<NSImage>,
    bookmark: Retained<NSImage>,
    search: Retained<NSImage>,
    add: Retained<NSImage>,
    notes: Retained<NSImage>,
    title: Retained<NSTextField>,
    row: f64,
}

define_class!(
    #[unsafe(super(objc2::runtime::NSObject))]
    #[thread_kind = MainThreadOnly]
    #[ivars = ContentsIvars]
    struct ContentsBar;

    unsafe impl NSObjectProtocol for ContentsBar {}

    unsafe impl NSToolbarDelegate for ContentsBar {}

    impl ContentsBar {
        #[unsafe(method_id(toolbar:itemForItemIdentifier:willBeInsertedIntoToolbar:))]
        fn item_for(
            &self,
            _toolbar: &NSToolbar,
            identifier: &NSString,
            _insert: bool,
        ) -> Option<Retained<NSToolbarItem>> {
            let mtm = MainThreadMarker::from(self);
            let id = identifier.to_string();
            if id == TITLE_ID {
                let item =
                    NSToolbarItem::initWithItemIdentifier(NSToolbarItem::alloc(mtm), identifier);
                item.setBordered(false);
                let field = self.ivars().title.clone();
                let hosted = field.into_super().into_super();
                item.setView(Some(&hosted));
                Some(item)
            } else {
                let image = match id.as_str() {
                    CONTENTS_ID => Some(&self.ivars().contents),
                    "books.bookmarks" => Some(&self.ivars().bookmark),
                    "books.search" => Some(&self.ivars().search),
                    "books.add-bookmark" => Some(&self.ivars().add),
                    "books.notes" => Some(&self.ivars().notes),
                    _ => None,
                };
                image.map(|image| icon_item(identifier, image, self.ivars().row, mtm))
            }
        }

        #[unsafe(method_id(toolbarDefaultItemIdentifiers:))]
        fn default_ids(&self, _toolbar: &NSToolbar) -> Retained<NSArray<NSString>> {
            bar_ids()
        }

        #[unsafe(method_id(toolbarAllowedItemIdentifiers:))]
        fn allowed_ids(&self, _toolbar: &NSToolbar) -> Retained<NSArray<NSString>> {
            bar_ids()
        }
    }
);

fn icon_item(
    identifier: &NSString,
    image: &NSImage,
    row: f64,
    mtm: MainThreadMarker,
) -> Retained<NSToolbarItem> {
    let item = NSToolbarItem::initWithItemIdentifier(NSToolbarItem::alloc(mtm), identifier);
    item.setBordered(false);
    // A template image tints white in this toolbar, so draw the black raster.
    image.setTemplate(false);
    let view = NSImageView::imageViewWithImage(image, mtm);
    view.setImageAlignment(NSImageAlignment::AlignCenter);
    view.setImageScaling(NSImageScaling::ScaleNone);
    let height = row.max(18.0);
    view.setFrame(NSRect::new(
        NSPoint::new(0.0, 0.0),
        NSSize::new(image.size().width.max(18.0), height),
    ));
    let hosted = view.into_super().into_super();
    item.setView(Some(&hosted));
    item
}

fn write_json(window: &NSWindow, rt: &Runtime, fullscreen: bool) {
    let page = rt.page.get();
    let frame = window.frame();
    let lights = rt.traffic.borrow().unwrap_or(TrafficLights {
        close_min_x: 0.0,
        close_max_x: 0.0,
        close_min_y: 0.0,
        close_max_y: 0.0,
        mini_min_x: 0.0,
        mini_max_x: 0.0,
        mini_min_y: 0.0,
        mini_max_y: 0.0,
        zoom_min_x: 0.0,
        zoom_max_x: 0.0,
        zoom_min_y: 0.0,
        zoom_max_y: 0.0,
        width: 0.0,
    });
    let mut body = serde_json::json!({
        "window_w": frame.size.width,
        "window_h": frame.size.height,
        "page_x": page.x,
        "page_y": page.y,
        "page_w": page.w,
        "page_h": page.h,
        "stage_w": page.stage_w,
        "stage_h": page.stage_h,
        "top": TOP,
        "side": SIDE,
        "fullscreen": fullscreen,
        "lights": rt.lights.get(),
        "ratio_w": page.ratio_w,
        "ratio_h": page.ratio_h,
        "title": rt.title.borrow().clone(),
        "appearance": window.effectiveAppearance().name().to_string(),
        "items": TOOLBAR_ORDER,
        "icon_gap": finite_json(rt.icon_gap.get()),
        "icon_dy": finite_json(rt.icon_dy.get()),
        "icon_frame": icon_frame(window),
        "close": button_frame(window, NSWindowButton::CloseButton),
        "miniaturize": button_frame(window, NSWindowButton::MiniaturizeButton),
        "zoom": button_frame(window, NSWindowButton::ZoomButton),
        "pid": std::process::id(),
    });
    let bar = serde_json::json!({
        "bar_x": 0.0,
        "bar_w": rt.bar_w.get(),
        "bar_h": page.stage_h,
        "bar_dirty": rt.bar_dirty.get(),
        "bar_gen": rt.bar_gen.get(),
        "immersive": rt.immersive.get(),
        "light_w": lights.width,
        "close_min_x": lights.close_min_x,
        "close_max_x": lights.close_max_x,
        "close_min_y": lights.close_min_y,
        "close_max_y": lights.close_max_y,
        "mini_min_x": lights.mini_min_x,
        "mini_max_x": lights.mini_max_x,
        "mini_min_y": lights.mini_min_y,
        "mini_max_y": lights.mini_max_y,
        "zoom_min_x": lights.zoom_min_x,
        "zoom_max_x": lights.zoom_max_x,
        "zoom_min_y": lights.zoom_min_y,
        "zoom_max_y": lights.zoom_max_y,
    });
    if let (Some(map), Some(extra)) = (body.as_object_mut(), bar.as_object()) {
        for (key, value) in extra {
            map.insert(key.clone(), value.clone());
        }
    }
    let text = serde_json::to_string_pretty(&body).unwrap_or_default();
    let dir = "/tmp/superreader-spike";
    let _ = std::fs::create_dir_all(dir);
    let pid = std::process::id();
    for path in [
        format!("{dir}/chrome.json"),
        format!("{dir}/chrome-{pid}.json"),
    ] {
        let tmp = format!("{path}.{pid}.tmp");
        if std::fs::write(&tmp, &text).is_ok() {
            let _ = std::fs::rename(&tmp, path);
        }
    }
}

fn button_frame(window: &NSWindow, kind: NSWindowButton) -> serde_json::Value {
    let Some(button) = window.standardWindowButton(kind) else {
        return serde_json::Value::Null;
    };
    view_frame(&button)
}

fn icon_frame(window: &NSWindow) -> serde_json::Value {
    let Some(toolbar) = window.toolbar() else {
        return serde_json::Value::Null;
    };
    let Some(view) = item_view(&toolbar, CONTENTS_ID) else {
        return serde_json::Value::Null;
    };
    view_frame(&view)
}

fn view_frame(view: &NSView) -> serde_json::Value {
    let bounds = view.bounds();
    let rect: NSRect =
        unsafe { msg_send![view, convertRect: bounds, toView: std::ptr::null::<NSView>()] };
    serde_json::json!({
        "x": rect.origin.x,
        "y": rect.origin.y,
        "w": rect.size.width,
        "h": rect.size.height,
    })
}

fn flag(name: &str) -> bool {
    std::env::var(name).ok().as_deref() == Some("1")
}

fn finite_json(value: f64) -> serde_json::Value {
    if value.is_finite() {
        serde_json::json!(value)
    } else {
        serde_json::Value::Null
    }
}

fn spike_window() -> Option<(f64, f64)> {
    parse_spike_window(&std::env::var("SPIKE_WINDOW").ok()?)
}

fn parse_spike_window(raw: &str) -> Option<(f64, f64)> {
    let (width, height) = raw.split_once('x')?;
    let width: f64 = width.trim().parse().ok()?;
    let height: f64 = height.trim().parse().ok()?;
    if !(200.0..=4000.0).contains(&width) || !(200.0..=4000.0).contains(&height) {
        return None;
    }
    Some((width, height))
}

#[cfg(test)]
mod tests {
    use super::*;

    fn close(got: f64, want: f64) {
        assert!((got - want).abs() < 0.02, "{got} vs {want}");
    }

    #[test]
    fn spike_window_parses_a_frame_and_rejects_a_partial_one() {
        assert_eq!(parse_spike_window("640x960"), Some((640.0, 960.0)));
        assert_eq!(parse_spike_window(" 700 x 800 "), Some((700.0, 800.0)));
        assert_eq!(parse_spike_window("640"), None);
        assert_eq!(parse_spike_window("10x10"), None);
    }

    #[test]
    fn paperback_on_a_1000_by_1200_frame_is_the_page_plus_the_top_mat() {
        let frame = fit(129.0, 198.0, 1000.0, 1200.0);
        close(frame.page_h, 1056.0 - TOP);
        close(frame.page_w, frame.page_h * 129.0 / 198.0);
        close(frame.window_h, frame.page_h + TOP);
        close(frame.window_w, frame.page_w + SIDE * 2.0);
        assert!(frame.window_w <= 1000.0 * FIT + 0.1);
        assert!(frame.window_h <= 1200.0 * FIT + 0.1);
    }

    #[test]
    fn the_page_starts_below_the_traffic_lights() {
        let (_, _, _, top) = content_box(800.0, 1000.0);
        assert!(top >= 44.0);
    }

    #[test]
    fn the_page_is_centered_in_the_area_beside_the_bar() {
        let x = content_page_x(1100.0, 400.0, 98.0).unwrap();
        let page_mid = x + 200.0;
        let area_mid = 98.0 + (1100.0 - 98.0) / 2.0;
        close(page_mid, area_mid);
        assert!(content_page_x(1100.0, 400.0, 0.0).is_none());
        assert!(content_page_x(200.0, 180.0, 98.0).is_none());
    }

    #[test]
    fn the_bar_width_centers_the_traffic_lights() {
        let close_min_x = 12.5;
        let zoom_max_x = 70.0;
        let width = centered_bar_width(close_min_x, zoom_max_x);
        close(width, 82.5);
        close(close_min_x, width - zoom_max_x);
    }

    #[test]
    fn a_cjk_title_uses_songti() {
        assert_eq!(title_face("朝花夕拾"), "Songti SC");
        assert_eq!(title_face("Pride and Prejudice"), "Iowan Old Style");
    }
}
